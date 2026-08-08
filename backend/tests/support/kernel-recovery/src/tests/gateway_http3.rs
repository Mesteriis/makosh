use std::sync::Arc;

use hyper::{Method, Request, StatusCode};
use makosh_gateway_runtime::{
    GatewayHttp3ListenerV1, GatewayTechnicalRouter, PairedRemoteProfileV1,
};
use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use quinn::{ClientConfig as QuinnClientConfig, Endpoint, ServerConfig as QuinnServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use tokio::sync::watch;

#[test]
fn paired_remote_http3_listener_serves_the_same_technical_health_route() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let (server_config, client_config) = http3_endpoints();
    let listener = runtime
        .block_on(async {
            GatewayHttp3ListenerV1::bind(
                "127.0.0.1:0".parse().expect("loopback address"),
                PairedRemoteProfileV1::new(true, false).expect("remote profile"),
                server_config,
            )
        })
        .expect("HTTP/3 listener");
    let address = listener.local_address().expect("local address");
    let (shutdown, receiver) = watch::channel(false);
    let server = runtime.spawn(async move {
        listener
            .serve_until_shutdown(GatewayTechnicalRouter::new(true), receiver)
            .await
            .expect("HTTP/3 server")
    });
    runtime.block_on(request_health(address, client_config, shutdown, server));
}

async fn request_health(
    address: std::net::SocketAddr,
    client_config: QuinnClientConfig,
    shutdown: watch::Sender<bool>,
    server: tokio::task::JoinHandle<()>,
) {
    let mut endpoint = Endpoint::client("127.0.0.1:0".parse().expect("client bind"))
        .expect("HTTP/3 client endpoint");
    endpoint.set_default_client_config(client_config);
    let connection = endpoint
        .connect(address, "localhost")
        .expect("HTTP/3 connect request")
        .await
        .expect("HTTP/3 TLS handshake");
    let (mut driver, mut sender) = h3::client::new(h3_quinn::Connection::new(connection))
        .await
        .expect("HTTP/3 client");
    let driver = tokio::spawn(async move {
        let _ = futures_util::future::poll_fn(|context| driver.poll_close(context)).await;
    });
    let request = Request::builder()
        .method(Method::GET)
        .uri("https://localhost/healthz")
        .body(())
        .expect("HTTP/3 request");
    let mut stream = sender
        .send_request(request)
        .await
        .expect("send HTTP/3 request");
    stream.finish().await.expect("finish HTTP/3 request");
    assert_eq!(
        stream
            .recv_response()
            .await
            .expect("HTTP/3 response")
            .status(),
        StatusCode::OK
    );
    drop(stream);
    drop(sender);
    endpoint.close(0_u32.into(), b"test complete");
    shutdown.send(true).expect("request shutdown");
    server.await.expect("server task");
    driver.abort();
}

fn http3_endpoints() -> (QuinnServerConfig, QuinnClientConfig) {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certified = generate_simple_self_signed(vec!["localhost".to_owned()])
        .expect("self-signed test certificate");
    let certificate = certified.cert.der().clone();
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
    let mut server_crypto = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], key)
        .expect("HTTP/3 server configuration");
    server_crypto.alpn_protocols = vec![b"h3".to_vec()];
    server_crypto.max_early_data_size = 0;
    let server = QuinnServerConfig::with_crypto(Arc::new(
        QuicServerConfig::try_from(server_crypto).expect("QUIC server crypto"),
    ));
    let mut roots = RootCertStore::empty();
    roots.add(certificate).expect("test certificate root");
    let mut client_crypto = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    client_crypto.alpn_protocols = vec![b"h3".to_vec()];
    let client = QuinnClientConfig::new(Arc::new(
        QuicClientConfig::try_from(client_crypto).expect("QUIC client crypto"),
    ));
    (server, client)
}
