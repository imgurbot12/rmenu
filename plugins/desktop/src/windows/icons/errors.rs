/// Stolen From: https://github.com/TeamDman/Cursor-Hero/blob/main/crates/winutils/src/win_errors.rs
/// License: MPL-2.0 (https://github.com/TeamDman/Cursor-Hero/blob/main/LICENSE)
use std::{rc::Rc, string::FromUtf16Error};
use widestring::error::ContainsNul;

#[derive(Debug, Clone)]
pub enum Error {
    Windows(windows::core::Error),
    WideString(ContainsNul<u16>),
    FromUtf16Error,
    Described(Rc<Error>, String),
    ImageContainerNotBigEnough,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Windows(e) => write!(f, "Windows error: {}", e.message()),
            Error::WideString(e) => write!(f, "Wide string error: {}", e),
            Error::FromUtf16Error => write!(f, "FromUtf16Error"),
            Error::Described(e, description) => write!(f, "{}: {}", e, description),
            Error::ImageContainerNotBigEnough => write!(f, "Image container not big enough"),
        }
    }
}
impl std::error::Error for Error {}
impl Error {
    pub fn from_win32() -> Self {
        Error::Windows(windows::core::Error::from_win32())
    }
    pub fn with_description(self, description: String) -> Self {
        Error::Described(Rc::new(self), description)
    }
}
impl From<windows::core::Error> for Error {
    fn from(e: windows::core::Error) -> Self {
        Error::Windows(e)
    }
}
impl From<ContainsNul<u16>> for Error {
    fn from(e: ContainsNul<u16>) -> Self {
        Error::WideString(e)
    }
}
impl From<FromUtf16Error> for Error {
    fn from(_e: FromUtf16Error) -> Self {
        Error::FromUtf16Error
    }
}

pub type Result<T> = std::result::Result<T, Error>;
