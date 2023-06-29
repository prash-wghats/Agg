use crate::{Transformer};

/// NOT TESTED
#[derive(Clone, Copy)]
pub struct TransWarpMagnifier {
	xc: f64,
	yc: f64,
	magn: f64,
	radius: f64,
}

impl TransWarpMagnifier {
	pub fn new() -> TransWarpMagnifier {
		TransWarpMagnifier {
			xc: 0.0,
			yc: 0.0,
			magn: 1.0,
			radius: 1.0,
		}
	}

	pub fn center(&mut self, x: f64, y: f64) {
		self.xc = x;
		self.yc = y;
	}

	pub fn magnification(&mut self, m: f64) {
		self.magn = m;
	}

	pub fn radius(&mut self, r: f64) {
		self.radius = r;
	}

	pub fn xc(&self) -> f64 {
		self.xc
	}

	pub fn yc(&self) -> f64 {
		self.yc
	}

	pub fn get_magnification(&self) -> f64 {
		self.magn
	}

	pub fn get_radius(&self) -> f64 {
		self.radius
	}

	pub fn inverse_transform(&self, x: &mut f64, y: &mut f64) {
		let dx = *x - self.xc;
		let dy = *y - self.yc;
		let r = (dx * dx + dy * dy).sqrt();

		if r < self.radius * self.magn {
			*x = self.xc + dx / self.magn;
			*y = self.yc + dy / self.magn;
		} else {
			let rnew = r - self.radius * (self.magn - 1.0);
			*x = self.xc + rnew * dx / r;
			*y = self.yc + rnew * dy / r;
		}
	}
}

impl Transformer for TransWarpMagnifier {
	fn transform(&self, x: &mut f64, y: &mut f64) {
		let dx = *x - self.xc;
		let dy = *y - self.yc;
		let r = (dx * dx + dy * dy).sqrt();
		if r < self.radius {
			*x = self.xc + dx * self.magn;
			*y = self.yc + dy * self.magn;
			return;
		}

		let m = (r + self.radius * (self.magn - 1.0)) / r;
		*x = self.xc + dx * m;
		*y = self.yc + dy * m;
	}

}