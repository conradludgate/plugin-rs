use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use super::pb;

use futures::{lock::Mutex, Stream, StreamExt};
use tokio::sync::mpsc::{channel, Receiver};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

pub struct StdioServer {
    io_stream: Arc<Mutex<IoStream>>,
}

impl StdioServer {
    pub fn new(err_rx: Receiver<Vec<u8>>, out_rx: Receiver<Vec<u8>>) -> StdioServer {
        StdioServer {
            io_stream: Arc::new(Mutex::new(IoStream {
                stderr: err_rx,
                stdout: out_rx,
                err_done: false,
                out_done: false,
            })),
        }
    }
}

pub struct IoStream {
    stderr: Receiver<Vec<u8>>,
    stdout: Receiver<Vec<u8>>,
    err_done: bool,
    out_done: bool,
}

impl Stream for IoStream {
    type Item = pb::StdioData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.err_done {
            match self.stderr.poll_recv(cx) {
                Poll::Ready(Some(data)) => {
                    return Poll::Ready(Some(pb::StdioData {
                        channel: pb::stdio_data::Channel::Stderr as i32,
                        data,
                    }))
                }
                Poll::Ready(None) => {
                    self.err_done = true;
                }
                Poll::Pending => {}
            }
        }

        if !self.out_done {
            match self.stdout.poll_recv(cx) {
                Poll::Ready(Some(data)) => {
                    return Poll::Ready(Some(pb::StdioData {
                        channel: pb::stdio_data::Channel::Stdout as i32,
                        data,
                    }))
                }
                Poll::Ready(None) => {
                    self.out_done = true;
                }
                Poll::Pending => {}
            }
        }

        if self.err_done && self.out_done {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

#[tonic::async_trait]
impl pb::grpc_stdio_server::GrpcStdio for StdioServer {
    async fn stream_stdio(
        &self,
        _: tonic::Request<pb::Empty>,
    ) -> Result<tonic::Response<Self::StreamStdioStream>, tonic::Status> {
        let (tx, rx) = channel(4);
        let io_stream = self.io_stream.clone();

        tokio::spawn(async move {
            let mut io_stream = io_stream.lock().await;
            while let Some(m) = io_stream.next().await {
                tx.send(Ok(m)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type StreamStdioStream = ReceiverStream<Result<pb::StdioData, Status>>;
}
