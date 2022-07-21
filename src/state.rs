use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub channel: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Request {
    pub oracle_script_id: u64,
    pub symbols: Vec<String>,
    pub multiplier: u64,
    pub calldata: Vec<u8>,
    pub ask_count: u64,
    pub min_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceData {
    pub rate: Decimal,
    pub bandchain_request_id: u64,
    pub bandchain_resolve_time: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const REQUESTS_COUNT: Item<u64> = Item::new("requests_count");
pub const REQUESTS: Map<&str, Request> = Map::new("request"); // client_id -> Request {}
pub const PRICES: Map<&str, PriceData> = Map::new("prices");
