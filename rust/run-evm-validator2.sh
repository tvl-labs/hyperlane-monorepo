rm -rf /tmp/evmDb/validator2/
mkdir -p /tmp/evmDb/validator2/
rm -rf /tmp/checkpoints/0xb49F8Df5009228976043068059A60B067bD9f0B8
source config/cardano/evm.validator2.env
CONFIG_FILES=./config/cardano/cardano.json cargo run --bin validator
