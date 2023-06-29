use crate::basics::{PathCmd, PointD};
use crate::math::*;
use crate::{array::*, VertexSource};
use crate::{CurveBase, CurveType3, CurveType4, Point};
use std::f64::consts::PI;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CurveApproximationMethod {
    CurveInc = 0,
    CurveDiv = 1,
}

pub const CURVE_DISTANCE_EPSILON: f64 = 1e-30;
pub const CURVE_COLLINEARITY_EPSILON: f64 = 1e-30;
pub const CURVE_ANGLE_TOLERANCE_EPSILON: f64 = 0.01;
pub const CURVE_RECURSION_EPSILON: u32 = 32;

//--------------------------------------------------------------curve3_inc
pub struct Curve3Inc {
    num_steps: i32,
    step: i32,
    scale: f64,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    fx: f64,
    fy: f64,
    dfx: f64,
    dfy: f64,
    ddfx: f64,
    ddfy: f64,
    saved_fx: f64,
    saved_fy: f64,
    saved_dfx: f64,
    saved_dfy: f64,
}

impl Curve3Inc {
    pub fn new() -> Curve3Inc {
        Curve3Inc {
            num_steps: 0,
            step: 0,
            scale: 1.0,
            start_x: 0.0,
            start_y: 0.0,
            end_x: 0.0,
            end_y: 0.0,
            fx: 0.0,
            fy: 0.0,
            dfx: 0.0,
            dfy: 0.0,
            ddfx: 0.0,
            ddfy: 0.0,
            saved_fx: 0.0,
            saved_fy: 0.0,
            saved_dfx: 0.0,
            saved_dfy: 0.0,
        }
    }

    pub fn new_with_params(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> Curve3Inc {
        let mut curve = Curve3Inc::new();
        curve.init(x1, y1, x2, y2, x3, y3);
        curve
    }

    pub fn approximation_scale(&self) -> f64 {
        self.scale
    }

    pub fn set_approximation_scale(&mut self, s: f64) {
        self.scale = s;
    }

    pub fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        self.start_x = x1;
        self.start_y = y1;
        self.end_x = x3;
        self.end_y = y3;

        let dx1 = x2 - x1;
        let dy1 = y2 - y1;
        let dx2 = x3 - x2;
        let dy2 = y3 - y2;

        let len = (dx1 * dx1 + dy1 * dy1).sqrt() + (dx2 * dx2 + dy2 * dy2).sqrt();

        self.num_steps = (len * 0.25 * self.scale).round() as i32;

        if self.num_steps < 4 {
            self.num_steps = 4;
        }

        let subdivide_step = 1.0 / self.num_steps as f64;
        let subdivide_step2 = subdivide_step * subdivide_step;

        let tmpx = (x1 - x2 * 2.0 + x3) * subdivide_step2;
        let tmpy = (y1 - y2 * 2.0 + y3) * subdivide_step2;

        self.fx = x1;
        self.fy = y1;
        self.saved_fx = x1;
        self.saved_fy = y1;

        self.dfx = tmpx + (x2 - x1) * (2.0 * subdivide_step);
        self.dfy = tmpy + (y2 - y1) * (2.0 * subdivide_step);
        self.saved_dfx = self.dfx;
        self.saved_dfy = self.dfy;

        self.ddfx = tmpx * 2.0;
        self.ddfy = tmpy * 2.0;

        self.step = self.num_steps;
    }

    pub fn rewind(&mut self, _: u32) {
        if self.num_steps == 0 {
            self.step = -1;
            return;
        }
        self.step = self.num_steps;
        self.fx = self.saved_fx;
        self.fy = self.saved_fy;
        self.dfx = self.saved_dfx;
        self.dfy = self.saved_dfy;
    }

    pub fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.step < 0 {
            return PathCmd::Stop as u32;
        }
        if self.step == self.num_steps {
            *x = self.start_x;
            *y = self.start_y;
            self.step -= 1;
            return PathCmd::MoveTo as u32;
        }
        if self.step == 0 {
            *x = self.end_x;
            *y = self.end_y;
            self.step -= 1;
            return PathCmd::LineTo as u32;
        }
        self.fx += self.dfx;
        self.fy += self.dfy;
        self.dfx += self.ddfx;
        self.dfy += self.ddfy;
        *x = self.fx;
        *y = self.fy;
        self.step -= 1;
        return PathCmd::LineTo as u32;
    }

    pub fn reset(&mut self) {
        self.num_steps = 0;
        self.step = -1;
    }

    pub fn set_approximation_method(&self, _: CurveApproximationMethod) {}
    pub fn approximation_method(&self) -> CurveApproximationMethod {
        CurveApproximationMethod::CurveInc
    }

    pub fn set_angle_tolerance(&self, _: f64) {}
    pub fn angle_tolerance_(&self) -> f64 {
        0.0
    }

    pub fn set_cusp_limit(&self, _: f64) {}
    pub fn cusp_limit(&self) -> f64 {
        0.0
    }
}

