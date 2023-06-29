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

use crate::basics::{iround, PolySubpixelScale, RectBase, Saturation};
use crate::clip_liang_barsky::*;
use crate::{AggPrimitive, RasClip, RasConv, Rasterizer};

pub type RasterizerSlClipInt = RasterizerSlClip<RasConvInt>;
pub type RasterizerSlClipIntSat = RasterizerSlClip<RasConvIntSat>;
pub type RasterizerSlClipInt3x = RasterizerSlClip<RasConvInt3x>;
pub type RasterizerSlClipDbl = RasterizerSlClip<RasConvDbl>;
pub type RasterizerSlClipDbl3x = RasterizerSlClip<RasConvDbl3x>;

const POLY_MAX_COORD: i32 = (1 << 30) - 1;

//------------------------------------------------------------RasConvInt
pub struct RasConvInt;
impl RasConv for RasConvInt {
    type CoordType = i32;

    fn mul_div(a: f64, b: f64, c: f64) -> Self::CoordType {
        iround(a * b / c)
    }
    fn xi(v: i32) -> i32 {
        v
    }
    fn yi(v: i32) -> i32 {
        v
    }
    fn upscale(v: f64) -> i32 {
        iround(v * PolySubpixelScale::Scale as i64 as f64)
    }
    fn downscale(v: i32) -> i32 {
        v
    }
}

//--------------------------------------------------------RasConvIntSat
pub struct RasConvIntSat;
impl RasConv for RasConvIntSat {
    type CoordType = i32;

    fn mul_div(a: f64, b: f64, c: f64) -> Self::CoordType {
        Saturation::<POLY_MAX_COORD>::iround(a * b / c)
    }
    fn xi(v: i32) -> i32 {
        v
    }
    fn yi(v: i32) -> i32 {
        v
    }
    fn upscale(v: f64) -> i32 {
        Saturation::<POLY_MAX_COORD>::iround(v * PolySubpixelScale::Scale as i32 as f64)
    }
    fn downscale(v: i32) -> i32 {
        v
    }
}

//---------------------------------------------------------RasConvInt3x
pub struct RasConvInt3x;
impl RasConv for RasConvInt3x {
    type CoordType = i32;

    fn mul_div(a: f64, b: f64, c: f64) -> Self::CoordType {
        return iround(a * b / c);
    }
    fn xi(v: i32) -> i32 {
        return v * 3;
    }
    fn yi(v: i32) -> i32 {
        return v;
    }
    fn upscale(v: f64) -> i32 {
        return iround(v * PolySubpixelScale::Scale as i32 as f64);
    }
    fn downscale(v: i32) -> i32 {
        return v;
    }
}

//-----------------------------------------------------------RasConvDbl
pub struct RasConvDbl;
impl RasConv for RasConvDbl {
    type CoordType = f64;

    fn mul_div(a: f64, b: f64, c: f64) -> Self::CoordType {
        a * b / c
    }
    fn xi(v: f64) -> i32 {
        iround(v * PolySubpixelScale::Scale as i32 as f64)
    }
    fn yi(v: f64) -> i32 {
        iround(v * PolySubpixelScale::Scale as i32 as f64)
    }
    fn upscale(v: f64) -> f64 {
        v
    }
    fn downscale(v: i32) -> f64 {
        v as f64 / PolySubpixelScale::Scale as i32 as f64
    }
}

//--------------------------------------------------------RasConvDbl3x
pub struct RasConvDbl3x;
impl RasConv for RasConvDbl3x {
    type CoordType = f64;

    fn mul_div(a: f64, b: f64, c: f64) -> Self::CoordType {
        a * b / c
    }
    fn xi(v: f64) -> i32 {
        iround(v * PolySubpixelScale::Scale as i32 as f64 * 3.0)
    }
    fn yi(v: f64) -> i32 {
        iround(v * PolySubpixelScale::Scale as i32 as f64)
    }
    fn upscale(v: f64) -> f64 {
        v
    }
    fn downscale(v: i32) -> f64 {
        v as f64 / PolySubpixelScale::Scale as i32 as f64
    }
}

pub struct RasterizerSlClip<Conv: RasConv> {
    m_clip_box: RectBase<Conv::CoordType>,
    m_x1: Conv::CoordType,
    m_y1: Conv::CoordType,
    m_f1: u32,
    m_clipping: bool,
}

impl<Conv: RasConv> RasClip for RasterizerSlClip<Conv> {
    type CoordType = Conv::CoordType;
    type ConvType = Conv;

    fn new() -> Self {
        let z = Conv::CoordType::from_i32(0);
        RasterizerSlClip {
            m_clip_box: RectBase::<Conv::CoordType>::new(z, z, z, z),
            m_x1: z,
            m_y1: z,
            m_f1: 0,
            m_clipping: false,
        }
    }

    fn reset_clipping(&mut self) {
        self.m_clipping = false;
    }

