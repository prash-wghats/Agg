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
// BoundingRect function template
//
//----------------------------------------------------------------------------

use crate::basics::{is_stop, is_vertex};
use crate::AggPrimitive;
use crate::VertexSource;

//-----------------------------------------------------------BoundingRect
pub fn bounding_rect<VS, CoordT, Idx: std::ops::Index<usize, Output = u32>>(
    vs: &mut VS, gi: Idx, start: u32, num: u32, x1: &mut CoordT, y1: &mut CoordT,
    x2: &mut CoordT, y2: &mut CoordT,
) -> bool
where
    VS: VertexSource,
    CoordT: AggPrimitive,
{
    let mut i: u32 = 0;
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut first: bool = true;

    *x1 = CoordT::from_u32(1);
    *y1 = CoordT::from_u32(1);
    *x2 = CoordT::from_u32(0);
    *y2 = CoordT::from_u32(0);

    while i < num {
        vs.rewind(gi[(start + i) as usize]);
        let mut cmd: u32;
        loop {
            cmd = vs.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if is_vertex(cmd) {
                if first {
                    *x1 = CoordT::from_f64(x);
                    *y1 = CoordT::from_f64(y);
                    *x2 = CoordT::from_f64(x);
                    *y2 = CoordT::from_f64(y);
                    first = false;
                } else {
                    if CoordT::from_f64(x) < *x1 {
                        *x1 = CoordT::from_f64(x);
                    }
                    if CoordT::from_f64(y) < *y1 {
                        *y1 = CoordT::from_f64(y);
                    }
                    if CoordT::from_f64(x) > *x2 {
                        *x2 = CoordT::from_f64(x);
                    }
                    if CoordT::from_f64(y) > *y2 {
                        *y2 = CoordT::from_f64(y);
                    }
                }
            }
        }
        i += 1;
    }
    *x1 <= *x2 && *y1 <= *y2
}

//-----------------------------------------------------bounding_rect_single
pub fn bounding_rect_single<VS, CoordT>(
    vs: &mut VS, path_id: u32, x1: &mut CoordT, y1: &mut CoordT, x2: &mut CoordT, y2: &mut CoordT,
) -> bool
where
    VS: VertexSource,
    CoordT: AggPrimitive,
{
    let mut x: f64 = 0.;
    let mut y: f64 = 0.;
    let mut first = true;

    *x1 = CoordT::from_u32(1);
    *y1 = CoordT::from_u32(1);
    *x2 = CoordT::from_u32(0);
    *y2 = CoordT::from_u32(0);

    vs.rewind(path_id);
    let mut cmd: u32;

    loop {
        cmd = vs.vertex(&mut x, &mut y);
        if is_stop(cmd) {
            break;
        }
        if is_vertex(cmd) {
            if first {
                *x1 = CoordT::from_f64(x);
                *y1 = CoordT::from_f64(y);
                *x2 = CoordT::from_f64(x);
                *y2 = CoordT::from_f64(y);
                first = false;
            } else {
                if CoordT::from_f64(x) < *x1 {
                    *x1 = CoordT::from_f64(x);
                }
                if CoordT::from_f64(y) < *y1 {
                    *y1 = CoordT::from_f64(y);
                }
                if CoordT::from_f64(x) > *x2 {
                    *x2 = CoordT::from_f64(x);
                }
                if CoordT::from_f64(y) > *y2 {
                    *y2 = CoordT::from_f64(y);
                }
            }
        }
    }
    *x1 <= *x2 && *y1 <= *y2
}