//-------------------------------------------------------------curve3_div
pub struct Curve3Div {
    pub approximation_scale: f64,
    pub distance_tolerance_square: f64,
    pub angle_tolerance: f64,
    pub count: u32,
    pub points: VecPodB<PointD>,
}

impl Curve3Div {
    pub fn new() -> Curve3Div {
        Curve3Div {
            approximation_scale: 1.0,
            distance_tolerance_square: 0.0,
            angle_tolerance: 0.0,
            count: 0,
            points: Vec::new(),
        }
    }
    pub fn new_with_points(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> Self {
        let mut c = Self::new();
        c.init(x1, y1, x2, y2, x3, y3);
        c
    }
    pub fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        self.points.clear();
        self.distance_tolerance_square = 0.5 / self.approximation_scale;
        self.distance_tolerance_square *= self.distance_tolerance_square;
        self.bezier(x1, y1, x2, y2, x3, y3);
        self.count = 0;
    }

    pub fn recursive_bezier(
        &mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, level: u32,
    ) {
        if level > CURVE_RECURSION_EPSILON {
            return;
        }

        // Calculate all the mid-points of the line segments
        //----------------------
        let x12 = (x1 + x2) / 2.0;
        let y12 = (y1 + y2) / 2.0;
        let x23 = (x2 + x3) / 2.0;
        let y23 = (y2 + y3) / 2.0;
        let x123 = (x12 + x23) / 2.0;
        let y123 = (y12 + y23) / 2.0;

        let dx = x3 - x1;
        let dy = y3 - y1;
        let mut d = ((x2 - x3) * dy - (y2 - y3) * dx).abs();
        let mut da;

        if d > CURVE_COLLINEARITY_EPSILON {
            // Regular case
            //-----------------
            if d * d <= self.distance_tolerance_square * (dx * dx + dy * dy) {
                // If the curvature doesn't exceed the distance_tolerance value
                // we tend to finish subdivisions.
                //----------------------
                if self.angle_tolerance < CURVE_ANGLE_TOLERANCE_EPSILON {
                    self.points.push(PointD::new(x123, y123));
                    return;
                }

                // Angle & Cusp Condition
                //----------------------
                da = (y3 - y2).atan2(x3 - x2) - (y2 - y1).atan2(x2 - x1);
                if da >= PI {
                    da = 2.0 * PI - da;
                }

                if da < self.angle_tolerance {
                    // Finally we can stop the recursion
                    //----------------------
                    self.points.push(PointD::new(x123, y123));
                    return;
                }
            }
        } else {
            // Collinear case
            //------------------
            da = dx * dx + dy * dy;
            if da == 0.0 {
                d = calc_sq_distance(x1, y1, x2, y2);
            } else {
                d = ((x2 - x1) * dx + (y2 - y1) * dy) / da;
                if d > 0.0 && d < 1.0 {
                    // Simple collinear case, 1---2---3
                    // We can leave just two endpoints
                    return;
                }
                if d <= 0.0 {
                    d = calc_sq_distance(x2, y2, x1, y1);
                } else if d >= 1.0 {
                    d = calc_sq_distance(x2, y2, x3, y3);
                } else {
                    d = calc_sq_distance(x2, y2, x1 + d * dx, y1 + d * dy);
                }
            }
            if d < self.distance_tolerance_square {
                self.points.push(PointD::new(x2, y2));
                return;
            }
        }

        // Continue subdivision
        //----------------------
        self.recursive_bezier(x1, y1, x12, y12, x123, y123, level + 1);
        self.recursive_bezier(x123, y123, x23, y23, x3, y3, level + 1);
    }

    pub fn bezier(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        self.points.push(PointD::new(x1, y1));
        self.recursive_bezier(x1, y1, x2, y2, x3, y3, 0);
        self.points.push(PointD::new(x3, y3));
    }

