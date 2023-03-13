use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub question: String,
    pub yes_votes: u64,
    pub no_votes: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const POLLS: Map<String, Poll> = Map::new("polls");
