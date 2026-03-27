fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;

    std::env::set_var("PROTOC", protoc);

    tonic_build::configure().compile(
        &[
            "proto/health.proto",
            "proto/hello.proto",
            "proto/common.proto",
            "proto/order.proto",
            "proto/auth.proto",
            "proto/user.proto",
            "proto/transaction.proto",
        ],
        &["proto"],
    )?;

    Ok(())
}