    fn clip_box(
        &mut self, x1: Conv::CoordType, y1: Conv::CoordType, x2: Conv::CoordType,
        y2: Conv::CoordType,
    ) {
        self.m_clip_box = RectBase::<Conv::CoordType>::new(x1, y1, x2, y2);
        self.m_clip_box.normalize();
        self.m_clipping = true;
    }

    fn move_to(&mut self, x1: Self::CoordType, y1: Self::CoordType) {
        self.m_x1 = x1;
        self.m_y1 = y1;
        if self.m_clipping {
            self.m_f1 = clipping_flags(self.m_x1, self.m_y1, &self.m_clip_box);
        }
    }

    fn line_to<R: Rasterizer>(&mut self, ras: &mut R, x2: Conv::CoordType, y2: Conv::CoordType) {
        if self.m_clipping {
            let f2 = clipping_flags(x2, y2, &self.m_clip_box);

            if (self.m_f1 & 10) == (f2 & 10) && (self.m_f1 & 10) != 0 {
                // Invisible by Y
                self.m_x1 = x2;
                self.m_y1 = y2;
                self.m_f1 = f2;
                return;
            }
            let x1 = self.m_x1;
            let y1 = self.m_y1;
            let f1 = self.m_f1;
            let y3;
            let y4;
            let f3;
            let f4;

            match ((f1 & 5) << 1) | (f2 & 5) {
                0 => {
                    // Visible by X
                    self.line_clip_y(ras, x1, y1, x2, y2, f1, f2);
                }
                1 => {
                    y3 = y1
                        + Conv::mul_div(
                            (self.m_clip_box.x2 - x1).into_f64(),
                            (y2 - y1).into_f64(),
                            (x2 - x1).into_f64(),
                        );
                    f3 = clipping_flags_y(y3, &self.m_clip_box);
                    self.line_clip_y(ras, x1, y1, self.m_clip_box.x2, y3, f1, f3);
                    self.line_clip_y(ras, self.m_clip_box.x2, y3, self.m_clip_box.x2, y2, f3, f2);
                }
                2 => {
                    let y3 = y1
                        + Conv::mul_div(
                            (self.m_clip_box.x2 - x1).into_f64(),
                            (y2 - y1).into_f64(),
                            (x2 - x1).into_f64(),
                        );
                    let f3 = clipping_flags_y(y3, &self.m_clip_box);
                    self.line_clip_y(ras, self.m_clip_box.x2, y1, self.m_clip_box.x2, y3, f1, f3);
                    self.line_clip_y(ras, self.m_clip_box.x2, y3, x2, y2, f3, f2);
                }
                3 => {
                    self.line_clip_y(ras, self.m_clip_box.x2, y1, self.m_clip_box.x2, y2, f1, f2);
                }
                4 => {
                    // x2 < clip.x1
                    y3 = y1
                        + Conv::mul_div(
                            (self.m_clip_box.x1 - x1).into_f64(),
                            (y2 - y1).into_f64(),
                            (x2 - x1).into_f64(),
                        );
                    f3 = clipping_flags_y(y3, &self.m_clip_box);
                    self.line_clip_y(ras, x1, y1, self.m_clip_box.x1, y3, f1, f3);
                    self.line_clip_y(ras, self.m_clip_box.x1, y3, self.m_clip_box.x1, y2, f3, f2);
                }

                6 => {
                    // x1 > clip.x2 && x2 < clip.x1
                    y3 = y1
                        + Conv::mul_div(
                            (self.m_clip_box.x2 - x1).into_f64(),
                            (y2 - y1).into_f64(),
                            (x2 - x1).into_f64(),
                        );
                    y4 = y1
                        + Conv::mul_div(
                            (self.m_clip_box.x1 - x1).into_f64(),
                            (y2 - y1).into_f64(),
                            (x2 - x1).into_f64(),
                        );
                    f3 = clipping_flags_y(y3, &self.m_clip_box);
                    f4 = clipping_flags_y(y4, &self.m_clip_box);
                    self.line_clip_y(ras, self.m_clip_box.x2, y1, self.m_clip_box.x2, y3, f1, f3);
                    self.line_clip_y(ras, self.m_clip_box.x2, y3, self.m_clip_box.x1, y4, f3, f4);
                    self.line_clip_y(ras, self.m_clip_box.x1, y4, self.m_clip_box.x1, y2, f4, f2);
                }

                8 => {
                    // x1 < clip.x1
                    y3 = y1
                        + Conv::mul_div(
                            (self.m_clip_box.x1 - x1).into_f64(),
                            (y2 - y1).into_f64(),
                            (x2 - x1).into_f64(),
                        );
                    f3 = clipping_flags_y(y3, &self.m_clip_box);
                    self.line_clip_y(ras, self.m_clip_box.x1, y1, self.m_clip_box.x1, y3, f1, f3);
                    self.line_clip_y(ras, self.m_clip_box.x1, y3, x2, y2, f3, f2);
                }

                9 => {
                    // x1 < clip.x1 && x2 > clip.x2
                    y3 = y1
                        + Conv::mul_div(
                            (self.m_clip_box.x1 - x1).into_f64(),
                            (y2 - y1).into_f64(),
                            (x2 - x1).into_f64(),
                        );
                    y4 = y1
                        + Conv::mul_div(
                            (self.m_clip_box.x2 - x1).into_f64(),
                            (y2 - y1).into_f64(),
                            (x2 - x1).into_f64(),
                        );
                    f3 = clipping_flags_y(y3, &self.m_clip_box);
                    f4 = clipping_flags_y(y4, &self.m_clip_box);
                    self.line_clip_y(ras, self.m_clip_box.x1, y1, self.m_clip_box.x1, y3, f1, f3);
                    self.line_clip_y(ras, self.m_clip_box.x1, y3, self.m_clip_box.x2, y4, f3, f4);
                    self.line_clip_y(ras, self.m_clip_box.x2, y4, self.m_clip_box.x2, y2, f4, f2);
                }

                12 => {
                    // x1 < clip.x1 && x2 < clip.x1
                    self.line_clip_y(ras, self.m_clip_box.x1, y1, self.m_clip_box.x1, y2, f1, f2);
                }
                _ => todo!(),
            }
            self.m_f1 = f2;
        } else {
            ras.line(
                Conv::xi(self.m_x1),
                Conv::yi(self.m_y1),
                Conv::xi(x2),
                Conv::yi(y2),
            );
        }
        self.m_x1 = x2;
        self.m_y1 = y2;
    }
}

