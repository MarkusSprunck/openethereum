# CHANGELOG

## OpenEthereum v3.5.1

This release is based on v3.5.0 and serves as a maintenance update focused on
security hardening, major RPC stack modernization, and Rust toolchain upgrade.

Enhancements

* Upgrade Rust toolchain from 1.88 to 1.97.1 (`scripts/setup-rust-1.97.1.sh`)
* Migrate JSON-RPC stack from `jsonrpc-*` v15 to v18
* Migrate all RPC code from `futures` 0.1 to `futures` 0.3 + async/await
* Replace `lru-cache 0.1` with `lru 0.7.8` across all dependent crates
* Replace `cargo-tarpaulin` (Linux-only) with `cargo-llvm-cov` for cross-platform code coverage with HTML and branch coverage reports

DevOps

* Add CI workflow `docker-ubuntu-latest.yml` (triggered on push to `main`, pushes tag `latest-rust-1.97`)
* Add CI workflow `docker-ubuntu-release.yml` (triggered on tag `v*`, pushes versioned tags)
* Remove legacy CI workflows `docker-ubuntu-rust-1.88-latest.yml` and `docker-ubuntu-rust-1.88-release.yml`
* Add new Docker build image `.github/docker/ubuntu-rust-1.97.1/Dockerfile.ci`
* Add `.github/dependabot.yml` to prevent Dependabot from breaking the `parity-crypto`/yanked-aes dependency chain
* Remove CodeQL from both CI workflows (unstable autobuild, non-deterministic results)
* Add `AGENTS.md`, `.github/copilot-instructions.md`, and `.github/templates/agents.md` for AI coding agent guidance

Cleanup

* Fix 44 Rust 1.97 compiler warnings: `mismatched_lifetime_syntaxes` (38 sites across 23 files), `unused_parens` (5 sites), and `dead_code` (annotated or removed)
* Migrate `cli-signer` to `futures` 0.3 and `parity-rpc` edition to 2021
* Remove unmaintained `wee_alloc 0.4.5` from `parity-util-mem-compat`
* Remove `tokio 0.1.22`, `hyper 0.12.36`, `net2 0.2.39`, `parity-tokio-ipc 0.4`, `parity-ws 0.10.1`, `futures-cpupool` from dependency tree

Security fixes

