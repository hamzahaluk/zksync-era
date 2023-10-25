use std::convert::TryFrom;
use std::time::Instant;
use zksync_contracts::{governance_contract, zksync_contract};
use zksync_dal::StorageProcessor;
use zksync_types::{
    ethabi::Contract, protocol_version::GovernanceOperation, web3::types::Log, Address,
    ProtocolUpgrade, ProtocolVersionId, H256,
};

use crate::eth_watch::{
    client::{Error, EthClient},
    event_processors::EventProcessor,
    metrics::{PollStage, METRICS},
};

/// Responsible for saving new protocol upgrade proposals to the database.
#[derive(Debug)]
pub struct UpgradesEventProcessor {
    diamond_proxy_address: Address,
    last_seen_version_id: ProtocolVersionId,
    upgrade_proposal_signature: H256,
    execute_upgrade_short_signature: [u8; 4],
}

impl UpgradesEventProcessor {
    pub fn new(diamond_proxy_address: Address, last_seen_version_id: ProtocolVersionId) -> Self {
        Self {
            diamond_proxy_address,
            last_seen_version_id,
            upgrade_proposal_signature: old_zksync_contract()
                .event("ProposeTransparentUpgrade")
                .expect("ProposeTransparentUpgrade event is missing in abi")
                .signature(),
            execute_upgrade_short_signature: zksync_contract()
                .function("executeUpgrade")
                .unwrap()
                .short_signature(),
        }
    }
}

#[async_trait::async_trait]
impl<W: EthClient + Sync> EventProcessor<W> for UpgradesEventProcessor {
    async fn process_events(
        &mut self,
        storage: &mut StorageProcessor<'_>,
        client: &W,
        events: Vec<Log>,
    ) -> Result<(), Error> {
        let mut upgrades = Vec::new();
        for event in events
            .into_iter()
            .filter(|event| event.topics[0] == self.upgrade_proposal_signature)
        {
            let upgrade = ProtocolUpgrade::try_from(event)
                .map_err(|err| Error::LogParse(format!("{:?}", err)))?;
            // Scheduler VK is not present in proposal event. It is hardcoded in verifier contract.
            let scheduler_vk_hash = if let Some(address) = upgrade.verifier_address {
                Some(client.scheduler_vk_hash(address).await?)
            } else {
                None
            };
            upgrades.push((upgrade, scheduler_vk_hash));
        }

        if upgrades.is_empty() {
            return Ok(());
        }

        let ids_str: Vec<_> = upgrades
            .iter()
            .map(|(u, _)| format!("{}", u.id as u16))
            .collect();
        tracing::debug!("Received upgrades with ids: {}", ids_str.join(", "));

        let new_upgrades: Vec<_> = upgrades
            .into_iter()
            .skip_while(|(v, _)| v.id as u16 <= self.last_seen_version_id as u16)
            .collect();
        if new_upgrades.is_empty() {
            return Ok(());
        }

        let last_id = new_upgrades.last().unwrap().0.id;
        let stage_latency = METRICS.poll_eth_node[&PollStage::PersistUpgrades].start();
        for (upgrade, scheduler_vk_hash) in new_upgrades {
            let previous_version = storage
                .protocol_versions_dal()
                .load_previous_version(upgrade.id)
                .await
                .expect("Expected previous version to be present in DB");
            let new_version = previous_version.apply_upgrade(upgrade, scheduler_vk_hash);
            storage
                .protocol_versions_dal()
                .save_protocol_version_with_tx(new_version)
                .await;
        }
        stage_latency.observe();
        self.last_seen_version_id = last_id;
        Ok(())
    }

    fn relevant_topic(&self) -> H256 {
        self.upgrade_proposal_signature
    }
}

