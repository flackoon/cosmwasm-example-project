use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum ExecuteMsg {
  ValidateMigrationMsg {
      current_version: u32,
      new_version: u32,
      reason: String,
  }
}

#[cw_serde]
pub enum QueryMsg { }

#[cw_serde]
pub struct InstantiateMsg { }
