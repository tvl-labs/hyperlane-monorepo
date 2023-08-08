rm -rf /tmp/evmDb/validator3/
mkdir -p /tmp/evmDb/validator3/
rm -rf /tmp/checkpoints/0xF2207C8AF16aEE882B887d56dEB0d20b0A76D5A5
source config/cardano/evm.validator3.env
CONFIG_FILES=./config/cardano/cardano.json cargo run --bin validator
