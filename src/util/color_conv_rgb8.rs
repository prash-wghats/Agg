//----------------------------------------------------------------------------
// Anti-Grain Geometry - Version 2.4
// Copyright (C) 2002-2005 Maxim Shemanarev (http://www.antigrain.com)
//
// Permission to copy, use, modify, sell and distribute this software
// is granted provided this copyright notice appears in all copies.
// This software is provided "as is" without express or implied
// warranty, and with no claim as to its suitability for any purpose.
//
//----------------------------------------------------------------------------
// Contact: mcseem@antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------
//
// A set of functors used with color_conv(). See file agg_color_conv.h
// These functors can convert images with up to 8 bits per component.
// Use convertors in the following way:
//
// agg::color_conv(dst, src, agg::color_conv_XXXX_to_YYYY());
// whare XXXX and YYYY can be any of:
//  rgb24
//  bgr24
//  rgba32
//  abgr32
//  argb32
//  bgra32
//  rgb555
//  rgb565
//----------------------------------------------------------------------------

//-----------------------------------------------------color_conv_rgb24
pub struct ConverterRgb24;
impl ConverterRgb24 {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 3).step_by(3) {
            dst[i + 0] = src[j + 2];
            dst[i + 1] = src[j + 1];
            dst[i + 2] = src[j + 0];
            j += 3;
        }
    }
}

pub type Rgb24ToBgr24 = ConverterRgb24;
pub type Bgr24ToRgb24 = ConverterRgb24;

//------------------------------------------------------color_conv_rgba32
pub struct ConverterRgba32<const I1: usize, const I2: usize, const I3: usize, const I4: usize>;
impl<const I1: usize, const I2: usize, const I3: usize, const I4: usize>
    ConverterRgba32<I1, I2, I3, I4>
{
    //------------------------------------------------------color_conv_rgba32
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 4).step_by(4) {
            dst[i + 0] = src[j + I1];
            dst[i + 1] = src[j + I2];
            dst[i + 2] = src[j + I3];
            dst[i + 3] = src[j + I4];
            j += 4;
        }
    }
}
pub type Argb32ToAbgr32 = ConverterRgba32<0, 3, 2, 1>;
pub type Argb32ToBgra32 = ConverterRgba32<3, 2, 1, 0>;
pub type Argb32ToRgba32 = ConverterRgba32<1, 2, 3, 0>;

pub type Bgra32ToAbgr32 = ConverterRgba32<3, 0, 1, 2>;
pub type Bgra32ToArgb32 = ConverterRgba32<3, 2, 1, 0>;
pub type Bgra32ToRgba32 = ConverterRgba32<2, 1, 0, 3>;

pub type Rgba32ToAbgr32 = ConverterRgba32<3, 2, 1, 0>;
pub type Rgba32ToArgb32 = ConverterRgba32<3, 0, 1, 2>;
pub type Rgba32ToBgra32 = ConverterRgba32<2, 1, 0, 3>;

pub type Abgr32ToArgb32 = ConverterRgba32<0, 3, 2, 1>;
pub type Abgr32ToBgra32 = ConverterRgba32<1, 2, 3, 0>;
pub type Abgr32ToRgba32 = ConverterRgba32<3, 2, 1, 0>;

//--------------------------------------------color_conv_rgb24_rgba32
pub struct ConverterRgb24ToRgba32<const I1: usize, const I2: usize, const I3: usize, const A: usize>;

impl<const I1: usize, const I2: usize, const I3: usize, const A: usize>
    ConverterRgb24ToRgba32<I1, I2, I3, A>
{
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 4).step_by(4) {
            dst[i + I1] = src[j + 0];
            dst[i + I2] = src[j + 1];
            dst[i + I3] = src[j + 2];
            dst[i + A] = 255;
            j += 3;
        }
    }
}

