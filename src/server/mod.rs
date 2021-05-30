pub(crate) mod broker;
pub(crate) mod controller;
pub(crate) mod stdio;

use std::thread;

use broker::BrokerServer;
use futures::channel::mpsc::{channel, Receiver};
use stdio::StdioServer;

use crate::proto::{
    broker_grpc::GRPCBrokerServer, controller_grpc::GRPCControllerServer,
    stdio_grpc::GRPCStdioServer,
};

use self::controller::Controller;

pub async fn create_server(err_rx: Receiver<Vec<u8>>, out_rx: Receiver<Vec<u8>>) {
    let (send, recv1) = channel(0);
    let (send1, recv) = channel(0);

    let broker_def = GRPCBrokerServer::new_service_def(BrokerServer::new(send, recv));
    let stdio_def = GRPCStdioServer::new_service_def(StdioServer::new(err_rx, out_rx));

    let controller_def = GRPCControllerServer::new_service_def(Controller {});

    let mut server_builder = grpc::ServerBuilder::new_plain();
    server_builder.add_service(broker_def);
    server_builder.add_service(stdio_def);
    server_builder.add_service(controller_def);
    server_builder.http.set_port(1234);
    let server = server_builder.build().expect("build");

    println!("server stared on addr {}", server.local_addr());

    loop {
        thread::park();
    }
}
