//! Bounded single-writer actor for all online Control Store operations.

use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::time::Duration;

use rusqlite::Connection;

use crate::StoreError;

pub(crate) const CONTROL_STORE_QUEUE_CAPACITY: usize = 64;
const CONTROL_STORE_DEADLINE: Duration = Duration::from_secs(2);
const CONTROL_STORE_MAINTENANCE_DEADLINE: Duration = Duration::from_secs(30);

type Task = Box<dyn FnOnce(&mut Connection) + Send + 'static>;

#[derive(Clone)]
pub struct ControlStoreHandle {
    pub(crate) sender: SyncSender<Task>,
}

impl ControlStoreHandle {
    pub(crate) fn spawn(connection: Connection) -> Result<Self, StoreError> {
        let (sender, receiver) = mpsc::sync_channel(CONTROL_STORE_QUEUE_CAPACITY);
        std::thread::Builder::new()
            .name("makosh-control-store".to_owned())
            .spawn(move || run_actor(connection, receiver))
            .map_err(StoreError::Io)?;
        Ok(Self { sender })
    }

    pub(crate) fn call<T, F>(&self, operation: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    {
        self.call_with_deadline(CONTROL_STORE_DEADLINE, operation)
    }

    pub(crate) fn maintenance<T, F>(&self, operation: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    {
        self.call_with_deadline(CONTROL_STORE_MAINTENANCE_DEADLINE, operation)
    }

    fn call_with_deadline<T, F>(&self, deadline: Duration, operation: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    {
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let task = Box::new(move |connection: &mut Connection| {
            let _ = response_sender.send(operation(connection));
        });
        match self.sender.try_send(task) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(StoreError::QueueFull),
            Err(TrySendError::Disconnected(_)) => return Err(StoreError::ActorStopped),
        }
        response_receiver.recv_timeout(deadline).map_err(|error| {
            if matches!(error, mpsc::RecvTimeoutError::Timeout) {
                StoreError::DeadlineExceeded
            } else {
                StoreError::ActorStopped
            }
        })?
    }
}

fn run_actor(mut connection: Connection, receiver: Receiver<Task>) {
    while let Ok(task) = receiver.recv() {
        task(&mut connection);
    }
}
