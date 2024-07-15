### 1. Build Release

```text
/home/circleci/.cargo/bin/cargo build --color=always --release --features final
    Updating crates.io index
    Updating git repository `https://github.com/paritytech/rust-secp256k1`
    Updating git repository `https://github.com/openethereum/app-dirs-rs`
    Updating git repository `https://github.com/rimrakhimov/ethabi`
    Updating git repository `https://github.com/gnosis/reth.git`
    Updating git repository `https://github.com/paritytech/bn`
    Updating git repository `https://github.com/matter-labs/eip1962.git`
    Updating git repository `https://github.com/paritytech/rust-ctrlc.git`
 Downloading crates ...
  Downloaded adler v1.0.2
  Downloaded addr2line v0.14.1
  Downloaded adler32 v1.2.0
  Downloaded aes v0.3.2
  Downloaded home v0.5.1
  Downloaded tokio-codec v0.1.1
  Downloaded tokio-fs v0.1.6
  Downloaded atty v0.2.13
  Downloaded tokio-current-thread v0.1.6
  Downloaded tokio-named-pipes v0.1.0
  Downloaded block-cipher-trait v0.6.2
  Downloaded tokio-rustls v0.9.4
  Downloaded tokio-retry v0.1.1
  Downloaded build_const v0.2.1
  Downloaded block-padding v0.1.4
  Downloaded byte-tools v0.3.1
  Downloaded const-random v0.1.13
  Downloaded crunchy v0.2.2
  Downloaded block-buffer v0.7.3
  Downloaded triehash v0.5.0
  Downloaded transient-hashmap v0.4.1
  Downloaded try-lock v0.2.2
  Downloaded edit-distance v2.1.0
  Downloaded elastic-array v0.10.2
  Downloaded ethbloom v0.9.2
  Downloaded trace-time v0.1.2
  Downloaded ethereum-types v0.9.2
  Downloaded untrusted v0.6.2
  Downloaded const-random-macro v0.1.13
  Downloaded failure_derive v0.1.8
  Downloaded fdlimit v0.1.1
  Downloaded try-lock v0.1.0
  Downloaded crunchy v0.1.6
  Downloaded validator v0.8.0
  Downloaded enum_primitive v0.1.1
  Downloaded hash-db v0.11.0
  Downloaded ethabi-contract v11.0.0
  Downloaded want v0.0.4
  Downloaded want v0.2.0
  Downloaded fs-swap v0.2.4
  Downloaded winapi-build v0.1.1
  Downloaded http-body v0.1.0
  Downloaded home v0.3.4
  Downloaded xmltree v0.7.0
  Downloaded fake-simd v0.1.2
  Downloaded impl-rlp v0.2.1
  Downloaded if_chain v0.1.3
  Downloaded interleaved-ordered v0.1.1
  Downloaded impl-codec v0.4.2
  Downloaded parity-bytes v0.1.1
  Downloaded primal-check v0.2.3
  Downloaded instant v0.1.9
  Downloaded target_info v0.1.0
  Downloaded string v0.2.1
  Downloaded keccak-hash v0.5.1
  Downloaded kvdb v0.1.1
  Downloaded ansi_term v0.11.0
  Downloaded ahash v0.2.19
  Downloaded autocfg v1.0.0
  Downloaded aes-ctr v0.3.0
  Downloaded ansi_term v0.10.2
  Downloaded matches v0.1.8
  Downloaded kvdb-memorydb v0.1.0
  Downloaded bitflags v0.7.0
  Downloaded bit-set v0.4.0
  Downloaded number_prefix v0.2.8
  Downloaded cfg-if v1.0.0
  Downloaded tokio-executor v0.1.10
  Downloaded bitflags v1.2.1
  Downloaded crc v1.8.1
  Downloaded lazy_static v1.4.0
  Downloaded crossbeam-deque v0.6.3
  Downloaded block-modes v0.3.3
  Downloaded crossbeam-queue v0.1.2
  Downloaded bit-vec v0.4.4
  Downloaded skeptic v0.4.0
  Downloaded tokio-buf v0.1.1
  Downloaded tokio-uds v0.2.5
  Downloaded digest v0.8.1
  Downloaded cfg-if v0.1.10
  Downloaded byte-slice-cast v0.3.5
  Downloaded ctr v0.3.2
  Downloaded cmake v0.1.42
  Downloaded byteorder v1.3.2
  Downloaded fnv v1.0.6
  Downloaded tokio-reactor v0.1.12
  Downloaded crossbeam-deque v0.7.1
  Downloaded tokio-timer v0.1.2
  Downloaded getopts v0.2.21
  Downloaded tokio-tcp v0.1.3
  Downloaded tokio-service v0.1.0
  Downloaded ct-logs v0.5.1
  Downloaded crypto-mac v0.7.0
  Downloaded unicode-xid v0.1.0
  Downloaded tokio-udp v0.1.5
  Downloaded either v1.5.3
  Downloaded globset v0.4.5
  Downloaded utf8-ranges v1.0.4
  Downloaded fixed-hash v0.6.1
  Downloaded unicode-xid v0.2.0
  Downloaded generic-array v0.12.3
  Downloaded futures-cpupool v0.1.8
  Downloaded hex v0.4.3
  Downloaded vec_map v0.8.1
  Downloaded unicode-width v0.1.6
  Downloaded hmac v0.7.1
  Downloaded hamming v0.1.3
  Downloaded walkdir v2.3.1
  Downloaded version_check v0.1.5
  Downloaded heapsize v0.4.2
  Downloaded vergen v0.1.1
  Downloaded arrayvec v0.4.12
  Downloaded xdg v2.2.0
  Downloaded humantime v1.3.0
  Downloaded zeroize v1.2.0
  Downloaded thread_local v1.0.1
  Downloaded termcolor v1.0.5
  Downloaded tokio-sync v0.1.7
  Downloaded rustc-hex v1.0.0
  Downloaded hyper-rustls v0.16.1
  Downloaded memoffset v0.5.2
  Downloaded httparse v1.3.4
  Downloaded impl-trait-for-tuples v0.1.3
  Downloaded igd v0.7.1
  Downloaded inflate v0.4.5
  Downloaded ipnetwork v0.12.8
  Downloaded jsonrpc-core v15.0.0
  Downloaded rustc-demangle v0.1.16
  Downloaded ripemd160 v0.8.0
  Downloaded siphasher v0.1.3
  Downloaded jsonrpc-pubsub v15.0.0
  Downloaded tiny-keccak v2.0.2
  Downloaded autocfg v0.1.7
  Downloaded jsonrpc-tcp-server v15.0.0
  Downloaded itoa v0.4.4
  Downloaded aho-corasick v0.6.10
  Downloaded lazycell v1.2.1
  Downloaded language-tags v0.2.2
  Downloaded kvdb-rocksdb v0.1.6
  Downloaded jsonrpc-server-utils v15.0.0
  Downloaded jsonrpc-ipc-server v15.0.0
  Downloaded arrayvec v0.5.1
  Downloaded base64 v0.9.3
  Downloaded iovec v0.1.4
  Downloaded beef v0.5.1
  Downloaded kernel32-sys v0.2.2
  Downloaded mime v0.3.14
  Downloaded crossbeam-utils v0.8.6
  Downloaded c2-chacha v0.2.3
  Downloaded rand_core v0.5.1
  Downloaded rand_core v0.4.2
  Downloaded primal-bit v0.2.4
  Downloaded opaque-debug v0.2.3
  Downloaded ethabi v12.0.0
  Downloaded crossbeam-utils v0.7.2
  Downloaded env_logger v0.5.13
  Downloaded ethereum-forkid v0.2.1
  Downloaded crossbeam-queue v0.2.3
  Downloaded gcc v0.3.55
  Downloaded getrandom v0.2.2
  Downloaded error-chain v0.12.1
  Downloaded docopt v1.1.0
  Downloaded failure v0.1.8
  Downloaded getrandom v0.1.13
  Downloaded tokio-timer v0.2.13
  Downloaded unicase v2.5.1
  Downloaded scrypt v0.2.0
  Downloaded ucd-util v0.1.8
  Downloaded thiserror v1.0.20
  Downloaded unicode-bidi v0.3.4
  Downloaded trie-db v0.11.0
  Downloaded uint v0.8.5
  Downloaded typenum v1.11.2
  Downloaded proc-macro-crate v0.1.4
  Downloaded validator_derive v0.8.0
  Downloaded maybe-uninit v2.0.0
  Downloaded parity-snappy v0.1.0
  Downloaded net2 v0.2.33
  Downloaded miow v0.3.7
  Downloaded quote v0.6.13
  Downloaded sha2 v0.8.0
  Downloaded sha-1 v0.8.1
  Downloaded timer v0.2.0
  Downloaded memchr v2.2.1
  Downloaded maplit v1.0.2
  Downloaded lock_api v0.4.3
  Downloaded rustc_version v0.2.3
  Downloaded owning_ref v0.4.0
  Downloaded smallvec v0.6.13
  Downloaded radium v0.3.0
  Downloaded proc-macro-hack v0.5.19
  Downloaded relay v0.1.1
  Downloaded plain_hasher v0.2.2
  Downloaded ahash v0.3.8
  Downloaded rlp-derive v0.1.0
  Downloaded proc-macro2 v0.4.30
  Downloaded quick-error v1.2.2
  Downloaded memmap v0.6.2
  Downloaded slab v0.2.0
  Downloaded pbkdf2 v0.3.0
  Downloaded parity-tokio-ipc v0.4.0
  Downloaded parity-daemonize v0.3.0
  Downloaded rand_xorshift v0.2.0
  Downloaded rand_chacha v0.2.1
  Downloaded primal-estimate v0.2.1
  Downloaded parity-path v0.1.2
  Downloaded base64 v0.10.1
  Downloaded crossbeam-epoch v0.7.2
  Downloaded cc v1.0.41
  Downloaded bytes v0.4.12
  Downloaded tempdir v0.3.7
  Downloaded strsim v0.9.2
  Downloaded crossbeam-utils v0.6.6
  Downloaded jsonrpc-derive v15.0.0
  Downloaded heck v0.3.1
  Downloaded toml v0.4.10
  Downloaded tokio-io v0.1.12
  Downloaded parity-scale-codec v1.3.5
  Downloaded derive_more v0.99.9
  Downloaded rprompt v1.0.3
  Downloaded slab v0.3.0
  Downloaded strsim v0.8.0
  Downloaded nan-preserving-float v0.1.0
  Downloaded parity-util-mem-derive v0.1.0
  Downloaded primitive-types v0.7.2
  Downloaded tokio-threadpool v0.1.18
  Downloaded toml v0.5.5
  Downloaded rand_pcg v0.1.2
  Downloaded order-stat v0.1.3
  Downloaded num v0.1.42
  Downloaded ppv-lite86 v0.2.6
  Downloaded subtle v2.3.0
  Downloaded serde_repr v0.1.6
  Downloaded rlp v0.4.6
  Downloaded rand_hc v0.1.0
  Downloaded textwrap v0.11.0
  Downloaded time v0.1.42
  Downloaded same-file v1.0.5
  Downloaded lock_api v0.3.4
  Downloaded local-encoding v0.2.0
  Downloaded parking_lot_core v0.7.2
  Downloaded scoped-tls v0.1.2
  Downloaded rand v0.3.23
  Downloaded lru-cache v0.1.2
  Downloaded memory_units v0.3.0
  Downloaded primal v0.2.3
  Downloaded parity-wasm v0.31.3
  Downloaded primal-sieve v0.2.9
  Downloaded ryu v1.0.2
  Downloaded subtle v1.0.0
  Downloaded stable_deref_trait v1.1.1
  Downloaded parking_lot v0.9.0
  Downloaded num-traits v0.2.8
  Downloaded parking_lot_core v0.8.3
  Downloaded rand_jitter v0.1.4
  Downloaded lock_api v0.1.5
  Downloaded percent-encoding v1.0.1
  Downloaded stream-cipher v0.3.2
  Downloaded term_size v0.3.1
  Downloaded rpassword v1.0.2
  Downloaded hashbrown v0.6.3
  Downloaded sct v0.5.0
  Downloaded aes-soft v0.3.3
  Downloaded rand_xorshift v0.1.1
  Downloaded num-integer v0.1.41
  Downloaded linked-hash-map v0.5.3
  Downloaded safemem v0.3.3
  Downloaded rand_isaac v0.1.1
  Downloaded rand_chacha v0.1.1
  Downloaded static_assertions v1.1.0
  Downloaded rustc-hex v2.1.0
  Downloaded remove_dir_all v0.5.2
  Downloaded parking_lot v0.6.4
  Downloaded slab v0.4.2
  Downloaded thiserror-impl v1.0.20
  Downloaded nodrop v0.1.14
  Downloaded logos v0.12.0
  Downloaded jsonrpc-http-server v15.0.0
  Downloaded log v0.4.8
  Downloaded jsonrpc-ws-server v15.0.0
  Downloaded num-traits v0.1.43
  Downloaded indexmap v1.3.0
  Downloaded rustc-serialize v0.3.24
  Downloaded thread_local v0.3.6
  Downloaded parity-wordlist v1.3.0
  Downloaded secp256k1 v0.17.2
  Downloaded rand_core v0.3.1
  Downloaded mio-extras v2.0.5
  Downloaded mio-named-pipes v0.1.6
  Downloaded spin v0.5.2
  Downloaded parity-rocksdb v0.5.1
  Downloaded pwasm-utils v0.6.2
  Downloaded mio-uds v0.6.7
  Downloaded lru v0.5.3
  Downloaded semver-parser v0.7.0
  Downloaded tiny-keccak v1.5.0
  Downloaded scopeguard v1.1.0
  Downloaded backtrace v0.3.56
  Downloaded hashbrown v0.8.2
  Downloaded url v2.1.0
  Downloaded wasmi v0.3.0
  Downloaded xml-rs v0.7.0
  Downloaded semver v0.9.0
  Downloaded textwrap v0.9.0
  Downloaded synstructure v0.12.2
  Downloaded parking_lot_core v0.6.2
  Downloaded scopeguard v0.3.3
  Downloaded parity-util-mem v0.7.0
  Downloaded parity-crypto v0.6.2
  Downloaded num_cpus v1.11.0
  Downloaded rand_os v0.1.3
  Downloaded num-bigint v0.1.44
  Downloaded webpki v0.19.1
  Downloaded tempfile v3.1.0
  Downloaded parking_lot_core v0.3.1
  Downloaded impl-serde v0.3.1
  Downloaded itertools v0.5.10
  Downloaded parity-ws v0.10.0
  Downloaded crossbeam-channel v0.5.2
  Downloaded once_cell v1.4.0
  Downloaded tokio-core v0.1.17
  Downloaded num-iter v0.1.39
  Downloaded logos-derive v0.12.0
  Downloaded quote v1.0.7
  Downloaded parking_lot v0.10.2
  Downloaded serde_derive v1.0.102
  Downloaded aho-corasick v0.7.6
  Downloaded url v1.7.2
  Downloaded unicode-normalization v0.1.8
  Downloaded percent-encoding v2.1.0
  Downloaded serde_json v1.0.41
  Downloaded smallvec v1.6.1
  Downloaded rayon-core v1.6.0
  Downloaded pulldown-cmark v0.0.3
  Downloaded serde v1.0.102
  Downloaded parking_lot v0.11.1
  Downloaded bitvec v0.17.4
  Downloaded num-bigint v0.2.3
  Downloaded proc-macro2 v1.0.36
  Downloaded http v0.1.21
  Downloaded mio v0.6.22
  Downloaded itertools v0.7.11
  Downloaded tokio v0.1.22
  Downloaded miniz_oxide v0.4.4
  Downloaded unicode-segmentation v1.5.0
  Downloaded prometheus v0.9.0
  Downloaded rand v0.4.6
  Downloaded rand v0.6.5
  Downloaded chrono v0.4.9
  Downloaded hyper v0.12.35
  Downloaded h2 v0.1.26
  Downloaded rand v0.7.3
  Downloaded futures v0.1.29
  Downloaded rand v0.5.6
  Downloaded protobuf v2.16.2
  Downloaded rayon v1.2.0
  Downloaded hyper v0.11.27
  Downloaded secp256k1-sys v0.1.2
  Downloaded webpki-roots v0.16.0
  Downloaded clap v2.33.0
  Downloaded regex v0.2.11
  Downloaded idna v0.2.0
  Downloaded idna v0.1.5
  Downloaded regex v1.3.9
  Downloaded syn v0.15.26
  Downloaded object v0.23.0
  Downloaded regex-syntax v0.5.6
  Downloaded syn v1.0.86
  Downloaded winapi v0.2.8
  Downloaded regex-syntax v0.6.18
  Downloaded bstr v0.2.8
  Downloaded rustls v0.15.2
  Downloaded libc v0.2.89
  Downloaded rust-crypto v0.2.36
  Downloaded winapi v0.3.8
  Downloaded parity-snappy-sys v0.1.2
  Downloaded gimli v0.23.0
  Downloaded parity-rocksdb-sys v0.5.6
  Downloaded ring v0.14.6
   Compiling libc v0.2.89
   Compiling proc-macro2 v1.0.36
   Compiling unicode-xid v0.2.0
   Compiling syn v1.0.86
   Compiling serde v1.0.102
   Compiling cfg-if v0.1.10
   Compiling byteorder v1.3.2
   Compiling lazy_static v1.4.0
   Compiling autocfg v0.1.7
   Compiling either v1.5.3
   Compiling log v0.4.8
   Compiling maybe-uninit v2.0.0
   Compiling autocfg v1.0.0
   Compiling scopeguard v1.1.0
   Compiling crunchy v0.2.2
   Compiling semver-parser v0.7.0
   Compiling getrandom v0.1.13
   Compiling arrayvec v0.4.12
   Compiling cc v1.0.41
   Compiling rustc-hex v2.1.0
   Compiling nodrop v0.1.14
   Compiling tiny-keccak v2.0.2
   Compiling futures v0.1.29
   Compiling ppv-lite86 v0.2.6
   Compiling static_assertions v1.1.0
   Compiling typenum v1.11.2
   Compiling rand_core v0.4.2
   Compiling fnv v1.0.6
   Compiling ryu v1.0.2
   Compiling smallvec v1.6.1
   Compiling slab v0.4.2
   Compiling radium v0.3.0
   Compiling arrayvec v0.5.1
   Compiling itoa v0.4.4
   Compiling byte-slice-cast v0.3.5
   Compiling memchr v2.2.1
   Compiling byte-tools v0.3.1
   Compiling opaque-debug v0.2.3
   Compiling cfg-if v1.0.0
   Compiling fake-simd v0.1.2
   Compiling proc-macro-hack v0.5.19
   Compiling getrandom v0.2.2
   Compiling safemem v0.3.3
   Compiling subtle v1.0.0
   Compiling proc-macro2 v0.4.30
   Compiling quick-error v1.2.2
   Compiling regex-syntax v0.6.18
   Compiling parity-bytes v0.1.1
   Compiling unicode-xid v0.1.0
   Compiling zeroize v1.2.0
   Compiling parity-util-mem v0.7.0
   Compiling subtle v2.3.0
   Compiling rustc-hex v1.0.0
   Compiling ahash v0.3.8
   Compiling memzero v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/memzero)
   Compiling version_check v0.1.5
   Compiling heapsize v0.4.2
   Compiling edit-distance v2.1.0
   Compiling spin v0.5.2
   Compiling adler32 v1.2.0
   Compiling unicode-width v0.1.6
   Compiling hex v0.4.3
   Compiling unexpected v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/unexpected)
   Compiling remove_dir_all v0.5.2
   Compiling httparse v1.3.4
   Compiling matches v0.1.8
   Compiling hash-db v0.11.0
   Compiling untrusted v0.6.2
   Compiling gimli v0.23.0
   Compiling adler v1.0.2
   Compiling protobuf v2.16.2
   Compiling stable_deref_trait v1.1.1
   Compiling object v0.23.0
   Compiling rustc-demangle v0.1.16
   Compiling try-lock v0.2.2
   Compiling prometheus v0.9.0
   Compiling scopeguard v0.3.3
   Compiling percent-encoding v2.1.0
   Compiling interleaved-ordered v0.1.1
   Compiling rustc-serialize v0.3.24
   Compiling bitflags v1.2.1
   Compiling hamming v0.1.3
   Compiling percent-encoding v1.0.1
   Compiling linked-hash-map v0.5.3
   Compiling primal-estimate v0.2.1
   Compiling rayon-core v1.6.0
   Compiling crunchy v0.1.6
   Compiling scoped-tls v0.1.2
   Compiling try-lock v0.1.0
   Compiling once_cell v1.4.0
   Compiling unicode-segmentation v1.5.0
   Compiling ansi_term v0.11.0
   Compiling crossbeam-utils v0.8.6
   Compiling gcc v0.3.55
   Compiling ucd-util v0.1.8
   Compiling regex v0.2.11
   Compiling macros v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/macros)
   Compiling ansi_term v0.10.2
   Compiling build_const v0.2.1
   Compiling mime v0.3.14
   Compiling language-tags v0.2.2
   Compiling bit-vec v0.4.4
   Compiling memory_units v0.3.0
   Compiling utf8-ranges v1.0.4
   Compiling slab v0.3.0
   Compiling winapi v0.3.8
   Compiling nan-preserving-float v0.1.0
   Compiling failure_derive v0.1.8
   Compiling siphasher v0.1.3
   Compiling ipnetwork v0.12.8
   Compiling home v0.5.1
   Compiling ethabi-contract v11.0.0
   Compiling maplit v1.0.2
   Compiling same-file v1.0.5
   Compiling lazycell v1.2.1
   Compiling using_queue v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/concensus/miner/using-queue)
   Compiling mio-named-pipes v0.1.6
   Compiling reth-util v0.1.0 (https://github.com/gnosis/reth.git?rev=573e128#573e1284)
   Compiling slab v0.2.0
   Compiling bitflags v0.7.0
   Compiling time-utils v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/time-utils)
   Compiling beef v0.5.1
   Compiling termcolor v1.0.5
   Compiling if_chain v0.1.3
   Compiling target_info v0.1.0
   Compiling winapi-build v0.1.1
   Compiling transient-hashmap v0.4.1
   Compiling order-stat v0.1.3
   Compiling winapi v0.2.8
   Compiling xdg v2.2.0
   Compiling rprompt v1.0.3
   Compiling home v0.3.4
   Compiling strsim v0.8.0
   Compiling strsim v0.9.2
   Compiling vec_map v0.8.1
   Compiling thread_local v1.0.1
   Compiling thread_local v0.3.6
   Compiling crossbeam-utils v0.6.6
   Compiling lock_api v0.3.4
   Compiling lock_api v0.4.3
   Compiling itertools v0.5.10
   Compiling itertools v0.7.11
   Compiling rlp v0.4.6
   Compiling eip-152 v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/EIP-152)
   Compiling crossbeam-utils v0.7.2
   Compiling hashbrown v0.8.2
   Compiling miniz_oxide v0.4.4
   Compiling cmake v0.1.42
   Compiling rand_pcg v0.1.2
   Compiling rand_chacha v0.1.1
   Compiling rand v0.6.5
   Compiling num-traits v0.2.8
   Compiling hashbrown v0.6.3
   Compiling num-integer v0.1.41
   Compiling indexmap v1.3.0
   Compiling num-bigint v0.2.3
   Compiling num-iter v0.1.39
   Compiling rand_core v0.3.1
   Compiling rand_jitter v0.1.4
   Compiling c2-chacha v0.2.3
   Compiling secp256k1-sys v0.1.2
   Compiling ring v0.14.6
   Compiling tokio-sync v0.1.7
   Compiling tokio-service v0.1.0
   Compiling relay v0.1.1
   Compiling bitvec v0.17.4
   Compiling eth-secp256k1 v0.5.7 (https://github.com/paritytech/rust-secp256k1?rev=9791e79f21a5309dcb6e0bd254b1ef88fca2f1f4#9791e79f)
   Compiling block-padding v0.1.4
   Compiling instant v0.1.9
   Compiling humantime v1.3.0
   Compiling getopts v0.2.21
   Compiling inflate v0.4.5
   Compiling unicase v2.5.1
   Compiling error-chain v0.12.1
   Compiling unicode-bidi v0.3.4
   Compiling owning_ref v0.4.0
   Compiling primal-bit v0.2.4
   Compiling lru-cache v0.1.2
   Compiling regex-syntax v0.5.6
   Compiling heck v0.3.1
   Compiling addr2line v0.14.1
   Compiling crc v1.8.1
   Compiling bit-set v0.4.0
   Compiling rust-crypto v0.2.36
   Compiling tokio-timer v0.1.2
   Compiling parity-path v0.1.2
   Compiling ethcore-bloom-journal v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/db/bloom)
   Compiling walkdir v2.3.1
   Compiling textwrap v0.11.0
   Compiling textwrap v0.9.0
   Compiling app_dirs v1.2.1 (https://github.com/openethereum/app-dirs-rs#0b37f948)
   Compiling kernel32-sys v0.2.2
   Compiling crossbeam-queue v0.1.2
   Compiling impl-rlp v0.2.1
   Compiling triehash v0.5.0
   Compiling parity-snappy-sys v0.1.2
   Compiling parity-rocksdb-sys v0.5.6
   Compiling rand_xorshift v0.1.1
   Compiling rand_isaac v0.1.1
   Compiling rand_hc v0.1.0
   Compiling pulldown-cmark v0.0.3
   Compiling lock_api v0.1.5
   Compiling tiny-keccak v1.5.0
   Compiling plain_hasher v0.2.2
   Compiling want v0.2.0
   Compiling trace-time v0.1.2
   Compiling want v0.0.4
   Compiling smallvec v0.6.13
   Compiling uint v0.8.5
   Compiling base64 v0.9.3
   Compiling base64 v0.10.1
   Compiling simple_uint v0.1.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
   Compiling parity-wasm v0.31.3
   Compiling iovec v0.1.4
   Compiling num_cpus v1.11.0
   Compiling net2 v0.2.33
   Compiling parking_lot_core v0.7.2
   Compiling rand_os v0.1.3
   Compiling rand v0.5.6
   Compiling time v0.1.42
   Compiling parking_lot_core v0.8.3
   Compiling rand v0.4.6
   Compiling fs-swap v0.2.4
   Compiling memmap v0.6.2
   Compiling atty v0.2.13
   Compiling rpassword v1.0.2
   Compiling term_size v0.3.1
   Compiling fdlimit v0.1.1
   Compiling quote v1.0.7
   Compiling aho-corasick v0.7.6
   Compiling bstr v0.2.8
   Compiling aho-corasick v0.6.10
   Compiling xml-rs v0.7.0
   Compiling elastic-array v0.10.2
   Compiling quote v0.6.13
   Compiling miow v0.3.7
   Compiling crossbeam-channel v0.5.2
   Compiling unicode-normalization v0.1.8
   Compiling primal-sieve v0.2.9
   Compiling txpool v1.0.0-alpha (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/transaction-pool)
   Compiling backtrace v0.3.56
   Compiling fixed_width_group_and_loop v0.1.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
   Compiling fixed_width_field v0.1.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
   Compiling rand_core v0.5.1
   Compiling num-traits v0.1.43
   Compiling number_prefix v0.2.8
   Compiling bytes v0.4.12
   Compiling futures-cpupool v0.1.8
   Compiling mio v0.6.22
   Compiling parking_lot v0.10.2
   Compiling parking_lot v0.11.1
   Compiling pwasm-utils v0.6.2
   Compiling wasmi v0.3.0
   Compiling clap v2.33.0
   Compiling bn v0.4.4 (https://github.com/paritytech/bn#6079255e)
   Compiling rand v0.3.23
   Compiling tempdir v0.3.7
   Compiling tokio-executor v0.1.10
   Compiling crossbeam-queue v0.2.3
   Compiling vergen v0.1.1
   Compiling kvdb v0.1.1
   Compiling rlp_compress v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/rlp-compress)
   Compiling syn v0.15.26
   Compiling regex v1.3.9
   Compiling xmltree v0.7.0
   Compiling ctrlc v1.1.1 (https://github.com/paritytech/rust-ctrlc.git#b5230171)
   Compiling panic_hook v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/panic-hook)
   Compiling rand_chacha v0.2.1
   Compiling rand_xorshift v0.2.0
   Compiling idna v0.2.0
   Compiling idna v0.1.5
   Compiling enum_primitive v0.1.1
   Compiling chrono v0.4.9
   Compiling num-bigint v0.1.44
   Compiling primal-check v0.2.3
   Compiling const-random-macro v0.1.13
   Compiling tokio-io v0.1.12
   Compiling http v0.1.21
   Compiling tokio-buf v0.1.1
   Compiling string v0.2.1
   Compiling len-caching-lock v0.1.1 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/len-caching-lock)
   Compiling mio-uds v0.6.7
   Compiling mio-extras v2.0.5
   Compiling parity-wordlist v1.3.0
   Compiling trie-db v0.11.0
   Compiling tokio-current-thread v0.1.6
   Compiling tokio-timer v0.2.13
   Compiling secp256k1 v0.17.2
   Compiling skeptic v0.4.0
   Compiling rand v0.7.3
   Compiling globset v0.4.5
   Compiling env_logger v0.5.13
   Compiling url v2.1.0
   Compiling url v1.7.2
   Compiling timer v0.2.0
   Compiling num v0.1.42
   Compiling primal v0.2.3
   Compiling synstructure v0.12.2
   Compiling tokio-codec v0.1.1
   Compiling webpki v0.19.1
   Compiling sct v0.5.0
   Compiling rlp_derive v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/rlp-derive)
   Compiling http-body v0.1.0
   Compiling h2 v0.1.26
   Compiling ethcore-logger v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/bin/oe/logger)
   Compiling local-encoding v0.2.0
   Compiling fixed-hash v0.6.1
   Compiling tempfile v3.1.0
   Compiling const-random v0.1.13
   Compiling ct-logs v0.5.1
   Compiling rustls v0.15.2
   Compiling webpki-roots v0.16.0
   Compiling generic-array v0.12.3
   Compiling serde_derive v1.0.102
   Compiling impl-trait-for-tuples v0.1.3
   Compiling serde_repr v0.1.6
   Compiling thiserror-impl v1.0.20
   Compiling eth_pairings_repr_derive v0.2.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
   Compiling derive_more v0.99.9
   Compiling rlp-derive v0.1.0
   Compiling logos-derive v0.12.0
   Compiling parity-util-mem-derive v0.1.0
   Compiling ahash v0.2.19
   Compiling digest v0.8.1
   Compiling block-buffer v0.7.3
   Compiling block-cipher-trait v0.6.2
   Compiling crypto-mac v0.7.0
   Compiling stream-cipher v0.3.2
   Compiling tokio-rustls v0.9.4
   Compiling aes-soft v0.3.3
   Compiling block-modes v0.3.3
   Compiling sha2 v0.8.0
   Compiling ripemd160 v0.8.0
   Compiling sha-1 v0.8.1
   Compiling hmac v0.7.1
   Compiling ctr v0.3.2
   Compiling parity-snappy v0.1.0
   Compiling lru v0.5.3
   Compiling aes v0.3.2
   Compiling parity-ws v0.10.0
   Compiling pbkdf2 v0.3.0
   Compiling aes-ctr v0.3.0
   Compiling scrypt v0.2.0
   Compiling eth_pairings v0.6.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
   Compiling thiserror v1.0.20
   Compiling failure v0.1.8
   Compiling stats v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/stats)
   Compiling parity-daemonize v0.3.0
   Compiling logos v0.12.0
   Compiling impl-serde v0.3.1
   Compiling parity-scale-codec v1.3.5
   Compiling serde_json v1.0.41
   Compiling semver v0.9.0
   Compiling oe-rpc-common v0.0.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/rpc-common)
   Compiling toml v0.4.10
   Compiling docopt v1.1.0
   Compiling toml v0.5.5
   Compiling ethbloom v0.9.2
   Compiling jsonrpc-core v15.0.0
   Compiling validator v0.8.0
   Compiling rustc_version v0.2.3
   Compiling blooms-db v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/db/blooms-db)
   Compiling proc-macro-crate v0.1.4
   Compiling impl-codec v0.4.2
   Compiling jsonrpc-pubsub v15.0.0
   Compiling parking_lot_core v0.6.2
   Compiling parking_lot v0.9.0
   Compiling memoffset v0.5.2
   Compiling parking_lot_core v0.3.1
   Compiling hyper v0.12.35
   Compiling parity-version v3.3.5 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/version)
   Compiling primitive-types v0.7.2
   Compiling jsonrpc-derive v15.0.0
   Compiling validator_derive v0.8.0
   Compiling ethereum-types v0.9.2
   Compiling keccak-hash v0.5.1
   Compiling parking_lot v0.6.4
   Compiling crossbeam-epoch v0.7.2
   Compiling parity-crypto v0.6.2
   Compiling keccak-hasher v0.1.1 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/keccak-hasher)
   Compiling ethabi v12.0.0
   Compiling fastmap v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/fastmap)
   Compiling ethash v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/concensus/ethash)
   Compiling tokio-reactor v0.1.12
   Compiling kvdb-memorydb v0.1.0
   Compiling ethabi v11.0.0 (https://github.com/rimrakhimov/ethabi?branch=rimrakhimov/remove-syn-export-span#222e6482)
   Compiling crossbeam-deque v0.7.1
   Compiling crossbeam-deque v0.6.3
   Compiling patricia-trie-ethereum v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/db/patricia-trie-ethereum)
   Compiling triehash-ethereum v0.2.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/triehash-ethereum)
   Compiling ethkey v0.3.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/accounts/ethkey)
   Compiling memory-db v0.11.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/db/memory-db)
   Compiling memory-cache v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/memory-cache)
   Compiling ethereum-forkid v0.2.1
   Compiling tokio-tcp v0.1.3
   Compiling tokio-udp v0.1.5
   Compiling tokio-uds v0.2.5
   Compiling tokio-threadpool v0.1.18
   Compiling common-types v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/ethcore/types)
   Compiling ethstore v0.2.1 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/accounts/ethstore)
   Compiling ethabi-derive v11.0.0 (https://github.com/rimrakhimov/ethabi?branch=rimrakhimov/remove-syn-export-span#222e6482)
   Compiling rayon v1.2.0
   Compiling tokio-fs v0.1.6
   Compiling tokio v0.1.22
   Compiling ethjson v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/ethjson)
   Compiling ethcore-call-contract v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/vm/call-contract)
   Compiling ethcore-accounts v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/accounts)
   Compiling jsonrpc-server-utils v15.0.0
   Compiling tokio-core v0.1.17
   Compiling ethcore-io v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/runtime/io)
   Compiling parity-runtime v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/runtime/runtime)
   Compiling tokio-named-pipes v0.1.0
   Compiling jsonrpc-tcp-server v15.0.0
   Compiling jsonrpc-ws-server v15.0.0
   Compiling tokio-retry v0.1.1
   Compiling hyper v0.11.27
   Compiling hyper-rustls v0.16.1
   Compiling jsonrpc-http-server v15.0.0
   Compiling vm v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/vm/vm)
   Compiling ethcore-network v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/net/network)
   Compiling parity-tokio-ipc v0.4.0
   Compiling ethcore-stratum v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/concensus/miner/stratum)
   Compiling fetch v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/net/fetch)
   Compiling wasm v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/vm/wasm)
   Compiling jsonrpc-ipc-server v15.0.0
   Compiling igd v0.7.1
   Compiling price-info v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/concensus/miner/price-info)
   Compiling oe-rpc-servers v0.0.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/rpc-servers)
   Compiling ethcore-network-devp2p v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/net/network-devp2p)
