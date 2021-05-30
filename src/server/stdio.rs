use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{channel::mpsc::Receiver, lock::Mutex, Stream, StreamExt};
use grpc::Metadata;

use crate::proto::{
    controller::Empty,
    stdio::{StdioData, StdioData_Channel},
    stdio_grpc::GRPCStdio,
};

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
    type Item = StdioData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.err_done {
            match self.stderr.poll_next_unpin(cx) {
                Poll::Ready(Some(data)) => {
                    return Poll::Ready(Some(StdioData {
                        channel: StdioData_Channel::STDERR,
                        data,
                        ..Default::default()
                    }))
                }
                Poll::Ready(None) => {
                    self.err_done = true;
                }
                Poll::Pending => {}
            }
        }

        if !self.out_done {
            match self.stdout.poll_next_unpin(cx) {
                Poll::Ready(Some(data)) => {
                    return Poll::Ready(Some(StdioData {
                        channel: StdioData_Channel::STDOUT,
                        data,
                        ..Default::default()
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

impl GRPCStdio for StdioServer {
    fn stream_stdio(
        &self,
        ctx: ::grpc::ServerHandlerContext,
        _: ::grpc::ServerRequestSingle<Empty>,
        mut resp: ::grpc::ServerResponseSink<StdioData>,
    ) -> ::grpc::Result<()> {
        let io_stream = self.io_stream.clone();

        ctx.spawn(async move {
            let mut io_stream = io_stream.lock().await;
            while let Some(m) = io_stream.next().await {
                resp.send_data(m)?;
            }
            resp.send_trailers(Metadata::new())
        });

        Ok(())
    }
}
