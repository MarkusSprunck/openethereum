#!/usr/bin/env bash

echo "###################################################################################"
echo "# 1. Create Secrets and Configuration"
echo "###################################################################################"

UTILS="../.artifacts"
BASE_DIR="$(pwd)"
PASSWORD="password"
CONFIG_FILE="authority.toml"
CONFIG_FILE_TEMPLATE="./template/authority.toml"
ACCOUNT_MNEMONIC_FILE="./secrets/AccountMnemonic"
NETWORK_MNEMONIC_FILE="./secrets/NetworkMnemonic"
MACHINE_DIR="$BASE_DIR/dist/"

echo "BASE_DIR -> $BASE_DIR"
echo "MACHINE_DIR -> $MACHINE_DIR"
echo "ACCOUNT_MNEMONIC_FILE -> $ACCOUNT_MNEMONIC_FILE"
echo "NETWORK_MNEMONIC_FILE -> $NETWORK_MNEMONIC_FILE"
echo "###################################################################################"

mkdir -p $BASE_DIR"/dist/"
mkdir -p $MACHINE_DIR
mkdir -p $MACHINE_DIR"chain"
mkdir -p $MACHINE_DIR"data"
mkdir -p $MACHINE_DIR"data/keys"
mkdir -p $MACHINE_DIR"data/network"

# read mnemonic from file
ACCOUNT_MNEMONIC=$(cat $ACCOUNT_MNEMONIC_FILE | head -1 | tail -1)
NETWORK_MNEMONIC=$(cat $NETWORK_MNEMONIC_FILE | head -1 | tail -1)

#TODO: remove old files and create a new dir for the machine

echo "Generating key material for validator node"
echo
echo "NETWORK_MNEMONIC -> '$NETWORK_MNEMONIC'"
echo "ACCOUNT_MNEMONIC -> '$ACCOUNT_MNEMONIC'"

PRIV_KEY=$($UTILS/ethkey info -b -s "$NETWORK_MNEMONIC")
PUB_KEY=$($UTILS/ethkey info -b -p  "$NETWORK_MNEMONIC")
echo $PRIV_KEY > "${MACHINE_DIR}data/network/key"

# generating private key for keystore file
PRIV_KEY=$($UTILS/ethkey info -b -s "$ACCOUNT_MNEMONIC")
ADDR=0x$($UTILS/ethkey info -b -a "$ACCOUNT_MNEMONIC")

echo "PRIV_KEY         -> $PRIV_KEY"
echo "ADDR             -> $ADDR"
echo

# generate password
echo "Generating password for keystore file for node $i"
openssl rand -hex 40 > "$MACHINE_DIR/$PASSWORD"

cp -f $BASE_DIR"/template/reserved_peers" $MACHINE_DIR"chain/reserved_peers"
cp -f $BASE_DIR"/template/spec.json" $MACHINE_DIR"chain/spec.json"

#replace mining address in cofig toml
cp -f $CONFIG_FILE_TEMPLATE $MACHINE_DIR
sed -i'' -e "s|engine_signer = \"\"|engine_signer = \"$ADDR\"|g" "$MACHINE_DIR/$CONFIG_FILE"
sed -i'' -e "s|BASE_DIRECTORY|$MACHINE_DIR|g"                    "$MACHINE_DIR/$CONFIG_FILE"

rm -f "$MACHINE_DIR/$CONFIG_FILE-e"

# remove all old keystore files
rm -f "$MACHINE_DIR/data/keys/UTC"*
# generate keystore file
$UTILS/ethstore insert $PRIV_KEY "$MACHINE_DIR/$PASSWORD" --dir "$MACHINE_DIR/data/keys/"