    pub fn reset(&mut self) {
        self.points.clear();
        self.count = 0;
    }

    pub fn set_approximation_method(&mut self, _: CurveApproximationMethod) {}
    pub fn approximation_method(&self) -> CurveApproximationMethod {
        CurveApproximationMethod::CurveDiv
    }

    pub fn set_approximation_scale(&mut self, s: f64) {
        self.approximation_scale = s;
    }
    pub fn approximation_scale(&self) -> f64 {
        self.approximation_scale
    }

    pub fn set_angle_tolerance(&mut self, a: f64) {
        self.angle_tolerance = a;
    }
    pub fn angle_tolerance(&self) -> f64 {
        self.angle_tolerance
    }

    pub fn set_cusp_limit(&mut self, _: f64) {}
    pub fn cusp_limit(&self) -> f64 {
        0.0
    }

    pub fn rewind(&mut self, _: u32) {
        self.count = 0;
    }

    pub fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.count >= self.points.len() as u32 {
            return PathCmd::Stop as u32;
        }
        let p = self.points[self.count as usize];
        *x = p.x;
        *y = p.y;
        self.count += 1;
        if self.count == 1 {
            PathCmd::MoveTo as u32
        } else {
            PathCmd::LineTo as u32
        }
    }
}

//-------------------------------------------------------------Curve4Points
#[derive(Copy, Clone)]
pub struct Curve4Points {
    pub cp: [f64; 8],
}
impl Curve4Points {
    pub fn new(
        x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
    ) -> Curve4Points {
        Curve4Points {
            cp: [x1, y1, x2, y2, x3, y3, x4, y4],
        }
    }
    pub fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64) {
        self.cp = [x1, y1, x2, y2, x3, y3, x4, y4];
    }
    pub fn get(&self, i: usize) -> f64 {
        self.cp[i]
    }
    pub fn set(&mut self, i: usize, val: f64) {
        self.cp[i] = val;
    }
}

impl Index<usize> for Curve4Points {
    type Output = f64;
    fn index(&self, index: usize) -> &Self::Output {
        &self.cp[index]
    }
}

impl IndexMut<usize> for Curve4Points {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cp[index]
    }
}

//-------------------------------------------------------------curve4_inc
pub struct Curve4Inc {
    num_steps: i32,
    step: i32,
    scale: f64,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    fx: f64,
    fy: f64,
    dfx: f64,
    dfy: f64,
    ddfx: f64,
    ddfy: f64,
    dddfx: f64,
    dddfy: f64,
    saved_fx: f64,
    saved_fy: f64,
    saved_dfx: f64,
    saved_dfy: f64,
    saved_ddfx: f64,
    saved_ddfy: f64,
}

impl Curve4Inc {
    pub fn new() -> Curve4Inc {
        Curve4Inc {
            num_steps: 0,
            step: 0,
            scale: 1.0,
            start_x: 0.0,
            start_y: 0.0,
            end_x: 0.0,
            end_y: 0.0,
            fx: 0.0,
            fy: 0.0,
            dfx: 0.0,
            dfy: 0.0,
            ddfx: 0.0,
            ddfy: 0.0,
            dddfx: 0.0,
            dddfy: 0.0,
            saved_fx: 0.0,
            saved_fy: 0.0,
            saved_dfx: 0.0,
            saved_dfy: 0.0,
            saved_ddfx: 0.0,
            saved_ddfy: 0.0,
        }
    }

    pub fn new_with_points(cp: &Curve4Points) -> Curve4Inc {
        let mut curve = Curve4Inc::new();
        curve.init(cp);
        curve
    }

    pub fn new_with_coords(
        x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
    ) -> Curve4Inc {
        let mut curve = Curve4Inc::new();
        curve.init_with_coords(x1, y1, x2, y2, x3, y3, x4, y4);
        curve
    }

    pub fn approximation_scale(&self) -> f64 {
        self.scale
    }

    pub fn set_approximation_scale(&mut self, s: f64) {
        self.scale = s;
    }

