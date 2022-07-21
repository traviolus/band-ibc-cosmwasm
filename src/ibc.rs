#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Binary, Decimal, DepsMut, Env, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcOrder, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::OracleResponsePacket;
use crate::obi::PriceDataOutput;
use crate::state::{PriceData, CONFIG, PRICES, REQUESTS};

pub const IBC_VERSION: &str = "bandchain-1";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<(), ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;

    let channel = msg.channel().endpoint.channel_id.clone();

    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_connect")
        .add_attribute("channel_id", channel))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let channel = msg.channel().endpoint.channel_id.clone();
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_close")
        .add_attribute("channel", channel))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    match try_ibc_packet_receive(deps, env, msg) {
        Ok(response) => Ok(response),
        Err(error) => fail_packet_receive(&error.to_string()),
    }
}

pub fn try_ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let channel = msg.packet.dest.channel_id;
    let OracleResponsePacket {
        client_id,
        request_id,
        resolve_time,
        resolve_status,
        result,
        ..
    } = from_binary(&msg.packet.data)?;
    execute_update(
        deps,
        channel,
        client_id,
        request_id,
        resolve_time,
        resolve_status,
        result,
    )
}

pub fn execute_update(
    deps: DepsMut,
    channel: String,
    client_id: String,
    request_id: String,
    resolve_time: String,
    resolve_status: String,
    result: String,
) -> Result<IbcReceiveResponse, ContractError> {
    if resolve_status != *"RESOLVE_STATUS_SUCCESS" {
        return fail_packet_receive("Band request did not resolve successfully");
    }

    let config = CONFIG.load(deps.storage)?;
    if channel != config.channel {
        return fail_packet_receive("Received packet coming from the wrong channel");
    }

    let PriceDataOutput { rates } = PriceDataOutput::decode_obi(result.as_str())?;
    let request = match REQUESTS.may_load(deps.storage, &client_id) {
        Ok(Some(data)) => data,
        Ok(None) => return fail_packet_receive("Invalid client id"),
        Err(e) => return fail_packet_receive(&e.to_string()),
    };

    if request.symbols.len() != rates.len() {
        return fail_packet_receive("Result and Calldata length mismatched");
    }
    request
        .symbols
        .iter()
        .zip(rates.iter())
        .for_each(|(symbol, &rate)| {
            PRICES
                .save(
                    deps.storage,
                    symbol,
                    &PriceData {
                        rate: Decimal::from_ratio(rate, request.multiplier),
                        bandchain_request_id: u64::from_str(request_id.as_str()).unwrap(),
                        bandchain_resolve_time: u64::from_str(resolve_time.as_str()).unwrap(),
                    },
                )
                .unwrap()
        });

    Ok(IbcReceiveResponse::new()
        .add_attributes(vec![
            attr("method", "execute_update"),
            attr("request_id", client_id),
        ])
        .set_ack(make_ack_success()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_ack"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_timeout"))
}

pub fn validate_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    if channel.order != IbcOrder::Unordered {
        return Err(ContractError::OrderedChannel {});
    }

    if channel.version != IBC_VERSION {
        return Err(ContractError::InvalidVersion {
            actual: channel.version.to_string(),
            expected: IBC_VERSION.to_string(),
        });
    }

    if let Some(counterparty_version) = counterparty_version {
        if counterparty_version != IBC_VERSION {
            return Err(ContractError::InvalidVersion {
                actual: counterparty_version.to_string(),
                expected: IBC_VERSION.to_string(),
            });
        }
    }

    Ok(())
}

pub fn fail_packet_receive(err: &str) -> Result<IbcReceiveResponse, ContractError> {
    Ok(IbcReceiveResponse::new()
        .add_attribute("method", "ibc_packet_receive")
        .add_attribute("error", err.to_string())
        .set_ack(make_ack_fail(err.to_string())))
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Ack {
    Result(Binary),
    Error(String),
}

pub fn make_ack_success() -> Binary {
    let res = Ack::Result(b"1".into());
    to_binary(&res).unwrap()
}

pub fn make_ack_fail(err: String) -> Binary {
    let res = Ack::Error(err);
    to_binary(&res).unwrap()
}
