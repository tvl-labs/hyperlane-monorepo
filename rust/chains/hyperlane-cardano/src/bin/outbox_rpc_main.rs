use cardano_rpc::apis::default_api::MessagesByBlockRangeError;
use cardano_rpc::apis::Error;
use hyperlane_cardano::rpc::outbox_rpc_main;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error<MessagesByBlockRangeError>> {
    outbox_rpc_main().await
}
