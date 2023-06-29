use crate::image_filters::{ImageFilterLut, ImageFilterScale, ImageSubpixelScale};
use crate::span_image_filter::{SpanImageFilter, SpanImageResample, SpanImageResampleAffine};
use crate::span_interpolator_linear::SpanIpLinear;
use crate::trans_affine::TransAffine;

use crate::{
    AggPrimitive, Args, Color, GrayArgs, ImageAccessorGray, Interpolator, PixFmt, SpanGenerator,
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
//==============================================SpanImageFilterGrayNn
pub struct SpanImageFilterGrayNn<'a, S: ImageAccessorGray, I: Interpolator> {
    base: SpanImageFilter<'a, S, I>,
}

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanImageFilterGrayNn<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I) -> Self {
        SpanImageFilterGrayNn {
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

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanGenerator for SpanImageFilterGrayNn<'a, S, I> {
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
                *span[i as usize].v_mut() = *fg_ptr.offset(0);
                *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);
            }
            self.base.interpolator.next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//=========================================SpanImageFilterGrayBilinear
pub struct SpanImageFilterGrayBilinear<'a, S: ImageAccessorGray, I: Interpolator> {
    base: SpanImageFilter<'a, S, I>,
}

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanImageFilterGrayBilinear<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I) -> Self {
        SpanImageFilterGrayBilinear {
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

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanGenerator
    for SpanImageFilterGrayBilinear<'a, S, I>
{
    type C = S::ColorType;
    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        self.base.interpolator.begin(
            x as f64 + self.base.filter_dx_dbl(),
            y as f64 + self.base.filter_dy_dbl(),
            len,
        );
        let mut fg: u32;

        for i in 0..len {
            let mut x_hr = 0;
            let mut y_hr = 0;
            self.base.interpolator.coordinates(&mut x_hr, &mut y_hr);
            x_hr -= self.base.filter_dx_int();
            y_hr -= self.base.filter_dy_int();
            let x_lr = x_hr >> ImageSubpixelScale::Shift as u32;
            let y_lr = y_hr >> ImageSubpixelScale::Shift as u32;

            let mut weight: u32;

            fg = (ImageSubpixelScale::Scale as i32 * ImageSubpixelScale::Scale as i32) as u32 / 2;

            x_hr &= ImageSubpixelScale::Mask as i32;
            y_hr &= ImageSubpixelScale::Mask as i32;

            let mut fg_ptr = self.base.source_mut().span(x_lr, y_lr, 2).as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((ImageSubpixelScale::Scale as i32 - x_hr)
                * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
            unsafe {
                fg += weight * (*fg_ptr.offset(0)).into_u32();
            }

            fg_ptr = self.base.source_mut().next_x().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = (x_hr * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
            unsafe {
                fg += weight * (*fg_ptr.offset(0)).into_u32();
            }

            fg_ptr = self.base.source_mut().next_y().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((ImageSubpixelScale::Scale as i32 - x_hr) * y_hr) as u32;
            unsafe {
                fg += weight * (*fg_ptr.offset(0)).into_u32();
            }

            fg_ptr = self.base.source_mut().next_x().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = (x_hr * y_hr) as u32;
            unsafe {
                fg += weight * (*fg_ptr.offset(0)).into_u32();
            }

            *span[i as usize].v_mut() = from_u32!(fg >> (ImageSubpixelScale::Shift as u32 * 2));
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator().next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//====================================SpanImageFilterGrayBilinearClip
pub struct SpanImageFilterGrayBilinearClip<
    'a,
    C: Color + GrayArgs,
    S: PixFmt<C = C>,
    I: Interpolator,
> {
    base: SpanImageFilter<'a, S, I>,
    back_color: S::C,
}

impl<'a, C: Color + GrayArgs, S: PixFmt<C = C>, I: Interpolator>
    SpanImageFilterGrayBilinearClip<'a, C, S, I>
{
    pub const BASE_SHIFT: u32 = S::C::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::C::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I, back_color: S::C) -> Self {
        SpanImageFilterGrayBilinearClip {
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

impl<'a, C: Color + GrayArgs, S: PixFmt<C = C>, I: Interpolator> SpanGenerator
    for SpanImageFilterGrayBilinearClip<'a, C, S, I>
{
    type C = C;
    fn generate(&mut self, span: &mut [S::C], x: i32, y: i32, len: u32) {
        self.base.interpolator.begin(
            x as f64 + self.base.filter_dx_dbl(),
            y as f64 + self.base.filter_dy_dbl(),
            len,
        );

        let mut fg: u32;
        let mut src_alpha; //: u32 = 0;
        let back_r = self.back_color.v();
        let back_a = self.back_color.a();

        let maxx = self.base.source().width() as i32 - 1;
        let maxy = self.base.source().height() as i32 - 1;

        for i in 0..len {
            let mut x_hr = 0;
            let mut y_hr = 0;

            self.base.interpolator.coordinates(&mut x_hr, &mut y_hr);

            x_hr -= self.base.filter_dx_int();
            y_hr -= self.base.filter_dy_int();

            let mut x_lr = x_hr >> ImageSubpixelScale::Shift as u32;
            let mut y_lr = y_hr >> ImageSubpixelScale::Shift as u32;

            let mut weight: u32;
            let mut off;
            if x_lr >= 0 && y_lr >= 0 && x_lr < maxx && y_lr < maxy {
                fg = (ImageSubpixelScale::Scale as i32 * ImageSubpixelScale::Scale as i32) as u32
                    / 2;

                x_hr &= ImageSubpixelScale::Mask as i32;
                y_hr &= ImageSubpixelScale::Mask as i32;

                let fg_ptr = self.base.source.row(y_lr);
                off = x_lr as usize;

                weight = ((ImageSubpixelScale::Scale as i32 - x_hr)
                    * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
                fg += weight * fg_ptr[off].into_u32();

                weight = (x_hr * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
                fg += weight * fg_ptr[off + 1].into_u32();

                y_lr += 1;
                let fg_ptr = self.base.source.row(y_lr);

                weight = ((ImageSubpixelScale::Scale as i32 - x_hr) * y_hr) as u32;
                fg += weight * fg_ptr[off].into_u32();

                weight = (x_hr * y_hr) as u32;
                fg += weight * fg_ptr[off + 1].into_u32();

                fg >>= ImageSubpixelScale::Shift as u32 * 2;
                src_alpha = Self::BASE_MASK;
            } else {
                if x_lr < -1 || y_lr < -1 || x_lr > maxx || y_lr > maxy {
                    fg = back_r.into_u32();
                    src_alpha = back_a.into_u32();
                } else {
                    fg = (ImageSubpixelScale::Scale as i32 * ImageSubpixelScale::Scale as i32)
                        as u32
                        / 2;
                    src_alpha = fg;

                    x_hr &= ImageSubpixelScale::Mask as i32;
                    y_hr &= ImageSubpixelScale::Mask as i32;

                    weight = ((ImageSubpixelScale::Scale as i32 - x_hr)
                        * (ImageSubpixelScale::Scale as i32 - y_hr))
                        as u32;
                    if x_lr >= 0 && y_lr >= 0 && x_lr <= maxx && y_lr <= maxy {
                        let fg_ptr = self.base.source.row(y_lr);

                        off = x_lr as usize;
                        fg += weight * fg_ptr[off].into_u32();
                        src_alpha += weight * Self::BASE_MASK;
                    } else {
                        fg += back_r.into_u32() * weight;
                        src_alpha += back_a.into_u32() * weight;
                    }

                    x_lr += 1;

                    weight = (x_hr * (ImageSubpixelScale::Scale as i32 - y_hr)) as u32;
                    if x_lr >= 0 && y_lr >= 0 && x_lr <= maxx && y_lr <= maxy {
                        let fg_ptr = self.base.source.row(y_lr);

                        off = x_lr as usize;
                        fg += weight * fg_ptr[off].into_u32();
                        src_alpha += weight * Self::BASE_MASK;
                    } else {
                        fg += back_r.into_u32() * weight;
                        src_alpha += back_a.into_u32() * weight;
                    }

                    x_lr -= 1;
                    y_lr += 1;

                    weight = ((ImageSubpixelScale::Scale as i32 - x_hr) * y_hr) as u32;
                    if x_lr >= 0 && y_lr >= 0 && x_lr <= maxx && y_lr <= maxy {
                        let fg_ptr = self.base.source.row(y_lr);

                        off = x_lr as usize;
                        fg += weight * fg_ptr[off].into_u32();
                        src_alpha += weight * Self::BASE_MASK;
                    } else {
                        fg += back_r.into_u32() * weight;
                        src_alpha += back_a.into_u32() * weight;
                    }

                    x_lr += 1;

                    weight = (x_hr * y_hr) as u32;
                    if x_lr >= 0 && y_lr >= 0 && x_lr <= maxx && y_lr <= maxy {
                        let fg_ptr = self.base.source.row(y_lr);

                        off = (x_lr * 3) as usize;
                        fg += weight * fg_ptr[off].into_u32();
                        src_alpha += weight * Self::BASE_MASK;
                    } else {
                        fg += back_r.into_u32() * weight;
                        src_alpha += back_a.into_u32() * weight;
                    }

                    fg >>= ImageSubpixelScale::Shift as u32 * 2;
                    src_alpha >>= ImageSubpixelScale::Shift as u32 * 2;
                }
            }
            *span[i as usize].v_mut() = from_u32!(fg);
            *span[i as usize].a_mut() = from_u32!(src_alpha);

            self.base.interpolator().next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//==============================================SpanImageFilterGray2x2
pub struct SpanImageFilterGray2x2<'a, S: ImageAccessorGray, I: Interpolator> {
    base: SpanImageFilter<'a, S, I>,
}

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanImageFilterGray2x2<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I, filter: ImageFilterLut) -> Self {
        SpanImageFilterGray2x2 {
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

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanGenerator for SpanImageFilterGray2x2<'a, S, I> {
    type C = S::ColorType;
    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let mut fg;

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
            fg = ImageFilterScale::Scale as i32 as u32 / 2;

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
                fg += weight * (*fg_ptr.offset(0)).into_u32();
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
                fg += weight * (*fg_ptr.offset(0)).into_u32();
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
                fg += weight * (*fg_ptr.offset(0)).into_u32();
            }

            fg_ptr = self.base.source.next_x().as_ptr() as *const u8
                as *mut <S::ColorType as Args>::ValueType;
            weight = ((self.base.filter.weight_array()[x_hr as usize + offset] as u32
                * self.base.filter.weight_array()[y_hr as usize + offset] as u32)
                + ImageFilterScale::Scale as u32 / 2)
                >> ImageFilterScale::Shift as u32;
            unsafe {
                fg += weight * (*fg_ptr.offset(0)).into_u32();
            }

            fg >>= ImageFilterScale::Shift as i32;

            if fg > Self::BASE_MASK {
                fg = Self::BASE_MASK;
            }

            *span[i as usize].v_mut() = from_u32!(fg);
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator().next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//==================================================SpanImageFilterGray
pub struct SpanImageFilterGray<'a, S: ImageAccessorGray, I: Interpolator> {
    base: SpanImageFilter<'a, S, I>,
}

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanImageFilterGray<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;

    pub fn new(src: &'a mut S, inter: &'a mut I, filter: ImageFilterLut) -> Self {
        SpanImageFilterGray {
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

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanGenerator for SpanImageFilterGray<'a, S, I> {
    type C = S::ColorType;
    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let (mut x, mut y) = (x, y);
        self.base.interpolator.begin(
            x as f64 + self.base.filter_dx_dbl(),
            y as f64 + self.base.filter_dy_dbl(),
            len,
        );

        let mut fg;
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

            fg = ImageFilterScale::Scale as i32 / 2;

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
                        fg += weight * (*fg_ptr.offset(0)).into_i32();
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

            fg >>= ImageFilterScale::Shift as i32;

            if fg < 0 {
                fg = 0;
            }

            if fg > Self::BASE_MASK as i32 {
                fg = Self::BASE_MASK as i32;
            }

            *span[i as usize].v_mut() = from_i32!(fg);
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator.next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//========================================SpanImageResampleGrayAffine
pub struct SpanImageResampleGrayAffine<'a, S: ImageAccessorGray> {
    base: SpanImageResampleAffine<'a, S>,
}

impl<'a, S: ImageAccessorGray> SpanImageResampleGrayAffine<'a, S> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;
    const DOWNSCALE_SHIFT: u32 = ImageFilterScale::Shift as i32 as u32;
    pub fn new(
        src: &'a mut S, inter: &'a mut SpanIpLinear<TransAffine>, filter: ImageFilterLut,
    ) -> Self {
        SpanImageResampleGrayAffine {
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

impl<'a, S: ImageAccessorGray> SpanGenerator for SpanImageResampleGrayAffine<'a, S> {
    type C = S::ColorType;
    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let (mut x, mut y) = (x, y);
        let (dx, dy) = (self.base.filter_dx_dbl(), self.base.filter_dy_dbl());
        self.base
            .interpolator
            .begin(x as f64 + dx, y as f64 + dy, len);

        let mut fg;

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

            fg = ImageFilterScale::Scale as i32 / 2;

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
                        fg += weight * (*fg_ptr.offset(0)).into_i32();
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

            fg /= total_weight;

            if fg < 0 {
                fg = 0;
            }

            if fg > Self::BASE_MASK as i32 {
                fg = Self::BASE_MASK as i32;
            }

            *span[i as usize].v_mut() = from_i32!(fg);
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator.next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}

//==============================================SpanImageResampleGray
pub struct SpanImageResampleGray<'a, S: ImageAccessorGray, I: Interpolator> {
    base: SpanImageResample<'a, S, I>,
}

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanImageResampleGray<'a, S, I> {
    pub const BASE_SHIFT: u32 = S::ColorType::BASE_SHIFT;
    pub const BASE_MASK: u32 = S::ColorType::BASE_MASK;
    const DOWNSCALE_SHIFT: u32 = ImageFilterScale::Shift as i32 as u32;
    pub fn new(src: &'a mut S, inter: &'a mut I, filter: ImageFilterLut) -> Self {
        SpanImageResampleGray {
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

impl<'a, S: ImageAccessorGray, I: Interpolator> SpanGenerator for SpanImageResampleGray<'a, S, I> {
    type C = S::ColorType;
    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let diameter = self.base.filter.diameter() as i32;
        let filter_scale = diameter << ImageSubpixelScale::Shift as i32;

        let (dx, dy) = (self.base.filter_dx_dbl(), self.base.filter_dy_dbl());
        self.base
            .interpolator
            .begin(x as f64 + dx, y as f64 + dy, len as u32);
        let mut fg;
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
            fg = ImageFilterScale::Scale as i32 / 2;

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
                        fg += weight * (*fg_ptr.offset(0)).into_i32();
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
            fg /= total_weight;

            if fg < 0 {
                fg = 0;
            }

            if fg > Self::BASE_MASK as i32 {
                fg = Self::BASE_MASK as i32;
            }

            *span[i as usize].v_mut() = from_i32!(fg);
            *span[i as usize].a_mut() = from_u32!(Self::BASE_MASK);

            self.base.interpolator.next();
        }
    }
    fn prepare(&mut self) {
        self.base.prepare()
    }
}
