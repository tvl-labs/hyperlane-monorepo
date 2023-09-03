rm -rf /tmp/evmDb/validator/
mkdir -p /tmp/evmDb/validator/
rm -rf /tmp/checkpoints/0xeb382e56eff04da7ad115e494207308bb84d82c3
source config/cardano/evm.validator.env
CONFIG_FILES=./config/cardano/cardano.json cargo run --bin validator
