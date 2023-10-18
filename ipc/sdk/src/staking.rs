// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Staking module related types and functions

use crate::ethers_address_to_fil_address;
use ethers::contract::EthEvent;
use fvm_shared::address::Address;

pub type ConfigurationNumber = u64;

#[derive(Clone, Debug)]
pub enum StakingOperation {
    Deposit,
    Withdraw,
    SetMetadata,
}

impl From<u8> for StakingOperation {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Deposit,
            1 => Self::Withdraw,
            _ => Self::SetMetadata,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StakingChangeRequest {
    pub configuration_number: ConfigurationNumber,
    pub change: StakingChange,
}

/// The change request to validator staking
#[derive(Clone, Debug)]
pub struct StakingChange {
    pub op: StakingOperation,
    pub payload: Vec<u8>,
    pub validator: Address,
}

/// The event emitted when a staking request is perform in solidity contracts
#[derive(Clone, Debug, EthEvent)]
pub struct NewStakingRequest {
    op: u8,
    validator: ethers::types::Address,
    payload: ethers::types::Bytes,
    configuration_number: u64,
}

impl TryFrom<NewStakingRequest> for StakingChangeRequest {
    type Error = anyhow::Error;

    fn try_from(value: NewStakingRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            configuration_number: value.configuration_number,
            change: StakingChange {
                op: StakingOperation::from(value.op),
                payload: value.payload.to_vec(),
                validator: ethers_address_to_fil_address(&value.validator)?,
            },
        })
    }
}