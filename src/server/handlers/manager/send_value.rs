// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! SendValue subnet handler and parameters

use crate::manager::evm::ethers_address_to_fil_address;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::handlers::manager::{check_subnet, parse_from};
use crate::server::{handlers, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendValueParams {
    pub subnet: String,
    pub from: Option<String>,
    pub to: String,
    /// In FIL, not atto
    pub amount: f64,
}

/// Send value between two addresses within a subnet
pub(crate) struct SendValueHandler {
    pool: Arc<SubnetManagerPool>,
}

impl SendValueHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for SendValueHandler {
    type Request = SendValueParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let amount = handlers::f64_to_token_amount(request.amount)?;
        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let from = parse_from(subnet_config, request.from)?;
        let to = match Address::from_str(&request.to) {
            Ok(addr) => addr,
            Err(_) => {
                // we need to check if an 0x address was passed and convert
                // to a delegated address
                ethers_address_to_fil_address(&ethers::types::Address::from_str(&request.to)?)?
            }
        };

        log::debug!("json rpc: received request to send amount: {amount:} from {from:} to {to:}");

        conn.manager().send_value(from, to, amount).await?;

        Ok(())
    }
}
