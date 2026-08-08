use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use hyper::{Method, Request, StatusCode};
use makosh_gateway_runtime::{
    BrowserBootstrapRouter, GatewayLoopbackListenerV1, GatewayLoopbackTlsListenerV1,
    GatewayTechnicalRouter, GatewayTlsListenerV1, GatewayTransportProfileV1, PairedRemoteProfileV1,
};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::time::timeout;
use tokio_rustls::{TlsAcceptor, TlsConnector};

use crate::platform::gateway::{
    BrowserGatewayConfigurationV1, required_browser_bootstrap_manifest,
};

#[test]
fn installed_browser_listener_requires_a_signed_bootstrap_artifact() {
    assert_eq!(
        required_browser_bootstrap_manifest(&[]).expect_err("missing artifact must fail"),
        "signed browser bootstrap artifact is required"
    );
}

#[test]
fn gateway_profiles_reject_remote_plaintext_and_http3_early_data() {
    let loopback = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let non_loopback = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10));

    assert_eq!(
        GatewayTransportProfileV1::LocalEmbedded
            .validate_bind(non_loopback, false)
            .expect_err("local profile must not expose a non-loopback listener"),
        "local Gateway listener must bind loopback only"
    );
    assert!(
        GatewayTransportProfileV1::LocalEmbedded
            .validate_bind(loopback, false)
            .is_ok()
    );
    assert_eq!(
        GatewayTransportProfileV1::PairedRemote(
            PairedRemoteProfileV1::new(true, false).expect("remote profile"),
        )
        .validate_bind(non_loopback, false)
        .expect_err("remote profile requires TLS"),
        "paired remote Gateway listener requires TLS"
    );
    assert_eq!(
        PairedRemoteProfileV1::new(true, true).expect_err("0-RTT must stay disabled"),
        "Gateway HTTP/3 early data is forbidden"
    );
}

#[test]
fn developer_gateway_configuration_requires_one_exact_private_lan_origin() {
    BrowserGatewayConfigurationV1::new_lan_development(
        "192.168.1.10:9443".parse().expect("private address"),
        "http://192.168.1.10:9443".to_owned(),
        "192.168.1.10".to_owned(),
    )
    .expect("exact private LAN configuration");

    assert!(
        BrowserGatewayConfigurationV1::new_lan_development(
            "0.0.0.0:9443".parse().expect("wildcard address"),
            "http://0.0.0.0:9443".to_owned(),
            "0.0.0.0".to_owned(),
        )
        .is_err()
    );
    assert!(
        BrowserGatewayConfigurationV1::new_lan_development(
            "192.168.1.10:9443".parse().expect("private address"),
            "https://makosh.sh-inc.ru".to_owned(),
            "makosh.sh-inc.ru".to_owned(),
        )
        .is_err()
    );
}

#[test]
fn loopback_development_proxy_configuration_is_literal_and_proof_bound() {
    let proof = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    BrowserGatewayConfigurationV1::new_loopback_development_proxy(
        "127.0.0.1:9444".parse().expect("loopback address"),
        "http://127.0.0.1:5173".to_owned(),
        "127.0.0.1".to_owned(),
        proof.to_owned(),
    )
    .expect("exact loopback development proxy");

    for (address, origin, rp_id, proof) in [
        ("0.0.0.0:9444", "http://127.0.0.1:5173", "127.0.0.1", proof),
        (
            "127.0.0.1:9444",
            "http://localhost:5173",
            "127.0.0.1",
            proof,
        ),
        (
            "127.0.0.1:9444",
            "http://127.0.0.1:5173",
            "localhost",
            proof,
        ),
        (
            "127.0.0.1:9444",
            "http://127.0.0.1:5173",
            "127.0.0.1",
            "bad",
        ),
    ] {
        assert!(
            BrowserGatewayConfigurationV1::new_loopback_development_proxy(
                address.parse().expect("socket address"),
                origin.to_owned(),
                rp_id.to_owned(),
                proof.to_owned(),
            )
            .is_err()
        );
    }
}

