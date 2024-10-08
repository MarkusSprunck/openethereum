
sudo mkdir -p /home/parity 
sudo chown -R codespace:codespace /home/parity 

mkdir -p /home/parity/chain
mkdir -p /home/parity/data
mkdir -p /home/parity/data/network
mkdir -p /home/parity/data/keys

ln -sf  ${PWD}/dist/staging/authority.toml           /home/parity/authority.toml
ln -sf  ${PWD}/dist/staging/reserved_peers           /home/parity/chain/reserved_peers
ln -sf  ${PWD}/dist/staging/spec.json                /home/parity/chain/spec.json
ln -sf  ${PWD}/dist/staging/password                 /home/parity/password
ln -sf  ${PWD}/dist/staging/leopold                  /home/parity/data/keys
ln -sf  ${PWD}/dist/staging/key.priv                 /home/parity/data/network/key

tree /home/parity/
