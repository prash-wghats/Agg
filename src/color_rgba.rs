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
//
// Adaptation for high precision colors has been sponsored by
// Liberty Technology Systems, Inc., visit http://lib-sys.com
//
// Liberty Technology Systems, Inc. is the provider of
// PostScript and PDF technology for software developers.
//
//----------------------------------------------------------------------------
// Contact: mcseem@antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------

use crate::basics::{uround, CoverScale};
use crate::{AggPrimitive, Color, Order, RgbArgs, };
use wrapping_arithmetic::wrappit;

/// Supported byte orders for RGB and RGBA pixel formats
pub struct OrderRgb;
impl Order for OrderRgb {
    const R: usize = 0;
    const G: usize = 1;
    const B: usize = 2;
    const A: usize = 3;
    const TAG: usize = 3;
}

pub struct OrderBgr;
impl Order for OrderBgr {
    const R: usize = 2;
    const G: usize = 1;
    const B: usize = 0;
    const A: usize = 3;
    const TAG: usize = 3;
}

pub struct OrderRgba;
impl Order for OrderRgba {
    const R: usize = 0;
    const G: usize = 1;
    const B: usize = 2;
    const A: usize = 3;
    const TAG: usize = 4;
}

pub struct OrderArgb;
impl Order for OrderArgb {
    const R: usize = 1;
    const G: usize = 2;
    const B: usize = 3;
    const A: usize = 0;
    const TAG: usize = 4;
}

pub struct OrderAbgr;
impl Order for OrderAbgr {
    const R: usize = 3;
    const G: usize = 2;
    const B: usize = 1;
    const A: usize = 0;
    const TAG: usize = 4;
}

