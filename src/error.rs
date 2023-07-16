use std::error::Error;
use std::fmt::{self, Display};

pub type ATResult<T> = Result<T, ErrAutoType>;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrType {
    SequenceNotExist(String),
    WrongSequenceArg(String),
    InvalidKeyFormat(String),
    KeyIsInSequences(String),
    UnknownSequence(String),
    KeyIsInCombinations(String),
}

#[derive(Debug)]
pub struct ErrAutoType {
    err_type: ErrType,
    message: Option<String>,
}

impl Error for ErrAutoType {}

impl Display for ErrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ErrType::*;
        match self {
            SequenceNotExist(s) => write!(f, "Key: {s} not found"),
            WrongSequenceArg(s) => write!(f, "Sequence argument \"{s}\" have wrong format"),
            InvalidKeyFormat(s) => write!(f, "Invalid sequence key: {s}"),
            KeyIsInSequences(s) => write!(f, "Key \"{s}\" is now in sequences"),
            UnknownSequence(s) => write!(f, "Sequence \"{s}\" is not registered"),
            KeyIsInCombinations(s) => write!(f, "Key \"{s}\" is now in combinations."),
        }
    }
}

impl Display for ErrAutoType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.message {
            Some(m) => write!(f, "Error: {} {}", self.err_type, m),
            None => write!(f, "Error: {}", self.err_type),
        }
    }
}

impl PartialEq for ErrAutoType {
    fn eq(&self, other: &Self) -> bool {
        self.err_type == other.err_type
    }
}

impl ErrAutoType {
    pub fn new(err_type: ErrType) -> ErrAutoType {
        ErrAutoType {
            err_type,
            message: None,
        }
    }

    pub fn new_with_message(err_type: ErrType, msg: String) -> ErrAutoType {
        ErrAutoType {
            err_type,
            message: Some(msg),
        }
    }

    pub fn get_type(&self) -> &ErrType {
        &self.err_type
    }

    pub fn get_message(&self) -> Option<&String> {
        self.message.as_ref()
    }
}
