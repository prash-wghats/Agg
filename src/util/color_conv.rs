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
// Conversion from one colorspace/pixel format to another
//
//----
use crate::{util::CopyRowFn, RenderBuffer};

//--------------------------------------------------------------color_conv
pub fn color_conv<RenBuf: RenderBuffer<T = u8>>(
    dst: &mut RenBuf, src: &RenBuf, copy_row_functor: CopyRowFn,
) {
    let mut width = src.width();
    let mut height = src.height();

    if dst.width() < width {
        width = dst.width();
    }
    if dst.height() < height {
        height = dst.height();
    }

    if width > 0 {
        for y in 0..height as i32 {
            copy_row_functor(dst.row_mut(y), src.row(y), width);
        }
    }
}

//---------------------------------------------------------color_conv_row
pub fn color_conv_row(dst: &mut [u8], src: &[u8], width: u32, copy_row_functor: CopyRowFn) {
    copy_row_functor(dst, src, width);
}

//---------------------------------------------------------color_conv_same
pub struct ConverterSame<const BPP: u32>;
impl<const BPP: u32> ConverterSame<BPP> {
    pub fn convert(dst: &mut [u8], src: &[u8], width: u32) {
        unsafe {
            std::ptr::copy(src.as_ptr(), dst.as_mut_ptr(), (width * BPP) as usize);
        }
    }
}

pub type Bgr24ToBgr24 = ConverterSame<3> ;
pub type Rgb24ToRgb24 = ConverterSame<3> ;

//------------------------------------------------------------------------
pub type Rgba32ToRgba32 = ConverterSame<4> ; //----color_conv_rgba32_to_rgba32
pub type Argb32ToArgb32 = ConverterSame<4> ; //----color_conv_argb32_to_argb32
pub type Bgra32ToBgra32 = ConverterSame<4> ; //----color_conv_bgra32_to_bgra32
pub type Abgr32ToAbgr32 = ConverterSame<4> ; //----color_conv_abgr32_to_abgr32

//----------------------------------------------color_conv_rgb565_to_rgb555
//------------------------------------------------------------------------
pub type Rgb555ToRgb555 = ConverterSame<2> ; //----color_conv_rgb555_to_rgb555
pub type Rgb565ToRgb565 = ConverterSame<2> ; //----color_conv_rgb565_to_rgb565
