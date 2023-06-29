use crate::{basics::*, GrayArgs};
use crate::{pixfmt_transposer::*, AggInteger, AggPrimitive};
use std::marker::PhantomData;

use crate::{
    Args, BlurCalcGray, BlurCalcRecuGray, BlurCalcRecuRgb, BlurCalcRgb, Color, Order, PixFmt,
    PixFmtGray, RgbArgs,
};

macro_rules! fromu8 {
    ($v:expr) => {
        AggPrimitive::from_u8($v)
    };
}

macro_rules! from_f64 {
    ($v:expr) => {
        AggPrimitive::from_f64($v)
    };
}

macro_rules! into {
    ($v:expr) => {
        AggPrimitive::from_u32($v.into_u32())
    };
}

pub struct StackBlurTables {
    pub g_stack_blur8_mul: [u16; 255],
    pub g_stack_blur8_shr: [u8; 255],
}

impl StackBlurTables {
    pub const STACK_BLUR8_MUL: [u16; 255] = [
        512, 512, 456, 512, 328, 456, 335, 512, 405, 328, 271, 456, 388, 335, 292, 512, 454, 405,
        364, 328, 298, 271, 496, 456, 420, 388, 360, 335, 312, 292, 273, 512, 482, 454, 428, 405,
        383, 364, 345, 328, 312, 298, 284, 271, 259, 496, 475, 456, 437, 420, 404, 388, 374, 360,
        347, 335, 323, 312, 302, 292, 282, 273, 265, 512, 497, 482, 468, 454, 441, 428, 417, 405,
        394, 383, 373, 364, 354, 345, 337, 328, 320, 312, 305, 298, 291, 284, 278, 271, 265, 259,
        507, 496, 485, 475, 465, 456, 446, 437, 428, 420, 412, 404, 396, 388, 381, 374, 367, 360,
        354, 347, 341, 335, 329, 323, 318, 312, 307, 302, 297, 292, 287, 282, 278, 273, 269, 265,
        261, 512, 505, 497, 489, 482, 475, 468, 461, 454, 447, 441, 435, 428, 422, 417, 411, 405,
        399, 394, 389, 383, 378, 373, 368, 364, 359, 354, 350, 345, 341, 337, 332, 328, 324, 320,
        316, 312, 309, 305, 301, 298, 294, 291, 287, 284, 281, 278, 274, 271, 268, 265, 262, 259,
        257, 507, 501, 496, 491, 485, 480, 475, 470, 465, 460, 456, 451, 446, 442, 437, 433, 428,
        424, 420, 416, 412, 408, 404, 400, 396, 392, 388, 385, 381, 377, 374, 370, 367, 363, 360,
        357, 354, 350, 347, 344, 341, 338, 335, 332, 329, 326, 323, 320, 318, 315, 312, 310, 307,
        304, 302, 299, 297, 294, 292, 289, 287, 285, 282, 280, 278, 275, 273, 271, 269, 267, 265,
        263, 261, 259,
    ];
    pub const STACK_BLUR8_SHR: [u8; 255] = [
        9, 11, 12, 13, 13, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 17, 17, 17, 17, 17, 17, 17, 18,
        18, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 20,
        20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 21,
        21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
        22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
        23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
        23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 24, 24, 24,
        24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
        24, 24,
    ];
}
pub struct StackBlur<C: Color, T: BlurCalcRgb> {
    buf: Vec<C>,
    stack: Vec<C>,
    dum: PhantomData<T>,
}

impl<C: Color + RgbArgs, T: BlurCalcRgb> StackBlur<C, T> {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            stack: Vec::new(),
            dum: PhantomData,
        }
    }

    pub fn blur_x<Img: PixFmt<C = C>>(&mut self, img: &mut Img, radius: u32) {
        if radius < 1 {
            return;
        }

        let w = img.width();
        let h = img.height();
        let wm = w - 1;
        let div = radius * 2 + 1;
        let mut sum = T::new();
        let mut sum_in = T::new();
        let mut sum_out = T::new();

        let div_sum = (radius + 1) * (radius + 1);
        let mut mul_sum = 0 as u32;
        let mut shr_sum = 0 as u32;
        let max_val = C::BASE_MASK;

        if max_val <= 255 && radius < 255 {
            mul_sum = StackBlurTables::STACK_BLUR8_MUL[radius as usize] as u32;
            shr_sum = StackBlurTables::STACK_BLUR8_SHR[radius as usize] as u32;
        }

        self.buf.resize(w as usize, C::new());
        self.stack.resize(div as usize, C::new());

        for y in 0..h {
            sum.clear();
            sum_in.clear();
            sum_out.clear();

            let mut pix = img.pixel(0, y as i32);
            for i in 0..=radius {
                self.stack[i as usize] = pix;
                sum.add_k(&pix, i + 1);
                sum_out.add(&pix);
            }
            for i in 1..=radius {
                let xp = if i > wm { wm } else { i };
                pix = img.pixel(xp as i32, y as i32);
                self.stack[(i + radius) as usize] = pix;
                sum.add_k(&pix, radius + 1 - i);
                sum_in.add(&pix);
            }

            let mut stack_ptr = radius;
            for x in 0..w {
                if mul_sum != 0 {
                    sum.calc_pix_mul(&mut self.buf[x as usize], mul_sum, shr_sum);
                } else {
                    sum.calc_pix(&mut self.buf[x as usize], div_sum);
                }

                sum.sub(&sum_out);

                let stack_start = stack_ptr + div - radius;
                let stack_start = if stack_start >= div {
                    stack_start - div
                } else {
                    stack_start
                };
                let stack_pix = &mut self.stack[stack_start as usize];

                sum_out.sub(&*stack_pix);

                let xp = x + radius + 1;
                let xp = if xp > wm { wm } else { xp };
                pix = img.pixel(xp as i32, y as i32);

                *stack_pix = pix;

                sum_in.add(&pix);
                sum.add(&sum_in);

                stack_ptr += 1;
                if stack_ptr >= div {
                    stack_ptr = 0;
                }
                let stack_pix = &mut self.stack[stack_ptr as usize];

                sum_out.add(&*stack_pix);
                sum_in.sub(&*stack_pix);
            }
            img.copy_color_hspan(0, y as i32, w, &self.buf[..]);
        }
    }

    pub fn blur_y<Img: PixFmt<C = C>>(&mut self, img: &mut Img, radius: u32) {
        let mut img2 = PixfmtTransposer::new(img);
        self.blur_x(&mut img2, radius);
    }

    pub fn blur<Img: PixFmt<C = C>>(&mut self, img: &mut Img, radius: u32) {
        self.blur_x(img, radius);
        let mut img2 = PixfmtTransposer::new(img);
        self.blur_x(&mut img2, radius);
    }
}