#[test]
fn gateway_technical_router_exposes_only_health_and_readiness() {
    let ready = GatewayTechnicalRouter::new(true);
    let unready = GatewayTechnicalRouter::new(false);
    let health = ready.route(&Method::GET, "/healthz");

    assert_eq!(health.status(), StatusCode::OK);
    assert!(
        health
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );
    assert_eq!(
        ready.route(&Method::GET, "/readyz").status(),
        StatusCode::OK
    );
    assert_eq!(
        unready.route(&Method::GET, "/readyz").status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        ready.route(&Method::POST, "/healthz").status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        ready.route(&Method::GET, "/owner/api").status(),
        StatusCode::NOT_FOUND
    );
}

#[test]
fn signed_browser_bootstrap_router_serves_only_the_root_document() {
    let router = BrowserBootstrapRouter::new(b"<!doctype html><title>makosh</title>".to_vec())
        .expect("bounded UTF-8 bootstrap");
    let root = router.route(&Method::GET, "/");
    assert_eq!(root.status(), StatusCode::OK);
    assert_eq!(
        root.headers().get("content-type"),
        Some(&"text/html; charset=utf-8".parse().unwrap())
    );
    assert_eq!(
        root.headers().get("cache-control"),
        Some(&"no-store".parse().unwrap())
    );
    assert!(root.headers().contains_key("content-security-policy"));
    assert_eq!(
        router.route(&Method::GET, "/assets/app.js").status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        router.route(&Method::POST, "/").status(),
        StatusCode::NOT_FOUND
    );
    assert!(BrowserBootstrapRouter::new(Vec::new()).is_err());
    assert!(BrowserBootstrapRouter::new(vec![0xff]).is_err());
}

#[test]
fn signed_browser_bootstrap_router_serves_only_exact_declared_assets() {
    let router = BrowserBootstrapRouter::new(b"<!doctype html><title>makosh</title>".to_vec())
        .expect("bounded UTF-8 bootstrap")
        .with_assets([(
            "/assets/app.js".to_owned(),
            b"console.log('makosh')".to_vec(),
        )])
        .expect("exact signed browser asset");
    let asset = router.route(&Method::GET, "/assets/app.js");
    assert_eq!(asset.status(), StatusCode::OK);
    assert_eq!(
        asset.headers().get("content-type"),
        Some(&"text/javascript; charset=utf-8".parse().unwrap())
    );
    assert_eq!(
        asset.headers().get("cache-control"),
        Some(&"public, max-age=31536000, immutable".parse().unwrap())
    );
    assert_eq!(
        router.route(&Method::GET, "/assets/missing.js").status(),
        StatusCode::NOT_FOUND
    );
    assert!(
        BrowserBootstrapRouter::new(b"ok".to_vec())
            .expect("bootstrap")
            .with_assets([("/assets/../secret.js".to_owned(), b"no".to_vec())])
            .is_err()
    );

    let webp = BrowserBootstrapRouter::new(b"ok".to_vec())
        .expect("bootstrap")
        .with_assets([("/assets/shell-backgrounds/network.webp".to_owned(), vec![1])])
        .expect("signed WebP asset")
        .route(&Method::GET, "/assets/shell-backgrounds/network.webp");
    assert_eq!(webp.status(), StatusCode::OK);
    assert_eq!(
        webp.headers().get("content-type"),
        Some(&"image/webp".parse().unwrap())
    );
}

#[test]
fn loopback_listener_serves_the_local_technical_surface_and_rejects_lan_bind() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let rejected = runtime
        .block_on(GatewayLoopbackListenerV1::bind(
            "192.0.2.10:0".parse().expect("non-loopback address"),
        ))
        .err()
        .expect("loopback listener must reject LAN bind");
    assert_eq!(rejected, "local Gateway listener must bind loopback only");
    let listener = runtime
        .block_on(GatewayLoopbackListenerV1::bind(
            "127.0.0.1:0".parse().expect("loopback address"),
        ))
        .expect("loopback listener");
    let address = listener.local_address().expect("local address");
    let server = runtime.spawn(async move {
        listener
            .serve_once(GatewayTechnicalRouter::new(true))
            .await
            .expect("local HTTP server")
    });

    runtime.block_on(async move {
        let mut stream = TcpStream::connect(address).await.expect("TCP connection");
        stream
            .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .expect("HTTP request");
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .expect("HTTP response");
        assert!(
            std::str::from_utf8(&response)
                .expect("HTTP response text")
                .starts_with("HTTP/1.1 200 OK\r\n")
        );
        server.await.expect("server task");
    });
}

