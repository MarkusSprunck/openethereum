# OpenEthereum Test Client for Leopold Blockchain 

How can I access the Leopold test environment?

## 2. Getting Started

### 2.1 Two Secret Files will be needed

- /environment/staging/secrets/AccountMnemonic
- /environment/staging/secrets/NetworkMnemonic

Please, get in contact with [sprunck@muenchen.ihk.de](mailto:sprunck@muenchen.ihk.de) for more details.

### 2.2 Authorize executable files

After downloading, the executable files under Linux must also be given appropriate rights. Since different binaries 
are needed for Ubuntu and macOS, the genKeys.sh script decides which ones should be executed.

```shell
chmod +x *.sh
chmod +x ./ethUtils_ubuntu/*
chmod +x ./ethUtils_mac/*
```

### 2.3 Start the test node

```shell
./start_staging.sh
```

## 2. Architecture

The following diagram shows the Leopold OTC staging environment. Within the IHK Munich (green box), there are 
three OpenEthereum nodes that represent the actual blockchain. Two of these nodes are validator nodes, namely 
Host 1 and 2. A third node serves only as an API interface, providing an RPC interface to the outside, namely 
Host 3. All other software components are used for monitoring the Leopold blockchain.

### 2.1 Deployment

![](images/leopold-infrastructure-view-staging.png)

### 2.2 Topology

![](images/leopold-topologie-stag-6.2.1.png)


### 3.0 Create Symbolic Links for local testing

Create needed folder

```shell
sudo mkdir /home/parity 
mkdir /home/parity/chain
mkdir /home/parity/data
```

Create symbolic links to run in local development environment.

```shell
ln -sf  ${PWD}/dist/staging/authority.toml           /home/parity/authority.toml
ln -sf  ${PWD}/dist/staging/reserved_peers           /home/parity/chain/reserved_peers
ln -sf  ${PWD}/dist/staging/spec.json                /home/parity/chain/spec.json
ln -sf  ${PWD}/dist/staging/password                 /home/parity/password
ln -sf  ${PWD}/dist/staging/leopold                  /home/parity/data/keys/leopold
ln -sf  ${PWD}/dist/staging/key.priv                 /home/parity/data/network/key/key.priv
ln -sf  ${PWD}/dist/staging/data                     /home/parity/data
```

