use hyperlane_cardano::rpc::OutboxRpc;
use hyperlane_cardano::{CardanoMailbox, CardanoMailboxIndexer, ConnectionConf};
use hyperlane_core::{
    ChainResult, ContractLocator, HyperlaneDomain, HyperlaneMessage, IndexRange, Indexer,
    KnownHyperlaneDomain, Mailbox, H256,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ChainResult<()> {
    let outbox_rpc = OutboxRpc::new(&"http://localhost:3000".parse().unwrap());
    let finalized_block_number = outbox_rpc.get_finalized_block_number().await.unwrap();
    let messages = outbox_rpc.get_messages_by_block_range(0, 10).await.unwrap();
    let merkle_tree = outbox_rpc.get_latest_merkle_tree().await.unwrap();
    println!("{:?}", finalized_block_number);
    println!("{:?}", merkle_tree);
    println!("{:?}", messages);

    let locator = ContractLocator {
        domain: &HyperlaneDomain::Known(KnownHyperlaneDomain::CardanoTest1),
        address: H256::zero(),
    };
    let conf = ConnectionConf {
        url: "http://localhost:3000".parse().unwrap(),
    };
    let mailbox = CardanoMailbox::new(&conf, locator.clone(), Option::None).unwrap();
    let tree = mailbox.tree(Option::None).await.unwrap();
    println!("{:?}", tree.count());
    println!("{:?}", tree.root());
    println!("{:?}", tree.branch());

    let cardano_mailbox_indexer = CardanoMailboxIndexer::new(&conf, locator.clone()).unwrap();
    let finalized_block_number = Indexer::<HyperlaneMessage>::fetch_logs(
        &cardano_mailbox_indexer,
        IndexRange::Blocks(0, 10),
    )
    .await
    .unwrap();
    println!("{:?}", finalized_block_number);

    Ok(())
}
