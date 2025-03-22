pub mod common;
pub mod lexpr_ext;
pub mod sch;

use {
    lexpr::Value,
    std::{
        error::Error,
        fmt::{Display, Formatter, Result as FmtResult},
    },
};

pub(crate) use lexpr_ext::*;

#[derive(Debug)]
pub enum ParseError {
    ExpectedList(Value),
    ExpectedListFloatHead(Value),
    ExpectedListIntHead(Value),
    ExpectedListStrHead(Value),
    ExpectedListSymbolHead(Value),
    ExpectedNil(Value),
    ExpectedSymbol(Value, String),
    InvalidHeight(f64),
    InvalidPaperSize(String),
    InvalidUuid(String),
    InvalidWidth(f64),
    MissingField(String, String, Value),
    Unexpected(Value),
}

impl ParseError {
    pub fn missing_field<S, F, V>(struct_name: S, field_name: F, value: V) -> Self
    where
        S: Into<String>,
        F: Into<String>,
        V: Into<Value>,
    {
        Self::MissingField(struct_name.into(), field_name.into(), value.into())
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::ExpectedList(value) => write!(f, "Expected list, got {value}"),
            Self::ExpectedListFloatHead(value) => {
                write!(f, "Expected list with floating-point head, got {value}")
            }
            Self::ExpectedListIntHead(value) => write!(f, "Expected list with integer head, got {value}"),
            Self::ExpectedListStrHead(value) => write!(f, "Expected list with str head, got {value}"),
            Self::ExpectedListSymbolHead(value) => write!(f, "Expected list with symbol head, got {value}"),
            Self::ExpectedNil(value) => write!(f, "Expected nil, got {value}"),
            Self::ExpectedSymbol(value, symbol) => write!(f, "Expected symbol {symbol}, got {value}"),
            Self::InvalidHeight(height) => write!(f, "Invalid height value {height}"),
            Self::InvalidPaperSize(paper_size) => write!(f, "Invalid paper size {paper_size}"),
            Self::InvalidUuid(value) => write!(f, "Invalid UUID {value}"),
            Self::InvalidWidth(width) => write!(f, "Invalid width value {width}"),
            Self::MissingField(struct_name, field_name, value) => write!(f, "Missing {struct_name} field {field_name}: {value}"),
            Self::Unexpected(value) => write!(f, "Unexpected value {value}"),
        }
    }
}

impl Error for ParseError {}

#[macro_export]
macro_rules! impl_try_from_cons_value {
    ($name:tt) => {
        impl ::std::convert::TryFrom<&::lexpr::Value> for $name {
            type Error = $crate::ParseError;

            fn try_from(value: &::lexpr::Value) -> ::std::result::Result<Self, Self::Error> {
                let cons = value.expect_cons()?;
                Self::try_from(cons)
            }
        }
    };
}
