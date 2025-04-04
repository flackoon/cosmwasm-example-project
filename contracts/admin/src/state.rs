use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Map, Item};

pub const ADMINS: Map<&Addr, Empty> = Map::new("admins");
pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");
pub const VERSION: Item<u32> = Item::new("version");
pub const VERIFIER: Item<Addr> = Item::new("verifier");
