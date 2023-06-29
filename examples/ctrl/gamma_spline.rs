use agg::basics::PathCmd;
use agg::bspline::Bspline;
use agg::VertexSource;

pub struct GammaSpline {
    gamma: [u8; 256],
    x: [f64; 4],
    y: [f64; 4],
    spline: Bspline,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    cur_x: f64,
}

impl GammaSpline {
    pub fn new() -> GammaSpline {
        GammaSpline {
            gamma: [0; 256],
            x: [0.0; 4],
            y: [0.0; 4],
            spline: Bspline::new(),
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            cur_x: 0.0,
        }
    }

    pub fn gamma(&self) -> &[u8] {
        &self.gamma
    }

    pub fn y(&self, x: f64) -> f64 {
        let mut x = x;
        if x < 0.0 {
            x = 0.0;
        }
        if x > 1.0 {
            x = 1.0;
        }
        let mut val = self.spline.get(x);
        if val < 0.0 {
            val = 0.0;
        }
        if val > 1.0 {
            val = 1.0;
        }
        val
    }

    pub fn set_values(&mut self, kx1: f64, ky1: f64, kx2: f64, ky2: f64) {
        let (mut kx1, mut ky1, mut kx2, mut ky2) = (kx1, ky1, kx2, ky2);
        if kx1 < 0.001 {
            kx1 = 0.001;
        }
        if kx1 > 1.999 {
            kx1 = 1.999;
        }
        if ky1 < 0.001 {
            ky1 = 0.001;
        }
        if ky1 > 1.999 {
            ky1 = 1.999;
        }
        if kx2 < 0.001 {
            kx2 = 0.001;
        }
        if kx2 > 1.999 {
            kx2 = 1.999;
        }
        if ky2 < 0.001 {
            ky2 = 0.001;
        }
        if ky2 > 1.999 {
            ky2 = 1.999;
        }

        self.x[0] = 0.0;
        self.y[0] = 0.0;
        self.x[1] = kx1 * 0.25;
        self.y[1] = ky1 * 0.25;
        self.x[2] = 1.0 - kx2 * 0.25;
        self.y[2] = 1.0 - ky2 * 0.25;
        self.x[3] = 1.0;
        self.y[3] = 1.0;

        self.spline.init_with_points(4, &self.x, &self.y);

        for i in 0..256 {
            self.gamma[i] = (self.y(i as f64 / 255.0) * 255.0) as u8;
        }
    }

    pub fn values(&self, kx1: &mut f64, ky1: &mut f64, kx2: &mut f64, ky2: &mut f64) {
        *kx1 = self.x[1] * 4.0;
        *ky1 = self.y[1] * 4.0;
        *kx2 = (1.0 - self.x[2]) * 4.0;
        *ky2 = (1.0 - self.y[2]) * 4.0;
    }

    pub fn set_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.x1 = x1;
        self.y1 = y1;
        self.x2 = x2;
        self.y2 = y2;
    }
}

impl VertexSource for GammaSpline {
    fn rewind(&mut self, _: u32) {
        self.cur_x = 0.0;
    }

    fn vertex(&mut self, vx: &mut f64, vy: &mut f64) -> u32 {
        if self.cur_x == 0.0 {
            *vx = self.x1;
            *vy = self.y1;
            self.cur_x += 1.0 / (self.x2 - self.x1);
            return PathCmd::MoveTo as u32;
        }

        if self.cur_x > 1.0 {
            return PathCmd::Stop as u32;
        }

        *vx = self.x1 + self.cur_x * (self.x2 - self.x1);
        *vy = self.y1 + self.y(self.cur_x) * (self.y2 - self.y1);

        self.cur_x += 1.0 / (self.x2 - self.x1);
        return PathCmd::LineTo as u32;
    }
}
