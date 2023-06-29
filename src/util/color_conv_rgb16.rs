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
// This part of the library has been sponsored by
// Liberty Technology Systems, Inc., visit http://lib-sys.com
//
// Liberty Technology Systems, Inc. is the provider of
// PostScript and PDF technology for software developers.
//
//----------------------------------------------------------------------------
//
// A set of functors used with color_conv(). See file agg_color_conv.h
// These functors can convert images with up to 8 bits per component.
// Use convertors in the following way:
//
// agg::color_conv(dst, src, agg::color_conv_XXXX_to_YYYY());
//----------------------------------------------------------------------------

#![allow(non_snake_case)]

//----------------------------------------------color_conv_gray16_to_gray8
pub struct ConverterGray16ToGray8;
impl ConverterGray16ToGray8 {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 1).step_by(1) {
            let s = unsafe { *(&src[j] as *const u8 as *const u16) };
            dst[i] = (s >> 8) as u8;

            j += 2;
        }
    }
}

pub type Gray16ToGray8 = ConverterGray16ToGray8;
//-----------------------------------------------------color_conv_rgb24_rgb48
pub struct ConverterRgb24ToRgb48<const I1: usize, const I3: usize>;
impl<const I1: usize, const I3: usize> ConverterRgb24ToRgb48<I1, I3> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        unsafe {
            let mut j = 0;
            for i in (0..width as usize * 6).step_by(6) {
                let d = &mut dst[i] as *mut u8 as *mut u16;
                *d.offset(0) = ((src[j + I1] as u16) << 8) | src[j + I1] as u16;
                *d.offset(1) = ((src[j + 1] as u16) << 8) | src[j + 1] as u16;
                *d.offset(2) = ((src[j + I3] as u16) << 8) | src[j + I3] as u16;

                j += 3;
            }
        }
    }
}

pub type Rgb24ToRgb48 = ConverterRgb24ToRgb48<0, 2>;
pub type Bgr24ToBgr48 = ConverterRgb24ToRgb48<0, 2>;
pub type Rgb24ToBgr48 = ConverterRgb24ToRgb48<2, 0>;
pub type Bgr24ToRgb48 = ConverterRgb24ToRgb48<2, 0>;

//-----------------------------------------------------color_conv_rgb48_rgb24
pub struct ConverterRgb48ToRgb24<const I1: usize, const I3: usize>;
impl<const I1: usize, const I3: usize> ConverterRgb48ToRgb24<I1, I3> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        unsafe {
            let mut j = 0;
            for i in (0..width as usize * 3).step_by(3) {
                let s = &src[j] as *const u8 as *const u16;
                dst[i + 0] = (*s.offset(I1 as isize) >> 8) as u8;
                dst[i + 1] = (*s.offset(1) >> 8) as u8;
                dst[i + 2] = (*s.offset(I3 as isize) >> 8) as u8;
                j += 6;
            }
        }
    }
}

pub type Rgb48ToRgb24 = ConverterRgb48ToRgb24<0, 2>;
pub type Bgr48ToBgr24 = ConverterRgb48ToRgb24<0, 2>;
pub type Rgb48ToBgr24 = ConverterRgb48ToRgb24<2, 0>;
pub type Bgr48ToRgb24 = ConverterRgb48ToRgb24<2, 0>;

//-----------------------------------------------------color_conv_rgbAAA_rgb24
pub struct ConverterRgbAAAToRgb24<const R: usize, const B: usize>;
impl<const R: usize, const B: usize> ConverterRgbAAAToRgb24<R, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 3).step_by(3) {
            let rgb = unsafe { *(&src[j] as *const u8 as *const u32) };
            dst[i + R] = (rgb >> 22) as u8;
            dst[i + 1] = (rgb >> 12) as u8;
            dst[i + B] = (rgb >> 2) as u8;
            j += 4;
        }
    }
}

