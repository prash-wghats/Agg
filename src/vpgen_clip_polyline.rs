use crate::basics::{PathCmd, RectD};
use crate::{clip_liang_barsky::*, VpGenerator};

/// XXXXX NEEDS TESTING
//======================================================VpgenClipPolyline
pub struct VpgenClipPolyline {
    clip_box: RectD,
    x1: f64,
    y1: f64,
    num_vertices: usize,
    vertex: usize,
    move_to: bool,
    x: [f64; 2],
    y: [f64; 2],
    cmd: [u32; 2],
}

impl VpgenClipPolyline {
    pub fn clip_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.clip_box.x1 = x1;
        self.clip_box.y1 = y1;
        self.clip_box.x2 = x2;
        self.clip_box.y2 = y2;
        self.clip_box.normalize();
    }

    pub fn x1(&self) -> f64 {
        return self.clip_box.x1;
    }

    pub fn y1(&self) -> f64 {
        return self.clip_box.y1;
    }

    pub fn x2(&self) -> f64 {
        return self.clip_box.x2;
    }

    pub fn y2(&self) -> f64 {
        return self.clip_box.y2;
    }
}

impl VpGenerator for VpgenClipPolyline {
    fn new() -> VpgenClipPolyline {
        VpgenClipPolyline {
            clip_box: RectD {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
            x1: 0.0,
            y1: 0.0,
            num_vertices: 0,
            vertex: 0,
            move_to: false,
            x: [0.0; 2],
            y: [0.0; 2],
            cmd: [0; 2],
        }
    }
    fn reset(&mut self) {}
    fn move_to(&mut self, x: f64, y: f64) {
        self.vertex = 0;
        self.num_vertices = 0;
        self.x1 = x;
        self.y1 = y;
        self.move_to = true;
    }

    fn line_to(&mut self, x: f64, y: f64) {
        let mut x2 = x;
        let mut y2 = y;
        let flags = clip_line_segment(&mut self.x1, &mut self.y1, &mut x2, &mut y2, &self.clip_box);

        self.vertex = 0;
        self.num_vertices = 0;
        if (flags & 4) == 0 {
            if (flags & 1) != 0 || self.move_to {
                self.x[0] = self.x1;
                self.y[0] = self.y1;
                self.cmd[0] = PathCmd::MoveTo as u32;
                self.num_vertices = 1;
            }
            self.x[self.num_vertices] = x2;
            self.y[self.num_vertices] = y2;
            self.cmd[self.num_vertices] = PathCmd::LineTo as u32;
            self.num_vertices += 1;
            self.move_to = (flags & 2) != 0;
        }
        self.x1 = x;
        self.y1 = y;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.vertex < self.num_vertices {
            *x = self.x[self.vertex];
            *y = self.y[self.vertex];
            let cmd = self.cmd[self.vertex];
            self.vertex += 1;
            return cmd;
        }
        return PathCmd::Stop as u32;
    }

    fn auto_close(&self) -> bool {
        return false;
    }

    fn auto_unclose(&self) -> bool {
        return true;
    }
}
