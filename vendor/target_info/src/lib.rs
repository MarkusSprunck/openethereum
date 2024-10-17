//! Get information concerning the build target.

macro_rules! return_cfg {
	($i:ident : $s:expr) => ( if cfg!($i = $s) { return $s; } );
	($i:ident : $s:expr, $($t:expr),+) => ( return_cfg!($i: $s); return_cfg!($i: $($t),+) );
}

/// Collection of functions to give information on the build target.
pub struct Target;
impl Target {
	/// Architecture; given by `target_arch`.
	pub fn arch() -> &'static str {
		return_cfg!(target_arch: "x86", "x86_64", "mips", "powerpc", "arm", "aarch64");
		"unknown"
	}

	/// Endianness; given by `target_endian`.
	pub fn endian() -> &'static str {
		return_cfg!(target_endian: "little", "big");
		""
	}

	/// Toolchain environment; given by `target_environment`.
	pub fn env() -> &'static str {
		return_cfg!(target_env: "musl", "msvc", "gnu");
		""
	}

	/// OS familt; given by `target_family`.
	pub fn family() -> &'static str {
		return_cfg!(target_family: "unix", "windows");
		"unknown"
	}

	/// Operating system; given by `target_os`.
	pub fn os() -> &'static str {
		return_cfg!(target_os: "windows", "macos", "ios", "linux", "android", "freebsd", "dragonfly", "bitrig", "openbsd", "netbsd");
		"unknown"
	}

	/// Pointer width; given by `target_pointer_width`.
	pub fn pointer_width() -> &'static str {
		return_cfg!(target_pointer_width: "32", "64");
		"unknown"
	}

	// TODO: enable once it's not experimental API.
	// 	pub fn vendor() -> &'static str {
	// return_cfg!(target_vendor: "apple", "pc");
	// "unknown"
	// }
}

