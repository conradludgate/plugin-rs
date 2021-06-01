pub mod pb {
    tonic::include_proto!("plugin");
}

pub(crate) mod broker;
pub(crate) mod controller;
pub(crate) mod stdio;

use std::net::SocketAddr;

use futures::channel::mpsc::channel as futures_channel;
use futures::{FutureExt, StreamExt};
use stdio::StdioServer;
use tokio::sync::mpsc::Receiver;
use tonic::body::BoxBody;
use tonic::codegen::http::{Request, Response};
use tonic::transport::server::Router;
use tonic::transport::server::Unimplemented;
use tonic::transport::service::Or;
use tonic::transport::Body;
use tonic::transport::NamedService;
use tonic::transport::Server;
use tower::Service;

use self::controller::Controller;

pub struct PluginServer<A, B> {
    router: Router<A, B>,
}

pub trait Plugin {
    type Service;
    fn create_service(self) -> Self::Service;
}

impl PluginServer<pb::grpc_stdio_server::GrpcStdioServer<StdioServer>, Unimplemented> {
    pub fn new(stderr_rx: Receiver<Vec<u8>>, stdout_rx: Receiver<Vec<u8>>) -> Self {
        let stdio_server = StdioServer::new(stderr_rx, stdout_rx);

        let router = Server::builder()
            .add_service(pb::grpc_stdio_server::GrpcStdioServer::new(stdio_server));

        Self { router }
    }
}

impl<A, B> PluginServer<A, B>
where
    A: Service<Request<Body>, Response = Response<BoxBody>> + Clone + Send + 'static,
    A::Future: Send + 'static,
    A::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    B: Service<Request<Body>, Response = Response<BoxBody>> + Clone + Send + 'static,
    B::Future: Send + 'static,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    /// Add a new service to this router.
    pub fn add_plugin<P: Plugin>(
        self,
        plugin: P,
    ) -> PluginServer<P::Service, Or<A, B, Request<Body>>>
    where
        P::Service: Service<Request<Body>, Response = Response<BoxBody>>
            + NamedService
            + Clone
            + Send
            + 'static,
        <P::Service as Service<Request<Body>>>::Future: Send + 'static,
        <P::Service as Service<Request<Body>>>::Error:
            Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        let router = self.router.add_service(plugin.create_service());
        PluginServer { router }
    }

    pub async fn start(self, addr: SocketAddr) {
        let (shutdown, signal) = futures_channel::<()>(0);
        let signal = signal.into_future().map(|_| ());

        let controller = Controller::new(shutdown);

        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_service_status("plugin", tonic_health::ServingStatus::Serving)
            .await;

        self.router
            .add_service(pb::grpc_controller_server::GrpcControllerServer::new(
                controller,
            ))
            .add_service(health_service)
            .serve_with_shutdown(addr, signal)
            .await
            .unwrap();
    }
}
