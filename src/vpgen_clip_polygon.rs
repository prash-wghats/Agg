use crate::basics::{PathCmd, RectD};
use crate::{clip_liang_barsky::*, VpGenerator};

//======================================================vpgen_clip_polygon
pub struct VpgenClipPolygon {
    clip_box: RectD,
    x1: f64,
    y1: f64,
    clip_flags: u32,
    num_vertices: usize,
    vertex: usize,
    cmd: u32,
    x: [f64; 4],
    y: [f64; 4],
}

impl VpgenClipPolygon {
    pub fn clip_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.clip_box.x1 = x1;
        self.clip_box.y1 = y1;
        self.clip_box.x2 = x2;
        self.clip_box.y2 = y2;
        self.clip_box.normalize();
    }

    //------------------------------------------------------------------------
    // Determine the clipping code of the vertex according to the
    // Cyrus-Beck line clipping algorithm
    //
    //        |        |
    //  0110  |  0010  | 0011
    //        |        |
    // -------+--------+-------- clip_box.y2
    //        |        |
    //  0100  |  0000  | 0001
    //        |        |
    // -------+--------+-------- clip_box.y1
    //        |        |
    //  1100  |  1000  | 1001
    //        |        |
    //  clip_box.x1  clip_box.x2
    //
    //
    pub fn clipping_flags(&self, x: f64, y: f64) -> u32 {
        if x < self.clip_box.x1 {
            if y > self.clip_box.y2 {
                return 6;
            }
            if y < self.clip_box.y1 {
                return 12;
            }
            return 4;
        }

        if x > self.clip_box.x2 {
            if y > self.clip_box.y2 {
                return 3;
            }
            if y < self.clip_box.y1 {
                return 9;
            }
            return 1;
        }

        if y > self.clip_box.y2 {
            return 2;
        }
        if y < self.clip_box.y1 {
            return 8;
        }

        return 0;
    }

    pub fn x1(&self) -> f64 {
        self.clip_box.x1
    }

    pub fn y1(&self) -> f64 {
        self.clip_box.y1
    }

    pub fn x2(&self) -> f64 {
        self.clip_box.x2
    }

    pub fn y2(&self) -> f64 {
        self.clip_box.y2
    }
}

impl VpGenerator for VpgenClipPolygon {
    fn new() -> VpgenClipPolygon {
        VpgenClipPolygon {
            clip_box: RectD::new(0.0, 0.0, 1.0, 1.0),
            x1: 0.0,
            y1: 0.0,
            clip_flags: 0,
            num_vertices: 0,
            vertex: 0,
            cmd: PathCmd::MoveTo as u32,
            x: [0.0; 4],
            y: [0.0; 4],
        }
    }

    fn reset(&mut self) {
        self.vertex = 0;
        self.num_vertices = 0;
    }

    fn move_to(&mut self, x: f64, y: f64) {
        self.vertex = 0;
        self.num_vertices = 0;
        self.clip_flags = self.clipping_flags(x, y);
        if self.clip_flags == 0 {
            self.x[0] = x;
            self.y[0] = y;
            self.num_vertices = 1;
        }
        self.x1 = x;
        self.y1 = y;
        self.cmd = PathCmd::MoveTo as u32;
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.vertex = 0;
        self.num_vertices = 0;
        let flags = self.clipping_flags(x, y);

        if self.clip_flags == flags {
            if flags == 0 {
                self.x[0] = x;
                self.y[0] = y;
                self.num_vertices = 1;
            }
        } else {
            self.num_vertices = clip_liang_barsky(
                self.x1,
                self.y1,
                x,
                y,
                &self.clip_box,
                &mut self.x,
                &mut self.y,
            );
        }

        self.clip_flags = flags;
        self.x1 = x;
        self.y1 = y;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.vertex < self.num_vertices {
            *x = self.x[self.vertex as usize];
            *y = self.y[self.vertex as usize];
            self.vertex += 1;
            let cmd = self.cmd;
            self.cmd = PathCmd::LineTo as u32;
            return cmd;
        }
        return PathCmd::Stop as u32;
    }

    fn auto_close(&self) -> bool {
        true
    }

    fn auto_unclose(&self) -> bool {
        false
    }
}
