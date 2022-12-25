use blockchaintree::block::{self, BasicInfo};
use blockchaintree::{self, transaction::Transactionable};
use num_bigint::{BigUint, ToBigUint};

static sender: &[u8; 33] = b"123456789012345678901234567890123";
static reciever: &[u8; 33] = b"123456789012345678901234567890123";
static signature: &[u8; 64] = b"1234567890123456789012345678901234567890123456789012345678901234";
static prev_hash: &[u8; 32] = b"12345678901234567890123456789012";

#[tokio::test]
async fn chain_test() {
    let mut blockchain = blockchaintree::blockchaintree::BlockChainTree::without_config().unwrap();

    let default_info = BasicInfo::new(
        500,
        1000u64.to_biguint().unwrap(),
        [0u8; 32],
        [1u8; 32],
        0,
        [5u8; 32],
    );
    let tr = blockchaintree::transaction::Transaction::new(
        sender,
        reciever,
        121212,
        signature,
        2222222288u64.to_biguint().unwrap(),
    );

    let block = block::TokenBlock::new(default_info, String::new(), tr);

    let mut derivative_chain = if let Some(chain) = blockchain.get_derivative_chain(sender).unwrap()
    {
        chain
    } else {
        blockchaintree::blockchaintree::BlockChainTree::create_derivative_chain(
            sender, prev_hash, 0,
        )
        .unwrap()
    };

    derivative_chain.add_block(&block).await.unwrap();

    let block_db = derivative_chain.find_by_height(0).unwrap().unwrap();
    assert_eq!(block_db.payment_transaction.get_sender(), sender);
}