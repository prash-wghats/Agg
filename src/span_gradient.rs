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

use crate::{
    basics::{iround, uround},
    Color, ColorFn, GradientFunc, Interpolator, SpanGenerator,
};
use std::marker::PhantomData;
use wrapping_arithmetic::wrappit;

pub enum GradientSubpixelScale {
    Shift = 4,                       //-----Shift
    Scale = 1 << Self::Shift as u32, //-----Scale
    Mask = Self::Scale as isize - 1, //-----Mask
}

use self::GradientSubpixelScale::*;
//==========================================================SpanGradient
pub struct SpanGradient<'a, C: Color, I: Interpolator, GF: GradientFunc, CF: ColorFn<C>> {
    interpolator: &'a mut I,
    gradient_function: &'a mut GF,
    color_function: &'a mut CF,
    d1: i32,
    d2: i32,
    _c: PhantomData<C>,
}

impl<'a, C: Color, I: Interpolator, GF: GradientFunc, CF: ColorFn<C>> SpanGradient<'a, C, I, GF, CF> {
    pub fn new(
        interpolator: &'a mut I, gradient_function: &'a mut GF, color_function: &'a mut CF, d1: f64, d2: f64,
    ) -> Self {
        SpanGradient {
            interpolator: interpolator,
            gradient_function: gradient_function,
            color_function: color_function,
            d1: (d1 * Scale as u32 as f64) as i32,
            d2: (d2 * Scale as u32 as f64) as i32,
            _c: PhantomData,
        }
    }
    const DOWNSCALE_SHIFT: u32 = I::SUBPIXEL_SHIFT - Shift as u32;

    pub fn interpolator_mut(&mut self) -> &mut I {
        self.interpolator
    }

    pub fn gradient_function_mut(&mut self) -> &mut GF {
        self.gradient_function
    }

    pub fn color_function_mut(&mut self) -> &mut CF {
        self.color_function
    }

    pub fn d1(&self) -> f64 {
        self.d1 as f64 / Scale as u32 as f64
    }

    pub fn d2(&self) -> f64 {
        self.d2 as f64 / Scale as u32 as f64
    }

    pub fn set_interpolator(&mut self, i: &'a mut I) {
        self.interpolator = i;
    }

    pub fn set_gradient_function(&mut self, gf: &'a mut GF) {
        self.gradient_function = gf;
    }

    pub fn set_color_function(&mut self, cf: &'a mut CF) {
        self.color_function = cf;
    }

    pub fn set_d1(&mut self, v: f64) {
        self.d1 = (v * Scale as u32 as f64) as i32;
    }

    pub fn set_d2(&mut self, v: f64) {
        self.d2 = (v * Scale as u32 as f64) as i32;
    }
}

impl<'a, C: Color, I: Interpolator, GF: GradientFunc, CF: ColorFn<C>> SpanGenerator
    for SpanGradient<'a, C, I, GF, CF>
{
    type C = C;

    fn prepare(&mut self) {}

    fn generate(&mut self, span: &mut [C], x: i32, y: i32, len: u32) {
        let mut dd = self.d2 - self.d1;
        if dd < 1 {
            dd = 1;
        }
        self.interpolator.begin(x as f64 + 0.5, y as f64 + 0.5, len);
        for i in 0..len {
            let (mut x, mut y) = (0, 0);
            self.interpolator.coordinates(&mut x, &mut y);
            let mut d = self.gradient_function.calculate(
                x as i32 >> Self::DOWNSCALE_SHIFT,
                y as i32 >> Self::DOWNSCALE_SHIFT,
                self.d2,
            );
            d = ((d - self.d1) * self.color_function.size() as i32) / dd;
            if d < 0 {
                d = 0;
            }
            if d >= self.color_function.size() as i32 {
                d = self.color_function.size() as i32 - 1;
            }

            span[i as usize] = self.color_function.get(d as u32);
            self.interpolator.next();
        }
    }
}

