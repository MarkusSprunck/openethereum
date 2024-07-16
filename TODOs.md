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
  Downloaded adler32 v1.2.0
  Downloaded block-cipher-trait v0.6.2
  Downloaded ahash v0.2.19
  Downloaded aes-ctr v0.3.0
  Downloaded crunchy v0.1.6
  Downloaded const-random-macro v0.1.13
  Downloaded build_const v0.2.1
  Downloaded block-padding v0.1.4
  Downloaded atty v0.2.13
  Downloaded ahash v0.3.8
  Downloaded bit-set v0.4.0
  Downloaded crossbeam-queue v0.2.3
  Downloaded term_size v0.3.1
  Downloaded autocfg v0.1.7
  Downloaded bitflags v0.7.0
  Downloaded tokio-retry v0.1.1
  Downloaded ethabi-contract v11.0.0
  Downloaded adler v1.0.2
  Downloaded fs-swap v0.2.4
  Downloaded hash-db v0.11.0
  Downloaded enum_primitive v0.1.1
  Downloaded ansi_term v0.11.0
  Downloaded fake-simd v0.1.2
  Downloaded byte-tools v0.3.1
  Downloaded crypto-mac v0.7.0
  Downloaded ctr v0.3.2
  Downloaded tokio-named-pipes v0.1.0
  Downloaded crossbeam-deque v0.6.3
  Downloaded cmake v0.1.42
  Downloaded tokio-codec v0.1.1
  Downloaded bitflags v1.2.1
  Downloaded cfg-if v1.0.0
  Downloaded crunchy v0.2.2
  Downloaded home v0.5.1
  Downloaded cfg-if v0.1.10
  Downloaded addr2line v0.14.1
  Downloaded beef v0.5.1
  Downloaded try-lock v0.1.0
  Downloaded ethereum-forkid v0.2.1
  Downloaded fixed-hash v0.6.1
  Downloaded digest v0.8.1
  Downloaded generic-array v0.12.3
  Downloaded trace-time v0.1.2
  Downloaded arrayvec v0.5.1
  Downloaded want v0.0.4
  Downloaded elastic-array v0.10.2
  Downloaded validator v0.8.0
  Downloaded hmac v0.7.1
  Downloaded futures-cpupool v0.1.8
  Downloaded untrusted v0.6.2
  Downloaded either v1.5.3
  Downloaded failure_derive v0.1.8
  Downloaded edit-distance v2.1.0
  Downloaded home v0.3.4
  Downloaded ethereum-types v0.9.2
  Downloaded ethbloom v0.9.2
  Downloaded const-random v0.1.13
  Downloaded block-buffer v0.7.3
  Downloaded autocfg v1.0.0
  Downloaded byte-slice-cast v0.3.5
  Downloaded crc v1.8.1
  Downloaded fdlimit v0.1.1
  Downloaded block-modes v0.3.3
  Downloaded ansi_term v0.10.2
  Downloaded aho-corasick v0.6.10
  Downloaded byteorder v1.3.2
  Downloaded crossbeam-deque v0.7.1
  Downloaded base64 v0.10.1
  Downloaded docopt v1.1.0
  Downloaded impl-rlp v0.2.1
  Downloaded getrandom v0.1.13
  Downloaded crossbeam-queue v0.1.2
  Downloaded bit-vec v0.4.4
  Downloaded base64 v0.9.3
  Downloaded ct-logs v0.5.1
  Downloaded crossbeam-epoch v0.7.2
  Downloaded fnv v1.0.6
  Downloaded error-chain v0.12.1
  Downloaded xmltree v0.7.0
  Downloaded bytes v0.4.12
  Downloaded crossbeam-utils v0.7.2
  Downloaded hamming v0.1.3
  Downloaded heapsize v0.4.2
  Downloaded instant v0.1.9
  Downloaded cc v1.0.41
  Downloaded crossbeam-utils v0.8.6
  Downloaded tokio-fs v0.1.6
  Downloaded tokio-current-thread v0.1.6
  Downloaded thiserror v1.0.20
  Downloaded textwrap v0.11.0
  Downloaded textwrap v0.9.0
  Downloaded backtrace v0.3.56
  Downloaded try-lock v0.2.2
  Downloaded jsonrpc-ipc-server v15.0.0
  Downloaded tiny-keccak v1.5.0
  Downloaded failure v0.1.8
  Downloaded hex v0.4.3
  Downloaded gcc v0.3.55
  Downloaded triehash v0.5.0
  Downloaded ethabi v12.0.0
  Downloaded env_logger v0.5.13
  Downloaded want v0.2.0
  Downloaded mio-uds v0.6.7
  Downloaded kernel32-sys v0.2.2
  Downloaded winapi-build v0.1.1
  Downloaded derive_more v0.99.9
  Downloaded impl-codec v0.4.2
  Downloaded transient-hashmap v0.4.1
  Downloaded aes v0.3.2
  Downloaded interleaved-ordered v0.1.1
  Downloaded heck v0.3.1
  Downloaded aes-soft v0.3.3
  Downloaded if_chain v0.1.3
  Downloaded http-body v0.1.0
  Downloaded proc-macro-crate v0.1.4
  Downloaded getrandom v0.2.2
  Downloaded crossbeam-channel v0.5.2
  Downloaded crossbeam-utils v0.6.6
  Downloaded memmap v0.6.2
  Downloaded lock_api v0.4.3
  Downloaded lazy_static v1.4.0
  Downloaded thread_local v0.3.6
  Downloaded order-stat v0.1.3
  Downloaded num_cpus v1.11.0
  Downloaded slab v0.3.0
  Downloaded rprompt v1.0.3
  Downloaded tokio-timer v0.1.2
  Downloaded termcolor v1.0.5
  Downloaded arrayvec v0.4.12
  Downloaded hashbrown v0.8.2
  Downloaded getopts v0.2.21
  Downloaded tokio-udp v0.1.5
  Downloaded tokio-tcp v0.1.3
  Downloaded rand_xorshift v0.2.0
  Downloaded unicode-xid v0.1.0
  Downloaded log v0.4.8
  Downloaded rand_chacha v0.2.1
  Downloaded tokio-service v0.1.0
  Downloaded bitvec v0.17.4
  Downloaded rand_chacha v0.1.1
  Downloaded proc-macro-hack v0.5.19
  Downloaded rustc_version v0.2.3
  Downloaded remove_dir_all v0.5.2
  Downloaded parking_lot v0.9.0
  Downloaded matches v0.1.8
  Downloaded parity-util-mem v0.7.0
  Downloaded parity-rocksdb v0.5.1
  Downloaded globset v0.4.5
  Downloaded c2-chacha v0.2.3
  Downloaded hashbrown v0.6.3
  Downloaded aho-corasick v0.7.6
  Downloaded nan-preserving-float v0.1.0
  Downloaded version_check v0.1.5
  Downloaded vec_map v0.8.1
  Downloaded timer v0.2.0
  Downloaded thread_local v1.0.1
  Downloaded jsonrpc-tcp-server v15.0.0
  Downloaded subtle v1.0.0
  Downloaded primal v0.2.3
  Downloaded num-iter v0.1.39
  Downloaded maplit v1.0.2
  Downloaded hyper-rustls v0.16.1
  Downloaded rustc-hex v1.0.0
  Downloaded percent-encoding v1.0.1
  Downloaded pbkdf2 v0.3.0
  Downloaded parity-tokio-ipc v0.4.0
  Downloaded igd v0.7.1
  Downloaded ryu v1.0.2
  Downloaded xdg v2.2.0
  Downloaded rand_core v0.5.1
  Downloaded siphasher v0.1.3
  Downloaded parity-util-mem-derive v0.1.0
  Downloaded local-encoding v0.2.0
  Downloaded strsim v0.9.2
  Downloaded chrono v0.4.9
  Downloaded jsonrpc-pubsub v15.0.0
  Downloaded lock_api v0.3.4
  Downloaded scopeguard v0.3.3
  Downloaded rand_isaac v0.1.1
  Downloaded rand_hc v0.1.0
  Downloaded num-traits v0.1.43
  Downloaded primal-estimate v0.2.1
  Downloaded num-bigint v0.1.44
  Downloaded httparse v1.3.4
  Downloaded tokio-buf v0.1.1
  Downloaded pulldown-cmark v0.0.3
  Downloaded serde_json v1.0.41
  Downloaded thiserror-impl v1.0.20
  Downloaded h2 v0.1.26
  Downloaded tiny-keccak v2.0.2
  Downloaded tokio-executor v0.1.10
  Downloaded strsim v0.8.0
  Downloaded num-traits v0.2.8
  Downloaded itoa v0.4.4
  Downloaded rand_pcg v0.1.2
  Downloaded rustc-hex v2.1.0
  Downloaded futures v0.1.29
  Downloaded skeptic v0.4.0
  Downloaded smallvec v0.6.13
  Downloaded lazycell v1.2.1
  Downloaded trie-db v0.11.0
  Downloaded once_cell v1.4.0
  Downloaded unicase v2.5.1
  Downloaded vergen v0.1.1
  Downloaded validator_derive v0.8.0
  Downloaded utf8-ranges v1.0.4
  Downloaded mio-extras v2.0.5
  Downloaded tokio-uds v0.2.5
  Downloaded net2 v0.2.33
  Downloaded tokio-rustls v0.9.4
  Downloaded parking_lot_core v0.8.3
  Downloaded unicode-width v0.1.6
  Downloaded unicode-xid v0.2.0
  Downloaded relay v0.1.1
  Downloaded clap v2.33.0
  Downloaded jsonrpc-server-utils v15.0.0
  Downloaded scopeguard v1.1.0
  Downloaded rustc-serialize v0.3.25
  Downloaded humantime v1.3.0
  Downloaded rand v0.3.23
  Downloaded iovec v0.1.4
  Downloaded kvdb-rocksdb v0.1.6
  Downloaded serde v1.0.102
  Downloaded zeroize v1.2.0
  Downloaded rand_os v0.1.3
  Downloaded parity-daemonize v0.3.0
  Downloaded primal-bit v0.2.4
  Downloaded num-integer v0.1.41
  Downloaded ipnetwork v0.12.8
  Downloaded rand v0.4.6
  Downloaded parking_lot_core v0.6.2
  Downloaded walkdir v2.3.1
  Downloaded sct v0.5.0
  Downloaded tokio-reactor v0.1.12
  Downloaded parking_lot_core v0.3.1
  Downloaded syn v1.0.86
  Downloaded keccak-hash v0.5.1
  Downloaded parity-crypto v0.6.2
  Downloaded secp256k1 v0.17.2
  Downloaded scoped-tls v0.1.2
  Downloaded same-file v1.0.5
  Downloaded mime v0.3.14
  Downloaded memory_units v0.3.0
  Downloaded prometheus v0.9.0
  Downloaded rand_xorshift v0.1.1
  Downloaded kvdb-memorydb v0.1.0
  Downloaded tempfile v3.1.0
  Downloaded parking_lot v0.11.1
  Downloaded primal-check v0.2.3
  Downloaded rpassword v1.0.2
  Downloaded num v0.1.42
  Downloaded language-tags v0.2.2
  Downloaded mio v0.6.22
  Downloaded rand_core v0.4.2
  Downloaded ppv-lite86 v0.2.6
  Downloaded protobuf v2.16.2
  Downloaded parity-scale-codec v1.3.5
  Downloaded slab v0.4.2
  Downloaded memoffset v0.5.2
  Downloaded linked-hash-map v0.5.3
  Downloaded scrypt v0.2.0
  Downloaded logos v0.12.0
  Downloaded target_info v0.1.0
  Downloaded object v0.23.0
  Downloaded plain_hasher v0.2.2
  Downloaded unicode-bidi v0.3.4
  Downloaded typenum v1.11.2
  Downloaded time v0.1.42
  Downloaded quick-error v1.2.2
  Downloaded rand_core v0.3.1
  Downloaded percent-encoding v2.1.0
  Downloaded bstr v0.2.8
  Downloaded rlp v0.4.6
  Downloaded parity-snappy v0.1.0
  Downloaded quote v1.0.7
  Downloaded sha-1 v0.8.1
  Downloaded radium v0.3.0
  Downloaded parity-path v0.1.2
  Downloaded miow v0.3.7
  Downloaded serde_derive v1.0.102
  Downloaded proc-macro2 v1.0.36
  Downloaded logos-derive v0.12.0
  Downloaded rustls v0.15.2
  Downloaded rand_jitter v0.1.4
  Downloaded kvdb v0.1.1
  Downloaded slab v0.2.0
  Downloaded number_prefix v0.2.8
  Downloaded rlp-derive v0.1.0
  Downloaded primitive-types v0.7.2
  Downloaded string v0.2.1
  Downloaded stable_deref_trait v1.1.1
  Downloaded opaque-debug v0.2.3
  Downloaded parity-bytes v0.1.1
  Downloaded rayon v1.2.0
  Downloaded lock_api v0.1.5
  Downloaded subtle v2.3.0
  Downloaded ucd-util v0.1.8
  Downloaded safemem v0.3.3
  Downloaded rustc-demangle v0.1.16
  Downloaded regex v1.3.9
  Downloaded parity-wordlist v1.3.0
  Downloaded tempdir v0.3.7
  Downloaded inflate v0.4.5
  Downloaded lru v0.5.3
  Downloaded tokio-timer v0.2.13
  Downloaded serde_repr v0.1.6
  Downloaded ripemd160 v0.8.0
  Downloaded lru-cache v0.1.2
  Downloaded primal-sieve v0.2.9
  Downloaded parity-wasm v0.31.3
  Downloaded jsonrpc-core v15.0.0
  Downloaded parity-ws v0.10.0
  Downloaded rand v0.6.5
  Downloaded impl-trait-for-tuples v0.1.3
  Downloaded regex v0.2.11
  Downloaded semver-parser v0.7.0
  Downloaded rand v0.7.3
  Downloaded mio-named-pipes v0.1.6
  Downloaded semver v0.9.0
  Downloaded rand v0.5.6
  Downloaded gimli v0.23.0
  Downloaded quote v0.6.13
  Downloaded jsonrpc-http-server v15.0.0
  Downloaded uint v0.8.5
  Downloaded proc-macro2 v0.4.30
  Downloaded jsonrpc-derive v15.0.0
  Downloaded indexmap v1.3.0
  Downloaded parking_lot v0.10.2
  Downloaded static_assertions v1.1.0
  Downloaded parity-snappy-sys v0.1.2
  Downloaded tokio-io v0.1.12
  Downloaded spin v0.5.2
  Downloaded nodrop v0.1.14
  Downloaded sha2 v0.8.0
  Downloaded owning_ref v0.4.0
  Downloaded maybe-uninit v2.0.0
  Downloaded synstructure v0.12.2
  Downloaded stream-cipher v0.3.2
  Downloaded tokio-sync v0.1.7
  Downloaded xml-rs v0.7.0
  Downloaded url v2.1.0
  Downloaded toml v0.4.10
  Downloaded pwasm-utils v0.6.2
  Downloaded jsonrpc-ws-server v15.0.0
  Downloaded tokio v0.1.22
  Downloaded itertools v0.5.10
  Downloaded smallvec v1.6.1
  Downloaded memchr v2.2.1
  Downloaded unicode-normalization v0.1.8
  Downloaded tokio-threadpool v0.1.18
  Downloaded toml v0.5.5
  Downloaded impl-serde v0.3.1
  Downloaded parking_lot v0.6.4
  Downloaded webpki v0.19.1
  Downloaded wasmi v0.3.0
  Downloaded tokio-core v0.1.17
  Downloaded parking_lot_core v0.7.2
  Downloaded miniz_oxide v0.4.4
  Downloaded url v1.7.2
  Downloaded itertools v0.7.11
  Downloaded ring v0.14.6
  Downloaded rayon-core v1.6.0
  Downloaded unicode-segmentation v1.5.0
  Downloaded http v0.1.21
  Downloaded num-bigint v0.2.3
  Downloaded hyper v0.12.35
  Downloaded hyper v0.11.27
  Downloaded syn v0.15.26
  Downloaded secp256k1-sys v0.1.2
  Downloaded regex-syntax v0.5.6
  Downloaded webpki-roots v0.16.0
  Downloaded idna v0.2.0
  Downloaded idna v0.1.5
  Downloaded regex-syntax v0.6.18
  Downloaded winapi v0.2.8
  Downloaded libc v0.2.89
  Downloaded rust-crypto v0.2.36
  Downloaded winapi v0.3.8
  Downloaded parity-rocksdb-sys v0.5.6
   Compiling parity-version v3.3.6 (/tmp/44d4a1f8-4d7e-4541-b879-2a728fc0ded4/crates/util/version)
   Compiling parity-rpc v1.12.0 (/tmp/44d4a1f8-4d7e-4541-b879-2a728fc0ded4/crates/rpc)
   Compiling parity-rpc-client v1.4.0 (/tmp/44d4a1f8-4d7e-4541-b879-2a728fc0ded4/crates/util/cli-signer/rpc-client)
   Compiling cli-signer v1.4.0 (/tmp/44d4a1f8-4d7e-4541-b879-2a728fc0ded4/crates/util/cli-signer)
   Compiling openethereum v3.3.6-rc0 (/tmp/44d4a1f8-4d7e-4541-b879-2a728fc0ded4)
    Finished release [optimized] target(s) in 7m 23s