    pub fn init_with_coords(
        &mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
    ) {
        self.start_x = x1;
        self.start_y = y1;
        self.end_x = x4;
        self.end_y = y4;

        let dx1 = x2 - x1;
        let dy1 = y2 - y1;
        let dx2 = x3 - x2;
        let dy2 = y3 - y2;
        let dx3 = x4 - x3;
        let dy3 = y4 - y3;

        let len = (dx1 * dx1 + dy1 * dy1).sqrt()
            + (dx2 * dx2 + dy2 * dy2).sqrt()
            + (dx3 * dx3 + dy3 * dy3).sqrt();
        self.num_steps = (len * 0.25 * self.scale) as i32;

        if self.num_steps < 4 {
            self.num_steps = 4;
        }

        let subdivide_step = 1.0 / self.num_steps as f64;
        let subdivide_step2 = subdivide_step * subdivide_step;
        let subdivide_step3 = subdivide_step * subdivide_step * subdivide_step;

        let pre1 = 3.0 * subdivide_step;
        let pre2 = 3.0 * subdivide_step2;
        let pre4 = 6.0 * subdivide_step2;
        let pre5 = 6.0 * subdivide_step3;

        let tmp1x = x1 - x2 * 2.0 + x3;
        let tmp1y = y1 - y2 * 2.0 + y3;

        let tmp2x = (x2 - x3) * 3.0 - x1 + x4;
        let tmp2y = (y2 - y3) * 3.0 - y1 + y4;

        self.fx = x1;
        self.fy = y1;
        self.saved_fx = x1;
        self.saved_fy = y1;

        self.dfx = (x2 - x1) * pre1 + tmp1x * pre2 + tmp2x * subdivide_step3;
        self.dfy = (y2 - y1) * pre1 + tmp1y * pre2 + tmp2y * subdivide_step3;
        self.saved_dfx = self.dfx;
        self.saved_dfy = self.dfy;

        self.ddfx = tmp1x * pre4 + tmp2x * pre5;
        self.ddfy = tmp1y * pre4 + tmp2y * pre5;
        self.saved_ddfx = self.ddfx;
        self.saved_ddfy = self.ddfy;

        self.dddfx = tmp2x * pre5;
        self.dddfy = tmp2y * pre5;

        self.step = self.num_steps;
    }

    pub fn init(&mut self, cp: &Curve4Points) {
        self.init_with_coords(cp[0], cp[1], cp[2], cp[3], cp[4], cp[5], cp[6], cp[7]);
    }

    pub fn rewind(&mut self, _: u32) {
        if self.num_steps == 0 {
            self.step = -1;
            return;
        }
        self.step = self.num_steps;
        self.fx = self.saved_fx;
        self.fy = self.saved_fy;
        self.dfx = self.saved_dfx;
        self.dfy = self.saved_dfy;
        self.ddfx = self.saved_ddfx;
        self.ddfy = self.saved_ddfy;
    }

    pub fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.step < 0 {
            return PathCmd::Stop as u32;
        }
        if self.step == self.num_steps {
            *x = self.start_x;
            *y = self.start_y;
            self.step -= 1;
            return PathCmd::MoveTo as u32;
        }

        if self.step == 0 {
            *x = self.end_x;
            *y = self.end_y;
            self.step -= 1;
            return PathCmd::LineTo as u32;
        }

        self.fx += self.dfx;
        self.fy += self.dfy;
        self.dfx += self.ddfx;
        self.dfy += self.ddfy;
        self.ddfx += self.dddfx;
        self.ddfy += self.dddfy;

        *x = self.fx;
        *y = self.fy;
        self.step -= 1;
        PathCmd::LineTo as u32
    }

    pub fn reset(&mut self) {
        self.num_steps = 0;
        self.step = -1;
    }

    pub fn set_approximation_method(&mut self, _e: CurveApproximationMethod) {}
    pub fn approximation_method(&self) -> CurveApproximationMethod {
        CurveApproximationMethod::CurveInc
    }

    pub fn set_angle_tolerance(&mut self, _f: f64) {}
    pub fn angle_tolerance(&self) -> f64 {
        return 0.0;
    }

    pub fn set_cusp_limit(&mut self, _f: f64) {}
    pub fn cusp_limit() -> f64 {
        return 0.0;
    }
}

////
pub struct Curve4Div {
    approximation_scale: f64,
    distance_tolerance_square: f64,
    angle_tolerance: f64,
    cusp_limit: f64,
    count: usize,
    points: VecPodB<PointD>,
}

impl Curve4Div {
    pub fn new() -> Curve4Div {
        Curve4Div {
            approximation_scale: 1.0,
            distance_tolerance_square: 0.0,
            angle_tolerance: 0.0,
            cusp_limit: 0.0,
            count: 0,
            points: VecPodB::new(),
        }
    }