pub struct StackBlurCalcRgba<T: AggInteger = u32> {
    r: T,
    g: T,
    b: T,
    a: T,
}

impl<T: AggInteger> Args for StackBlurCalcRgba<T> {
    type ValueType = T;
    #[inline]
    fn a(&self) -> Self::ValueType {
        self.a
    }
    #[inline]
    fn a_mut(&mut self) -> &mut Self::ValueType {
        &mut self.a
    }
}

impl<T: AggInteger> RgbArgs for StackBlurCalcRgba<T> {
	fn new_init(r_: Self::ValueType, g_: Self::ValueType, b_: Self::ValueType, a_: Self::ValueType) -> Self {
        Self {
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

impl<T: AggInteger> BlurCalcRgb for StackBlurCalcRgba<T> {
    fn new() -> Self {
        Self {
            r: Default::default(),
            g: Default::default(),
            b: Default::default(),
            a: Default::default(),
        }
    }

    fn clear(&mut self) {
        self.r = Default::default();
        self.g = Default::default();
        self.b = Default::default();
        self.a = Default::default();
    }

    fn add<R: RgbArgs>(&mut self, v: &R) {
        self.r += into!(v.r());
        self.g += into!(v.g());
        self.b += into!(v.b());
        self.a += into!(v.a());
    }

    fn add_k<R: RgbArgs>(&mut self, v: &R, k: u32) {
        self.r += into!(v.r().into_u32() * k);
        self.g += into!(v.g().into_u32() * k);
        self.b += into!(v.b().into_u32() * k);
        self.a += into!(v.a().into_u32() * k);
    }

    fn sub<R: RgbArgs>(&mut self, v: &R) {
        self.r -= into!(v.r());
        self.g -= into!(v.g());
        self.b -= into!(v.b());
        self.a -= into!(v.a());
    }

    fn calc_pix<R: RgbArgs>(&mut self, v: &mut R, div: u32) {
        *v.r_mut() = into!(self.r.into_u32() / div);
        *v.g_mut() = into!(self.g.into_u32() / div);
        *v.b_mut() = into!(self.b.into_u32() / div);
        *v.a_mut() = into!(self.a.into_u32() / div);
    }

    fn calc_pix_mul<R: RgbArgs>(&mut self, v: &mut R, mul: u32, shr: u32) {
        *v.r_mut() = into!((self.r.into_u32() * mul) >> shr);
        *v.g_mut() = into!((self.g.into_u32() * mul) >> shr);
        *v.b_mut() = into!((self.b.into_u32() * mul) >> shr);
        *v.a_mut() = into!((self.a.into_u32() * mul) >> shr);
    }
}

pub type StackBlurCalcRgb<T = u32> = StackBlurCalcRgba<T>;

pub fn stack_blur_gray8<Img: PixFmtGray>(img: &mut Img, rx: u32, ry: u32) {
    let (mut rx, mut ry) = (rx, ry);
    let mut xp: u32;
    let mut yp: u32;
    let mut stack_ptr: u32;
    let mut stack_start: u32;

    let mut src_pix_ptr;
    let mut dst_pix_ptr;
    let mut tmp_dst_ptr;
    let mut pix: u32;
    let mut stack_pix: u32;
    let mut sum: u32;
    let mut sum_in: u32;
    let mut sum_out: u32;

    let w: u32 = img.width();
    let h: u32 = img.height();
    let wm: u32 = w - 1;
    let hm: u32 = h - 1;

    let mut div: u32;
    let mut mul_sum: u32;
    let mut shr_sum: u32;

    let mut stack: Vec<u8> = Vec::new();

    if rx > 0 {
        if rx > 254 {
            rx = 254;
        }
        div = rx * 2 + 1;
        mul_sum = StackBlurTables::STACK_BLUR8_MUL[rx as usize] as u32;
        shr_sum = StackBlurTables::STACK_BLUR8_SHR[rx as usize] as u32;
        stack.resize(div as usize, 0);
        let mut src_ptr: usize;
        let mut dst_ptr: usize;
        for y in 0..h {
            sum = 0;
            sum_in = 0;
            sum_out = 0;

            (src_pix_ptr, src_ptr) = img.pix_ptr(0, y as i32);

            pix = src_pix_ptr[src_ptr] as u32;

            for i in 0..=rx {
                stack[i as usize] = pix as u8;
                sum += pix * (i + 1);
                sum_out += pix;
            }
            for i in 1..=rx {
                if i <= wm {
                    src_ptr += Img::PIXEL_STEP as usize;
                }
                pix = src_pix_ptr[src_ptr] as u32;
                stack[(i + rx) as usize] = pix as u8;
                sum += pix * (rx + 1 - i);
                sum_in += pix;
            }

            stack_ptr = rx;
            xp = rx;
            if xp > wm {
                xp = wm;
            }

            (src_pix_ptr, src_ptr) = img.pix_ptr(xp as i32, y as i32);
            (tmp_dst_ptr, dst_ptr) = img.pix_ptr(0, y as i32);
            dst_pix_ptr = unsafe {
                std::slice::from_raw_parts_mut(tmp_dst_ptr.as_ptr() as *mut u8, tmp_dst_ptr.len())
            };
            for _ in 0..w {
                dst_pix_ptr[dst_ptr] = ((sum * mul_sum) >> shr_sum) as u8;
                dst_ptr += Img::PIXEL_STEP as usize;

                sum -= sum_out;

                stack_start = stack_ptr + div - rx;
                if stack_start >= div {
                    stack_start -= div;
                }
                sum_out -= stack[stack_start as usize] as u32;

                if xp < wm {
                    src_ptr += Img::PIXEL_STEP as usize;
                    pix = src_pix_ptr[src_ptr] as u32;

                    xp += 1;
                }

                stack[stack_start as usize] = pix as u8;

                sum_in += pix;
                sum += sum_in;

                stack_ptr += 1;
                if stack_ptr >= div {
                    stack_ptr = 0;
                }
                stack_pix = stack[stack_ptr as usize] as u32;

                sum_out += stack_pix;
                sum_in -= stack_pix;
            }
        }
    }

    if ry > 0 {
        if ry > 254 {
            ry = 254;
        }
        div = ry * 2 + 1;
        mul_sum = StackBlurTables::STACK_BLUR8_MUL[ry as usize] as u32;
        shr_sum = StackBlurTables::STACK_BLUR8_SHR[ry as usize] as u32;
        stack.resize(div as usize, 0);

        let stride = img.stride();
        let mut src_ptr: usize;
        let mut dst_ptr: usize;

        for x in 0..w {
            sum = 0;
            sum_in = 0;
            sum_out = 0;

            (src_pix_ptr, src_ptr) = img.pix_ptr(x as i32, 0);

            pix = src_pix_ptr[src_ptr] as u32;
            for i in 0..=ry {
                stack[i as usize] = pix as u8;
                sum += pix * (i + 1);
                sum_out += pix;
            }
            for i in 1..=ry {
                if i <= hm {
                    src_ptr = (src_ptr as i32 + stride) as usize;
                }

                pix = src_pix_ptr[src_ptr] as u32;

                stack[(i + ry) as usize] = pix as u8;
                sum += pix * (ry + 1 - i);
                sum_in += pix;
            }

            stack_ptr = ry;
            yp = ry;
            if yp > hm {
                yp = hm;
            }

            (src_pix_ptr, src_ptr) = img.pix_ptr(x as i32, yp as i32);
            (tmp_dst_ptr, dst_ptr) = img.pix_ptr(x as i32, 0);
            dst_pix_ptr = unsafe {
                std::slice::from_raw_parts_mut(tmp_dst_ptr.as_ptr() as *mut u8, tmp_dst_ptr.len())
            };
            for _i in 0..h {
                dst_pix_ptr[dst_ptr] = ((sum * mul_sum) >> shr_sum) as u8;
                dst_ptr = (dst_ptr as i32 + stride) as usize;

                sum -= sum_out;

                stack_start = stack_ptr + div - ry;
                if stack_start >= div {
                    stack_start -= div;
                }
                sum_out -= stack[stack_start as usize] as u32;

                if yp < hm {
                    src_ptr = (src_ptr as i32 + stride) as usize;
                    pix = src_pix_ptr[src_ptr] as u32;
                    yp += 1;
                }

                stack[stack_start as usize] = pix as u8;

                sum_in += pix;
                sum += sum_in;

                stack_ptr += 1;
                if stack_ptr >= div {
                    stack_ptr = 0;
                }
                stack_pix = stack[stack_ptr as usize] as u32;

                sum_out += stack_pix;
                sum_in -= stack_pix;
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn stack_blur_rgb24<C: Color + RgbArgs, Img: PixFmt<C = C>>(img: &mut Img, rx: u32, ry: u32) {
    let (mut rx, mut ry) = (rx, ry);
    let R = Img::O::R as usize;
    let G = Img::O::G as usize;
    let B = Img::O::B as usize;

    let mut xp: u32;
    let mut yp: u32;
    let mut stack_ptr: u32;
    let mut stack_start: u32;
    let mut src_pix_ptr;
    let mut dst_pix_ptr;
    let mut tmp_dst_ptr;
    let mut sum_r: u32;
    let mut sum_g: u32;
    let mut sum_b: u32;
    let mut sum_in_r: u32;
    let mut sum_in_g: u32;
    let mut sum_in_b: u32;
    let mut sum_out_r: u32;
    let mut sum_out_g: u32;
    let mut sum_out_b: u32;
    let mut div: u32;
    let mut mul_sum: u32;
    let mut shr_sum: u32;
    let mut stack: Vec<Img::C>;
    let w: u32 = img.width();
    let h: u32 = img.height();
    let wm: u32 = w - 1;
    let hm: u32 = h - 1;

    if rx > 0 {
        if rx > 254 {
            rx = 254;
        }
        div = rx * 2 + 1;
        mul_sum = StackBlurTables::STACK_BLUR8_MUL[rx as usize] as u32;
        shr_sum = StackBlurTables::STACK_BLUR8_SHR[rx as usize] as u32;
        stack = Vec::new();
        stack.resize(div as usize, Img::C::new());

        let mut src_ptr: usize;
        let mut dst_ptr: usize;

        for y in 0..h {
            sum_r = 0;
            sum_g = 0;
            sum_b = 0;
            sum_in_r = 0;
            sum_in_g = 0;
            sum_in_b = 0;
            sum_out_r = 0;
            sum_out_g = 0;
            sum_out_b = 0;

            (src_pix_ptr, src_ptr) = img.pix_ptr(0, y as i32);
            for i in 0..=rx {
                *stack[i as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[i as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[i as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                sum_r += src_pix_ptr[src_ptr + R] as u32 * (i + 1);
                sum_g += src_pix_ptr[src_ptr + G] as u32 * (i + 1);
                sum_b += src_pix_ptr[src_ptr + B] as u32 * (i + 1);
                sum_out_r += src_pix_ptr[src_ptr + R] as u32;
                sum_out_g += src_pix_ptr[src_ptr + G] as u32;
                sum_out_b += src_pix_ptr[src_ptr + B] as u32;
            }
            for i in 1..=rx {
                if i <= wm {
                    src_ptr += Img::PIXEL_WIDTH as usize;
                }

                *stack[(i + rx) as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[(i + rx) as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[(i + rx) as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                sum_r += src_pix_ptr[src_ptr + R] as u32 * (rx + 1 - i);
                sum_g += src_pix_ptr[src_ptr + G] as u32 * (rx + 1 - i);
                sum_b += src_pix_ptr[src_ptr + B] as u32 * (rx + 1 - i);
                sum_in_r += src_pix_ptr[src_ptr + R] as u32;
                sum_in_g += src_pix_ptr[src_ptr + G] as u32;
                sum_in_b += src_pix_ptr[src_ptr + B] as u32;
            }
            stack_ptr = rx;
            xp = rx;
            if xp > wm {
                xp = wm;
            }

            (src_pix_ptr, src_ptr) = img.pix_ptr(xp as i32, y as i32);
            (tmp_dst_ptr, dst_ptr) = img.pix_ptr(0, y as i32);
            dst_pix_ptr = unsafe {
                std::slice::from_raw_parts_mut(tmp_dst_ptr.as_ptr() as *mut u8, tmp_dst_ptr.len())
            };

            for _ in 0..w {
                dst_pix_ptr[dst_ptr + R] = ((sum_r.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + G] = ((sum_g.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + B] = ((sum_b.wrapping_mul(mul_sum)) >> shr_sum) as u8;

                dst_ptr += Img::PIXEL_WIDTH as usize;

                sum_r = sum_r.wrapping_sub(sum_out_r);
                sum_g = sum_g.wrapping_sub(sum_out_g);
                sum_b = sum_b.wrapping_sub(sum_out_b);
                stack_start = stack_ptr + div - rx;
                if stack_start >= div {
                    stack_start -= div;
                }
                //let mut stack_pix_ptr = &mut stack[stack_start as usize];
                sum_out_r -= stack[stack_start as usize].r().into_u32();
                sum_out_g -= stack[stack_start as usize].g().into_u32();
                sum_out_b -= stack[stack_start as usize].b().into_u32();
                if xp < wm {
                    src_ptr += Img::PIXEL_WIDTH as usize;

                    xp += 1;
                }

                *stack[stack_start as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[stack_start as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[stack_start as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                sum_in_r = sum_in_r.wrapping_add(src_pix_ptr[src_ptr + R] as u32);
                sum_in_g = sum_in_g.wrapping_add(src_pix_ptr[src_ptr + G] as u32);
                sum_in_b = sum_in_b.wrapping_add(src_pix_ptr[src_ptr + B] as u32);

                sum_r = sum_r.wrapping_add(sum_in_r);
                sum_g = sum_g.wrapping_add(sum_in_g);
                sum_b = sum_b.wrapping_add(sum_in_b);
                stack_ptr += 1;
                if stack_ptr >= div {
                    stack_ptr = 0;
                }
                //let mut stack_pix_ptr = &mut stack[stack_ptr as usize];
                sum_out_r += stack[stack_ptr as usize].r().into_u32();
                sum_out_g += stack[stack_ptr as usize].g().into_u32();
                sum_out_b += stack[stack_ptr as usize].b().into_u32();
                sum_in_r = sum_in_r.wrapping_sub(stack[stack_ptr as usize].r().into_u32());
                sum_in_g = sum_in_g.wrapping_sub(stack[stack_ptr as usize].g().into_u32());
                sum_in_b = sum_in_b.wrapping_sub(stack[stack_ptr as usize].b().into_u32());
            }
        }
    }

    if ry > 0 {
        if ry > 254 {
            ry = 254;
        }
        div = ry * 2 + 1;
        mul_sum = StackBlurTables::STACK_BLUR8_MUL[ry as usize] as u32;
        shr_sum = StackBlurTables::STACK_BLUR8_SHR[ry as usize] as u32;
        stack = Vec::new();
        stack.resize(div as usize, Img::C::new());
        let stride = img.stride();

        let mut src_ptr: usize;
        let mut dst_ptr: usize;

        for x in 0..w {
            sum_r = 0;
            sum_g = 0;
            sum_b = 0;
            sum_in_r = 0;
            sum_in_g = 0;
            sum_in_b = 0;
            sum_out_r = 0;
            sum_out_g = 0;
            sum_out_b = 0;

            (src_pix_ptr, src_ptr) = img.pix_ptr(x as i32, 0);
            for i in 0..=ry {
                *stack[i as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[i as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[i as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                sum_r += src_pix_ptr[src_ptr + R] as u32 * (i + 1);
                sum_g += src_pix_ptr[src_ptr + G] as u32 * (i + 1);
                sum_b += src_pix_ptr[src_ptr + B] as u32 * (i + 1);
                sum_out_r += src_pix_ptr[src_ptr + R] as u32;
                sum_out_g += src_pix_ptr[src_ptr + G] as u32;
                sum_out_b += src_pix_ptr[src_ptr + B] as u32;
            }
            for i in 1..=ry {
                if i <= hm {
                    src_ptr = (src_ptr as i32 + stride) as usize;
                }

                *stack[(i + ry) as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[(i + ry) as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[(i + ry) as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                sum_r += src_pix_ptr[src_ptr + R] as u32 * (ry + 1 - i);
                sum_g += src_pix_ptr[src_ptr + G] as u32 * (ry + 1 - i);
                sum_b += src_pix_ptr[src_ptr + B] as u32 * (ry + 1 - i);
                sum_in_r += src_pix_ptr[src_ptr + R] as u32;
                sum_in_g += src_pix_ptr[src_ptr + G] as u32;
                sum_in_b += src_pix_ptr[src_ptr + B] as u32;
            }
            stack_ptr = ry;
            yp = ry;
            if yp > hm {
                yp = hm;
            }

            (src_pix_ptr, src_ptr) = img.pix_ptr(x as i32, yp as i32);
            (tmp_dst_ptr, dst_ptr) = img.pix_ptr(x as i32, 0);
            dst_pix_ptr = unsafe {
                std::slice::from_raw_parts_mut(tmp_dst_ptr.as_ptr() as *mut u8, tmp_dst_ptr.len())
            };

            for _ in 0..h {
                dst_pix_ptr[dst_ptr + R] = ((sum_r.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + G] = ((sum_g.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + B] = ((sum_b.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_ptr = (dst_ptr as i32 + stride) as usize;

                sum_r = sum_r.wrapping_sub(sum_out_r);
                sum_g = sum_g.wrapping_sub(sum_out_g);
                sum_b = sum_b.wrapping_sub(sum_out_b);
                stack_start = stack_ptr + div - ry;
                if stack_start >= div {
                    stack_start -= div;
                }
                //let mut stack_pix_ptr = &mut stack[stack_start as usize];
                sum_out_r -= stack[stack_start as usize].r().into_u32();
                sum_out_g -= stack[stack_start as usize].g().into_u32();
                sum_out_b -= stack[stack_start as usize].b().into_u32();
                if yp < hm {
                    src_ptr = (src_ptr as i32 + stride) as usize;

                    yp += 1;
                }

                *stack[stack_start as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[stack_start as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[stack_start as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                sum_in_r = sum_in_r.wrapping_add(src_pix_ptr[src_ptr + R] as u32);
                sum_in_g = sum_in_g.wrapping_add(src_pix_ptr[src_ptr + G] as u32);
                sum_in_b = sum_in_b.wrapping_add(src_pix_ptr[src_ptr + B] as u32);

                sum_r = sum_r.wrapping_add(sum_in_r);
                sum_g = sum_g.wrapping_add(sum_in_g);
                sum_b = sum_b.wrapping_add(sum_in_b);
                stack_ptr += 1;
                if stack_ptr >= div {
                    stack_ptr = 0;
                }
                //let mut stack_pix_ptr = &mut stack[stack_ptr as usize];
                sum_out_r += stack[stack_ptr as usize].r().into_u32();
                sum_out_g += stack[stack_ptr as usize].g().into_u32();
                sum_out_b += stack[stack_ptr as usize].b().into_u32();
                sum_in_r = sum_in_r.wrapping_sub(stack[stack_ptr as usize].r().into_u32());
                sum_in_g = sum_in_g.wrapping_sub(stack[stack_ptr as usize].g().into_u32());
                sum_in_b = sum_in_b.wrapping_sub(stack[stack_ptr as usize].b().into_u32());
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn stack_blur_rgb32<C: Color + RgbArgs, Img: PixFmt<C = C>>(img: &mut Img, rx: u32, ry: u32) {
    let (mut rx, mut ry) = (rx, ry);
    let R = Img::O::R as usize;
    let G = Img::O::G as usize;
    let B = Img::O::B as usize;
    let A = Img::O::A as usize;

    let mut xp: u32;
    let mut yp: u32;
    let mut stack_ptr: u32;
    let mut stack_start: u32;
    let mut src_pix_ptr;
    let mut dst_pix_ptr;
    let mut tmp_dst_ptr;

    let mut sum_r: u32;
    let mut sum_g: u32;
    let mut sum_b: u32;
    let mut sum_a: u32;
    let mut sum_in_r: u32;
    let mut sum_in_g: u32;
    let mut sum_in_b: u32;
    let mut sum_in_a: u32;
    let mut sum_out_r: u32;
    let mut sum_out_g: u32;
    let mut sum_out_b: u32;
    let mut sum_out_a: u32;
    let mut div: u32;
    let mut mul_sum: u32;
    let mut shr_sum: u32;
    let mut stack: Vec<Img::C>;
    let w: u32 = img.width();
    let h: u32 = img.height();
    let wm: u32 = w - 1;
    let hm: u32 = h - 1;

    if rx > 0 {
        if rx > 254 {
            rx = 254;
        }
        div = rx * 2 + 1;
        mul_sum = StackBlurTables::STACK_BLUR8_MUL[rx as usize] as u32;
        shr_sum = StackBlurTables::STACK_BLUR8_SHR[rx as usize] as u32;
        stack = Vec::new();
        stack.resize(div as usize, Img::C::new());

        let mut src_ptr: usize;
        let mut dst_ptr: usize;

        for y in 0..h {
            sum_r = 0;
            sum_g = 0;
            sum_b = 0;
            sum_a = 0;
            sum_in_r = 0;
            sum_in_g = 0;
            sum_in_b = 0;
            sum_in_a = 0;
            sum_out_r = 0;
            sum_out_g = 0;
            sum_out_b = 0;
            sum_out_a = 0;

            (src_pix_ptr, src_ptr) = img.pix_ptr(0, y as i32);
            for i in 0..=rx {
                *stack[i as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[i as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[i as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                *stack[i as usize].a_mut() = fromu8!(src_pix_ptr[src_ptr + A]);
                sum_r += src_pix_ptr[src_ptr + R] as u32 * (i + 1);
                sum_g += src_pix_ptr[src_ptr + G] as u32 * (i + 1);
                sum_b += src_pix_ptr[src_ptr + B] as u32 * (i + 1);
                sum_a += src_pix_ptr[src_ptr + A] as u32 * (i + 1);
                sum_out_r += src_pix_ptr[src_ptr + R] as u32;
                sum_out_g += src_pix_ptr[src_ptr + G] as u32;
                sum_out_b += src_pix_ptr[src_ptr + B] as u32;
                sum_out_a += src_pix_ptr[src_ptr + A] as u32;
            }
            for i in 1..=rx {
                if i <= wm {
                    src_ptr += Img::PIXEL_WIDTH as usize;
                }

                *stack[(i + rx) as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[(i + rx) as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[(i + rx) as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                *stack[(i + rx) as usize].a_mut() = fromu8!(src_pix_ptr[src_ptr + A]);
                sum_r += src_pix_ptr[src_ptr + R] as u32 * (rx + 1 - i);
                sum_g += src_pix_ptr[src_ptr + G] as u32 * (rx + 1 - i);
                sum_b += src_pix_ptr[src_ptr + B] as u32 * (rx + 1 - i);
                sum_a += src_pix_ptr[src_ptr + A] as u32 * (rx + 1 - i);
                sum_out_r += src_pix_ptr[src_ptr + R] as u32;
                sum_out_g += src_pix_ptr[src_ptr + G] as u32;
                sum_out_b += src_pix_ptr[src_ptr + B] as u32;
                sum_out_a += src_pix_ptr[src_ptr + A] as u32;
            }
            stack_ptr = rx;
            xp = rx;
            if xp > wm {
                xp = wm;
            }

            (src_pix_ptr, src_ptr) = img.pix_ptr(xp as i32, y as i32);
            (tmp_dst_ptr, dst_ptr) = img.pix_ptr(0, y as i32);
            dst_pix_ptr = unsafe {
                std::slice::from_raw_parts_mut(tmp_dst_ptr.as_ptr() as *mut u8, tmp_dst_ptr.len())
            };

            for _ in 0..w {
                dst_pix_ptr[dst_ptr + R] = ((sum_r.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + G] = ((sum_g.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + B] = ((sum_b.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + A] = ((sum_a.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_ptr += Img::PIXEL_WIDTH as usize;

                sum_r = sum_r.wrapping_sub(sum_out_r);
                sum_g = sum_g.wrapping_sub(sum_out_g);
                sum_b = sum_b.wrapping_sub(sum_out_b);
                sum_a = sum_a.wrapping_sub(sum_out_a);

                stack_start = stack_ptr + div - rx;
                if stack_start >= div {
                    stack_start -= div;
                }
                //let mut stack_pix_ptr = &mut stack[stack_start as usize];
                sum_out_r -= stack[stack_start as usize].r().into_u32();
                sum_out_g -= stack[stack_start as usize].g().into_u32();
                sum_out_b -= stack[stack_start as usize].b().into_u32();
                sum_out_a -= stack[stack_start as usize].a().into_u32();
                if xp < wm {
                    src_ptr += Img::PIXEL_WIDTH as usize;

                    xp += 1;
                }

                *stack[stack_start as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[stack_start as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[stack_start as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                *stack[stack_start as usize].a_mut() = fromu8!(src_pix_ptr[src_ptr + A]);
                sum_in_r = sum_in_r.wrapping_add(src_pix_ptr[src_ptr + R] as u32);
                sum_in_g = sum_in_g.wrapping_add(src_pix_ptr[src_ptr + G] as u32);
                sum_in_b = sum_in_b.wrapping_add(src_pix_ptr[src_ptr + B] as u32);
                sum_in_a = sum_in_a.wrapping_add(src_pix_ptr[src_ptr + A] as u32);

                sum_r = sum_r.wrapping_add(sum_in_r);
                sum_g = sum_g.wrapping_add(sum_in_g);
                sum_b = sum_b.wrapping_add(sum_in_b);
                sum_a = sum_a.wrapping_add(sum_in_a);

                stack_ptr += 1;
                if stack_ptr >= div {
                    stack_ptr = 0;
                }
                //let mut stack_pix_ptr = &mut stack[stack_ptr as usize];
                sum_out_r += stack[stack_ptr as usize].r().into_u32();
                sum_out_g += stack[stack_ptr as usize].g().into_u32();
                sum_out_b += stack[stack_ptr as usize].b().into_u32();
                sum_out_a += stack[stack_ptr as usize].a().into_u32();
                sum_in_r = sum_in_r.wrapping_sub(stack[stack_ptr as usize].r().into_u32());
                sum_in_g = sum_in_g.wrapping_sub(stack[stack_ptr as usize].g().into_u32());
                sum_in_b = sum_in_b.wrapping_sub(stack[stack_ptr as usize].b().into_u32());
                sum_in_a = sum_in_a.wrapping_sub(stack[stack_ptr as usize].a().into_u32());
            }
        }
    }

    if ry > 0 {
        if ry > 254 {
            ry = 254;
        }
        div = ry * 2 + 1;
        mul_sum = StackBlurTables::STACK_BLUR8_MUL[ry as usize] as u32;
        shr_sum = StackBlurTables::STACK_BLUR8_SHR[ry as usize] as u32;
        stack = Vec::new();
        stack.resize(div as usize, Img::C::new());
        let stride = img.stride();

        let mut src_ptr: usize;
        let mut dst_ptr: usize;
        let mut tmp_dst_ptr;
        for x in 0..w {
            sum_r = 0;
            sum_g = 0;
            sum_b = 0;
            sum_a = 0;
            sum_in_r = 0;
            sum_in_g = 0;
            sum_in_b = 0;
            sum_in_a = 0;
            sum_out_r = 0;
            sum_out_g = 0;
            sum_out_b = 0;
            sum_out_a = 0;

            (src_pix_ptr, src_ptr) = img.pix_ptr(x as i32, 0);
            for i in 0..=ry {
                *stack[i as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[i as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[i as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                *stack[i as usize].a_mut() = fromu8!(src_pix_ptr[src_ptr + A]);
                sum_r += src_pix_ptr[src_ptr + R] as u32 * (i + 1);
                sum_g += src_pix_ptr[src_ptr + G] as u32 * (i + 1);
                sum_b += src_pix_ptr[src_ptr + B] as u32 * (i + 1);
                sum_a += src_pix_ptr[src_ptr + A] as u32 * (i + 1);
                sum_out_r += src_pix_ptr[src_ptr + R] as u32;
                sum_out_g += src_pix_ptr[src_ptr + G] as u32;
                sum_out_b += src_pix_ptr[src_ptr + B] as u32;
                sum_out_a += src_pix_ptr[src_ptr + A] as u32;
            }
            for i in 1..=ry {
                if i <= hm {
                    src_ptr = (src_ptr as i32 + stride) as usize;
                }

                *stack[(i + ry) as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[(i + ry) as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[(i + ry) as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                *stack[(i + ry) as usize].a_mut() = fromu8!(src_pix_ptr[src_ptr + A]);
                sum_r += src_pix_ptr[src_ptr + R] as u32 * (ry + 1 - i);
                sum_g += src_pix_ptr[src_ptr + G] as u32 * (ry + 1 - i);
                sum_b += src_pix_ptr[src_ptr + B] as u32 * (ry + 1 - i);
                sum_a += src_pix_ptr[src_ptr + A] as u32 * (ry + 1 - i);
                sum_out_r += src_pix_ptr[src_ptr + R] as u32;
                sum_out_g += src_pix_ptr[src_ptr + G] as u32;
                sum_out_b += src_pix_ptr[src_ptr + B] as u32;
                sum_out_a += src_pix_ptr[src_ptr + A] as u32;
            }
            stack_ptr = ry;
            yp = ry;
            if yp > hm {
                yp = hm;
            }

            (src_pix_ptr, src_ptr) = img.pix_ptr(x as i32, yp as i32);
            (tmp_dst_ptr, dst_ptr) = img.pix_ptr(x as i32, 0);
            dst_pix_ptr = unsafe {
                std::slice::from_raw_parts_mut(tmp_dst_ptr.as_ptr() as *mut u8, tmp_dst_ptr.len())
            };

            for _ in 0..h {
                dst_pix_ptr[dst_ptr + R] = ((sum_r.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + G] = ((sum_g.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + B] = ((sum_b.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_pix_ptr[dst_ptr + A] = ((sum_a.wrapping_mul(mul_sum)) >> shr_sum) as u8;
                dst_ptr = (dst_ptr as i32 + stride) as usize;

                sum_r = sum_r.wrapping_sub(sum_out_r);
                sum_g = sum_g.wrapping_sub(sum_out_g);
                sum_b = sum_b.wrapping_sub(sum_out_b);
                sum_a = sum_a.wrapping_sub(sum_out_a);

                stack_start = stack_ptr + div - ry;
                if stack_start >= div {
                    stack_start -= div;
                }
                //let mut stack_pix_ptr = &mut stack[stack_start as usize];
                sum_out_r -= stack[stack_start as usize].r().into_u32();
                sum_out_g -= stack[stack_start as usize].g().into_u32();
                sum_out_b -= stack[stack_start as usize].b().into_u32();
                sum_out_a -= stack[stack_start as usize].a().into_u32();
                if yp < hm {
                    src_ptr = (src_ptr as i32 + stride) as usize;

                    yp += 1;
                }

                *stack[stack_start as usize].r_mut() = fromu8!(src_pix_ptr[src_ptr + R]);
                *stack[stack_start as usize].g_mut() = fromu8!(src_pix_ptr[src_ptr + G]);
                *stack[stack_start as usize].b_mut() = fromu8!(src_pix_ptr[src_ptr + B]);
                *stack[stack_start as usize].a_mut() = fromu8!(src_pix_ptr[src_ptr + A]);
                sum_in_r = sum_in_r.wrapping_add(src_pix_ptr[src_ptr + R] as u32);
                sum_in_g = sum_in_g.wrapping_add(src_pix_ptr[src_ptr + G] as u32);
                sum_in_b = sum_in_b.wrapping_add(src_pix_ptr[src_ptr + B] as u32);
                sum_in_a = sum_in_a.wrapping_add(src_pix_ptr[src_ptr + A] as u32);

                sum_r = sum_r.wrapping_add(sum_in_r);
                sum_g = sum_g.wrapping_add(sum_in_g);
                sum_b = sum_b.wrapping_add(sum_in_b);
                sum_a = sum_a.wrapping_add(sum_in_a);

                stack_ptr += 1;
                if stack_ptr >= div {
                    stack_ptr = 0;
                }
                //let mut stack_pix_ptr = &mut stack[stack_ptr as usize];
                sum_out_r += stack[stack_ptr as usize].r().into_u32();
                sum_out_g += stack[stack_ptr as usize].g().into_u32();
                sum_out_b += stack[stack_ptr as usize].b().into_u32();
                sum_out_a += stack[stack_ptr as usize].a().into_u32();
                sum_in_r = sum_in_r.wrapping_sub(stack[stack_ptr as usize].r().into_u32());
                sum_in_g = sum_in_g.wrapping_sub(stack[stack_ptr as usize].g().into_u32());
                sum_in_b = sum_in_b.wrapping_sub(stack[stack_ptr as usize].b().into_u32());
                sum_in_a = sum_in_a.wrapping_sub(stack[stack_ptr as usize].a().into_u32());
            }
        }
    }
}

//===========================================================recursive_blur
pub struct RecursiveBlur<C: Color + RgbArgs, CX: BlurCalcRecuRgb> {
    dum_c: PhantomData<C>,
    dum_cx: PhantomData<CX>,
}

impl<C: Color + RgbArgs, CX: BlurCalcRecuRgb> RecursiveBlur<C, CX> {
    pub fn new() -> Self {
        Self {
            dum_c: PhantomData,
            dum_cx: PhantomData,
        }
    }

    //--------------------------------------------------------------------
    pub fn blur_x<Pix: PixFmt<C = C>>(&mut self, img: &mut Pix, radius: f64) {
        if radius < 0.62 {
            return;
        }
        if img.width() < 3 {
            return;
        }

        let s = radius * 0.5;
        let q = if s < 2.5 {
            3.97156 - 4.14554 * (1.0 - 0.26891 * s).sqrt()
        } else {
            0.98711 * s - 0.96330
        };

        let q2 = q * q;
        let q3 = q2 * q;

        let b0: CX::ValueType =
            from_f64!(1.0 / (1.578250 + 2.444130 * q + 1.428100 * q2 + 0.422205 * q3));

        let b1: CX::ValueType = from_f64!(2.44413 * q + 2.85619 * q2 + 1.26661 * q3);

        let b2: CX::ValueType = from_f64!(-1.42810 * q2 - 1.26661 * q3);

        let b3: CX::ValueType = from_f64!(0.422205 * q3);

        let b: CX::ValueType = CX::ValueType::from_f64(1.0) - (b1 + b2 + b3) * b0;

        let b1 = b1 * b0;
        let b2 = b2 * b0;
        let b3 = b3 * b0;

        let w = img.width() as usize;
        let h = img.height();
        let wm = w as usize - 1;

        let sum1 = vec![CX::new(); w];
        let sum2 = vec![CX::new(); w];
        let mut buf = vec![C::new(); w];

        for y in 0..h {
            let mut c = CX::new();
            c.from_pix(&img.pixel(0, y as i32));
            sum1[0].calc(b, b1, b2, b3, &c, &c, &c, &c);
            c.from_pix(&img.pixel(1, y as i32));

            sum1[1].calc(b, b1, b2, b3, &c, &sum1[0], &sum1[0], &sum1[0]);
            c.from_pix(&img.pixel(2, y as i32));
            sum1[2].calc(b, b1, b2, b3, &c, &sum1[1], &sum1[0], &sum1[0]);

            for x in 3..w as usize {
                c.from_pix(&img.pixel(x as i32, y as i32));
                sum1[x].calc(b, b1, b2, b3, &c, &sum1[x - 1], &sum1[x - 2], &sum1[x - 3]);
            }

            sum2[wm].calc(b, b1, b2, b3, &sum1[wm], &sum1[wm], &sum1[wm], &sum1[wm]);
            sum2[wm - 1].calc(
                b,
                b1,
                b2,
                b3,
                &sum1[wm - 1],
                &sum2[wm],
                &sum2[wm],
                &sum2[wm],
            );
            sum2[wm - 2].calc(
                b,
                b1,
                b2,
                b3,
                &sum1[wm - 2],
                &sum2[wm - 1],
                &sum2[wm],
                &sum2[wm],
            );

            sum2[wm].to_pix(&mut buf[wm]);
            sum2[wm - 1].to_pix(&mut buf[wm - 1]);
            sum2[wm - 2].to_pix(&mut buf[wm - 2]);

            for x in (0..wm - 2).rev() {
                sum2[x].calc(
                    b,
                    b1,
                    b2,
                    b3,
                    &sum1[x],
                    &sum2[x + 1],
                    &sum2[x + 2],
                    &sum2[x + 3],
                );
                sum2[x].to_pix(&mut buf[x]);
            }
            img.copy_color_hspan(0, y as i32, w as u32, buf.as_slice());
        }
    }

    //--------------------------------------------------------------------
    pub fn blur_y<Pix: PixFmt<C = C>>(&mut self, img: &mut Pix, radius: f64) {
        let mut img2 = PixfmtTransposer::new(img);
        self.blur_x(&mut img2, radius);
    }

    //--------------------------------------------------------------------
    pub fn blur<Pix: PixFmt<C = C>>(&mut self, img: &mut Pix, radius: f64) {
        self.blur_x(img, radius);
        let mut img2 = PixfmtTransposer::new(img);
        self.blur_x(&mut img2, radius);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecursiveBlurCalcRgba<T: AggPrimitive = f64> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T: AggPrimitive> BlurCalcRecuRgb for RecursiveBlurCalcRgba<T> {
    type ValueType = T;

    fn new() -> Self {
        Self {
            r: Default::default(),
            g: Default::default(),
            b: Default::default(),
            a: Default::default(),
        }
    }

    fn from_pix<C: RgbArgs>(&mut self, v: &C) {
        self.r = into!(v.r());
        self.g = into!(v.g());
        self.b = into!(v.b());
        self.a = into!(v.a());
    }

    fn calc(&self, b1: T, b2: T, b3: T, b4: T, c1: &Self, c2: &Self, c3: &Self, c4: &Self) {
        unsafe {
            let s = &mut *(self as *const Self as *mut Self);
            s.r = b1 * (*c1).r + b2 * (*c2).r + b3 * (*c3).r + b4 * (*c4).r;
            s.g = b1 * (*c1).g + b2 * (*c2).g + b3 * (*c3).g + b4 * (*c4).g;
            s.b = b1 * (*c1).b + b2 * (*c2).b + b3 * (*c3).b + b4 * (*c4).b;
            s.a = b1 * (*c1).a + b2 * (*c2).a + b3 * (*c3).a + b4 * (*c4).a;
        }
    }

    fn to_pix<C: RgbArgs>(&self, c: &mut C) {
        *c.r_mut() = into!(uround(self.r.into_f64()));
        *c.g_mut() = into!(uround(self.g.into_f64()));
        *c.b_mut() = into!(uround(self.b.into_f64()));
        *c.a_mut() = into!(uround(self.a.into_f64()));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecursiveBlurCalcRgb<T: AggPrimitive = f64> {
    pub r: T,
    pub g: T,
    pub b: T,
    //pub a: T,
}

impl<T: AggPrimitive> BlurCalcRecuRgb for RecursiveBlurCalcRgb<T> {
    type ValueType = T;

    fn new() -> Self {
        Self {
            r: Default::default(),
            g: Default::default(),
            b: Default::default(),
        }
    }

    fn from_pix<C: RgbArgs>(&mut self, v: &C) {
        self.r = into!(v.r());
        self.g = into!(v.g());
        self.b = into!(v.b());
    }

    fn calc(&self, b1: T, b2: T, b3: T, b4: T, c1: &Self, c2: &Self, c3: &Self, c4: &Self) {
        unsafe {
            let s = &mut *(self as *const Self as *mut Self);
            s.r = b1 * (*c1).r + b2 * (*c2).r + b3 * (*c3).r + b4 * (*c4).r;
            s.g = b1 * (*c1).g + b2 * (*c2).g + b3 * (*c3).g + b4 * (*c4).g;
            s.b = b1 * (*c1).b + b2 * (*c2).b + b3 * (*c3).b + b4 * (*c4).b;
        }
    }

    fn to_pix<C: RgbArgs>(&self, c: &mut C) {
        *c.r_mut() = into!(uround(self.r.into_f64()));
        *c.g_mut() = into!(uround(self.g.into_f64()));
        *c.b_mut() = into!(uround(self.b.into_f64()));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecursiveBlurCalcGray<T: AggPrimitive> {
    pub v: T,
}

impl<T: AggPrimitive> BlurCalcRecuGray for RecursiveBlurCalcGray<T> {
    type ValueType = T;

    fn new() -> Self {
        Self {
            v: Default::default(),
        }
    }
    fn from_pix<C: GrayArgs>(&mut self, c: &C) {
        self.v = into!(c.v());
    }

    fn calc(&self, b1: T, b2: T, b3: T, b4: T, c1: &Self, c2: &Self, c3: &Self, c4: &Self) {
        unsafe {
            let s = &mut *(self as *const Self as *mut Self);
            s.v = b1 * (*c1).v + b2 * (*c2).v + b3 * (*c3).v + b4 * (*c4).v;
        }
    }

    fn to_pix<C: GrayArgs>(&self, c: &mut C) {
        *c.v_mut() = into!(uround(self.v.into_f64()));
    }
}

pub struct StackBlurCalcGray<T> {
    pub v: T,
    pub a: T,
}

impl<T: AggInteger> Args for StackBlurCalcGray<T> {
    type ValueType = T;
    #[inline]
    fn a(&self) -> Self::ValueType {
        self.a
    }
    #[inline]
    fn a_mut(&mut self) -> &mut Self::ValueType {
        &mut self.a
    }
}

impl<T: AggInteger> GrayArgs for StackBlurCalcGray<T> {
	fn new_init(v: T, a: T) -> Self {
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

impl<T: AggInteger> BlurCalcGray for StackBlurCalcGray<T> {
    fn new() -> Self {
        Self {
            v: Default::default(),
            a: Default::default(),
        }
    }

    fn clear(&mut self) {
        self.v = Default::default();
        self.a = Default::default();
    }

    fn add<G: GrayArgs>(&mut self, g: &G) {
        self.a += into!(g.a());
        self.v += into!(g.v());
    }

    fn add_k<G: GrayArgs>(&mut self, g: &G, k: u32) {
        self.v += into!(g.v().into_u32() * k);
        self.a += into!(g.a().into_u32() * k);
    }

    fn sub<G: GrayArgs>(&mut self, g: &G) {
        self.v -= into!(g.v());
        self.a -= into!(g.a());
    }

    fn calc_pix<G: GrayArgs>(&mut self, g: &mut G, div: u32) {
        *g.v_mut() = into!(self.v.into_u32() / div);
        *g.a_mut() = into!(self.a.into_u32() / div);
    }

    fn calc_pix_mul<G: GrayArgs>(&mut self, g: &mut G, mul: u32, shr: u32) {
        *g.v_mut() = into!((self.v.into_u32() * mul) >> shr);
        *g.a_mut() = into!((self.a.into_u32() * mul) >> shr);
    }
}
