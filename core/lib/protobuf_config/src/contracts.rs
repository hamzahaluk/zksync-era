use anyhow::Context as _;
use zksync_config::configs::{ContractsConfig, EcosystemContracts};
use zksync_protobuf::{repr::ProtoRepr, required};

use crate::{parse_h160, parse_h256, proto::contracts as proto};

impl ProtoRepr for proto::Contracts {
    type Type = ContractsConfig;

    fn read(&self) -> anyhow::Result<Self::Type> {
        let l1 = required(&self.l1).context("l1")?;
        let l2 = required(&self.l2).context("l2")?;
        let bridges = required(&self.bridges).context("bridges")?;
        let shared = required(&bridges.shared).context("shared")?;
        let erc20 = required(&bridges.erc20).context("erc20")?;
        let weth_bridge = &bridges.weth;

        let ecosystem_contracts = if let Some(ecosystem_contracts) = &self.ecosystem_contracts {
            Some(EcosystemContracts {
                bridgehub_proxy_addr: required(&ecosystem_contracts.bridgehub_proxy_addr)
                    .and_then(|x| parse_h160(x))
                    .context("bridgehub_proxy_addr")?,
                state_transition_proxy_addr: required(
                    &ecosystem_contracts.state_transition_proxy_addr,
                )
                .and_then(|x| parse_h160(x))
                .context("state_transition_proxy_addr")?,
                transparent_proxy_admin_addr: required(
                    &ecosystem_contracts.transparent_proxy_admin_addr,
                )
                .and_then(|x| parse_h160(x))
                .context("transparent_proxy_admin_addr")?,
                l1_bytecodes_supplier_addr: ecosystem_contracts
                    .l1_bytecodes_supplier_addr
                    .as_ref()
                    .map(|x| parse_h160(x).expect("Invalid address")),
            })
        } else {
            None
        };

        Ok(Self::Type {
            ecosystem_contracts,
            governance_addr: required(&l1.governance_addr)
                .and_then(|x| parse_h160(x))
                .context("governance_addr")?,
            verifier_addr: required(&l1.verifier_addr)
                .and_then(|x| parse_h160(x))
                .context("verifier_addr")?,
            default_upgrade_addr: required(&l1.default_upgrade_addr)
                .and_then(|x| parse_h160(x))
                .context("diamond_init_addr")?,
            diamond_proxy_addr: required(&l1.diamond_proxy_addr)
                .and_then(|x| parse_h160(x))
                .context("diamond_proxy_addr")?,
            validator_timelock_addr: required(&l1.validator_timelock_addr)
                .and_then(|x| parse_h160(x))
                .context("validator_timelock_addr")?,
            l1_erc20_bridge_proxy_addr: erc20
                .l1_address
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("l1_erc20_bridge_addr")?,
            l2_erc20_bridge_addr: erc20
                .l2_address
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("l2_erc20_bridge_addr")?,
            l1_shared_bridge_proxy_addr: shared
                .l1_address
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("l1_shared_bridge_proxy_addr")?,
            l2_shared_bridge_addr: shared
                .l2_address
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("l2_shared_bridge_addr")?,
            l2_legacy_shared_bridge_addr: l2
                .legacy_shared_bridge_addr
                .as_ref()
                .and_then(|x| (!x.is_empty()).then_some(x))
                .map(|x| parse_h160(x))
                .transpose()
                .context("l2_legacy_shared_bridge_addr")?,
            l1_weth_bridge_proxy_addr: weth_bridge
                .as_ref()
                .and_then(|bridge| bridge.l1_address.as_ref().map(|x| parse_h160(x)))
                .transpose()
                .context("l1_weth_bridge_addr")?,
            l2_weth_bridge_addr: weth_bridge
                .as_ref()
                .and_then(|bridge| bridge.l2_address.as_ref().map(|x| parse_h160(x)))
                .transpose()
                .context("l2_weth_bridge_addr")?,
            l2_testnet_paymaster_addr: l2
                .testnet_paymaster_addr
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("l2_testnet_paymaster_addr")?,
            l1_multicall3_addr: required(&l1.multicall3_addr)
                .and_then(|x| parse_h160(x))
                .context("l1_multicall3_addr")?,
            base_token_addr: l1
                .base_token_addr
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("base_token_addr")?,
            base_token_asset_id: l1
                .base_token_asset_id
                .as_ref()
                .map(|x| parse_h256(x))
                .transpose()
                .context("base_token_asset_id")?,
            predeployed_l2_weth_token_address: l1
                .predeployed_l2_weth_token_address
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("predeployed_l2_weth_token_address")?,
            user_facing_bridgehub_proxy_addr: self
                .user_facing_bridgehub
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("base_token_addr")?,
            user_facing_diamond_proxy_addr: self
                .user_facing_diamond_proxy
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("base_token_addr")?,
            chain_admin_addr: l1
                .chain_admin_addr
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("chain_admin_addr")?,
            settlement_layer: self.settlement_layer,
            l2_da_validator_addr: l2
                .da_validator_addr
                .as_ref()
                .map(|x| parse_h160(x))
                .transpose()
                .context("l2_da_validator_addr")?,
        })
    }

