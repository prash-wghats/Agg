use crate::{
    basics::{uround, CoverScale},
    AggPrimitive, Color, GrayArgs,
};

use crate::color_rgba::{Rgba, Rgba8};

//use wrapping_arithmetic::wrappit;
//===================================================================Gray8
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gray8 {
    pub v: u8,
    pub a: u8,
}

impl crate::Args for Gray8 {
    type ValueType = u8;
    #[inline]
    fn a(&self) -> Self::ValueType {
        self.a
    }
    #[inline]
    fn a_mut(&mut self) -> &mut Self::ValueType {
        &mut self.a
    }
}

impl GrayArgs for Gray8 {
	fn new_init(v: u8, a: u8) -> Self {
		Self { v: v, a: a }
	}
	
    #[inline]
    fn v(&self) -> Self::ValueType {
        self.v
    }
    #[inline]
    fn v_mut(&mut self) -> &mut Self::ValueType {
        &mut self.v
    }
}

impl Color for Gray8 {
    const BASE_SHIFT: u32 = 8;
    const BASE_SCALE: u32 = 1 << Self::BASE_SHIFT;
    const BASE_MASK: u32 = Self::BASE_SCALE - 1;

    fn new() -> Self {
        Self { v: 0, a: 0 }
    }

    fn new_from_rgba(c: &Rgba) -> Self {
        Gray8 {
            v: uround((0.299 * c.r + 0.587 * c.g + 0.114 * c.b) * Self::BASE_MASK as f64) as u8,
            a: uround(c.a as f64 * Self::BASE_MASK as f64) as u8,
        }
    }

    fn add(&mut self, c: &Self, cover: u32) {
        let (cv, ca);
        if cover == CoverScale::Mask as u32 {
            if c.a == Self::BASE_MASK as u8 {
                *self = *c;
            } else {
                cv = self.v as u32 + c.v as u32;
                self.v = if cv > Self::BASE_MASK {
                    Self::BASE_SHIFT as u8
                } else {
                    cv as u8
                };
                ca = self.a as u32 + c.a as u32;
                self.a = if ca > Self::BASE_MASK {
                    Self::BASE_MASK as u8
                } else {
                    ca as u8
                };
            }
        } else {
            cv = self.v as u32
                + ((c.v as u32 * cover + CoverScale::Mask as u32 / 2) >> CoverScale::Shift as u32);
            ca = self.a as u32
                + ((c.a as u32 * cover + CoverScale::Mask as u32 / 2) >> CoverScale::Shift as u32);
            self.v = (if cv > Self::BASE_MASK {
                Self::BASE_MASK
            } else {
                cv
            }) as u8;
            self.a = (if ca > Self::BASE_MASK {
                Self::BASE_MASK
            } else {
                ca
            }) as u8;
        }
    }

    fn transparent(&mut self) -> &Self {
        self.a = 0;
        self
    }

    fn set_opacity(&mut self, a: f64) -> &Self {
        let mut a = a;
        if a < 0.0 {
            a = 0.0;
        }
        if a > 1.0 {
            a = 1.0;
        }
        self.a = uround(a * Self::BASE_MASK as f64) as u8;
        self
    }

    fn opacity(&self) -> f64 {
        self.a as f64 / Self::BASE_MASK as f64
    }

    fn premultiply(&mut self) -> &Self {
        if self.a == Self::BASE_MASK as u8 {
            return self;
        }
        if self.a == 0 {
            self.v = 0;
            return self;
        }
        self.v = ((self.v as u32 * self.a as u32) >> Self::BASE_SHIFT) as u8;
        self
    }

    fn premultiply_a(&mut self, a: u32) -> &Self {
        if self.a == Self::BASE_MASK as u8 && a >= Self::BASE_MASK {
            return self;
        }
        if self.a == 0 || a == 0 {
            self.v = 0;
            self.a = 0;
            return self;
        }
        let v = (self.v as u32 * a as u32) / self.a as u32;
        self.v = (if v > a { a } else { v }) as u8;
        self.a = a as u8;
        self
    }

    fn demultiply(&mut self) -> &Self {
        if self.a == Self::BASE_MASK as u8 {
            return self;
        }
        if self.a == 0 {
            self.v = 0;
            return self;
        }
        let v = (self.v as u32 * Self::BASE_MASK) / self.a as u32;
        self.v = (if v > Self::BASE_MASK {
            Self::BASE_MASK
        } else {
            v
        }) as u8;
        self
    }

    fn gradient(&self, c: &Self, k: f64) -> Self {
        let ik = uround(k * Self::BASE_SCALE as f64) as u32;
        Gray8 {
            v: (self.v as u32 + (((c.v as u32 - self.v as u32) * ik) >> Self::BASE_SHIFT)) as u8,
            a: (self.a as u32 + (((c.a as u32 - self.a as u32) * ik) >> Self::BASE_SHIFT)) as u8,
        }
    }
    fn no_color() -> Gray8 {
        Gray8 { v: 0, a: 0 }
    }
    fn clear(&mut self) {
        self.v = 0;
        self.a = 0;
    }
}