warning: unused borrow that must be used
-->    crates/net/network-devp2p/src/connection.rs:437:9
|    
437|          &mut packet[..HEADER_LEN].copy_from_slice(&mut header);
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^the borrow produces a value             
|    
= note    : `#[warn(unused_must_use)]` on by default
help: use `let _ = ...` to ignore the resulting value
|    
437| let _ =          &mut packet[..HEADER_LEN].copy_from_slice(&mut header);
| +++++++            

warning: unused borrow that must be used
-->    crates/net/network-devp2p/src/connection.rs:447:9
|    
447|          &mut packet[32..32 + len].copy_from_slice(payload);
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^the borrow produces a value             
|    
help: use `let _ = ...` to ignore the resulting value
|    
447| let _ =          &mut packet[32..32 + len].copy_from_slice(payload);
| +++++++            

warning: unused borrow that must be used
-->    crates/net/network-devp2p/src/connection.rs:529:9
|    
529|          &mut enc[..].copy_from_slice(prev.as_bytes());
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^the borrow produces a value             
|    
help: use `let _ = ...` to ignore the resulting value
|    
529| let _ =          &mut enc[..].copy_from_slice(prev.as_bytes());
| +++++++            

   Compiling ethcore-builtin v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/vm/builtin)
   Compiling evm v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/vm/evm)
