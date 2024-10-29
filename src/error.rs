// sixela::error

use devela::Error;

/// A sixel-related result.
pub type SixelResult<T> = Result<T, SixelError>;

/// A sixel-related error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SixelError {
    /// Bad argument detected.
    BadArgument,
    /// Bad input detected.
    BadInput,
    /// Integer overflow.
    BadIntegerOverflow,
    /// Feature not implemented.
    NotImplemented,
}

mod _core_impls {
    use super::{Error, SixelError};
    use core::fmt;

    impl Error for SixelError {}

    impl fmt::Display for SixelError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SixelError::BadArgument => write!(f, "bad argument detected"),
                SixelError::BadInput => write!(f, "bad input detected"),
                SixelError::BadIntegerOverflow => write!(f, "integer overflow"),
                SixelError::NotImplemented => write!(f, "feature not implemented"),
            }
        }
    }
}