pub type Rgb24ToArgb32 = ConverterRgb24ToRgba32<1, 2, 3, 0>; //----color_conv_rgb24_to_argb32
pub type Rgb24ToAbgr32 = ConverterRgb24ToRgba32<3, 2, 1, 0>; //----color_conv_rgb24_to_abgr32
pub type Rgb24ToBgra32 = ConverterRgb24ToRgba32<2, 1, 0, 3>; //----color_conv_rgb24_to_bgra32
pub type Rgb24ToRgba32 = ConverterRgb24ToRgba32<0, 1, 2, 3>; //----color_conv_rgb24_to_rgba32
pub type Bgr24ToArgb32 = ConverterRgb24ToRgba32<3, 2, 1, 0>; //----color_conv_bgr24_to_argb32
pub type Bgr24ToAbgr32 = ConverterRgb24ToRgba32<1, 2, 3, 0>; //----color_conv_bgr24_to_abgr32
pub type Bgr24ToBgra32 = ConverterRgb24ToRgba32<0, 1, 2, 3>; //----color_conv_bgr24_to_bgra32
pub type Bgr24ToRgba32 = ConverterRgb24ToRgba32<2, 1, 0, 3>; //----color_conv_bgr24_to_rgba32

//-------------------------------------------------color_conv_rgba32_rgb24
pub struct ConverterRgba32ToRgb24<const I1: usize, const I2: usize, const I3: usize>;
impl<const I1: usize, const I2: usize, const I3: usize> ConverterRgba32ToRgb24<I1, I2, I3> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 3).step_by(3) {
            dst[i + 0] = src[j + I1];
            dst[i + 1] = src[j + I2];
            dst[i + 2] = src[j + I3];
            j += 4;
        }
    }
}

pub type Argb32ToRgb24 = ConverterRgba32ToRgb24<1, 2, 3>; //----color_conv_argb32_to_rgb24
pub type Abgr32ToRgb24 = ConverterRgba32ToRgb24<3, 2, 1>; //----color_conv_abgr32_to_rgb24
pub type Bgra32ToRgb24 = ConverterRgba32ToRgb24<2, 1, 0>; //----color_conv_bgra32_to_rgb24
pub type Rgba32ToRgb24 = ConverterRgba32ToRgb24<0, 1, 2>; //----color_conv_rgba32_to_rgb24
pub type Argb32ToBgr24 = ConverterRgba32ToRgb24<3, 2, 1>; //----color_conv_argb32_to_bgr24
pub type Abgr32ToBgr24 = ConverterRgba32ToRgb24<1, 2, 3>; //----color_conv_abgr32_to_bgr24
pub type Bgra32ToBgr24 = ConverterRgba32ToRgb24<0, 1, 2>; //----color_conv_bgra32_to_bgr24
pub type Rgba32ToBgr24 = ConverterRgba32ToRgb24<2, 1, 0>; //----color_conv_rgba32_to_bgr24
                                                          //------------------------------------------------color_conv_rgb555_rgb24
pub struct ConverterRgb555ToRgb24<const R: usize, const B: usize>;
impl<const R: usize, const B: usize> ConverterRgb555ToRgb24<R, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 3).step_by(3) {
            let rgb = unsafe { *(&src[j] as *const u8 as *const u16) };
            dst[i + R] = ((rgb >> 7) & 0xF8) as u8;
            dst[i + 1] = ((rgb >> 2) & 0xF8) as u8;
            dst[i + B] = ((rgb << 3) & 0xF8) as u8;
            j += 2;
        }
    }
}

pub type Rgb555ToBgr24 = ConverterRgb555ToRgb24<2, 0>; //----color_conv_rgb555_to_bgr24
pub type Rgb555ToRgb24 = ConverterRgb555ToRgb24<0, 2>; //----color_conv_rgb555_to_rgb24

//------------------------------------------------color_conv_rgb24_rgb555
pub struct ConverterRgb24ToRgb555<const R: usize, const B: usize>;
impl<const R: usize, const B: usize> ConverterRgb24ToRgb555<R, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        unsafe {
            for i in (0..width as usize * 2).step_by(2) {
                *(&mut dst[i] as *mut u8 as *mut u16) = ((((src[j + R] as u32) << 7) & 0x7C00)
                    | (((src[j + 1] as u32) << 2) & 0x3E0)
                    | ((src[j + B] as u32) >> 3))
                    as u16;
                j += 3;
            }
        }
    }
}

pub type Bgr24ToRgb555 = ConverterRgb24ToRgb555<2, 0>; //----color_conv_bgr24_to_rgb555
pub type Rgb24ToRgb555 = ConverterRgb24ToRgb555<0, 2>; //----color_conv_rgb24_to_rgb555

