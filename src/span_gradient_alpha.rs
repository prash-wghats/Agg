use crate::span_gradient::GradientSubpixelScale;
use crate::{AlphaFn, Color, GradientFunc, Interpolator, SpanConverter};

use std::marker::PhantomData;

//======================================================SpanGradientAlpha
pub struct SpanGradientAlpha<
    'a,
    C: Color,
    I: Interpolator,
    GF: GradientFunc,
    A: AlphaFn<C::ValueType>,
> {
    interpolator: &'a mut I,
    gradient_function: &'a mut GF,
    alpha_function: &'a mut A,
    d1: i32,
    d2: i32,
    dum: PhantomData<C>,
}

impl<'a, C: Color, I: Interpolator, GF: GradientFunc, A: AlphaFn<C::ValueType>>
    SpanGradientAlpha<'a, C, I, GF, A>
{
    const DOWNSCALE_SHIFT: u32 = I::SUBPIXEL_SHIFT - GradientSubpixelScale::Shift as u32;
    pub fn new(
        interpolator: &'a mut I, gradient_function: &'a mut GF, alpha_function: &'a mut A, d1: f64,
        d2: f64,
    ) -> Self {
        SpanGradientAlpha {
            interpolator: interpolator,
            gradient_function: gradient_function,
            alpha_function: alpha_function,
            d1: (d1 * GradientSubpixelScale::Scale as u32 as f64) as i32,
            d2: (d2 * GradientSubpixelScale::Scale as u32 as f64) as i32,
            dum: PhantomData,
        }
    }

    pub fn d1(&self) -> f64 {
        self.d1 as f64 / GradientSubpixelScale::Scale as u32 as f64
    }

    pub fn d2(&self) -> f64 {
        self.d2 as f64 / GradientSubpixelScale::Scale as u32 as f64
    }

    pub fn interpolator_mut(&mut self) -> &mut I {
        self.interpolator
    }

    pub fn gradient_function_mut(&mut self) -> &mut GF {
        self.gradient_function
    }

    pub fn alpha_function_mut(&mut self) -> &mut A {
        self.alpha_function
    }

    pub fn d1_mut(&mut self, v: f64) {
        self.d1 = (v * GradientSubpixelScale::Scale as u32 as f64) as i32;
    }

    pub fn d2_mut(&mut self, v: f64) {
        self.d2 = (v * GradientSubpixelScale::Scale as u32 as f64) as i32;
    }
}

impl<'a, C: Color, I: Interpolator, GF: GradientFunc, A: AlphaFn<C::ValueType>> SpanConverter
    for SpanGradientAlpha<'a, C, I, GF, A>
{
    type C = C;
    fn prepare(&mut self) {}

    fn generate(&mut self, span: &mut [C], x: i32, y: i32, len: u32) {
        let (mut x, mut y) = (x, y);
        let mut dd = self.d2 - self.d1;
        if dd < 1 {
            dd = 1;
        }
        self.interpolator.begin(x as f64 + 0.5, y as f64 + 0.5, len);
        for i in 0..len {
            self.interpolator.coordinates(&mut x, &mut y);
            let d = self.gradient_function.calculate(
                x as i32 >> Self::DOWNSCALE_SHIFT,
                y as i32 >> Self::DOWNSCALE_SHIFT,
                self.d2,
            );
            let mut d = ((d - self.d1) * self.alpha_function.size() as i32) / dd;
            if d < 0 {
                d = 0;
            }
            if d >= self.alpha_function.size() as i32 {
                d = self.alpha_function.size() as i32 - 1;
            }
            *span[i as usize].a_mut() = self.alpha_function.get(d as u32);
            self.interpolator.next();
        }
    }
}

//=======================================================GradientAlphaX
pub struct GradientAlphaX<C: Color> {
    _marker: PhantomData<C>,
}

impl<C: Color> GradientAlphaX<C> {
    pub fn new() -> Self {
        GradientAlphaX {
            _marker: PhantomData,
        }
    }
    pub fn get(&self, x: C::ValueType) -> C::ValueType {
        x
    }
}

//====================================================GradientAlphaXU8
pub struct GradientAlphaXU8;

impl GradientAlphaXU8 {
    pub fn new() -> Self {
        GradientAlphaXU8 {}
    }
    pub fn get(&self, x: u8) -> u8 {
        x
    }
}

//==========================================GradientAlphaOneMinus
pub struct GradientAlphaOneMinus;

impl GradientAlphaOneMinus {
    pub fn new() -> Self {
        GradientAlphaOneMinus {}
    }
    pub fn get(&self, x: u8) -> u8 {
        255 - x
    }
}
