use windows_sys::{Win32::Foundation::*, Win32::Graphics::Gdi::*};

use core::ptr::*;
use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::mem::*;
use std::ptr;
use std::str;

#[derive(Copy, Clone)]
pub struct PixelMap {
    bmp: *mut BITMAPINFO,
    buf: *mut u8,
    bpp: u32,
    is_internal: bool,
    img_size: u32,
    full_size: u32,
}

#[repr(u32)]
pub enum Org {
    Undefined = 0,
    Mono8 = 8,
    Color16 = 16,
    Color24 = 24,
    Color32 = 32,
    Color48 = 48,
    Color64 = 64,
}

/*impl Drop for PixelMap {
    fn drop(&mut self) {
        self.destroy();
    }
}*/

impl PixelMap {
    pub fn new() -> Self {
        PixelMap {
            bmp: null_mut(),
            buf: 0 as *mut u8,
            bpp: 9,
            is_internal: false,
            img_size: 0,
            full_size: 0,
        }
    }

    pub fn destroy(&mut self) {
        if !self.bmp.is_null() && self.is_internal {
            unsafe {
                libc::free(self.bmp as *mut libc::c_void);
            }
        }
        self.bmp = 0 as *mut BITMAPINFO;
        self.is_internal = false;
        self.buf = 0 as *mut u8;
    }

    pub fn bpp(&self) -> u32 {
        self.bpp
    }

    pub fn create(&mut self, w: u32, h: u32, org: Org, clear_val: u32 /* = 256*/) {
        let (mut width, mut height) = (w, h);
        self.destroy();
        if width == 0 {
            width = 1;
        }
        if height == 0 {
            height = 1;
        }
        self.bpp = org as u32;
        self.create_from_bmp(Self::create_bitmap_info(width, height, self.bpp));
        Self::create_gray_scale_palette(self.bmp);
        self.is_internal = true;
        if clear_val <= 255 {
            unsafe {
                std::ptr::write_bytes(self.buf, clear_val as u8, self.img_size as usize);
            }
        }
    }

    pub fn create_dib_section(
        &mut self, h_dc: HDC, w: u32, h: u32, org: Org, clear_val: u32, /* = 256*/
    ) -> HBITMAP {
        let (mut width, mut height) = (w, h);
        self.destroy();
        if width == 0 {
            width = 1;
        }
        if height == 0 {
            height = 1;
        }
        self.bpp = org as u32;
        let h_bitmap = self.create_dib_section_from_args(h_dc, width, height, self.bpp);
        Self::create_gray_scale_palette(self.bmp);
        self.is_internal = true;
        if clear_val <= 255 {
            unsafe {
                std::ptr::write_bytes(self.buf, clear_val as u8, self.img_size as usize);
            }
        }
        h_bitmap
    }

    pub fn clear(&mut self, clear_val: u32) {
        if !self.buf.is_null() {
            unsafe {
                std::ptr::write_bytes(self.buf, clear_val as u8, self.img_size as usize);
            }
        }
    }

    pub fn attach_to_bmp(&mut self, bmp: *mut BITMAPINFO) {
        if !bmp.is_null() {
            self.destroy();
            self.create_from_bmp(bmp);
            self.is_internal = false;
        }
    }

    pub fn calc_full_size(bmp: *mut BITMAPINFO) -> u32 {
        if bmp.is_null() {
            return 0;
        }
        return (size_of::<BITMAPINFOHEADER>()
            + size_of::<RGBQUAD>() * Self::calc_palette_size(bmp) as usize
            + unsafe { (*bmp).bmiHeader.biSizeImage } as usize) as u32;
    }

    pub fn calc_header_size(bmp: *mut BITMAPINFO) -> u32 {
        if bmp.is_null() {
            return 0;
        }
        return (size_of::<BITMAPINFOHEADER>()
            + size_of::<RGBQUAD>() * Self::calc_palette_size(bmp) as usize) as u32;
    }