warning: anonymous parameters are deprecated and will be removed in the next edition
-->   crates/vm/evm/src/interpreter/memory.rs:36:45
|   
36|      fn write_slice(&mut self, offset: U256, &[u8]);
| ^^^^^help: try naming the parameter or explicitly ignoring it: `_: &[u8]`                                                
|   
= note   : `#[warn(anonymous_parameters)]` on by default
= warning   : this is accepted in the current edition (Rust 2015) but is a hard error in Rust 2018!
= note   : for more information, see issue #41686 <https://github.com/rust-lang/rust/issues/41686>

warning: field is never read: `code_address`
-->    crates/vm/evm/src/interpreter/mod.rs:128:5
|    
128|      pub code_address: Address,
| ^^^^^^^^^^^^^^^^^^^^^^^^^        
|    
= note    : `#[warn(dead_code)]` on by default

warning: field is never read: `call_type`
-->    crates/vm/evm/src/interpreter/mod.rs:147:5
|    
147|      pub call_type: CallType,
| ^^^^^^^^^^^^^^^^^^^^^^^        

warning: field is never read: `params_type`
-->    crates/vm/evm/src/interpreter/mod.rs:149:5
|    
149|      pub params_type: ParamsType,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^        

warning: panic message is not a string literal
-->    crates/vm/evm/src/interpreter/mod.rs:835:33
|    
835|                        _ => panic!(format!(
| _________________________________^     
836| |                          "Unexpected instruction {:?} in CALL branch.",
837| |                          instruction
838| |                      )),
| |_____________________^    
|    
= note    : `#[warn(non_fmt_panics)]` on by default
= note    : this usage of panic!() is deprecated; it will be a hard error in Rust 2021
= note    : for more information, see <https://doc.rust-lang.org/nightly/edition-guide/rust-2021/panic-macro-consistency.html>
= note    : the panic!() macro supports formatting, so there's no need for the format!() macro here
help: remove the `format!(..)` macro call
|    
835~                      _ => panic!(
836|                          "Unexpected instruction {:?} in CALL branch.",
837|                          instruction
838~                      ),
|    

