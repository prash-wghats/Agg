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

use crate::Point;

#[inline]
pub fn iround(v: f64) -> i32 {
    return (if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i32;
}

#[inline]
pub fn uround(v: f64) -> i32 {
    return (v + 0.5) as u32 as i32;
}

#[inline]
pub fn ufloor(v: f64) -> u32 {
    return v as u32;
}

#[inline]
pub fn uceil(v: f64) -> u32 {
    return (v.ceil()) as u32;
}

//---------------------------------------------------------------Saturation
pub struct Saturation<const LIMIT: i32>;
impl<const LIMIT: i32> Saturation<LIMIT> {
    #[inline]
    pub fn iround(v: f64) -> i32 {
        if v < (-LIMIT as f64) {
            return -LIMIT;
        }
        if v > (LIMIT as f64) {
            return LIMIT;
        }
        return iround(v);
    }
}

//------------------------------------------------------------------MulOne
pub struct MulOne<const SHIFT: i32>;
impl<const SHIFT: i32> Saturation<SHIFT> {
    #[inline]
    pub fn mul(a: u32, b: u32) -> u32 {
        let q = a * b + (1 << (SHIFT - 1));
        return (q + (q >> SHIFT)) >> SHIFT;
    }
}

pub type CoverType = u8; //----CoverType

#[repr(i32)]
pub enum CoverScale {
    Shift = 8,                        //----Shift
    Size = (1 << Self::Shift as i32), //----Size
    Mask = (Self::Size as i32) - 1,   //----Mask
    None = 0,                         //----None
}

impl CoverScale {
    pub const FULL: CoverScale = Self::Mask;
}
//----------------------------------------------------PolySubpixelScale
// These constants determine the subpixel accuracy, to be more precise,
// the number of bits of the fractional part of the coordinates.
// The possible coordinate capacity in bits can be calculated by formula:
// sizeof(int) * 8 - PolySubpixelShift, i.e, for 32-bit integers and
// 8-bits fractional part the capacity is 24 bits.
#[repr(i32)]
pub enum PolySubpixelScale {
    Shift = 8,                       //----PolySubpixelShift
    Scale = 1 << Self::Shift as i32, //----PolySubpixelScale
    Mask = (Self::Scale as i32) - 1, //----PolySubpixelMask
}

//----------------------------------------------------------FillingRule
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FillingRule {
    FillNonZero,
    FillEvenOdd,
}

//-----------------------------------------------------------------------PI
const PI: f64 = 3.14159265358979323846;

//------------------------------------------------------------------deg2rad
#[inline]
pub fn deg2rad(deg: f64) -> f64 {
    return deg * PI / 180.0;
}

//------------------------------------------------------------------rad2deg
#[inline]
pub fn rad2deg(rad: f64) -> f64 {
    return rad * 180.0 / PI;
}

//----------------------------------------------------------------RectBase
#[derive(Copy, Clone)]
pub struct RectBase<T: std::cmp::PartialOrd + Copy> {
    pub x1: T,
    pub y1: T,
    pub x2: T,
    pub y2: T,
}

impl<T: std::cmp::PartialOrd + Copy> RectBase<T> {
    pub fn new(x1_: T, y1_: T, x2_: T, y2_: T) -> Self {
        RectBase {
            x1: x1_,
            y1: y1_,
            x2: x2_,
            y2: y2_,
        }
    }

    pub fn init(&mut self, x1_: T, y1_: T, x2_: T, y2_: T) {
        self.x1 = x1_;
        self.y1 = y1_;
        self.x2 = x2_;
        self.y2 = y2_;
    }

    pub fn normalize(&mut self) -> &Self {
        let mut t: T;
        if self.x1 > self.x2 {
            t = self.x1;
            self.x1 = self.x2;
            self.x2 = t;
        }
        if self.y1 > self.y2 {
            t = self.y1;
            self.y1 = self.y2;
            self.y2 = t;
        }
        self
    }

    pub fn clip(&mut self, r: &Self) -> bool {
        if self.x2 > r.x2 {
            self.x2 = r.x2;
        }
        if self.y2 > r.y2 {
            self.y2 = r.y2;
        }
        if self.x1 < r.x1 {
            self.x1 = r.x1;
        }
        if self.y1 < r.y1 {
            self.y1 = r.y1;
        }
        return self.x1 <= self.x2 && self.y1 <= self.y2;
    }

    pub fn is_valid(&self) -> bool {
        return self.x1 <= self.x2 && self.y1 <= self.y2;
    }

    pub fn hit_test(&self, x: T, y: T) -> bool {
        return x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2;
    }
}

//-----------------------------------------------------intersect_rectangles
#[inline]
pub fn intersect_rectangles<'a, T: std::cmp::PartialOrd + Copy>(
    r1: &'a mut RectBase<T>, r2: &'a RectBase<T>,
) -> &'a RectBase<T> {
    let mut r = r1;

    // First process x2,y2 because the other order
    // results in Internal Compiler Error under
    // Microsoft Visual C++ .NET 2003 69462-335-0000007-18038 in
    // case of "Maximize Speed" optimization option.
    //-----------------
    if r.x2 > r2.x2 {
        r.x2 = r2.x2;
    }
    if r.y2 > r2.y2 {
        r.y2 = r2.y2;
    }
    if r.x1 < r2.x1 {
        r.x1 = r2.x1;
    }
    if r.y1 < r2.y1 {
        r.y1 = r2.y1;
    }
    return r;
}