pub struct GradientX;
impl GradientFunc for GradientX {
    fn calculate(&self, x: i32, _: i32, _: i32) -> i32 {
        x
    }
}

pub struct GradientY;
impl GradientFunc for GradientY {
    fn calculate(&self, _: i32, y: i32, _: i32) -> i32 {
        y
    }
}

pub struct GradientDiamond;
impl GradientFunc for GradientDiamond {
    fn calculate(&self, x: i32, y: i32, _: i32) -> i32 {
        let ax = x.abs();
        let ay = y.abs();
        if ax > ay {
            ax
        } else {
            ay
        }
    }
}

pub struct GradientXY;
impl GradientFunc for GradientXY {
	#[wrappit]
    fn calculate(&self, x: i32, y: i32, d: i32) -> i32 {
        x.abs() * y.abs() / d
    }
}

use num::integer::Roots; //XXXX
pub struct GradientSqrtXY;
impl GradientFunc for GradientSqrtXY {
	#[wrappit]
    fn calculate(&self, x: i32, y: i32, _: i32) -> i32 {
        //fast_sqrt(x.abs() * y.abs())
        (x.abs() * y.abs()).sqrt()
    }
}

pub struct GradientConic;
impl GradientFunc for GradientConic {
    fn calculate(&self, x: i32, y: i32, d: i32) -> i32 {
        (f64::atan2(y as f64, x as f64).abs() * d as f64 / std::f64::consts::PI) as i32
    }
}

// NOT TESTED
//=====================================================gradient_linear_color
pub struct GradientLinearColor<C: Color> {
    pub c1: C,
    pub c2: C,
    pub size: u32,
}

impl<C: Color> GradientLinearColor<C> {
    pub fn new(c1: C, c2: C, size: u32) -> Self {
        Self {
            c1: c1,
            c2: c2,
            size: size,
        }
    }

    pub fn colors(&mut self, c1: C, c2: C, size: u32) {
        self.c1 = c1;
        self.c2 = c2;
        self.size = size;
    }
}

impl<C: Color> ColorFn<C> for GradientLinearColor<C> {
    fn size(&self) -> u32 {
        self.size
    }

    fn get(&mut self, v: u32) -> C {
        self.c1
            .gradient(&self.c2, v as f64 / (self.size - 1) as f64)
    }
}

// same as radial. Just for compatibility
pub struct GradientCircle;
impl GradientFunc for GradientCircle {
	#[wrappit]
    fn calculate(&self, x: i32, y: i32, _: i32) -> i32 {
        (x * x + y * y).sqrt() as i32
    }
}

pub struct GradientRadial;
impl GradientFunc for GradientRadial {
	#[wrappit]
    fn calculate(&self, x: i32, y: i32, _: i32) -> i32 {
        (x * x + y * y).sqrt() as i32
    }
}

pub struct GradientRadialD;
impl GradientFunc for GradientRadialD {
    fn calculate(&self, x: i32, y: i32, _: i32) -> i32 {
		let sum = x as f64 * x as f64 + y as f64 * y as f64;
        uround(sum).sqrt()
    }
}

pub struct GradientRadialFocus {
    r: i32,
    fx: i32,
    fy: i32,
    r2: f64,
    fx2: f64,
    fy2: f64,
    mul: f64,
}

impl GradientFunc for GradientRadialFocus {
    fn calculate(&self, x: i32, y: i32, _: i32) -> i32 {
        let dx = x - self.fx;
        let dy = y - self.fy;
        let d2 = dx * self.fy - dy * self.fx;
        let d3 = self.r2 * (dx * dx + dy * dy) as f64 - (d2 * d2) as f64;
        iround((dx * self.fx + dy * self.fy) as f64 + d3.abs().sqrt() * self.mul)
    }
}

