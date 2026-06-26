use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("protobuf error: {0}")]
    Protobuf(#[from] protobuf::Error),
    #[error("PLMN value {0} is out of the 24-bit range")]
    PlmnOutOfRange(i32),
    #[error("invalid PLMN string `{0}` (expected MCC-MNC, e.g. 250-01)")]
    PlmnFormat(String),
    #[error("entry #{index} is missing required field `{field}`")]
    MissingField { index: usize, field: &'static str },
    #[error("duplicate carrier id {0} in TOML")]
    DuplicateId(i32),
    #[error("duplicate mapping name `{0}` in TOML")]
    DuplicateName(String),
    #[error("no mapping named `{0}`")]
    MappingNotFound(String),
}
