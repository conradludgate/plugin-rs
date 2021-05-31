use std::collections::HashMap;
use std::sync::Arc;

use futures::lock::Mutex;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Response, Status};

use super::pb;

pub struct Broker {
    send: Sender<pb::ConnInfo>,
    recv: Receiver<pb::ConnInfo>,
    streams: HashMap<usize, BrokerPending>,
    id: usize,
}

pub struct BrokerPending {

}

impl Broker {
    pub fn new(send: Sender<pb::ConnInfo>, recv: Receiver<pb::ConnInfo>) -> Self {
        Broker {
            send,
            recv,
            streams: HashMap::new(),
            id: 0,
        }
    }

    pub fn start(self) {

    }

    // fn get_stream(&mut self) -> &mut BrokerPending {
    //     let entry = self.streams.entry(self.id);
    //     self.id += 1;

    //     entry.or_insert_with(|| BrokerPending {

    //     })
    // }
}

pub struct BrokerServer {
    send: Arc<Mutex<Sender<pb::ConnInfo>>>,
    recv: Arc<Mutex<Receiver<pb::ConnInfo>>>,
}

impl BrokerServer {
    pub fn new(send: Sender<pb::ConnInfo>, recv: Receiver<pb::ConnInfo>) -> Self {
        BrokerServer {
            send: Arc::new(Mutex::new(send)),
            recv: Arc::new(Mutex::new(recv)),
        }
    }
}

#[tonic::async_trait]
impl pb::grpc_broker_server::GrpcBroker for BrokerServer {
    async fn start_stream(
        &self,
        request: tonic::Request<tonic::Streaming<pb::ConnInfo>>,
    ) -> Result<tonic::Response<Self::StartStreamStream>, tonic::Status> {
        let send = self.send.clone();
        let recv = self.recv.clone();

        tokio::spawn(async move {
            let send = send.lock().await;
            let mut stream = request.into_inner();
            while let Some(m) = stream.next().await {
                send.send(m.unwrap()).await.unwrap();
            }
        });

        let (tx, rx) = channel(4);
        tokio::spawn(async move {
            let mut recv = recv.lock().await;
            while let Some(m) = recv.recv().await {
                tx.send(Ok(m)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type StartStreamStream = ReceiverStream<Result<pb::ConnInfo, Status>>;
}
