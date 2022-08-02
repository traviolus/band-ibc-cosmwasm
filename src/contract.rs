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
use crate::state::{Config, ConfigResponse, Job, PriceData, CONFIG, JOBS, JOB_COUNT, PRICES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:band-ibc";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const JOB_ID_PREFIX: &str = "tvl";

/// ## Description
/// Creates a new contract with the specified parameters packed in the `msg` variable.
/// Returns a [`Response`] with the specified attributes if the operation was successful,
/// or a [`ContractError`] if the contract was not created.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **env** is an object of type [`Env`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **msg** is a message of type [`InstantiateMsg`] which contains the parameters used for creating the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        channel: String::new(),
    };

    CONFIG.save(deps.storage, &config)?;
    JOB_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

/// ## Description
/// Exposes all the execute functions available in the contract.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **env** is an object of type [`Env`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **msg** is an object of type [`ExecuteMsg`].
/// ## Commands
/// - **ExecuteMsg::SetChannel {
///             channel
///         }** Set the IBC channel to be used for the oracle requests.
///
/// - **ExecuteMsg::RegisterNewRequest {
///             oracle_script_id,
///             symbols,
///             multiplier,
///             ask_count,
///             min_count
///             }** Register a new oracle request job.
///
/// - **ExecuteMsg::UpdateJobData {
///             job_id,
///             }** Request and update oracle data for the specified request job ID.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetChannel { channel } => try_set_channel(deps, info, channel),
        ExecuteMsg::RegisterJob {
            oracle_script_id,
            symbols,
            multiplier,
            ask_count,
            min_count,
        } => try_register_job(
            deps,
            info,
            oracle_script_id,
            symbols,
            multiplier,
            ask_count,
            min_count,
        ),
        ExecuteMsg::UpdateJobData { job_id } => try_update_job_data(deps, env, job_id),
    }
}

/// ## Description
/// Set the IBC channel to be used for the oracle requests into the contract's config
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **channel** is an object of type [`String`] which is the channel name to use for the oracle requests.
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

/// ## Description
/// Register a new oracle request job.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **oracle_script_id** is an object of type [`u64`] which is the ID of the oracle script on BandChain to query the data from.
///
/// - **symbols** is an object of type [`Vec<String>`] which is the list of symbols to query the price for.
///
/// - **multiplier** is an object of type [`u64`] the multiplier to use to multiply the oracle price by.
///
/// - **ask_count** is an object of type [`u64`] which is the number of BandChain validators that are requested to respond to this oracle request.
///
/// - **min_count** is an object of type [`u64`] which is the minimum number of validators necessary for the request to proceed to the execution phase.
pub fn try_register_job(
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

    let new_job_count = JOB_COUNT.load(deps.storage)? + 1;
    let job_id = format!("{}-{}", JOB_ID_PREFIX, new_job_count);
    JOB_COUNT.save(deps.storage, &new_job_count)?;

    let calldata = PriceDataInput {
        symbol: symbols.clone(),
        multiplier,
    }
    .encode_obi()?;

    let job = Job {
        oracle_script_id,
        symbols,
        multiplier,
        calldata,
        ask_count,
        min_count,
    };
    JOBS.save(deps.storage, job_id.as_str(), &job)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "register_new_job"),
        attr("job_id", job_id),
    ]))
}

/// ## Description
/// Sends out a new IBC oracle request for the specified job
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **job_id** is an object of type [`String`] which is the ID of the oracle request job to update.
pub fn try_update_job_data(
    deps: DepsMut,
    env: Env,
    job_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.channel == String::new() {
        return Err(ContractError::ChannelNotSet {});
    }

    let job = match JOBS.may_load(deps.storage, &job_id) {
        Ok(Some(data)) => data,
        Ok(None) => return Err(ContractError::JobNotFound {}),
        Err(e) => return Err(ContractError::Std(e)),
    };

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "update_data"),
            attr("channel", config.channel.clone()),
            attr("job_id", job_id.clone()),
        ])
        .add_message(IbcMsg::SendPacket {
            channel_id: config.channel,
            data: to_binary(&OracleRequestPacket {
                client_id: job_id,
                oracle_script_id: job.oracle_script_id,
                calldata: job.calldata,
                ask_count: job.ask_count,
                min_count: job.min_count,
                fee_limit: coins(1000000u128, "uband"),
                prepare_gas: 100000u64,
                execute_gas: 4000000u64,
            })?,
            timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300u64)),
        }))
}

/// ## Description
/// Exposes all the queries available in the contract.
///
/// ## Params
/// - **deps** is an object of type [`Deps`].
///
/// - **_env** is an object of type [`Env`].
///
/// - **msg** is an object of type [`QueryMsg`].
///
/// ## Commands
/// - **QueryMsg::Config {}** Returns general contract parameters using a custom [`ConfigResponse`] structure.
///
/// - **QueryMsg::Job { job_id }** Returns information about the specified job using a custom [`Job`] structure.
///
/// - **QueryMsg::Price { symbol }** Returns the latest price for the specified asset symbol using a custom [`PriceData`] structure.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Job { job_id } => to_binary(&query_job(deps, job_id)?),
        QueryMsg::Price { symbol } => to_binary(&query_price(deps, symbol)?),
    }
}

/// ## Description
/// Returns general contract parameters using a custom [`ConfigResponse`] structure.
///
/// ## Params
/// - **deps** is an object of type [`Deps`].
fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        channel: config.channel,
    })
}

/// ## Description
/// Returns information about the specified job using a custom [`Job`] structure.
///
/// ## Params
/// - **deps** is an object of type [`Deps`].
/// - **job_id** is the ID of the registered job to query the information for.
fn query_job(deps: Deps, job_id: String) -> StdResult<Job> {
    JOBS.load(deps.storage, &job_id)
}

/// ## Description
/// Returns the latest price for the specified asset symbol using a custom [`PriceData`] structure.
///
/// ## Params
/// - **deps** is an object of type [`Deps`].
/// - **symbol** is symbol of the asset to query the latest price data for,
fn query_price(deps: Deps, symbol: String) -> StdResult<PriceData> {
    PRICES.load(deps.storage, &symbol)
}

/// ## Description
/// Exposes the migrate functionality in the contract.
///
/// ## Params
/// - **_deps** is an object of type [`DepsMut`].
///
/// - **_env** is an object of type [`Env`].
///
/// - **_msg** is an object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}
