use crate::ctrl::CtrlBase;
use agg::basics::{PathCmd, PathFlag, is_stop};
use agg::color_rgba::Rgba;
use agg::conv_stroke::ConvStroke;
use agg::ellipse::Ellipse;

use agg::{Color, VertexSource};
use super::{Ctrl, CtrlColor};
use std::ops::{Deref, DerefMut};

pub struct SimplePolygonVertexSource {
    polygon: *const f64,
    num_points: u32,
    vertex: u32,
    roundoff: bool,
    close: bool,
}

impl SimplePolygonVertexSource {
    pub fn new(
        polygon: *const f64, num_points: u32, roundoff: bool, close: bool,
    ) -> SimplePolygonVertexSource {
        SimplePolygonVertexSource {
            polygon: polygon,
            num_points: num_points,
            vertex: 0,
            roundoff: roundoff,
            close: close,
        }
    }

    pub fn set_close(&mut self, f: bool) {
        self.close = f;
    }

    pub fn close(&self) -> bool {
        self.close
    }
}

impl VertexSource for SimplePolygonVertexSource {
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
                    PathFlag::None as u32
                });
        }
        unsafe {
            *x = *self.polygon.offset((self.vertex * 2) as isize);
            *y = *self.polygon.offset((self.vertex * 2 + 1) as isize);
        }
        if self.roundoff {
            *x = (*x).floor() + 0.5;
            *y = (*y).floor() + 0.5;
        }
        self.vertex += 1;
        if self.vertex == 1 {
            PathCmd::MoveTo as u32
        } else {
            PathCmd::LineTo as u32
        }
    }
}

pub struct Polygon<'a, C: Color> {
    ctrl: CtrlBase,
    polygon: Vec<f64>,
    num_points: u32,
    node: i32,
    edge: i32,
    stroke: ConvStroke<'a, SimplePolygonVertexSource>,
    ellipse: Ellipse,
    point_radius: f64,
    status: u32,
    dx: f64,
    dy: f64,
    in_polygon_check: bool,
    line_color: C,
}

impl<'a, C: Color> Deref for Polygon<'a, C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.ctrl
    }
}

impl<'a, C: Color> DerefMut for Polygon<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctrl
    }
}

impl<'a, C: Color> Polygon<'a, C> {
    pub fn new(np: u32, point_radius: f64) -> Self {
        let mut vp = vec![0.; (np * 2) as usize];
        let vs = SimplePolygonVertexSource::new(vp.as_mut_ptr() as *const f64, np, false, true);
        Self {
            ctrl: CtrlBase::new(0., 0., 1., 1., false),
            polygon: vp,
            num_points: np,
            node: -1,
            edge: -1,
            stroke: ConvStroke::new_owned(vs),
            ellipse: Ellipse::new(),
            point_radius: point_radius,
            status: 0,
            dx: 0.0,
            dy: 0.0,
            in_polygon_check: true,
            line_color: C::new_from_rgba(&Rgba::new_params(1.0, 1.0, 0.9, 1.0)),
        }
    }
    pub fn set_line_color(&mut self, c: C) {
        self.line_color = c;
    }
    pub fn xn(&self, n: u32) -> f64 {
        self.polygon[(n * 2) as usize]
    }

    pub fn yn(&self, n: u32) -> f64 {
        self.polygon[(n * 2 + 1) as usize]
    }

    pub fn xn_mut(&mut self, n: u32) -> &mut f64 {
        &mut self.polygon[(n * 2) as usize]
    }

    pub fn yn_mut(&mut self, n: u32) -> &mut f64 {
        &mut self.polygon[(n * 2 + 1) as usize]
    }

    pub fn polygon(&self) -> &[f64] {
        &self.polygon
    }

    pub fn set_line_width(&mut self, w: f64) {
        self.stroke.set_width(w);
    }

    pub fn line_width(&self) -> f64 {
        self.stroke.width()
    }

    pub fn set_point_radius(&mut self, r: f64) {
        self.point_radius = r;
    }

