use crate::proto::{controller::Empty, controller_grpc::GRPCController};

pub struct Controller {
}

impl GRPCController for Controller {
    fn shutdown(
        &self,
        ctx: ::grpc::ServerHandlerContext,
        _: ::grpc::ServerRequestSingle<Empty>,
        resp: ::grpc::ServerResponseUnarySink<Empty>,
    ) -> ::grpc::Result<()> {
        resp.finish(Empty::new())
    }
}