    pub fn new_with_points(
        x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
    ) -> Curve4Div {
        let mut curve = Curve4Div::new();
        curve.init(x1, y1, x2, y2, x3, y3, x4, y4);
        curve
    }

    pub fn new_with_curve4_points(cp: &Curve4Points) -> Curve4Div {
        let mut curve = Curve4Div::new();
        curve.init(cp[0], cp[1], cp[2], cp[3], cp[4], cp[5], cp[6], cp[7]);
        curve
    }

    pub fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64) {
        self.points.clear();
        self.distance_tolerance_square = 0.5 / self.approximation_scale;
        self.distance_tolerance_square *= self.distance_tolerance_square;
        self.bezier(x1, y1, x2, y2, x3, y3, x4, y4);
        self.count = 0;
    }

    pub fn bezier(
        &mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
    ) {
        self.points.push(PointD::new(x1, y1));
        self.recursive_bezier(x1, y1, x2, y2, x3, y3, x4, y4, 0);
        self.points.push(PointD::new(x4, y4));
    }

    pub fn reset(&mut self) {
        self.points.clear();
        self.count = 0;
    }

    pub fn init_with_curve4_points(&mut self, cp: &Curve4Points) {
        self.init(cp[0], cp[1], cp[2], cp[3], cp[4], cp[5], cp[6], cp[7]);
    }

    pub fn set_approximation_method(&mut self, _: CurveApproximationMethod) {}

    pub fn approximation_method(&self) -> CurveApproximationMethod {
        CurveApproximationMethod::CurveDiv
    }

    pub fn set_approximation_scale(&mut self, s: f64) {
        self.approximation_scale = s;
    }

    pub fn approximation_scale(&self) -> f64 {
        self.approximation_scale
    }

    pub fn set_angle_tolerance(&mut self, a: f64) {
        self.angle_tolerance = a;
    }

    pub fn angle_tolerance(&self) -> f64 {
        self.angle_tolerance
    }

    pub fn set_cusp_limit(&mut self, v: f64) {
        self.cusp_limit = if v == 0.0 { 0.0 } else { PI - v };
    }

    pub fn cusp_limit(&self) -> f64 {
        if self.cusp_limit == 0.0 {
            0.0
        } else {
            PI - self.cusp_limit
        }
    }

    pub fn rewind(&mut self, _: u32) {
        self.count = 0;
    }

    pub fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.count >= self.points.len() {
            return PathCmd::Stop as u32;
        }
        let p = self.points[self.count];
        *x = p.x;
        *y = p.y;
        self.count += 1;
        if self.count == 1 {
            PathCmd::MoveTo as u32
        } else {
            PathCmd::LineTo as u32
        }
    }

    fn recursive_bezier(
        &mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
        level: u32,
    ) {
        if level > CURVE_RECURSION_EPSILON {
            return;
        }

        // Calculate all the mid-points of the line segments
        //----------------------
        let x12 = (x1 + x2) / 2.0;
        let y12 = (y1 + y2) / 2.0;
        let x23 = (x2 + x3) / 2.0;
        let y23 = (y2 + y3) / 2.0;
        let x34 = (x3 + x4) / 2.0;
        let y34 = (y3 + y4) / 2.0;
        let x123 = (x12 + x23) / 2.0;
        let y123 = (y12 + y23) / 2.0;
        let x234 = (x23 + x34) / 2.0;
        let y234 = (y23 + y34) / 2.0;
        let x1234 = (x123 + x234) / 2.0;
        let y1234 = (y123 + y234) / 2.0;

        // Try to approximate the full cubic curve by a single straight line
        //------------------
        let dx = x4 - x1;
        let dy = y4 - y1;

        let mut d2 = ((x2 - x4) * dy - (y2 - y4) * dx).abs();
        let mut d3 = ((x3 - x4) * dy - (y3 - y4) * dx).abs();
        let (mut da1, mut da2, mut k);

        match (((d2 > CURVE_COLLINEARITY_EPSILON) as u32) << 1)
            + (d3 > CURVE_COLLINEARITY_EPSILON) as u32
        {
            0 => {
                // All collinear OR p1==p4
                //----------------------
                k = dx * dx + dy * dy;
                if k == 0.0 {
                    d2 = calc_sq_distance(x1, y1, x2, y2);
                    d3 = calc_sq_distance(x4, y4, x3, y3);
                } else {
                    k = 1.0 / k;
                    da1 = x2 - x1;
                    da2 = y2 - y1;
                    d2 = k * (da1 * dx + da2 * dy);
                    da1 = x3 - x1;
                    da2 = y3 - y1;
                    d3 = k * (da1 * dx + da2 * dy);
                    if d2 > 0.0 && d2 < 1.0 && d3 > 0.0 && d3 < 1.0 {
                        // Simple collinear case, 1---2---3---4
                        // We can leave just two endpoints
                        return;
                    }
                    if d2 <= 0.0 {
                        d2 = calc_sq_distance(x2, y2, x1, y1);
                    } else if d2 >= 1.0 {
                        d2 = calc_sq_distance(x2, y2, x4, y4);
                    } else {
                        d2 = calc_sq_distance(x2, y2, x1 + d2 * dx, y1 + d2 * dy);
                    }

                    if d3 <= 0.0 {
                        d3 = calc_sq_distance(x3, y3, x1, y1);
                    } else if d3 >= 1.0 {
                        d3 = calc_sq_distance(x3, y3, x4, y4);
                    } else {
                        d3 = calc_sq_distance(x3, y3, x1 + d3 * dx, y1 + d3 * dy);
                    }
                }
                if d2 > d3 {
                    if d2 < self.distance_tolerance_square {
                        self.points.push(PointD::new(x2, y2));
                        return;
                    }
                } else {
                    if d3 < self.distance_tolerance_square {
                        self.points.push(PointD::new(x3, y3));
                        return;
                    }
                }
            }

            1 => {
                // p1,p2,p4 are collinear, p3 is significant
                //----------------------
                if d3 * d3 <= self.distance_tolerance_square * (dx * dx + dy * dy) {
                    if self.angle_tolerance < CURVE_ANGLE_TOLERANCE_EPSILON {
                        self.points.push(PointD::new(x23, y23));
                        return;
                    }

                    // Angle Condition
                    //----------------------
                    da1 = (y4 - y3).abs() - (x4 - x3).abs();
                    if da1 >= PI {
                        da1 = 2.0 * PI - da1;
                    }

                    if da1 < self.angle_tolerance {
                        self.points.push(PointD::new(x2, y2));
                        self.points.push(PointD::new(x3, y3));
                        return;
                    }

                    if self.cusp_limit != 0.0 {
                        if da1 > self.cusp_limit {
                            self.points.push(PointD::new(x3, y3));
                            return;
                        }
                    }
                }
            }

            2 => {
                // p1,p3,p4 are collinear, p2 is significant
                //----------------------
                if d2 * d2 <= self.distance_tolerance_square * (dx * dx + dy * dy) {
                    if self.angle_tolerance < CURVE_ANGLE_TOLERANCE_EPSILON {
                        self.points.push(PointD::new(x23, y23));
                        return;
                    }

                    // Angle Condition
                    //----------------------
                    da1 = (y3 - y2).abs() - (x3 - x2).abs();
                    if da1 >= PI {
                        da1 = 2.0 * PI - da1;
                    }

                    if da1 < self.angle_tolerance {
                        self.points.push(PointD::new(x2, y2));
                        self.points.push(PointD::new(x3, y3));
                        return;
                    }

                    if self.cusp_limit != 0.0 {
                        if da1 > self.cusp_limit {
                            self.points.push(PointD::new(x2, y2));
                            return;
                        }
                    }
                }
            }

            3 =>
            // Regular case
            //-----------------
            {
                if (d2 + d3) * (d2 + d3) <= self.distance_tolerance_square * (dx * dx + dy * dy) {
                    // If the curvature doesn't exceed the distance_tolerance value
                    // we tend to finish subdivisions.
                    //----------------------
                    if self.angle_tolerance < CURVE_ANGLE_TOLERANCE_EPSILON {
                        self.points.push(PointD::new(x23, y23));
                        return;
                    }

                    // Angle & Cusp Condition
                    //----------------------
                    k = (y3 - y2).atan2(x3 - x2);
                    da1 = (k - (y2 - y1).atan2(x2 - x1)).abs();
                    da2 = ((y4 - y3).atan2(x4 - x3) - k).abs();
                    if da1 >= PI {
                        da1 = 2.0 * PI - da1;
                    }
                    if da2 >= PI {
                        da2 = 2.0 * PI - da2;
                    }

                    if da1 + da2 < self.angle_tolerance {
                        // Finally we can stop the recursion
                        //----------------------
                        self.points.push(PointD::new(x23, y23));
                        return;
                    }

                    if self.cusp_limit != 0.0 {
                        if da1 > self.cusp_limit {
                            self.points.push(PointD::new(x2, y2));
                            return;
                        }

                        if da2 > self.cusp_limit {
                            self.points.push(PointD::new(x3, y3));
                            return;
                        }
                    }
                }
            }
            _ => {}
        }

        // Continue subdivision
        //----------------------
        self.recursive_bezier(x1, y1, x12, y12, x123, y123, x1234, y1234, level + 1);
        self.recursive_bezier(x1234, y1234, x234, y234, x34, y34, x4, y4, level + 1);
    }
}

