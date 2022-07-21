use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetChannel {
        channel: String,
    },
    RegisterNewRequest {
        oracle_script_id: u64,
        symbols: Vec<String>,
        multiplier: u64,
        ask_count: u64,
        min_count: u64,
    },
    UpdateData {
        request_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleRequestPacket {
    pub client_id: String,
    pub oracle_script_id: u64,
    pub calldata: Vec<u8>,
    pub ask_count: u64,
    pub min_count: u64,
    pub fee_limit: Vec<Coin>,
    pub prepare_gas: u64,
    pub execute_gas: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleResponsePacket {
    pub client_id: String,
    pub request_id: String,
    pub ans_count: String,
    pub request_time: String,
    pub resolve_time: String,
    pub resolve_status: String,
    pub result: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Request { request_id: String },
    Price { symbol: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