pub fn old_zksync_contract() -> Contract {
    let json = r#"[
        {
          "anonymous": false,
          "inputs": [
            {
              "components": [
                {
                  "components": [
                    {
                      "internalType": "address",
                      "name": "facet",
                      "type": "address"
                    },
                    {
                      "internalType": "enum Diamond.Action",
                      "name": "action",
                      "type": "uint8"
                    },
                    {
                      "internalType": "bool",
                      "name": "isFreezable",
                      "type": "bool"
                    },
                    {
                      "internalType": "bytes4[]",
                      "name": "selectors",
                      "type": "bytes4[]"
                    }
                  ],
                  "internalType": "struct Diamond.FacetCut[]",
                  "name": "facetCuts",
                  "type": "tuple[]"
                },
                {
                  "internalType": "address",
                  "name": "initAddress",
                  "type": "address"
                },
                {
                  "internalType": "bytes",
                  "name": "initCalldata",
                  "type": "bytes"
                }
              ],
              "indexed": false,
              "internalType": "struct Diamond.DiamondCutData",
              "name": "diamondCut",
              "type": "tuple"
            },
            {
              "indexed": true,
              "internalType": "uint256",
              "name": "proposalId",
              "type": "uint256"
            },
            {
              "indexed": false,
              "internalType": "bytes32",
              "name": "proposalSalt",
              "type": "bytes32"
            }
          ],
          "name": "ProposeTransparentUpgrade",
          "type": "event"
        },
        {
          "inputs": [
            {
              "components": [
                {
                  "components": [
                    {
                      "internalType": "address",
                      "name": "facet",
                      "type": "address"
                    },
                    {
                      "internalType": "enum Diamond.Action",
                      "name": "action",
                      "type": "uint8"
                    },
                    {
                      "internalType": "bool",
                      "name": "isFreezable",
                      "type": "bool"
                    },
                    {
                      "internalType": "bytes4[]",
                      "name": "selectors",
                      "type": "bytes4[]"
                    }
                  ],
                  "internalType": "struct Diamond.FacetCut[]",
                  "name": "facetCuts",
                  "type": "tuple[]"
                },
                {
                  "internalType": "address",
                  "name": "initAddress",
                  "type": "address"
                },
                {
                  "internalType": "bytes",
                  "name": "initCalldata",
                  "type": "bytes"
                }
              ],
              "internalType": "struct Diamond.DiamondCutData",
              "name": "_diamondCut",
              "type": "tuple"
            },
            {
              "internalType": "bytes32",
              "name": "_proposalSalt",
              "type": "bytes32"
            }
          ],
          "name": "executeUpgrade",
          "outputs": [],
          "stateMutability": "nonpayable",
          "type": "function"
        },
        {
          "anonymous": false,
          "inputs": [
            {
              "indexed": false,
              "internalType": "uint256",
              "name": "txId",
              "type": "uint256"
            },
            {
              "indexed": false,
              "internalType": "bytes32",
              "name": "txHash",
              "type": "bytes32"
            },
            {
              "indexed": false,
              "internalType": "uint64",
              "name": "expirationTimestamp",
              "type": "uint64"
            },
            {
              "components": [
                {
                  "internalType": "uint256",
                  "name": "txType",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "from",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "to",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "gasLimit",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "gasPerPubdataByteLimit",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "maxFeePerGas",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "maxPriorityFeePerGas",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "paymaster",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "nonce",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256",
                  "name": "value",
                  "type": "uint256"
                },
                {
                  "internalType": "uint256[4]",
                  "name": "reserved",
                  "type": "uint256[4]"
                },
                {
                  "internalType": "bytes",
                  "name": "data",
                  "type": "bytes"
                },
                {
                  "internalType": "bytes",
                  "name": "signature",
                  "type": "bytes"
                },
                {
                  "internalType": "uint256[]",
                  "name": "factoryDeps",
                  "type": "uint256[]"
                },
                {
                  "internalType": "bytes",
                  "name": "paymasterInput",
                  "type": "bytes"
                },
                {
                  "internalType": "bytes",
                  "name": "reservedDynamic",
                  "type": "bytes"
                }
              ],
              "indexed": false,
              "internalType": "struct IMailbox.L2CanonicalTransaction",
              "name": "transaction",
              "type": "tuple"
            },
            {
              "indexed": false,
              "internalType": "bytes[]",
              "name": "factoryDeps",
              "type": "bytes[]"
            }
          ],
          "name": "NewPriorityRequest",
          "type": "event"
        }
    ]"#;
    serde_json::from_str(json).unwrap()
}
