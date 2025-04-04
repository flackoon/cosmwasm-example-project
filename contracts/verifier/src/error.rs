use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
  #[error("Contract already at version {version}")]
  AlreadyMigrated { version: u32 },

  #[error("Invalid migration reason")]
  InvalidMigrationReason { },
}