    pub fn point_radius(&self) -> f64 {
        self.point_radius
    }

    pub fn set_polygon_check(&mut self, f: bool) {
        self.in_polygon_check = f;
    }

    pub fn in_polygon_check(&self) -> bool {
        self.in_polygon_check
    }

    pub fn set_close(&mut self, f: bool) {
        self.stroke.source_mut().set_close(f);
    }

    pub fn close(&self) -> bool {
        self.stroke.source().close()
    }

    pub fn num_points(&self) -> u32 {
        self.num_points
    }

    pub fn check_edge(&self, i: u32, x: f64, y: f64) -> bool {
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

    //======= Crossings Multiply algorithm of InsideTest ========================
    //
    // By Eric Haines, 3D/Eye Inc, erich@eye.com
    //
    // This version is usually somewhat faster than the original published in
    // Graphics Gems IV; by turning the division for testing the X axis crossing
    // into a tricky multiplication test this part of the test became faster,
    // which had the additional effect of making the test for "both to left or
    // both to right" a bit slower for triangles than simply computing the
    // intersection each time.  The main increase is in triangle testing speed,
    // which was about 15% faster; all other polygon complexities were pretty much
    // the same as before.  On machines where division is very expensive (not the
    // case on the HP 9000 series on which I tested) this test should be much
    // faster overall than the old code.  Your mileage may (in fact, will) vary,
    // depending on the machine and the test data, but in general I believe this
    // code is both shorter and faster.  This test was inspired by unpublished
    // Graphics Gems submitted by Joseph Samosky and Mark Haigh-Hutchinson.
    // Related work by Samosky is in:
    //
    // Samosky, Joseph, "SectionView: A system for interactively specifying and
    // visualizing sections through three-dimensional medical image data",
    // M.S. Thesis, Department of Electrical Engineering and Computer Science,
    // Massachusetts Institute of Technology, 1993.
    //
    // Shoot a test ray along +X axis.  The strategy is to compare vertex Y values
    // to the testing point's Y and quickly discard edges which are entirely to one
    // side of the test ray.  Note that CONVEX and WINDING code can be added as
    // for the CrossingsTest() code; it is left out here for clarity.
    //
    // Input 2D polygon _pgon_ with _numverts_ number of vertices and test point
    // _point_, returns 1 if inside, 0 if outside.
    fn point_in_polygon(&self, tx: f64, ty: f64) -> bool {
        if self.num_points < 3 {
            return false;
        }
        if !self.in_polygon_check {
            return false;
        }

        let mut j = 1;
        // get test bit for above/below X axis
        let mut yflag0 = (self.yn(self.num_points - 1) >= ty) as i32;
        let mut vtx0 = self.xn(self.num_points - 1);
        let mut vty0 = self.yn(self.num_points - 1);
        let mut vtx1 = self.xn(0);
        let mut vty1 = self.yn(0);
        let mut inside_flag = 0;
        while j <= self.num_points {
            let yflag1 = (vty1 >= ty) as i32;
            // Check if endpoints straddle (are on opposite sides) of X axis
            // (i.e. the Y's differ); if so, +X ray could intersect this edge.
            // The old test also checked whether the endpoints are both to the
            // right or to the left of the test point.  However, given the faster
            // intersection point computation used below, this test was found to
            // be a break-even proposition for most polygons and a loser for
            // triangles (where 50% or more of the edges which survive this test
            // will cross quadrants and so have to have the X intersection computed
            // anyway).  I credit Joseph Samosky with inspiring me to try dropping
            // the "both left or both right" part of my code.
            if yflag0 != yflag1 {
                // Check intersection of pgon segment with +X ray.
                // Note if >= point's X; if so, the ray hits it.
                // The division operation is avoided for the ">=" test by checking
                // the sign of the first vertex wrto the test point; idea inspired
                // by Joseph Samosky's and Mark Haigh-Hutchinson's different
                // polygon inclusion tests.
                if ((vty1 - ty) * (vtx0 - vtx1) >= (vtx1 - tx) * (vty0 - vty1)) == (yflag1 != 0) {
                    inside_flag ^= 1;
                }
            }

            // Move to the next pair of vertices, retaining info as possible.
            yflag0 = yflag1;
            vtx0 = vtx1;
            vty0 = vty1;
            let k = if j >= self.num_points {
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

impl<'a, C: Color> Ctrl for Polygon<'a, C> {
    fn num_paths(&self) -> u32 {
        1
    }

	fn set_transform(&mut self, mtx: &agg::TransAffine) {
		self.ctrl.set_transform(&mtx);
	}

    fn in_rect(&self, _x: f64, _y: f64) -> bool {
        false
    }

    fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
        let (mut x, mut y) = (x, y);
        let mut ret = false;
        self.node = -1;
        self.edge = -1;
        self.inverse_transform_xy(&mut x, &mut y);
        for i in 0..self.num_points {
            if (x - self.xn(i)) * (x - self.xn(i)) + (y - self.yn(i)) * (y - self.yn(i)).sqrt()
                < self.point_radius
            {
                self.dx = x - self.xn(i);
                self.dy = y - self.yn(i);
                self.node = i as i32;
                ret = true;
                break;
            }
        }

        if !ret {
            for i in 0..self.num_points {
                if self.check_edge(i, x, y) {
                    self.dx = x;
                    self.dy = y;
                    self.edge = i as i32;
                    ret = true;
                    break;
                }
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

    fn on_mouse_move(&mut self, x: f64, y: f64, _button_flag: bool) -> bool {
        let mut ret = false;
        let dx;
        let dy;
        let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        if self.node == self.num_points as i32 {
            dx = x - self.dx;
            dy = y - self.dy;

            for i in 0..self.num_points {
                *self.xn_mut(i) += dx;
                *self.yn_mut(i) += dy;
            }
            self.dx = x;
            self.dy = y;
            ret = true;
        } else {
            if self.edge >= 0 {
                let n1 = self.edge as u32;
                let n2 = (n1 + self.num_points - 1) % self.num_points;
                dx = x - self.dx;
                dy = y - self.dy;
                *self.xn_mut(n1) += dx;
                *self.yn_mut(n1) += dy;
                *self.xn_mut(n2) += dx;
                *self.yn_mut(n2) += dy;
                self.dx = x;
                self.dy = y;
                ret = true;
            } else {
                if self.node >= 0 {
                    *self.xn_mut(self.node as u32) = x - self.dx;
                    *self.yn_mut(self.node as u32) = y - self.dy;
                    ret = true;
                }
            }
        }
        return ret;
    }

    fn on_mouse_button_up(&mut self, _x: f64, _y: f64) -> bool {
        let ret = (self.node >= 0) || (self.edge >= 0);
        self.node = -1;
        self.edge = -1;
        ret
    }

    fn on_arrow_keys(&mut self, _left: bool, _right: bool, _down: bool, _up: bool) -> bool {
        false
    }
}

impl<'a, C: Color> VertexSource for Polygon<'a, C> {
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
                self.transform_xy(x, y);
                return cmd;
            }
            if self.node >= 0 && self.node == self.status as i32 {
                r *= 1.2;
            }
            self.ellipse
                .init(self.xn(self.status), self.yn(self.status), r, r, 32, false);
            self.status += 1;
        }
        cmd = self.ellipse.vertex(x, y);
        if !is_stop(cmd) {
            self.transform_xy(x, y);
            return cmd;
        }
        if self.status >= self.num_points {
            return PathCmd::Stop as u32;
        }
        if self.node >= 0 && self.node == self.status as i32 {
            r *= 1.2;
        }
        self.ellipse
            .init(self.xn(self.status), self.yn(self.status), r, r, 32, false);
        self.status += 1;
        cmd = self.ellipse.vertex(x, y);
        if !is_stop(cmd) {
            self.transform_xy(x, y);
        }
        cmd
    }
}

impl<'a, C: Color> CtrlColor for Polygon<'a, C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            _ => self.line_color,
        }
    }
}
