use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");
pub const JOB_COUNT: Item<u64> = Item::new("job_count");
pub const JOBS: Map<&str, Job> = Map::new("job"); // job_id -> Job {}
pub const PRICES: Map<&str, PriceData> = Map::new("prices");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Contract owner address
    pub owner: Addr,
    /// The channel name to use for the oracle requests
    pub channel: String,
}

/// ## Description
/// This structure is used to return the contract's [`Config`] details to the caller.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// Contract owner address
    pub owner: String,
    /// The channel name to use for the oracle requests
    pub channel: String,
}

/// ## Description
/// This structure holds the information related to an oracle request job.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Job {
    /// The ID of the oracle script on BandChain to request the price data from
    pub oracle_script_id: u64,
    /// The list of asset symbols to request the price data for.
    pub symbols: Vec<String>,
    /// The multiplier value used to multiply the price data value by (to preserve precision)
    pub multiplier: u64,
    /// The OBI-encoded calldata bytes available for oracle executor to read.
    pub calldata: Vec<u8>,
    /// The number of validators that are requested to respond to this oracle request. Higher value means more security, at a higher gas cost.
    pub ask_count: u64,
    /// The minimum number of validators necessary for the request to proceed to the execution phase. Higher value means more security, at the cost of liveness.
    pub min_count: u64,
}

/// ## Description
// A custom struct for each query response that returns the latest oracle price of an asset.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceData {
    /// The latest price data for the requested asset
    pub rate: Decimal,
    /// The BandChain request ID associated with this price data.
    pub bandchain_request_id: u64,
    /// The time the request for this price data was resolved on BandChain.
    pub bandchain_resolve_time: u64,
}
