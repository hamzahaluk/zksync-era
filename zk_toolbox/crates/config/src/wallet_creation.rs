use std::path::{Path, PathBuf};

use common::wallets::Wallet;
use rand::thread_rng;
use types::WalletCreation;
use xshell::Shell;

use crate::{
    consts::{BASE_PATH, TEST_CONFIG_PATH},
    traits::{ReadConfig, SaveConfigWithBasePath},
    EthMnemonicConfig, WalletsConfig,
};

pub fn create_wallets(
    shell: &Shell,
    base_path: &Path,
    link_to_code: &Path,
    id: u32,
    wallet_creation: WalletCreation,
    initial_wallet_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    let wallets = match wallet_creation {
        WalletCreation::Random => {
            let rng = &mut thread_rng();
            WalletsConfig::random(rng)
        }
        WalletCreation::Empty => WalletsConfig::empty(),
        // Use id of chain for creating
        WalletCreation::Localhost => create_localhost_wallets(shell, link_to_code, id)?,
        WalletCreation::InFile => {
            let path = initial_wallet_path.ok_or(anyhow::anyhow!(
                "Wallet path for in file option is required"
            ))?;
            WalletsConfig::read(shell, path)?
        }
    };

    wallets.save_with_base_path(shell, base_path)?;
    Ok(())
}

// Create wallets based on id
pub fn create_localhost_wallets(
    shell: &Shell,
    link_to_code: &Path,
    id: u32,
) -> anyhow::Result<WalletsConfig> {
    let path = link_to_code.join(TEST_CONFIG_PATH);
    let eth_mnemonic = EthMnemonicConfig::read(shell, path)?;
    let base_path = format!("{}/{}", BASE_PATH, id);
    Ok(WalletsConfig {
        deployer: Some(Wallet::from_mnemonic(
            &eth_mnemonic.test_mnemonic,
            &base_path,
            0,
        )?),
        operator: Wallet::from_mnemonic(&eth_mnemonic.test_mnemonic, &base_path, 1)?,
        blob_operator: Wallet::from_mnemonic(&eth_mnemonic.test_mnemonic, &base_path, 2)?,
        fee_account: Wallet::from_mnemonic(&eth_mnemonic.test_mnemonic, &base_path, 3)?,
        governor: Wallet::from_mnemonic(&eth_mnemonic.test_mnemonic, &base_path, 4)?,
        token_multiplier_setter: Some(Wallet::from_mnemonic(
            &eth_mnemonic.test_mnemonic,
            &base_path,
            5,
        )?),
        // governance: Some(Wallet::from_mnemonic(
        //     &eth_mnemonic.test_mnemonic,
        //     &base_path,
        //     6,
        // )?),
        // chain_admin: Some(Wallet::from_mnemonic(
        //     &eth_mnemonic.test_mnemonic,
        //     &base_path,
        //     7,
        // )?),
        // deployer_gateway: Some(Wallet::from_mnemonic(
        //     &eth_mnemonic.test_mnemonic,
        //     &base_path,
        //     8,
        // )?),
        // governor_gateway: Some(Wallet::from_mnemonic(
        //     &eth_mnemonic.test_mnemonic,
        //     &base_path,
        //     9,
        // )?),
    })
}
