//! HTTP serving boundary for Макошь.
//!
//! Route construction is supplied by composition. This crate owns only Axum
//! serving and graceful termination, so it stays independent of SQL, vaults,
//! provider implementations and domain stores.

use std::io;

use axum::Router;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub async fn serve(
    listener: TcpListener,
    router: Router,
    termination: CancellationToken,
) -> Result<(), io::Error> {
    axum::serve(listener, router)
        .with_graceful_shutdown(async move { termination.cancelled().await })
        .await
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::Router;
    use tokio::net::TcpListener;
    use tokio_util::sync::CancellationToken;

    use super::serve;

    #[tokio::test]
    async fn cancellation_stops_the_http_server() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let termination = CancellationToken::new();
        let server = tokio::spawn(serve(listener, Router::new(), termination.clone()));

        tokio::time::sleep(Duration::from_millis(5)).await;
        termination.cancel();

        server
            .await
            .expect("server task join")
            .expect("graceful server shutdown");
    }
}
