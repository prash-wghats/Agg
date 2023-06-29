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
// Liang-Barsky clipping
//
//----------------------------------------------------------------------------

use std::ops::Neg;

use crate::basics::RectBase;
use crate::AggPrimitive;

//------------------------------------------------------------------------
// enum clipping_flags_e
// {
pub const CF_X1_CLIPPED: u32 = 4;
pub const CF_X2_CLIPPED: u32 = 1;
pub const CF_Y1_CLIPPED: u32 = 8;
pub const CF_Y2_CLIPPED: u32 = 2;
pub const CF_X_CLIPPED: u32 = CF_X1_CLIPPED | CF_X2_CLIPPED;
pub const CF_Y_CLIPPED: u32 = CF_Y1_CLIPPED | CF_Y2_CLIPPED;
//}

//----------------------------------------------------------clipping_flags
// Determine the clipping code of the vertex according to the
// Cyrus-Beck line clipping algorithm
//
//        |        |
//  0110  |  0010  | 0011
//        |        |
// -------+--------+-------- clip_box.y2
//        |        |
//  0100  |  0000  | 0001
//        |        |
// -------+--------+-------- clip_box.y1
//        |        |
//  1100  |  1000  | 1001
//        |        |
//  clip_box.x1  clip_box.x2
//
//

pub fn clipping_flags<T: Copy + PartialOrd>(x: T, y: T, clip_box: &RectBase<T>) -> u32 {
    (x > clip_box.x2) as u32
        | ((y > clip_box.y2) as u32) << 1
        | ((x < clip_box.x1) as u32) << 2
        | ((y < clip_box.y1) as u32) << 3
}

pub fn clipping_flags_x<T: Copy + PartialOrd>(x: T, clip_box: &RectBase<T>) -> u32 {
    (x > clip_box.x2) as u32 | ((x < clip_box.x1) as u32) << 2
}

pub fn clipping_flags_y<T: Copy + PartialOrd>(y: T, clip_box: &RectBase<T>) -> u32 {
    ((y > clip_box.y2) as u32) << 1 | ((y < clip_box.y1) as u32) << 3
}

#[inline]
pub fn clip_line_segment<T: AggPrimitive>(
    x1: &mut T, y1: &mut T, x2: &mut T, y2: &mut T, clip_box: &RectBase<T>,
) -> u8 {
    let f1 = clipping_flags(*x1, *y1, clip_box);
    let f2 = clipping_flags(*x2, *y2, clip_box);
    let mut ret = 0;

    if (f2 | f1) == 0 {
        // Fully visible
        return 0;
    }

    if (f1 & CF_X_CLIPPED) != 0 && (f1 & CF_X_CLIPPED) == (f2 & CF_X_CLIPPED) {
        // Fully clipped
        return 4;
    }
    let tx1 = *x1;
    let ty1 = *y1;
    let tx2 = *x2;
    let ty2 = *y2;
    if f1 != 0 {
        if !clip_move_point(tx1, ty1, tx2, ty2, clip_box, x1, y1, f1) {
            return 4;
        }
        if *x1 == *x2 && *y1 == *y2 {
            return 4;
        }
        ret |= 1;
    }
    if f2 != 0 {
        if !clip_move_point(tx1, ty1, tx2, ty2, clip_box, x2, y2, f2) {
            return 4;
        }
        if *x1 == *x2 && *y1 == *y2 {
            return 4;
        }
        ret |= 2;
    }
    return ret;
}