Process finished with exit code 0
```


### Start Node (Leopold Staging)

```shell
2024-07-16 16:12:24 Loading config file from /home/parity/authority.toml
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC Starting OpenEthereum/v3.3.6-stable-36742adda-20240715/x86_64-linux-gnu/rustc1.61.0
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC Hello!
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC Keys path /home/parity/data/keys/leopold
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC DB path /home/parity/data/chains/leopold/db/024f447b30f5b4b8
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC State DB configuration: fast
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC Operating mode: active
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC Not preparing block; cannot sign.
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC Configured for leopold using AuthorityRound engine
2024-07-16 16:12:24 2024-07-16 14:12:24 UTC Running without a persistent transaction queue.
2024-07-16 16:12:30 2024-07-16 14:12:30 UTC Public node URL: enode://8d8dcfd66723689469ba1d64e08a9fc12b9e20a7f416c7a656078491f24396c70517a109bea773a473a5aa60b804563e35def8197fe58cd879d36cc635c20685@172.25.0.3:30303
2024-07-16 16:14:27 2024-07-16 14:14:27 UTC Signal for switch to contract-based validator set.
2024-07-16 16:14:27 2024-07-16 14:14:27 UTC Initial contract validators: [0x0065916b857b61089d8162132f3f04eebf4a4cce, 0x008beefdff2e550207cc17990b03abd18d7fb197]
2024-07-16 16:14:27 2024-07-16 14:14:27 UTC Applying validator set change signalled at block 34
2024-07-16 16:14:29 2024-07-16 14:14:29 UTC Syncing     #588 0xaff8…17bc   117.58 blk/s   96.4 tx/s    5.5 Mgas/s      0+ 1316 Qed LI:#1905    2/ 2 peers    263 KiB chain    7 MiB queue  RPC:  0 conn,    1 req/s,  152 µs
2024-07-16 16:14:30 2024-07-16 14:14:30 UTC Signal for transition within contract. New list: [0x0065916b857b61089d8162132f3f04eebf4a4cce, 0x008beefdff2e550207cc17990b03abd18d7fb197, 0x001ca517d9f0bdee7906f49295a892ac7151f101]
2024-07-16 16:14:30 2024-07-16 14:14:30 UTC Applying validator set change signalled at block 1061
2024-07-16 16:14:34 2024-07-16 14:14:34 UTC Syncing    #2522 0x2939…2b0c   386.72 blk/s  119.4 tx/s    4.8 Mgas/s      0+ 3954 Qed LI:#6477    2/ 2 peers    908 KiB chain   23 MiB queue  RPC:  0 conn,    0 req/s,  155 µs
```