//-------------------------------------------------color_conv_rgb565_rgb24
pub struct ConverterRgb565ToRgb24<const R: usize, const B: usize>;
impl<const R: usize, const B: usize> ConverterRgb565ToRgb24<R, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 3).step_by(3) {
            let rgb = unsafe { *(&src[j] as *const u8 as *const u16) };
            dst[i + R] = ((rgb >> 8) & 0xF8) as u8;
            dst[i + 1] = ((rgb >> 3) & 0xFC) as u8;
            dst[i + B] = ((rgb << 3) & 0xF8) as u8;

            j += 2;
        }
    }
}

pub type Rgb565ToBgr24 = ConverterRgb565ToRgb24<2, 0>; //----color_conv_rgb565_to_bgr24
pub type Rgb565ToRgb24 = ConverterRgb565ToRgb24<0, 2>; //----color_conv_rgb565_to_rgb24

//-------------------------------------------------color_conv_rgb24_rgb565
pub struct ConverterRgb24ToRgb565<const R: usize, const B: usize>;
impl<const R: usize, const B: usize> ConverterRgb24ToRgb565<R, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        unsafe {
            for i in (0..width as usize * 2).step_by(2) {
                *(&mut dst[i] as *mut u8 as *mut u16) = ((((src[j + R] as u32) << 8) & 0xF800)
                    | (((src[j + 1] as u32) << 3) & 0x7E0)
                    | ((src[j + B] as u32) >> 3))
                    as u16;

                j += 3;
            }
        }
    }
}

pub type Bgr24ToRgb565 = ConverterRgb24ToRgb565<2, 0>; //----color_conv_bgr24_to_rgb565
pub type Rgb24ToRgb565 = ConverterRgb24ToRgb565<0, 2>; //----color_conv_rgb24_to_rgb565

//-------------------------------------------------color_conv_rgb555_rgba32
pub struct ConverterRgb555ToRgba32<const R: usize, const G: usize, const B: usize, const A: usize>;
impl<const R: usize, const G: usize, const B: usize, const A: usize>
    ConverterRgb555ToRgba32<R, G, B, A>
{
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 4).step_by(4) {
            let rgb = unsafe { *(&src[j] as *const u8 as *const u16) };
            dst[i + R] = ((rgb >> 7) & 0xF8) as u8;
            dst[i + G] = ((rgb >> 2) & 0xF8) as u8;
            dst[i + B] = ((rgb << 3) & 0xF8) as u8;
            dst[i + A] = (rgb >> 15) as u8;
            j += 2;
        }
    }
}

pub type Rgb555ToArgb32 = ConverterRgb555ToRgba32<1, 2, 3, 0>; //----color_conv_rgb555_to_argb32
pub type Rgb555ToAbgr32 = ConverterRgb555ToRgba32<3, 2, 1, 0>; //----color_conv_rgb555_to_abgr32
pub type Rgb555ToBgra32 = ConverterRgb555ToRgba32<2, 1, 0, 3>; //----color_conv_rgb555_to_bgra32
pub type Rgb555ToRgba32 = ConverterRgb555ToRgba32<0, 1, 2, 3>; //----color_conv_rgb555_to_rgba32

//------------------------------------------------color_conv_rgba32_rgb555
pub struct ConverterRgba32ToRgb555<const R: usize, const G: usize, const B: usize, const A: usize>;
impl<const R: usize, const G: usize, const B: usize, const A: usize>
    ConverterRgba32ToRgb555<R, G, B, A>
{
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        unsafe {
            for i in (0..width as usize * 2).step_by(2) {
                *(&mut dst[i] as *mut u8 as *mut u16) = ((((src[j + R] as u32) << 7) & 0x7C00)
                    | (((src[j + G] as u32) << 2) & 0x3E0)
                    | ((src[j + B] as u32) >> 3)
                    | (((src[j + A] as u32) << 2) & 0x8000))
                    as u16;

                j += 4;
            }
        }
    }
}
pub type Argb32ToRgb555 = ConverterRgba32ToRgb555<1, 2, 3, 0>; //----color_conv_argb32_to_rgb555
pub type Abgr32ToRgb555 = ConverterRgba32ToRgb555<3, 2, 1, 0>; //----color_conv_abgr32_to_rgb555
pub type Bgra32ToRgb555 = ConverterRgba32ToRgb555<2, 1, 0, 3>; //----color_conv_bgra32_to_rgb555
pub type Rgba32ToRgb555 = ConverterRgba32ToRgb555<0, 1, 2, 3>; //----color_conv_rgba32_to_rgb555