    pub fn calc_palette_size_bpp(clr_used: u32, bits_per_pixel: u32) -> u32 {
        let mut palette_size = 0;
        if bits_per_pixel <= 8 {
            palette_size = clr_used as i32;
            if palette_size == 0 {
                palette_size = 1 << bits_per_pixel;
            }
        }
        return palette_size as u32;
    }

    pub fn calc_palette_size(bmp: *mut BITMAPINFO) -> u32 {
        if bmp.is_null() {
            return 0;
        }
        return Self::calc_palette_size_bpp(unsafe { (*bmp).bmiHeader.biClrUsed }, unsafe {
            (*bmp).bmiHeader.biBitCount as u32
        });
    }

    pub fn calc_img_ptr(bmp: *mut BITMAPINFO) -> *mut u8 {
        if bmp.is_null() {
            return 0 as *mut u8;
        }
        return unsafe { (bmp as *mut u8).offset(Self::calc_header_size(bmp) as isize) };
    }

    fn create_bitmap_info(width: u32, height: u32, bits_per_pixel: u32) -> *mut BITMAPINFO {
        let line_len = Self::calc_row_len(width, bits_per_pixel);
        let img_size = line_len * height;
        let rgb_size =
            Self::calc_palette_size_bpp(0, bits_per_pixel) * mem::size_of::<RGBQUAD>() as u32;
        let full_size = mem::size_of::<BITMAPINFOHEADER>() + rgb_size as usize + img_size as usize;

        let bmp = unsafe {
            std::alloc::alloc(std::alloc::Layout::from_size_align(full_size, 1).unwrap())
        } as *mut BITMAPINFO;
        let bmpr = unsafe { &mut *(bmp as *mut BITMAPINFOHEADER) };

        bmpr.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmpr.biWidth = width as i32;
        bmpr.biHeight = height as i32;
        bmpr.biPlanes = 1;
        bmpr.biBitCount = bits_per_pixel as u16;
        bmpr.biCompression = 0;
        bmpr.biSizeImage = img_size;
        bmpr.biXPelsPerMeter = 0;
        bmpr.biYPelsPerMeter = 0;
        bmpr.biClrUsed = 0;
        bmpr.biClrImportant = 0;

        bmp
    }

    fn create_gray_scale_palette(bmp: *mut BITMAPINFO) {
        unsafe {
            let rgb_size = Self::calc_palette_size(bmp);
            let mut rgb = ((bmp as *const u8).offset(mem::size_of::<BITMAPINFOHEADER>() as isize))
                as *mut RGBQUAD;
            let mut brightness;

            for i in 0..rgb_size {
                brightness = (255 * i) / (rgb_size - 1);
                (*rgb).rgbBlue = brightness as u8;
                (*rgb).rgbGreen = brightness as u8;
                (*rgb).rgbRed = brightness as u8;
                (*rgb).rgbReserved = 0;
                rgb = rgb.offset(1);
            }
        }
    }

    fn calc_row_len(width: u32, bits_per_pixel: u32) -> u32 {
        let mut n = width;
        let k;

        match bits_per_pixel {
            1 => {
                k = n;
                n = n >> 3;
                if k & 7 != 0 {
                    n += 1;
                }
            }
            4 => {
                k = n;
                n = n >> 1;
                if k & 3 != 0 {
                    n += 1;
                }
            }
            8 => {}
            16 => {
                n *= 2;
            }
            24 => {
                n *= 3;
            }
            32 => {
                n *= 4;
            }
            48 => {
                n *= 6;
            }
            64 => {
                n *= 8;
            }
            _ => {
                n = 0;
            }
        }
        ((n + 3) >> 2) << 2
    }