warning: `ethcore-network-devp2p` (lib) generated 3 warnings
   Compiling eip-712 v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/EIP-712)
warning: derive helper attribute is used before it is introduced
-->   crates/util/EIP-712/src/eip712.rs:33:3
|   
33|  #[serde(rename_all = "camelCase")]
| ^^^^^     
34|  #[serde(deny_unknown_fields)]
35|  #[derive(Deserialize, Serialize, Validate, Debug, Clone)]
| -----------the attribute is introduced here             
|   
= note   : `#[warn(legacy_derive_helpers)]` on by default
= warning   : this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
= note   : for more information, see issue #79202 <https://github.com/rust-lang/rust/issues/79202>

warning: derive helper attribute is used before it is introduced
-->   crates/util/EIP-712/src/eip712.rs:45:3
|   
45|  #[serde(rename_all = "camelCase")]
| ^^^^^     
46|  #[serde(deny_unknown_fields)]
47|  #[derive(Deserialize, Debug, Clone)]
| -----------the attribute is introduced here             
|   
= warning   : this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
= note   : for more information, see issue #79202 <https://github.com/rust-lang/rust/issues/79202>

warning: derive helper attribute is used before it is introduced
-->   crates/util/EIP-712/src/eip712.rs:34:3
|   
34|  #[serde(deny_unknown_fields)]
| ^^^^^     
35|  #[derive(Deserialize, Serialize, Validate, Debug, Clone)]
| -----------the attribute is introduced here             
|   
= warning   : this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
= note   : for more information, see issue #79202 <https://github.com/rust-lang/rust/issues/79202>

