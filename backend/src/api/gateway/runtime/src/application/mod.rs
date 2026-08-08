//! Detached composition of the narrow pre-owner Gateway HTTP surface.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;

use bytes::Bytes;
use http_body_util::Full;
use hyper::Request;
use hyper::body::Body;
use hyper::header::{HOST, ORIGIN};
use hyper::service::Service;
use hyper::{Response, StatusCode};
use makosh_gateway_session_contract::{
    BrowserAuthenticationAuthority, BrowserEnrollmentAuthority, ClientBootstrapAuthority,
};

use crate::{
    BrowserAuthenticationRouter, BrowserBootstrapRouter, BrowserPairingRouter,
    BrowserRealtimeRouter, BrowserRealtimeSubscriptionSource, BrowserSessionStatusRouter,
    ClientBlobRouter, ClientBootstrapRouter, ClientRpcRouter, GatewayHttpResponse,
    GatewayTechnicalRouter, OwnerModuleSettingsRouter, OwnerVaultProvisioningRouter,
    SharedBrowserGatewaySessionService,
};

const AUTHENTICATION_PREFIX: &str = "/browser/v1/authentication/";
const PAIRING_PREFIX: &str = "/browser/v1/pairing/";
const REALTIME_PATH: &str = "/api/realtime/v1/events";
const SESSION_STATUS_PATH: &str = "/makosh.gateway.v1.BrowserSessionService/GetStatus";
const CLIENT_BOOTSTRAP_PATH: &str = "/makosh.gateway.v1.ClientBootstrapService/GetBootstrap";
const DEVELOPMENT_PROXY_PROOF_HEADER: &str = "x-makosh-development-proxy-proof";

/// Composes technical health, browser authentication and client-safe realtime
/// without adding an owner API or mounting a listener.
pub struct GatewayApplicationRouter<A, S> {
    technical: GatewayTechnicalRouter,
    browser_authentication: BrowserAuthenticationRouter<A>,
    browser_pairing: Option<BrowserPairingRouter<A>>,
    browser_bootstrap: Option<BrowserBootstrapRouter>,
    browser_session_status: BrowserSessionStatusRouter<A>,
    client_bootstrap: ClientBootstrapRouter<A>,
    client_blob_routes: Vec<ClientBlobRouter<A>>,
    client_rpc_routes: Vec<ClientRpcRouter<A>>,
    owner_module_settings: Option<OwnerModuleSettingsRouter<A>>,
    owner_vault_provisioning: Option<OwnerVaultProvisioningRouter<A>>,
    browser_realtime: BrowserRealtimeRouter<A, S>,
    development_policy: Option<DevelopmentRequestPolicyV1>,
}

#[derive(Clone)]
struct DevelopmentRequestPolicyV1 {
    exact_origin: String,
    exact_authority: String,
    proxy_proof: Option<String>,
}

impl<A, S> Clone for GatewayApplicationRouter<A, S> {
    fn clone(&self) -> Self {
        Self {
            technical: self.technical,
            browser_authentication: self.browser_authentication.clone(),
            browser_pairing: self.browser_pairing.clone(),
            browser_bootstrap: self.browser_bootstrap.clone(),
            browser_session_status: self.browser_session_status.clone(),
            client_bootstrap: self.client_bootstrap.clone(),
            client_blob_routes: self.client_blob_routes.clone(),
            client_rpc_routes: self.client_rpc_routes.clone(),
            owner_module_settings: self.owner_module_settings.clone(),
            owner_vault_provisioning: self.owner_vault_provisioning.clone(),
            browser_realtime: self.browser_realtime.clone(),
            development_policy: self.development_policy.clone(),
        }
    }
}

