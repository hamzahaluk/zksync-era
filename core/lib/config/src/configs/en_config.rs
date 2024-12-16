use std::{num::NonZeroUsize, time::Duration};

use smart_config::{
    de::{Optional, Serde},
    DescribeConfig, DeserializeConfig,
};
use zksync_basic_types::{
    commitment::L1BatchCommitmentMode, url::SensitiveUrl, L1ChainId, L2ChainId, SLChainId,
};

/// Temporary config for initializing external node, will be completely replaced by consensus config later
#[derive(Debug, Clone, PartialEq, DescribeConfig, DeserializeConfig)]
pub struct ENConfig {
    // Genesis
    #[config(with = Serde![int])]
    pub l2_chain_id: L2ChainId,
    #[config(with = Optional(Serde![int]))]
    pub sl_chain_id: Option<SLChainId>,
    #[config(with = Serde![int])]
    pub l1_chain_id: L1ChainId,
    #[config(default, with = Serde![str])]
    pub l1_batch_commit_data_generator_mode: L1BatchCommitmentMode,

    // Main node configuration
    #[config(secret, with = Serde![str])]
    pub main_node_url: SensitiveUrl,
    #[config(default_t = NonZeroUsize::new(100).unwrap())]
    pub main_node_rate_limit_rps: NonZeroUsize,
    #[config(secret, with = Optional(Serde![str]))]
    pub gateway_url: Option<SensitiveUrl>,
    pub bridge_addresses_refresh_interval: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use smart_config::{ConfigRepository, ConfigSchema, Environment, Yaml};

    use super::*;

    fn expected_config() -> ENConfig {
        ENConfig {
            l2_chain_id: L2ChainId::from(271),
            sl_chain_id: None,
            l1_chain_id: L1ChainId(9),
            l1_batch_commit_data_generator_mode: L1BatchCommitmentMode::Rollup,
            main_node_url: "http://127.0.0.1:3050/".parse().unwrap(),
            main_node_rate_limit_rps: NonZeroUsize::new(200).unwrap(),
            gateway_url: None,
            bridge_addresses_refresh_interval: Some(Duration::from_secs(15)),
        }
    }

    fn create_schema() -> ConfigSchema {
        let mut schema = ConfigSchema::default();
        schema
            .insert(&ENConfig::DESCRIPTION, "external_node")
            .unwrap()
            .push_alias("")
            .unwrap();
        schema
    }

    // FIXME: EN_BRIDGE_ADDRESSES_REFRESH_INTERVAL_SEC=15 doesn't work
    #[test]
    fn parsing_from_env() {
        let env = r#"
            EN_L1_CHAIN_ID=9
            EN_L2_CHAIN_ID=271
            EN_MAIN_NODE_URL=http://127.0.0.1:3050/
            EN_MAIN_NODE_RATE_LIMIT_RPS=200
            EN_L1_BATCH_COMMIT_DATA_GENERATOR_MODE=Rollup
            EN_BRIDGE_ADDRESSES_REFRESH_INTERVAL="15s"
        "#;
        let env = Environment::from_dotenv("test.env", env)
            .unwrap()
            .strip_prefix("EN_");

        let schema = create_schema();
        let repo = ConfigRepository::new(&schema).with(env);
        let config: ENConfig = repo.single().unwrap().parse().unwrap();
        assert_eq!(config, expected_config());
    }

    #[test]
    fn parsing_from_yaml() {
        let yaml = r#"
            main_node_url: http://127.0.0.1:3050/
            main_node_rate_limit_rps: 200
            gateway_url: null
            l2_chain_id: 271
            l1_chain_id: 9
            sl_chain_id: null
            l1_batch_commit_data_generator_mode: Rollup
            bridge_addresses_refresh_interval: '15s'
        "#;
        let yaml = Yaml::new("test.yml", serde_yaml::from_str(yaml).unwrap()).unwrap();

        let schema = create_schema();
        let repo = ConfigRepository::new(&schema).with(yaml);
        let config: ENConfig = repo.single().unwrap().parse().unwrap();
        assert_eq!(config, expected_config());
    }

    #[test]
    fn parsing_from_canonical_yaml() {
        let yaml = r#"
          external_node:
            main_node_url: http://127.0.0.1:3050/
            main_node_rate_limit_rps: 200
            gateway_url: null
            l2_chain_id: 271
            l1_chain_id: 9
            sl_chain_id: null
            l1_batch_commit_data_generator_mode: Rollup
            bridge_addresses_refresh_interval: '15s'
        "#;
        let yaml = Yaml::new("test.yml", serde_yaml::from_str(yaml).unwrap()).unwrap();

        let schema = create_schema();
        let repo = ConfigRepository::new(&schema).with(yaml);
        let config: ENConfig = repo.single().unwrap().parse().unwrap();
        assert_eq!(config, expected_config());
    }
}
