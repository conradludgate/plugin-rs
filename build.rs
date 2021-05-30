fn main() {
    protoc_rust_grpc::Codegen::new()
        .out_dir("src/proto")
        .input("src/proto/controller.proto")
        .input("src/proto/broker.proto")
        .input("src/proto/stdio.proto")
        .rust_protobuf(true)
        .run()
        .expect("protoc-rust-grpc");
}
