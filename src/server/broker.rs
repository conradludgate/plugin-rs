use std::sync::Arc;

use futures::StreamExt;
use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::Sender;
use futures::lock::Mutex;
use grpc::Metadata;

use crate::proto::broker::ConnInfo;
use crate::proto::broker_grpc::GRPCBroker;

pub struct BrokerServer {
    send: Arc<Mutex<Sender<ConnInfo>>>,
    recv: Arc<Mutex<Receiver<ConnInfo>>>,
}

impl BrokerServer {
    pub fn new(send: Sender<ConnInfo>, recv: Receiver<ConnInfo>) -> BrokerServer {
        BrokerServer {
            send: Arc::new(Mutex::new(send)),
            recv: Arc::new(Mutex::new(recv)),
        }
    }
}

impl GRPCBroker for BrokerServer {
    fn start_stream(
        &self,
        ctx: grpc::ServerHandlerContext,
        req: grpc::ServerRequest<ConnInfo>,
        mut resp: grpc::ServerResponseSink<ConnInfo>,
    ) -> grpc::Result<()> {
        let send = self.send.clone();
        let recv = self.recv.clone();

        ctx.spawn(async move {
            let mut recv = recv.lock().await;
            while let Some(m) = recv.next().await {
                resp.send_data(m)?;
            }
            resp.send_trailers(Metadata::new())
        });

        let mut req = req.into_stream();
        ctx.spawn(async move {
            let mut send = send.lock().await;
            while let Some(m) = req.next().await {
                send.start_send(m?).unwrap()
            }
            Ok(())
        });

        Ok(())
    }
}