/////////
pub struct Curve3 {
    curve_inc: Curve3Inc,
    curve_div: Curve3Div,
    approximation_method: CurveApproximationMethod,
}

impl Curve3 {
    pub fn new_with_points(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> Self {
        let mut curve = Curve3::new();
        curve.init(x1, y1, x2, y2, x3, y3);
        curve
    }
}

impl CurveType3 for Curve3 {
    fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        if self.approximation_method == CurveApproximationMethod::CurveInc {
            self.curve_inc.init(x1, y1, x2, y2, x3, y3);
        } else {
            self.curve_div.init(x1, y1, x2, y2, x3, y3);
        }
    }
}

impl CurveBase for Curve3 {
    fn new() -> Self {
        Curve3 {
            curve_inc: Curve3Inc::new(),
            curve_div: Curve3Div::new(),
            approximation_method: CurveApproximationMethod::CurveDiv,
        }
    }
    fn reset(&mut self) {
        self.curve_inc.reset();
        self.curve_div.reset();
    }

    fn approximation_method(&self) -> CurveApproximationMethod {
        self.approximation_method
    }

    fn set_approximation_method(&mut self, v: CurveApproximationMethod) {
        self.approximation_method = v;
    }

    fn approximation_scale(&self) -> f64 {
        self.curve_inc.approximation_scale()
    }

