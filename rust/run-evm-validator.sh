# rm -rf /tmp/evmDb/validator/
# mkdir -p /tmp/evmDb/validator/
# rm -rf /tmp/test_evm_checkpoints_0xeb382E56eFF04DA7ad115E494207308bb84d82C3
source config/cardano/evm.validator.env
CONFIG_FILES=./config/cardano/cardano.json cargo run --bin validator
