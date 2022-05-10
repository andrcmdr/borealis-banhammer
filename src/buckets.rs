use crate::de::Token;
use ethereum_types::Address;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr};

#[derive(Debug, Hash, Clone)]
pub enum BucketKind {
    IP,
    Address,
    Token,
}

#[derive(Debug, Hash, Clone)]
pub enum BucketValue {
    IP(IpAddr),
    Address(Address),
    Token(Token),
}

#[derive(Debug, Hash, Clone)]
pub struct BucketName {
    kind: BucketKind,
    value: BucketValue,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct BucketConfig {
    pub base_size: u64,
    pub leak_rate: u64,
    pub overflow_size: u64,
    pub retention: u64,
}
/*
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamedBucketConfig {
    pub name: String,
    pub base_size: u64,
    pub leak_rate: u64,
    pub overflow_size: u64,
    pub retention: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BucketsConfig {
    pub near_gas: BucketConfig,
    pub eth_gas: BucketConfig,
    pub free_gas: BucketConfig,
    pub default_relayer_err: BucketConfig,
    pub default_engine_err: BucketConfig,
    pub default_evm_revert: BucketConfig,
    pub relayer_errors: Vec<NamedBucketConfig>,
    pub engine_errors: Vec<NamedBucketConfig>,
    pub evm_reverts: Vec<NamedBucketConfig>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub struct Bucket {
    value: u64,
    #[serde(rename = "BaseSize")]
    base_size: u64,
    #[serde(rename = "LeakRate")]
    leak_rate: u64,
    #[serde(rename = "OverflowSize")]
    overflow_size: u64,
    #[serde(rename = "Retention")]
    retention: u64,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum BucketKind {
    /// Near gas unit is NEAR gas.
    NearGas(Bucket),
    /// ETH gas unit is ETH gas.
    EthGas(Bucket),
    /// Relayer error bucket values are per error.
    RelayerErrors(HashMap<String, Bucket>),
    /// Engine error bucket values are per error.
    EngineErrors(HashMap<String, Bucket>),
    /// Revert value is a single revert.
    Reverts(Bucket),
    /// Free gas value is a single transaction.
    FreeGas(Bucket),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Buckets(Vec<BucketKind>);

impl Buckets {
    pub fn new(config: BucketsConfig) -> Self {
        let buckets: Vec<BucketKind> = {
            let capacity = 6
                + config.relayer_errors.len()
                + config.engine_errors.len()
                + config.evm_reverts.len();
            Vec::with_capacity(capacity)
        };
        Buckets(buckets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thing() {
        let mut map = HashMap::new();
        let bucket = BucketConfig {
            base_size: 10,
            leak_rate: 1,
            overflow_size: 1,
            retention: 1000,
        };
        let named_bucket = NamedBucketConfig {
            name: "test".to_string(),
            base_size: 0,
            leak_rate: 0,
            overflow_size: 0,
            retention: 0,
        };
        map.insert("EXAMPLE_ERROR".to_string(), bucket);
        let buckets_config = BucketsConfig {
            near_gas: bucket,
            eth_gas: bucket,
            default_relayer_error: BucketConfig {
                base_size: 0,
                leak_rate: 0,
                overflow_size: 0,
                retention: 0,
            },
            default_engine_error: BucketConfig {
                base_size: 0,
                leak_rate: 0,
                overflow_size: 0,
                retention: 0,
            },
            default_evm_revert: BucketConfig {
                base_size: 0,
                leak_rate: 0,
                overflow_size: 0,
                retention: 0,
            },
            relayer_errors: vec![named_bucket.clone(), named_bucket.clone()],
            engine_errors: vec![named_bucket.clone(), named_bucket.clone()],
            evm_reverts: vec![named_bucket.clone(), named_bucket],
        };

        let toml = toml::to_string_pretty(&buckets_config).unwrap();
        println!("{}", toml);

        // let json = serde_json::to_string_pretty(&buckets).unwrap();
        // println!("{}", json);
    }
}
*/
