use thiserror::Error;
/// Inspired by https://nick.groenen.me/posts/rust-error-handling/

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("unknown data store error")]
    Unknown,
}