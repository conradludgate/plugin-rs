use std::sync::Arc;

use futures::channel::mpsc::Sender;
use tokio::sync::Mutex;
use tonic::Response;

use super::pb;

pub struct Controller {
    shutdown: Arc<Mutex<Sender<()>>>,
}

impl Controller {
    pub fn new(shutdown: Sender<()>) -> Self {
        Controller {
            shutdown: Arc::new(Mutex::new(shutdown)),
        }
    }
}

#[tonic::async_trait]
impl pb::grpc_controller_server::GrpcController for Controller {
    async fn shutdown(
        &self,
        _: tonic::Request<pb::Empty>,
    ) -> Result<tonic::Response<pb::Empty>, tonic::Status> {
        let shutdown = self.shutdown.clone();
        let mut shutdown = shutdown.lock().await;
        shutdown.start_send(()).unwrap();
        Ok(Response::new(pb::Empty::default()))
    }
}
