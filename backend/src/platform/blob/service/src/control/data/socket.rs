//! Owns the owner-private direct Blob socket lifecycle.

use std::fs;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::Path;

pub(crate) struct PrivateBlobDataListener(UnixListener);

impl PrivateBlobDataListener {
    pub(crate) fn bind(path: &Path) -> Result<Self, ()> {
        remove_owned_stale_socket(path)?;
        let listener = UnixListener::bind(path).map_err(|_| ())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|_| ())?;
        listener.set_nonblocking(true).map_err(|_| ())?;
        Ok(Self(listener))
    }

    pub(crate) fn accept(&self) -> Result<Option<std::os::unix::net::UnixStream>, ()> {
        match self.0.accept() {
            Ok((stream, _)) => {
                // Listener readiness must never leak into the accepted data
                // channel. On platforms that inherit `O_NONBLOCK`, a large
                // framed write can otherwise be truncated at the first
                // `WouldBlock` and surface as a client-side broken pipe.
                stream.set_nonblocking(false).map_err(|_| ())?;
                Ok(Some(stream))
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(_) => Err(()),
        }
    }
}

fn remove_owned_stale_socket(path: &Path) -> Result<(), ()> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.file_type().is_socket()
                && metadata.uid() == unsafe { libc::geteuid() }
                && metadata.mode() & 0o077 == 0 =>
        {
            fs::remove_file(path).map_err(|_| ())
        }
        Ok(_) => Err(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    use super::PrivateBlobDataListener;

    static NEXT_SOCKET: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn accepted_data_stream_waits_for_the_complete_frame() {
        let root = std::env::temp_dir().join(format!(
            "makosh-blob-data-socket-{}-{}",
            std::process::id(),
            NEXT_SOCKET.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).expect("create private Blob socket test directory");
        let path = root.join("blob.sock");
        let listener = PrivateBlobDataListener::bind(&path).expect("bind Blob data listener");
        let client_path = path.clone();
        let writer = std::thread::spawn(move || {
            let mut stream = UnixStream::connect(client_path).expect("connect Blob data socket");
            stream.write_all(&[7]).expect("write first frame byte");
            std::thread::sleep(Duration::from_millis(50));
            stream
                .write_all(&vec![9; 128 * 1024 - 1])
                .expect("write remaining frame bytes");
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut stream = loop {
            if let Some(stream) = listener.accept().expect("accept Blob data stream") {
                break stream;
            }
            assert!(
                Instant::now() < deadline,
                "Blob data client did not connect"
            );
            std::thread::sleep(Duration::from_millis(1));
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set test read timeout");
        let mut frame = vec![0; 128 * 1024];
        stream
            .read_exact(&mut frame)
            .expect("accepted stream must remain blocking across partial writes");
        assert_eq!(frame[0], 7);
        assert!(frame[1..].iter().all(|byte| *byte == 9));
        writer.join().expect("join Blob data writer");
        drop(stream);
        drop(listener);
        std::fs::remove_file(&path).expect("remove Blob test socket");
        std::fs::remove_dir(&root).expect("remove Blob socket test directory");
    }
}
