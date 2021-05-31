fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &[
            "protos/controller.proto",
            "protos/broker.proto",
            "protos/stdio.proto",
        ],
        &["protos"],
    )?;
    Ok(())
}
