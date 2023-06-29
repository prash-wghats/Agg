use crate::image_filters::{ImageFilterLut, ImageFilterScale, ImageSubpixelScale};
use crate::span_image_filter::{SpanImageFilter, SpanImageResample, SpanImageResampleAffine};
use crate::span_interpolator_linear::SpanIpLinear;
use crate::trans_affine::TransAffine;

use crate::{
    AggPrimitive, Args, Color, ImageAccessorRgb, Interpolator, Order, PixFmt, RgbArgs,
    SpanGenerator,
};

macro_rules! from_u32 {
    ($v:expr) => {
        AggPrimitive::from_u32($v)
    };
}

macro_rules! from_i32 {
    ($v:expr) => {
        AggPrimitive::from_i32($v)
    };
}

// NOT TESTED

//===============================================SpanImageFilterRgbNn
pub struct SpanImageFilterRgbNn<'a, S: ImageAccessorRgb, I: Interpolator> {
    base: SpanImageFilter<'a, S, I>,
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanImageFilterRgbNn<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I) -> Self {
        SpanImageFilterRgbNn {
            base: SpanImageFilter::new(src, inter, ImageFilterLut::new()),
        }
    }

    pub fn base(&self) -> &SpanImageFilter<'a, S, I> {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpanImageFilter<'a, S, I> {
        &mut self.base
    }
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanGenerator for SpanImageFilterRgbNn<'a, S, I> {
    type C = S::ColorType;

    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        self.base.interpolator.begin(
            x as f64 + self.base.filter_dx_dbl(),
            y as f64 + self.base.filter_dy_dbl(),
            len,
        );
        for i in 0..len {
            let mut x = 0;
            let mut y = 0;
            self.base.interpolator.coordinates(&mut x, &mut y);
            let fg_ptr = self
                .base
                .source
                .span(
                    x >> ImageSubpixelScale::Shift as u32,
                    y >> ImageSubpixelScale::Shift as u32,
                    1,
                )
                .as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            unsafe {
                *span[i as usize].r_mut() = *fg_ptr.offset(S::OrderType::R as isize);
                *span[i as usize].g_mut() = *fg_ptr.offset(S::OrderType::G as isize);
                *span[i as usize].b_mut() = *fg_ptr.offset(S::OrderType::B as isize);
                *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);
            }
            self.base.interpolator.next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//==========================================SpanImageFilterRgbBilinear