    fn set_approximation_scale(&mut self, s: f64) {
        self.curve_inc.set_approximation_scale(s);
        self.curve_div.set_approximation_scale(s);
    }

    fn angle_tolerance(&self) -> f64 {
        self.curve_div.angle_tolerance()
    }

    fn set_angle_tolerance(&mut self, v: f64) {
        self.curve_div.set_angle_tolerance(v);
    }

    fn cusp_limit(&self) -> f64 {
        self.curve_div.cusp_limit()
    }

    fn set_cusp_limit(&mut self, v: f64) {
        self.curve_div.set_cusp_limit(v);
    }
}

impl VertexSource for Curve3 {
    fn rewind(&mut self, path_id: u32) {
        if self.approximation_method == CurveApproximationMethod::CurveInc {
            self.curve_inc.rewind(path_id);
        } else {
            self.curve_div.rewind(path_id);
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.approximation_method == CurveApproximationMethod::CurveInc {
            self.curve_inc.vertex(x, y)
        } else {
            self.curve_div.vertex(x, y)
        }
    }
}

pub struct Curve4 {
    curve_inc: Curve4Inc,
    curve_div: Curve4Div,
    approximation_method: CurveApproximationMethod,
}

impl Curve4 {
    pub fn new_with_points(
        x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
    ) -> Self {
        let mut curve = Curve4::new();
        curve.init(x1, y1, x2, y2, x3, y3, x4, y4);
        curve
    }

    pub fn new_with_curve4_points(cp: &Curve4Points) -> Self {
        let mut curve = Curve4::new();
        curve.init_with_curve4_points(cp);
        curve
    }

    pub fn init_with_curve4_points(&mut self, cp: &Curve4Points) {
        self.init(cp[0], cp[1], cp[2], cp[3], cp[4], cp[5], cp[6], cp[7]);
    }
}

impl CurveType4 for Curve4 {
    fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64) {
        if self.approximation_method == CurveApproximationMethod::CurveInc {
            self.curve_inc
                .init_with_coords(x1, y1, x2, y2, x3, y3, x4, y4);
        } else {
            self.curve_div.init(x1, y1, x2, y2, x3, y3, x4, y4);
        }
    }
}

impl CurveBase for Curve4 {
    fn new() -> Self {
        Curve4 {
            curve_inc: Curve4Inc::new(),
            curve_div: Curve4Div::new(),
            approximation_method: CurveApproximationMethod::CurveDiv,
        }
    }

