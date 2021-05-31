pub mod pb {
    tonic::include_proto!("plugin");
}

pub(crate) mod broker;
pub(crate) mod controller;
pub(crate) mod stdio;

use broker::BrokerServer;
use futures::channel::mpsc::channel as futures_channel;
use futures::{FutureExt, StreamExt};
use stdio::StdioServer;
use tokio::sync::mpsc::{self, Receiver};
use tonic::transport::Server;

use self::broker::Broker;
use self::controller::Controller;

pub async fn create_server(err_rx: Receiver<Vec<u8>>, out_rx: Receiver<Vec<u8>>) {
    // let (send, recv1) = mpsc::channel(0);
    // let (send1, recv) = mpsc::channel(0);

    // let broker_server = BrokerServer::new(send, recv);
    // let broker = Broker::new(send1, recv1);
    let stdio_server = StdioServer::new(err_rx, out_rx);

    let (shutdown, signal) = futures_channel::<()>(0);
    let signal = signal.into_future().map(|_| ());

    let controller = Controller::new(shutdown);

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_service_status("plugin", tonic_health::ServingStatus::Serving).await;

    let addr = "[::1]:10000".parse().unwrap();
    Server::builder()
        // .add_service(pb::grpc_broker_server::GrpcBrokerServer::new(broker_server))
        .add_service(pb::grpc_stdio_server::GrpcStdioServer::new(stdio_server))
        .add_service(pb::grpc_controller_server::GrpcControllerServer::new(
            controller,
        ))
        .add_service(health_service)
        .serve_with_shutdown(addr, signal)
        .await
        .unwrap();
}
