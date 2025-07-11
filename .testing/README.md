# OpenEthereum Test Client for Leopold Blockchain

How can I access the Leopold test environment?

## 1.0 Contact

Please, get in contact with [sprunck@muenchen.ihk.de](mailto:sprunck@muenchen.ihk.de)

## 2.0 Architecture

The following diagram shows the Leopold staging environment. Within the IHK Munich (green box), there are
three OpenEthereum nodes that represent the actual blockchain. Two of these nodes are validator nodes, namely
Host 1 and 2. A third node serves only as an API interface, providing an RPC interface to the outside, namely
Host 3. All other software components are used for monitoring the Leopold blockchain.

### 2.1 Deployment

![Leopold staging environment infrastructure diagram](images/leopold-infrastructure-view-staging.png)

### 2.2 Topology

![Leopold staging environment topology diagram](images/leopold-topologie-stag-6.2.1.png)

## 3.0 Getting Started

Before testing we have to create target folders and configuration on local machine.

### 3.1 Build Artefacts (once)

For the generation of secrets we need two applications, i.e. *ethkey* and *ethstore*

```bash
../scripts/build-artifacts-cli-tools-macos-arm64.sh
```

### 3.2 Create Secrets (once)

These mnmonics determine the identiy of the blockchain client, there should be not
two identical clients at the same time, so in the case you work without knowing that
the node is already running, pease change the content of the mnemonic files.

```bash
echo "<your 1st random string>" > ./secrets/AccountMnemonic
echo "<your 2nd random string>" > ./secrets/NetworkMnemonic
```

```bash
./leopold-secrets-generation.sh
```

### 3.3 Start local Leopold Node

```bash
./leopold-run.sh
```