warning: derive helper attribute is used before it is introduced
-->   crates/util/EIP-712/src/eip712.rs:46:3
|   
46|  #[serde(deny_unknown_fields)]
| ^^^^^     
47|  #[derive(Deserialize, Debug, Clone)]
| -----------the attribute is introduced here             
|   
= warning   : this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
= note   : for more information, see issue #79202 <https://github.com/rust-lang/rust/issues/79202>

warning: `evm` (lib) generated 5 warnings
warning: `eip-712` (lib) generated 4 warnings
   Compiling ethcore-miner v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/concensus/miner)
warning: anonymous parameters are deprecated and will be removed in the next edition
-->   crates/concensus/miner/src/local_accounts.rs:26:24
|   
26|      fn is_local(&self, &Address) -> bool;
| ^^^^^^^^help: try naming the parameter or explicitly ignoring it: `_: &Address`                           
|   
= note   : `#[warn(anonymous_parameters)]` on by default
= warning   : this is accepted in the current edition (Rust 2015) but is a hard error in Rust 2018!
= note   : for more information, see issue #41686 <https://github.com/rust-lang/rust/issues/41686>

warning: `ethcore-miner` (lib) generated 1 warning
   Compiling parity-rocksdb v0.5.1
   Compiling kvdb-rocksdb v0.1.6
   Compiling ethcore-db v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/db/db)
   Compiling migration-rocksdb v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/db/migration-rocksdb)
   Compiling journaldb v0.2.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/db/journaldb)
   Compiling ethcore-blockchain v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/ethcore/blockchain)
   Compiling parity-local-store v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/concensus/miner/local-store)
   Compiling dir v0.1.2 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/dir)
   Compiling ethcore v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/ethcore)