    fn build(this: &Self::Type) -> Self {
        let ecosystem_contracts = this
            .ecosystem_contracts
            .as_ref()
            .map(|ecosystem_contracts| proto::EcosystemContracts {
                bridgehub_proxy_addr: Some(format!(
                    "{:?}",
                    ecosystem_contracts.bridgehub_proxy_addr
                )),
                state_transition_proxy_addr: Some(format!(
                    "{:?}",
                    ecosystem_contracts.state_transition_proxy_addr
                )),
                transparent_proxy_admin_addr: Some(format!(
                    "{:?}",
                    ecosystem_contracts.transparent_proxy_admin_addr,
                )),
                l1_bytecodes_supplier_addr: ecosystem_contracts
                    .l1_bytecodes_supplier_addr
                    .map(|x| format!("{:?}", x)),
            });
        Self {
            ecosystem_contracts,
            l1: Some(proto::L1 {
                governance_addr: Some(format!("{:?}", this.governance_addr)),
                verifier_addr: Some(format!("{:?}", this.verifier_addr)),
                diamond_proxy_addr: Some(format!("{:?}", this.diamond_proxy_addr)),
                validator_timelock_addr: Some(format!("{:?}", this.validator_timelock_addr)),
                default_upgrade_addr: Some(format!("{:?}", this.default_upgrade_addr)),
                multicall3_addr: Some(format!("{:?}", this.l1_multicall3_addr)),
                base_token_addr: this.base_token_addr.map(|a| format!("{:?}", a)),
                base_token_asset_id: this.base_token_asset_id.map(|x| format!("{:?}", x)),
                chain_admin_addr: this.chain_admin_addr.map(|a| format!("{:?}", a)),
                predeployed_l2_weth_token_address: this
                    .predeployed_l2_weth_token_address
                    .map(|x| format!("{:?}", x)),
            }),
            l2: Some(proto::L2 {
                testnet_paymaster_addr: this.l2_testnet_paymaster_addr.map(|a| format!("{:?}", a)),
                da_validator_addr: this.l2_da_validator_addr.map(|a| format!("{:?}", a)),
                legacy_shared_bridge_addr: this
                    .l2_legacy_shared_bridge_addr
                    .map(|a| format!("{:?}", a)),
            }),
            bridges: Some(proto::Bridges {
                shared: Some(proto::Bridge {
                    l1_address: this.l1_shared_bridge_proxy_addr.map(|a| format!("{:?}", a)),
                    l2_address: this.l2_shared_bridge_addr.map(|a| format!("{:?}", a)),
                }),
                erc20: Some(proto::Bridge {
                    l1_address: this.l1_erc20_bridge_proxy_addr.map(|a| format!("{:?}", a)),
                    l2_address: this.l2_erc20_bridge_addr.map(|a| format!("{:?}", a)),
                }),
                weth: Some(proto::Bridge {
                    l1_address: this.l1_weth_bridge_proxy_addr.map(|a| format!("{:?}", a)),
                    l2_address: this.l2_weth_bridge_addr.map(|a| format!("{:?}", a)),
                }),
            }),
            user_facing_bridgehub: this
                .user_facing_bridgehub_proxy_addr
                .map(|a| format!("{:?}", a)),
            user_facing_diamond_proxy: this
                .user_facing_diamond_proxy_addr
                .map(|a| format!("{:?}", a)),
            settlement_layer: this.settlement_layer,
        }
    }
}
