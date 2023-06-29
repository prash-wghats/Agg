use std::marker::PhantomData;
use crate::line_aa_basics::{LineSubpixel};
use crate::{Color, PatternFilter, AggPrimitive, RgbArgs};
use crate::color_rgba::{Rgba8, Rgba16};       

// NOT TESTED

macro_rules! from_u32 {
    ($v:expr) => {
        AggPrimitive::from_u32($v)
    };
}


//=======================================================PatternFilterNn
pub struct PatternFilterNn<C:Color> {
	dummy: PhantomData<C>
}

impl<C:Color> PatternFilter for PatternFilterNn<C> {
	type ColorType = C;
	fn new() -> Self {
		Self {
			dummy: PhantomData,
		}
	}
	fn dilation(&self) -> u32 { 0 }

	fn pixel_low_res(&self, buf: &[&[C]], p: &mut C, x: i32, y: i32) {
	 *p = buf[y as usize][x as usize]; 
	}

	fn pixel_high_res(&self, buf: &[&[C]], p: &mut C, x: i32, y: i32) {
		*p = buf[(y >> LineSubpixel::Shift as i32) as usize]
				[(x >> LineSubpixel::Shift as i32) as usize];
	}
}
pub type PatternFilterNNRGBA8 = PatternFilterNn<Rgba8>;
pub type PatternFilterNNRGBA16 = PatternFilterNn<Rgba16>;

//===========================================PatternFilterBilinearRgba
pub struct PatternFilterBilinearRgba<C:Color + RgbArgs> {
	dummy: PhantomData<C>
}

impl<C:Color + RgbArgs> PatternFilter for PatternFilterBilinearRgba<C> {
	type ColorType = C;

	fn new() -> Self {
		Self {
			dummy: PhantomData,
		}
	}
	
	fn dilation(&self) -> u32 { 1 }

	fn pixel_low_res(&self, buf: &[&[C]], p: &mut C, x: i32, y: i32) {
		*p = buf[y as usize][x as usize]; 
	}

	fn pixel_high_res(&self, buf: &[&[C]], p: &mut C, x: i32, y: i32) {
		let (mut x, mut y) = (x, y);
		let mut r: u32 = (LineSubpixel::Scale as i32 * LineSubpixel::Scale as i32) as u32 / 2;
		let mut g: u32 = r;
		let mut b: u32 = r;
		let mut a: u32 = r;

		let mut weight: u32;
		let x_lr = x >> LineSubpixel::Shift as i32;
		let y_lr = y >> LineSubpixel::Shift as i32;

		x &= LineSubpixel::Mask as i32;
		y &= LineSubpixel::Mask as i32;
		let ptr = &buf[y_lr as usize][x_lr as usize] ;

		weight = ((LineSubpixel::Scale as i32 - x) * 
				 (LineSubpixel::Scale as i32 - y)) as u32;
		r += weight * ptr.r().into_u32();
		g += weight * ptr.g().into_u32();
		b += weight * ptr.b().into_u32();
		a += weight * ptr.a().into_u32();

		let ptr = &buf[y_lr as usize][(x_lr + 1) as usize];

		weight = (x * (LineSubpixel::Scale as i32 - y)) as u32;
		r += weight * ptr.r().into_u32();
		g += weight * ptr.g().into_u32();
		b += weight * ptr.b().into_u32();
		a += weight * ptr.a().into_u32();

		let ptr = &buf[(y_lr + 1) as usize][x_lr as usize];

		weight = ((LineSubpixel::Scale as i32 - x) * y) as u32;
		r += weight * ptr.r().into_u32();
		g += weight * ptr.g().into_u32();
		b += weight * ptr.b().into_u32();
		a += weight * ptr.a().into_u32();

		let ptr = &buf[(y_lr + 1) as usize][(x_lr + 1) as usize];

		weight = (x * y) as u32;
		r += weight * ptr.r().into_u32();
		g += weight * ptr.g().into_u32();
		b += weight * ptr.b().into_u32();
		a += weight * ptr.a().into_u32();

		*p.r_mut() = from_u32!(r >> LineSubpixel::Shift as i32 * 2);
		*p.g_mut() = from_u32!(g >> LineSubpixel::Shift as i32 * 2);
		*p.b_mut() = from_u32!(b >> LineSubpixel::Shift as i32 * 2);
		*p.a_mut() = from_u32!(a >> LineSubpixel::Shift as i32 * 2);
	}
}


pub type PatternFilterBilinearRgba8 = PatternFilterBilinearRgba<Rgba8>;
pub type PatternFilterBilinearRgba16 = PatternFilterBilinearRgba<Rgba16>;