    fn reset(&mut self) {
        self.curve_inc.reset();
        self.curve_div.reset();
    }

    fn approximation_method(&self) -> CurveApproximationMethod {
        self.approximation_method
    }

    fn set_approximation_method(&mut self, v: CurveApproximationMethod) {
        self.approximation_method = v;
    }

    fn approximation_scale(&self) -> f64 {
        self.curve_inc.approximation_scale()
    }

    fn set_approximation_scale(&mut self, s: f64) {
        self.curve_inc.set_approximation_scale(s);
        self.curve_div.set_approximation_scale(s);
    }

    fn angle_tolerance(&self) -> f64 {
        self.curve_div.angle_tolerance()
    }

    fn set_angle_tolerance(&mut self, v: f64) {
        self.curve_div.set_angle_tolerance(v);
    }

    fn cusp_limit(&self) -> f64 {
        self.curve_div.cusp_limit()
    }

    fn set_cusp_limit(&mut self, v: f64) {
        self.curve_div.set_cusp_limit(v);
    }
}

impl VertexSource for Curve4 {
    fn rewind(&mut self, path_id: u32) {
        if self.approximation_method == CurveApproximationMethod::CurveInc {
            self.curve_inc.rewind(path_id);
        } else {
            self.curve_div.rewind(path_id);
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.approximation_method == CurveApproximationMethod::CurveInc {
            self.curve_inc.vertex(x, y)
        } else {
            self.curve_div.vertex(x, y)
        }
    }
}

//-------------------------------------------------------catrom_to_bezier
pub fn catrom_to_bezier(
    x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
) -> Curve4Points {
    // Trans. matrix Catmull-Rom to Bezier
    //
    //  0       1       0       0
    //  -1/6    1       1/6     0
    //  0       1/6     1       -1/6
    //  0       0       1       0
    //
    Curve4Points::new(
        x2,
        y2,
        (-x1 + 6. * x2 + x3) / 6.,
        (-y1 + 6. * y2 + y3) / 6.,
        (x2 + 6. * x3 - x4) / 6.,
        (y2 + 6. * y3 - y4) / 6.,
        x3,
        y3,
    )
}

pub fn catrom_to_bezier_cp(cp: &Curve4Points) -> Curve4Points {
    catrom_to_bezier(cp[0], cp[1], cp[2], cp[3], cp[4], cp[5], cp[6], cp[7])
}

//-----------------------------------------------------ubspline_to_bezier
pub fn ubspline_to_bezier(
    x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
) -> Curve4Points {
    // Trans. matrix Uniform BSpline to Bezier
    //
    //  1/6     4/6     1/6     0
    //  0       4/6     2/6     0
    //  0       2/6     4/6     0
    //  0       1/6     4/6     1/6
    //
    Curve4Points::new(
        (x1 + 4. * x2 + x3) / 6.,
        (y1 + 4. * y2 + y3) / 6.,
        (4. * x2 + 2. * x3) / 6.,
        (4. * y2 + 2. * y3) / 6.,
        (2. * x2 + 4. * x3) / 6.,
        (2. * y2 + 4. * y3) / 6.,
        (x2 + 4. * x3 + x4) / 6.,
        (y2 + 4. * y3 + y4) / 6.,
    )
}

pub fn ubspline_to_bezier_cp(cp: &Curve4Points) -> Curve4Points {
    ubspline_to_bezier(cp[0], cp[1], cp[2], cp[3], cp[4], cp[5], cp[6], cp[7])
}

//------------------------------------------------------hermite_to_bezier
pub fn hermite_to_bezier(
    x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
) -> Curve4Points {
    // Trans. matrix Hermite to Bezier
    //
    //  1       0       0       0
    //  1       0       1/3     0
    //  0       1       0       -1/3
    //  0       1       0       0
    //
    Curve4Points::new(
        x1,
        y1,
        (3. * x1 + x3) / 3.,
        (3. * y1 + y3) / 3.,
        (3. * x2 - x4) / 3.,
        (3. * y2 - y4) / 3.,
        x2,
        y2,
    )
}

pub fn hermite_to_bezier_cp(cp: &Curve4Points) -> Curve4Points {
    hermite_to_bezier(cp[0], cp[1], cp[2], cp[3], cp[4], cp[5], cp[6], cp[7])
}