impl<Conv: RasConv> RasterizerSlClip<Conv> {
    #[inline]
    fn line_clip_y<R: Rasterizer>(
        &self, ras: &mut R, x1: <Self as RasClip>::CoordType, y1: <Self as RasClip>::CoordType,
        x2: <Self as RasClip>::CoordType, y2: <Self as RasClip>::CoordType, f1: u32, f2: u32,
    ) {
        let f1 = f1 & 10;
        let f2 = f2 & 10;
        if f1 | f2 == 0 {
            // Fully visible
            ras.line(Conv::xi(x1), Conv::yi(y1), Conv::xi(x2), Conv::yi(y2));
        } else {
            if f1 == f2 {
                // Invisible by Y
                return;
            }
            let mut tx1 = x1;
            let mut ty1 = y1;
            let mut tx2 = x2;
            let mut ty2 = y2;

            if f1 & 8 != 0 {
                tx1 = x1
                    + Conv::mul_div(
                        (self.m_clip_box.y1 - y1).into_f64(),
                        (x2 - x1).into_f64(),
                        (y2 - y1).into_f64(),
                    );
                ty1 = self.m_clip_box.y1;
            }
            if f1 & 2 != 0
            // y1 > clip.y2
            {
                tx1 = x1
                    + Conv::mul_div(
                        (self.m_clip_box.y2 - y1).into_f64(),
                        (x2 - x1).into_f64(),
                        (y2 - y1).into_f64(),
                    );
                ty1 = self.m_clip_box.y2;
            }

            if f2 & 8 != 0
            // y2 < clip.y1
            {
                tx2 = x1
                    + Conv::mul_div(
                        (self.m_clip_box.y1 - y1).into_f64(),
                        (x2 - x1).into_f64(),
                        (y2 - y1).into_f64(),
                    );
                ty2 = self.m_clip_box.y1;
            }

            if f2 & 2 != 0
            // y2 > clip.y2
            {
                tx2 = x1
                    + Conv::mul_div(
                        (self.m_clip_box.y2 - y1).into_f64(),
                        (x2 - x1).into_f64(),
                        (y2 - y1).into_f64(),
                    );
                ty2 = self.m_clip_box.y2;
            }
            ras.line(Conv::xi(tx1), Conv::yi(ty1), Conv::xi(tx2), Conv::yi(ty2));
        }
    }
}

// RasterizerSlNoClip
pub struct RasterizerSlNoClip {
    x1: i32,
    y1: i32,
}

impl RasClip for RasterizerSlNoClip {
    type CoordType = i32;
    type ConvType = RasConvInt;

    fn new() -> Self {
        RasterizerSlNoClip { x1: 0, y1: 0 }
    }

    fn reset_clipping(&mut self) {}

    fn clip_box(&mut self, _x1: i32, _y1: i32, _x2: i32, _y2: i32) {}

    fn move_to(&mut self, x1: i32, y1: i32) {
        self.x1 = x1;
        self.y1 = y1;
    }

    fn line_to<R: Rasterizer>(&mut self, ras: &mut R, x2: i32, y2: i32) {
        ras.line(self.x1, self.y1, x2, y2);
        self.x1 = x2;
        self.y1 = y2;
    }
}