pub type RgbAAAToRgb24 = ConverterRgbAAAToRgb24<0, 2>;
pub type RgbAAAToBgr24 = ConverterRgbAAAToRgb24<2, 0>;
pub type BgrAAAToRgb24 = ConverterRgbAAAToRgb24<2, 0>;
pub type BgrAAAToBgr24 = ConverterRgbAAAToRgb24<0, 2>;

//-----------------------------------------------------color_conv_rgbBBA_rgb24
pub struct ConverterRgbBBAToRgb24<const R: usize, const B: usize>;
impl<const R: usize, const B: usize> ConverterRgbBBAToRgb24<R, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 3).step_by(3) {
            let rgb = unsafe { *(&src[j] as *const u8 as *const u32) };
            dst[i + R] = (rgb >> 24) as u8;
            dst[i + 1] = (rgb >> 13) as u8;
            dst[i + B] = (rgb >> 2) as u8;
            j += 4;
        }
    }
}

pub type RgbBBAToRgb24 = ConverterRgbBBAToRgb24<0, 2>;
pub type RgbBBAToBgr24 = ConverterRgbBBAToRgb24<2, 0>;

//-----------------------------------------------------color_conv_bgrABB_rgb24
pub struct ConverterBgrABBToRgb24<const B: usize, const R: usize>;
impl<const B: usize, const R: usize> ConverterBgrABBToRgb24<B, R> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 3).step_by(3) {
            let bgr = unsafe { *(&src[j] as *const u8 as *const u32) };
            dst[i + R] = (bgr >> 3) as u8;
            dst[i + 1] = (bgr >> 14) as u8;
            dst[i + B] = (bgr >> 24) as u8;
            j += 4;
        }
    }
}

pub type BgrABBToRgb24 = ConverterBgrABBToRgb24<2, 0>;
pub type BgrABBToBgr24 = ConverterBgrABBToRgb24<0, 2>;

//-----------------------------------------------------color_conv_rgba64_rgba32
pub struct ConverterRgba64ToRgba32<
    const I1: usize,
    const I2: usize,
    const I3: usize,
    const I4: usize,
>;
impl<const I1: usize, const I2: usize, const I3: usize, const I4: usize>
    ConverterRgba64ToRgba32<I1, I2, I3, I4>
{
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        unsafe {
            let mut j = 0;
            for i in (0..width as usize * 4).step_by(4) {
                let s = &src[j] as *const u8 as *const u16;
                dst[i + 0] = (*s.offset(I1 as isize) >> 8) as u8;
                dst[i + 1] = (*s.offset(I2 as isize) >> 8) as u8;
                dst[i + 2] = (*s.offset(I3 as isize) >> 8) as u8;
                dst[i + 3] = (*s.offset(I4 as isize) >> 8) as u8;
                j += 8;
            }
        }
    }
}

pub type Rgba64ToRgba32 = ConverterRgba64ToRgba32<0, 1, 2, 3>; //----color_conv_rgba64_to_rgba32
pub type Argb64ToArgb32 = ConverterRgba64ToRgba32<0, 1, 2, 3>; //----color_conv_argb64_to_argb32
pub type Bgra64ToBgra32 = ConverterRgba64ToRgba32<0, 1, 2, 3>; //----color_conv_bgra64_to_bgra32
pub type Abgr64ToAbgr32 = ConverterRgba64ToRgba32<0, 1, 2, 3>; //----color_conv_abgr64_to_abgr32
pub type Argb64ToAbgr32 = ConverterRgba64ToRgba32<0, 3, 2, 1>; //----color_conv_argb64_to_abgr32
pub type Argb64ToBgra32 = ConverterRgba64ToRgba32<3, 2, 1, 0>; //----color_conv_argb64_to_bgra32
pub type Argb64ToRgba32 = ConverterRgba64ToRgba32<1, 2, 3, 0>; //----color_conv_argb64_to_rgba32
pub type Bgra64ToAbgr32 = ConverterRgba64ToRgba32<3, 0, 1, 2>; //----color_conv_bgra64_to_abgr32
pub type Bgra64ToArgb32 = ConverterRgba64ToRgba32<3, 2, 1, 0>; //----color_conv_bgra64_to_argb32
pub type Bgra64ToRgba32 = ConverterRgba64ToRgba32<2, 1, 0, 3>; //----color_conv_bgra64_to_rgba32
pub type Rgba64ToAbgr32 = ConverterRgba64ToRgba32<3, 2, 1, 0>; //----color_conv_rgba64_to_abgr32
pub type Rgba64ToArgb32 = ConverterRgba64ToRgba32<3, 0, 1, 2>; //----color_conv_rgba64_to_argb32
pub type Rgba64ToBgra32 = ConverterRgba64ToRgba32<2, 1, 0, 3>; //----color_conv_rgba64_to_bgra32
pub type Abgr64ToArgb32 = ConverterRgba64ToRgba32<0, 3, 2, 1>; //----color_conv_abgr64_to_argb32
pub type Abgr64ToBgra32 = ConverterRgba64ToRgba32<1, 2, 3, 0>; //----color_conv_abgr64_to_bgra32
pub type Abgr64ToRgba32 = ConverterRgba64ToRgba32<3, 2, 1, 0>; //----color_conv_abgr64_to_rgba32

