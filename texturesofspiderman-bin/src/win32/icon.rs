#![allow(clippy::upper_case_acronyms)]

use std::ffi::CStr;

use eframe::IconData;
use tracing::{event, Level};
use windows::core::{Error, Result, PCSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::Graphics::Gdi::{
    DeleteObject,
    GetDC,
    GetDIBits,
    ReleaseDC,
    BITMAPINFO,
    BITMAPINFOHEADER,
    BI_RGB,
    DIB_RGB_COLORS,
    HBITMAP,
    HDC,
    HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon,
    GetIconInfo,
    LoadImageA,
    HICON,
    ICONINFO,
    IMAGE_ICON,
    LR_DEFAULTCOLOR,
};

pub fn icon_data() -> Result<IconData> {
    const ICON_SIZE: u32 = 32;

    let hinstance = unsafe { GetModuleHandleA(None) }?;
    let icon = OwnedHICON::load_image(Some(hinstance), 1, ICON_SIZE as i32)?;
    let hdc = OwnedHDC::new()?;
    let (color, mask) = icon.bitmaps()?;

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: (ICON_SIZE as i32),
            biHeight: -(ICON_SIZE as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB as u32,
            ..BITMAPINFOHEADER::default()
        },
        ..BITMAPINFO::default()
    };

    let mut bits = unsafe {
        let mut bits = Vec::<u32>::with_capacity((ICON_SIZE * ICON_SIZE) as usize);
        if GetDIBits(
            &hdc,
            color.as_inner(),
            0,
            ICON_SIZE,
            bits.as_mut_ptr().cast(),
            &mut bmi,
            DIB_RGB_COLORS,
        ) != ICON_SIZE as i32
        {
            return Err(Error::from_win32());
        }
        bits.set_len(bits.capacity());
        bits
    };

    let mask_bits = unsafe {
        let mut mask_bits = Vec::<u32>::with_capacity((ICON_SIZE * ICON_SIZE) as usize);
        if GetDIBits(
            &hdc,
            mask.as_inner(),
            0,
            ICON_SIZE,
            mask_bits.as_mut_ptr().cast(),
            &mut bmi,
            DIB_RGB_COLORS,
        ) != ICON_SIZE as i32
        {
            return Err(Error::from_win32());
        }
        mask_bits.set_len(mask_bits.capacity());
        mask_bits
    };

    let mut rgba = Vec::<u8>::with_capacity((ICON_SIZE * ICON_SIZE) as usize * 4);

    for (p, m) in bits.iter_mut().zip(mask_bits) {
        if m == 0 {
            *p |= 0xff_00_00_00;
        }

        rgba.extend(p.to_le_bytes());
    }
    debug_assert_eq!(rgba.len(), rgba.capacity());

    Ok(IconData {
        width: ICON_SIZE as u32,
        height: ICON_SIZE as u32,
        rgba,
    })
}

pub enum PCSTROrNumber<'cstr> {
    PCSTR(&'cstr CStr),
    Number(u16),
}
impl<'cstr> PCSTROrNumber<'cstr> {
    #[must_use]
    pub const fn as_pcstr(&self) -> PCSTR {
        match self {
            PCSTROrNumber::PCSTR(cstr) => PCSTR(cstr.as_ptr().cast::<u8>()),
            PCSTROrNumber::Number(number) => PCSTR(*number as *const u8),
        }
    }
}

impl<'cstr> From<&'cstr CStr> for PCSTROrNumber<'cstr> {
    fn from(cstr: &'cstr CStr) -> Self { Self::PCSTR(cstr) }
}

impl From<u16> for PCSTROrNumber<'static> {
    fn from(number: u16) -> Self { Self::Number(number) }
}

impl<'cstr> From<PCSTROrNumber<'cstr>> for PCSTR {
    fn from(value: PCSTROrNumber<'cstr>) -> Self { value.as_pcstr() }
}

pub struct OwnedHICON(HICON);
impl OwnedHICON {
    pub fn load_image<'a>(
        hinstance: Option<HINSTANCE>,
        name: impl Into<PCSTROrNumber<'a>>,
        size: i32,
    ) -> Result<Self> {
        unsafe {
            LoadImageA(
                hinstance,
                name.into(),
                IMAGE_ICON,
                size,
                size,
                LR_DEFAULTCOLOR,
            )
        }
        .map(|handle| Self(HICON(handle.0)))
    }

    pub fn bitmaps(&self) -> Result<(OwnedHGDIOBJ<HBITMAP>, OwnedHGDIOBJ<HBITMAP>)> {
        let mut icon_info = ICONINFO::default();
        unsafe { GetIconInfo(self.0, &mut icon_info) }
            .ok()
            .map(|_| {
                (
                    OwnedHGDIOBJ(icon_info.hbmColor),
                    OwnedHGDIOBJ(icon_info.hbmMask),
                )
            })
    }
}

impl Drop for OwnedHICON {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                DestroyIcon(self.0);
            }
        }
    }
}

pub struct OwnedHDC {
    inner: HDC,
    hwnd:  Option<HWND>,
}
impl OwnedHDC {
    pub fn new() -> Result<Self> { Self::from_opt_hwnd(None) }

    fn from_opt_hwnd(hwnd: Option<HWND>) -> Result<Self> {
        let hdc = unsafe { GetDC(hwnd) };
        if hdc.is_invalid() {
            Err(Error::from_win32())
        } else {
            Ok(Self { inner: hdc, hwnd })
        }
    }
}

impl From<&OwnedHDC> for HDC {
    fn from(&OwnedHDC { inner, .. }: &OwnedHDC) -> Self { inner }
}

impl Drop for OwnedHDC {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(self.hwnd, self.inner);
        }
    }
}

pub struct OwnedHGDIOBJ<T>(T)
where T: Into<HGDIOBJ> + Copy;

impl<T> OwnedHGDIOBJ<T>
where T: Into<HGDIOBJ> + Copy
{
    pub fn as_hgdiobj(&self) -> HGDIOBJ { self.0.into() }

    pub const fn as_inner(&self) -> T { self.0 }
}

impl<T> From<T> for OwnedHGDIOBJ<T>
where T: Into<HGDIOBJ> + Copy
{
    fn from(obj: T) -> Self { Self(obj) }
}

impl<T> Drop for OwnedHGDIOBJ<T>
where T: Into<HGDIOBJ> + Copy
{
    fn drop(&mut self) {
        let hgdiobj: HGDIOBJ = self.0.into();
        if !hgdiobj.is_invalid() {
            unsafe {
                DeleteObject(hgdiobj);
            }
        }
    }
}
