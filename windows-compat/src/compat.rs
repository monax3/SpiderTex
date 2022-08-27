use std::{ffi::OsStr, fmt::Display};

pub type Result<T> = std::result::Result<T, Error>;

#[repr(C)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Error(pub u32);

impl std::error::Error for Error {}

impl From<HRESULT> for Error {
    fn from(value: HRESULT) -> Self {
        Error(value.0)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HRESULT({})", self.0)
    }
}

#[repr(transparent)]
pub struct HRESULT(pub u32);

impl HRESULT {
    pub const S_OK: u32 = 0;

    pub fn ok(self) -> Result<()> {
        if self.0 == Self::S_OK {
            Ok(())
        } else {
            Err(self.into())
        }
    }
}

pub struct HSTRING(Vec<u16>);

impl HSTRING {
    pub fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr()
    }

    fn from_str(string: impl AsRef<str>) -> Self {
        Self(string.as_ref().encode_utf16().chain(std::iter::once(0)).collect())
    }
}

impl From<&OsStr> for HSTRING {
    fn from(from: &OsStr) -> HSTRING {
        Self(from.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect())
    }
}

impl From<&str> for HSTRING {
    fn from(value: &str) -> HSTRING {
        HSTRING::from_str(value)
    }
}