#[test]
fn loopback_listener_serves_multiple_connections_until_its_owner_shuts_it_down() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let listener = runtime
        .block_on(GatewayLoopbackListenerV1::bind(
            "127.0.0.1:0".parse().expect("loopback address"),
        ))
        .expect("loopback listener");
    let address = listener.local_address().expect("local address");
    let (shutdown, receiver) = watch::channel(false);
    let server = runtime.spawn(async move {
        listener
            .serve_until_shutdown(GatewayTechnicalRouter::new(true), receiver)
            .await
            .expect("local HTTP server")
    });

    runtime.block_on(async move {
        for _ in 0..2 {
            let mut stream = TcpStream::connect(address).await.expect("TCP connection");
            stream
                .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .await
                .expect("HTTP request");
            let mut response = Vec::new();
            stream
                .read_to_end(&mut response)
                .await
                .expect("HTTP response");
            assert!(
                std::str::from_utf8(&response)
                    .expect("HTTP response text")
                    .starts_with("HTTP/1.1 200 OK\r\n")
            );
        }
        shutdown.send(true).expect("request shutdown");
        server.await.expect("server task");
    });
}

#[test]
fn loopback_listener_bounds_slow_connection_storm_and_reclaims_a_released_slot() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let listener = runtime
        .block_on(GatewayLoopbackListenerV1::bind(
            "127.0.0.1:0".parse().expect("loopback address"),
        ))
        .expect("loopback listener");
    let address = listener.local_address().expect("local address");
    let (shutdown, receiver) = watch::channel(false);
    let server = runtime.spawn(async move {
        listener
            .serve_until_shutdown(GatewayTechnicalRouter::new(true), receiver)
            .await
            .expect("local HTTP server")
    });

    runtime.block_on(async move {
        let mut stalled = Vec::new();
        for _ in 0..128 {
            stalled.push(TcpStream::connect(address).await.expect("slow TCP peer"));
        }
        assert_over_budget_peer_is_dropped(address).await;
        drop(stalled.pop().expect("one stalled peer"));
        assert_released_slot_serves_health(address).await;
        shutdown.send(true).expect("request shutdown");
        server.await.expect("server task");
    });
}

async fn assert_over_budget_peer_is_dropped(address: std::net::SocketAddr) {
    let mut rejected = TcpStream::connect(address)
        .await
        .expect("overflow TCP peer");
    rejected
        .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .expect("overflow request write");
    let mut rejected_response = [0_u8; 1];
    let rejected_result = timeout(
        Duration::from_millis(250),
        rejected.read(&mut rejected_response),
    )
    .await
    .expect("overflow peer must not retain an active Gateway task");
    assert!(
        match rejected_result {
            Ok(0) => true,
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => true,
            Ok(_) | Err(_) => false,
        },
        "connection over the fixed admission budget must be dropped"
    );
}

async fn assert_released_slot_serves_health(address: std::net::SocketAddr) {
    let response = timeout(Duration::from_secs(2), async {
        loop {
            let mut stream = TcpStream::connect(address)
                .await
                .expect("replacement TCP peer");
            stream
                .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .await
                .expect("replacement request write");
            let mut response = Vec::new();
            match stream.read_to_end(&mut response).await {
                Ok(_) if response.starts_with(b"HTTP/1.1 200 OK\r\n") => return response,
                Ok(_) | Err(_) => tokio::task::yield_now().await,
            }
        }
    })
    .await
    .expect("released connection slot must be reaped");
    assert!(response.starts_with(b"HTTP/1.1 200 OK\r\n"));
}

