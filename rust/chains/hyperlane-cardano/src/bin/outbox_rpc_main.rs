use cardano_rpc::apis::default_api::MessagesByBlockRangeError;
use cardano_rpc::apis::Error;
use hyperlane_cardano::rpc::{
    get_finalized_block_number, get_merkle_trees_at_block_number, get_messages_by_block_range,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error<MessagesByBlockRangeError>> {
    let finalized_block_number = get_finalized_block_number().await.expect("");
    let messages = get_messages_by_block_range(0, 10).await.expect("");
    let merkle_trees_at_block_number = get_merkle_trees_at_block_number(8).await.expect("");
    println!("{:?}", finalized_block_number);
    println!("{:?}", merkle_trees_at_block_number.merkle_trees);
    println!("{:?}", messages);
    Ok(())
}
