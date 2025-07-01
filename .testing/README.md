# OpenEthereum Test Client for Leopold Blockchain 

How can I access the Leopold test environment?

## 2. Contact

Please, get in contact with [sprunck@muenchen.ihk.de](mailto:sprunck@muenchen.ihk.de)

## 2. Architecture

The following diagram shows the Leopold staging environment. Within the IHK Munich (green box), there are 
three OpenEthereum nodes that represent the actual blockchain. Two of these nodes are validator nodes, namely 
Host 1 and 2. A third node serves only as an API interface, providing an RPC interface to the outside, namely 
Host 3. All other software components are used for monitoring the Leopold blockchain.

### 2.1 Deployment

![](images/leopold-infrastructure-view-staging.png)

### 2.2 Topology

![](images/leopold-topologie-stag-6.2.1.png)


### 3.0 Getting Started

Before testing we have to create target folders and 
configuration on local machine.

#### Install GCC-12 and G++-12 and set environment

```shell
sudo apt install cmake
sudo apt install gcc-12 g++-12
```


#### Build Artefacts (once)

For the generation of secrets we need two applications, i.e. *ethkey* and *ethstore*

```bash
.scripts/build-artifacts-cli-tools.sh
```

#### Create Secrets (once)

```bash
echo "1234" > ./secrets/AccountMnemonic
echo "5678" > ./secrets/NetworkMnemonic
```

```bash
cd .testing
./secrets_generation.sh
```

Expected result:

#### Start local Leopold Node

```bash
cd .testing
./test-leopold.sh
```
