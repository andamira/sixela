// sixela::error

use devela::{Box, Error};

/// A sixel-related result.
pub type SixelResult<T> = Result<T, Box<dyn Error>>; // TODO: IMPROVE

/// A sixel-related error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SixelError {
    /// Runtime error.
    RuntimeError,
    /// Logic error.
    LogicError,
    /// Feature not enabled.
    FeatureError,
    /// Errors caused by curl.
    LibcError,
    /// Errors occures in libc functions.
    CurlError,
    /// Errors occures in libjpeg functions.
    JpegError,
    /// Errors occures in libpng functions.
    PngError,
    /// Errors occures in gdk functions.
    GdkError,
    /// Errors occures in gd functions.
    GdError,
    /// Errors occures in stb_image functions.
    StbiError,
    /// Errors occures in stb_image_write functions.
    StbiwError,
    /// Interrupted by a signal.
    INTERRUPTED,
    /// malloc() failed.
    BadAllocation,
    /// Bad argument detected.
    BadArgument,
    /// Bad input detected.
    BadInput,
    /// Integer overflow.
    BadIntegerOverflow,
    /// Feature not implemented.
    NotImplemented,
}

mod core_impls {
    use super::{Error, SixelError};
    use core::fmt;

    impl Error for SixelError {}

    impl fmt::Display for SixelError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SixelError::RuntimeError => write!(f, "runtime error"),
                SixelError::LogicError => write!(f, "logic error"),
                SixelError::FeatureError => write!(f, "feature not enabled"),
                SixelError::LibcError => write!(f, "errors occures in libc functions"),
                SixelError::CurlError => write!(f, "errors caused by curl"),
                SixelError::JpegError => write!(f, "errors occures in libjpeg functions"),
                SixelError::PngError => write!(f, "errors occures in libpng functions"),
                SixelError::GdkError => write!(f, "errors occures in gdk functions"),
                SixelError::GdError => write!(f, "errors occures in gd functions"),
                SixelError::StbiError => write!(f, "errors occures in stb_image functions"),
                SixelError::StbiwError => write!(f, "errors occures in stb_image_write functions"),
                SixelError::INTERRUPTED => write!(f, "interrupted by a signal"),
                SixelError::BadAllocation => write!(f, "malloc() failed"),
                SixelError::BadArgument => write!(f, "bad argument detected"),
                SixelError::BadInput => write!(f, "bad input detected"),
                SixelError::BadIntegerOverflow => write!(f, "integer overflow"),
                SixelError::NotImplemented => write!(f, "feature not implemented"),
            }
        }
    }
}