pub fn clip_move_point<T: AggPrimitive>(
    x1: T, y1: T, x2: T, y2: T, clip_box: &RectBase<T>, x: &mut T, y: &mut T, flags: u32,
) -> bool {
    let mut bound: T;
    let mut flags = flags;

    if flags & CF_X_CLIPPED != 0 {
        if x1 == x2 {
            return false;
        }
        bound = if flags & CF_X1_CLIPPED != 0 {
            clip_box.x1
        } else {
            clip_box.x2
        };
        *y = AggPrimitive::from_f64(
            (bound - x1).into_f64() * (y2 - y1).into_f64() / (x2 - x1).into_f64() + y1.into_f64(),
        );
        *x = bound;
    }

    flags = clipping_flags_y(*y, clip_box);
    if flags & CF_Y_CLIPPED != 0 {
        if y1 == y2 {
            return false;
        }
        bound = if (flags & CF_Y1_CLIPPED) != 0 {
            clip_box.y1
        } else {
            clip_box.y2
        };
        *x = AggPrimitive::from_f64(
            (bound - y1).into_f64() * (x2 - x1).into_f64() / (y2 - y1).into_f64() + x1.into_f64(),
        );
        *y = bound;
    }
    return true;
}

//-------------------------------------------------------clip_liang_barsky
pub fn clip_liang_barsky<T: AggPrimitive + Neg<Output = T>>(
    x1: T, y1: T, x2: T, y2: T, clip_box: &RectBase<T>, x: &mut [T], y: &mut [T],
) -> usize {
    let nearzero = 1e-30;

    let mut deltax = x2 - x1;
    let mut deltay = y2 - y1;
    let xin;
    let xout;
    let yin;
    let yout;
    let tinx;
    let tiny;
    let toutx;
    let touty;
    let tin1;
    let tin2;
    let tout1;
    let mut np = 0;

    if deltax == T::default() {
        // bump off of the vertical
        deltax = if x1 > clip_box.x1 {
            -(T::from_f64(nearzero))
        } else {
            T::from_f64(nearzero)
        };
    }

    if deltay == T::default() {
        // bump off of the horizontal
        deltay = if y1 > clip_box.y1 {
            -(T::from_f64(nearzero))
        } else {
            T::from_f64(nearzero)
        };
    }

    if deltax > T::default() {
        // points to right
        xin = clip_box.x1;
        xout = clip_box.x2;
    } else {
        xin = clip_box.x2;
        xout = clip_box.x1;
    }

    if deltay > T::default() {
        // points up
        yin = clip_box.y1;
        yout = clip_box.y2;
    } else {
        yin = clip_box.y2;
        yout = clip_box.y1;
    }

    tinx = (xin - x1) / deltax;
    tiny = (yin - y1) / deltay;

    if tinx < tiny {
        // hits x first
        tin1 = tinx;
        tin2 = tiny;
    } else {
        // hits y first
        tin1 = tiny;
        tin2 = tinx;
    }

    if tin1 <= T::from_f64(1.0) {
        if T::default() < tin1 {
            x[np] = xin;
            y[np] = yin;
            np += 1;
        }

        if tin2 <= T::from_f64(1.0) {
            toutx = (xout - x1) / deltax;
            touty = (yout - y1) / deltay;

            tout1 = if toutx < touty { toutx } else { touty };

            if tin2 > T::default() || tout1 > T::default() {
                if tin2 <= tout1 {
                    if tin2 > T::default() {
                        if tinx > tiny {
                            x[np] = xin;
                            y[np] = y1 + tinx * deltay;
                        } else {
                            x[np] = x1 + tiny * deltax;
                            y[np] = yin;
                        }
                        np += 1;
                    }

                    if tout1 < T::from_f64(1.0) {
                        if toutx < touty {
                            x[np] = xout;
                            y[np] = y1 + toutx * deltay;
                        } else {
                            x[np] = x1 + touty * deltax;
                            y[np] = yout;
                        }
                    } else {
                        x[np] = x2;
                        y[np] = y2;
                    }
                    np += 1;
                } else {
                    if tinx > tiny {
                        x[np] = xin;
                        y[np] = yout;
                    } else {
                        x[np] = xout;
                        y[np] = yin;
                    }
                    np += 1;
                }
            }
        }
    }
    np
}
