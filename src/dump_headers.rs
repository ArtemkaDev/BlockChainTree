use crate::errors::{DumpHeadersError, DumpHeadersErrorKind};
use error_stack::{Report, Result};

#[repr(u8)]
pub enum Headers {
    Transaction = 0,
    Token = 1,
    TransactionBlock = 2,
    TokenBlock = 3,
    SummarizeBlock = 4,
}

impl Headers {
    pub fn from_u8(header: u8) -> Result<Headers, DumpHeadersError> {
        match header {
            0 => Ok(Headers::Transaction),
            1 => Ok(Headers::Token),
            2 => Ok(Headers::TransactionBlock),
            3 => Ok(Headers::TokenBlock),
            4 => Ok(Headers::SummarizeBlock),
            _ => Err(Report::new(DumpHeadersError::DumpHeadersError(
                DumpHeadersErrorKind::UknownHeader,
            ))),
        }
    }
}
