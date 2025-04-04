use cosmwasm_std::{Addr, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
  #[error("{0}")]
  StdError(#[from] StdError),

  #[error("{sender} is not contract admin")]
  Unauthorized { sender: Addr },

  #[error("Payment error: {0}")]
  Payment(#[from] PaymentError),

  #[error("{admin} is already an admin")]
  AdminExists { admin: Addr },

  #[error("Contract already at version {version}")]
  AlreadyMigrated { version: u32 },

  #[error("Invalid migration reason")]
  InvalidMigrationReason { },

  #[error("Unknown execute action")]
  UnknownExecuteAction
}
