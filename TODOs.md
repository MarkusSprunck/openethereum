### 1. Build Release

```text
#15 0.149 + cargo build --color=always --release --features final
#15 0.380     Updating git repository `https://github.com/paritytech/rust-secp256k1`
#15 1.096     Updating crates.io index
#15 98.77     Updating git repository `https://github.com/openethereum/app-dirs-rs`
#15 99.14     Updating git repository `https://github.com/rimrakhimov/ethabi`
#15 99.58     Updating git repository `https://github.com/gnosis/reth.git`
#15 100.1     Updating git repository `https://github.com/paritytech/bn`
#15 100.9     Updating git repository `https://github.com/matter-labs/eip1962.git`
#15 106.6     Updating git repository `https://github.com/paritytech/rust-ctrlc.git`
#15 107.1  Downloading crates ...
#15 107.3   Downloaded crunchy v0.2.2
#15 107.3   Downloaded adler32 v1.2.0
#15 107.3   Downloaded aes-ctr v0.3.0
#15 107.3   Downloaded adler v1.0.2
#15 107.3   Downloaded addr2line v0.14.1
#15 107.3   Downloaded block-buffer v0.7.3
#15 107.3   Downloaded target_info v0.1.0
#15 107.3   Downloaded tokio-retry v0.1.1
#15 107.3   Downloaded tokio-codec v0.1.1
#15 107.3   Downloaded byte-slice-cast v0.3.5
#15 107.3   Downloaded thiserror-impl v1.0.20
#15 107.3   Downloaded tokio-reactor v0.1.12
#15 107.3   Downloaded tokio-current-thread v0.1.6
#15 107.3   Downloaded validator v0.8.0
#15 107.3   Downloaded try-lock v0.2.2
#15 107.3   Downloaded winapi-build v0.1.1
#15 107.3   Downloaded bitflags v1.2.1
#15 107.3   Downloaded edit-distance v2.1.0
#15 107.3   Downloaded untrusted v0.6.2
#15 107.3   Downloaded transient-hashmap v0.4.1
#15 107.3   Downloaded crossbeam-queue v0.1.2
#15 107.3   Downloaded autocfg v0.1.7
#15 107.3   Downloaded tokio-executor v0.1.10
#15 107.3   Downloaded tokio-buf v0.1.1
#15 107.3   Downloaded tiny-keccak v2.0.2
#15 107.3   Downloaded term_size v0.3.1
#15 107.3   Downloaded want v0.0.4
#15 107.3   Downloaded synstructure v0.12.2
#15 107.3   Downloaded crossbeam-utils v0.6.6
#15 107.3   Downloaded instant v0.1.9
#15 107.3   Downloaded fs-swap v0.2.4
#15 107.3   Downloaded version_check v0.1.5
#15 107.3   Downloaded tokio-rustls v0.9.4
#15 107.4   Downloaded tokio-uds v0.2.5
#15 107.4   Downloaded tokio-io v0.1.12
#15 107.4   Downloaded ctr v0.3.2
#15 107.4   Downloaded generic-array v0.12.3
#15 107.4   Downloaded unicase v2.5.1
#15 107.4   Downloaded vec_map v0.8.1
#15 107.4   Downloaded crypto-mac v0.7.0
#15 107.4   Downloaded iovec v0.1.4
#15 107.4   Downloaded digest v0.8.1
#15 107.4   Downloaded futures-cpupool v0.1.8
#15 107.4   Downloaded humantime v1.3.0
#15 107.4   Downloaded tokio-timer v0.2.13
#15 107.4   Downloaded heapsize v0.4.2
#15 107.4   Downloaded ucd-util v0.1.8
#15 107.4   Downloaded toml v0.5.5
#15 107.4   Downloaded chrono v0.4.9
#15 107.4   Downloaded tokio-threadpool v0.1.18
#15 107.4   Downloaded indexmap v1.3.0
#15 107.4   Downloaded h2 v0.1.26
#15 107.4   Downloaded fake-simd v0.1.2
#15 107.4   Downloaded url v2.1.0
#15 107.4   Downloaded tokio-sync v0.1.7
#15 107.5   Downloaded tokio-core v0.1.17
#15 107.5   Downloaded ethabi v12.0.0
#15 107.5   Downloaded want v0.2.0
#15 107.5   Downloaded itertools v0.7.11
#15 107.5   Downloaded trace-time v0.1.2
#15 107.5   Downloaded interleaved-ordered v0.1.1
#15 107.5   Downloaded itertools v0.5.10
#15 107.5   Downloaded impl-serde v0.3.1
#15 107.5   Downloaded byte-tools v0.3.1
#15 107.5   Downloaded utf8-ranges v1.0.4
#15 107.5   Downloaded tokio-service v0.1.0
#15 107.5   Downloaded tokio-udp v0.1.5
#15 107.5   Downloaded validator_derive v0.8.0
#15 107.5   Downloaded clap v2.33.0
#15 107.5   Downloaded block-padding v0.1.4
#15 107.5   Downloaded kvdb v0.1.1
#15 107.5   Downloaded bytes v0.4.12
#15 107.5   Downloaded uint v0.8.5
#15 107.5   Downloaded crossbeam-epoch v0.7.2
#15 107.5   Downloaded crc v1.8.1
#15 107.5   Downloaded unicode-bidi v0.3.4
#15 107.5   Downloaded crossbeam-deque v0.6.3
#15 107.5   Downloaded unicode-xid v0.1.0
#15 107.6   Downloaded cfg-if v1.0.0
#15 107.6   Downloaded tokio-timer v0.1.2
#15 107.6   Downloaded typenum v1.11.2
#15 107.6   Downloaded tempfile v3.1.0
#15 107.6   Downloaded try-lock v0.1.0
#15 107.6   Downloaded crunchy v0.1.6
#15 107.6   Downloaded toml v0.4.10
#15 107.6   Downloaded trie-db v0.11.0
#15 107.6   Downloaded impl-trait-for-tuples v0.1.3
#15 107.6   Downloaded fdlimit v0.1.1
#15 107.6   Downloaded hmac v0.7.1
#15 107.6   Downloaded walkdir v2.3.1
#15 107.6   Downloaded const-random-macro v0.1.13
#15 107.6   Downloaded ipnetwork v0.12.8
#15 107.6   Downloaded xml-rs v0.7.0
#15 107.6   Downloaded hyper v0.12.35
#15 107.6   Downloaded enum_primitive v0.1.1
#15 107.6   Downloaded const-random v0.1.13
#15 107.6   Downloaded ethabi-contract v11.0.0
#15 107.6   Downloaded c2-chacha v0.2.3
#15 107.6   Downloaded libc v0.2.89
#15 107.7   Downloaded url v1.7.2
#15 107.7   Downloaded failure_derive v0.1.8
#15 107.7   Downloaded unicode-xid v0.2.0
#15 107.7   Downloaded byteorder v1.3.2
#15 107.7   Downloaded either v1.5.3
#15 107.7   Downloaded globset v0.4.5
#15 107.7   Downloaded ethereum-forkid v0.2.1
#15 107.7   Downloaded inflate v0.4.5
#15 107.7   Downloaded hyper-rustls v0.16.1
#15 107.7   Downloaded getopts v0.2.21
#15 107.7   Downloaded ct-logs v0.5.1
#15 107.7   Downloaded unicode-segmentation v1.5.0
#15 107.7   Downloaded parity-util-mem-derive v0.1.0
#15 107.7   Downloaded arrayvec v0.4.12
#15 107.7   Downloaded ahash v0.2.19
#15 107.7   Downloaded memory_units v0.3.0
#15 107.7   Downloaded gcc v0.3.55
#15 107.7   Downloaded failure v0.1.8
#15 107.7   Downloaded error-chain v0.12.1
#15 107.7   Downloaded safemem v0.3.3
#15 107.7   Downloaded syn v1.0.86
#15 107.8   Downloaded crossbeam-utils v0.7.2
#15 107.8   Downloaded fnv v1.0.6
#15 107.8   Downloaded log v0.4.8
#15 107.8   Downloaded hash-db v0.11.0
#15 107.8   Downloaded ethereum-types v0.9.2
#15 107.8   Downloaded heck v0.3.1
#15 107.8   Downloaded ethbloom v0.9.2
#15 107.8   Downloaded derive_more v0.99.9
#15 107.8   Downloaded parity-bytes v0.1.1
#15 107.8   Downloaded maplit v1.0.2
#15 107.8   Downloaded getrandom v0.1.13
#15 107.8   Downloaded stream-cipher v0.3.2
#15 107.8   Downloaded itoa v0.4.4
#15 107.8   Downloaded slab v0.4.2
#15 107.8   Downloaded idna v0.1.5
#15 107.8   Downloaded memmap v0.6.2
#15 107.8   Downloaded jsonrpc-core v15.0.0
#15 107.8   Downloaded hyper v0.11.27
#15 107.8   Downloaded mio-uds v0.6.7
#15 107.8   Downloaded matches v0.1.8
#15 107.8   Downloaded nodrop v0.1.14
#15 107.8   Downloaded smallvec v0.6.13
#15 107.8   Downloaded rand_hc v0.1.0
#15 107.9   Downloaded remove_dir_all v0.5.2
#15 107.9   Downloaded static_assertions v1.1.0
#15 107.9   Downloaded triehash v0.5.0
#15 107.9   Downloaded logos-derive v0.12.0
#15 107.9   Downloaded relay v0.1.1
#15 107.9   Downloaded rayon-core v1.6.0
#15 107.9   Downloaded proc-macro-crate v0.1.4
#15 107.9   Downloaded if_chain v0.1.3
#15 107.9   Downloaded kvdb-memorydb v0.1.0
#15 107.9   Downloaded ppv-lite86 v0.2.6
#15 107.9   Downloaded zeroize v1.2.0
#15 107.9   Downloaded num v0.1.42
#15 107.9   Downloaded maybe-uninit v2.0.0
#15 107.9   Downloaded httparse v1.3.4
#15 107.9   Downloaded rand_chacha v0.2.1
#15 107.9   Downloaded primal-estimate v0.2.1
#15 107.9   Downloaded impl-codec v0.4.2
#15 107.9   Downloaded ryu v1.0.2
#15 107.9   Downloaded xmltree v0.7.0
#15 107.9   Downloaded tokio-named-pipes v0.1.0
#15 107.9   Downloaded scrypt v0.2.0
#15 107.9   Downloaded lru v0.5.3
#15 107.9   Downloaded jsonrpc-pubsub v15.0.0
#15 107.9   Downloaded serde_derive v1.0.102
#15 107.9   Downloaded rand_jitter v0.1.4
#15 107.9   Downloaded unicode-width v0.1.6
#15 107.9   Downloaded timer v0.2.0
#15 107.9   Downloaded radium v0.3.0
#15 107.9   Downloaded num-traits v0.2.8
#15 107.9   Downloaded rand_isaac v0.1.1
#15 107.9   Downloaded parking_lot v0.6.4
#15 107.9   Downloaded jsonrpc-http-server v15.0.0
#15 107.9   Downloaded primitive-types v0.7.2
#15 107.9   Downloaded same-file v1.0.5
#15 107.9   Downloaded string v0.2.1
#15 107.9   Downloaded proc-macro-hack v0.5.19
#15 108.0   Downloaded time v0.1.42
#15 108.0   Downloaded sha-1 v0.8.1
#15 108.0   Downloaded igd v0.7.1
#15 108.0   Downloaded vergen v0.1.1
#15 108.0   Downloaded primal v0.2.3
#15 108.0   Downloaded parity-tokio-ipc v0.4.0
#15 108.0   Downloaded num-traits v0.1.43
#15 108.0   Downloaded proc-macro2 v0.4.30
#15 108.0   Downloaded rand v0.4.6
#15 108.0   Downloaded jsonrpc-ipc-server v15.0.0
#15 108.0   Downloaded mio-named-pipes v0.1.6
#15 108.0   Downloaded hashbrown v0.8.2
#15 108.0   Downloaded mio v0.6.22
#15 108.0   Downloaded num-integer v0.1.41
#15 108.0   Downloaded rand_core v0.5.1
#15 108.0   Downloaded ripemd160 v0.8.0
#15 108.0   Downloaded tokio-fs v0.1.6
#15 108.0   Downloaded futures v0.1.29
#15 108.0   Downloaded rayon v1.2.0
#15 108.1   Downloaded num-iter v0.1.39
#15 108.1   Downloaded num-bigint v0.1.44
#15 108.1   Downloaded http v0.1.21
#15 108.1   Downloaded xdg v2.2.0
#15 108.1   Downloaded protobuf v2.16.2
#15 108.1   Downloaded regex v0.2.11
#15 108.1   Downloaded proc-macro2 v1.0.36
#15 108.1   Downloaded parity-wasm v0.31.3
#15 108.1   Downloaded jsonrpc-derive v15.0.0
#15 108.1   Downloaded getrandom v0.2.2
#15 108.1   Downloaded opaque-debug v0.2.3
#15 108.1   Downloaded docopt v1.1.0
#15 108.1   Downloaded bitvec v0.17.4
#15 108.1   Downloaded pbkdf2 v0.3.0
#15 108.2   Downloaded cc v1.0.41
#15 108.2   Downloaded webpki v0.19.1
#15 108.2   Downloaded quote v1.0.7
#15 108.2   Downloaded slab v0.3.0
#15 108.2   Downloaded sct v0.5.0
#15 108.2   Downloaded primal-sieve v0.2.9
#15 108.2   Downloaded sha2 v0.8.0
#15 108.2   Downloaded siphasher v0.1.3
#15 108.2   Downloaded rand_core v0.3.1
#15 108.2   Downloaded smallvec v1.6.1
#15 108.2   Downloaded unicode-normalization v0.1.8
#15 108.2   Downloaded serde_repr v0.1.6
#15 108.2   Downloaded parking_lot v0.9.0
#15 108.2   Downloaded gimli v0.23.0
#15 108.2   Downloaded webpki-roots v0.16.0
#15 108.2   Downloaded rprompt v1.0.3
#15 108.2   Downloaded rand_os v0.1.3
#15 108.2   Downloaded spin v0.5.2
#15 108.2   Downloaded rpassword v1.0.2
#15 108.3   Downloaded mio-extras v2.0.5
#15 108.3   Downloaded parity-scale-codec v1.3.5
#15 108.3   Downloaded jsonrpc-tcp-server v15.0.0
#15 108.3   Downloaded jsonrpc-server-utils v15.0.0
#15 108.3   Downloaded parity-util-mem v0.7.0
#15 108.3   Downloaded jsonrpc-ws-server v15.0.0
#15 108.3   Downloaded primal-bit v0.2.4
#15 108.3   Downloaded plain_hasher v0.2.2
#15 108.3   Downloaded parity-path v0.1.2
#15 108.3   Downloaded lru-cache v0.1.2
#15 108.3   Downloaded parity-wordlist v1.3.0
#15 108.3   Downloaded scoped-tls v0.1.2
#15 108.3   Downloaded slab v0.2.0
#15 108.3   Downloaded num-bigint v0.2.3
#15 108.3   Downloaded once_cell v1.4.0
#15 108.3   Downloaded keccak-hash v0.5.1
#15 108.3   Downloaded order-stat v0.1.3
#15 108.3   Downloaded miniz_oxide v0.4.4
#15 108.3   Downloaded percent-encoding v1.0.1
#15 108.3   Downloaded strsim v0.8.0
#15 108.3   Downloaded rand v0.3.23
#15 108.3   Downloaded skeptic v0.4.0
#15 108.3   Downloaded pulldown-cmark v0.0.3
#15 108.3   Downloaded rustc-serialize v0.3.25
#15 108.3   Downloaded stable_deref_trait v1.1.1
#15 108.3   Downloaded wasmi v0.3.0
#15 108.3   Downloaded rustc-hex v2.1.0
#15 108.3   Downloaded rand_xorshift v0.2.0
#15 108.3   Downloaded rustc-hex v1.0.0
#15 108.3   Downloaded net2 v0.2.33
#15 108.3   Downloaded number_prefix v0.2.8
#15 108.3   Downloaded rlp v0.4.6
#15 108.3   Downloaded quote v0.6.13
#15 108.3   Downloaded miow v0.3.7
#15 108.3   Downloaded serde_json v1.0.41
#15 108.3   Downloaded lock_api v0.1.5
#15 108.4   Downloaded lazycell v1.2.1
#15 108.4   Downloaded pwasm-utils v0.6.2
#15 108.4   Downloaded lazy_static v1.4.0
#15 108.4   Downloaded num_cpus v1.11.0
#15 108.4   Downloaded quick-error v1.2.2
#15 108.4   Downloaded rand_xorshift v0.1.1
#15 108.4   Downloaded mime v0.3.14
#15 108.4   Downloaded language-tags v0.2.2
#15 108.4   Downloaded prometheus v0.9.0
#15 108.4   Downloaded local-encoding v0.2.0
#15 108.4   Downloaded idna v0.2.0
#15 108.4   Downloaded parking_lot_core v0.7.2
#15 108.4   Downloaded parking_lot_core v0.3.1
#15 108.4   Downloaded parking_lot_core v0.6.2
#15 108.4   Downloaded nan-preserving-float v0.1.0
#15 108.4   Downloaded rand_core v0.4.2
#15 108.4   Downloaded memoffset v0.5.2
#15 108.4   Downloaded parity-daemonize v0.3.0
#15 108.4   Downloaded secp256k1 v0.17.2
#15 108.4   Downloaded semver-parser v0.7.0
#15 108.4   Downloaded memchr v2.2.1
#15 108.4   Downloaded scopeguard v1.1.0
#15 108.4   Downloaded lock_api v0.3.4
#15 108.4   Downloaded kernel32-sys v0.2.2
#15 108.4   Downloaded parking_lot v0.10.2
#15 108.4   Downloaded rustc-demangle v0.1.16
#15 108.4   Downloaded subtle v1.0.0
#15 108.4   Downloaded winapi v0.2.8
#15 108.5   Downloaded owning_ref v0.4.0
#15 108.5   Downloaded strsim v0.9.2
#15 108.5   Downloaded scopeguard v0.3.3
#15 108.5   Downloaded subtle v2.3.0
#15 108.5   Downloaded semver v0.9.0
#15 108.5   Downloaded serde v1.0.102
#15 108.5   Downloaded parity-crypto v0.6.2
#15 108.5   Downloaded parking_lot v0.11.1
#15 108.5   Downloaded parking_lot_core v0.8.3
#15 108.5   Downloaded rand_chacha v0.1.1
#15 108.5   Downloaded rand_pcg v0.1.2
#15 108.5   Downloaded percent-encoding v2.1.0
#15 108.5   Downloaded logos v0.12.0
#15 108.5   Downloaded regex-syntax v0.6.18
#15 108.5   Downloaded linked-hash-map v0.5.3
#15 108.5   Downloaded rand v0.5.6
#15 108.5   Downloaded parity-rocksdb v0.5.1
#15 108.5   Downloaded lock_api v0.4.3
#15 108.5   Downloaded rand v0.6.5
#15 108.5   Downloaded rustc_version v0.2.3
#15 108.5   Downloaded rlp-derive v0.1.0
#15 108.5   Downloaded primal-check v0.2.3
#15 108.6   Downloaded kvdb-rocksdb v0.1.6
#15 108.6   Downloaded rand v0.7.3
#15 108.6   Downloaded syn v0.15.26
#15 108.6   Downloaded parity-ws v0.10.0
#15 108.6   Downloaded parity-snappy v0.1.0
#15 108.6   Downloaded bitflags v0.7.0
#15 108.6   Downloaded regex-syntax v0.5.6
#15 108.6   Downloaded atty v0.2.13
#15 108.6   Downloaded ring v0.14.6
#15 108.7   Downloaded crossbeam-queue v0.2.3
#15 108.7   Downloaded block-cipher-trait v0.6.2
#15 108.7   Downloaded bit-set v0.4.0
#15 108.7   Downloaded secp256k1-sys v0.1.2
#15 108.8   Downloaded object v0.23.0
#15 108.8   Downloaded aho-corasick v0.6.10
#15 108.8   Downloaded tokio-tcp v0.1.3
#15 108.8   Downloaded build_const v0.2.1
#15 108.8   Downloaded regex v1.3.9
#15 108.8   Downloaded autocfg v1.0.0
#15 108.8   Downloaded impl-rlp v0.2.1
#15 108.8   Downloaded aho-corasick v0.7.6
#15 108.8   Downloaded base64 v0.10.1
#15 108.8   Downloaded ansi_term v0.10.2
#15 108.8   Downloaded bstr v0.2.8
#15 108.8   Downloaded hashbrown v0.6.3
#15 108.8   Downloaded tokio v0.1.22
#15 108.9   Downloaded home v0.3.4
#15 108.9   Downloaded env_logger v0.5.13
#15 108.9   Downloaded crossbeam-channel v0.5.2
#15 108.9   Downloaded backtrace v0.3.56
#15 108.9   Downloaded fixed-hash v0.6.1
#15 108.9   Downloaded hex v0.4.3
#15 108.9   Downloaded hamming v0.1.3
#15 108.9   Downloaded elastic-array v0.10.2
#15 108.9   Downloaded rustls v0.15.2
#15 108.9   Downloaded ahash v0.3.8
#15 108.9   Downloaded arrayvec v0.5.1
#15 108.9   Downloaded base64 v0.9.3
#15 108.9   Downloaded home v0.5.1
#15 108.9   Downloaded termcolor v1.0.5
#15 108.9   Downloaded aes-soft v0.3.3
#15 108.9   Downloaded thread_local v1.0.1
#15 108.9   Downloaded crossbeam-utils v0.8.6
#15 108.9   Downloaded crossbeam-deque v0.7.1
#15 109.0   Downloaded aes v0.3.2
#15 109.0   Downloaded beef v0.5.1
#15 109.0   Downloaded block-modes v0.3.3
#15 109.0   Downloaded bit-vec v0.4.4
#15 109.0   Downloaded ansi_term v0.11.0
#15 109.0   Downloaded http-body v0.1.0
#15 109.0   Downloaded tiny-keccak v1.5.0
#15 109.0   Downloaded thread_local v0.3.6
#15 109.0   Downloaded thiserror v1.0.20
#15 109.0   Downloaded textwrap v0.9.0
#15 109.0   Downloaded cmake v0.1.42
#15 109.0   Downloaded textwrap v0.11.0
#15 109.0   Downloaded tempdir v0.3.7
#15 109.0   Downloaded cfg-if v0.1.10
#15 109.0   Downloaded rust-crypto v0.2.36
#15 109.0   Downloaded winapi v0.3.8
#15 109.1   Downloaded parity-snappy-sys v0.1.2
#15 109.1   Downloaded parity-rocksdb-sys v0.5.6
#15 109.5    Compiling libc v0.2.89
#15 109.5    Compiling proc-macro2 v1.0.36
#15 109.5    Compiling unicode-xid v0.2.0
#15 109.5    Compiling syn v1.0.86
#15 109.6    Compiling serde v1.0.102
#15 110.0    Compiling cfg-if v0.1.10
#15 110.0    Compiling byteorder v1.3.2
#15 110.0    Compiling lazy_static v1.4.0
#15 110.1    Compiling autocfg v0.1.7
#15 110.1    Compiling either v1.5.3
#15 110.1    Compiling log v0.4.8
#15 110.3    Compiling maybe-uninit v2.0.0
#15 110.5    Compiling autocfg v1.0.0
#15 110.6    Compiling scopeguard v1.1.0
#15 110.6    Compiling crunchy v0.2.2
#15 110.7    Compiling semver-parser v0.7.0
#15 110.7    Compiling getrandom v0.1.13
#15 111.0    Compiling arrayvec v0.4.12
#15 111.0    Compiling rustc-hex v2.1.0
#15 111.1    Compiling cc v1.0.41
#15 111.2    Compiling tiny-keccak v2.0.2
#15 111.2    Compiling futures v0.1.29
#15 111.5    Compiling nodrop v0.1.14
#15 111.5    Compiling ppv-lite86 v0.2.6
#15 111.6    Compiling static_assertions v1.1.0
#15 111.7    Compiling typenum v1.11.2
#15 111.9    Compiling rand_core v0.4.2
#15 112.2    Compiling fnv v1.0.6
#15 112.3    Compiling ryu v1.0.2
#15 112.3    Compiling radium v0.3.0
#15 112.5    Compiling smallvec v1.6.1
#15 112.6    Compiling slab v0.4.2
#15 112.8    Compiling arrayvec v0.5.1
#15 112.8    Compiling byte-slice-cast v0.3.5
#15 112.8    Compiling itoa v0.4.4
#15 113.0    Compiling memchr v2.2.1
#15 113.3    Compiling opaque-debug v0.2.3
#15 113.4    Compiling byte-tools v0.3.1
#15 113.5    Compiling cfg-if v1.0.0
#15 113.5    Compiling fake-simd v0.1.2
#15 113.6    Compiling proc-macro-hack v0.5.19
#15 113.6    Compiling getrandom v0.2.2
#15 113.7    Compiling safemem v0.3.3
#15 113.7    Compiling subtle v1.0.0
#15 113.8    Compiling regex-syntax v0.6.18
#15 113.9    Compiling parity-bytes v0.1.1
#15 114.0    Compiling quick-error v1.2.2
#15 114.0    Compiling proc-macro2 v0.4.30
#15 114.1    Compiling unicode-xid v0.1.0
#15 114.2    Compiling subtle v2.3.0
#15 114.3    Compiling rustc-hex v1.0.0
#15 114.4    Compiling ahash v0.3.8
#15 114.6    Compiling zeroize v1.2.0
#15 114.6    Compiling parity-util-mem v0.7.0
#15 114.6    Compiling version_check v0.1.5
#15 114.8    Compiling spin v0.5.2
#15 114.9    Compiling edit-distance v2.1.0
#15 114.9    Compiling memzero v0.1.0 (/build/crates/util/memzero)
#15 115.0    Compiling heapsize v0.4.2
#15 115.1    Compiling unicode-width v0.1.6
#15 115.1    Compiling adler32 v1.2.0
#15 115.2    Compiling remove_dir_all v0.5.2
#15 115.3    Compiling httparse v1.3.4
#15 115.3    Compiling unexpected v0.1.0 (/build/crates/util/unexpected)
#15 115.4    Compiling hex v0.4.3
#15 115.5    Compiling matches v0.1.8
#15 115.6    Compiling hash-db v0.11.0
#15 115.8    Compiling untrusted v0.6.2
#15 115.8    Compiling gimli v0.23.0
#15 116.0    Compiling adler v1.0.2
#15 116.2    Compiling object v0.23.0
#15 116.4    Compiling rustc-demangle v0.1.16
#15 117.3    Compiling protobuf v2.16.2
#15 117.8    Compiling stable_deref_trait v1.1.1
#15 117.9    Compiling scopeguard v0.3.3
#15 118.0    Compiling try-lock v0.2.2
#15 118.3    Compiling prometheus v0.9.0
#15 119.0    Compiling interleaved-ordered v0.1.1
#15 119.1    Compiling percent-encoding v2.1.0
#15 119.9    Compiling bitflags v1.2.1
#15 120.2    Compiling rustc-serialize v0.3.25
#15 120.2    Compiling linked-hash-map v0.5.3
#15 120.4    Compiling percent-encoding v1.0.1
#15 120.5    Compiling hamming v0.1.3
#15 120.7    Compiling rayon-core v1.6.0
#15 120.7    Compiling crunchy v0.1.6
#15 121.0    Compiling primal-estimate v0.2.1
#15 121.1    Compiling scoped-tls v0.1.2
#15 121.3    Compiling crossbeam-utils v0.8.6
#15 121.3    Compiling macros v0.1.0 (/build/crates/util/macros)
#15 121.4    Compiling regex v0.2.11
#15 121.6    Compiling once_cell v1.4.0
#15 121.9    Compiling ansi_term v0.11.0
#15 121.9    Compiling gcc v0.3.55
#15 122.0    Compiling unicode-segmentation v1.5.0
#15 122.3    Compiling try-lock v0.1.0
#15 122.4    Compiling ansi_term v0.10.2
#15 122.6    Compiling ucd-util v0.1.8
#15 123.0    Compiling build_const v0.2.1
#15 123.3    Compiling memory_units v0.3.0
#15 123.4    Compiling slab v0.3.0
#15 123.5    Compiling winapi v0.3.8
#15 123.6    Compiling utf8-ranges v1.0.4
#15 123.7    Compiling mime v0.3.14
#15 123.9    Compiling language-tags v0.2.2
#15 124.3    Compiling nan-preserving-float v0.1.0
#15 124.3    Compiling bit-vec v0.4.4
#15 124.5    Compiling siphasher v0.1.3
#15 124.6    Compiling failure_derive v0.1.8
#15 124.8    Compiling ethabi-contract v11.0.0
#15 124.9    Compiling same-file v1.0.5
#15 125.0    Compiling maplit v1.0.2
#15 125.1    Compiling home v0.5.1
#15 125.2    Compiling ipnetwork v0.12.8
#15 125.4    Compiling using_queue v0.1.0 (/build/crates/concensus/miner/using-queue)
#15 125.4    Compiling bitflags v0.7.0
#15 125.5    Compiling lazycell v1.2.1
#15 125.5    Compiling mio-named-pipes v0.1.6
#15 125.6    Compiling time-utils v0.1.0 (/build/crates/util/time-utils)
#15 125.7    Compiling slab v0.2.0
#15 125.8    Compiling reth-util v0.1.0 (https://github.com/gnosis/reth.git?rev=573e128#573e1284)
#15 125.9    Compiling beef v0.5.1
#15 126.0    Compiling if_chain v0.1.3
#15 126.1    Compiling termcolor v1.0.5
#15 126.1    Compiling target_info v0.1.0
#15 126.1    Compiling winapi-build v0.1.1
#15 126.2    Compiling transient-hashmap v0.4.1
#15 126.4    Compiling order-stat v0.1.3
#15 126.5    Compiling rprompt v1.0.3
#15 126.6    Compiling xdg v2.2.0
#15 126.7    Compiling winapi v0.2.8
#15 126.8    Compiling strsim v0.9.2
#15 126.8    Compiling home v0.3.4
#15 126.8    Compiling vec_map v0.8.1
#15 127.0    Compiling strsim v0.8.0
#15 127.6    Compiling crossbeam-utils v0.6.6
#15 127.6    Compiling thread_local v1.0.1
#15 127.6    Compiling thread_local v0.3.6
#15 127.7    Compiling itertools v0.5.10
#15 127.9    Compiling itertools v0.7.11
#15 128.1    Compiling rand_pcg v0.1.2
#15 128.5    Compiling rand_chacha v0.1.1
#15 128.5    Compiling rand v0.6.5
#15 128.5    Compiling num-traits v0.2.8
#15 128.8    Compiling hashbrown v0.6.3
#15 128.8    Compiling num-integer v0.1.41
#15 128.8    Compiling indexmap v1.3.0
#15 128.9    Compiling num-iter v0.1.39
#15 129.1    Compiling num-bigint v0.2.3
#15 129.1    Compiling lock_api v0.3.4
#15 129.1    Compiling lock_api v0.4.3
#15 129.4    Compiling crossbeam-utils v0.7.2
#15 129.4    Compiling hashbrown v0.8.2
#15 129.4    Compiling miniz_oxide v0.4.4
#15 129.5    Compiling rlp v0.4.6
#15 129.7    Compiling eip-152 v0.1.0 (/build/crates/util/EIP-152)
#15 129.7    Compiling cmake v0.1.42
#15 129.8    Compiling c2-chacha v0.2.3
#15 129.9    Compiling rand_core v0.3.1
#15 130.0    Compiling rand_jitter v0.1.4
#15 130.2    Compiling secp256k1-sys v0.1.2
#15 130.3    Compiling ring v0.14.6
#15 130.4    Compiling bitvec v0.17.4
#15 130.5    Compiling tokio-sync v0.1.7
#15 130.7    Compiling tokio-service v0.1.0
#15 130.8    Compiling relay v0.1.1
#15 131.1    Compiling eth-secp256k1 v0.5.7 (https://github.com/paritytech/rust-secp256k1?rev=9791e79f21a5309dcb6e0bd254b1ef88fca2f1f4#9791e79f)
#15 131.2    Compiling block-padding v0.1.4
#15 131.4    Compiling instant v0.1.9
#15 131.6    Compiling humantime v1.3.0
#15 131.7    Compiling unicase v2.5.1
#15 131.7    Compiling error-chain v0.12.1
#15 131.8    Compiling getopts v0.2.21
#15 132.0    Compiling inflate v0.4.5
#15 132.1    Compiling unicode-bidi v0.3.4
#15 132.9    Compiling owning_ref v0.4.0
#15 132.9    Compiling addr2line v0.14.1
#15 133.2    Compiling lru-cache v0.1.2
#15 133.2    Compiling primal-bit v0.2.4
#15 133.4    Compiling heck v0.3.1
#15 133.4    Compiling regex-syntax v0.5.6
#15 133.6    Compiling rust-crypto v0.2.36
#15 134.0    Compiling crc v1.8.1
#15 134.3    Compiling tokio-timer v0.1.2
#15 134.5    Compiling bit-set v0.4.0
#15 134.5    Compiling ethcore-bloom-journal v0.1.0 (/build/crates/db/bloom)
#15 134.7    Compiling walkdir v2.3.1
#15 135.2    Compiling parity-path v0.1.2
#15 135.4    Compiling kernel32-sys v0.2.2
#15 135.4    Compiling textwrap v0.11.0
#15 135.6    Compiling textwrap v0.9.0
#15 135.6    Compiling app_dirs v1.2.1 (https://github.com/openethereum/app-dirs-rs#0b37f948)
#15 136.9    Compiling crossbeam-queue v0.1.2
#15 138.4    Compiling impl-rlp v0.2.1
#15 138.5    Compiling triehash v0.5.0
#15 138.7    Compiling rand_hc v0.1.0
#15 138.8    Compiling rand_xorshift v0.1.1
#15 138.8    Compiling rand_isaac v0.1.1
#15 139.0    Compiling parity-snappy-sys v0.1.2
#15 139.0    Compiling parity-rocksdb-sys v0.5.6
#15 142.6    Compiling pulldown-cmark v0.0.3
#15 143.8    Compiling lock_api v0.1.5
#15 144.4    Compiling quote v1.0.7
#15 144.7    Compiling iovec v0.1.4
#15 144.9    Compiling num_cpus v1.11.0
#15 144.9    Compiling net2 v0.2.33
#15 144.9    Compiling parking_lot_core v0.7.2
#15 145.4    Compiling rand_os v0.1.3
#15 145.4    Compiling rand v0.5.6
#15 145.6    Compiling time v0.1.42
#15 145.9    Compiling parking_lot_core v0.8.3
#15 146.3    Compiling rand v0.4.6
#15 146.5    Compiling fs-swap v0.2.4
#15 146.6    Compiling memmap v0.6.2
#15 146.9    Compiling atty v0.2.13
#15 147.1    Compiling rpassword v1.0.2
#15 147.3    Compiling fdlimit v0.1.1
#15 147.5    Compiling term_size v0.3.1
#15 148.0    Compiling want v0.2.0
#15 148.2    Compiling trace-time v0.1.2
#15 148.3    Compiling want v0.0.4
#15 148.4    Compiling base64 v0.9.3
#15 148.6    Compiling base64 v0.10.1
#15 148.6    Compiling parity-wasm v0.31.3
#15 149.8    Compiling smallvec v0.6.13
#15 150.4    Compiling uint v0.8.5
#15 150.5    Compiling tiny-keccak v1.5.0
#15 150.6    Compiling plain_hasher v0.2.2
#15 150.7    Compiling simple_uint v0.1.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
#15 152.2    Compiling generic-array v0.12.3
#15 152.5    Compiling aho-corasick v0.7.6
#15 152.7    Compiling bstr v0.2.8
#15 153.6    Compiling aho-corasick v0.6.10
#15 155.3    Compiling quote v0.6.13
#15 155.7    Compiling elastic-array v0.10.2
#15 156.1    Compiling xml-rs v0.7.0
#15 157.2    Compiling miow v0.3.7
#15 157.4    Compiling crossbeam-channel v0.5.2
#15 158.4    Compiling rand_core v0.5.1
#15 158.7    Compiling bytes v0.4.12
#15 158.9    Compiling futures-cpupool v0.1.8
#15 159.5    Compiling mio v0.6.22
#15 159.9    Compiling parking_lot v0.10.2
#15 161.1    Compiling parking_lot v0.11.1
#15 161.4    Compiling bn v0.4.4 (https://github.com/paritytech/bn#6079255e)
#15 161.9    Compiling rand v0.3.23
#15 162.0    Compiling tempdir v0.3.7
#15 162.3    Compiling clap v2.33.0
#15 162.7    Compiling vergen v0.1.1
#15 163.2    Compiling wasmi v0.3.0
#15 164.6    Compiling pwasm-utils v0.6.2
#15 167.8    Compiling num-traits v0.1.43
#15 167.9    Compiling number_prefix v0.2.8
#15 168.0    Compiling unicode-normalization v0.1.8
#15 168.1    Compiling primal-sieve v0.2.9
#15 169.2    Compiling txpool v1.0.0-alpha (/build/crates/transaction-pool)
#15 169.6    Compiling fixed_width_field v0.1.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
#15 170.1    Compiling fixed_width_group_and_loop v0.1.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
#15 170.5    Compiling const-random-macro v0.1.13
#15 171.3    Compiling tokio-executor v0.1.10
#15 171.7    Compiling crossbeam-queue v0.2.3
#15 171.8    Compiling backtrace v0.3.56
#15 171.8    Compiling digest v0.8.1
#15 172.0    Compiling block-buffer v0.7.3
#15 172.1    Compiling block-cipher-trait v0.6.2
#15 172.3    Compiling crypto-mac v0.7.0
#15 172.4    Compiling stream-cipher v0.3.2
#15 172.5    Compiling regex v1.3.9
#15 175.5    Compiling syn v0.15.26
#15 177.4    Compiling kvdb v0.1.1
#15 177.7    Compiling rlp_compress v0.1.0 (/build/crates/util/rlp-compress)
#15 178.3    Compiling xmltree v0.7.0
#15 180.1    Compiling sct v0.5.0
#15 180.3    Compiling webpki v0.19.1
#15 180.4    Compiling ctrlc v1.1.1 (https://github.com/paritytech/rust-ctrlc.git#b5230171)
#15 180.5    Compiling rand_chacha v0.2.1
#15 180.7    Compiling rand_xorshift v0.2.0
#15 180.9    Compiling tokio-io v0.1.12
#15 181.1    Compiling http v0.1.21
#15 181.6    Compiling tokio-buf v0.1.1
#15 181.7    Compiling string v0.2.1
#15 181.8    Compiling mio-uds v0.6.7
#15 182.0    Compiling mio-extras v2.0.5
#15 182.3    Compiling len-caching-lock v0.1.1 (/build/crates/util/len-caching-lock)
#15 182.8    Compiling skeptic v0.4.0
#15 183.2    Compiling synstructure v0.12.2
#15 184.3    Compiling secp256k1 v0.17.2
#15 184.3    Compiling parity-wordlist v1.3.0
#15 185.3    Compiling trie-db v0.11.0
#15 185.3    Compiling chrono v0.4.9
#15 186.1    Compiling num-bigint v0.1.44
#15 188.1    Compiling primal-check v0.2.3
#15 188.3    Compiling serde_derive v1.0.102
#15 188.8    Compiling impl-trait-for-tuples v0.1.3
#15 191.9    Compiling serde_repr v0.1.6
#15 193.3    Compiling thiserror-impl v1.0.20
#15 194.5    Compiling eth_pairings_repr_derive v0.2.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
#15 195.8    Compiling derive_more v0.99.9
#15 196.1    Compiling rlp-derive v0.1.0
#15 196.3    Compiling logos-derive v0.12.0
#15 197.5    Compiling enum_primitive v0.1.1
#15 197.6    Compiling idna v0.2.0
#15 200.3    Compiling idna v0.1.5
#15 201.3    Compiling const-random v0.1.13
#15 201.4    Compiling tokio-current-thread v0.1.6
#15 201.8    Compiling tokio-timer v0.2.13
#15 202.8    Compiling panic_hook v0.1.0 (/build/crates/util/panic-hook)
#15 202.8    Compiling sha2 v0.8.0
#15 203.0    Compiling ripemd160 v0.8.0
#15 203.6    Compiling sha-1 v0.8.1
#15 203.6    Compiling aes-soft v0.3.3
#15 203.9    Compiling block-modes v0.3.3
#15 204.1    Compiling hmac v0.7.1
#15 204.1    Compiling ctr v0.3.2
#15 204.1    Compiling globset v0.4.5
#15 204.2    Compiling env_logger v0.5.13
#15 204.3    Compiling parity-snappy v0.1.0
#15 204.5    Compiling rlp_derive v0.1.0 (/build/crates/util/rlp-derive)
#15 205.6    Compiling ct-logs v0.5.1
#15 205.7    Compiling rustls v0.15.2
#15 205.8    Compiling webpki-roots v0.16.0
#15 205.9    Compiling rand v0.7.3
#15 207.4    Compiling tokio-codec v0.1.1
#15 207.9    Compiling h2 v0.1.26
#15 209.1    Compiling http-body v0.1.0
#15 209.2    Compiling local-encoding v0.2.0
#15 209.6    Compiling parity-util-mem-derive v0.1.0
#15 212.3    Compiling timer v0.2.0
#15 213.0    Compiling num v0.1.42
#15 213.1    Compiling primal v0.2.3
#15 214.1    Compiling thiserror v1.0.20
#15 214.2    Compiling eth_pairings v0.6.0 (https://github.com/matter-labs/eip1962.git?rev=ece6cbabc41948db4200e41f0bfdab7ab94c7af8#ece6cbab)
#15 217.3    Compiling url v2.1.0
#15 217.7    Compiling logos v0.12.0
#15 217.9    Compiling url v1.7.2
#15 220.3    Compiling ahash v0.2.19
#15 220.5    Compiling aes v0.3.2
#15 220.6    Compiling pbkdf2 v0.3.0
#15 220.7    Compiling aes-ctr v0.3.0
#15 220.9    Compiling ethcore-logger v1.12.0 (/build/bin/oe/logger)
#15 221.3    Compiling fixed-hash v0.6.1
#15 221.4    Compiling tempfile v3.1.0
#15 221.9    Compiling tokio-rustls v0.9.4
#15 222.2    Compiling failure v0.1.8
#15 222.6    Compiling semver v0.9.0
#15 223.1    Compiling serde_json v1.0.41
#15 225.0    Compiling impl-serde v0.3.1
#15 225.3    Compiling parity-scale-codec v1.3.5
#15 228.6    Compiling toml v0.4.10
#15 229.6    Compiling toml v0.5.5
#15 235.4    Compiling oe-rpc-common v0.0.0 (/build/crates/rpc-common)
#15 235.5    Compiling docopt v1.1.0
#15 239.1    Compiling parity-ws v0.10.0
#15 240.4    Compiling scrypt v0.2.0
#15 241.1    Compiling parity-daemonize v0.3.0
#15 241.2    Compiling rustc_version v0.2.3
#15 241.6    Compiling stats v0.1.0 (/build/crates/util/stats)
#15 241.6    Compiling validator v0.8.0
#15 242.3    Compiling ethbloom v0.9.2
#15 242.4    Compiling impl-codec v0.4.2
#15 242.7    Compiling proc-macro-crate v0.1.4
#15 243.0    Compiling jsonrpc-core v15.0.0
#15 245.1    Compiling lru v0.5.3
#15 245.3    Compiling parking_lot_core v0.6.2
#15 245.7    Compiling parking_lot v0.9.0
#15 245.7    Compiling memoffset v0.5.2
#15 246.1    Compiling parking_lot_core v0.3.1
#15 246.1    Compiling hyper v0.12.35
#15 246.4    Compiling parity-version v3.3.6 (/build/crates/util/version)
#15 246.5    Compiling validator_derive v0.8.0
#15 246.9    Compiling primitive-types v0.7.2
#15 248.9    Compiling blooms-db v0.1.0 (/build/crates/db/blooms-db)
#15 249.3    Compiling jsonrpc-derive v15.0.0
#15 249.4    Compiling jsonrpc-pubsub v15.0.0
#15 254.5    Compiling ethereum-types v0.9.2
#15 255.4    Compiling keccak-hash v0.5.1
#15 256.6    Compiling ethabi v11.0.0 (https://github.com/rimrakhimov/ethabi?branch=rimrakhimov/remove-syn-export-span#222e6482)
#15 257.0    Compiling parity-crypto v0.6.2
#15 258.7    Compiling keccak-hasher v0.1.1 (/build/crates/util/keccak-hasher)
#15 258.9    Compiling ethabi v12.0.0
#15 259.2    Compiling fastmap v0.1.0 (/build/crates/util/fastmap)
#15 259.3    Compiling ethash v1.12.0 (/build/crates/concensus/ethash)
#15 261.4    Compiling crossbeam-epoch v0.7.2
#15 262.1    Compiling parking_lot v0.6.4
#15 262.1    Compiling ethkey v0.3.0 (/build/crates/accounts/ethkey)
#15 262.6    Compiling ethabi-derive v11.0.0 (https://github.com/rimrakhimov/ethabi?branch=rimrakhimov/remove-syn-export-span#222e6482)
#15 263.3    Compiling memory-db v0.11.0 (/build/crates/db/memory-db)
#15 263.5    Compiling memory-cache v0.1.0 (/build/crates/util/memory-cache)
#15 263.7    Compiling ethereum-forkid v0.2.1
#15 264.1    Compiling patricia-trie-ethereum v0.1.0 (/build/crates/db/patricia-trie-ethereum)
#15 264.3    Compiling triehash-ethereum v0.2.0 (/build/crates/util/triehash-ethereum)
#15 264.5    Compiling eip-712 v0.1.0 (/build/crates/util/EIP-712)
#15 265.8    Compiling tokio-reactor v0.1.12
#15 267.3    Compiling crossbeam-deque v0.7.1
#15 267.5    Compiling crossbeam-deque v0.6.3
#15 267.6    Compiling kvdb-memorydb v0.1.0
#15 267.7    Compiling common-types v0.1.0 (/build/crates/ethcore/types)
#15 268.7    Compiling ethstore v0.2.1 (/build/crates/accounts/ethstore)
#15 270.5    Compiling tokio-udp v0.1.5
#15 271.0    Compiling tokio-tcp v0.1.3
#15 271.7    Compiling tokio-uds v0.2.5
#15 272.5    Compiling tokio-threadpool v0.1.18
#15 276.2    Compiling ethjson v0.1.0 (/build/crates/ethjson)
#15 276.7    Compiling ethcore-call-contract v0.1.0 (/build/crates/vm/call-contract)
#15 276.9    Compiling ethcore-accounts v0.1.0 (/build/crates/accounts)
#15 279.2    Compiling tokio-fs v0.1.6
#15 279.9    Compiling rayon v1.2.0
#15 281.9    Compiling tokio v0.1.22
#15 282.5    Compiling vm v0.1.0 (/build/crates/vm/vm)
#15 283.8    Compiling jsonrpc-server-utils v15.0.0
#15 286.1    Compiling ethcore-io v1.12.0 (/build/crates/runtime/io)
#15 286.6    Compiling tokio-core v0.1.17
#15 289.0    Compiling parity-runtime v0.1.0 (/build/crates/runtime/runtime)
#15 289.7    Compiling tokio-named-pipes v0.1.0
#15 289.8    Compiling wasm v0.1.0 (/build/crates/vm/wasm)
#15 290.4    Compiling jsonrpc-tcp-server v15.0.0
#15 291.6    Compiling jsonrpc-ws-server v15.0.0
#15 293.3    Compiling hyper-rustls v0.16.1
#15 293.6    Compiling jsonrpc-http-server v15.0.0
#15 294.0    Compiling ethcore-network v1.12.0 (/build/crates/net/network)
#15 295.2    Compiling tokio-retry v0.1.1
#15 295.4    Compiling hyper v0.11.27
#15 296.6    Compiling parity-tokio-ipc v0.4.0
#15 297.2    Compiling ethcore-stratum v1.12.0 (/build/crates/concensus/miner/stratum)
#15 301.3    Compiling fetch v0.1.0 (/build/crates/net/fetch)
#15 302.9    Compiling jsonrpc-ipc-server v15.0.0
#15 303.9    Compiling igd v0.7.1
#15 312.2    Compiling price-info v1.12.0 (/build/crates/concensus/miner/price-info)
#15 312.4    Compiling oe-rpc-servers v0.0.0 (/build/crates/rpc-servers)
#15 312.4    Compiling ethcore-network-devp2p v1.12.0 (/build/crates/net/network-devp2p)
#15 312.5    Compiling ethcore-miner v1.12.0 (/build/crates/concensus/miner)
#15 348.2    Compiling ethcore-builtin v0.1.0 (/build/crates/vm/builtin)
#15 349.5    Compiling evm v0.1.0 (/build/crates/vm/evm)
#15 455.9    Compiling parity-rocksdb v0.5.1
#15 456.0    Compiling kvdb-rocksdb v0.1.6
#15 456.3    Compiling ethcore-db v0.1.0 (/build/crates/db/db)
#15 456.3    Compiling migration-rocksdb v0.1.0 (/build/crates/db/migration-rocksdb)
#15 457.0    Compiling journaldb v0.2.0 (/build/crates/db/journaldb)
#15 457.0    Compiling ethcore-blockchain v0.1.0 (/build/crates/ethcore/blockchain)
#15 457.2    Compiling parity-local-store v0.1.0 (/build/crates/concensus/miner/local-store)
#15 457.5    Compiling dir v0.1.2 (/build/crates/util/dir)
#15 457.7    Compiling ethcore v1.12.0 (/build/crates/ethcore)
#15 461.6    Compiling ethcore-sync v1.12.0 (/build/crates/ethcore/sync)
#15 461.6    Compiling node-filter v1.12.0 (/build/crates/net/node-filter)
#15 462.6    Compiling parity-rpc v1.12.0 (/build/crates/rpc)
#15 462.6    Compiling ethcore-service v0.1.0 (/build/crates/ethcore/service)
#15 472.5    Compiling parity-rpc-client v1.4.0 (/build/crates/util/cli-signer/rpc-client)
#15 478.4    Compiling cli-signer v1.4.0 (/build/crates/util/cli-signer)
#15 479.9    Compiling openethereum v3.3.6 (/build)
#15 757.4     Finished release [optimized] target(s) in 12m 37s
#15 DONE 757.6s
```


### Start Node (Leopold Staging)

```shell
2024-07-17 20:26:06 Loading config file from /home/parity/authority.toml
2024-07-17 20:26:07 2024-07-17 18:26:07 UTC Starting OpenEthereum/v3.3.6-stable/x86_64-linux-gnu/rustc1.62.1
2024-07-17 20:26:07 2024-07-17 18:26:07 UTC Hello!
2024-07-17 20:26:07 2024-07-17 18:26:07 UTC Keys path /home/parity/data/keys/leopold
2024-07-17 20:26:07 2024-07-17 18:26:07 UTC DB path /home/parity/data/chains/leopold/db/024f447b30f5b4b8
2024-07-17 20:26:07 2024-07-17 18:26:07 UTC State DB configuration: fast
2024-07-17 20:26:07 2024-07-17 18:26:07 UTC Operating mode: active
2024-07-17 20:26:07 2024-07-17 18:26:07 UTC Not preparing block; cannot sign.
2024-07-17 20:26:07 2024-07-17 18:26:07 UTC Configured for leopold using AuthorityRound engine
2024-07-17 20:26:09 2024-07-17 18:26:09 UTC Running without a persistent transaction queue.
2024-07-17 20:26:14 2024-07-17 18:26:14 UTC Public node URL: enode://c9dada15ff32565d0213263db8948e52731d2009cc83b3bb038ad9e36d05088d9bae1381ede5bd60d2dd5046844cb6a2420d4faa306010138a17f1ebcd095b82@172.28.0.3:30303
2024-07-17 20:26:44 2024-07-17 18:26:44 UTC   2/ 2 peers     76 KiB chain  0 bytes queue  RPC:  0 conn,    1 req/s,  544 µs
2024-07-17 20:27:14 2024-07-17 18:27:14 UTC   2/ 2 peers     76 KiB chain  0 bytes queue  RPC:  0 conn,    1 req/s,  489 µs
2024-07-17 20:27:44 2024-07-17 18:27:44 UTC   2/ 2 peers     76 KiB chain  0 bytes queue  RPC:  0 conn,    1 req/s,  494 µs
2024-07-17 20:27:51 2024-07-17 18:27:51 UTC Imported #559940 0x868a…291b (0 txs, 0.00 Mgas, 3 ms, 2.67 KiB) + another 1 block(s) containing 0 tx(s)
```