pub struct SpanImageFilterRgbBilinear<'a, S: ImageAccessorRgb, I: Interpolator> {
    base: SpanImageFilter<'a, S, I>,
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanImageFilterRgbBilinear<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I) -> Self {
        SpanImageFilterRgbBilinear {
            base: SpanImageFilter::new(src, inter, ImageFilterLut::new()),
        }
    }

    pub fn base(&self) -> &SpanImageFilter<'a, S, I> {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpanImageFilter<'a, S, I> {
        &mut self.base
    }
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanGenerator
    for SpanImageFilterRgbBilinear<'a, S, I>
{
    type C = S::ColorType;

    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        self.base.interpolator.begin(
            x as f64 + self.base.filter_dx_dbl(),
            y as f64 + self.base.filter_dy_dbl(),
            len,
        );
        let mut fg = [0u32; 3];

        for i in 0..len {
            let mut x_hr = 0;
            let mut y_hr = 0;
            self.base.interpolator.coordinates(&mut x_hr, &mut y_hr);
            x_hr -= self.base.filter_dx_int();
            y_hr -= self.base.filter_dy_int();
            let x_lr = x_hr >> ImageSubpixelScale::Shift as u32;
            let y_lr = y_hr >> ImageSubpixelScale::Shift as u32;

            let mut weight: u32;

            fg[0] =
                (ImageSubpixelScale::Scale as i32 * ImageSubpixelScale::Scale as i32) as u32 / 2;
            fg[1] = fg[0];
            fg[2] = fg[0];

            x_hr &= ImageSubpixelScale::Mask as i32;
            y_hr &= ImageSubpixelScale::Mask as i32;

            let mut fg_ptr = self.base.source_mut().span(x_lr, y_lr, 2).as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((ImageSubpixelScale::Scale as i32 - x_hr)
                * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
            unsafe {
                fg[0] += weight * (*fg_ptr.offset(0)).into_u32();
                fg[1] += weight * (*fg_ptr.offset(1)).into_u32();
                fg[2] += weight * (*fg_ptr.offset(2)).into_u32();
            }

            fg_ptr = self.base.source_mut().next_x().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = (x_hr * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
            unsafe {
                fg[0] += weight * (*fg_ptr.offset(0)).into_u32();
                fg[1] += weight * (*fg_ptr.offset(1)).into_u32();
                fg[2] += weight * (*fg_ptr.offset(2)).into_u32();
            }

            fg_ptr = self.base.source_mut().next_y().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((ImageSubpixelScale::Scale as i32 - x_hr) * y_hr) as u32;
            unsafe {
                fg[0] += weight * (*fg_ptr.offset(0)).into_u32();
                fg[1] += weight * (*fg_ptr.offset(1)).into_u32();
                fg[2] += weight * (*fg_ptr.offset(2)).into_u32();
            }

            fg_ptr = self.base.source_mut().next_x().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = (x_hr * y_hr) as u32;
            unsafe {
                fg[0] += weight * (*fg_ptr.offset(0)).into_u32();
                fg[1] += weight * (*fg_ptr.offset(1)).into_u32();
                fg[2] += weight * (*fg_ptr.offset(2)).into_u32();
            }

            *span[i as usize].r_mut() =
                from_u32!(fg[S::OrderType::R as usize] >> (ImageSubpixelScale::Shift as u32 * 2));
            *span[i as usize].g_mut() =
                from_u32!(fg[S::OrderType::G as usize] >> (ImageSubpixelScale::Shift as u32 * 2));
            *span[i as usize].b_mut() =
                from_u32!(fg[S::OrderType::B as usize] >> (ImageSubpixelScale::Shift as u32 * 2));
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator().next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//====================================SpanImageFilterRgbBilinearClip
pub struct SpanImageFilterRgbBilinearClip<'a, C: Color + RgbArgs, S: PixFmt<C = C>, I: Interpolator>
{
    base: SpanImageFilter<'a, S, I>,
    back_color: S::C,
}

impl<'a, C: Color + RgbArgs, S: PixFmt<C = C>, I: Interpolator>
    SpanImageFilterRgbBilinearClip<'a, C, S, I>
{
    pub const BASE_SHIFT: u32 = S::C::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::C::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I, back_color: S::C) -> Self {
        SpanImageFilterRgbBilinearClip {
            base: SpanImageFilter::new(src, inter, ImageFilterLut::new()),
            back_color: back_color,
        }
    }

    pub fn base(&self) -> &SpanImageFilter<'a, S, I> {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpanImageFilter<'a, S, I> {
        &mut self.base
    }

    pub fn background_color(&self) -> &S::C {
        &self.back_color
    }

    pub fn set_background_color(&mut self, v: S::C) {
        self.back_color = v;
    }
}

impl<'a, C: Color + RgbArgs, S: PixFmt<C = C>, I: Interpolator> SpanGenerator
    for SpanImageFilterRgbBilinearClip<'a, C, S, I>
{
    type C = S::C;

    fn generate(&mut self, span: &mut [S::C], x: i32, y: i32, len: u32) {
        self.base.interpolator.begin(
            x as f64 + self.base.filter_dx_dbl(),
            y as f64 + self.base.filter_dy_dbl(),
            len,
        );

        let mut fg: [u32; 4] = [0; 4];
        let mut src_alpha: u32; // = 0;
        let back_r = self.back_color.r();
        let back_g = self.back_color.g();
        let back_b = self.back_color.b();
        let back_a = self.back_color.a();

        let maxx = self.base.source().width() as i32 - 1;
        let maxy = self.base.source().height() as i32 - 1;
        let mut off;

        for i in 0..len {
            let mut x_hr = 0;
            let mut y_hr = 0;

            self.base.interpolator.coordinates(&mut x_hr, &mut y_hr);

            x_hr -= self.base.filter_dx_int();
            y_hr -= self.base.filter_dy_int();

            let mut x_lr = x_hr >> ImageSubpixelScale::Shift as u32;
            let mut y_lr = y_hr >> ImageSubpixelScale::Shift as u32;

            let mut weight: u32;

            if x_lr >= 0 && y_lr >= 0 && x_lr < maxx && y_lr < maxy {
                fg[0] = (ImageSubpixelScale::Scale as i32 * ImageSubpixelScale::Scale as i32)
                    as u32
                    / 2;
                fg[1] = fg[0];
                fg[2] = fg[0];

                x_hr &= ImageSubpixelScale::Mask as i32;
                y_hr &= ImageSubpixelScale::Mask as i32;

                let fg_ptr = self.base.source.row(y_lr);

                off = (x_lr * 3) as usize;

                weight = ((ImageSubpixelScale::Scale as i32 - x_hr)
                    * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
                fg[0] += weight * fg_ptr[off].into_u32();
                fg[1] += weight * fg_ptr[off + 1].into_u32();
                fg[2] += weight * fg_ptr[off + 2].into_u32();

                weight = (x_hr * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
                fg[0] += weight * fg_ptr[off + 3].into_u32();
                fg[1] += weight * fg_ptr[off + 4].into_u32();
                fg[2] += weight * fg_ptr[off + 5].into_u32();

                y_lr += 1;
                let fg_ptr = self.base.source.row(y_lr);

                off = (x_lr * 3) as usize;

                weight = ((ImageSubpixelScale::Scale as i32 - x_hr) * y_hr) as u32;
                fg[0] += weight * fg_ptr[off].into_u32();
                fg[1] += weight * fg_ptr[off + 1].into_u32();
                fg[2] += weight * fg_ptr[off + 2].into_u32();

                weight = (x_hr * y_hr) as u32;
                fg[0] += weight * fg_ptr[off + 3].into_u32();
                fg[1] += weight * fg_ptr[off + 4].into_u32();
                fg[2] += weight * fg_ptr[off + 5].into_u32();

                fg[0] >>= ImageSubpixelScale::Shift as u32 * 2;
                fg[1] >>= ImageSubpixelScale::Shift as u32 * 2;
                fg[2] >>= ImageSubpixelScale::Shift as u32 * 2;
                src_alpha = Self::BASE_MASK;
            } else {
                if x_lr < -1 || y_lr < -1 || x_lr > maxx || y_lr > maxy {
                    fg[S::O::R as usize] = back_r.into_u32();
                    fg[S::O::G as usize] = back_g.into_u32();
                    fg[S::O::B as usize] = back_b.into_u32();
                    src_alpha = back_a.into_u32();
                } else {
                    fg[0] = (ImageSubpixelScale::Scale as i32 * ImageSubpixelScale::Scale as i32)
                        as u32
                        / 2;
                    fg[1] = fg[0];
                    fg[2] = fg[0];
                    src_alpha = fg[0];

                    x_hr &= ImageSubpixelScale::Mask as i32;
                    y_hr &= ImageSubpixelScale::Mask as i32;

                    weight = ((ImageSubpixelScale::Scale as i32 - x_hr)
                        * (ImageSubpixelScale::Scale as i32 - y_hr))
                        as u32;
                    if x_lr >= 0 && y_lr >= 0 && x_lr <= maxx && y_lr <= maxy {
                        let fg_ptr = self.base.source.row(y_lr);

                        off = (x_lr * 3) as usize;

                        fg[0] += weight * fg_ptr[off].into_u32();
                        fg[1] += weight * fg_ptr[off + 1].into_u32();
                        fg[2] += weight * fg_ptr[off + 2].into_u32();
                        src_alpha += weight * Self::BASE_MASK;
                    } else {
                        fg[S::O::R as usize] += back_r.into_u32() * weight;
                        fg[S::O::G as usize] += back_g.into_u32() * weight;
                        fg[S::O::B as usize] += back_b.into_u32() * weight;
                        src_alpha += back_a.into_u32() * weight;
                    }

                    x_lr += 1;

                    weight = (x_hr * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
                    if x_lr >= 0 && y_lr >= 0 && x_lr <= maxx && y_lr <= maxy {
                        let fg_ptr = self.base.source.row(y_lr);

                        off = (x_lr * 3) as usize;

                        fg[0] += weight * fg_ptr[off].into_u32();
                        fg[1] += weight * fg_ptr[off + 1].into_u32();
                        fg[2] += weight * fg_ptr[off + 2].into_u32();
                        src_alpha += weight * Self::BASE_MASK;
                    } else {
                        fg[S::O::R as usize] += back_r.into_u32() * weight;
                        fg[S::O::G as usize] += back_g.into_u32() * weight;
                        fg[S::O::B as usize] += back_b.into_u32() * weight;
                        src_alpha += back_a.into_u32() * weight;
                    }

                    x_lr -= 1;
                    y_lr += 1;

                    weight = ((ImageSubpixelScale::Scale as i32 - x_hr) * y_hr) as u32;
                    if x_lr >= 0 && y_lr >= 0 && x_lr <= maxx && y_lr <= maxy {
                        let fg_ptr = self.base.source.row(y_lr);

                        off = (x_lr * 3) as usize;

                        fg[0] += weight * fg_ptr[off].into_u32();
                        fg[1] += weight * fg_ptr[off + 1].into_u32();
                        fg[2] += weight * fg_ptr[off + 2].into_u32();
                        src_alpha += weight * Self::BASE_MASK;
                    } else {
                        fg[S::O::R as usize] += back_r.into_u32() * weight;
                        fg[S::O::G as usize] += back_g.into_u32() * weight;
                        fg[S::O::B as usize] += back_b.into_u32() * weight;
                        src_alpha += back_a.into_u32() * weight;
                    }

                    x_lr += 1;

                    weight = (x_hr * y_hr) as u32;
                    if x_lr >= 0 && y_lr >= 0 && x_lr <= maxx && y_lr <= maxy {
                        let fg_ptr = self.base.source.row(y_lr);

                        off = (x_lr * 3) as usize;

                        fg[0] += weight * fg_ptr[off].into_u32();
                        fg[1] += weight * fg_ptr[off + 1].into_u32();
                        fg[2] += weight * fg_ptr[off + 2].into_u32();
                        src_alpha += weight * Self::BASE_MASK;
                    } else {
                        fg[S::O::R as usize] += back_r.into_u32() * weight;
                        fg[S::O::G as usize] += back_g.into_u32() * weight;
                        fg[S::O::B as usize] += back_b.into_u32() * weight;
                        src_alpha += back_a.into_u32() * weight;
                    }

                    fg[0] >>= ImageSubpixelScale::Shift as u32 * 2;
                    fg[1] >>= ImageSubpixelScale::Shift as u32 * 2;
                    fg[2] >>= ImageSubpixelScale::Shift as u32 * 2;
                    src_alpha >>= ImageSubpixelScale::Shift as u32 * 2;
                }
            }
            *span[i as usize].r_mut() = from_u32!(fg[S::O::R as usize]);
            *span[i as usize].g_mut() = from_u32!(fg[S::O::G as usize]);
            *span[i as usize].b_mut() = from_u32!(fg[S::O::B as usize]);
            *span[i as usize].a_mut() = from_u32!(src_alpha);

            self.base.interpolator().next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//==============================================SpanImageFilterRgb2x2
pub struct SpanImageFilterRgb2x2<'a, S: ImageAccessorRgb, I: Interpolator> {
    base: SpanImageFilter<'a, S, I>,
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanImageFilterRgb2x2<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I, filter: ImageFilterLut) -> Self {
        SpanImageFilterRgb2x2 {
            base: SpanImageFilter::new(src, inter, filter),
        }
    }

    pub fn base(&self) -> &SpanImageFilter<'a, S, I> {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpanImageFilter<'a, S, I> {
        &mut self.base
    }
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanGenerator for SpanImageFilterRgb2x2<'a, S, I> {
    type C = S::ColorType;

    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let mut fg: [u32; 3] = [0; 3];

        let diameter = self.base.filter.diameter();
        let offset = ((diameter / 2 - 1) << ImageSubpixelScale::Shift as i32) as usize;

        self.base.interpolator.begin(
            x as f64 + self.base.filter_dx_dbl(),
            y as f64 + self.base.filter_dy_dbl(),
            len,
        );

        for i in 0..len {
            let mut x_hr = 0;
            let mut y_hr = 0;
            self.base.interpolator.coordinates(&mut x_hr, &mut y_hr);

            x_hr = x_hr - self.base.filter_dx_int();
            y_hr = y_hr - self.base.filter_dy_int();

            let x_lr = x_hr >> ImageSubpixelScale::Shift as i32;
            let y_lr = y_hr >> ImageSubpixelScale::Shift as i32;

            let mut weight: u32;
            fg[0] = ImageFilterScale::Scale as u32 / 2;
            fg[1] = fg[0];
            fg[2] = fg[0];

            let x_hr = x_hr & ImageSubpixelScale::Mask as i32;
            let y_hr = y_hr & ImageSubpixelScale::Mask as i32;

            let mut fg_ptr = self.base.source.span(x_lr, y_lr, 2).as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((self.base.filter.weight_array()
                [(x_hr + ImageSubpixelScale::Scale as i32) as usize + offset]
                as u32
                * self.base.filter.weight_array()
                    [(y_hr + ImageSubpixelScale::Scale as i32) as usize + offset]
                    as u32)
                + ImageFilterScale::Scale as u32 / 2)
                >> ImageFilterScale::Shift as u32;
            unsafe {
                fg[0] += weight * (*fg_ptr.offset(0)).into_u32();
                fg[1] += weight * (*fg_ptr.offset(1)).into_u32();
                fg[2] += weight * (*fg_ptr.offset(2)).into_u32();
            }

            fg_ptr = self.base.source.next_x().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((self.base.filter.weight_array()[x_hr as usize + offset] as u32
                * self.base.filter.weight_array()
                    [(y_hr + ImageSubpixelScale::Scale as i32) as usize + offset]
                    as u32)
                + ImageFilterScale::Scale as u32 / 2)
                >> ImageFilterScale::Shift as u32;
            unsafe {
                fg[0] += weight * (*fg_ptr.offset(0)).into_u32();
                fg[1] += weight * (*fg_ptr.offset(1)).into_u32();
                fg[2] += weight * (*fg_ptr.offset(2)).into_u32();
            }

            fg_ptr = self.base.source.next_y().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((self.base.filter.weight_array()
                [(x_hr + ImageSubpixelScale::Scale as i32) as usize + offset]
                as u32
                * self.base.filter.weight_array()[y_hr as usize + offset] as u32)
                + ImageFilterScale::Scale as u32 / 2)
                >> ImageFilterScale::Shift as u32;
            unsafe {
                fg[0] += weight * (*fg_ptr.offset(0)).into_u32();
                fg[1] += weight * (*fg_ptr.offset(1)).into_u32();
                fg[2] += weight * (*fg_ptr.offset(2)).into_u32();
            }

            fg_ptr = self.base.source.next_x().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((self.base.filter.weight_array()[x_hr as usize + offset] as u32
                * self.base.filter.weight_array()[y_hr as usize + offset] as u32)
                + ImageFilterScale::Scale as u32 / 2)
                >> ImageFilterScale::Shift as u32;
            unsafe {
                fg[0] += weight * (*fg_ptr.offset(0)).into_u32();
                fg[1] += weight * (*fg_ptr.offset(1)).into_u32();
                fg[2] += weight * (*fg_ptr.offset(2)).into_u32();
            }

            fg[0] >>= ImageFilterScale::Shift as i32;
            fg[1] >>= ImageFilterScale::Shift as i32;
            fg[2] >>= ImageFilterScale::Shift as i32;

            if fg[S::OrderType::R] > Self::BASE_MASK {
                fg[S::OrderType::R] = Self::BASE_MASK;
            }
            if fg[S::OrderType::G] > Self::BASE_MASK {
                fg[S::OrderType::G] = Self::BASE_MASK;
            }
            if fg[S::OrderType::B] > Self::BASE_MASK {
                fg[S::OrderType::B] = Self::BASE_MASK;
            }

            *span[i as usize].r_mut() = from_u32!(fg[S::OrderType::R as usize]);
            *span[i as usize].g_mut() = from_u32!(fg[S::OrderType::G as usize]);
            *span[i as usize].b_mut() = from_u32!(fg[S::OrderType::B as usize]);
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator().next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//==================================================SpanImageFilterRgb
pub struct SpanImageFilterRgb<'a, S: ImageAccessorRgb, I: Interpolator> {
    base: SpanImageFilter<'a, S, I>,
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanImageFilterRgb<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I, filter: ImageFilterLut) -> Self {
        SpanImageFilterRgb {
            base: SpanImageFilter::new(src, inter, filter),
        }
    }

    pub fn base(&self) -> &SpanImageFilter<'a, S, I> {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpanImageFilter<'a, S, I> {
        &mut self.base
    }
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanGenerator for SpanImageFilterRgb<'a, S, I> {
    type C = S::ColorType;
    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let (mut x, mut y) = (x, y);
        self.base.interpolator.begin(
            x as f64 + self.base.filter_dx_dbl(),
            y as f64 + self.base.filter_dy_dbl(),
            len,
        );

        let mut fg = [0; 3];
        let mut fg_ptr: *mut <S::ColorType as Args>::ValueType;

        let diameter = self.base.filter.diameter();
        let start = self.base.filter.start();
        let weight_array = self.base.filter.weight_array();

        let mut x_count;
        let mut weight_y;

        for i in 0..len {
            self.base.interpolator.coordinates(&mut x, &mut y);

            x -= self.base.filter_dx_int();
            y -= self.base.filter_dy_int();

            let mut x_hr = x;
            let mut y_hr = y;

            let x_lr = x_hr >> ImageSubpixelScale::Shift as i32;
            let y_lr = y_hr >> ImageSubpixelScale::Shift as i32;

            fg[0] = ImageFilterScale::Scale as i32 / 2;
            fg[1] = fg[0];
            fg[2] = fg[0];

            let x_fract = x_hr & ImageSubpixelScale::Mask as i32;
            let mut y_count = diameter;

            y_hr = ImageSubpixelScale::Mask as i32 - (y_hr & ImageSubpixelScale::Mask as i32);
            fg_ptr = self
                .base
                .source
                .span(x_lr + start, y_lr + start, diameter)
                .as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            loop {
                x_count = diameter;
                weight_y = weight_array[y_hr as usize];
                x_hr = ImageSubpixelScale::Mask as i32 - x_fract;
                loop {
                    let weight = ((weight_y as i32 * weight_array[x_hr as usize] as i32)
                        + ImageFilterScale::Scale as i32 / 2)
                        >> ImageFilterScale::Shift as i32;

                    unsafe {
                        fg[0] += weight * (*fg_ptr.offset(0)).into_i32();
                        fg[1] += weight * (*fg_ptr.offset(1)).into_i32();
                        fg[2] += weight * (*fg_ptr.offset(2)).into_i32();
                    }

                    x_count -= 1;
                    if x_count == 0 {
                        break;
                    }
                    x_hr += ImageSubpixelScale::Scale as i32;
                    fg_ptr = self.base.source.next_x().as_ptr() as *const u8
                        as *mut <S::ColorType as Args>::ValueType;
                }

                y_count -= 1;
                if y_count == 0 {
                    break;
                }
                y_hr += ImageSubpixelScale::Scale as i32;
                fg_ptr = self.base.source.next_y().as_ptr() as *const u8
                    as *mut <S::ColorType as Args>::ValueType;
            }

            fg[0] >>= ImageFilterScale::Shift as i32;
            fg[1] >>= ImageFilterScale::Shift as i32;
            fg[2] >>= ImageFilterScale::Shift as i32;

            if fg[0] < 0 {
                fg[0] = 0;
            }
            if fg[1] < 0 {
                fg[1] = 0;
            }
            if fg[2] < 0 {
                fg[2] = 0;
            }

            if fg[S::OrderType::R] > Self::BASE_MASK as i32 {
                fg[S::OrderType::R] = Self::BASE_MASK as i32;
            }
            if fg[S::OrderType::G] > Self::BASE_MASK as i32 {
                fg[S::OrderType::G] = Self::BASE_MASK as i32;
            }
            if fg[S::OrderType::B] > Self::BASE_MASK as i32 {
                fg[S::OrderType::B] = Self::BASE_MASK as i32;
            }

            *span[i as usize].r_mut() = from_i32!(fg[S::OrderType::R as usize]);
            *span[i as usize].g_mut() = from_i32!(fg[S::OrderType::G as usize]);
            *span[i as usize].b_mut() = from_i32!(fg[S::OrderType::B as usize]);
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator.next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//========================================SpanImageResampleRgbAffine
pub struct SpanImageResampleRgbAffine<'a, S: ImageAccessorRgb> {
    base: SpanImageResampleAffine<'a, S>,
}

impl<'a, S: ImageAccessorRgb> SpanImageResampleRgbAffine<'a, S> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;
    const DOWNSCALE_SHIFT: u32 = ImageFilterScale::Shift as u32;
    pub fn new(
        src: &'a mut S, inter: &'a mut SpanIpLinear<TransAffine>, filter: ImageFilterLut,
    ) -> Self {
        SpanImageResampleRgbAffine {
            base: SpanImageResampleAffine::new(src, inter, filter),
        }
    }

    pub fn base(&self) -> &SpanImageResampleAffine<'a, S> {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpanImageResampleAffine<'a, S> {
        &mut self.base
    }
}

impl<'a, S: ImageAccessorRgb> SpanGenerator for SpanImageResampleRgbAffine<'a, S> {
    type C = S::ColorType;
    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let (mut x, mut y) = (x, y);
        let (dx, dy) = (self.base.filter_dx_dbl(), self.base.filter_dy_dbl());
        self.base
            .interpolator
            .begin(x as f64 + dx, y as f64 + dy, len);

        let mut fg = [0; 3];

        let diameter = self.base.filter.diameter() as i32;
        let filter_scale = diameter << ImageSubpixelScale::Shift as i32;
        let radius_x = (diameter * self.base.rx) >> 1;
        let radius_y = (diameter * self.base.ry) >> 1;
        let len_x_lr = (diameter * self.base.rx + ImageSubpixelScale::Mask as i32)
            >> ImageSubpixelScale::Shift as i32;

        for i in 0..len {
            self.base.interpolator.coordinates(&mut x, &mut y);

            x += self.base.filter_dx_int() - radius_x;
            y += self.base.filter_dy_int() - radius_y;

            fg[0] = ImageFilterScale::Scale as i32 / 2;
            fg[1] = fg[0];
            fg[2] = fg[0];

            let y_lr = y >> ImageSubpixelScale::Shift as i32;
            let mut y_hr = ((ImageSubpixelScale::Mask as i32
                - (y & ImageSubpixelScale::Mask as i32))
                * self.base.ry_inv)
                >> ImageSubpixelScale::Shift as i32;
            let mut total_weight = 0;
            let x_lr = x >> ImageSubpixelScale::Shift as i32;
            let mut x_hr = ((ImageSubpixelScale::Mask as i32
                - (x & ImageSubpixelScale::Mask as i32))
                * self.base.rx_inv)
                >> ImageSubpixelScale::Shift as i32;

            let x_hr2 = x_hr;
            let mut fg_ptr = self.base.source.span(x_lr, y_lr, len_x_lr as u32).as_ptr()
                as *const u8 as *mut <S::ColorType as Args>::ValueType;
            loop {
                let weight_y = self.base.filter.weight_array()[y_hr as usize];
                x_hr = x_hr2;
                loop {
                    let weight = ((weight_y as i32
                        * self.base.filter.weight_array()[x_hr as usize] as i32)
                        + ImageFilterScale::Scale as i32 / 2)
                        >> Self::DOWNSCALE_SHIFT;

                    unsafe {
                        fg[0] += weight * (*fg_ptr.offset(0)).into_i32();
                        fg[1] += weight * (*fg_ptr.offset(1)).into_i32();
                        fg[2] += weight * (*fg_ptr.offset(2)).into_i32();
                    }
                    total_weight += weight;
                    x_hr += self.base.rx_inv;
                    if x_hr >= filter_scale {
                        break;
                    }
                    fg_ptr = self.base.source.next_x().as_ptr() as *const u8
                        as *mut <S::ColorType as Args>::ValueType;
                }
                y_hr += self.base.ry_inv;
                if y_hr >= filter_scale {
                    break;
                }
                fg_ptr = self.base.source.next_y().as_ptr() as *const u8
                    as *mut <S::ColorType as Args>::ValueType;
            }

            fg[0] /= total_weight;
            fg[1] /= total_weight;
            fg[2] /= total_weight;

            if fg[0] < 0 {
                fg[0] = 0;
            }
            if fg[1] < 0 {
                fg[1] = 0;
            }
            if fg[2] < 0 {
                fg[2] = 0;
            }

            if fg[S::OrderType::R] > Self::BASE_MASK as i32 {
                fg[S::OrderType::R] = Self::BASE_MASK as i32;
            }
            if fg[S::OrderType::G] > Self::BASE_MASK as i32 {
                fg[S::OrderType::G] = Self::BASE_MASK as i32;
            }
            if fg[S::OrderType::B] > Self::BASE_MASK as i32 {
                fg[S::OrderType::B] = Self::BASE_MASK as i32;
            }

            *span[i as usize].r_mut() = from_i32!(fg[S::OrderType::R as usize]);
            *span[i as usize].g_mut() = from_i32!(fg[S::OrderType::G as usize]);
            *span[i as usize].b_mut() = from_i32!(fg[S::OrderType::B as usize]);
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator.next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//==============================================SpanImageResampleRgb
pub struct SpanImageResampleRgb<'a, S: ImageAccessorRgb, I: Interpolator> {
    base: SpanImageResample<'a, S, I>,
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanImageResampleRgb<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;
    const DOWNSCALE_SHIFT: u32 = ImageFilterScale::Shift as i32 as u32;
    pub fn new(src: &'a mut S, inter: &'a mut I, filter: ImageFilterLut) -> Self {
        SpanImageResampleRgb {
            base: SpanImageResample::new(src, inter, filter),
        }
    }

    pub fn base(&self) -> &SpanImageResample<'a, S, I> {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpanImageResample<'a, S, I> {
        &mut self.base
    }
}

impl<'a, S: ImageAccessorRgb, I: Interpolator> SpanGenerator for SpanImageResampleRgb<'a, S, I> {
    type C = S::ColorType;

    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let diameter = self.base.filter.diameter() as i32;
        let filter_scale = diameter << ImageSubpixelScale::Shift as i32;

        let (dx, dy) = (self.base.filter_dx_dbl(), self.base.filter_dy_dbl());
        self.base
            .interpolator
            .begin(x as f64 + dx, y as f64 + dy, len as u32);

        let mut fg = [0; 3];
        let mut x = x;
        let mut y = y;
        let len = span.len();
        for i in 0..len {
            let mut rx = 0;
            let mut ry = 0;
            let rx_inv; // = ImageSubpixelScale::Scale as i32;
            let ry_inv; // = ImageSubpixelScale::Scale as i32;
            self.base.interpolator.coordinates(&mut x, &mut y);
            self.base.interpolator.local_scale(&mut rx, &mut ry);
            self.base.adjust_scale(&mut rx, &mut ry);
            rx_inv = ImageSubpixelScale::Scale as i32 * ImageSubpixelScale::Scale as i32 / rx;
            ry_inv = ImageSubpixelScale::Scale as i32 * ImageSubpixelScale::Scale as i32 / ry;
            let radius_x = (diameter * rx) >> 1;
            let radius_y = (diameter * ry) >> 1;
            let len_x_lr = (diameter * rx + ImageSubpixelScale::Mask as i32)
                >> ImageSubpixelScale::Shift as i32;
            x += self.base.filter_dx_int() - radius_x;
            y += self.base.filter_dy_int() - radius_y;
            fg[0] = ImageFilterScale::Scale as i32 / 2;
            fg[1] = fg[0];
            fg[2] = fg[0];

            let y_lr = y >> ImageSubpixelScale::Shift as i32;
            let mut y_hr = ((ImageSubpixelScale::Mask as i32
                - (y & ImageSubpixelScale::Mask as i32))
                * ry_inv)
                >> ImageSubpixelScale::Shift as i32;
            let mut total_weight = 0;
            let x_lr = x >> ImageSubpixelScale::Shift as i32;
            let mut x_hr = ((ImageSubpixelScale::Mask as i32
                - (x & ImageSubpixelScale::Mask as i32))
                * rx_inv)
                >> ImageSubpixelScale::Shift as i32;
            let x_hr2 = x_hr;
            let mut fg_ptr = self.base.source.span(x_lr, y_lr, len_x_lr as u32).as_ptr()
                as *const u8 as *mut <S::ColorType as Args>::ValueType;
            loop {
                let weight_y = self.base.filter.weight_array()[y_hr as usize];
                x_hr = x_hr2;
                loop {
                    let weight = ((weight_y as i32
                        * self.base.filter.weight_array()[x_hr as usize] as i32)
                        + ImageFilterScale::Scale as i32 / 2)
                        >> Self::DOWNSCALE_SHIFT;
                    unsafe {
                        fg[0] += weight * (*fg_ptr.offset(0)).into_i32();
                        fg[1] += weight * (*fg_ptr.offset(1)).into_i32();
                        fg[2] += weight * (*fg_ptr.offset(2)).into_i32();
                    }
                    total_weight += weight;
                    x_hr += rx_inv;
                    if x_hr >= filter_scale {
                        break;
                    }
                    fg_ptr = self.base.source.next_x().as_ptr() as *const u8
                        as *mut <S::ColorType as Args>::ValueType;
                }
                y_hr += ry_inv;
                if y_hr >= filter_scale {
                    break;
                }
                fg_ptr = self.base.source.next_y().as_ptr() as *const u8
                    as *mut <S::ColorType as Args>::ValueType;
            }
            fg[0] /= total_weight;
            fg[1] /= total_weight;
            fg[2] /= total_weight;

            if fg[0] < 0 {
                fg[0] = 0;
            }
            if fg[1] < 0 {
                fg[1] = 0;
            }
            if fg[2] < 0 {
                fg[2] = 0;
            }

            if fg[S::OrderType::R] > Self::BASE_MASK as i32 {
                fg[S::OrderType::R] = Self::BASE_MASK as i32;
            }
            if fg[S::OrderType::G] > Self::BASE_MASK as i32 {
                fg[S::OrderType::G] = Self::BASE_MASK as i32;
            }
            if fg[S::OrderType::B] > Self::BASE_MASK as i32 {
                fg[S::OrderType::B] = Self::BASE_MASK as i32;
            }

            *span[i as usize].r_mut() = from_i32!(fg[S::OrderType::R as usize]);
            *span[i as usize].g_mut() = from_i32!(fg[S::OrderType::G as usize]);
            *span[i as usize].b_mut() = from_i32!(fg[S::OrderType::B as usize]);
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator.next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}
