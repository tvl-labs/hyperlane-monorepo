use cardano_rpc::apis::configuration::Configuration;
use cardano_rpc::apis::default_api::{api_outbox_messages_get, ApiOutboxMessagesGetError};
use cardano_rpc::apis::Error;
use serde::{Deserialize, Serialize};

const RPC_URL: &str = "http://localhost:4010";

pub async fn outbox_rpc_main() -> Result<(), Error<ApiOutboxMessagesGetError>> {
    let configuration = configuration();
    let messages = api_outbox_messages_get(&configuration).await?;
    println!("{:#?}", messages);

    Ok(())
}

fn configuration() -> Configuration {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Prefer",
        reqwest::header::HeaderValue::from_static("example=multipleMessages"),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    Configuration {
        base_path: String::from(RPC_URL),
        client,
        ..Configuration::new().clone()
    }
}