impl<A, S> GatewayApplicationRouter<A, S>
where
    A: BrowserAuthenticationAuthority + BrowserEnrollmentAuthority + ClientBootstrapAuthority,
    S: BrowserRealtimeSubscriptionSource,
{
    #[must_use]
    pub fn new(ready: bool, service: SharedBrowserGatewaySessionService<A>, source: S) -> Self {
        Self {
            technical: GatewayTechnicalRouter::new(ready),
            browser_authentication: BrowserAuthenticationRouter::from_shared(service.clone()),
            browser_pairing: None,
            browser_bootstrap: None,
            browser_session_status: BrowserSessionStatusRouter::from_shared(service.clone()),
            client_bootstrap: ClientBootstrapRouter::from_shared(service.clone()),
            client_blob_routes: Vec::new(),
            client_rpc_routes: Vec::new(),
            owner_module_settings: None,
            owner_vault_provisioning: None,
            browser_realtime: BrowserRealtimeRouter::new(service, source),
            development_policy: None,
        }
    }

    #[must_use]
    pub fn with_browser_pairing(mut self, router: BrowserPairingRouter<A>) -> Self {
        self.browser_pairing = Some(router);
        self
    }

    #[must_use]
    pub fn with_browser_bootstrap(mut self, router: BrowserBootstrapRouter) -> Self {
        self.browser_bootstrap = Some(router);
        self
    }

    pub fn with_client_rpc_routes(
        mut self,
        routes: Vec<ClientRpcRouter<A>>,
    ) -> Result<Self, &'static str> {
        let mut paths = std::collections::BTreeSet::new();
        if !routes.iter().all(|route| paths.insert(route.path())) {
            return Err("duplicate owner ClientRpc route");
        }
        if routes.iter().any(|route| {
            self.client_blob_routes
                .iter()
                .any(|blob| blob.path() == route.path())
        }) {
            return Err("owner client route path conflict");
        }
        self.client_rpc_routes = routes;
        Ok(self)
    }

    pub fn with_client_blob_routes(
        mut self,
        routes: Vec<ClientBlobRouter<A>>,
    ) -> Result<Self, &'static str> {
        let mut paths = std::collections::BTreeSet::new();
        if !routes.iter().all(|route| paths.insert(route.path())) {
            return Err("duplicate owner client Blob route");
        }
        if routes.iter().any(|route| {
            self.client_rpc_routes
                .iter()
                .any(|rpc| rpc.path() == route.path())
        }) {
            return Err("owner client route path conflict");
        }
        self.client_blob_routes = routes;
        Ok(self)
    }

    #[must_use]
    pub fn with_owner_vault_provisioning(
        mut self,
        router: OwnerVaultProvisioningRouter<A>,
    ) -> Self {
        self.owner_vault_provisioning = Some(router);
        self
    }

    #[must_use]
    pub fn with_owner_module_settings(mut self, router: OwnerModuleSettingsRouter<A>) -> Self {
        self.owner_module_settings = Some(router);
        self
    }

    pub fn with_lan_development_policy(mut self, exact_origin: &str) -> Result<Self, &'static str> {
        let exact_authority = exact_origin
            .strip_prefix("http://")
            .filter(|authority| !authority.is_empty() && !authority.contains('/'))
            .ok_or("developer mode origin is invalid")?;
        self.development_policy = Some(DevelopmentRequestPolicyV1 {
            exact_origin: exact_origin.to_owned(),
            exact_authority: exact_authority.to_owned(),
            proxy_proof: None,
        });
        Ok(self)
    }

    pub fn with_loopback_development_proxy_policy(
        mut self,
        exact_origin: &str,
        proxy_proof: &str,
    ) -> Result<Self, &'static str> {
        let exact_authority = exact_origin
            .strip_prefix("http://")
            .filter(|authority| !authority.is_empty() && !authority.contains('/'))
            .ok_or("loopback development origin is invalid")?;
        if proxy_proof.len() != 64 || !proxy_proof.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("loopback development proxy proof is invalid");
        }
        self.development_policy = Some(DevelopmentRequestPolicyV1 {
            exact_origin: exact_origin.to_owned(),
            exact_authority: exact_authority.to_owned(),
            proxy_proof: Some(proxy_proof.to_owned()),
        });
        Ok(self)
    }

    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let path = request.uri().path();
        let route = route_class(path, &self.client_blob_routes, &self.client_rpc_routes);
        let method = request.method().clone();
        if let Some(policy) = &self.development_policy
            && !policy.admits(&request)
        {
            println!(
                "developer_gateway_request method={} route={} status={} admission=rejected",
                method,
                route,
                StatusCode::FORBIDDEN.as_u16()
            );
            return forbidden();
        }
        let response = self.route_admitted(request).await;
        if self.development_policy.is_some() {
            println!(
                "developer_gateway_request method={} route={} status={} admission=accepted",
                method,
                route,
                response.status().as_u16()
            );
        }
        response
    }

    async fn route_admitted<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let path = request.uri().path();
        if request.uri().query().is_some() {
            return self.technical.route(request.method(), "");
        }
        if path == "/" || path.starts_with("/assets/") {
            return match &self.browser_bootstrap {
                Some(router) => router.route(request.method(), path),
                None => self.technical.route(request.method(), path),
            };
        }
        if is_technical_path(path) {
            return self.technical.route(request.method(), path);
        }
        if path == REALTIME_PATH {
            return self.browser_realtime.route(request);
        }
        if path == SESSION_STATUS_PATH {
            return self.browser_session_status.route(request).await;
        }
        if path == CLIENT_BOOTSTRAP_PATH {
            return self.client_bootstrap.route(request).await;
        }
        if OwnerVaultProvisioningRouter::<A>::admits_path(path) {
            return match &self.owner_vault_provisioning {
                Some(router) => router.route(request).await,
                None => self.technical.route(request.method(), path),
            };
        }
        if OwnerModuleSettingsRouter::<A>::admits_path(path) {
            return match &self.owner_module_settings {
                Some(router) => router.route(request).await,
                None => self.technical.route(request.method(), path),
            };
        }
        if let Some(router) = self
            .client_blob_routes
            .iter()
            .find(|router| router.path() == path)
        {
            return router.route(request).await;
        }
        if let Some(router) = self
            .client_rpc_routes
            .iter()
            .find(|router| router.path() == path)
        {
            return router.route(request).await;
        }
        if path.starts_with(AUTHENTICATION_PREFIX) {
            if self.development_policy.is_some() {
                return self.technical.route(request.method(), path);
            }
            return self.browser_authentication.route(request).await;
        }
        if path.starts_with(PAIRING_PREFIX) {
            if self.development_policy.is_some() {
                return self.technical.route(request.method(), path);
            }
            return match &self.browser_pairing {
                Some(router) => router.route(request).await,
                None => self.technical.route(request.method(), path),
            };
        }
        self.technical.route(request.method(), path)
    }
}

