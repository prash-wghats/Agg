#![allow(dead_code)]
use agg::basics::{is_stop, PathCmd, PathFlag};
use agg::conv_stroke::ConvStroke;
use agg::ellipse::Ellipse;
use agg::VertexSource;

pub struct PolySrc {
    polygon: *const f64,
    num_points: usize,
    vertex: usize,
    roundoff: bool,
    close: bool,
}

impl PolySrc {
    pub fn new(polygon: *const f64, np: usize, roundoff: bool, close: bool) -> PolySrc {
        PolySrc {
            polygon: polygon,
            num_points: np,
            vertex: 0,
            roundoff: roundoff,
            close: close,
        }
    }

    pub fn set_close(&mut self, f: bool) {
        self.close = f;
    }

    pub fn is_close(&self) -> bool {
        self.close
    }
}

impl VertexSource for PolySrc {
    fn rewind(&mut self, _: u32) {
        self.vertex = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.vertex > self.num_points {
            return PathCmd::Stop as u32;
        }
        if self.vertex == self.num_points {
            self.vertex += 1;
            return PathCmd::EndPoly as u32
                | (if self.close {
                    PathFlag::Close as u32
                } else {
                    0
                });
        }
        unsafe {
            *x = *self.polygon.offset((self.vertex * 2) as isize);
            *y = *self.polygon.offset((self.vertex * 2 + 1) as isize);

            if self.roundoff {
                *x = (*x).floor() + 0.5;
                *y = (*y).floor() + 0.5;
            }
        }
        self.vertex += 1;
        if self.vertex == 1 {
            PathCmd::MoveTo as u32
        } else {
            PathCmd::LineTo as u32
        }
    }
}

pub struct InteractivePolygon<'a> {
    polygon: Vec<f64>,
    num_points: usize,
    node: i32,
    edge: i32,
    stroke: ConvStroke<'a, PolySrc>,
    ellipse: Ellipse,
    point_radius: f64,
    status: usize,
    dx: f64,
    dy: f64,
}

impl<'a> VertexSource for InteractivePolygon<'a> {
    fn rewind(&mut self, _: u32) {
        self.status = 0;
        self.stroke.rewind(0);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd;
        let mut r = self.point_radius;
        if self.status == 0 {
            cmd = self.stroke.vertex(x, y);
            if !is_stop(cmd) {
                return cmd;
            }
            if self.node >= 0 && self.node as usize == self.status {
                r *= 1.2;
            }
            self.ellipse
                .init(self.xn(self.status), self.yn(self.status), r, r, 32, false);
            self.status += 1;
        }
        cmd = self.ellipse.vertex(x, y);
        if !is_stop(cmd) {
            return cmd;
        }
        if self.status >= self.num_points {
            return PathCmd::Stop as u32;
        }
        if self.node >= 0 && self.node as usize == self.status {
            r *= 1.2;
        }
        self.ellipse
            .init(self.xn(self.status), self.yn(self.status), r, r, 32, false);
        self.status += 1;
        self.ellipse.vertex(x, y)
    }
}

impl<'a> InteractivePolygon<'a> {
    pub fn new(np: usize, point_radius: f64) -> Self {
        let mut v = vec![0.0; np * 2];
        let vs = PolySrc::new(v.as_mut_ptr(), np, false, true);
        InteractivePolygon {
            polygon: v,
            num_points: np,
            node: -1,
            edge: -1,

            stroke: ConvStroke::new_owned(vs),
            ellipse: Ellipse::new(),
            point_radius: point_radius,
            status: 0,
            dx: 0.0,
            dy: 0.0,
        }
    }

    pub fn num_points(&self) -> usize {
        self.num_points
    }

    pub fn xn(&self, n: usize) -> f64 {
        self.polygon[n * 2]
    }

    pub fn yn(&self, n: usize) -> f64 {
        self.polygon[n * 2 + 1]
    }

    pub fn xn_mut(&mut self, n: usize) -> &mut f64 {
        &mut self.polygon[n * 2]
    }

    pub fn yn_mut(&mut self, n: usize) -> &mut f64 {
        &mut self.polygon[n * 2 + 1]
    }

    pub fn polygon(&self) -> &[f64] {
        &self.polygon
    }

    pub fn node(&self) -> i32 {
        self.node
    }

    pub fn set_node(&mut self, n: i32) {
        self.node = n;
    }

    pub fn set_close(&mut self, f: bool) {
        self.stroke.source_mut().set_close(f);
    }

    pub fn is_close(&self) -> bool {
        self.stroke.source().is_close()
    }

