#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, coins, to_binary, Binary, Deps, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, OracleRequestPacket, QueryMsg};
use crate::obi::PriceDataInput;
use crate::state::{Config, PriceData, Request, CONFIG, PRICES, REQUESTS, REQUESTS_COUNT};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:band-ibc";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REQUEST_ID_PREFIX: &str = "tvl";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender,
        channel: String::new(),
    };
    CONFIG.save(deps.storage, &config)?;
    REQUESTS_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetChannel { channel } => try_set_channel(deps, info, channel),
        ExecuteMsg::RegisterNewRequest {
            oracle_script_id,
            symbols,
            multiplier,
            ask_count,
            min_count,
        } => try_register_new_request(
            deps,
            info,
            oracle_script_id,
            symbols,
            multiplier,
            ask_count,
            min_count,
        ),
        ExecuteMsg::UpdateData { request_id } => try_update_data(deps, env, request_id),
    }
}

pub fn try_set_channel(
    deps: DepsMut,
    info: MessageInfo,
    channel: String,
) -> Result<Response, ContractError> {
    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            if config.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            config.channel = channel.clone();

            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "set_channel")
        .add_attribute("channel", channel))
}

pub fn try_register_new_request(
    deps: DepsMut,
    info: MessageInfo,
    oracle_script_id: u64,
    symbols: Vec<String>,
    multiplier: u64,
    ask_count: u64,
    min_count: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let new_requests_count = REQUESTS_COUNT.load(deps.storage)? + 1;
    let request_id = format!("{}-{}", REQUEST_ID_PREFIX, new_requests_count);
    REQUESTS_COUNT.save(deps.storage, &new_requests_count)?;

    let calldata = PriceDataInput {
        symbol: symbols.clone(),
        multiplier,
    }
    .encode_obi()?;
    let request = Request {
        oracle_script_id,
        symbols,
        multiplier,
        calldata,
        ask_count,
        min_count,
    };
    REQUESTS.save(deps.storage, request_id.as_str(), &request)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "register_new_request"),
        attr("request_id", request_id),
    ]))
}

pub fn try_update_data(
    deps: DepsMut,
    env: Env,
    request_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.channel == String::new() {
        return Err(ContractError::ChannelNotSet {});
    }

    let request = match REQUESTS.may_load(deps.storage, &request_id) {
        Ok(Some(data)) => data,
        Ok(None) => return Err(ContractError::RequestNotFound {}),
        Err(e) => return Err(ContractError::Std(e)),
    };

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "update_data"),
            attr("channel", config.channel.clone()),
            attr("request_id", request_id.clone()),
        ])
        .add_message(IbcMsg::SendPacket {
            channel_id: config.channel,
            data: to_binary(&OracleRequestPacket {
                client_id: request_id,
                oracle_script_id: request.oracle_script_id,
                calldata: request.calldata,
                ask_count: request.ask_count,
                min_count: request.min_count,
                fee_limit: coins(1000000u128, "uband"),
                prepare_gas: 100000u64,
                execute_gas: 4000000u64,
            })?,
            timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300u64)),
        }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Request { request_id } => to_binary(&query_request(deps, request_id)?),
        QueryMsg::Price { symbol } => to_binary(&query_price(deps, symbol)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_request(deps: Deps, request_id: String) -> StdResult<Request> {
    REQUESTS.load(deps.storage, &request_id)
}

fn query_price(deps: Deps, symbol: String) -> StdResult<PriceData> {
    PRICES.load(deps.storage, &symbol)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}
