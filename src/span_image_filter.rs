use crate::basics::{iround, uround};
use crate::image_filters::{ImageFilterLut, ImageSubpixelScale};
use crate::span_interpolator_linear::SpanIpLinear;
use crate::trans_affine::TransAffine;
use crate::{ImageSrc, Interpolator, Transformer};
use std::ops::{Deref, DerefMut};
// NOT TESTED
//-------------------------------------------------------SpanImageFilter
pub struct SpanImageFilter<'a, S: ImageSrc, I: Interpolator> {
    pub source: &'a mut S,
    pub interpolator: &'a mut I,
    pub filter: ImageFilterLut,
    pub dx_dbl: f64,
    pub dy_dbl: f64,
    pub dx_int: u32,
    pub dy_int: u32,
}

impl<'a, S: ImageSrc, I: Interpolator> SpanImageFilter<'a, S, I> {
    pub fn new(source: &'a mut S, interpolator: &'a mut I, filter: ImageFilterLut) -> Self {
        SpanImageFilter {
            source: source,
            interpolator: interpolator,
            filter: filter,
            dx_dbl: 0.5,
            dy_dbl: 0.5,
            dx_int: ImageSubpixelScale::Scale as u32 / 2,
            dy_int: ImageSubpixelScale::Scale as u32 / 2,
        }
    }

    pub fn source(&self) -> &S {
        &self.source
    }
    pub fn source_mut(&mut self) -> &mut S {
        self.source
    }

    pub fn filter(&self) -> &ImageFilterLut {
        &self.filter
    }

	pub fn filter_mut(&mut self) -> &mut ImageFilterLut {
        &mut self.filter
    }

    pub fn filter_dx_int(&self) -> i32 {
        self.dx_int as i32
    }

    pub fn filter_dy_int(&self) -> i32 {
        self.dy_int as i32
    }

    pub fn filter_dx_dbl(&self) -> f64 {
        self.dx_dbl
    }

    pub fn filter_dy_dbl(&self) -> f64 {
        self.dy_dbl
    }

    pub fn interpolator(&mut self) -> &mut I {
        &mut self.interpolator
    }

    pub fn filter_offset(&mut self, dx: f64, dy: f64) {
        self.dx_dbl = dx;
        self.dy_dbl = dy;
        self.dx_int = iround(dx * ImageSubpixelScale::Scale as i32 as f64) as u32;
        self.dy_int = iround(dy * ImageSubpixelScale::Scale as i32 as f64) as u32;
    }

    pub fn filter_offset_d(&mut self, d: f64) {
        self.filter_offset(d, d);
    }

    pub fn prepare(&mut self) {}
}

//==============================================SpanImageResampleAffine
pub struct SpanImageResampleAffine<'a, S: ImageSrc> {
    pub base: SpanImageFilter<'a, S, SpanIpLinear<TransAffine>>,
    pub scale_limit: f64,
    pub blur_x: f64,
    pub blur_y: f64,
    pub rx: i32,
    pub ry: i32,
    pub rx_inv: i32,
    pub ry_inv: i32,
}