impl From<Rgba8> for Gray8 {
	fn from(c: Rgba8) -> Self {
		Self::new_from_rgba8(&c)
	}
}

impl Gray8 {
    pub fn new_params(v: u32, a: u32) -> Self {
        Gray8 {
            v: v as u8,
            a: a as u8,
        }
    }
    pub fn new_from_self_a(c: &Self, a: u32) -> Self {
        Gray8 { v: c.v, a: a as u8 }
    }

    pub fn new_from_rgba_a(c: &Rgba, a: f64) -> Self {
        Gray8 {
            v: uround((0.299 * c.r + 0.587 * c.g + 0.114 * c.b) * Self::BASE_MASK as f64) as u8,
            a: uround(a * Self::BASE_MASK as f64) as u8,
        }
    }
    pub fn new_from_rgba8(c: &Rgba8) -> Self {
        Gray8 {
            v: ((c.r.into_u32() * 77 + c.g.into_u32() * 150 + c.b.into_u32() * 29) >> 8) as u8,
            a: c.a,
        }
    }
    pub fn new_from_rgba8_a(c: &Rgba8, a: u32) -> Self {
        Gray8 {
            v: ((c.r.into_u32() * 77 + c.g.into_u32() * 150 + c.b.into_u32() * 29) >> 8) as u8,
            a: a as u8,
        }
    }
}

//-------------------------------------------------------------gray8_pre
pub fn gray8_pre(v: u32, a: u32) -> Gray8 {
    let mut g = Gray8::new_params(v, a);
    g.premultiply();
    g
}
pub fn gray8_pre_c(c: &Gray8, a: u32) -> Gray8 {
    let mut g = Gray8::new_from_self_a(c, a);
    g.premultiply();
    g
}
pub fn gray8_pre_rgba(c: &Rgba) -> Gray8 {
    let mut g = Gray8::new_from_rgba(c);
    g.premultiply();
    g
}
pub fn gray8_pre_rgba_d(c: &Rgba, a: f64) -> Gray8 {
    let mut g = Gray8::new_from_rgba_a(c, a);
    g.premultiply();
    g
}
pub fn gray8_pre_rgba8(c: &Rgba8) -> Gray8 {
    let mut g = Gray8::new_from_rgba8(c);
    g.premultiply();
    g
}
pub fn gray8_pre_rgba8_u(c: &Rgba8, a: u32) -> Gray8 {
    let mut g = Gray8::new_from_rgba8_a(c, a);
    g.premultiply();
    g
}

///////////

//========================================================================Gray16
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gray16 {
    pub v: u16,
    pub a: u16,
}
impl crate::Args for Gray16 {
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

impl GrayArgs for Gray16 {
	fn new_init(v: u16, a: u16) -> Self {
		Self { v: v, a: a }
	}
    #[inline]
    fn v(&self) -> Self::ValueType {
        self.v
    }
    #[inline]
    fn v_mut(&mut self) -> &mut Self::ValueType {
        &mut self.v
    }
}

impl From<Rgba8> for Gray16 {
	fn from(c: Rgba8) -> Self {
		Self::new_from_rgba8(&c)
	}
}

impl Color for Gray16 {
    const BASE_SHIFT: u32 = 16;
    const BASE_SCALE: u32 = 1 << Self::BASE_SHIFT;
    const BASE_MASK: u32 = Self::BASE_SCALE - 1;

    fn new() -> Gray16 {
        Gray16 { v: 0, a: 0xffff }
    }

    fn new_from_rgba(c: &Rgba) -> Self {
        Gray16 {
            v: uround((0.299 * c.r + 0.587 * c.g + 0.114 * c.b) * Self::BASE_MASK as f64) as u16,
            a: uround(c.a * Self::BASE_MASK as f64) as u16,
        }
    }
    fn clear(&mut self) {
        self.v = 0;
        self.a = 0;
    }
    fn no_color() -> Self {
        Self { v: 0, a: 0 }
    }
    fn add(&mut self, c: &Self, cover: u32) {
        let cv: u32;
        let ca: u32;
        if cover == CoverScale::Mask as u32 {
            if c.a == Self::BASE_MASK as u16 {
                *self = *c;
            } else {
                cv = (self.v + c.v) as u32;
                self.v = (if cv > Self::BASE_MASK {
                    Self::BASE_MASK
                } else {
                    cv
                }) as u16;
                ca = (self.a + c.a) as u32;
                self.a = (if ca > Self::BASE_MASK {
                    Self::BASE_MASK
                } else {
                    ca
                }) as u16;
            }
        } else {
            cv = self.v as u32
                + ((c.v as u32 * cover + CoverScale::Mask as u32 / 2) >> CoverScale::Shift as u32);
            ca = self.a as u32
                + ((c.a as u32 * cover + CoverScale::Mask as u32 / 2) >> CoverScale::Shift as u32);
            self.v = (if cv > Self::BASE_MASK {
                Self::BASE_MASK
            } else {
                cv
            }) as u16;
            self.a = (if ca > Self::BASE_MASK {
                Self::BASE_MASK
            } else {
                ca
            }) as u16;
        }
    }

