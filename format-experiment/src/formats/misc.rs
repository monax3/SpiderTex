use std::fmt::Display;

use serde::{Deserialize, Serialize};
use super::{DXGI_FORMAT, DxgiFormatExt};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq)]
pub struct Dimensions {
    pub data_size: usize,
    pub width: usize,
    pub height: usize,
    pub mipmaps: u8,
}

impl PartialEq for Dimensions {
    fn eq(&self, other: &Self) -> bool {
        self.wh() == other.wh()
    }
}
pub enum ColorPlanes {
    Rgb,
    Rgba,
    Hdr,
    Luma,
}

impl Dimensions {
    #[inline]
    #[must_use]
    pub const fn wh(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    #[inline]
    #[must_use]
    pub fn mip_levels(self, is_highres: bool) -> u8 {
        if !(self.width.is_power_of_two() && self.height.is_power_of_two()) {
            1
        } else if is_highres {
            2
        } else {
            let pow2 = std::cmp::min(self.width, self.height).trailing_zeros();
            if pow2 <= 5 {
                1
            } else {
                (pow2 - 2) as u8
            }
        }
    }

    pub fn data_size(&self, format: DXGI_FORMAT, array_size: usize) -> usize {
        let mut data_size = format.planes().bpp() * self.width * self.height * array_size / format.compression_ratio();
        // println!("{} * {} * {} * {} / {} = {data_size}", format.planes().bpp(), self.width, self.height, array_size, format.compression_ratio());
        let start = data_size.next_power_of_two().trailing_zeros() as usize;

        for i in 1 .. (self.mipmaps as usize) {
            // println!("{start} {}", 2*i);
            data_size += 1 << (start - 2 * i);
        }
        data_size
    }
}

impl Display for Dimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}/{}", self.width, self.height, self.mipmaps)
    }
}

impl ColorPlanes {
    fn bpp(&self)  -> usize{
        match self {
            ColorPlanes::Rgb => 4,
            ColorPlanes::Rgba => 4,
            ColorPlanes::Hdr => 12,
            ColorPlanes::Luma => 1,
        }
    }
}