warning: missing documentation for an associated function
-->     crates/ethcore/src/spec/spec.rs:1194:5
|     
1194|      pub fn new_test_round_rewrite_bytecode_transitions() -> Self {
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^         
|     
note: the lint level is defined here
-->     crates/ethcore/src/lib.rs:17:9
|     
17|    #![warn(missing_docs, unused_extern_crates)]
| ^^^^^^^^^^^^             

warning: panic message is not a string literal
-->    crates/ethcore/src/externalities.rs:237:25
|    
237| /                          format!(
238| |                              "Inconsistent env_info, should contain at least {:?} last hashes",
239| |                              index + 1
240| |                          )
| |_________________________^    
|    
= note    : `#[warn(non_fmt_panics)]` on by default
= note    : this usage of assert!() is deprecated; it will be a hard error in Rust 2021
= note    : for more information, see <https://doc.rust-lang.org/nightly/edition-guide/rust-2021/panic-macro-consistency.html>
= note    : the assert!() macro supports formatting, so there's no need for the format!() macro here
help: remove the `format!(..)` macro call
|    
237~                          
238|                              "Inconsistent env_info, should contain at least {:?} last hashes",
239|                              index + 1
240~                          
|    

warning: unused borrow that must be used
-->   crates/ethcore/src/executive.rs:67:13
|   
67|              &mut buffer[1..(1 + 20)].copy_from_slice(&sender[..]);
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^the borrow produces a value                
|   
= note   : `#[warn(unused_must_use)]` on by default
help: use `let _ = ...` to ignore the resulting value
|   
67| let _ =              &mut buffer[1..(1 + 20)].copy_from_slice(&sender[..]);
| +++++++               

warning: unused borrow that must be used
-->   crates/ethcore/src/executive.rs:68:13
|   
68|              &mut buffer[(1 + 20)..(1 + 20 + 32)].copy_from_slice(&salt[..]);
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^the borrow produces a value                
|   
help: use `let _ = ...` to ignore the resulting value
|   
68| let _ =              &mut buffer[(1 + 20)..(1 + 20 + 32)].copy_from_slice(&salt[..]);
| +++++++               

warning: unused borrow that must be used
-->   crates/ethcore/src/executive.rs:69:13
|   
69|              &mut buffer[(1 + 20 + 32)..].copy_from_slice(&code_hash[..]);
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^the borrow produces a value                
|   
help: use `let _ = ...` to ignore the resulting value
|   
69| let _ =              &mut buffer[(1 + 20 + 32)..].copy_from_slice(&code_hash[..]);
| +++++++               

warning: unused borrow that must be used
-->   crates/ethcore/src/executive.rs:75:13
|   
75|              &mut buffer[..20].copy_from_slice(&sender[..]);
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^the borrow produces a value                
|   
help: use `let _ = ...` to ignore the resulting value
|   
75| let _ =              &mut buffer[..20].copy_from_slice(&sender[..]);
| +++++++               

warning: unused borrow that must be used
-->   crates/ethcore/src/executive.rs:76:13
|   
76|              &mut buffer[20..].copy_from_slice(&code_hash[..]);
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^the borrow produces a value                
|   
help: use `let _ = ...` to ignore the resulting value
|   
76| let _ =              &mut buffer[20..].copy_from_slice(&code_hash[..]);
| +++++++               

   Compiling ethcore-sync v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/ethcore/sync)
   Compiling node-filter v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/net/node-filter)
