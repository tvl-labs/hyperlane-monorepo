use cardano_rpc::apis::configuration::Configuration;
use cardano_rpc::apis::default_api::{messages_by_block_range, MessagesByBlockRangeError};
use cardano_rpc::apis::Error;
use serde::{Deserialize, Serialize};

const RPC_URL: &str = "http://localhost:4010";

pub async fn outbox_rpc_main() -> Result<(), Error<MessagesByBlockRangeError>> {
    let configuration = configuration();
    let messages = messages_by_block_range(&configuration, 0, 10).await?;
    println!("{:#?}", messages);

    Ok(())
}

fn configuration() -> Configuration {
    let client = reqwest::Client::builder().build().unwrap();
    Configuration {
        base_path: String::from(RPC_URL),
        client,
        ..Configuration::new().clone()
    }
}
