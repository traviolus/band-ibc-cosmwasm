use cosmwasm_std::StdError;
use std::io::Error as IOError;
use thiserror::Error;

/// ## Description
/// This enum describes possible contract errors.
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    IOError(#[from] IOError),

    #[error("Unsupported message")]
    Unsupported {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Local Channel is not set")]
    ChannelNotSet {},

    #[error("Provided job id is not registered")]
    JobNotFound {},

    #[error("Only unordered channels are supported.")]
    OrderedChannel {},

    #[error("Invalid IBC channel version. Got ({actual}), expected ({expected}).")]
    InvalidVersion { actual: String, expected: String },
}
