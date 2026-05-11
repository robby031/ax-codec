#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    InsufficientCapacity,
    LimitExceeded,
}

impl core::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EncodeError::InsufficientCapacity => write!(f, "insufficient buffer capacity"),
            EncodeError::LimitExceeded => write!(f, "encoding limit exceeded"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedEOF,
    InvalidVarint,
    InvalidUtf8,
    AllocationLimitExceeded,
    DepthLimitExceeded,
    InvalidBool,
    InvalidOptionTag,
    InvalidChecksum,
    InvalidVersion,
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::UnexpectedEOF => write!(f, "unexpected end of buffer"),
            DecodeError::InvalidVarint => write!(f, "invalid varint encoding"),
            DecodeError::InvalidUtf8 => write!(f, "invalid utf-8 sequence"),
            DecodeError::AllocationLimitExceeded => write!(f, "allocation limit exceeded"),
            DecodeError::DepthLimitExceeded => write!(f, "depth limit exceeded"),
            DecodeError::InvalidBool => write!(f, "invalid boolean value"),
            DecodeError::InvalidOptionTag => write!(f, "invalid option tag"),
            DecodeError::InvalidChecksum => write!(f, "checksum mismatch"),
            DecodeError::InvalidVersion => write!(f, "unsupported wire format version"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidateError {
    UnexpectedEOF,
    InvalidVarint,
    InvalidUtf8,
    AllocationLimitExceeded,
    DepthLimitExceeded,
    InvalidBool,
    InvalidOptionTag,
    InvalidChecksum,
    InvalidVersion,
}

impl core::fmt::Display for ValidateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValidateError::UnexpectedEOF => write!(f, "unexpected end of buffer"),
            ValidateError::InvalidVarint => write!(f, "invalid varint encoding"),
            ValidateError::InvalidUtf8 => write!(f, "invalid utf-8 sequence"),
            ValidateError::AllocationLimitExceeded => write!(f, "allocation limit exceeded"),
            ValidateError::DepthLimitExceeded => write!(f, "depth limit exceeded"),
            ValidateError::InvalidBool => write!(f, "invalid boolean value"),
            ValidateError::InvalidOptionTag => write!(f, "invalid option tag"),
            ValidateError::InvalidChecksum => write!(f, "checksum mismatch"),
            ValidateError::InvalidVersion => write!(f, "unsupported wire format version"),
        }
    }
}

impl From<DecodeError> for ValidateError {
    fn from(e: DecodeError) -> Self {
        match e {
            DecodeError::UnexpectedEOF => ValidateError::UnexpectedEOF,
            DecodeError::InvalidVarint => ValidateError::InvalidVarint,
            DecodeError::InvalidUtf8 => ValidateError::InvalidUtf8,
            DecodeError::AllocationLimitExceeded => ValidateError::AllocationLimitExceeded,
            DecodeError::DepthLimitExceeded => ValidateError::DepthLimitExceeded,
            DecodeError::InvalidBool => ValidateError::InvalidBool,
            DecodeError::InvalidOptionTag => ValidateError::InvalidOptionTag,
            DecodeError::InvalidChecksum => ValidateError::InvalidChecksum,
            DecodeError::InvalidVersion => ValidateError::InvalidVersion,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ValidateError {}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}