//------------------------------------------------color_conv_rgb565_rgba32
pub struct ConverterRgb565ToRgba32<const R: usize, const G: usize, const B: usize, const A: usize>;
impl<const R: usize, const G: usize, const B: usize, const A: usize>
    ConverterRgb565ToRgba32<R, G, B, A>
{
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 4).step_by(4) {
            let rgb = unsafe { *(&src[j] as *const u8 as *const u16) };
            dst[i + R] = ((rgb >> 8) & 0xF8) as u8;
            dst[i + G] = ((rgb >> 3) & 0xFC) as u8;
            dst[i + B] = ((rgb << 3) & 0xF8) as u8;
            dst[i + A] = 255;

            j += 2;
        }
    }
}

pub type Rgb565ToArgb32 = ConverterRgb565ToRgba32<1, 2, 3, 0>; //----color_conv_rgb565_to_argb32
pub type Rgb565ToAbgr32 = ConverterRgb565ToRgba32<3, 2, 1, 0>; //----color_conv_rgb565_to_abgr32
pub type Rgb565ToBgra32 = ConverterRgb565ToRgba32<2, 1, 0, 3>; //----color_conv_rgb565_to_bgra32
pub type Rgb565ToRgba32 = ConverterRgb565ToRgba32<0, 1, 2, 3>; //----color_conv_rgb565_to_rgba32

//------------------------------------------------color_conv_rgba32_rgb565
pub struct ConverterRgba32ToRgb565<const R: usize, const G: usize, const B: usize>;
impl<const R: usize, const G: usize, const B: usize> ConverterRgba32ToRgb565<R, G, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        unsafe {
            for i in (0..width as usize * 2).step_by(2) {
                *(&mut dst[i] as *mut u8 as *mut u16) = ((((src[j + R] as u32) << 8) & 0xF800)
                    | (((src[j + G] as u32) << 3) & 0x7E0)
                    | ((src[j + B] as u32) >> 3))
                    as u16;

                j += 4;
            }
        }
    }
}

pub type Argb32ToRgb565 = ConverterRgba32ToRgb565<1, 2, 3>; //----color_conv_argb32_to_rgb565
pub type Abgr32ToRgb565 = ConverterRgba32ToRgb565<3, 2, 1>; //----color_conv_abgr32_to_rgb565
pub type Bgra32ToRgb565 = ConverterRgba32ToRgb565<2, 1, 0>; //----color_conv_bgra32_to_rgb565
pub type Rgba32ToRgb565 = ConverterRgba32ToRgb565<0, 1, 2>; //----color_conv_rgba32_to_rgb565

//---------------------------------------------color_conv_rgb555_to_rgb565
pub struct ConverterRgb555ToRgb565;
impl ConverterRgb555ToRgb565 {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        unsafe {
            for i in (0..width as usize * 2).step_by(2) {
                let rgb = *(&src[j] as *const u8 as *const u16);
                *(&mut dst[i] as *mut u8 as *mut u16) =
                    (((rgb << 1) & 0xFFC0) | (rgb & 0x1F)) as u16;

                j += 2;
            }
        }
    }
}
pub type Rgb555ToRgb565 = ConverterRgb555ToRgb565;
//----------------------------------------------color_conv_rgb565_to_rgb555
pub struct ConverterRgb565ToRgb555;
impl ConverterRgb565ToRgb555 {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        unsafe {
            for i in (0..width as usize * 2).step_by(2) {
                let rgb = *(&src[j] as *const u8 as *const u16);
                *(&mut dst[i] as *mut u8 as *mut u16) =
                    (((rgb >> 1) & 0x7FE0) | (rgb & 0x1F)) as u16;

                j += 2;
            }
        }
    }
}
pub type Rgb565ToRgb555 = ConverterRgb565ToRgb555;
//----------------------------------------------color_conv_rgb24_gray8
pub struct ConverterRgb24ToGray8<const R: usize, const B: usize>;
impl<const R: usize, const B: usize> ConverterRgb24ToGray8<R, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in 0..width as usize {
            dst[i] = ((src[j + R] as u32 * 77 + src[j + 1] as u32 * 150 + src[j + B] as u32 * 29)
                >> 8) as u8;
            j += 3;
        }
    }
}

pub type Rgb24ToGray8 = ConverterRgb24ToGray8<0, 2>; //----color_conv_rgb24_to_gray8
pub type Bgr24ToGray8 = ConverterRgb24ToGray8<2, 0>; //----color_conv_bgr24_to_gray8
