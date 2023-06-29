use crate::basics::{CoverScale, CoverType};
use crate::{GlyphGenerator};

#[derive(Default, Clone, Copy)]
pub struct GlyphRect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub dx: f64,
    pub dy: f64,
}
impl GlyphRect {
	pub fn new() -> Self {
		Self {..Default::default()}
	}
}
//========================================================GlyphRasterBin
pub struct GlyphRasterBin {
    font: *const u8,
    big_endian: bool,
    span: [CoverType; 32],
    bits: *const u8,
    glyph_width: u32,
    glyph_byte_width: u32,
}

impl GlyphRasterBin {
    pub fn new(font: *const u8) -> GlyphRasterBin {
        let mut big_endian = false;
        let t = 1;
        if font != core::ptr::null() && unsafe { *(t as *const i8) } == 0 {
            big_endian = true;
        }
        GlyphRasterBin {
            font: font,
            big_endian: big_endian,
            span: [0; 32],
            bits: std::ptr::null(),
            glyph_width: 0,
            glyph_byte_width: 0,
        }
    }

    pub fn font(&self) -> *const u8 {
        self.font
    }

    pub fn set_font(&mut self, f: *const u8) {
        self.font = f;
    }

    pub fn height(&self) -> f64 {
        unsafe { *self.font as f64 }
    }

    pub fn base_line(&self) -> f64 {
        unsafe { *self.font.offset(1) as f64 }
    }

    pub fn width(&self, str: &str) -> f64 {
        let start_char = unsafe { *self.font.offset(2) };
        let num_chars = unsafe { *self.font.offset(3) };

        let mut w = 0;
        for c in str.chars() {
            let glyph = c as u8;
            unsafe {
                let bits = self.font.offset(
                    4 + num_chars as isize * 2
                        + self.value(self.font.offset(4 + (glyph - start_char) as isize * 2))
                            as isize,
                );
                w += *bits;
            }
        }
        w as f64
    }
	
    fn value(&self, p: *const u8) -> u16 {
        let mut v0: u16 = 0;
		let v = &mut v0 as *mut u16;
        if self.big_endian {
            unsafe {
                *(v as *mut u8) = *p.offset(1);
                *((v as *mut u8).offset(1)) = *p;
            }
        } else {
            unsafe {
                *(v as *mut u8) = *p;
                *((v as *mut u8).offset(1)) = *p.offset(1);
            }
        }
        v0
    }
}

impl GlyphGenerator for GlyphRasterBin {
    fn prepare(&mut self, r: &mut GlyphRect, x: f64, y: f64, glyph: u32, flip: bool) {
        let start_char = unsafe { *self.font.offset(2) } as u32;
        let num_chars = unsafe { *self.font.offset(3) } as u32;
        unsafe {
            self.bits = self.font.offset(
                4 + num_chars as isize * 2
                    + self.value(self.font.offset(4 + (glyph - start_char) as isize * 2)) as isize,
            );

            self.glyph_width =  *self.bits  as u32;
			self.bits = self.bits.offset(1);
        }
        self.glyph_byte_width = (self.glyph_width + 7) >> 3;

        r.x1 = x as i32;
        r.x2 = r.x1 + self.glyph_width as i32 - 1;
        if flip {
            r.y1 = y as i32 - self.height() as i32 + self.base_line() as i32;
            r.y2 = r.y1 + self.height() as i32 - 1;
        } else {
            r.y1 = y as i32 - self.base_line() as i32 + 1;
            r.y2 = r.y1 + self.height() as i32 - 1;
        }
        r.dx = self.glyph_width as f64;
        r.dy = 0.0;
    }

    fn span(&mut self, i0: u32) -> &mut [u8] {
        let i = unsafe { *self.font.offset(0) } as u32 - i0 - 1;
        let bits = unsafe {
            self.bits
                .offset(i as isize * self.glyph_byte_width as isize)
        };
        let mut val = unsafe { *bits };
        let mut nb = 0;
        for j in 0..self.glyph_width {
            self.span[j as usize] = {
                if (val & 0x80) != 0 {
                    CoverScale::FULL
                } else {
                    CoverScale::None
                }
            } as u8;
            val <<= 1;
			nb += 1;
            if nb >= 8 {
                val = unsafe { *bits.offset(1) };
                nb = 0;
            }
        }
        &mut self.span
    }
}
