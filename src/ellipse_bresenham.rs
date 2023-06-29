pub struct EllipseBresenhamIp {
	rx2: i32,
	ry2: i32,
	two_rx2: i32,
	two_ry2: i32,
	dx: i32,
	dy: i32,
	inc_x: i32,
	inc_y: i32,
	cur_f: i32,
}

impl EllipseBresenhamIp {
	pub fn new(rx: i32, ry: i32) -> EllipseBresenhamIp {
		EllipseBresenhamIp {
			rx2: rx * rx,
			ry2: ry * ry,
			two_rx2: (rx * rx) << 1,
			two_ry2: (ry * ry) << 1,
			dx: 0,
			dy: 0,
			inc_x: 0,
			inc_y: -ry * ((rx * rx) << 1),
			cur_f: 0,
		}
	}

	pub fn dx(&self) -> i32 {
		self.dx
	}

	pub fn dy(&self) -> i32 {
		self.dy
	}

	pub fn inc(&mut self) {
		let mut mx = self.cur_f + self.inc_x + self.ry2;
		let fx = mx;
		if mx < 0 {
			mx = -mx;
		}

		let mut my = self.cur_f + self.inc_y + self.rx2;
		let fy = my;
		if my < 0 {
			my = -my;
		}

		let mut mxy = self.cur_f + self.inc_x + self.ry2 + self.inc_y + self.rx2;
		let fxy = mxy;
		if mxy < 0 {
			mxy = -mxy;
		}

		let mut min_m = mx;
		let mut flag = true;

		if min_m > my {
			min_m = my;
			flag = false;
		}

		self.dx = 0;
		self.dy = 0;

		if min_m > mxy {
			self.inc_x += self.two_ry2;
			self.inc_y += self.two_rx2;
			self.cur_f = fxy;
			self.dx = 1;
			self.dy = 1;
			return;
		}

		if flag {
			self.inc_x += self.two_ry2;
			self.cur_f = fx;
			self.dx = 1;
			return;
		}

		self.inc_y += self.two_rx2;
		self.cur_f = fy;
		self.dy = 1;
	}
}
