rm -rf /tmp/cardanoDb/relayer/
mkdir /tmp/cardanoDb/relayer/
source config/cardano/relayer.env
CONFIG_FILES=./config/cardano/cardano.json cargo run --bin relayer