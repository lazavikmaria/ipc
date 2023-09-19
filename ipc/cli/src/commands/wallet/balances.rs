// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet balances cli handler

use async_trait::async_trait;
use clap::Args;
use futures_util::future::join_all;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_identity::{EthKeyAddress, EvmKeyStore, WalletType};
use ipc_provider::manager::evm::ethers_address_to_fil_address;
use ipc_sdk::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

pub(crate) struct WalletBalances;

#[async_trait]
impl CommandLineHandler for WalletBalances {
    type Arguments = WalletBalancesArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list wallets with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;

        let wallet_type = WalletType::from_str(&arguments.wallet_type)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        match wallet_type {
            WalletType::Evm => {
                let wallet = provider.evm_wallet();
                let addresses = wallet.read().unwrap().list()?;
                let r = addresses
                    .iter()
                    .map(|addr| {
                        let provider = provider.clone();
                        let subnet = subnet.clone();
                        async move {
                            provider
                                .wallet_balance(
                                    &subnet,
                                    &ethers_address_to_fil_address(&(addr.clone()).into())?,
                                )
                                .await
                                .map(|balance| (balance, addr))
                        }
                    })
                    .collect::<Vec<_>>();

                let r = join_all(r)
                    .await
                    .into_iter()
                    .collect::<anyhow::Result<Vec<(TokenAmount, &EthKeyAddress)>>>()?;

                for (balance, addr) in r {
                    println!("{:?} - Balance: {}", addr.to_string(), balance);
                }
            }
            WalletType::Fvm => {
                let wallet = provider.fvm_wallet();
                let addresses = wallet.read().unwrap().list_addrs()?;
                let r = addresses
                    .iter()
                    .map(|addr| {
                        let provider = provider.clone();
                        let subnet = subnet.clone();
                        async move {
                            provider
                                .wallet_balance(&subnet, addr)
                                .await
                                .map(|balance| (balance, addr))
                        }
                    })
                    .collect::<Vec<_>>();

                let r = join_all(r)
                    .await
                    .into_iter()
                    .collect::<anyhow::Result<Vec<(TokenAmount, &Address)>>>()?;
                for (balance, addr) in r {
                    println!("{:?} - Balance: {}", addr, balance);
                }
            }
        };

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List balance of wallets in a subnet")]
pub(crate) struct WalletBalancesArgs {
    #[arg(long, short, help = "The subnet to list wallets from")]
    pub subnet: String,
    #[arg(long, short, help = "The type of the wallet, i.e. fvm, evm")]
    pub wallet_type: String,
}