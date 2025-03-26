use {
    lexpr::Value,
    std::{
        error::Error,
        fmt::{Display, Formatter, Result as FmtResult},
    },
};

#[derive(Debug)]
pub enum ParseError {
    DuplicateField(String, String, Value),
    ExpectedEnumSymbol(Value, &'static [&'static str]),
    ExpectedList(Value),
    ExpectedFloat(Value),
    ExpectedInt(Value),
    ExpectedStr(Value),
    ExpectedSym(Value),
    ExpectedNil(Value),
    ExpectedNamedSym(Value, String),
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
            Self::DuplicateField(struct_name, field_name, value) => write!(f, "Duplicate {struct_name} field {field_name}: {value}"),
            Self::ExpectedEnumSymbol(value, symbols) => write!(f, "Expected one of {}, got {}", symbols.join(", "), value),
            Self::ExpectedList(value) => write!(f, "Expected list: {value}"),
            Self::ExpectedFloat(value) =>
                write!(f, "Expected float: {value}"),
            Self::ExpectedInt(value) => write!(f, "Expected int: {value}"),
            Self::ExpectedStr(value) => write!(f, "Expected str: {value}"),
            Self::ExpectedSym(value) => write!(f, "Expected symbol: {value}"),
            Self::ExpectedNil(value) => write!(f, "Expected nil: {value}"),
            Self::ExpectedNamedSym(value, symbol) => write!(f, "Expected symbol {symbol}: {value}"),
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
