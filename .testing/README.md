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

```shell
echo "1234" > environment/staging/secrets/AccountMnemonic
echo "5678" > environment/staging/secrets/NetworkMnemonic
```

```shell
./setup_folders.sh
```

```shell
./secrets_generation.sh
```

Expected result:

```text
###################################################################################
# 1. Create Secrets and Configuration
###################################################################################
Generating key material for validator node

NETWORK_MNEMONIC -> '5678'
ACCOUNT_MNEMONIC -> '1234'
PRIV_KEY         -> 5bdbac19375a24cac7fea60830773f1c9497012394c7b059efd2c9a3669810fa
ADDR             -> 0x0012a3c9f542a2753a7d46f8b2df7fe5762a548a

Generating password for keystore file for node 
0x0012a3c9f542a2753a7d46f8b2df7fe5762a548a
```

#### Start local Leopold Node

```shell
./test-leopold.sh
```

Expected result:

```text
 ./build-artifacts.sh
_____ Set GCC-12 and G++-12 as default compiler _____
_____ Build tools _____
   Compiling libc v0.2.159
   Compiling proc-macro2 v1.0.87
   Compiling unicode-ident v1.0.13
   Compiling serde v1.0.210
   Compiling byteorder v1.5.0
   Compiling crunchy v0.2.2
   Compiling cfg-if v0.1.10
   Compiling rand_core v0.4.2
   Compiling autocfg v0.1.7
   Compiling typenum v1.11.2
   Compiling quote v1.0.37
   Compiling syn v2.0.79
   Compiling cc v1.0.41
   Compiling tiny-keccak v2.0.2
   Compiling rand_core v0.3.1
   Compiling generic-array v0.12.3
   Compiling getrandom v0.1.13
   Compiling rustc-hex v2.1.0
   Compiling rand_core v0.5.1
   Compiling either v1.5.3
   Compiling semver-parser v0.7.0
   Compiling static_assertions v1.1.0
   Compiling radium v0.3.0
   Compiling rlp v0.4.6
   Compiling bitvec v0.17.4
   Compiling byte-slice-cast v0.3.5
   Compiling arrayvec v0.5.1
   Compiling impl-rlp v0.2.1
   Compiling memchr v2.7.4
   Compiling uint v0.8.5
   Compiling byte-tools v0.3.1
   Compiling opaque-debug v0.2.3
   Compiling block-padding v0.1.4
   Compiling rand_pcg v0.1.2
   Compiling rand_chacha v0.1.1
   Compiling proc-macro-hack v0.5.19
   Compiling getrandom v0.2.2
   Compiling maybe-uninit v2.0.0
   Compiling scopeguard v1.1.0
   Compiling digest v0.8.1
   Compiling block-cipher-trait v0.6.2
   Compiling rand v0.5.6
   Compiling rand v0.6.5
   Compiling cfg-if v1.0.0
   Compiling syn v1.0.109
   Compiling spin v0.9.8
   Compiling unicode-width v0.1.14
   Compiling lazy_static v1.5.0
   Compiling getopts v0.2.21
   Compiling block-buffer v0.7.3
   Compiling rand v0.4.6
   Compiling cmake v0.1.42
   Compiling rand_hc v0.1.0
   Compiling rand_xorshift v0.1.1
   Compiling rand_isaac v0.1.1
   Compiling rand_os v0.1.3
   Compiling rand_jitter v0.1.4
   Compiling smallvec v1.6.1
   Compiling subtle v1.0.0
   Compiling remove_dir_all v0.5.2
   Compiling tempdir v0.3.7
   Compiling crypto-mac v0.7.0
   Compiling const-random-macro v0.1.13
   Compiling smallvec v0.6.14
   Compiling pulldown-cmark v0.0.3
   Compiling serde_derive v1.0.210
   Compiling zerocopy-derive v0.7.35
   Compiling secp256k1-sys v0.1.2
   Compiling fake-simd v0.1.2
   Compiling safemem v0.3.3
   Compiling base64 v0.9.3
   Compiling sha2 v0.8.0
   Compiling zerocopy v0.7.35
   Compiling skeptic v0.4.0
   Compiling ppv-lite86 v0.2.20
   Compiling const-random v0.1.13
   Compiling hmac v0.7.1
   Compiling parity-snappy-sys v0.1.2
   Compiling c2-chacha v0.2.3
   Compiling rand_chacha v0.2.1
   Compiling aes-soft v0.3.3
   Compiling rand v0.7.3
   Compiling lock_api v0.3.4
   Compiling stream-cipher v0.3.2
   Compiling hashbrown v0.6.3
   Compiling autocfg v1.0.0
   Compiling fixed-hash v0.6.1
   Compiling arrayvec v0.4.12
   Compiling heapsize v0.4.2
   Compiling proc-macro2 v0.4.30
   Compiling log v0.4.22
   Compiling hashbrown v0.8.2
   Compiling eth-secp256k1 v0.5.7 (https://github.com/paritytech/rust-secp256k1?rev=9791e79f21a5309dcb6e0bd254b1ef88fca2f1f4#9791e79f)
   Compiling ctr v0.3.2
   Compiling pbkdf2 v0.3.0
   Compiling ahash v0.2.19
   Compiling local-encoding v0.2.0
   Compiling aho-corasick v1.1.3
   Compiling quick-error v1.2.2
   Compiling regex-syntax v0.8.5
   Compiling semver v0.9.0
   Compiling parity-scale-codec v1.3.5
   Compiling rustc_version v0.2.3
   Compiling impl-serde v0.3.1
   Compiling ethbloom v0.9.2
   Compiling unicode-xid v0.2.0
   Compiling unicode-xid v0.1.0
   Compiling ryu v1.0.2
   Compiling nodrop v0.1.14
   Compiling synstructure v0.12.2
   Compiling regex-automata v0.4.8
   Compiling parking_lot_core v0.6.2
   Compiling parking_lot_core v0.3.1
   Compiling secp256k1 v0.17.2
   Compiling impl-codec v0.4.2
   Compiling primitive-types v0.7.2
   Compiling scrypt v0.2.0
   Compiling aes-ctr v0.3.0
   Compiling aes v0.3.2
   Compiling parking_lot_core v0.7.2
   Compiling ethereum-types v0.9.2
   Compiling parity-rocksdb-sys v0.5.6
   Compiling ripemd160 v0.8.0
   Compiling block-modes v0.3.3
   Compiling tiny-keccak v1.5.0
   Compiling protobuf v2.16.2
   Compiling parking_lot_core v0.9.10
   Compiling stable_deref_trait v1.1.1
   Compiling zeroize v1.2.0
   Compiling ahash v0.3.8
   Compiling parity-util-mem v0.7.0
   Compiling subtle v2.3.0
   Compiling parity-bytes v0.1.2
   Compiling thiserror v1.0.64
   Compiling serde_json v1.0.128
   Compiling parity-crypto v0.6.2
   Compiling owning_ref v0.4.0
   Compiling regex v1.11.0
   Compiling parking_lot v0.10.2
   Compiling elastic-array v0.10.2
   Compiling lru v0.5.3
   Compiling parity-util-mem-derive v0.1.0
   Compiling quote v0.6.13
   Compiling parking_lot v0.9.0
   Compiling impl-trait-for-tuples v0.1.3
   Compiling thiserror-impl v1.0.64
   Compiling parity-wordlist v1.3.0
   Compiling edit-distance v2.1.0
   Compiling itoa v1.0.11
   Compiling memzero v0.1.0 (/home/parallels/Projects/openethereum/crates/util/memzero)
   Compiling rustc-hex v1.0.0
   Compiling scopeguard v0.3.3
   Compiling prometheus v0.9.0
   Compiling lock_api v0.1.5
   Compiling ethkey v0.3.0 (/home/parallels/Projects/openethereum/crates/accounts/ethkey)
   Compiling syn v0.15.26
   Compiling kvdb v0.1.1
   Compiling lock_api v0.4.6
   Compiling fnv v1.0.6
   Compiling adler32 v1.2.0
   Compiling spin v0.5.2
   Compiling rlp_derive v0.1.0 (/home/parallels/Projects/openethereum/crates/util/rlp-derive)
   Compiling inflate v0.4.5
   Compiling parking_lot v0.12.3
   Compiling parking_lot v0.6.4
   Compiling keccak-hash v0.5.1
   Compiling serde_repr v0.1.6
   Compiling plain_hasher v0.2.2
   Compiling fs-swap v0.2.6
   Compiling num_cpus v1.16.0
   Compiling interleaved-ordered v0.1.1
   Compiling hex v0.4.3
   Compiling unexpected v0.1.0 (/home/parallels/Projects/openethereum/crates/util/unexpected)
   Compiling hash-db v0.11.0
   Compiling common-types v0.1.0 (/home/parallels/Projects/openethereum/crates/ethcore/types)
   Compiling stats v0.1.0 (/home/parallels/Projects/openethereum/crates/util/stats)
   Compiling kvdb-memorydb v0.1.0
   Compiling gimli v0.31.1
   Compiling adler2 v2.0.0
   Compiling miniz_oxide v0.8.0
   Compiling memory-db v0.11.0 (/home/parallels/Projects/openethereum/crates/db/memory-db)
   Compiling keccak-hasher v0.1.1 (/home/parallels/Projects/openethereum/crates/util/keccak-hasher)
   Compiling fastmap v0.1.0 (/home/parallels/Projects/openethereum/crates/util/fastmap)
   Compiling object v0.36.5
   Compiling addr2line v0.24.2
   Compiling rustc-demangle v0.1.24
   Compiling xdg v2.2.0
   Compiling app_dirs v1.2.1 (https://github.com/openethereum/app-dirs-rs#0b37f948)
   Compiling humantime v1.3.0
   Compiling itertools v0.5.10
   Compiling time v0.1.42
   Compiling backtrace v0.3.74
   Compiling atty v0.2.14
   Compiling home v0.3.4
   Compiling termcolor v1.0.5
   Compiling strsim v0.10.0
   Compiling docopt v1.1.1
   Compiling env_logger v0.5.13
   Compiling panic_hook v0.1.0 (/home/parallels/Projects/openethereum/crates/util/panic-hook)
   Compiling ethstore v0.2.1 (/home/parallels/Projects/openethereum/crates/accounts/ethstore)
   Compiling parity-rocksdb v0.5.1
   Compiling kvdb-rocksdb v0.1.6
   Compiling ethcore-db v0.1.0 (/home/parallels/Projects/openethereum/crates/db/db)
   Compiling journaldb v0.2.0 (/home/parallels/Projects/openethereum/crates/db/journaldb)
   Compiling dir v0.1.2 (/home/parallels/Projects/openethereum/crates/util/dir)
   Compiling ethstore-cli v0.1.1 (/home/parallels/Projects/openethereum/bin/ethstore)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 27s
warning: the following packages contain code that will be rejected by a future version of Rust: protobuf v2.16.2
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 2`

real    1m27.241s
user    4m47.199s
sys     0m28.103s
   Compiling threadpool v1.7.1
   Compiling ethkey-cli v0.1.0 (/home/parallels/Projects/openethereum/bin/ethkey)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.35s

real    0m1.414s
user    0m0.789s
sys     0m0.312s
_____ Post-processing binaries _____
'target/debug/ethstore' -> '.artifacts/ethstore'
'target/debug/ethkey' -> '.artifacts/ethkey'
```

