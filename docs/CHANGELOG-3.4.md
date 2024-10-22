## OpenEthereum v3.4.0

This release is based on the last stable version, v3.3.6, and serves as a maintenance update with various improvements, security patches, and enhancements. Key highlights include the introduction of JSON logging support, migration to _Rust Version 1.79_ and several security fixes.

### Enhancements

- Introduced JSON logging as `feature`.
- Added debug configurations for VSCode Debugging.
- Prepared code coverage tool `cargo-tarpaulin`.
- Added quality-of-life scripts for building, testing, and running the client.

### DevOps

-   Upgraded Rust to Version 1.79 by fixing runtime `mio` errors and resolving IPv6 discovery issues in test cases.
-   Migrated the Docker base image to a scratch image with static linking, optimizing for minimal size and security.
-   Activate Dependabot for automatic dependency updates.

### Cleanup

-   Migrated to using the `substrate-bn` crate from crates.io instead of the GitHub repository.
-   Added a development profile without optimizations for faster compilation times.
-   Resolved several compiler warnings in new Rust Version.
-   Updated `num-bigint` and related types for future compatibility.

### Security fixes

-   Removed the deprecated `failure` crate, replacing it with daemonize to mitigate critical vulnerabilities.
-   Updated `crossbeam-deque` and `crossbeam-utils` to version 0.8.20 to fix data race vulnerabilities.
-   Bumped the `time` crate to address a segmentation fault issue.
-   Updated `regex` and related dependencies to resolve a denial-of-service vulnerability.
-   Applied further minor version upgrades to dependencies to ensure better security...

### Bug fixes

-   Resolved issues with test case in version 1.79.
