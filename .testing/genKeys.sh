#!/bin/bash
# import .env vars
if [ -f .env ]; then
  export $(cat .env | sed 's/#.*//g' | xargs)
fi

USAGE=$(cat <<-END
One parameter is required.

./genKeys.sh -x [Stage]

Example:
./genKeys.sh -x development
END
)

COUNTER="0"
while getopts "x:" options; do
  case $options in
    x ) if [[ -n $OPTARG ]]; then
            STAGE=$OPTARG
            ((COUNTER+=1))
        fi
        ;;
     \? ) echo ${USAGE}
        exit 1
        ;;
  esac
done

if (( ${COUNTER} != 1 )); then
    echo "${USAGE}"
    exit 0
fi

OS=$(uname -s)
echo "Detected:" $OS
if [[ "$OS" == 'Linux' ]]; then
    UTILS="ethUtils_ubuntu"
elif [[ "$OS" == 'Darwin' ]]; then
    UTILS="ethUtils_mac"
fi

BASE_DIR="."
NETWORK_NAME="leopold"
PASSWORD="password"
CONFIG_FILE="authority.toml"
CONFIG_FILE_TEMPLATE="./environment/$STAGE/template/authority.toml"
ACCOUNT_MNEMONIC_FILE="./environment/$STAGE/secrets/AccountMnemonic"
NETWORK_MNEMONIC_FILE="./environment/$STAGE/secrets/NetworkMnemonic"
MACHINE_DIR="$BASE_DIR/dist/$STAGE/"

mkdir $BASE_DIR"/dist/"
mkdir $MACHINE_DIR
mkdir $MACHINE_DIR"/scripts/"

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
echo $PRIV_KEY > "$MACHINE_DIR/key.priv"

# generating private key for keystore file
PRIV_KEY=$($UTILS/ethkey info -b -s "$ACCOUNT_MNEMONIC")
ADDR=0x$($UTILS/ethkey info -b -a "$ACCOUNT_MNEMONIC")

echo "PRIV_KEY         -> $PRIV_KEY"
echo "ADDR             -> $ADDR"
echo

# generate password
echo "Generating password for keystore file for node $i"
openssl rand -hex 40 > "$MACHINE_DIR/$PASSWORD"

cp -f $BASE_DIR"/environment/$STAGE/template/reserved_peers" $MACHINE_DIR"/reserved_peers"
cp -f $BASE_DIR"/environment/$STAGE/template/spec.json" $MACHINE_DIR"/spec.json"
cp -f $BASE_DIR"/environment/$STAGE/files/start.sh" $MACHINE_DIR"/scripts/start.sh"
cp -f $BASE_DIR"/environment/$STAGE/files/format2json.sh" $MACHINE_DIR"/scripts/format2json.sh"

#replace mining address in cofig toml
cp -f $CONFIG_FILE_TEMPLATE $MACHINE_DIR
sed -i'' -e "s/engine_signer = \"\"/engine_signer = \"$ADDR\"/g" "$MACHINE_DIR/$CONFIG_FILE"
rm -f "$MACHINE_DIR/$CONFIG_FILE-e"

# remove all old keystore files
rm -f "$MACHINE_DIR$NETWORK_NAME/UTC"*
# generate keystore file
$UTILS/ethstore insert $PRIV_KEY "$MACHINE_DIR/$PASSWORD" --dir "$MACHINE_DIR/$NETWORK_NAME"