* Created `crates/util/atty-compat` shim to fix RUSTSEC-2021-0017 (`atty` Windows CVE)
* Created `crates/util/tempdir-compat` shim to fix RUSTSEC-2021-0126 (`remove_dir_all` TOCTOU on Windows)
* Created `crates/util/lock-api-compat` shim to fix CVE-2020-35910..35914 (`lock_api` missing Send/Sync bounds)
* Created `crates/util/parity-util-mem-compat` shim to fix lru RUSTSEC vulnerabilities (Dependabot #12/#18) by upgrading `lru` from 0.5.3 to 0.7.8
* Upgraded `rpassword` from 1.0.2 to 7.5.0 to fix GHSA-2p6r-x3vv-xqm2
* Removed `h2 0.1.26` (CVE-2023-44487 HTTP/2 Rapid Reset), `time 0.1.45` (RUSTSEC-2020-0071), and `crossbeam-utils 0.7.2` from dependency tree via jsonrpc-* v18 migration

Bug fixes

* Fix flaky test `should_not_return_pending_external_transactions_with_too_low_priority_fee_if_priority_fees_are_enforced` (allocator-dependent tx eviction on Linux CI)

## OpenEthereum v3.5.0

This release is based on the last stable version, v3.4.0, and serves as a maintenance
update with various improvements, security patches, and enhancements.

Enhancements

* Update Rust Version from 1.79 to 1.88
* Update Rust Edition where possible
* Enable macOS development support (1.88-aarch64-apple-darwin)

DevOps

* Remove alpine image build
* Update build images
* Simplify testing and debugging with Leopold blockchain (Staging)

Cleanup

* Remove support for Windows Development

Security fixes

* Vendored `aes`, `aes-soft`, `block-cipher-trait`, `stream-cipher`, and `aesni` because the used versions were yanked
* Upgraded `hyper`, `tokio`, `time`, `prometheus`, `validator`, `validator_derive`, `crossbeam-channel`, and `generic-array` to fix several vulnerabilities

Bug fixes

* None

## OpenEthereum v3.4.0

This release is based on the last stable version, v3.3.5, and serves as a maintenance
update with various improvements, security patches, and enhancements. Key highlights
include the introduction of JSON logging support, migration to _Rust Version 1.79_
and several security fixes.

Enhancements

* Introduced JSON logging
* Added debug configurations for VSCode Debugging
* Prepared code coverage tool _cargo-tarpaulin_
* Added quality-of-life scripts for building, testing, and running the client
* Add testing support for Leopold PoA blockchain

DevOps

* Upgraded Rust to Version 1.79 by fixing runtime `mio` errors and resolving IPv6 discovery issues in test cases
* Migrated the Docker base image to a scratch image with static linking, optimizing for minimal size and security
* Activate Dependabot for automatic dependency updates

Cleanup

* Migrated to using the `substrate-bn` crate from crates.io instead of the GitHub repository
* Added a development profile without optimizations for faster compilation times
* Resolved several compiler warnings in new Rust Version
* Updated `num-bigint` and related types for future compatibility

Security fixes

* Removed the deprecated `failure` crate, replacing it with daemonize to mitigate critical vulnerabilities
* Updated `crossbeam-deque` and `crossbeam-utils` to version 0.8.20 to fix data race vulnerabilities
* Bumped the `time` crate to address a segmentation fault issue
* Updated `regex` and related dependencies to resolve a denial-of-service vulnerability
* Applied further minor version upgrades to dependencies to ensure better security
* Update Dockerfiles for more security

Bug fixes

* Resolved issues with test case in version 1.79.0
* Fix build for alpine images

<!-- markdownlint-disable -->
## OpenEthereum v3.3.5

Enhancements:
* Support for POSDAO contract hardfork (#633)
* Update rpc server (#619)

## OpenEthereum v3.3.4

Enhancements:
* EIP-712: Update logos and rewrite type parser (now builds on Rust 1.58.1) (#463)
* Handling of incoming transactions with maxFeePerGas lower than current baseFee (#604)
* Update transaction replacement (#607)

## OpenEthereum v3.3.3

Enhancements:
* Implement eip-3607 (#593)

Bug fixes:
* Add type field for legacy transactions in RPC calls (#580)
* Makes eth_mining to return False if not is not allowed to seal (#581)
* Made nodes data concatenate as RLP sequences instead of bytes (#598)

## OpenEthereum v3.3.2

Enhancements:
* London hardfork block: Sokol (24114400)

Bug fixes:
* Fix for maxPriorityFeePerGas overflow

## OpenEthereum v3.3.1

Enhancements:
* Add eth_maxPriorityFeePerGas implementation (#570)
* Add a bootnode for Kovan

Bug fixes:
* Fix for modexp overflow in debug mode (#578)

## OpenEthereum v3.3.0

Enhancements:
* Add `validateServiceTransactionsTransition` spec option to be able to enable additional checking of zero gas price transactions by block verifier

## OpenEthereum v3.3.0-rc.15

* Revert eip1559BaseFeeMinValue activation on xDai at London hardfork block

## OpenEthereum v3.3.0-rc.14

Enhancements:
* Add eip1559BaseFeeMinValue and eip1559BaseFeeMinValueTransition spec options
* Activate eip1559BaseFeeMinValue on xDai at London hardfork block (19040000), set it to 20 GWei
* Activate eip1559BaseFeeMinValue on POA Core at block 24199500 (November 8, 2021), set it to 10 GWei
* Delay difficulty bomb to June 2022 for Ethereum Mainnet (EIP-4345)

## OpenEthereum v3.3.0-rc.13

Enhancements:
* London hardfork block: POA Core (24090200)

## OpenEthereum v3.3.0-rc.12

Enhancements:
* London hardfork block: xDai (19040000)

## OpenEthereum v3.3.0-rc.11

Bug fixes:
* Ignore GetNodeData requests only for non-AuRa chains

## OpenEthereum v3.3.0-rc.10

Enhancements:
* Add eip1559FeeCollector and eip1559FeeCollectorTransition spec options

## OpenEthereum v3.3.0-rc.9

Bug fixes:
* Add service transactions support for EIP-1559
* Fix MinGasPrice config option for POSDAO and EIP-1559

Enhancements:
* min_gas_price becomes min_effective_priority_fee
* added version 4 for TxPermission contract

## OpenEthereum v3.3.0-rc.8

Bug fixes:
* Ignore GetNodeData requests (#519)

## OpenEthereum v3.3.0-rc.7

Bug fixes:
* GetPooledTransactions is sent in invalid form (wrong packet id)

## OpenEthereum v3.3.0-rc.6

Enhancements:
* London hardfork block: kovan (26741100) (#502)

## OpenEthereum v3.3.0-rc.4

Enhancements:
* London hardfork block: mainnet (12,965,000) (#475)
* Support for eth/66 protocol version (#465)
* Bump ethereum/tests to v9.0.3
* Add eth_feeHistory

Bug fixes:
* GetNodeData from eth63 is missing (#466)
* Effective gas price not omitting (#477)
* London support in openethereum-evm (#479)
* gasPrice is required field for Transaction object (#481)

## OpenEthereum v3.3.0-rc.3

Bug fixes:
* Add effective_gas_price to eth_getTransactionReceipt #445 (#450)
* Update eth_gasPrice to support EIP-1559 #449 (#458)
* eth_estimateGas returns "Requires higher than upper limit of X" after London Ropsten Hard Fork #459 (#460)

## OpenEthereum v3.3.0-rc.2

Enhancements:
* EIP-1559: Fee market change for ETH 1.0 chain
* EIP-3198: BASEFEE opcode
* EIP-3529: Reduction in gas refunds
* EIP-3541: Reject new contracts starting with the 0xEF byte
* Delay difficulty bomb to December 2021 (EIP-3554)
* London hardfork blocks: goerli (5,062,605), rinkeby (8,897,988), ropsten (10,499,401)
* Add chainspecs for aleut and baikal
* Bump ethereum/tests to v9.0.2

## OpenEthereum v3.2.6

Enhancement:
* Berlin hardfork blocks: poacore (21,364,900), poasokol (21,050,600)

## OpenEthereum v3.2.5

Bug fixes:
* Backport: Block sync stopped without any errors. #277 (#286)
* Strict memory order (#306)

Enhancements:
* Executable queue for ancient blocks inclusion (#208)
* Backport AuRa commits for xdai (#330)
* Add Nethermind to clients that accept service transactions (#324)
* Implement the filter argument in parity_pendingTransactions (#295)
* Ethereum-types and various libs upgraded (#315)
* [evmbin] Omit storage output, now for std-json (#311)
* Freeze pruning while creating snapshot (#205)
* AuRa multi block reward (#290)
* Improved metrics. DB read/write. prometheus prefix config (#240)
* Send RLPx auth in EIP-8 format (#287)
* rpc module reverted for RPC JSON api (#284)
* Revert "Remove eth/63 protocol version (#252)"
* Support for eth/65 protocol version (#366)
* Berlin hardfork blocks: kovan (24,770,900), xdai (16,101,500)
* Bump ethereum/tests to v8.0.3

devops:
* Upgrade docker alpine to `v1.13.2`. for rust `v1.47`.
* Send SIGTERM instead of SIGHUP to OE daemon (#317)

## OpenEthereum v3.2.4

* Fix for Typed transaction broadcast.

## OpenEthereum v3.2.3

* Hotfix for berlin consensus error.

## OpenEthereum v3.2.2-rc.1

Bug fixes:
* Backport: Block sync stopped without any errors. #277 (#286)
* Strict memory order (#306)

Enhancements:
* Executable queue for ancient blocks inclusion (#208)
* Backport AuRa commits for xdai (#330)
* Add Nethermind to clients that accept service transactions (#324)
* Implement the filter argument in parity_pendingTransactions (#295)
* Ethereum-types and various libs upgraded (#315)
* Bump ethereum/tests to v8.0.2
* [evmbin] Omit storage output, now for std-json (#311)
* Freeze pruning while creating snapshot (#205)
* AuRa multi block reward (#290)
* Improved metrics. DB read/write. prometheus prefix config (#240)
* Send RLPx auth in EIP-8 format (#287)
* rpc module reverted for RPC JSON api (#284)
* Revert "Remove eth/63 protocol version (#252)"

devops:
* Upgrade docker alpine to `v1.13.2`. for rust `v1.47`.
* Send SIGTERM instead of SIGHUP to OE daemon (#317)

## OpenEthereum v3.2.1

Hot fix issue, related to initial sync:
* Initial sync gets stuck. (#318)

## OpenEthereum v3.2.0

Bug fixes:
* Update EWF's chains with Istanbul transition block numbers (#11482) (#254)
* fix Supplied instant is later than self (#169)
* ethcore/snapshot: fix double-lock in Service::feed_chunk (#289)

Enhancements:
* Berlin hardfork blocks: mainnet (12,244,000), goerli (4,460,644), rinkeby (8,290,928) and ropsten (9,812,189)
* yolo3x spec (#241)
* EIP-2930 RPC support
* Remove eth/63 protocol version (#252)
* Snapshot manifest block added to prometheus (#232)
* EIP-1898: Allow default block parameter to be blockHash
* Change ProtocolId to U64
* Update ethereum/tests