    fn transparent(&mut self) -> &Self {
        self.a = 0;
        self
    }

    fn set_opacity(&mut self, a_: f64) -> &Self {
        let mut a_ = a_;
        if a_ < 0.0 {
            a_ = 0.0;
        }
        if a_ > 1.0 {
            a_ = 1.0;
        }
        self.a = uround(a_ * Self::BASE_MASK as f64) as u16;
        self
    }

    fn opacity(&self) -> f64 {
        self.a as f64 / Self::BASE_MASK as f64
    }

    fn premultiply(&mut self) -> &Self {
        if self.a == Self::BASE_MASK as u16 {
            return self;
        }
        if self.a == 0 {
            self.v = 0;
            return self;
        }
        self.v = ((self.v as u32 * self.a as u32) >> Self::BASE_SHIFT) as u16;
        self
    }

    fn premultiply_a(&mut self, a_: u32) -> &Self {
        if self.a == Self::BASE_MASK as u16 && a_ >= Self::BASE_MASK {
            return self;
        }
        if self.a == 0 || a_ == 0 {
            self.v = 0;
            self.a = 0;
            return self;
        }
        let v_ = ((self.v as u32) * a_) / self.a as u32;
        self.v = (if v_ > a_ { a_ } else { v_ }) as u16;
        self.a = a_ as u16;
        self
    }

    fn demultiply(&mut self) -> &Self {
        if self.a == Self::BASE_MASK as u16 {
            return self;
        }
        if self.a == 0 {
            self.v = 0;
            return self;
        }
        let v_ = (self.v as u32 * Self::BASE_MASK) / self.a as u32;
        self.v = (if v_ > Self::BASE_MASK {
            Self::BASE_MASK
        } else {
            v_
        }) as u16;
        self
    }

    fn gradient(&self, c: &Gray16, k: f64) -> Self {
        let mut ret = Gray16::new();
        let ik = uround(k * Self::BASE_SCALE as f64) as u32;
        ret.v =
            ((self.v as u32) + (((c.v as u32 - self.v as u32) * ik) >> Self::BASE_SHIFT)) as u16;
        ret.a =
            ((self.a as u32) + (((c.a as u32 - self.a as u32) * ik) >> Self::BASE_SHIFT)) as u16;
        ret
    }
}

impl Gray16 {
    pub fn new_params(v: u32, a: u32) -> Self {
        Gray16 {
            v: v as u16,
            a: a as u16,
        }
    }
    pub fn new_from_self_a(c: &Self, a: u32) -> Self {
        Gray16 {
            v: c.v,
            a: a as u16,
        }
    }

    pub fn new_from_rgba_a(c: &Rgba, a: f64) -> Self {
        Gray16 {
            v: uround((0.299 * c.r + 0.587 * c.g + 0.114 * c.b) * Self::BASE_MASK as f64) as u16,
            a: uround(a * Self::BASE_MASK as f64) as u16,
        }
    }
    pub fn new_from_rgba8(c: &Rgba8) -> Self {
        Gray16 {
            v: (c.r.into_u32() * 77 + c.g.into_u32() * 150 + c.b.into_u32() * 29) as u16,
            a: ((c.a as u16) << 8) | c.a as u16,
        }
    }
    pub fn new_from_rgba8_a(c: &Rgba8, a: u32) -> Self {
        Gray16 {
            v: (c.r.into_u32() * 77 + c.g.into_u32() * 150 + c.b.into_u32() * 29) as u16,
            a: ((a as u16) << 8) | a as u16,
        }
    }
}

//------------------------------------------------------------gray16_pre
pub fn gray16_pre(v: u32, a: u32) -> Gray16 {
    let mut g = Gray16::new_params(v, a);
    g.premultiply();
    g
}
pub fn gray16_pre_c(c: &Gray16, a: u32) -> Gray16 {
    let mut g = Gray16::new_from_self_a(c, a);
    g.premultiply();
    g
}
pub fn gray16_pre_rgba(c: &Rgba) -> Gray16 {
    let mut g = Gray16::new_from_rgba(c);
    g.premultiply();
    g
}
pub fn gray16_pre_rgba_d(c: &Rgba, a: f64) -> Gray16 {
    let mut g = Gray16::new_from_rgba_a(c, a);
    g.premultiply();
    g
}
pub fn gray16_pre_rgba8(c: &Rgba8) -> Gray16 {
    let mut g = Gray16::new_from_rgba8(c);
    g.premultiply();
    g
}
pub fn gray16_pre_rgba8_u(c: &Rgba8, a: u32) -> Gray16 {
    let mut g = Gray16::new_from_rgba8_a(c, a);
    g.premultiply();
    g
}
