rm -rf /tmp/evmDb/relayer/
mkdir /tmp/evmDb/relayer/
source config/cardano/relayer.env
MERKLE_TREE_HASH_BLAKE2B=true CONFIG_FILES=./config/cardano/cardano.json cargo run --bin relayer