    pub fn check_edge(&self, i: usize, x: f64, y: f64) -> bool {
        let mut ret = false;

        let n1 = i;
        let n2 = (i + self.num_points - 1) % self.num_points;
        let x1 = self.xn(n1);
        let y1 = self.yn(n1);
        let x2 = self.xn(n2);
        let y2 = self.yn(n2);

        let dx = x2 - x1;
        let dy = y2 - y1;

        if (dx * dx + dy * dy).sqrt() > 0.0000001 {
            let x3 = x;
            let y3 = y;
            let x4 = x3 - dy;
            let y4 = y3 + dx;

            let den = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
            let u1 = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / den;

            let xi = x1 + u1 * (x2 - x1);
            let yi = y1 + u1 * (y2 - y1);

            let dx = xi - x;
            let dy = yi - y;

            if u1 > 0.0 && u1 < 1.0 && (dx * dx + dy * dy).sqrt() <= self.point_radius {
                ret = true;
            }
        }
        ret
    }

    pub fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
        let mut i: usize = 0;
        let mut ret: bool = false;
        self.node = -1;
        self.edge = -1;
        while i < self.num_points {
            if f64::sqrt(
                (x - self.xn(i)) * (x - self.xn(i)) + (y - self.yn(i)) * (y - self.yn(i)),
            ) < self.point_radius
            {
                self.dx = x - self.xn(i);
                self.dy = y - self.yn(i);
                self.node = i as i32;
                ret = true;
                break;
            }
            i += 1;
        }
        if !ret {
            i = 0;
            while i < self.num_points {
                if self.check_edge(i, x, y) {
                    self.dx = x;
                    self.dy = y;
                    self.edge = i as i32;
                    ret = true;
                    break;
                }
                i += 1;
            }
        }
        if !ret {
            if self.point_in_polygon(x, y) {
                self.dx = x;
                self.dy = y;
                self.node = self.num_points as i32;
                ret = true;
            }
        }
        ret
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) -> bool {
        let mut ret: bool = false;
        let dx: f64;
        let dy: f64;
        if self.node == self.num_points as i32 {
            dx = x - self.dx;
            dy = y - self.dy;
            let mut i: usize = 0;
            while i < self.num_points {
                *self.xn_mut(i) = self.xn(i) + dx;
                *self.yn_mut(i) = self.yn(i) + dy;
                i += 1;
            }
            self.dx = x;
            self.dy = y;
            ret = true;
        } else {
            if self.edge >= 0 {
                let n1: usize = self.edge as usize;
                let n2: usize = (n1 + self.num_points - 1) % self.num_points;
                dx = x - self.dx;
                dy = y - self.dy;
                *self.xn_mut(n1) = self.xn(n1) + dx;
                *self.yn_mut(n1) = dy + self.yn(n1);
                *self.xn_mut(n2) = dx + self.xn(n2);
                *self.yn_mut(n2) = dy + self.yn(n2);
                self.dx = x;
                self.dy = y;
                ret = true;
            } else {
                if self.node >= 0 {
                    *self.xn_mut(self.node as usize) = x - self.dx;
                    *self.yn_mut(self.node as usize) = y - self.dy;
                    ret = true;
                }
            }
        }
        ret
    }

    pub fn on_mouse_button_up(&mut self, _x: f64, _y: f64) -> bool {
        let ret: bool = (self.node >= 0) || (self.edge >= 0);
        self.node = -1;
        self.edge = -1;
        ret
    }

    fn point_in_polygon(&self, tx: f64, ty: f64) -> bool {
        if self.num_points < 3 {
            return false;
        }
        let mut j: usize = 0;
        let mut yflag0: i32;
        let mut yflag1: i32;
        let mut inside_flag: i32;
        let mut vtx0: f64;
        let mut vty0: f64;
        let mut vtx1: f64;
        let mut vty1: f64;
        vtx0 = self.xn(self.num_points - 1);
        vty0 = self.yn(self.num_points - 1);
        yflag0 = if vty0 >= ty { 1 } else { 0 };
        vtx1 = self.xn(0);
        vty1 = self.yn(0);
        inside_flag = 0;
        while j <= self.num_points {
            yflag1 = if vty1 >= ty { 1 } else { 0 };
            if yflag0 != yflag1 {
                if ((vty1 - ty) * (vtx0 - vtx1) >= (vtx1 - tx) * (vty0 - vty1)) == (yflag1 != 0) {
                    inside_flag ^= 1;
                }
            }
            yflag0 = yflag1;
            vtx0 = vtx1;
            vty0 = vty1;
            let k: usize = if j >= self.num_points {
                j - self.num_points
            } else {
                j
            };
            vtx1 = self.xn(k);
            vty1 = self.yn(k);
            j += 1;
        }
        inside_flag != 0
    }
}
