use std::ops::{Deref, DerefMut};

use crate::{Interpolator, Distortion};


//===============================================span_interpolator_adaptor
pub struct SpanIpAdaptor<I: Interpolator, D: Distortion> {
	base_type: I,
	distortion: D,
}

impl<I: Interpolator, D: Distortion> SpanIpAdaptor<I, D> {
	pub fn new(trans: I, dist: D) -> Self {
		SpanIpAdaptor {
			base_type: trans,
			distortion: dist,
		}
	}

	pub fn distortion(&self) -> &D {
		&self.distortion
	}

	pub fn distortion_mut(&mut self) -> &mut D {
		&mut self.distortion
	}
}

impl<I: Interpolator, D: Distortion> Interpolator for SpanIpAdaptor<I, D> {
    type Trf = I::Trf;
    const SUBPIXEL_SHIFT: u32 = 8;

    fn begin(&mut self, x: f64, y: f64, len: u32){self.base_type.begin(x, y, len)}
    fn next(&mut self) {self.base_type.next()}
    fn coordinates(&self, x: &mut i32, y: &mut i32) {
		self.base_type.coordinates(x, y);
		self.distortion.calculate(x, y);
	}
}

impl<I: Interpolator, D: Distortion> Deref for SpanIpAdaptor<I, D> {
    type Target = I;
    fn deref(&self) -> &I {
        &self.base_type
    }
}

impl<I: Interpolator, D: Distortion> DerefMut for SpanIpAdaptor<I, D> {
    fn deref_mut(&mut self) -> &mut I {
        &mut self.base_type
    }
}