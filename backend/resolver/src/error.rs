use shared::evm::broadcast::TxSendError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("Relayer call failed: {0}")]
    Relayer(String),
    #[error("ICP factory failed: {0}")]
    Factory(String),
    #[error("Send error: {0}")]
    SendError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Candid error: {0}")]
    CandidError(String),
    #[error("IC CDK error: {0}")]
    IcError(String),
    #[error("Other: {0}")]
    Other(String),
}

impl From<std::string::String> for ResolverError {
    fn from(err: std::string::String) -> Self {
        ResolverError::InternalError(err)
    }
}

impl From<ic_cdk::call::CandidDecodeFailed> for ResolverError {
    fn from(err: ic_cdk::call::CandidDecodeFailed) -> Self {
        ResolverError::CandidError(format!("failed to encode: {:?}", err))
    }
}

impl From<ic_cdk::call::CallFailed> for ResolverError {
    fn from(err: ic_cdk::call::CallFailed) -> Self {
        ResolverError::IcError(format!("call failed: {:?}", err))
    }
}

impl From<TxSendError> for ResolverError {
    fn from(err: TxSendError) -> Self {
        ResolverError::SendError(format!("{:?}", err))
    }
}

pub type Result<T, E = ResolverError> = std::result::Result<T, E>;
