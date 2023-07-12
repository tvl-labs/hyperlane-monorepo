rm -rf /tmp/cardanoDb/validator/
mkdir -p /tmp/cardanoDb/validator/
rm -rf /tmp/test_cardano_checkpoints_0x70997970c51812dc3a010c7d01b50e0d17dc79c8
source config/cardano/validator.env
HASH_BLAKE2B=true CONFIG_FILES=./config/cardano/cardano.json cargo run --bin validator