warning: trailing semicolon in macro used in expression position
-->    crates/ethcore/sync/src/block_sync.rs:49:77
|    
49|           trace!(target: "sync", concat!("{:?}: ", $fmt), $self.block_set, $($arg)+);
| ^                                                                                      
...
321|                              trace_sync!(self, "Header already in chain {} ({})", number, hash)
| ------------------------------------------------------------------in this macro invocation                                 
|    
= note    : `#[warn(semicolon_in_expressions_from_macros)]` on by default
= warning    : this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
= note    : for more information, see issue #79813 <https://github.com/rust-lang/rust/issues/79813>
= note    : macro invocations at the end of a block are treated as expressions
= note    : to ignore the value produced by the macro, add a semicolon after the invocation of `trace_sync`
= note    : this warning originates in the macro `trace_sync` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: trailing semicolon in macro used in expression position
-->    crates/ethcore/sync/src/block_sync.rs:49:77
|    
49|             trace!(target: "sync", concat!("{:?}: ", $fmt), $self.block_set, $($arg)+);
| ^                                                                                        
...
323|                            _ => trace_sync!(
| ______________________________-     
324| |                              self,
325| |                              "Header already in chain {} ({}), state = {:?}",
326| |                              number,
327| |                              hash,
328| |                              self.state
329| |                          ),
| |_________________________-in this macro invocation     
|    
= warning    : this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
= note    : for more information, see issue #79813 <https://github.com/rust-lang/rust/issues/79813>
= note    : this warning originates in the macro `trace_sync` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: trailing semicolon in macro used in expression position
-->    crates/ethcore/sync/src/block_sync.rs:49:77
|    
49|           trace!(target: "sync", concat!("{:?}: ", $fmt), $self.block_set, $($arg)+);
| ^                                                                                      
...
414|              _ => trace_sync!(self, "Unexpected headers({})", headers.len()),
| ----------------------------------------------------------in this macro invocation                      
|    
= warning    : this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
= note    : for more information, see issue #79813 <https://github.com/rust-lang/rust/issues/79813>
= note    : this warning originates in the macro `trace_sync` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: field is never read: `client_version`
-->    crates/ethcore/sync/src/chain/mod.rs:370:5
|    
370|      client_version: ClientVersion,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^        
|    
= note    : `#[warn(dead_code)]` on by default

   Compiling parity-rpc v1.12.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/rpc)
   Compiling ethcore-service v0.1.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/ethcore/service)
