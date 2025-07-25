// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use std::{collections::HashSet, fmt, fs, num::NonZeroU32, str, time::Duration};

use crate::{
    miner::{
        gas_price_calibrator::{GasPriceCalibrator, GasPriceCalibratorOptions},
        gas_pricer::GasPricer,
    },
    user_defaults::UserDefaults,
};
use ethcore::{
    client::Mode,
    ethereum,
    spec::{Spec, SpecParams},
};
use ethereum_types::{Address, U256};
use fetch::Client as FetchClient;
use journaldb::Algorithm;
use parity_runtime::Executor;
use parity_version::version_data;

use crate::configuration;

#[derive(Debug, PartialEq, Default)]
pub enum SpecType {
    #[default]
    Foundation,
    Poanet,
    Xdai,
    Volta,
    Ewc,
    Musicoin,
    Ellaism,
    Mix,
    Callisto,
    Morden,
    Ropsten,
    Kovan,
    Rinkeby,
    Goerli,
    Sokol,
    Yolo3,
    Dev,
    Custom(String),
}

impl str::FromStr for SpecType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let spec = match s {
            "eth" | "ethereum" | "foundation" | "mainnet" => SpecType::Foundation,
            "poanet" | "poacore" => SpecType::Poanet,
            "xdai" => SpecType::Xdai,
            "volta" => SpecType::Volta,
            "ewc" | "energyweb" => SpecType::Ewc,
            "musicoin" => SpecType::Musicoin,
            "ellaism" => SpecType::Ellaism,
            "mix" => SpecType::Mix,
            "callisto" => SpecType::Callisto,
            "morden" => SpecType::Morden,
            "ropsten" => SpecType::Ropsten,
            "kovan" => SpecType::Kovan,
            "rinkeby" => SpecType::Rinkeby,
            "goerli" | "görli" | "testnet" => SpecType::Goerli,
            "sokol" | "poasokol" => SpecType::Sokol,
            "yolo3" => SpecType::Yolo3,
            "dev" => SpecType::Dev,
            other => SpecType::Custom(other.into()),
        };
        Ok(spec)
    }
}

impl fmt::Display for SpecType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            SpecType::Foundation => "foundation",
            SpecType::Poanet => "poanet",
            SpecType::Xdai => "xdai",
            SpecType::Volta => "volta",
            SpecType::Ewc => "energyweb",
            SpecType::Musicoin => "musicoin",
            SpecType::Ellaism => "ellaism",
            SpecType::Mix => "mix",
            SpecType::Callisto => "callisto",
            SpecType::Morden => "morden",
            SpecType::Ropsten => "ropsten",
            SpecType::Kovan => "kovan",
            SpecType::Rinkeby => "rinkeby",
            SpecType::Goerli => "goerli",
            SpecType::Sokol => "sokol",
            SpecType::Yolo3 => "yolo3",
            SpecType::Dev => "dev",
            SpecType::Custom(ref custom) => custom,
        })
    }
}

impl SpecType {
    pub fn spec<'a, T: Into<SpecParams<'a>>>(&self, params: T) -> Result<Spec, String> {
        let params = params.into();
        match *self {
            SpecType::Foundation => Ok(ethereum::new_foundation(params)),
            SpecType::Poanet => Ok(ethereum::new_poanet(params)),
            SpecType::Xdai => Ok(ethereum::new_xdai(params)),
            SpecType::Volta => Ok(ethereum::new_volta(params)),
            SpecType::Ewc => Ok(ethereum::new_ewc(params)),
            SpecType::Musicoin => Ok(ethereum::new_musicoin(params)),
            SpecType::Ellaism => Ok(ethereum::new_ellaism(params)),
            SpecType::Mix => Ok(ethereum::new_mix(params)),
            SpecType::Callisto => Ok(ethereum::new_callisto(params)),
            SpecType::Morden => Ok(ethereum::new_morden(params)),
            SpecType::Ropsten => Ok(ethereum::new_ropsten(params)),
            SpecType::Kovan => Ok(ethereum::new_kovan(params)),
            SpecType::Rinkeby => Ok(ethereum::new_rinkeby(params)),
            SpecType::Goerli => Ok(ethereum::new_goerli(params)),
            SpecType::Sokol => Ok(ethereum::new_sokol(params)),
            SpecType::Yolo3 => Ok(ethereum::new_yolo3(params)),
            SpecType::Dev => Ok(Spec::new_instant()),
            SpecType::Custom(ref filename) => {
                let file = fs::File::open(filename)
                    .map_err(|e| format!("Could not load specification file at {filename}: {e}"))?;
                Spec::load(params, file)
            }
        }
    }