//---------------------------------------------------------unite_rectangles
#[inline]
pub fn unite_rectangles<'a, T: std::cmp::PartialOrd + Copy>(
    r1: &'a mut RectBase<T>, r2: &'a RectBase<T>,
) -> &'a RectBase<T> {
    let mut r = r1;

    if r.x2 < r2.x2 {
        r.x2 = r2.x2;
    }
    if r.y2 < r2.y2 {
        r.y2 = r2.y2;
    }
    if r.x1 > r2.x1 {
        r.x1 = r2.x1;
    }
    if r.y1 > r2.y1 {
        r.y1 = r2.y1;
    }
    return r;
}

pub type RectI = RectBase<i32>;
pub type RectF = RectBase<f32>;
pub type RectD = RectBase<f64>;

//---------------------------------------------------------Path Commands
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum PathCmd {
    Stop = 0,     //----Stop
    MoveTo = 1,   //----MoveTo
    LineTo = 2,   //----LineTo
    Curve3 = 3,   //----Curve3
    Curve4 = 4,   //----Curve4
    CurveN = 5,   //----CurveN
    Catrom = 6,   //----Catrom
    Ubspline = 7, //----Ubspline
    EndPoly = 0x0F, //----EndPoly
                  //Mask     = 0x0F,      //----Mask
}

//------------------------------------------------------------PathFlags
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum PathFlag {
    None = 0,     //----None
    Ccw = 0x10,   //----Ccw
    Cw = 0x20,    //----Cw
    Close = 0x40, //----Close
    Mask = 0xF0,  //----Mask
}

//---------------------------------------------------------------is_vertex
#[inline]
pub fn is_vertex(c: u32) -> bool {
    return c >= PathCmd::MoveTo as u32 && c < PathCmd::EndPoly as u32;
}

//--------------------------------------------------------------is_drawing
#[inline]
pub fn is_drawing(c: u32) -> bool {
    return c >= PathCmd::LineTo as u32 && c < PathCmd::EndPoly as u32;
}

//-----------------------------------------------------------------is_stop
#[inline]
pub fn is_stop(c: u32) -> bool {
    return c == PathCmd::Stop as u32;
}

//--------------------------------------------------------------is_move_to
#[inline]
pub fn is_move_to(c: u32) -> bool {
    return c == PathCmd::MoveTo as u32;
}

//--------------------------------------------------------------is_line_to
#[inline]
pub fn is_line_to(c: u32) -> bool {
    return c == PathCmd::LineTo as u32;
}

//----------------------------------------------------------------is_curve
#[inline]
pub fn is_curve(c: u32) -> bool {
    return c == PathCmd::Curve3 as u32 || c == PathCmd::Curve4 as u32;
}

//---------------------------------------------------------------is_curve3
#[inline]
pub fn is_curve3(c: u32) -> bool {
    return c == PathCmd::Curve3 as u32;
}

//---------------------------------------------------------------is_curve4
#[inline]
pub fn is_curve4(c: u32) -> bool {
    return c == PathCmd::Curve4 as u32;
}

//-------------------------------------------------------------is_end_poly
#[inline]
pub fn is_end_poly(c: u32) -> bool {
    return (c & PathCmd::EndPoly as u32) == PathCmd::EndPoly as u32;
}

