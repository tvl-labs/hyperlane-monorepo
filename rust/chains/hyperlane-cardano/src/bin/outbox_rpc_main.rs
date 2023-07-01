use cardano_rpc::apis::default_api::MessagesByBlockRangeError;
use cardano_rpc::apis::Error;
use hyperlane_cardano::rpc::OutboxRpc;
use hyperlane_cardano::{CardanoMailbox, ConnectionConf};
use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneDomain, KnownHyperlaneDomain, Mailbox, H256,
};
use url::Url;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ChainResult<()> {
    let outbox_rpc = OutboxRpc::new(&"http://localhost:3000".parse().unwrap());
    let finalized_block_number = outbox_rpc.get_finalized_block_number().await.unwrap();
    let messages = outbox_rpc.get_messages_by_block_range(0, 10).await.unwrap();
    let merkle_trees_at_block_number = outbox_rpc
        .get_merkle_trees_at_block_number(8)
        .await
        .unwrap();
    println!("{:?}", finalized_block_number);
    println!("{:?}", merkle_trees_at_block_number.merkle_trees);
    println!("{:?}", messages);

    let locator = ContractLocator {
        domain: &HyperlaneDomain::Known(KnownHyperlaneDomain::CardanoTest1),
        address: H256::zero(),
    };
    let conf = ConnectionConf {
        url: "http://localhost:3000".parse().unwrap(),
    };
    let mailbox = CardanoMailbox::new(&conf, locator, Option::None).unwrap();
    let tree = mailbox.tree(Option::None).await.unwrap();
    println!("{:?}", tree.count());
    println!("{:?}", tree.root());
    println!("{:?}", tree.branch());

    Ok(())
}