    pub fn legacy_fork_name(&self) -> Option<String> {
        match *self {
            SpecType::Musicoin => Some("musicoin".to_owned()),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub enum Pruning {
    Specific(Algorithm),
    #[default]
    Auto,
}

impl str::FromStr for Pruning {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Pruning::Auto),
            other => other.parse().map(Pruning::Specific),
        }
    }
}

impl Pruning {
    pub fn to_algorithm(&self, user_defaults: &UserDefaults) -> Algorithm {
        match *self {
            Pruning::Specific(algo) => algo,
            Pruning::Auto => user_defaults.pruning,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ResealPolicy {
    pub own: bool,
    pub external: bool,
}

impl Default for ResealPolicy {
    fn default() -> Self {
        ResealPolicy {
            own: true,
            external: true,
        }
    }
}

impl str::FromStr for ResealPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (own, external) = match s {
            "none" => (false, false),
            "own" => (true, false),
            "ext" => (false, true),
            "all" => (true, true),
            x => return Err(format!("Invalid reseal value: {x}")),
        };

        let reseal = ResealPolicy { own, external };

        Ok(reseal)
    }
}

#[derive(Debug, PartialEq)]
pub struct AccountsConfig {
    pub iterations: NonZeroU32,
    pub refresh_time: u64,
    pub testnet: bool,
    pub password_files: Vec<String>,
    pub unlocked_accounts: Vec<Address>,
    pub enable_fast_unlock: bool,
}

impl Default for AccountsConfig {
    fn default() -> Self {
        AccountsConfig {
            iterations: NonZeroU32::new(10240).expect("10240 > 0; qed"),
            refresh_time: 5,
            testnet: false,
            password_files: Vec::new(),
            unlocked_accounts: Vec::new(),
            enable_fast_unlock: false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GasPricerConfig {
    Fixed(U256),
    Calibrated {
        usd_per_tx: f32,
        recalibration_period: Duration,
        api_endpoint: String,
    },
}

impl Default for GasPricerConfig {
    fn default() -> Self {
        GasPricerConfig::Calibrated {
            usd_per_tx: 0.0001f32,
            recalibration_period: Duration::from_secs(3600),
            api_endpoint: configuration::ETHERSCAN_ETH_PRICE_ENDPOINT.to_string(),
        }
    }
}

impl GasPricerConfig {
    pub fn to_gas_pricer(&self, fetch: FetchClient, p: Executor) -> GasPricer {
        match *self {
            GasPricerConfig::Fixed(u) => GasPricer::Fixed(u),
            GasPricerConfig::Calibrated {
                usd_per_tx,
                recalibration_period,
                ref api_endpoint,
            } => GasPricer::new_calibrated(GasPriceCalibrator::new(
                GasPriceCalibratorOptions {
                    usd_per_tx,
                    recalibration_period,
                },
                fetch,
                p,
                api_endpoint.clone(),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MinerExtras {
    pub author: Address,
    pub engine_signer: Address,
    pub extra_data: Vec<u8>,
    pub gas_range_target: (U256, U256),
    pub work_notify: Vec<String>,
    pub local_accounts: HashSet<Address>,
}

impl Default for MinerExtras {
    fn default() -> Self {
        MinerExtras {
            author: Default::default(),
            engine_signer: Default::default(),
            extra_data: version_data(),
            gas_range_target: (8_000_000.into(), 10_000_000.into()),
            work_notify: Default::default(),
            local_accounts: Default::default(),
        }
    }
}

/// 3-value enum.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Switch {
    /// True.
    On,
    /// False.
    Off,
    /// Auto.
    #[default]
    Auto,
}

impl str::FromStr for Switch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(Switch::On),
            "off" => Ok(Switch::Off),
            "auto" => Ok(Switch::Auto),
            other => Err(format!("Invalid switch value: {other}")),
        }
    }
}

pub fn tracing_switch_to_bool(
    switch: Switch,
    user_defaults: &UserDefaults,
) -> Result<bool, String> {
    match (user_defaults.is_first_launch, switch, user_defaults.tracing) {
        (false, Switch::On, false) => Err("TraceDB resync required".into()),
        (_, Switch::On, _) => Ok(true),
        (_, Switch::Off, _) => Ok(false),
        (_, Switch::Auto, def) => Ok(def),
    }
}

pub fn fatdb_switch_to_bool(
    switch: Switch,
    user_defaults: &UserDefaults,
    _algorithm: Algorithm,
) -> Result<bool, String> {
    match (user_defaults.is_first_launch, switch, user_defaults.fat_db) {
        (false, Switch::On, false) => Err("FatDB resync required".into()),
        (_, Switch::On, _) => Ok(true),
        (_, Switch::Off, _) => Ok(false),
        (_, Switch::Auto, def) => Ok(def),
    }
}

pub fn mode_switch_to_bool(
    switch: Option<Mode>,
    user_defaults: &UserDefaults,
) -> Result<Mode, String> {
    Ok(switch.unwrap_or(user_defaults.mode().clone()))
}

#[cfg(test)]
mod tests {
    use super::{tracing_switch_to_bool, Pruning, ResealPolicy, SpecType, Switch};
    use crate::user_defaults::UserDefaults;
    use journaldb::Algorithm;

    #[test]
    fn test_spec_type_parsing() {
        assert_eq!(SpecType::Foundation, "eth".parse().unwrap());
        assert_eq!(SpecType::Foundation, "ethereum".parse().unwrap());
        assert_eq!(SpecType::Foundation, "foundation".parse().unwrap());
        assert_eq!(SpecType::Foundation, "mainnet".parse().unwrap());
        assert_eq!(SpecType::Poanet, "poanet".parse().unwrap());
        assert_eq!(SpecType::Poanet, "poacore".parse().unwrap());
        assert_eq!(SpecType::Xdai, "xdai".parse().unwrap());
        assert_eq!(SpecType::Volta, "volta".parse().unwrap());
        assert_eq!(SpecType::Ewc, "ewc".parse().unwrap());
        assert_eq!(SpecType::Ewc, "energyweb".parse().unwrap());
        assert_eq!(SpecType::Musicoin, "musicoin".parse().unwrap());
        assert_eq!(SpecType::Ellaism, "ellaism".parse().unwrap());
        assert_eq!(SpecType::Mix, "mix".parse().unwrap());
        assert_eq!(SpecType::Callisto, "callisto".parse().unwrap());
        assert_eq!(SpecType::Morden, "morden".parse().unwrap());
        assert_eq!(SpecType::Ropsten, "ropsten".parse().unwrap());
        assert_eq!(SpecType::Kovan, "kovan".parse().unwrap());
        assert_eq!(SpecType::Rinkeby, "rinkeby".parse().unwrap());
        assert_eq!(SpecType::Goerli, "goerli".parse().unwrap());
        assert_eq!(SpecType::Goerli, "görli".parse().unwrap());
        assert_eq!(SpecType::Goerli, "testnet".parse().unwrap());
        assert_eq!(SpecType::Sokol, "sokol".parse().unwrap());
        assert_eq!(SpecType::Sokol, "poasokol".parse().unwrap());
    }

    #[test]
    fn test_spec_type_default() {
        assert_eq!(SpecType::Foundation, SpecType::default());
    }

    #[test]
    fn test_spec_type_display() {
        assert_eq!(format!("{}", SpecType::Foundation), "foundation");
        assert_eq!(format!("{}", SpecType::Poanet), "poanet");
        assert_eq!(format!("{}", SpecType::Xdai), "xdai");
        assert_eq!(format!("{}", SpecType::Volta), "volta");
        assert_eq!(format!("{}", SpecType::Ewc), "energyweb");
        assert_eq!(format!("{}", SpecType::Musicoin), "musicoin");
        assert_eq!(format!("{}", SpecType::Ellaism), "ellaism");
        assert_eq!(format!("{}", SpecType::Mix), "mix");
        assert_eq!(format!("{}", SpecType::Callisto), "callisto");
        assert_eq!(format!("{}", SpecType::Morden), "morden");
        assert_eq!(format!("{}", SpecType::Ropsten), "ropsten");
        assert_eq!(format!("{}", SpecType::Kovan), "kovan");
        assert_eq!(format!("{}", SpecType::Rinkeby), "rinkeby");
        assert_eq!(format!("{}", SpecType::Goerli), "goerli");
        assert_eq!(format!("{}", SpecType::Sokol), "sokol");
        assert_eq!(format!("{}", SpecType::Dev), "dev");
        assert_eq!(format!("{}", SpecType::Custom("foo/bar".into())), "foo/bar");
    }

    #[test]
    fn test_pruning_parsing() {
        assert_eq!(Pruning::Auto, "auto".parse().unwrap());
        assert_eq!(
            Pruning::Specific(Algorithm::Archive),
            "archive".parse().unwrap()
        );
        assert_eq!(
            Pruning::Specific(Algorithm::EarlyMerge),
            "light".parse().unwrap()
        );
        assert_eq!(
            Pruning::Specific(Algorithm::OverlayRecent),
            "fast".parse().unwrap()
        );
        assert_eq!(
            Pruning::Specific(Algorithm::RefCounted),
            "basic".parse().unwrap()
        );
    }

    #[test]
    fn test_pruning_default() {
        assert_eq!(Pruning::Auto, Pruning::default());
    }

    #[test]
    fn test_reseal_policy_parsing() {
        let none = ResealPolicy {
            own: false,
            external: false,
        };
        let own = ResealPolicy {
            own: true,
            external: false,
        };
        let ext = ResealPolicy {
            own: false,
            external: true,
        };
        let all = ResealPolicy {
            own: true,
            external: true,
        };
        assert_eq!(none, "none".parse().unwrap());
        assert_eq!(own, "own".parse().unwrap());
        assert_eq!(ext, "ext".parse().unwrap());
        assert_eq!(all, "all".parse().unwrap());
    }

    #[test]
    fn test_reseal_policy_default() {
        let all = ResealPolicy {
            own: true,
            external: true,
        };
        assert_eq!(all, ResealPolicy::default());
    }

    #[test]
    fn test_switch_parsing() {
        assert_eq!(Switch::On, "on".parse().unwrap());
        assert_eq!(Switch::Off, "off".parse().unwrap());
        assert_eq!(Switch::Auto, "auto".parse().unwrap());
    }

    #[test]
    fn test_switch_default() {
        assert_eq!(Switch::default(), Switch::Auto);
    }

    fn user_defaults_with_tracing(first_launch: bool, tracing: bool) -> UserDefaults {
        let mut ud = UserDefaults::default();
        ud.is_first_launch = first_launch;
        ud.tracing = tracing;
        ud
    }

    #[test]
    fn test_switch_to_bool() {
        assert!(
            !tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(true, true)).unwrap()
        );
        assert!(
            !tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(true, false)).unwrap()
        );
        assert!(
            !tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(false, true)).unwrap()
        );
        assert!(
            !tracing_switch_to_bool(Switch::Off, &user_defaults_with_tracing(false, false))
                .unwrap()
        );

        assert!(
            tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(true, true)).unwrap()
        );
        assert!(
            tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(true, false)).unwrap()
        );
        assert!(
            tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(false, true)).unwrap()
        );
        assert!(
            tracing_switch_to_bool(Switch::On, &user_defaults_with_tracing(false, false)).is_err()
        );
    }
}