impl<'a, S: ImageSrc> Deref for SpanImageResampleAffine<'a, S> {
    type Target = SpanImageFilter<'a, S, SpanIpLinear<TransAffine>>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a, S: ImageSrc> DerefMut for SpanImageResampleAffine<'a, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<'a, S: ImageSrc> SpanImageResampleAffine<'a, S> {
    pub fn new(
        source: &'a mut S, interpolator: &'a mut SpanIpLinear<TransAffine>, filter: ImageFilterLut,
    ) -> Self {
        Self {
            base: SpanImageFilter::new(source, interpolator, filter),
            scale_limit: 200.0,
            blur_x: 1.0,
            blur_y: 1.0,
            rx: 0,
            ry: 0,
            rx_inv: 0,
            ry_inv: 0,
        }
    }

    pub fn scale_limit(&self) -> u32 {
        self.scale_limit.round() as u32
    }

    pub fn scale_limit_f(&self) -> f64 {
        self.scale_limit
    }

    pub fn set_scale_limit(&mut self, v: u32) {
        self.scale_limit = v as f64;
    }

    pub fn blur_x(&self) -> f64 {
        self.blur_x
    }

    pub fn blur_y(&self) -> f64 {
        self.blur_y
    }

    pub fn set_blur_x(&mut self, v: f64) {
        self.blur_x = v;
    }

    pub fn set_blur_y(&mut self, v: f64) {
        self.blur_y = v;
    }

    pub fn set_blur(&mut self, v: f64) {
        self.blur_x = v;
        self.blur_y = v;
    }

    pub fn prepare(&mut self) {
        let (mut scale_x, mut scale_y) = (0., 0.);
        self.base
            .interpolator()
            .transformer()
            .scaling_abs(&mut scale_x, &mut scale_y);

        if scale_x * scale_y > self.scale_limit {
            scale_x = scale_x * self.scale_limit / (scale_x * scale_y);
            scale_y = scale_y * self.scale_limit / (scale_x * scale_y);
        }

        if scale_x < 1.0 {
            scale_x = 1.0;
        }
        if scale_y < 1.0 {
            scale_y = 1.0;
        }

        if scale_x > self.scale_limit {
            scale_x = self.scale_limit;
        }
        if scale_y > self.scale_limit {
            scale_y = self.scale_limit;
        }

        scale_x *= self.blur_x;
        scale_y *= self.blur_y;

        if scale_x < 1.0 {
            scale_x = 1.0;
        }
        if scale_y < 1.0 {
            scale_y = 1.0;
        }

        self.rx = uround(scale_x * ImageSubpixelScale::Scale as i32 as f64);
        self.rx_inv = uround(1.0 / scale_x * ImageSubpixelScale::Scale as i32 as f64);

        self.ry = uround(scale_y * ImageSubpixelScale::Scale as i32 as f64);
        self.ry_inv = uround(1.0 / scale_y * ImageSubpixelScale::Scale as i32 as f64);
    }
}

//=====================================================SpanImageResample
pub struct SpanImageResample<'a, S: ImageSrc, I: Interpolator> {
    pub base: SpanImageFilter<'a, S, I>,
    pub scale_limit: i32,
    pub blur_x: i32,
    pub blur_y: i32,
}

impl<'a, S: ImageSrc, I: Interpolator> Deref for SpanImageResample<'a, S, I> {
    type Target = SpanImageFilter<'a, S, I>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a, S: ImageSrc, I: Interpolator> DerefMut for SpanImageResample<'a, S, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<'a, S: ImageSrc, I: Interpolator> SpanImageResample<'a, S, I> {
    pub fn new(source: &'a mut S, interpolator: &'a mut I, filter: ImageFilterLut) -> Self {
        SpanImageResample {
            base: SpanImageFilter::new(source, interpolator, filter),
            scale_limit: 20,
            blur_x: ImageSubpixelScale::Scale as i32,
            blur_y: ImageSubpixelScale::Scale as i32,
        }
    }

    pub fn scale_limit(&self) -> i32 {
        self.scale_limit
    }

    pub fn set_scale_limit(&mut self, v: i32) {
        self.scale_limit = v;
    }

    pub fn blur_x(&self) -> f64 {
        self.blur_x as f64 / ImageSubpixelScale::Scale as i32 as f64
    }

    pub fn blur_y(&self) -> f64 {
        self.blur_y as f64 / ImageSubpixelScale::Scale as i32 as f64
    }

    pub fn set_blur_x(&mut self, v: f64) {
        self.blur_x = v.round() as i32 * ImageSubpixelScale::Scale as i32;
    }

    pub fn set_blur_y(&mut self, v: f64) {
        self.blur_y = v.round() as i32 * ImageSubpixelScale::Scale as i32;
    }

    pub fn set_blur(&mut self, v: f64) {
        self.blur_x = v.round() as i32 * ImageSubpixelScale::Scale as i32;
        self.blur_y = v.round() as i32 * ImageSubpixelScale::Scale as i32;
    }

    pub fn adjust_scale(&self, rx: &mut i32, ry: &mut i32) {
        if *rx < ImageSubpixelScale::Scale as i32 {
            *rx = ImageSubpixelScale::Scale as i32;
        }
        if *ry < ImageSubpixelScale::Scale as i32 {
            *ry = ImageSubpixelScale::Scale as i32;
        }
        if *rx > ImageSubpixelScale::Scale as i32 * self.scale_limit {
            *rx = ImageSubpixelScale::Scale as i32 * self.scale_limit;
        }
        if *ry > ImageSubpixelScale::Scale as i32 * self.scale_limit {
            *ry = ImageSubpixelScale::Scale as i32 * self.scale_limit;
        }
        *rx = (*rx * self.blur_x) >> ImageSubpixelScale::Shift as i32;
        *ry = (*ry * self.blur_y) >> ImageSubpixelScale::Shift as i32;
        if *rx < ImageSubpixelScale::Scale as i32 {
            *rx = ImageSubpixelScale::Scale as i32;
        }
        if *ry < ImageSubpixelScale::Scale as i32 {
            *ry = ImageSubpixelScale::Scale as i32;
        }
    }
}
