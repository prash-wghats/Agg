use crate::basics::PathCmd;
use crate::VpGenerator;

//=======================================================vpgen_segmentator
pub struct VpgenSegmentator {
    approximation_scale: f64,
    x1: f64,
    y1: f64,
    dx: f64,
    dy: f64,
    dl: f64,
    ddl: f64,
    cmd: PathCmd,
}

impl VpgenSegmentator {
    pub fn approximation_scale(&self) -> f64 {
        self.approximation_scale
    }

    pub fn set_approximation_scale(&mut self, s: f64) {
        self.approximation_scale = s;
    }
}

impl VpGenerator for VpgenSegmentator {
    fn new() -> VpgenSegmentator {
        VpgenSegmentator {
            approximation_scale: 1.0,
            x1: 0.0,
            y1: 0.0,
            dx: 0.0,
            dy: 0.0,
            dl: 2.0,
            ddl: 2.0,
            cmd: PathCmd::Stop,
        }
    }

    fn auto_close(&self) -> bool {
        false
    }

    fn auto_unclose(&self) -> bool {
        false
    }

    fn reset(&mut self) {
        self.cmd = PathCmd::Stop;
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.x1 = x;
        self.y1 = y;
        self.dx = 0.0;
        self.dy = 0.0;
        self.dl = 2.0;
        self.ddl = 2.0;
        self.cmd = PathCmd::MoveTo;
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.x1 += self.dx;
        self.y1 += self.dy;
        self.dx = x - self.x1;
        self.dy = y - self.y1;
        let mut len = (self.dx * self.dx + self.dy * self.dy).sqrt() * self.approximation_scale;
        len = if len < 1e-30 { 1e-30 } else { len };
        self.ddl = 1.0 / len;
        self.dl = if self.cmd == PathCmd::MoveTo {
            0.0
        } else {
            self.ddl
        };
        if self.cmd == PathCmd::Stop {
            self.cmd = PathCmd::LineTo;
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.cmd == PathCmd::Stop {
            return PathCmd::Stop as u32;
        }
        let cmd = self.cmd;
        self.cmd = PathCmd::LineTo;
        if self.dl >= 1.0 - self.ddl {
            self.dl = 1.0;
            self.cmd = PathCmd::Stop;
            *x = self.x1 + self.dx;
            *y = self.y1 + self.dy;
            return cmd as u32;
        }
        *x = self.x1 + self.dx * self.dl;
        *y = self.y1 + self.dy * self.dl;
        self.dl += self.ddl;
        return cmd as u32;
    }
}