//-----------------------------------------------------color_conv_rgb24_rgba64
pub struct ConverterRgb24ToRgba64<const I1: usize, const I2: usize, const I3: usize, const A: usize>;
impl<const I1: usize, const I2: usize, const I3: usize, const A: usize>
    ConverterRgb24ToRgba64<I1, I2, I3, A>
{
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        unsafe {
            let mut j = 0;
            for i in (0..width as usize * 8).step_by(8) {
                let d = &mut dst[i] as *mut u8 as *mut u16;
                *d.offset(I1 as isize) = ((src[j + 0] as u16) << 8) | src[j + 0] as u16;
                *d.offset(I2 as isize) = ((src[j + 1] as u16) << 8) as u16 | src[j + 1] as u16;
                *d.offset(I3 as isize) = ((src[j + 2] as u16) << 8) as u16 | src[j + 2] as u16;
                *d.offset(A as isize) = 65535;

                j += 3;
            }
        }
    }
}

pub type Rgb24ToArgb64 = ConverterRgb24ToRgba64<1, 2, 3, 0>; //----color_conv_rgb24_to_argb64
pub type Rgb24ToAbgr64 = ConverterRgb24ToRgba64<3, 2, 1, 0>; //----color_conv_rgb24_to_abgr64
pub type Rgb24ToBgra64 = ConverterRgb24ToRgba64<2, 1, 0, 3>; //----color_conv_rgb24_to_bgra64
pub type Rgb24ToRgba64 = ConverterRgb24ToRgba64<0, 1, 2, 3>; //----color_conv_rgb24_to_rgba64
pub type Bgr24ToArgb64 = ConverterRgb24ToRgba64<3, 2, 1, 0>; //----color_conv_bgr24_to_argb64
pub type Bgr24ToAbgr64 = ConverterRgb24ToRgba64<1, 2, 3, 0>; //----color_conv_bgr24_to_abgr64
pub type Bgr24ToBgra64 = ConverterRgb24ToRgba64<0, 1, 2, 3>; //----color_conv_bgr24_to_bgra64
pub type Bgr24ToRgba64 = ConverterRgb24ToRgba64<2, 1, 0, 3>; //----color_conv_bgr24_to_rgba64

//----------------------------------------------color_conv_rgb24_gray16
pub struct ConverterRgb24ToGray16<const R: usize, const B: usize>;
impl<const R: usize, const B: usize> ConverterRgb24ToGray16<R, B> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        let mut j = 0;
        for i in (0..width as usize * 2).step_by(2) {
            unsafe {
                *(&mut dst[i] as *mut u8 as *mut u16) =
                    (src[j + R] * 77 + src[j + 1] * 150 + src[j + B] * 29) as u16;
            }

            j += 3;
        }
    }
}

pub type Rgb24ToGray16 = ConverterRgb24ToGray16<0, 2>;
pub type Bgr24ToGray16 = ConverterRgb24ToGray16<2, 0>;