//----------------------------------------------------------------is_close
#[inline]
pub fn is_close(c: u32) -> bool {
    return (c & !(PathFlag::Cw as u32 | PathFlag::Ccw as u32))
        == (PathCmd::EndPoly as u32 | PathFlag::Close as u32);
}

//------------------------------------------------------------is_next_poly
#[inline]
pub fn is_next_poly(c: u32) -> bool {
    return is_stop(c) || is_move_to(c) || is_end_poly(c);
}

//-------------------------------------------------------------------is_cw
#[inline]
pub fn is_cw(c: u32) -> bool {
    return (c & PathFlag::Cw as u32) != 0;
}

//------------------------------------------------------------------is_ccw
#[inline]
pub fn is_ccw(c: u32) -> bool {
    return (c & PathFlag::Ccw as u32) != 0;
}

//-------------------------------------------------------------is_oriented
#[inline]
pub fn is_oriented(c: u32) -> bool {
    return (c & (PathFlag::Cw as u32 | PathFlag::Ccw as u32)) != 0;
}

//---------------------------------------------------------------is_closed
#[inline]
pub fn is_closed(c: u32) -> bool {
    return (c & PathFlag::Close as u32) != 0;
}

//----------------------------------------------------------get_close_flag
#[inline]
pub fn get_close_flag(c: u32) -> u32 {
    return c & PathFlag::Close as u32;
}

//-------------------------------------------------------clear_orientation
#[inline]
pub fn clear_orientation(c: u32) -> u32 {
    return c & !(PathFlag::Cw as u32 | PathFlag::Ccw as u32);
}

//---------------------------------------------------------get_orientation
#[inline]
pub fn get_orientation(c: u32) -> u32 {
    return c & (PathFlag::Cw as u32 | PathFlag::Ccw as u32);
}

//---------------------------------------------------------set_orientation
#[inline]
pub fn set_orientation(c: u32, o: u32) -> u32 {
    return clear_orientation(c) | o;
}

//--------------------------------------------------------------PointBase
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PointBase<T> {
    pub x: T,
    pub y: T,
}
impl<T> Point for PointBase<T> {
    /*pub fn new() -> Self {
        Self {

        }
    }*/
    type T = T;
    fn new(x_: T, y_: T) -> Self {
        Self { x: x_, y: y_ }
    }
}

pub type PointI = PointBase<i32>;
pub type PointF = PointBase<f32>;
pub type PointD = PointBase<f64>;

//-------------------------------------------------------------VertexBase
pub struct VertexBase<T> {
    pub x: T,
    pub y: T,
    pub cmd: u32,
}
impl<T> VertexBase<T> {
    pub fn new(x_: T, y_: T, cmd_: u32) -> Self {
        Self {
            x: x_,
            y: y_,
            cmd: cmd_,
        }
    }
}

pub type VertexI = VertexBase<i32>;
pub type VertexF = VertexBase<f32>;
pub type VertexD = VertexBase<f64>;

//----------------------------------------------------------------RowInfo
pub struct RowInfo<T> {
    pub x1: i32,
    pub x2: i32,
    pub ptr: *mut T,
}

impl<T> RowInfo<T> {
    pub fn new(x1_: i32, x2_: i32, ptr_: *mut T) -> Self {
        Self {
            x1: x1_,
            x2: x2_,
            ptr: ptr_,
        }
    }
}

//----------------------------------------------------------ConstRowInfo
pub type RowData<T> = ConstRowInfo<T>;
#[derive(Clone, Copy)]
pub struct ConstRowInfo<T> {
    pub x1: i32,
    pub x2: i32,
    pub ptr: *const T, //const
}
impl<T> ConstRowInfo<T> {
    pub fn new(x1_: i32, x2_: i32, ptr_: *const T) -> Self {
        Self {
            x1: x1_,
            x2: x2_,
            ptr: ptr_,
        }
    }
}

//------------------------------------------------------------is_equal_eps
#[inline]
pub fn is_equal_eps<T: Into<f64>>(v1: T, v2: T, epsilon: T) -> bool {
    return (v1.into() - v2.into()).abs() <= (epsilon.into());
}

//--------------------------------------------------------------------
#[derive(Copy, Clone)]
pub struct Span {
    pub x: i32,
    pub len: i32,
    pub covers: *mut CoverType,
}