impl DevelopmentRequestPolicyV1 {
    fn admits<B>(&self, request: &Request<B>) -> bool {
        const FORWARDED_HEADERS: [&str; 7] = [
            "forwarded",
            "x-forwarded-for",
            "x-forwarded-host",
            "x-forwarded-proto",
            "cf-connecting-ip",
            "true-client-ip",
            "x-real-ip",
        ];
        let headers = request.headers();
        if FORWARDED_HEADERS
            .iter()
            .any(|name| headers.contains_key(*name))
        {
            return false;
        }
        let mut proof_headers = headers.get_all(DEVELOPMENT_PROXY_PROOF_HEADER).iter();
        let first_proof = proof_headers.next();
        match &self.proxy_proof {
            Some(expected)
                if first_proof.and_then(|value| value.to_str().ok()) == Some(expected.as_str())
                    && proof_headers.next().is_none() => {}
            None if first_proof.is_none() => {}
            Some(_) | None => return false,
        }
        let header_authority = headers.get(HOST).and_then(|value| value.to_str().ok());
        let uri_authority = request.uri().authority().map(|value| value.as_str());
        if header_authority
            .into_iter()
            .chain(uri_authority)
            .any(|authority| authority != self.exact_authority)
            || (header_authority.is_none() && uri_authority.is_none())
        {
            return false;
        }
        let origin = headers.get(ORIGIN).and_then(|value| value.to_str().ok());
        if self.proxy_proof.is_some() && origin != Some(self.exact_origin.as_str()) {
            return false;
        }
        if self.proxy_proof.is_none() && origin.is_some_and(|origin| origin != self.exact_origin) {
            return false;
        }
        headers
            .get("sec-fetch-site")
            .and_then(|value| value.to_str().ok())
            .is_none_or(|site| matches!(site, "same-origin" | "none"))
    }
}

fn route_class<A>(
    path: &str,
    client_blob_routes: &[ClientBlobRouter<A>],
    client_rpc_routes: &[ClientRpcRouter<A>],
) -> &'static str
where
    A: BrowserAuthenticationAuthority,
{
    match path {
        "/" => "browser_bootstrap",
        "/healthz" => "health",
        "/readyz" => "readiness",
        REALTIME_PATH => "client_realtime",
        SESSION_STATUS_PATH => "browser_session_status",
        CLIENT_BOOTSTRAP_PATH => "client_bootstrap",
        path if path.starts_with(AUTHENTICATION_PREFIX) => "browser_authentication",
        path if path.starts_with(PAIRING_PREFIX) => "browser_pairing",
        path if OwnerVaultProvisioningRouter::<A>::admits_path(path) => "owner_vault_provisioning",
        path if OwnerModuleSettingsRouter::<A>::admits_path(path) => "owner_module_settings",
        path if client_blob_routes.iter().any(|route| route.path() == path) => "client_blob",
        path if client_rpc_routes.iter().any(|route| route.path() == path) => "client_rpc",
        _ => "unknown",
    }
}

fn forbidden() -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("cache-control", "no-store")
        .body(crate::full_gateway_body(Bytes::from_static(
            b"developer admission rejected\n",
        )))
        .expect("Gateway rejection response is valid")
}

impl<A, S> Service<Request<hyper::body::Incoming>> for GatewayApplicationRouter<A, S>
where
    A: BrowserAuthenticationAuthority
        + BrowserEnrollmentAuthority
        + ClientBootstrapAuthority
        + Send
        + Sync
        + 'static,
    S: BrowserRealtimeSubscriptionSource + 'static,
{
    type Response = GatewayHttpResponse;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<hyper::body::Incoming>) -> Self::Future {
        let router = self.clone();
        Box::pin(async move { Ok(router.route(request).await) })
    }
}

/// HTTP/3 buffers its bounded request body before invoking the same Gateway
/// router. Keeping this adapter body explicit avoids treating QUIC streams as
/// a second owner API surface.
impl<A, S> Service<Request<Full<Bytes>>> for GatewayApplicationRouter<A, S>
where
    A: BrowserAuthenticationAuthority
        + BrowserEnrollmentAuthority
        + ClientBootstrapAuthority
        + Send
        + Sync
        + 'static,
    S: BrowserRealtimeSubscriptionSource + 'static,
{
    type Response = GatewayHttpResponse;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<Full<Bytes>>) -> Self::Future {
        let router = self.clone();
        Box::pin(async move { Ok(router.route(request).await) })
    }
}

fn is_technical_path(path: &str) -> bool {
    matches!(path, "/healthz" | "/readyz")
}