warning: `ethcore-sync` (lib) generated 4 warnings
warning: function is never used: `serve`
-->   crates/rpc/src/tests/rpc.rs:23:4
|   
23|  fn serve(handler: Option<MetaIoHandler<Metadata>>) -> Server<HttpServer> {
| ^^^^^      
|   
= note   : `#[warn(dead_code)]` on by default

warning: function is never used: `request`
-->   crates/rpc/src/tests/rpc.rs:47:4
|   
47|  fn request(server: Server<HttpServer>, request: &str) -> http_client::Response {
| ^^^^^^^      

   Compiling parity-rpc-client v1.4.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/cli-signer/rpc-client)
   Compiling cli-signer v1.4.0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f/crates/util/cli-signer)
   Compiling openethereum v3.3.6-rc0 (/tmp/9caedcec-e886-4e2f-b074-66146e43181f)
warning: `parity-rpc` (lib) generated 2 warnings
warning: `ethcore` (lib) generated 7 warnings
    Finished release [optimized] target(s) in 15m 30s

Process finished with exit code 0

```


### Start Node (Leopold Staging)

```shell
2024-07-15 19:10:35 Loading config file from /home/parity/authority.toml
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC Starting OpenEthereum/v3.3.5-stable-6c2d392d8-20220405/x86_64-linux-gnu/rustc1.59.0
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC Hello!
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC Keys path /home/parity/data/keys/leopold
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC DB path /home/parity/data/chains/leopold/db/024f447b30f5b4b8
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC State DB configuration: fast
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC Operating mode: active
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC Not preparing block; cannot sign.
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC Configured for leopold using AuthorityRound engine
2024-07-15 19:10:35 2024-07-15 17:10:35 UTC Running without a persistent transaction queue.
2024-07-15 19:10:37 2024-07-15 17:10:37 UTC Signal for switch to contract-based validator set.
2024-07-15 19:10:37 2024-07-15 17:10:37 UTC Initial contract validators: [0x0065916b857b61089d8162132f3f04eebf4a4cce, 0x008beefdff2e550207cc17990b03abd18d7fb197]
2024-07-15 19:10:37 2024-07-15 17:10:37 UTC Applying validator set change signalled at block 34
2024-07-15 19:10:40 2024-07-15 17:10:40 UTC Syncing     #536 0x7e19…11a2   107.31 blk/s   95.9 tx/s    5.5 Mgas/s      0+ 1242 Qed LI:#1778    2/ 2 peers    264 KiB chain    6 MiB queue  RPC:  0 conn,    0 req/s,    0 µs
2024-07-15 19:10:41 2024-07-15 17:10:41 UTC Public node URL: enode://8d8dcfd66723689469ba1d64e08a9fc12b9e20a7f416c7a656078491f24396c70517a109bea773a473a5aa60b804563e35def8197fe58cd879d36cc635c20685@172.18.0.2:30303
2024-07-15 19:10:43 2024-07-15 17:10:43 UTC Signal for transition within contract. New list: [0x0065916b857b61089d8162132f3f04eebf4a4cce, 0x008beefdff2e550207cc17990b03abd18d7fb197, 0x001ca517d9f0bdee7906f49295a892ac7151f101]
2024-07-15 19:10:43 2024-07-15 17:10:43 UTC Applying validator set change signalled at block 1061
2024-07-15 19:10:45 2024-07-15 17:10:45 UTC Syncing    #1585 0x74c5…ab56   209.76 blk/s  102.0 tx/s    4.1 Mgas/s      0+ 4257 Qed LI:#5842    2/ 2 peers    586 KiB chain   24 MiB queue  RPC:  0 conn,    0 req/s,    0 µs
```
