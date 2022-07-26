use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// This structure stores the basic settings for creating a new contract instance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Contract owner address
    pub owner: String,
}

/// ## Description
/// This structure describes the contract's possible execute messages.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Set the IBC channel to be used for the oracle requests.
    SetChannel {
        /// The channel name to use for the oracle requests
        channel: String,
    },
    /// Register a new oracle request job.
    RegisterJob {
        /// ID of the oracle script on BandChain to query the data from
        oracle_script_id: u64,
        /// The list of symbols to query the price for
        symbols: Vec<String>,
        /// The multiplier to use to multiply the oracle price by.
        multiplier: u64,
        /// The number of BandChain validators that are requested to respond to this  oracle request.
        ask_count: u64,
        /// The minimum number of validators necessary for the request to proceed to the execution phase.
        min_count: u64,
    },
    /// Request and update oracle data for the specified request job ID.
    UpdateJobData {
        /// The ID of the oracle request job to update.
        job_id: String,
    },
}

/// ## Description
/// This structure defines the request bundle to be send to BandChain
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleRequestPacket {
    /// The unique identifier of the oracle request. This same unique ID will be sent back to the requester with the oracle response.
    pub client_id: String,
    /// ID of the oracle script on BandChain that is queried from.
    pub oracle_script_id: u64,
    /// The data passed over to the oracle script to execute.
    pub calldata: Vec<u8>,
    /// The number of BandChain validators that are requested to respond to this  oracle request.
    pub ask_count: u64,
    /// The minimum number of validators necessary for the request to proceed to the execution phase.
    pub min_count: u64,
    /// The maximum tokens that will be paid to all data source providers.
    pub fee_limit: Vec<Coin>,
    /// The maxmimum gas to be used during the oracle requests' prepare phase.
    pub prepare_gas: u64,
    /// The maxmimum gas to be used during the oracle requests' execution phase.
    pub execute_gas: u64,
}

/// ## Description
/// This structure defines the packet received back from BandChain for an oracle request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleResponsePacket {
    /// The unique identifier of the oracle request. This same unique ID will be sent back to the requester with the oracle response.
    pub client_id: String,
    // The unique identifier for the oracle request associated with the response.
    pub request_id: String,
    // The number of validators among to the asked validators that actually responded to this oracle request prior to this oracle request being resolved.
    pub ans_count: String,
    // The UNIX epoch time at which the request was sent to BandChain.
    pub request_time: String,
    // The UNIX epoch time at which the request was resolved to the final result.
    pub resolve_time: String,
    // The status of this oracle request. One of OK, FAILURE, or EXPIRED.
    pub resolve_status: String,
    // The final aggregated value encoded in OBI format. Only available if status if OK.
    pub result: String,
}

/// ## Description
/// This structure describes the available query messages for the airdrop contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Config returns contract settings specified in the custom [`ConfigResponse`] structure.
    Config {},
    // Job returns information about the specified job using a custom [`Job`] structure.
    Job { job_id: String },
    // Price returns the latest price for the specified asset symbol using a custom [`PriceData`] structure.
    Price { symbol: String },
}

/// ## Description
/// A struct used for migrating contracts.
/// Currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