pub struct OrderBgra;
impl Order for OrderBgra {
    const R: usize = 2;
    const G: usize = 1;
    const B: usize = 0;
    const A: usize = 3;
    const TAG: usize = 4;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RgbaBase<T: AggPrimitive> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

/*#[derive(Copy, Clone, Debug, Default)]
pub struct Rgba {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}*/

pub type Rgba = RgbaBase<f64>;
impl Rgba {
    pub fn new() -> Rgba {
        Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn new_params(r: f64, g: f64, b: f64, a: f64) -> Rgba {
        Rgba {
            r: r,
            g: g,
            b: b,
            a: a,
        }
    }

    pub fn new_from_rgba_a(c: &Rgba, a: f64) -> Rgba {
        Rgba {
            r: c.r,
            g: c.g,
            b: c.b,
            a: a,
        }
    }

    pub fn new_from_gamma(wavelen: f64, gamma: f64) -> Self {
        Self::from_wavelength(wavelen, gamma)
    }

    pub fn no_color() -> Rgba {
        Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    pub fn from_wavelength(wl: f64, gamma: f64) -> Self {
        let mut t = Self::new_params(0.0, 0.0, 0.0, 0.0);

        if wl >= 380.0 && wl <= 440.0 {
            t.r = -1.0 * (wl - 440.0) / (440.0 - 380.0);
            t.b = 1.0;
        } else if wl >= 440.0 && wl <= 490.0 {
            t.g = (wl - 440.0) / (490.0 - 440.0);
            t.b = 1.0;
        } else if wl >= 490.0 && wl <= 510.0 {
            t.g = 1.0;
            t.b = -1.0 * (wl - 510.0) / (510.0 - 490.0);
        } else if wl >= 510.0 && wl <= 580.0 {
            t.r = (wl - 510.0) / (580.0 - 510.0);
            t.g = 1.0;
        } else if wl >= 580.0 && wl <= 645.0 {
            t.r = 1.0;
            t.g = -1.0 * (wl - 645.0) / (645.0 - 580.0);
        } else if wl >= 645.0 && wl <= 780.0 {
            t.r = 1.0;
        }

        let mut s = 1.0;
        if wl > 700.0 {
            s = 0.3 + 0.7 * (780.0 - wl) / (780.0 - 700.0);
        } else if wl < 420.0 {
            s = 0.3 + 0.7 * (wl - 380.0) / (420.0 - 380.0);
        }

        t.r = t.r * s.powf(gamma);
        t.g = t.g * s.powf(gamma);
        t.b = t.b * s.powf(gamma);
        t
    }
}

impl Rgba {
    pub const BASE_SHIFT: u32 = 0;
    pub const BASE_SCALE: u32 = 1 << Rgba::BASE_SHIFT;
    pub const BASE_MASK: u32 = Rgba::BASE_SCALE - 1;

    pub fn clear(&mut self) {
        self.r = 0.;
        self.g = 0.;
        self.b = 0.;
        self.a = 0.;
    }

    pub fn transparent(&mut self) -> &Rgba {
        self.a = 0.0;
        self
    }

    pub fn set_opacity(&mut self, a: f64) -> &Rgba {
        let mut a_ = a;
        if a_ < 0.0 {
            a_ = 0.0;
        }
        if a_ > 1.0 {
            a_ = 1.0;
        }
        self.a = a_;
        self
    }

    pub fn opacity(&self) -> f64 {
        self.a
    }

    pub fn premultiply(&mut self) -> &Self {
        self.r *= self.a;
        self.g *= self.a;
        self.b *= self.a;
        self
    }

    pub fn premultiply_a(&mut self, a: f64) -> &Self {
        let mut a_ = a;
        if self.a <= 0.0 || a_ <= 0.0 {
            self.r = 0.0;
            self.g = 0.0;
            self.b = 0.0;
            self.a = 0.0;
            return self;
        }
        a_ /= self.a;
        self.r *= a_;
        self.g *= a_;
        self.b *= a_;
        self.a = a_;
        self
    }

    pub fn demultiply(&mut self) -> &Self {
        if self.a == 0. {
            self.r = 0.;
            self.g = 0.;
            self.b = 0.;
            return self;
        }
        let a_ = 1.0 / self.a;
        self.r *= a_;
        self.g *= a_;
        self.b *= a_;
        return self;
    }

    pub fn gradient(&self, c: &Rgba, k: f64) -> Rgba {
        Rgba {
            r: self.r + (c.r - self.r) * k,
            g: self.g + (c.g - self.g) * k,
            b: self.b + (c.b - self.b) * k,
            a: self.a + (c.a - self.a) * k,
        }
    }
}

//----------------------------------------------------------------rgba_pre
#[inline]
pub fn rgba_pre(r: f64, g: f64, b: f64, a: f64) -> Rgba {
    *Rgba::new_params(r, g, b, a).premultiply()
}

#[inline]
pub fn rgba_pre_rgba(c: &Rgba) -> Rgba {
    *Rgba::new_params(c.r, c.g, c.b, c.a).premultiply()
}

#[inline]
pub fn rgba_pre_rgba_a(c: &Rgba, a: f64) -> Rgba {
    *Rgba::new_params(c.r, c.g, c.b, a).premultiply()
}

//===================================================================Rgba8
/*#[derive(Copy, Clone, Debug, Default)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}*/

pub type Rgba8 = RgbaBase<u8>;

impl Rgba8 {
    pub fn new_params(r_: u32, g_: u32, b_: u32, a_: u32) -> Self {
        Rgba8 {
            r: r_ as u8,
            g: g_ as u8,
            b: b_ as u8,
            a: a_ as u8,
        }
    }

    pub fn new_from_rgba_a(c: &Rgba, a_: f64) -> Self {
        Rgba8 {
            r: (c.r * Rgba8::BASE_MASK as f64) as u8,
            g: (c.g * Rgba8::BASE_MASK as f64) as u8,
            b: (c.b * Rgba8::BASE_MASK as f64) as u8,
            a: (a_ * Rgba8::BASE_MASK as f64) as u8,
        }
    }

    pub fn new_from_self_a(c: &Rgba8, a_: u32) -> Self {
        Rgba8 {
            r: c.r,
            g: c.g,
            b: c.b,
            a: a_ as u8,
        }
    }

    #[inline]
    pub fn apply_gamma_dir<Gamma: crate::Gamma<u8, u8>>(&mut self, gamma: &Gamma) {
        self.r = gamma.dir(self.r);
        self.g = gamma.dir(self.g);
        self.b = gamma.dir(self.b);
    }

    #[inline]
    pub fn apply_gamma_inv<Gamma: crate::Gamma<u8, u8>>(&mut self, gamma: &Gamma) {
        self.r = gamma.inv(self.r);
        self.g = gamma.inv(self.g);
        self.b = gamma.inv(self.b);
    }

    pub fn from_wavelength(wl: f64, gamma: f64) -> Self {
        Self::new_from_rgba(&Rgba::from_wavelength(wl, gamma))
    }
}

impl crate::Args for Rgba8 {
    type ValueType = u8;
	fn a(&self) -> Self::ValueType {
        self.a
    }

    #[inline]
    fn a_mut(&mut self) -> &mut Self::ValueType {
        &mut self.a
    }
}

impl RgbArgs for Rgba8 {
	fn new_init(r_: u8, g_: u8, b_: u8, a_: u8) -> Self {
        Rgba8 {
            r: r_ ,
            g: g_ ,
            b: b_ ,
            a: a_,
        }
    }

    #[inline]
    fn r(&self) -> Self::ValueType {
        self.r
    }

    #[inline]
    fn g(&self) -> Self::ValueType {
        self.g
    }

    #[inline]
    fn b(&self) -> Self::ValueType {
        self.b
    }

    #[inline]
    fn r_mut(&mut self) -> &mut Self::ValueType {
        &mut self.r
    }

    #[inline]
    fn g_mut(&mut self) -> &mut Self::ValueType {
        &mut self.g
    }

    #[inline]
    fn b_mut(&mut self) -> &mut Self::ValueType {
        &mut self.b
    }
}

impl crate::Color for Rgba8 {
    const BASE_SHIFT: u32 = 8;
    const BASE_SCALE: u32 = 1 << Rgba8::BASE_SHIFT;
    const BASE_MASK: u32 = Rgba8::BASE_SCALE - 1;

    fn new() -> Self {
        Rgba8 {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    fn new_from_rgba(c: &Rgba) -> Self {
        Rgba8 {
            r: (c.r * Rgba8::BASE_MASK as f64) as u8,
            g: (c.g * Rgba8::BASE_MASK as f64) as u8,
            b: (c.b * Rgba8::BASE_MASK as f64) as u8,
            a: (c.a * Rgba8::BASE_MASK as f64) as u8,
        }
    }

    fn no_color() -> Self {
        Rgba8 {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
    #[inline]
    fn add(&mut self, c: &Self, cover: u32) {
        let cr: u32;
        let cg: u32;
        let cb: u32;
        let ca: u32;
        if cover == CoverScale::Mask as u32 {
            if c.a == Rgba8::BASE_MASK as u8 {
                *self = *c;
            } else {
                cr = (self.r + c.r) as u32;
                self.r = (if cr > Rgba8::BASE_MASK as u32 {
                    Rgba8::BASE_MASK as u32
                } else {
                    cr
                }) as u8;
                cg = (self.g + c.g) as u32;
                self.g = (if cg > Rgba8::BASE_MASK as u32 {
                    Rgba8::BASE_MASK as u32
                } else {
                    cg
                }) as u8;
                cb = (self.b + c.b) as u32;
                self.b = (if cb > Rgba8::BASE_MASK as u32 {
                    Rgba8::BASE_MASK as u32
                } else {
                    cb
                }) as u8;
                ca = (self.a + c.a) as u32;
                self.a = (if ca > Rgba8::BASE_MASK as u32 {
                    Rgba8::BASE_MASK as u32
                } else {
                    ca
                }) as u8;
            }
        } else {
            cr = self.r as u32
                + ((c.r as u32 * cover + CoverScale::Mask as u32 / 2) >> CoverScale::Shift as u32);
            cg = self.g as u32
                + ((c.g as u32 * cover + CoverScale::Mask as u32 / 2) >> CoverScale::Shift as u32);
            cb = self.b as u32
                + ((c.b as u32 * cover + CoverScale::Mask as u32 / 2) >> CoverScale::Shift as u32);
            ca = self.a as u32
                + ((c.a as u32 * cover + CoverScale::Mask as u32 / 2) >> CoverScale::Shift as u32);
            self.r = (if cr > Rgba8::BASE_MASK as u32 {
                Rgba8::BASE_MASK as u32
            } else {
                cr
            }) as u8;
            self.g = (if cg > Rgba8::BASE_MASK as u32 {
                Rgba8::BASE_MASK as u32
            } else {
                cg
            }) as u8;
            self.b = (if cb > Rgba8::BASE_MASK as u32 {
                Rgba8::BASE_MASK as u32
            } else {
                cb
            }) as u8;
            self.a = (if ca > Rgba8::BASE_MASK as u32 {
                Rgba8::BASE_MASK as u32
            } else {
                ca
            }) as u8;
        }
    }

    fn clear(&mut self) {
        self.r = 0;
        self.g = 0;
        self.b = 0;
        self.a = 0;
    }

    fn transparent(&mut self) -> &Self {
        self.a = 0;
        self
    }

    fn set_opacity(&mut self, a: f64) -> &Self {
        let mut a_ = a;
        if a_ < 0.0 {
            a_ = 0.0;
        }
        if a_ > 1.0 {
            a_ = 1.0;
        }
        self.a = uround(a_ * Rgba8::BASE_MASK as f64) as u8;
        self
    }

    fn opacity(&self) -> f64 {
        self.a as f64 / Rgba8::BASE_MASK as f64
    }

    #[inline]
    fn premultiply(&mut self) -> &Self {
        if self.a as u32 == Rgba8::BASE_MASK {
            return self;
        }
        if self.a as u32 == 0 {
            self.r = 0;
            self.g = 0;
            self.b = 0;
            return self;
        }
        self.r = ((self.r as u32 * self.a as u32) >> Rgba8::BASE_SHIFT) as u8;
        self.g = ((self.g as u32 * self.a as u32) >> Rgba8::BASE_SHIFT) as u8;
        self.b = ((self.b as u32 * self.a as u32) >> Rgba8::BASE_SHIFT) as u8;
        self
    }

    #[inline]
    fn premultiply_a(&mut self, a_: u32) -> &Self {
        if self.a as u32 == Rgba8::BASE_MASK && a_ as u32 >= Rgba8::BASE_MASK {
            return self;
        }
        if self.a as u32 == 0 || a_ == 0 {
            self.r = 0;
            self.g = 0;
            self.b = 0;
            self.a = 0;
            return self;
        }
        let r_ = (self.r as u32 * a_ as u32) / self.a as u32;
        let g_ = (self.g as u32 * a_ as u32) / self.a as u32;
        let b_ = (self.b as u32 * a_ as u32) / self.a as u32;
        self.r = ((r_ > a_ as u32) as u32 * a_ as u32 + (r_ <= a_ as u32) as u32 * r_) as u8;
        self.g = ((g_ > a_ as u32) as u32 * a_ as u32 + (g_ <= a_ as u32) as u32 * g_) as u8;
        self.b = ((b_ > a_ as u32) as u32 * a_ as u32 + (b_ <= a_ as u32) as u32 * b_) as u8;
        self.a = a_ as u8;
        self
    }

    #[inline]
    fn demultiply(&mut self) -> &Self {
        if self.a as u32 == Rgba8::BASE_MASK {
            return self;
        }
        if self.a as u32 == 0 {
            self.r = 0;
            self.g = 0;
            self.b = 0;
            return self;
        }
        let r_ = (self.r as u32 * Rgba8::BASE_MASK as u32) / self.a as u32;
        let g_ = (self.g as u32 * Rgba8::BASE_MASK as u32) / self.a as u32;
        let b_ = (self.b as u32 * Rgba8::BASE_MASK as u32) / self.a as u32;
        self.r = ((r_ > Rgba8::BASE_MASK as u32) as u32 * Rgba8::BASE_MASK as u32
            + (r_ <= Rgba8::BASE_MASK as u32) as u32 * r_) as u8;
        self.g = ((g_ > Rgba8::BASE_MASK as u32) as u32 * Rgba8::BASE_MASK as u32
            + (g_ <= Rgba8::BASE_MASK as u32) as u32 * g_) as u8;
        self.b = ((b_ > Rgba8::BASE_MASK as u32) as u32 * Rgba8::BASE_MASK as u32
            + (b_ <= Rgba8::BASE_MASK as u32) as u32 * b_) as u8;
        self
    }

    #[wrappit]
    #[inline] //XXX check u16
    fn gradient(&self, c: &Self, k: f64) -> Self {
        let mut ret = Self::new();
        let ik = uround(k * Rgba8::BASE_SCALE as f64) as u32;

        ret.r = ((self.r as u32) + ((((c.r as u32) - self.r as u32) * ik) >> Rgba8::BASE_SHIFT))
            as u8;
        ret.g = ((self.g as u32) + ((((c.g as u32) - self.g as u32) * ik) >> Rgba8::BASE_SHIFT))
            as u8;
        ret.b = ((self.b as u32) + ((((c.b as u32) - self.b as u32) * ik) >> Rgba8::BASE_SHIFT))
            as u8;
        ret.a = ((self.a as u32) + ((((c.a as u32) - self.a as u32) * ik) >> Rgba8::BASE_SHIFT))
            as u8;

        return ret;
    }
}

//-------------------------------------------------------------rgba8_pre
#[inline]
pub fn rgba8_pre(r: u32, g: u32, b: u32, a: u32) -> Rgba8 {
    *Rgba8::new_params(r, g, b, a).premultiply()
}

#[inline]
pub fn rgba8_pre_from_rgba8(c: &Rgba8) -> Rgba8 {
    *Rgba8::new_from_self_a(c, 255).premultiply()
}

#[inline]
pub fn rgba8_pre_from_rgba8_a(c: &Rgba8, a: u32) -> Rgba8 {
    *Rgba8::new_from_self_a(c, a).premultiply()
}

#[inline]
pub fn rgba8_pre_crgba(c: &Rgba) -> Rgba8 {
    *Rgba8::new_from_rgba(c).premultiply()
}

#[inline]
pub fn rgba8_pre_from_rgba_a(c: &Rgba, a: f64) -> Rgba8 {
    *Rgba8::new_from_rgba_a(c, a).premultiply()
}

//-------------------------------------------------------------rgb8_packed
#[inline]
pub fn rgb8_packed(v: u32) -> Rgba8 {
    Rgba8::new_params(
        (v >> 16) & 0xFF,
        (v >> 8) & 0xFF,
        v & 0xFF,
        Rgba8::BASE_MASK,
    )
}

//-------------------------------------------------------------bgr8_packed
#[inline]
pub fn bgr8_packed(v: u32) -> Rgba8 {
    Rgba8::new_params(
        v & 0xFF,
        (v >> 8) & 0xFF,
        (v >> 16) & 0xFF,
        Rgba8::BASE_MASK,
    )
}

//------------------------------------------------------------argb8_packed
#[inline]
pub fn argb8_packed(v: u32) -> Rgba8 {
    Rgba8::new_params((v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF, v >> 24)
}

//---------------------------------------------------------rgba8_gamma_dir
pub fn rgba8_gamma_dir<Gamma: crate::Gamma<u8, u8>>(c: Rgba8, gamma: &Gamma) -> Rgba8 {
    Rgba8::new_params(
        gamma.dir(c.r) as u32,
        gamma.dir(c.g) as u32,
        gamma.dir(c.b) as u32,
        c.a as u32,
    )
}

//---------------------------------------------------------rgba8_gamma_inv
pub fn rgba8_gamma_inv<Gamma: crate::Gamma<u8, u8>>(c: Rgba8, gamma: &Gamma) -> Rgba8 {
    Rgba8::new_params(
        gamma.inv(c.r) as u32,
        gamma.inv(c.g) as u32,
        gamma.inv(c.b) as u32,
        c.a as u32,
    )
}

//==================================================================Rgba16
/*#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rgba16 {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub a: u16,
}*/

pub type Rgba16 = RgbaBase<u16>;

impl Rgba16 {
    pub fn new_params(r_: u32, g_: u32, b_: u32, a_: u32) -> Self {
        Rgba16 {
            r: r_ as u16,
            g: g_ as u16,
            b: b_ as u16,
            a: a_ as u16,
        }
    }

    pub fn new_from_self_a(c: &Rgba16, a_: u32) -> Self {
        Rgba16 {
            r: c.r,
            g: c.g,
            b: c.b,
            a: a_ as u16,
        }
    }

    pub fn new_from_rgba_a(c: &Rgba, a_: f64) -> Self {
        Rgba16 {
            r: (c.r * Rgba16::BASE_MASK as f64) as u16,
            g: (c.g * Rgba16::BASE_MASK as f64) as u16,
            b: (c.b * Rgba16::BASE_MASK as f64) as u16,
            a: (a_ * Rgba16::BASE_MASK as f64) as u16,
        }
    }

    pub fn new_from_rgba8(c: &Rgba8) -> Self {
        let u = Rgba16 {
            r: (c.r as u16) << 8 | c.r as u16,
            g: (c.g as u16) << 8 | c.g as u16,
            b: (c.b as u16) << 8 | c.b as u16,
            a: (c.a as u16) << 8 | c.a as u16,
        };
		let mut i = u;
		i.a += 0;
		u
    }

    pub fn new_from_rgba8_a(c: &Rgba8, a_: u16) -> Self {
        Rgba16 {
            r: (c.r as u16) << 8 | c.r as u16,
            g: (c.g as u16) << 8 | c.g as u16,
            b: (c.b as u16) << 8 | c.b as u16,
            a: (a_ as u16) << 8 | c.a as u16,
        }
    }

    #[inline]
    pub fn apply_gamma_dir<Gamma: crate::Gamma<u16, u16>>(&mut self, gamma: &Gamma) {
        self.r = gamma.dir(self.r);
        self.g = gamma.dir(self.g);
        self.b = gamma.dir(self.b);
    }

    #[inline]
    pub fn apply_gamma_inv<Gamma: crate::Gamma<u16, u16>>(&mut self, gamma: &Gamma) {
        self.r = gamma.inv(self.r);
        self.g = gamma.inv(self.g);
        self.b = gamma.inv(self.b);
    }

    pub fn from_wavelength(wl: f64, gamma: f64) -> Self {
        Self::new_from_rgba(&Rgba::from_wavelength(wl, gamma))
    }
}

impl crate::Args for Rgba16 {
    type ValueType = u16;
	#[inline]
    fn a(&self) -> Self::ValueType {
        self.a
    }
    #[inline]
    fn a_mut(&mut self) -> &mut Self::ValueType {
        &mut self.a
    }
}

impl RgbArgs for Rgba16 {
	fn new_init(r_: u16, g_: u16, b_: u16, a_: u16) -> Self {
        Rgba16 {
            r: r_ ,
            g: g_ ,
            b: b_ ,
            a: a_,
        }
    }

    #[inline]
    fn r(&self) -> Self::ValueType {
        self.r
    }

    #[inline]
    fn g(&self) -> Self::ValueType {
        self.g
    }
    #[inline]
    fn b(&self) -> Self::ValueType {
        self.b
    }

    #[inline]
    fn r_mut(&mut self) -> &mut Self::ValueType {
        &mut self.r
    }

    #[inline]
    fn g_mut(&mut self) -> &mut Self::ValueType {
        &mut self.g
    }

    #[inline]
    fn b_mut(&mut self) -> &mut Self::ValueType {
        &mut self.b
    }
}

impl From<Rgba8> for Rgba16 {
	fn from(c: Rgba8) -> Self {
		Self::new_from_rgba8(&c)
	}
}

impl crate::Color for Rgba16 {
    const BASE_SHIFT: u32 = 16;
    const BASE_SCALE: u32 = 1 << Rgba16::BASE_SHIFT;
    const BASE_MASK: u32 = Rgba16::BASE_SCALE - 1;

    fn new() -> Self {
        Rgba16 {
            r: 0,
            g: 0,
            b: 0,
            a: 0xffff,
        }
    }

    fn new_from_rgba(c: &Rgba) -> Rgba16 {
        Rgba16 {
            r: (c.r * Rgba16::BASE_MASK as f64) as u16,
            g: (c.g * Rgba16::BASE_MASK as f64) as u16,
            b: (c.b * Rgba16::BASE_MASK as f64) as u16,
            a: (c.a * Rgba16::BASE_MASK as f64) as u16,
        }
    }

    fn no_color() -> Self {
        Rgba16 {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    #[inline]
    fn add(&mut self, c: &Self, cover: u32) {
        let cr: u32;
        let cg: u32;
        let cb: u32;
        let ca: u32;
        if cover == CoverScale::Mask as u32 {
            if c.a as u32 == Rgba16::BASE_MASK {
                *self = *c;
            } else {
                cr = (self.r + c.r) as u32;
                self.r = if cr as u32 > Rgba16::BASE_MASK {
                    Rgba16::BASE_MASK as u16
                } else {
                    cr as u16
                };
                cg = (self.g + c.g) as u32;
                self.g = if cg as u32 > Rgba16::BASE_MASK {
                    Rgba16::BASE_MASK as u16
                } else {
                    cg as u16
                };
                cb = (self.b + c.b) as u32;
                self.b = if cb as u32 > Rgba16::BASE_MASK {
                    Rgba16::BASE_MASK as u16
                } else {
                    cb as u16
                };
                ca = (self.a + c.a) as u32;
                self.a = if ca as u32 > Rgba16::BASE_MASK {
                    Rgba16::BASE_MASK as u16
                } else {
                    ca as u16
                };
            }
        } else {
            cr = self.r as u32
                + ((c.r as u32 * cover + CoverScale::Mask as u32) >> CoverScale::Shift as i32);
            cg = self.g as u32
                + ((c.g as u32 * cover + CoverScale::Mask as u32) >> CoverScale::Shift as i32);
            cb = self.b as u32
                + ((c.b as u32 * cover + CoverScale::Mask as u32) >> CoverScale::Shift as i32);
            ca = self.a as u32
                + ((c.a as u32 * cover + CoverScale::Mask as u32) >> CoverScale::Shift as i32);
            self.r = if cr as i32 > Rgba16::BASE_MASK as i32 {
                Rgba16::BASE_MASK as u16
            } else {
                cr as u16
            };
            self.g = if cg as i32 > Rgba16::BASE_MASK as i32 {
                Rgba16::BASE_MASK as u16
            } else {
                cg as u16
            };
            self.b = if cb as i32 > Rgba16::BASE_MASK as i32 {
                Rgba16::BASE_MASK as u16
            } else {
                cb as u16
            };
            self.a = if ca as i32 > Rgba16::BASE_MASK as i32 {
                Rgba16::BASE_MASK as u16
            } else {
                ca as u16
            };
        }
    }

    fn clear(&mut self) {
        self.r = 0;
        self.g = 0;
        self.b = 0;
        self.a = 0;
    }

    fn transparent(&mut self) -> &Self {
        self.a = 0;
        self
    }

    fn set_opacity(&mut self, a: f64) -> &Self {
        let mut a_ = a;

        if a_ < 0.0 {
            a_ = 0.0;
        }
        if a_ > 1.0 {
            a_ = 1.0;
        }
        self.a = uround(a_ * Rgba16::BASE_MASK as f64) as u16;
        self
    }

    fn opacity(&self) -> f64 {
        self.a as f64 / Rgba16::BASE_MASK as f64
    }

    #[inline]
    fn premultiply(&mut self) -> &Self {
        if self.a as u32 == Rgba16::BASE_MASK {
            return self;
        }
        if self.a as u32 == 0 {
            self.r = 0;
            self.g = 0;
            self.b = 0;
            return self;
        }
        self.r = ((self.r as u32 * self.a as u32) >> Rgba16::BASE_SHIFT) as u16;
        self.g = ((self.g as u32 * self.a as u32) >> Rgba16::BASE_SHIFT) as u16;
        self.b = ((self.b as u32 * self.a as u32) >> Rgba16::BASE_SHIFT) as u16;
        self
    }

    #[inline]
    fn premultiply_a(&mut self, a_: u32) -> &Self {
        if self.a as u32 == Rgba16::BASE_MASK && a_ >= Rgba16::BASE_MASK {
            return self;
        }
        if self.a as u32 == 0 || a_ == 0 {
            self.r = 0;
            self.g = 0;
            self.b = 0;
            self.a = 0;
            return self;
        }
        let r_ = (self.r as u32 * a_) / self.a as u32;
        let g_ = (self.g as u32 * a_) / self.a as u32;
        let b_ = (self.b as u32 * a_) / self.a as u32;
        self.r = (if r_ > a_ { a_ } else { r_ }) as u16;
        self.g = (if g_ > a_ { a_ } else { g_ }) as u16;
        self.b = (if b_ > a_ { a_ } else { b_ }) as u16;
        self.a = a_ as u16;
        self
    }

    #[inline]
    fn demultiply(&mut self) -> &Self {
        if self.a as u32 == Rgba16::BASE_MASK {
            return self;
        }
        if self.a as u32 == 0 {
            self.r = 0;
            self.g = 0;
            self.b = 0;
            return self;
        }
        let r_ = (self.r as u32 * Rgba16::BASE_MASK) / self.a as u32;
        let g_ = (self.g as u32 * Rgba16::BASE_MASK) / self.a as u32;
        let b_ = (self.b as u32 * Rgba16::BASE_MASK) / self.a as u32;
        self.r = ((r_ > Rgba16::BASE_MASK as u32) as u32 * Rgba16::BASE_MASK) as u16;
        self.g = ((g_ > Rgba16::BASE_MASK as u32) as u32 * Rgba16::BASE_MASK) as u16;
        self.b = ((b_ > Rgba16::BASE_MASK as u32) as u32 * Rgba16::BASE_MASK) as u16;
        self
    }

    #[inline]
    fn gradient(&self, c: &Rgba16, k: f64) -> Rgba16 {
        let mut ret = Rgba16::new();
        let ik = uround(k * Rgba16::BASE_SCALE as f64) as u32;
        ret.r = ((self.r) as u32 + ((((c.r) - self.r) as u32 * ik) >> Rgba16::BASE_SHIFT)) as u16;
        ret.g = ((self.g) as u32 + ((((c.g) - self.g) as u32 * ik) >> Rgba16::BASE_SHIFT)) as u16;
        ret.b = ((self.b) as u32 + ((((c.b) - self.b) as u32 * ik) >> Rgba16::BASE_SHIFT)) as u16;
        ret.a = ((self.a) as u32 + ((((c.a) - self.a) as u32 * ik) >> Rgba16::BASE_SHIFT)) as u16;
        ret.r = (self.r as u32 + (((c.r - self.r) as u32 * ik) >> Rgba16::BASE_SHIFT)) as u16;
        ret.g = (self.g as u32 + (((c.g - self.g) as u32 * ik) >> Rgba16::BASE_SHIFT)) as u16;
        ret.b = (self.b as u32 + (((c.b - self.b) as u32 * ik) >> Rgba16::BASE_SHIFT)) as u16;
        ret.a = (self.a as u32 + (((c.a - self.a) as u32 * ik) >> Rgba16::BASE_SHIFT)) as u16;

        return ret;
    }
}

//--------------------------------------------------------------rgba16_pre
#[inline]
pub fn rgba16_pre(r: u32, g: u32, b: u32, a: u32) -> Rgba16 {
    *Rgba16::new_params(r, g, b, a).premultiply()
}

#[inline]
pub fn rgba16_pre_rgba16_a(c: &Rgba16, a: u32) -> Rgba16 {
    *Rgba16::new_from_self_a(c, a).premultiply()
}

#[inline]
pub fn rgba16_pre_from_rgba(c: &Rgba) -> Rgba16 {
    *Rgba16::new_from_rgba(c).premultiply()
}

#[inline]
pub fn rgba16_pre_from_rgba_a(c: &Rgba, a: f64) -> Rgba16 {
    *Rgba16::new_from_rgba_a(c, a).premultiply()
}

#[inline]
pub fn rgba16_pre_from_rgba8(c: &Rgba8) -> Rgba16 {
    *Rgba16::new_from_rgba8(c).premultiply()
}

#[inline]
pub fn rgba16_pre_from_rgba8_a(c: &Rgba8, a: u16) -> Rgba16 {
    *Rgba16::new_from_rgba8_a(c, a).premultiply()
}

pub fn rgba16_gamma_dir<Gamma: crate::Gamma<u16, u16>>(c: Rgba16, gamma: &Gamma) -> Rgba16 {
    Rgba16::new_params(
        gamma.dir(c.r) as u32,
        gamma.dir(c.g) as u32,
        gamma.dir(c.b) as u32,
        c.a as u32,
    )
}

pub fn rgba16_gamma_inv<Gamma: crate::Gamma<u16, u16>>(c: Rgba16, gamma: &Gamma) -> Rgba16 {
    Rgba16::new_params(
        gamma.inv(c.r) as u32,
        gamma.inv(c.g) as u32,
        gamma.inv(c.b) as u32,
        c.a as u32,
    )
}