impl GradientRadialFocus {
    pub fn new() -> Self {
        let mut this = Self {
            r: 100 * GradientSubpixelScale::Scale as i32,
            fx: 0,
            fy: 0,
            r2: 0.0,
            fx2: 0.0,
            fy2: 0.0,
            mul: 0.0,
        };
        this.update_values();
        this
    }

    pub fn new_with_params(r: f64, fx: f64, fy: f64) -> Self {
        let mut this = Self {
            r: iround(r * GradientSubpixelScale::Scale as i32 as f64),
            fx: iround(fx * GradientSubpixelScale::Scale as i32 as f64),
            fy: iround(fy * GradientSubpixelScale::Scale as i32 as f64),
            r2: 0.0,
            fx2: 0.0,
            fy2: 0.0,
            mul: 0.0,
        };
        this.update_values();
        this
    }

    pub fn init(&mut self, r: f64, fx: f64, fy: f64) {
        self.r = iround(r * GradientSubpixelScale::Scale as i32 as f64);
        self.fx = iround(fx * GradientSubpixelScale::Scale as i32 as f64);
        self.fy = iround(fy * GradientSubpixelScale::Scale as i32 as f64);
        self.update_values();
    }

    pub fn radius(&self) -> f64 {
        self.r as f64 / GradientSubpixelScale::Scale as i32 as f64
    }

    pub fn focus_x(&self) -> f64 {
        self.fx as f64 / GradientSubpixelScale::Scale as i32 as f64
    }

    pub fn focus_y(&self) -> f64 {
        self.fy as f64 / GradientSubpixelScale::Scale as i32 as f64
    }

    fn update_values(&mut self) {
        self.r2 = self.r as f64 * self.r as f64;
        self.fx2 = self.fx as f64 * self.fx as f64;
        self.fy2 = self.fy as f64 * self.fy as f64;
        let mut d = self.r2 - (self.fx2 + self.fy2);
        if d == 0.0 {
            if self.fx != 0 {
                if self.fx < 0 {
                    self.fx += 1;
                } else {
                    self.fx -= 1;
                }
            }
            if self.fy != 0 {
                if self.fy < 0 {
                    self.fy += 1;
                } else {
                    self.fy -= 1;
                }
            }
            self.fx2 = self.fx as f64 * self.fx as f64;
            self.fy2 = self.fy as f64 * self.fy as f64;
            d = self.r2 - (self.fx2 + self.fy2);
        }
        self.mul = self.r as f64 / d;
    }
}

//=================================================gradient_repeat_adaptor
pub struct GradientRepeatAdaptor<GradientF: GradientFunc> {
    gradient: GradientF,
}

impl<GradientF: GradientFunc> GradientRepeatAdaptor<GradientF> {
    pub fn new(gradient: GradientF) -> Self {
        GradientRepeatAdaptor { gradient }
    }
}

impl<GradientF: GradientFunc> GradientFunc for GradientRepeatAdaptor<GradientF> {
    fn calculate(&self, x: i32, y: i32, d: i32) -> i32 {
        let mut ret = self.gradient.calculate(x, y, d) % d;
        if ret < 0 {
            ret += d;
        }
		ret
    }
}

//================================================gradient_reflect_adaptor
pub struct GradientReflectAdaptor<GradientF: GradientFunc> {
    gradient: GradientF,
}

impl<GradientF: GradientFunc> GradientReflectAdaptor<GradientF> {
    pub fn new(gradient: GradientF) -> Self {
        GradientReflectAdaptor { gradient }
    }
}

impl<GradientF: GradientFunc> GradientFunc for GradientReflectAdaptor<GradientF> {
    fn calculate(&self, x: i32, y: i32, d: i32) -> i32 {
        let d2 = d << 1;
        let mut ret = self.gradient.calculate(x, y, d) % d2;
        if ret < 0 {
            ret += d2;
        }
		if ret >= d {
            ret = d2 - ret;
        }
		ret
    }
}