#[test]
fn loopback_listener_refuses_to_accept_after_pre_requested_shutdown() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let listener = runtime
        .block_on(GatewayLoopbackListenerV1::bind(
            "127.0.0.1:0".parse().expect("loopback address"),
        ))
        .expect("loopback listener");
    let address = listener.local_address().expect("local address");
    let (shutdown, receiver) = watch::channel(true);
    runtime.block_on(async move {
        listener
            .serve_until_shutdown(GatewayTechnicalRouter::new(true), receiver)
            .await
            .expect("listener exits without accepting");
        assert!(
            TcpStream::connect(address).await.is_err(),
            "listener must release the socket when shutdown precedes its loop"
        );
    });
    drop(shutdown);
}

#[test]
fn loopback_tls_listener_serves_https_without_becoming_a_lan_listener() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let (acceptor, connector) = tls_endpoints();
    let rejected = runtime
        .block_on(GatewayLoopbackTlsListenerV1::bind(
            "192.0.2.10:0".parse().expect("non-loopback address"),
            acceptor.clone(),
        ))
        .err()
        .expect("local HTTPS listener must reject LAN bind");
    assert_eq!(rejected, "local Gateway listener must bind loopback only");
    let listener = runtime
        .block_on(GatewayLoopbackTlsListenerV1::bind(
            "127.0.0.1:0".parse().expect("loopback address"),
            acceptor,
        ))
        .expect("local TLS listener");
    let address = listener.local_address().expect("local address");
    let (shutdown, receiver) = watch::channel(false);
    let server = runtime.spawn(async move {
        listener
            .serve_until_shutdown(GatewayTechnicalRouter::new(true), receiver)
            .await
            .expect("local TLS server")
    });

    runtime.block_on(async move {
        let stream = TcpStream::connect(address).await.expect("TCP connection");
        let server_name = ServerName::try_from("localhost")
            .expect("server name")
            .to_owned();
        let stream = connector
            .connect(server_name, stream)
            .await
            .expect("TLS handshake");
        let mut stream = stream;
        stream
            .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .expect("HTTP request");
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .expect("HTTP response");
        assert!(
            std::str::from_utf8(&response)
                .expect("HTTP response text")
                .starts_with("HTTP/1.1 200 OK\r\n")
        );
        shutdown.send(true).expect("request shutdown");
        server.await.expect("server task");
    });
}

#[test]
fn paired_remote_listener_serves_technical_routes_only_after_tls_handshake() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let (acceptor, connector) = tls_endpoints();
    let listener = runtime.block_on(GatewayTlsListenerV1::bind(
        "127.0.0.1:0".parse().expect("loopback address"),
        PairedRemoteProfileV1::new(true, false).expect("remote profile"),
        acceptor,
    ));

    let listener = listener.expect("TLS listener");
    let address = listener.local_address().expect("local address");
    assert!(address.ip().is_loopback());
    let server = runtime.spawn(async move {
        listener
            .serve_once(GatewayTechnicalRouter::new(true))
            .await
            .expect("TLS HTTP/2 server")
    });

    runtime.block_on(async move {
        let stream = TcpStream::connect(address).await.expect("TCP connection");
        let server_name = ServerName::try_from("localhost")
            .expect("server name")
            .to_owned();
        let stream = connector
            .connect(server_name, stream)
            .await
            .expect("TLS handshake");
        let (mut client, connection) = h2::client::handshake(stream)
            .await
            .expect("HTTP/2 handshake");
        let connection = tokio::spawn(connection);
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://localhost/healthz")
            .body(())
            .expect("HTTP/2 request");
        let (response, _) = client
            .send_request(request, true)
            .expect("send HTTP/2 request");
        let response = response.await.expect("HTTP/2 response");
        assert_eq!(response.status(), StatusCode::OK);
        drop(response);
        drop(client);
        connection
            .await
            .expect("HTTP/2 connection task")
            .expect("HTTP/2 connection");
        server.await.expect("server task");
    });
}

#[path = "gateway_runtime/paired_peer.rs"]
mod paired_peer;

fn tls_endpoints() -> (TlsAcceptor, TlsConnector) {
    paired_peer::tls_endpoints()
}