    pub fn draw(&self, h_dc: HDC, device_rect: Option<&RECT>, bmp_rect: Option<&RECT>) {
        if self.bmp.is_null() || self.buf.is_null() {
            return;
        }

        let mut bmp_x = 0;
        let mut bmp_y = 0;
        let mut bmp_width = unsafe { *self.bmp }.bmiHeader.biWidth;
        let mut bmp_height = unsafe { *self.bmp }.bmiHeader.biHeight;
        let mut dvc_x;
        let mut dvc_y;
        let mut dvc_width;
        let mut dvc_height;

        if let Some(bmp_rect) = bmp_rect {
            bmp_x = bmp_rect.left;
            bmp_y = bmp_rect.top;
            bmp_width = bmp_rect.right - bmp_rect.left;
            bmp_height = bmp_rect.bottom - bmp_rect.top;
        }

        dvc_x = bmp_x;
        dvc_y = bmp_y;
        dvc_width = bmp_width;
        dvc_height = bmp_height;

        if let Some(device_rect) = device_rect {
            dvc_x = device_rect.left;
            dvc_y = device_rect.top;
            dvc_width = device_rect.right - device_rect.left;
            dvc_height = device_rect.bottom - device_rect.top;
        }

        if dvc_width != bmp_width || dvc_height != bmp_height {
            unsafe {
                SetStretchBltMode(h_dc, COLORONCOLOR);
                StretchDIBits(
                    h_dc,
                    dvc_x,
                    dvc_y,
                    dvc_width,
                    dvc_height,
                    bmp_x,
                    bmp_y,
                    bmp_width,
                    bmp_height,
                    self.buf as *const libc::c_void,
                    self.bmp,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
        } else {
            unsafe {
                SetDIBitsToDevice(
                    h_dc,
                    dvc_x,
                    dvc_y,
                    dvc_width as u32,
                    dvc_height as u32,
                    bmp_x,
                    bmp_y,
                    0,
                    bmp_height as u32,
                    self.buf as *const libc::c_void,
                    self.bmp,
                    DIB_RGB_COLORS,
                );
            }
        }
    }

    pub fn draw_xy(&self, h_dc: HDC, x: i32, y: i32, scale: f64 /*1.0 */) {
        if self.bmp.is_null() || self.buf.is_null() {
            return;
        }

        let width = (unsafe { *self.bmp }.bmiHeader.biWidth as f64 * scale) as u32;
        let height = (unsafe { *self.bmp }.bmiHeader.biHeight as f64 * scale) as u32;
        let mut rect: RECT = unsafe { mem::zeroed() };
        rect.left = x;
        rect.top = y;
        rect.right = x + width as i32;
        rect.bottom = y + height as i32;
        self.draw(h_dc, Some(&rect), None);
    }

    pub fn blend_rect(&self, h_dc: HDC, device_rect: Option<&RECT>, bmp_rect: Option<&RECT>) {
        self.draw(h_dc, device_rect, bmp_rect);
        return;
    }

    //OR AGG_BMP_ALPHA_BLEND
    /*
       pub fn blend__(&self, h_dc: HDC, device_rect: Option<&RECT>, bmp_rect: Option<&RECT>) {
           if self.bpp != 32 {
               self.draw(h_dc, device_rect, bmp_rect);
               return;
           }

           if self.bmp.is_null() || self.buf.is_null() {
               return;
           }

           let mut bmp_x = 0;
           let mut bmp_y = 0;
           let mut bmp_width = self.bmp.as_ref().unwrap().bmiHeader.biWidth;
           let mut bmp_height = self.bmp.as_ref().unwrap().bmiHeader.biHeight;
           let mut dvc_x = 0;
           let mut dvc_y = 0;
           let mut dvc_width = self.bmp.as_ref().unwrap().bmiHeader.biWidth;
           let mut dvc_height = self.bmp.as_ref().unwrap().bmiHeader.biHeight;

           if let Some(bmp_rect) = bmp_rect {
               bmp_x = bmp_rect.left;
               bmp_y = bmp_rect.top;
               bmp_width = bmp_rect.right - bmp_rect.left;
               bmp_height = bmp_rect.bottom - bmp_rect.top;
           }

           dvc_x = bmp_x;
           dvc_y = bmp_y;
           dvc_width = bmp_width;
           dvc_height = bmp_height;

           if let Some(device_rect) = device_rect {
               dvc_x = device_rect.left;
               dvc_y = device_rect.top;
               dvc_width = device_rect.right - device_rect.left;
               dvc_height = device_rect.bottom - device_rect.top;
           }

           let mem_dc = unsafe { CreateCompatibleDC(h_dc) };
           let mut buf: *mut c_void = ptr::null_mut();
           let bmp = unsafe {
               CreateDIBSection(
                   mem_dc,
                   self.bmp,
                   DIB_RGB_COLORS,
                   &mut buf,
                   0,
                   0,
               )
           };
           unsafe {
               ptr::copy_nonoverlapping(
                   self.buf,
                   buf as *mut u8,
                   (*self.bmp).bmiHeader.biSizeImage as usize,
               );
           }

           let temp = unsafe { SelectObject(mem_dc, bmp as HGDIOBJ) };

           let mut blend = BLENDFUNCTION {
               BlendOp: AC_SRC_OVER as u8,
               BlendFlags: 0,
               SourceConstantAlpha: 255,
               AlphaFormat: AC_SRC_ALPHA as u8,
           };

           unsafe {
               AlphaBlend(
                   h_dc, dvc_x, dvc_y, dvc_width, dvc_height, mem_dc, bmp_x, bmp_y, bmp_width,
                   bmp_height, blend,
               );
           }

           unsafe {
               SelectObject(mem_dc, temp as HGDIOBJ);
               DeleteObject(bmp as HGDIOBJ);
               DeleteObject(mem_dc as HGDIOBJ);
           }
       }
    */

    pub fn blend(&self, h_dc: HDC, x: i32, y: i32, scale: f64 /* 1.0 */) {
        if self.bmp.is_null() || self.buf.is_null() {
            return;
        }

        let width = (unsafe { *self.bmp }.bmiHeader.biWidth as f64 * scale) as u32;
        let height = (unsafe { *self.bmp }.bmiHeader.biHeight as f64 * scale) as u32;
        let rect = RECT {
            left: x,
            top: y,
            right: x + width as i32,
            bottom: y + height as i32,
        };
        self.blend_rect(h_dc, Some(&rect), None);
    }

    pub fn load_from_bmp(&mut self, fd: &mut File)  -> bool {
        let mut bmf: BITMAPFILEHEADER = unsafe { mem::zeroed() };
        let bmi;
        let bmp_size;

        unsafe {
            fd.read(std::slice::from_raw_parts_mut(
                &mut bmf as *mut BITMAPFILEHEADER as *mut u8,
                mem::size_of::<BITMAPFILEHEADER>(),
            ))
            .unwrap();
            if bmf.bfType != 0x4D42 {
                return false;
            }

            bmp_size = bmf.bfSize - mem::size_of::<BITMAPFILEHEADER>() as u32;

            bmi = std::alloc::alloc(
                std::alloc::Layout::from_size_align(bmp_size as usize, 1).unwrap(),
            ) as *mut BITMAPINFO;

            if let Ok(n) = fd.read(std::slice::from_raw_parts_mut(
                bmi as *mut u8,
                bmp_size as usize,
            )) {
                if n != bmp_size as usize {
                    return false;
                }
            } else {
                return false;
            }
            self.destroy();
            self.bpp = (*bmi).bmiHeader.biBitCount as u32;
            self.create_from_bmp(bmi);
            self.is_internal = true;
        }
        return true;
    }

    pub fn load_from_bmp_str(&mut self, filename: &str)  -> bool {
        let fd = File::open(filename);
        if let Ok(mut fd) = fd {
            return self.load_from_bmp(&mut fd);
        }
        false
    }

    pub fn save_as_bmp(&self, fd: &mut File)  -> bool {
        if self.bmp.is_null() {
            return false;
        }

        let mut bmf: BITMAPFILEHEADER = unsafe { mem::zeroed() };

        bmf.bfType = 0x4D42;
        bmf.bfOffBits =
            Self::calc_header_size(self.bmp) + mem::size_of::<BITMAPFILEHEADER>() as u32;
        bmf.bfSize = bmf.bfOffBits + self.img_size;
        bmf.bfReserved1 = 0;
        bmf.bfReserved2 = 0;

        unsafe {
            fd.write(std::slice::from_raw_parts(
                &bmf as *const BITMAPFILEHEADER as *const u8,
                mem::size_of::<BITMAPFILEHEADER>(),
            ))
            .unwrap();
            fd.write(std::slice::from_raw_parts(
                self.bmp as *const u8,
                self.full_size as usize,
            ))
            .unwrap();
        }
        return true;
    }

    pub fn save_as_bmp_str(&self, filename: &str)  -> bool {
        let fd = File::open(filename);
        if let Ok(mut fd) = fd {
            return self.save_as_bmp(&mut fd);
        }
        false
    }

    pub fn buf(&self) -> *mut u8 {
        return self.buf;
    }

    pub fn width(&self) -> u32 {
        return unsafe { *self.bmp }.bmiHeader.biWidth as u32;
    }

    pub fn height(&self) -> u32 {
        return unsafe { *self.bmp }.bmiHeader.biHeight as u32;
    }

    pub fn stride(&self) -> i32 {
        return unsafe {
            Self::calc_row_len(
                (*self.bmp).bmiHeader.biWidth as u32,
                (*self.bmp).bmiHeader.biBitCount as u32,
            )
        } as i32;
    }

    //private
    fn create_from_bmp(&mut self, bmp: *mut BITMAPINFO) {
        if !bmp.is_null() {
            self.img_size = unsafe {
                Self::calc_row_len(
                    (*bmp).bmiHeader.biWidth as u32,
                    (*bmp).bmiHeader.biBitCount as u32,
                ) * (*bmp).bmiHeader.biHeight as u32
            };

            self.full_size = Self::calc_full_size(bmp);
            self.bmp = bmp;
            self.buf = Self::calc_img_ptr(bmp);
        }
    }

    //private

    fn create_dib_section_from_args(
        &mut self, h_dc: HDC, width: u32, height: u32, bits_per_pixel: u32,
    ) -> HBITMAP {
        let line_len = Self::calc_row_len(width, bits_per_pixel);
        let img_size = line_len * height;
        let rgb_size =
            Self::calc_palette_size_bpp(0, bits_per_pixel) * mem::size_of::<RGBQUAD>() as u32;
        let full_size = mem::size_of::<BITMAPINFOHEADER>() as u32 + rgb_size;

        let mut bmp = unsafe {
            std::alloc::alloc(std::alloc::Layout::from_size_align(full_size as usize, 1).unwrap())
        } as *mut BITMAPINFO;

        unsafe {
            (*bmp).bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
            (*bmp).bmiHeader.biWidth = width as i32;
            (*bmp).bmiHeader.biHeight = height as i32;
            (*bmp).bmiHeader.biPlanes = 1;
            (*bmp).bmiHeader.biBitCount = bits_per_pixel as u16;
            (*bmp).bmiHeader.biCompression = 0;
            (*bmp).bmiHeader.biSizeImage = img_size;
            (*bmp).bmiHeader.biXPelsPerMeter = 0;
            (*bmp).bmiHeader.biYPelsPerMeter = 0;
            (*bmp).bmiHeader.biClrUsed = 0;
            (*bmp).bmiHeader.biClrImportant = 0;
        }

        let mut img_ptr: *mut u8 = ptr::null_mut();
        let h_bitmap = unsafe {
            CreateDIBSection(
                h_dc,
                bmp,
                DIB_RGB_COLORS,
                &mut img_ptr as *mut *mut u8 as *mut *mut ::core::ffi::c_void,
                0,
                0,
            )
        };

        if !img_ptr.is_null() {
            self.img_size = Self::calc_row_len(width, bits_per_pixel) * height;
            self.full_size = 0;
            self.bmp = bmp;
            self.buf = img_ptr;
        }

        return h_bitmap;
    }
}
