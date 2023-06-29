use crate::basics::*;
use crate::clip_liang_barsky::*;
use crate::dda_line::Dda2LineIp;
use crate::ellipse_bresenham::EllipseBresenhamIp;
use crate::line_aa_basics::{
    self, fix_degenerate_bisectrix_end, fix_degenerate_bisectrix_start, line_dbl_hr, line_mr,
    LineCoordSat, LineMRSubpixel, LineParameters, LineSubpixel,
};
use crate::math::calc_distance;
use crate::{Color, Coord, GammaFn, Renderer, RendererOutline};

#[repr(i32)]
pub enum SubPixel {
    Shift = LineSubpixel::Shift as i32,
    Scale = 1 << Self::Shift as i32,
    Mask = Self::Scale as i32 - 1,
}

#[repr(i32)]
pub enum AaScale {
    Shift = 8,
    Scale = 1 << Self::Shift as i32,
    Mask = Self::Scale as i32 - 1,
}

pub trait DistanceInterpolator {
    fn inc_x(&mut self) {}
    fn dec_x(&mut self) {}
    fn inc_x_dy(&mut self, _dy: i32) {}
    fn dec_x_dy(&mut self, _dy: i32) {}
    fn inc_y(&mut self) {}
    fn dec_y(&mut self) {}
    fn inc_y_dx(&mut self, _dx: i32) {}
    fn dec_y_dx(&mut self, _dx: i32) {}
    fn dist_start(&self) -> i32 {
        0
    }
    fn dist_end(&self) -> i32 {
        0
    }
    fn dist(&self) -> i32 {
        0
    }
    fn dist1(&self) -> i32 {
        0
    }
    fn dist2(&self) -> i32 {
        0
    }
    fn dx(&self) -> i32 {
        0
    }
    fn dy(&self) -> i32 {
        0
    }
    fn dx_start(&self) -> i32 {
        0
    }
    fn dy_start(&self) -> i32 {
        0
    }
    fn dx_end(&self) -> i32 {
        0
    }
    fn dy_end(&self) -> i32 {
        0
    }
}
//===================================================DistanceIp0
struct DistanceIp0 {
    _dx: i32,
    dy: i32,
    dist: i32,
}

impl DistanceIp0 {
    //---------------------------------------------------------------------
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32, x: i32, y: i32) -> DistanceIp0 {
        let dx = line_mr(x2) - line_mr(x1);
        let dy = line_mr(y2) - line_mr(y1);
        DistanceIp0 {
            _dx: dx << LineMRSubpixel::Shift as i32,
            dy: dy << LineMRSubpixel::Shift as i32,
            dist: (line_mr(x + LineSubpixel::Scale as i32 / 2) - line_mr(x2)) * dy
                - (line_mr(y + LineSubpixel::Scale as i32 / 2) - line_mr(y2)) * dx,
        }
    }
}
impl DistanceInterpolator for DistanceIp0 {
    //---------------------------------------------------------------------
    fn inc_x(&mut self) {
        self.dist += self.dy;
    }
    fn dist(&self) -> i32 {
        self.dist
    }
}

//==================================================DistanceIp00
struct DistanceIp00 {
    _dx1: i32,
    dy1: i32,
    _dx2: i32,
    dy2: i32,
    dist1: i32,
    dist2: i32,
}

impl DistanceIp00 {
    //---------------------------------------------------------------------
    pub fn new(
        xc: i32, yc: i32, x1: i32, y1: i32, x2: i32, y2: i32, x: i32, y: i32,
    ) -> DistanceIp00 {
        let dx1 = line_mr(x1) - line_mr(xc);
        let dy1 = line_mr(y1) - line_mr(yc);
        let dx2 = line_mr(x2) - line_mr(xc);
        let dy2 = line_mr(y2) - line_mr(yc);
        DistanceIp00 {
            _dx1: dx1 << LineMRSubpixel::Shift as i32,
            dy1: dy1 << LineMRSubpixel::Shift as i32,
            _dx2: dx2 << LineMRSubpixel::Shift as i32,
            dy2: dy2 << LineMRSubpixel::Shift as i32,
            dist1: (line_mr(x + LineSubpixel::Scale as i32 / 2) - line_mr(x1)) * dy1
                - (line_mr(y + LineSubpixel::Scale as i32 / 2) - line_mr(y1)) * dx1,
            dist2: (line_mr(x + LineSubpixel::Scale as i32 / 2) - line_mr(x2)) * dy2
                - (line_mr(y + LineSubpixel::Scale as i32 / 2) - line_mr(y2)) * dx2,
        }
    }
}
impl DistanceInterpolator for DistanceIp00 {
    //---------------------------------------------------------------------
    fn inc_x(&mut self) {
        self.dist1 += self.dy1;
        self.dist2 += self.dy2;
    }
    fn dist1(&self) -> i32 {
        self.dist1
    }
    fn dist2(&self) -> i32 {
        self.dist2
    }
}

//===================================================DistanceIp1
pub struct DistanceIp1 {
    dx: i32,
    dy: i32,
    dist: i32,
}

impl DistanceIp1 {
    //---------------------------------------------------------------------
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32, x: i32, y: i32) -> DistanceIp1 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dist = iround(
            (x + LineSubpixel::Scale as i32 / 2 - x2) as f64 * dy as f64
                - (y + LineSubpixel::Scale as i32 / 2 - y2) as f64 * dx as f64,
        );
        DistanceIp1 {
            dx: dx << LineSubpixel::Shift as i32,
            dy: dy << LineSubpixel::Shift as i32,
            dist: dist,
        }
    }
}
impl DistanceInterpolator for DistanceIp1 {
    //---------------------------------------------------------------------
    fn inc_x(&mut self) {
        self.dist += self.dy;
    }

    fn dec_x(&mut self) {
        self.dist -= self.dy;
    }

    fn inc_y(&mut self) {
        self.dist -= self.dx;
    }

    fn dec_y(&mut self) {
        self.dist += self.dx;
    }

    //---------------------------------------------------------------------
    fn inc_x_dy(&mut self, dy: i32) {
        self.dist += self.dy;
        if dy > 0 {
            self.dist -= self.dx;
        }
        if dy < 0 {
            self.dist += self.dx;
        }
    }

    //---------------------------------------------------------------------
    fn dec_x_dy(&mut self, dy: i32) {
        self.dist -= self.dy;
        if dy > 0 {
            self.dist -= self.dx;
        }
        if dy < 0 {
            self.dist += self.dx;
        }
    }

    //---------------------------------------------------------------------
    fn inc_y_dx(&mut self, dx: i32) {
        self.dist -= self.dx;
        if dx > 0 {
            self.dist += self.dy;
        }
        if dx < 0 {
            self.dist -= self.dy;
        }
    }

    fn dec_y_dx(&mut self, dx: i32) {
        self.dist += self.dx;
        if dx > 0 {
            self.dist += self.dy;
        }
        if dx < 0 {
            self.dist -= self.dy;
        }
    }

    //---------------------------------------------------------------------
    fn dist(&self) -> i32 {
        self.dist
    }

    fn dx(&self) -> i32 {
        self.dx
    }

    fn dy(&self) -> i32 {
        self.dy
    }
}

pub struct DistanceIp2 {
    dx: i32,
    dy: i32,
    dx_start: i32,
    dy_start: i32,
    dist: i32,
    dist_start: i32,
}

impl DistanceIp2 {
    pub fn new(
        x1: i32, y1: i32, x2: i32, y2: i32, sx: i32, sy: i32, x: i32, y: i32,
    ) -> DistanceIp2 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dx_start = line_mr(sx) - line_mr(x1);
        let dy_start = line_mr(sy) - line_mr(y1);
        let dist = iround(
            (x + LineSubpixel::Scale as i32 / 2 - x2) as f64 * dy as f64
                - (y + LineSubpixel::Scale as i32 / 2 - y2) as f64 * dx as f64,
        );
        let dist_start = (line_mr(x + LineSubpixel::Scale as i32 / 2) - line_mr(sx)) * dy_start
            - (line_mr(y + LineSubpixel::Scale as i32 / 2) - line_mr(sy)) * dx_start;
        DistanceIp2 {
            dx: dx << LineSubpixel::Shift as i32,
            dy: dy << LineSubpixel::Shift as i32,
            dx_start: dx_start << LineMRSubpixel::Shift as i32,
            dy_start: dy_start << LineMRSubpixel::Shift as i32,
            dist,
            dist_start,
        }
    }

    pub fn new_end(
        x1: i32, y1: i32, x2: i32, y2: i32, ex: i32, ey: i32, x: i32, y: i32,
    ) -> DistanceIp2 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dx_start = line_mr(ex) - line_mr(x2);
        let dy_start = line_mr(ey) - line_mr(y2);
        let dist = iround(
            (x + LineSubpixel::Scale as i32 / 2 - x2) as f64 * dy as f64
                - (y + LineSubpixel::Scale as i32 / 2 - y2) as f64 * dx as f64,
        );
        let dist_start = (line_mr(x + LineSubpixel::Scale as i32 / 2) - line_mr(ex)) * dy_start
            - (line_mr(y + LineSubpixel::Scale as i32 / 2) - line_mr(ey)) * dx_start;
        DistanceIp2 {
            dx: dx << LineSubpixel::Shift as i32,
            dy: dy << LineSubpixel::Shift as i32,
            dx_start: dx_start << LineMRSubpixel::Shift as i32,
            dy_start: dy_start << LineMRSubpixel::Shift as i32,
            dist,
            dist_start,
        }
    }
}

impl DistanceInterpolator for DistanceIp2 {
    fn inc_x(&mut self) {
        self.dist += self.dy;
        self.dist_start += self.dy_start;
    }

    fn dec_x(&mut self) {
        self.dist -= self.dy;
        self.dist_start -= self.dy_start;
    }

    fn inc_y(&mut self) {
        self.dist -= self.dx;
        self.dist_start -= self.dx_start;
    }

    fn dec_y(&mut self) {
        self.dist += self.dx;
        self.dist_start += self.dx_start;
    }

    fn inc_x_dy(&mut self, dy: i32) {
        self.dist += self.dy;
        self.dist_start += self.dy_start;
        if dy > 0 {
            self.dist -= self.dx;
            self.dist_start -= self.dx_start;
        }
        if dy < 0 {
            self.dist += self.dx;
            self.dist_start += self.dx_start;
        }
    }

    fn dec_x_dy(&mut self, dy: i32) {
        self.dist -= self.dy;
        self.dist_start -= self.dy_start;
        if dy > 0 {
            self.dist -= self.dx;
            self.dist_start -= self.dx_start;
        }
        if dy < 0 {
            self.dist += self.dx;
            self.dist_start += self.dx_start;
        }
    }

    fn inc_y_dx(&mut self, dx: i32) {
        self.dist -= self.dx;
        self.dist_start -= self.dx_start;
        if dx > 0 {
            self.dist += self.dy;
            self.dist_start += self.dy_start;
        }
        if dx < 0 {
            self.dist -= self.dy;
            self.dist_start -= self.dy_start;
        }
    }

    fn dec_y_dx(&mut self, dx: i32) {
        self.dist += self.dx;
        self.dist_start += self.dx_start;
        if dx > 0 {
            self.dist += self.dy;
            self.dist_start += self.dy_start;
        }
        if dx < 0 {
            self.dist -= self.dy;
            self.dist_start -= self.dy_start;
        }
    }

    fn dist(&self) -> i32 {
        self.dist
    }

    fn dist_start(&self) -> i32 {
        self.dist_start
    }

    fn dist_end(&self) -> i32 {
        self.dist_start
    }

    fn dx(&self) -> i32 {
        self.dx
    }

    fn dy(&self) -> i32 {
        self.dy
    }

    fn dx_start(&self) -> i32 {
        self.dx_start
    }

    fn dy_start(&self) -> i32 {
        self.dy_start
    }

    fn dx_end(&self) -> i32 {
        self.dx_start
    }

    fn dy_end(&self) -> i32 {
        self.dy_start
    }
}

pub struct DistanceIp3 {
    dx: i32,
    dy: i32,
    dx_start: i32,
    dy_start: i32,
    dx_end: i32,
    dy_end: i32,
    dist: i32,
    dist_start: i32,
    dist_end: i32,
}

impl DistanceIp3 {
    pub fn new(
        x1: i32, y1: i32, x2: i32, y2: i32, sx: i32, sy: i32, ex: i32, ey: i32, x: i32, y: i32,
    ) -> DistanceIp3 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dx_start = line_mr(sx) - line_mr(x1);
        let dy_start = line_mr(sy) - line_mr(y1);
        let dx_end = line_mr(ex) - line_mr(x2);
        let dy_end = line_mr(ey) - line_mr(y2);
        let dist = iround(
            (x + LineSubpixel::Scale as i32 / 2 - x2) as f64 * dy as f64
                - (y + LineSubpixel::Scale as i32 / 2 - y2) as f64 * dx as f64,
        );
        let dist_start = (line_mr(x + LineSubpixel::Scale as i32 / 2) - line_mr(sx)) * dy_start
            - (line_mr(y + LineSubpixel::Scale as i32 / 2) - line_mr(sy)) * dx_start;
        let dist_end = (line_mr(x + LineSubpixel::Scale as i32 / 2) - line_mr(ex)) * dy_end
            - (line_mr(y + LineSubpixel::Scale as i32 / 2) - line_mr(ey)) * dx_end;
        DistanceIp3 {
            dx: dx << LineSubpixel::Shift as i32,
            dy: dy << LineSubpixel::Shift as i32,
            dx_start: dx_start << LineMRSubpixel::Shift as i32,
            dy_start: dy_start << LineMRSubpixel::Shift as i32,
            dx_end: dx_end << LineMRSubpixel::Shift as i32,
            dy_end: dy_end << LineMRSubpixel::Shift as i32,
            dist: dist,
            dist_start: dist_start,
            dist_end: dist_end,
        }
    }
}

impl DistanceInterpolator for DistanceIp3 {
    fn inc_x(&mut self) {
        self.dist += self.dy;
        self.dist_start += self.dy_start;
        self.dist_end += self.dy_end;
    }

    fn dec_x(&mut self) {
        self.dist -= self.dy;
        self.dist_start -= self.dy_start;
        self.dist_end -= self.dy_end;
    }

    fn inc_y(&mut self) {
        self.dist -= self.dx;
        self.dist_start -= self.dx_start;
        self.dist_end -= self.dx_end;
    }

    fn dec_y(&mut self) {
        self.dist += self.dx;
        self.dist_start += self.dx_start;
        self.dist_end += self.dx_end;
    }

    fn inc_x_dy(&mut self, dy: i32) {
        self.dist += self.dy;
        self.dist_start += self.dy_start;
        self.dist_end += self.dy_end;
        if dy > 0 {
            self.dist -= self.dx;
            self.dist_start -= self.dx_start;
            self.dist_end -= self.dx_end;
        }
        if dy < 0 {
            self.dist += self.dx;
            self.dist_start += self.dx_start;
            self.dist_end += self.dx_end;
        }
    }

    fn dec_x_dy(&mut self, dy: i32) {
        self.dist -= self.dy;
        self.dist_start -= self.dy_start;
        self.dist_end -= self.dy_end;
        if dy > 0 {
            self.dist -= self.dx;
            self.dist_start -= self.dx_start;
            self.dist_end -= self.dx_end;
        }
        if dy < 0 {
            self.dist += self.dx;
            self.dist_start += self.dx_start;
            self.dist_end += self.dx_end;
        }
    }

    fn inc_y_dx(&mut self, dx: i32) {
        self.dist -= self.dx;
        self.dist_start -= self.dx_start;
        self.dist_end -= self.dx_end;
        if dx > 0 {
            self.dist += self.dy;
            self.dist_start += self.dy_start;
            self.dist_end += self.dy_end;
        }
        if dx < 0 {
            self.dist -= self.dy;
            self.dist_start -= self.dy_start;
            self.dist_end -= self.dy_end;
        }
    }

    fn dec_y_dx(&mut self, dx: i32) {
        self.dist += self.dx;
        self.dist_start += self.dx_start;
        self.dist_end += self.dx_end;
        if dx > 0 {
            self.dist += self.dy;
            self.dist_start += self.dy_start;
            self.dist_end += self.dy_end;
        }
        if dx < 0 {
            self.dist -= self.dy;
            self.dist_start -= self.dy_start;
            self.dist_end -= self.dy_end;
        }
    }

    fn dist(&self) -> i32 {
        self.dist
    }

    fn dist_start(&self) -> i32 {
        self.dist_start
    }

    fn dist_end(&self) -> i32 {
        self.dist_end
    }

    fn dx(&self) -> i32 {
        self.dx
    }

    fn dy(&self) -> i32 {
        self.dy
    }

    fn dx_start(&self) -> i32 {
        self.dx_start
    }

    fn dy_start(&self) -> i32 {
        self.dy_start
    }

    fn dx_end(&self) -> i32 {
        self.dx_end
    }

    fn dy_end(&self) -> i32 {
        self.dy_end
    }
}

const MAX_HALF_WIDTH: usize = 64;
//================================================LineIpAaBase
pub struct LineIpAaBase<'a, Ren: RendererOutline> {
    lp: &'a LineParameters,
    li: Dda2LineIp,
    ren: &'a mut Ren,
    len: i32,
    x: i32,
    y: i32,
    old_x: i32,
    old_y: i32,
    count: i32,
    width: i32,
    max_extent: i32,
    step: i32,
    dist: [i32; MAX_HALF_WIDTH + 1],
    covers: [CoverType; MAX_HALF_WIDTH * 2 + 4],
}

impl<'a, Ren: RendererOutline> LineIpAaBase<'a, Ren> {
    pub const MAX_HALF_WIDTH: usize = 64;
    pub fn new(ren: &'a mut Ren, lp: &'a LineParameters) -> Self {
        let li = if lp.vertical {
            Dda2LineIp::new_bwd_y(line_dbl_hr(lp.x2 - lp.x1), (lp.y2 - lp.y1).abs())
        } else {
            Dda2LineIp::new_bwd_y(line_dbl_hr(lp.y2 - lp.y1), (lp.x2 - lp.x1).abs() + 1)
        };

        let mut i = 0;
        let wid = ren.subpixel_width();
        let stop = wid + LineSubpixel::Scale as i32 * 2;
        let mut dist = [0; MAX_HALF_WIDTH + 1];
        let mut li0 = Dda2LineIp::new_fwd(  //XXXX
            0,
            if lp.vertical {
                lp.dy << LineSubpixel::Shift as i32
            } else {
                lp.dx << LineSubpixel::Shift as i32
            },
            lp.len,
        );
        while i < MAX_HALF_WIDTH {
            dist[i] = li0.y();
            if dist[i] >= stop {
                break;
            }
            li0.inc();
            i += 1;
        }

        dist[i] = 0x7FFF0000;

        LineIpAaBase {
            lp: lp,
            li: li,
            ren: ren,
            len: if lp.vertical == (lp.inc > 0) {
                -lp.len
            } else {
                lp.len
            },
            x: lp.x1 >> LineSubpixel::Shift as i32,
            y: lp.y1 >> LineSubpixel::Shift as i32,
            old_x: lp.x1 >> LineSubpixel::Shift as i32,
            old_y: lp.y1 >> LineSubpixel::Shift as i32,
            count: if lp.vertical {
                ((lp.y2 >> LineSubpixel::Shift as i32) - (lp.y1 >> LineSubpixel::Shift as i32))
                    .abs()
            } else {
                ((lp.x2 >> LineSubpixel::Shift as i32) - (lp.x1 >> LineSubpixel::Shift as i32))
                    .abs()
            },
            width: wid,
            max_extent: (wid + LineSubpixel::Mask as i32) >> LineSubpixel::Shift as i32,
            step: 0,
            dist: dist,
            covers: [0; MAX_HALF_WIDTH * 2 + 4],
        }
    }

    pub fn vertical(&self) -> bool {
        self.lp.vertical
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn count(&self) -> i32 {
        self.count
    }

    pub fn step_hor_base<DI: DistanceInterpolator>(&mut self, di: &mut DI) -> i32 {
        self.li.inc();
        self.x += self.lp.inc;
        self.y = (self.lp.y1 + self.li.y()) >> LineSubpixel::Shift as i32;

        if self.lp.inc > 0 {
            di.inc_x_dy(self.y - self.old_y);
        } else {
            di.dec_x_dy(self.y - self.old_y);
        }

        self.old_y = self.y;

        di.dist() / self.len
    }

    pub fn step_ver_base<DI: DistanceInterpolator>(&mut self, di: &mut DI) -> i32 {
        self.li.inc();
        self.y += self.lp.inc;
        self.x = (self.lp.x1 + self.li.y()) >> LineSubpixel::Shift as i32;

        if self.lp.inc > 0 {
            di.inc_y_dx(self.x - self.old_x);
        } else {
            di.dec_y_dx(self.x - self.old_x);
        }

        self.old_x = self.x;

        di.dist() / self.len
    }
}

//====================================================LineIpAa0
pub struct LineIpAa0<'a, Ren: RendererOutline> {
    base: LineIpAaBase<'a, Ren>,
    di: DistanceIp1,
}

impl<'a, Ren: RendererOutline> LineIpAa0<'a, Ren> {
    pub fn new(ren: &'a mut Ren, lp: &'a LineParameters) -> Self {
        let mut ret = Self {
            base: LineIpAaBase::new(ren, lp),
            di: DistanceIp1::new(
                lp.x1,
                lp.y1,
                lp.x2,
                lp.y2,
                lp.x1 & !(LineSubpixel::Mask as i32),
                lp.y1 & !(LineSubpixel::Mask as i32),
            ),
        };
        ret.base.li.adjust_forward();
        ret
    }

    //---------------------------------------------------------------------
    pub fn step_hor(&mut self) -> bool {
        let mut dist;
        let mut dy;
        let s1 = self.base.step_hor_base(&mut self.di);
        let mut p0 = LineIpAaBase::<Ren>::MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        self.base.covers[p1] = self.base.ren.cover(s1) as u8;
        p1 += 1;

        dy = 1;
        loop {
            dist = self.base.dist[dy] - s1;
            if dist > self.base.width() {
                break;
            }
            self.base.covers[p1] = self.base.ren.cover(dist) as u8;
            p1 += 1;
            dy += 1;
        }

        dy = 1;
        loop {
            dist = self.base.dist[dy] + s1;
            if dist > self.base.width {
                break;
            }
            p0 -= 1;
            self.base.covers[p0] = self.base.ren.cover(dist) as u8;
            dy += 1;
        }

        self.base.ren.blend_solid_vspan(
            self.base.x,
            self.base.y - dy as i32 + 1,
            (p1 - p0) as i32,
            &self.base.covers[p0..],
        );
        self.base.step += 1;
        self.base.step < self.base.count
    }

    //---------------------------------------------------------------------
    pub fn step_ver(&mut self) -> bool {
        let mut dist;
        let mut dx;
        let s1 = self.base.step_ver_base(&mut self.di);
        let mut p0 = LineIpAaBase::<Ren>::MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        self.base.covers[p1] = self.base.ren.cover(s1) as u8;
        p1 += 1;

        dx = 1;
        loop {
            dist = self.base.dist[dx] - s1;
            if dist > self.base.width {
                break;
            }
            self.base.covers[p1] = self.base.ren.cover(dist) as u8;
            p1 += 1;
            dx += 1;
        }

        dx = 1;
        loop {
            dist = self.base.dist[dx] + s1;
            if dist > self.base.width {
                break;
            }
            p0 -= 1;
            self.base.covers[p0] = self.base.ren.cover(dist) as u8;
            dx += 1;
        }

        self.base.ren.blend_solid_hspan(
            self.base.x - dx as i32 + 1,
            self.base.y,
            (p1 - p0) as i32,
            &self.base.covers[p0..],
        );
        self.base.step += 1;
        self.base.step < self.base.count
    }
}

//====================================================LineIpAa1
pub struct LineIpAa1<'a, Ren: RendererOutline> {
    base: LineIpAaBase<'a, Ren>,
    di: DistanceIp2,
}

impl<'a, Ren: RendererOutline> LineIpAa1<'a, Ren> {
    pub fn new(ren: &'a mut Ren, lp: &'a LineParameters, sx: i32, sy: i32) -> Self {
        let mut li = Self {
            base: LineIpAaBase::new(ren, lp),
            di: DistanceIp2::new(
                lp.x1,
                lp.y1,
                lp.x2,
                lp.y2,
                sx,
                sy,
                lp.x1 & !(LineSubpixel::Mask as i32),
                lp.y1 & !(LineSubpixel::Mask as i32),
            ),
        };

        let mut dist1_start;
        let mut dist2_start;

        let mut npix = 1;

        if lp.vertical {
            loop {
                li.base.li.dec();
                li.base.y -= lp.inc;
                li.base.x = (li.base.lp.x1 + li.base.li.y()) >> LineSubpixel::Shift as i32;

                if lp.inc > 0 {
                    li.di.dec_y_dx(li.base.x - li.base.old_x);
                } else {
                    li.di.inc_y_dx(li.base.x - li.base.old_x);
                }

                li.base.old_x = li.base.x;

                dist1_start = li.di.dist_start();
                dist2_start = li.di.dist_start();

                let mut dx = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start += li.di.dy_start();
                    dist2_start -= li.di.dy_start();
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dx += 1;
                    if li.base.dist[dx] <= li.base.width {
                        break;
                    }
                }
                li.base.step -= 1;
                if npix == 0 {
                    break;
                }
                npix = 0;
            }
        } else {
            loop {
                li.base.li.dec();
                li.base.x -= lp.inc;
                li.base.y = (li.base.lp.y1 + li.base.li.y()) >> LineSubpixel::Shift as i32;

                if lp.inc > 0 {
                    li.di.dec_x_dy(li.base.y - li.base.old_y);
                } else {
                    li.di.inc_x_dy(li.base.y - li.base.old_y);
                }

                li.base.old_y = li.base.y;

                dist1_start = li.di.dist_start();
                dist2_start = li.di.dist_start();

                let mut dy = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start -= li.di.dx_start();
                    dist2_start += li.di.dx_start();
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dy += 1;
                    if li.base.dist[dy] <= li.base.width {
                        break;
                    }
                }
                li.base.step -= 1;
                if npix == 0 {
                    break;
                }
                npix = 0;
            }
        }
        li.base.li.adjust_forward();
        li
    }

    //---------------------------------------------------------------------
    pub fn step_hor(&mut self) -> bool {
        let mut dist_start;
        let mut dist;
        let mut dy;
        let s1 = self.base.step_hor_base(&mut self.di);

        dist_start = self.di.dist_start();
        let mut p0 = LineIpAaBase::<Ren>::MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        self.base.covers[p1] = 0;
        if dist_start <= 0 {
            self.base.covers[p1] = self.base.ren.cover(s1) as u8;
        }
        p1 += 1;

        dy = 1;
        loop {
            dist = self.base.dist[dy] - s1;
            if dist > self.base.width {
                break;
            }
            dist_start -= self.di.dx_start();
            self.base.covers[p1] = 0;
            if dist_start <= 0 {
                self.base.covers[p1] = self.base.ren.cover(dist) as u8;
            }
            p1 += 1;
            dy += 1;
        }

        dy = 1;
        dist_start = self.di.dist_start();
        loop {
            dist = self.base.dist[dy] + s1;
            if dist > self.base.width {
                break;
            }
            dist_start += self.di.dx_start();
            p0 -= 1;
            self.base.covers[p0] = 0;
            if dist_start <= 0 {
                self.base.covers[p0] = self.base.ren.cover(dist) as u8;
            }
            dy += 1;
        }

        self.base.ren.blend_solid_vspan(
            self.base.x,
            self.base.y - dy as i32 + 1,
            (p1 - p0) as i32,
            &self.base.covers[p0..],
        );
        self.base.step += 1;
        self.base.step < self.base.count
    }

    //---------------------------------------------------------------------
    pub fn step_ver(&mut self) -> bool {
        let mut dist_start;
        let mut dist;
        let mut dx;
        let s1 = self.base.step_ver_base(&mut self.di);
        let mut p0 = LineIpAaBase::<Ren>::MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        dist_start = self.di.dist_start();

        self.base.covers[p1] = 0;
        if dist_start <= 0 {
            self.base.covers[p1] = self.base.ren.cover(s1) as u8;
        }
        p1 += 1;

        dx = 1;
        loop {
            dist = self.base.dist[dx] - s1;
            if dist > self.base.width {
                break;
            }
            dist_start += self.di.dy_start();
            self.base.covers[p1] = 0;
            if dist_start <= 0 {
                self.base.covers[p1] = self.base.ren.cover(dist) as u8;
            }
            p1 += 1;
            dx += 1;
        }

        dx = 1;
        dist_start = self.di.dist_start();
        loop {
            dist = self.base.dist[dx] + s1;
            if dist > self.base.width {
                break;
            }
            dist_start -= self.di.dy_start();
            p0 -= 1;
            self.base.covers[p0] = 0;
            if dist_start <= 0 {
                self.base.covers[p0] = self.base.ren.cover(dist) as u8;
            }
            dx += 1;
        }

        self.base.ren.blend_solid_hspan(
            self.base.x - dx as i32 + 1,
            self.base.y,
            (p1 - p0) as i32,
            &self.base.covers[p0..],
        );
        self.base.step += 1;
        return self.base.step < self.base.count;
    }
}

//====================================================LineIpAa2
pub struct LineIpAa2<'a, Ren: RendererOutline> {
    base: LineIpAaBase<'a, Ren>,
    di: DistanceIp2,
}

impl<'a, Ren: RendererOutline> LineIpAa2<'a, Ren> {
    pub fn new(ren: &'a mut Ren, lp: &'a LineParameters, ex: i32, ey: i32) -> Self {
        let mut ret = Self {
            base: LineIpAaBase::new(ren, lp),
            di: DistanceIp2::new_end(
                lp.x1,
                lp.y1,
                lp.x2,
                lp.y2,
                ex,
                ey,
                lp.x1 & !(LineSubpixel::Mask as i32),
                lp.y1 & !(LineSubpixel::Mask as i32),
            ),
        };
        ret.base.li.adjust_forward();
        ret.base.step -= ret.base.max_extent;
        ret
    }

    //---------------------------------------------------------------------
    pub fn step_hor(&mut self) -> bool {
        let mut dist_end;
        let mut dist;
        let mut dy;
        let s1 = self.base.step_hor_base(&mut self.di);
        let mut p0 = LineIpAaBase::<Ren>::MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        dist_end = self.di.dist_end();

        let mut npix = 0;
        self.base.covers[p1] = 0;
        if dist_end > 0 {
            self.base.covers[p1] = self.base.ren.cover(s1) as u8;
            npix += 1;
        }
        p1 += 1;

        dy = 1;
        loop {
            dist = self.base.dist[dy] - s1;
            if dist > self.base.width {
                break;
            }
            dist_end -= self.di.dx_end();
            self.base.covers[p1] = 0;
            if dist_end > 0 {
                self.base.covers[p1] = self.base.ren.cover(dist) as u8;
                npix += 1;
            }
            p1 += 1;
            dy += 1;
        }

        dy = 1;
        dist_end = self.di.dist_end();
        loop {
            dist = self.base.dist[dy] + s1;
            if dist > self.base.width {
                break;
            }
            dist_end += self.di.dx_end();
            p0 -= 1;
            self.base.covers[p0] = 0;
            if dist_end > 0 {
                self.base.covers[p0] = self.base.ren.cover(dist) as u8;
                npix += 1;
            }
            dy += 1;
        }

        self.base.ren.blend_solid_vspan(
            self.base.x,
            self.base.y - dy as i32 + 1,
            (p1 - p0) as i32,
            &self.base.covers[p0..],
        );
        self.base.step += 1;
        npix > 0 && self.base.step < self.base.count
    }

    //---------------------------------------------------------------------
    pub fn step_ver(&mut self) -> bool {
        let mut dist_end;
        let mut dist;
        let mut dx;
        let s1 = self.base.step_ver_base(&mut self.di);
        let mut p0 = LineIpAaBase::<Ren>::MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        dist_end = self.di.dist_end();

        let mut npix = 0;
        self.base.covers[p1] = 0;
        if dist_end > 0 {
            self.base.covers[p1] = self.base.ren.cover(s1) as u8;
            npix += 1;
        }
        p1 += 1;

        dx = 1;
        loop {
            dist = self.base.dist[dx] - s1;
            if dist > self.base.width {
                break;
            }
            dist_end += self.di.dy_end();
            self.base.covers[p1] = 0;
            if dist_end > 0 {
                self.base.covers[p1] = self.base.ren.cover(dist) as u8;
                npix += 1;
            }
            p1 += 1;
            dx += 1;
        }

        dx = 1;
        dist_end = self.di.dist_end();
        loop {
            dist = self.base.dist[dx] + s1;
            if dist > self.base.width {
                break;
            }
            dist_end -= self.di.dy_end();
            p0 -= 1;
            self.base.covers[p0] = 0;
            if dist_end > 0 {
                self.base.covers[p0] = self.base.ren.cover(dist) as u8;
                npix += 1;
            }
            dx += 1;
        }

        self.base.ren.blend_solid_hspan(
            self.base.x - dx as i32 + 1,
            self.base.y,
            (p1 - p0) as i32,
            &self.base.covers[p0..],
        );
        self.base.step += 1;
        npix > 0 && self.base.step < self.base.count
    }
}

//====================================================LineIpAa3
pub struct LineIpAa3<'a, Ren: RendererOutline> {
    base: LineIpAaBase<'a, Ren>,
    di: DistanceIp3,
}

impl<'a, Ren: RendererOutline> LineIpAa3<'a, Ren> {
    pub fn new(
        ren: &'a mut Ren, lp: &'a LineParameters, sx: i32, sy: i32, ex: i32, ey: i32,
    ) -> Self {
        let mut di = DistanceIp3::new(
            lp.x1,
            lp.y1,
            lp.x2,
            lp.y2,
            sx,
            sy,
            ex,
            ey,
            lp.x1 & !(LineSubpixel::Mask as i32),
            lp.y1 & !(LineSubpixel::Mask as i32),
        );
        let mut npix = 1;
        let mut base = LineIpAaBase::new(ren, lp);
        if lp.vertical {
            loop {
                base.li.dec();
                base.y -= lp.inc;
                base.x = (base.lp.x1 + base.li.y()) >> LineSubpixel::Shift as i32;

                if lp.inc > 0 {
                    di.dec_y_dx(base.x - base.old_x);
                } else {
                    di.inc_y_dx(base.x - base.old_x);
                }

                base.old_x = base.x;

                let mut dist1_start = di.dist_start();
                let mut dist2_start = di.dist_start();

                let mut dx = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start += di.dy_start();
                    dist2_start -= di.dy_start();
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dx += 1;
                    if base.dist[dx] > base.width {
                        break;
                    }
                }
                if npix == 0 {
                    break;
                }
                npix = 0;

				base.step -= 1;
				if base.step < -base.max_extent {
					break;
				}
            }
        } else {
            loop {
                base.li.dec();
                base.x -= lp.inc;
                base.y = (base.lp.y1 + base.li.y()) >> LineSubpixel::Shift as i32;

                if lp.inc > 0 {
                    di.dec_x_dy(base.y - base.old_y);
                } else {
                    di.inc_x_dy(base.y - base.old_y);
                }

                base.old_y = base.y;

                let mut dist1_start = di.dist_start();
                let mut dist2_start = di.dist_start();

                let mut dy = 0;
                if dist1_start < 0 {
                    npix += 1;
                }
                loop {
                    dist1_start -= di.dx_start();
                    dist2_start += di.dx_start();
                    if dist1_start < 0 {
                        npix += 1;
                    }
                    if dist2_start < 0 {
                        npix += 1;
                    }
                    dy += 1;
                    if base.dist[dy] > base.width {
                        break;
                    }
                }
                if npix == 0 {
                    break;
                }
                npix = 0;
				base.step -= 1;
				if base.step < -base.max_extent {
					break;
				}
            }
        }
        base.li.adjust_forward();
        base.step -= base.max_extent;
        LineIpAa3 { base: base, di: di }
    }

    //---------------------------------------------------------------------
    pub fn step_hor(&mut self) -> bool {
        let mut dist_start;
        let mut dist_end;
        let mut dist;
        let mut dy;
        let s1 = self.base.step_hor_base(&mut self.di);
        let mut p0 = LineIpAaBase::<Ren>::MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        dist_start = self.di.dist_start();
        dist_end = self.di.dist_end();

        let mut npix = 0;
        self.base.covers[p1] = 0;
        if dist_end > 0 {
            if dist_start <= 0 {
                self.base.covers[p1] = self.base.ren.cover(s1) as CoverType;
            }
            npix += 1;
        }
        p1 += 1;

        dy = 1;
        loop {
            dist = self.base.dist[dy] - s1;
            if dist > self.base.width {
                break;
            }
            dist_start -= self.di.dx_start();
            dist_end -= self.di.dx_end();
            self.base.covers[p1] = 0;
            if dist_end > 0 && dist_start <= 0 {
                self.base.covers[p1] = self.base.ren.cover(dist) as CoverType;
                npix += 1;
            }
            p1 += 1;
            dy += 1;
        }

        dy = 1;
        dist_start = self.di.dist_start();
        dist_end = self.di.dist_end();
        loop {
            dist = self.base.dist[dy] + s1;
            if dist > self.base.width {
                break;
            }
            dist_start += self.di.dx_start();
            dist_end += self.di.dx_end();
            p0 -= 1;
            self.base.covers[p0] = 0;
            if dist_end > 0 && dist_start <= 0 {
                self.base.covers[p0] = self.base.ren.cover(dist) as CoverType;
                npix += 1;
            }
            dy += 1;
        }

        self.base.ren.blend_solid_vspan(
            self.base.x,
            self.base.y - dy as i32 + 1,
            (p1 - p0) as i32,
            &self.base.covers[p0..],
        );
        npix != 0 && self.base.step < self.base.count
    }

    //---------------------------------------------------------------------
    pub fn step_ver(&mut self) -> bool {
        let mut dist_start;
        let mut dist_end;
        let mut dist;
        let mut dx;
        let s1 = self.base.step_ver_base(&mut self.di);
        let mut p0 = LineIpAaBase::<Ren>::MAX_HALF_WIDTH + 2;
        let mut p1 = p0;

        dist_start = self.di.dist_start();
        dist_end = self.di.dist_end();

        let mut npix = 0;
        self.base.covers[p1] = 0;
        if dist_end > 0 {
            if dist_start <= 0 {
                self.base.covers[p1] = self.base.ren.cover(s1) as CoverType;
            }
            npix += 1;
        }
        p1 += 1;

        dx = 1;
        loop {
            dist = self.base.dist[dx] - s1;
            if dist > self.base.width {
                break;
            }
            dist_start += self.di.dy_start();
            dist_end += self.di.dy_end();
            self.base.covers[p1] = 0;
            if dist_end > 0 && dist_start <= 0 {
                self.base.covers[p1] = self.base.ren.cover(dist) as u8;
                npix += 1;
            }
            p1 += 1;
            dx += 1;
        }

        dx = 1;
        dist_start = self.di.dist_start();
        dist_end = self.di.dist_end();
        loop {
            dist = self.base.dist[dx] + s1;
            if dist > self.base.width {
                break;
            }
            dist_start -= self.di.dy_start();
            dist_end -= self.di.dy_end();
            p0 -= 1;
            self.base.covers[p0] = 0;
            if dist_end > 0 && dist_start <= 0 {
                self.base.covers[p0] = self.base.ren.cover(dist) as u8;
                npix += 1;
            }
            dx += 1;
        }

        self.base.ren.blend_solid_hspan(
            self.base.x - dx as i32 + 1,
            self.base.y,
            (p1 - p0) as i32,
            &self.base.covers[p0..],
        );
        self.base.step += 1;
        return npix != 0 && self.base.step < self.base.count;
    }
}

pub struct LineProfileAA {
    profile: Vec<u8>,
    gamma: [u8; 256],
    subpixel_width: i32,
    min_width: f64,
    smoother_width: f64,
}

impl LineProfileAA {
    pub fn new() -> LineProfileAA {
        let mut gamma = [0; 256];
        for i in 0..256 {
            gamma[i] = i as u8;
        }
        LineProfileAA {
            profile: Vec::new(),
            gamma: gamma,
            subpixel_width: 0,
            min_width: 1.0,
            smoother_width: 1.0,
        }
    }

    pub fn new_gamma<Fn: GammaFn>(w: f64, f: Fn) -> LineProfileAA {
        let mut l = LineProfileAA {
            profile: Vec::new(),
            gamma: [0; 256],
            subpixel_width: 0,
            min_width: 1.0,
            smoother_width: 1.0,
        };
        l.gamma(&f);
        l.set_width(w);
        l
    }

    fn gamma<Gfn: GammaFn>(&mut self, f: &Gfn) {
        for i in 0..AaScale::Scale as usize {
            self.gamma[i] = uround(
                f.call(i as f64 / AaScale::Mask as i32 as f64) * AaScale::Mask as i32 as f64,
            ) as u8;
        }
    }
    pub fn set_width(&mut self, w: f64) {
        let mut w = w;
        if w < 0.0 {
            w = 0.0;
        }

        if w < self.smoother_width {
            w += w;
        } else {
            w += self.smoother_width;
        }

        w *= 0.5;

        w -= self.smoother_width;
        let mut s = self.smoother_width;
        if w < 0.0 {
            s += w;
            w = 0.0;
        }
        self.set(w, s);
    }

    pub fn set_profile(&mut self, w: f64) {
        //-> &mut [u8] {
        self.subpixel_width = uround(w * SubPixel::Scale as i32 as f64);
        let size = self.subpixel_width + 6 * SubPixel::Scale as i32;
        if size > self.profile.len() as i32 {
            self.profile.resize(size as usize, 0);
        }
        //&mut self.profile
    }

    pub fn set(&mut self, center_width: f64, smoother_width: f64) {
        let (mut center_width, mut smoother_width) = (center_width, smoother_width);
        let mut base_val = 1.0;
        if center_width == 0.0 {
            center_width = 1.0 / SubPixel::Scale as i32 as f64;
        }
        if smoother_width == 0.0 {
            smoother_width = 1.0 / SubPixel::Scale as i32 as f64;
        }

        let width = center_width + smoother_width;
        if width < self.min_width {
            let k = width / self.min_width;
            base_val *= k;
            center_width /= k;
            smoother_width /= k;
        }

        let mut ch: usize;
        self.set_profile(center_width + smoother_width); //XXXX
                                                     //let arr = &mut self.profile;
        let subpixel_center_width = (center_width * SubPixel::Scale as i32 as f64) as usize;
        let subpixel_smoother_width = (smoother_width * SubPixel::Scale as i32 as f64) as usize;

        let mut ch_center = (SubPixel::Scale as i32 * 2) as usize;
        let mut ch_smoother = ch_center + subpixel_center_width;

        let mut val = self.gamma[(base_val * AaScale::Mask as i32 as f64) as usize];
        ch = ch_center;
        for _ in 0..subpixel_center_width {
            self.profile[ch] = val;
            ch += 1;
        }

        for i in 0..subpixel_smoother_width {
            self.profile[ch_smoother] =
                self.gamma[((base_val - base_val * (i as f64 / subpixel_smoother_width as f64))
                    * AaScale::Mask as i32 as f64) as u32 as usize];
            ch_smoother += 1;
        }

        let n_smoother = self.profile_size()
            - subpixel_smoother_width as u32
            - subpixel_center_width as u32
            - SubPixel::Scale as u32 * 2;

        val = self.gamma[0];
        for _ in 0..n_smoother {
            self.profile[ch_smoother] = val;
            ch_smoother += 1;
        }

        ch = ch_center;
        for _ in 0..SubPixel::Scale as i32 * 2 {
            ch -= 1;
            self.profile[ch] = self.profile[ch_center];
            ch_center += 1;
        }
    }

    pub fn min_width(&self) -> f64 {
        self.min_width
    }

    pub fn smoother_width(&self) -> f64 {
        self.smoother_width
    }

	pub fn set_min_width(&mut self, v: f64) {
        self.min_width = v;
    }

    pub fn set_smoother_width(&mut self, v: f64) {
        self.smoother_width = v;
    }

    pub fn value(&self, dist: i32) -> u8 {
        self.profile[(dist + SubPixel::Scale as i32 * 2) as usize]
    }

    pub fn profile_size(&self) -> u32 {
        self.profile.len() as u32
    }

    pub fn subpixel_width(&self) -> i32 {
        self.subpixel_width
    }
}

//======================================================RendererOutlineAa
pub struct RendererOutlineAa<'a, R: Renderer> {
    ren: &'a mut R,
    profile: LineProfileAA,
    clip_box: RectI,
    clipping: bool,
    color: R::C,
}

impl<'a, R: Renderer> RendererOutline for RendererOutlineAa<'a, R> {
    type C = R::C;

    fn pixel(&self, _p: &mut Self::C, _x: i32, _y: i32) {
        todo!()
    }

    fn pattern_width(&self) -> i32 {
        todo!()
    }

    fn blend_color_hspan(&mut self, _x: i32, _y: i32, _len: u32, _colors: &[Self::C]) {
        todo!()
    }
    fn blend_color_vspan(&mut self, _x: i32, _y: i32, _len: u32, _colors: &[Self::C]) {
        todo!()
    }

    fn set_color(&mut self, c: R::C) {
        self.color = c;
    }

    fn cover(&self, d: i32) -> i32 {
        self.profile.value(d) as i32
    }

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: i32, covers: &[u8]) {
        self.ren
            .blend_solid_hspan(x, y, len as i32, &self.color, covers);
    }

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: i32, covers: &[u8]) {
        self.ren
            .blend_solid_vspan(x, y, len as i32, &self.color, covers);
    }
    fn subpixel_width(&self) -> i32 {
        self.profile.subpixel_width()
    }

    fn accurate_join_only(&self) -> bool {
        false
    }

    fn line0(&mut self, lp: &LineParameters) {
        if self.clipping {
            let mut x1 = lp.x1;
            let mut y1 = lp.y1;
            let mut x2 = lp.x2;
            let mut y2 = lp.y2;
            let flags = clip_line_segment(&mut x1, &mut y1, &mut x2, &mut y2, &self.clip_box);
            if (flags & 4) == 0 {
                if flags != 0 {
                    let lp2 = LineParameters::new(
                        x1,
                        y1,
                        x2,
                        y2,
                        uround(calc_distance(x1 as f64, y1 as f64, x2 as f64, y2 as f64)),
                    );
                    self.line0_no_clip(&lp2);
                } else {
                    self.line0_no_clip(lp);
                }
            }
        } else {
            self.line0_no_clip(lp);
        }
    }

    fn line1(&mut self, lp: &LineParameters, sx: i32, sy: i32) {
        let (mut sx, mut sy) = (sx, sy);
        if self.clipping {
            let mut x1 = lp.x1;
            let mut y1 = lp.y1;
            let mut x2 = lp.x2;
            let mut y2 = lp.y2;
            let flags = clip_line_segment(&mut x1, &mut y1, &mut x2, &mut y2, &self.clip_box);
            if (flags & 4) == 0 {
                if flags != 0 {
                    let lp2 = LineParameters::new(
                        x1,
                        y1,
                        x2,
                        y2,
                        uround(calc_distance(x1 as f64, y1 as f64, x2 as f64, y2 as f64)),
                    );
                    if (flags & 1) != 0 {
                        sx = x1 + (y2 - y1);
                        sy = y1 - (x2 - x1);
                    } else {
                        while (sx - lp.x1).abs() + (sy - lp.y1).abs() > lp2.len {
                            sx = (lp.x1 + sx) >> 1;
                            sy = (lp.y1 + sy) >> 1;
                        }
                    }
                    self.line1_no_clip(&lp2, sx, sy);
                } else {
                    self.line1_no_clip(lp, sx, sy);
                }
            }
        } else {
            self.line1_no_clip(lp, sx, sy);
        }
    }

    fn line2(&mut self, lp: &LineParameters, ex: i32, ey: i32) {
        let (mut ex, mut ey) = (ex, ey);
        if self.clipping {
            let mut x1 = lp.x1;
            let mut y1 = lp.y1;
            let mut x2 = lp.x2;
            let mut y2 = lp.y2;
            let flags = clip_line_segment(&mut x1, &mut y1, &mut x2, &mut y2, &self.clip_box);
            if (flags & 4) == 0 {
                if flags != 0 {
                    let lp2 = LineParameters::new(
                        x1,
                        y1,
                        x2,
                        y2,
                        uround(calc_distance(x1 as f64, y1 as f64, x2 as f64, y2 as f64)),
                    );
                    if flags & 2 != 0 {
                        ex = x2 + (y2 - y1);
                        ey = y2 - (x2 - x1);
                    } else {
                        while (ex - lp.x2).abs() + (ey - lp.y2).abs() > lp2.len {
                            ex = (lp.x2 + ex) >> 1;
                            ey = (lp.y2 + ey) >> 1;
                        }
                    }
                    self.line2_no_clip(&lp2, ex, ey);
                } else {
                    self.line2_no_clip(lp, ex, ey);
                }
            }
        } else {
            self.line2_no_clip(lp, ex, ey);
        }
    }

    fn line3(&mut self, lp: &LineParameters, sx: i32, sy: i32, ex: i32, ey: i32) {
        let (mut sx, mut sy, mut ex, mut ey) = (sx, sy, ex, ey);
        if self.clipping {
            let mut x1 = lp.x1;
            let mut y1 = lp.y1;
            let mut x2 = lp.x2;
            let mut y2 = lp.y2;
            let flags = clip_line_segment(&mut x1, &mut y1, &mut x2, &mut y2, &self.clip_box);
            if (flags & 4) == 0 {
                if flags != 0 {
                    let lp2 = LineParameters::new(
                        x1,
                        y1,
                        x2,
                        y2,
                        uround(calc_distance(x1 as f64, y1 as f64, x2 as f64, y2 as f64)),
                    );
                    if flags & 1 != 0 {
                        sx = x1 + (y2 - y1);
                        sy = y1 - (x2 - x1);
                    } else {
                        while (sx - lp.x1).abs() + (sy - lp.y1).abs() > lp2.len {
                            sx = (lp.x1 + sx) >> 1;
                            sy = (lp.y1 + sy) >> 1;
                        }
                    }
                    if flags & 2 != 0 {
                        ex = x2 + (y2 - y1);
                        ey = y2 - (x2 - x1);
                    } else {
                        while (ex - lp.x2).abs() + (ey - lp.y2).abs() > lp2.len {
                            ex = (lp.x2 + ex) >> 1;
                            ey = (lp.y2 + ey) >> 1;
                        }
                    }
                    self.line3_no_clip(&lp2, sx, sy, ex, ey);
                } else {
                    self.line3_no_clip(lp, sx, sy, ex, ey);
                }
            }
        } else {
            self.line3_no_clip(lp, sx, sy, ex, ey);
        }
    }

    fn pie(&mut self, xc: i32, yc: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut r =
            (self.subpixel_width() + LineSubpixel::Mask as i32) >> LineSubpixel::Shift as i32;
        if r < 1 {
            r = 1
        };
        let mut ei = EllipseBresenhamIp::new(r, r);
        let mut dx = 0;
        let mut dy = -r;
        let mut dy0 = dy;
        let mut dx0 = dx;
        let x = xc >> LineSubpixel::Shift as i32;
        let y = yc >> LineSubpixel::Shift as i32;

        loop {
            dx += ei.dx();
            dy += ei.dy();

            if dy != dy0 {
                self.pie_hline(xc, yc, x1, y1, x2, y2, x - dx0, y + dy0, x + dx0);
                self.pie_hline(xc, yc, x1, y1, x2, y2, x - dx0, y - dy0, x + dx0);
            }
            dx0 = dx;
            dy0 = dy;
            ei.inc();
            if dy >= 0 {
                break;
            }
        }

        self.pie_hline(xc, yc, x1, y1, x2, y2, x - dx0, y + dy0, x + dx0);
    }

    fn semidot<Cmp>(&mut self, cmp: Cmp, xc1: i32, yc1: i32, xc2: i32, yc2: i32)
    where
        Cmp: Fn(i32) -> bool,
    {
        if self.clipping && clipping_flags(xc1, yc1, &self.clip_box) != 0 {
            return;
        }

        let mut r =
            (self.subpixel_width() + LineSubpixel::Mask as i32) >> LineSubpixel::Shift as i32;
        if r < 1 {
            r = 1;
        }
        let mut ei = EllipseBresenhamIp::new(r, r);
        let mut dx = 0;
        let mut dy = -r;
        let mut dy0 = dy;
        let mut dx0 = dx;
        let x = xc1 >> LineSubpixel::Shift as i32;
        let y = yc1 >> LineSubpixel::Shift as i32;

        loop {
            dx += ei.dx();
            dy += ei.dy();

            if dy != dy0 {
                self.semidot_hline(&cmp, xc1, yc1, xc2, yc2, x - dx0, y + dy0, x + dx0);
                self.semidot_hline(&cmp, xc1, yc1, xc2, yc2, x - dx0, y - dy0, x + dx0);
            }
            dx0 = dx;
            dy0 = dy;
            ei.inc();
            if dy >= 0 {
                break;
            }
        }
        self.semidot_hline(cmp, xc1, yc1, xc2, yc2, x - dx0, y + dy0, x + dx0);
    }
}

impl<'a, R: Renderer + 'a> RendererOutlineAa<'a, R> {
    pub fn new(ren: &'a mut R, prof: LineProfileAA) -> Self {
        RendererOutlineAa {
            ren: ren,
            profile: prof,
            clip_box: RectI::new(0, 0, 0, 0),
            clipping: false,
            color: R::C::new(),
        }
    }

    fn get_color(&self) -> R::C {
        self.color
    }

    pub fn attach(&mut self, ren: &'a mut R) {
        self.ren = ren;
    }

	pub fn ren_mut(&mut self) -> &mut R {
        return self.ren;
    }

    pub fn ren(&self) -> &R {
        return &*self.ren;
    }
	
    pub fn profile(&self) -> &LineProfileAA {
        &self.profile
    }

    pub fn set_profile(&mut self, prof: LineProfileAA) {
        self.profile = prof;
    }

    pub fn reset_clipping(&mut self) {
        self.clipping = false;
    }

    pub fn clip_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.clip_box.x1 = LineCoordSat::conv(x1);
        self.clip_box.y1 = LineCoordSat::conv(y1);
        self.clip_box.x2 = LineCoordSat::conv(x2);
        self.clip_box.y2 = LineCoordSat::conv(y2);
        self.clipping = true;
    }

    pub fn semidot_hline<Cmp>(
        &mut self, cmp: Cmp, xc1: i32, yc1: i32, xc2: i32, yc2: i32, x1: i32, y1: i32, x2: i32,
    ) where
        Cmp: Fn(i32) -> bool,
    {
        let mut covers = [0u8; /*LineIpAaBase::<Self>::*/MAX_HALF_WIDTH * 2 + 4];
        let p0 = 0;
        let mut p1 = 0;
        let mut x = x1 << LineSubpixel::Shift as i32;
        let mut y = y1 << LineSubpixel::Shift as i32;
        let mut x1 = x1;

        let w = self.subpixel_width();
        let mut di = DistanceIp0::new(xc1, yc1, xc2, yc2, x, y);
        x += LineSubpixel::Scale as i32 / 2;
        y += LineSubpixel::Scale as i32 / 2;

        let x0 = x1;
        let mut dx = x - xc1;
        let dy = y - yc1;
        loop {
            let d = ((dx * dx + dy * dy) as f64).sqrt() as i32;
            covers[p1] = 0;
            if cmp(di.dist()) && d <= w {
                covers[p1] = self.cover(d) as u8;
            }
            p1 += 1;
            dx += LineSubpixel::Scale as i32;
            di.inc_x();
			x1 += 1;
            if x1 > x2 {
                break;
            }
            
        }

        self.ren
            .blend_solid_hspan(x0, y1, (p1 - p0) as i32, &self.get_color(), &covers);
    }

    fn pie_hline(
        &mut self, xc: i32, yc: i32, xp1: i32, yp1: i32, xp2: i32, yp2: i32, xh1: i32, yh1: i32,
        xh2: i32,
    ) {
        if self.clipping && clipping_flags(xc, yc, &self.clip_box) != 0 {
            return;
        }

        let mut covers = [0u8; /*LineIpAaBase::<Self>::*/MAX_HALF_WIDTH * 2 + 4];
        let p0 = 0;
        let mut p1 = 0;
        let mut x = xh1 << LineSubpixel::Shift as i32;
        let mut y = yh1 << LineSubpixel::Shift as i32;
        let w = self.subpixel_width();
        let mut xh1 = xh1;

        let mut di = DistanceIp00::new(xc, yc, xp1, yp1, xp2, yp2, x, y);
        x += LineSubpixel::Scale as i32 / 2;
        y += LineSubpixel::Scale as i32 / 2;

        let xh0 = xh1;
        let mut dx = x - xc;
        let dy = y - yc;
        while xh1 <= xh2 {
            let d = ((dx * dx + dy * dy) as f64).sqrt() as i32;
            covers[p1] = 0;
            if di.dist1() <= 0 && di.dist2() > 0 && d <= w {
                covers[p1] = self.cover(d) as u8;
            }
            p1 += 1;
            dx += LineSubpixel::Scale as i32;
            di.inc_x();
            xh1 += 1;
        }

        self.ren
            .blend_solid_hspan(xh0, yh1, (p1 - p0) as i32, &self.get_color(), &covers);
    }

    fn line0_no_clip(&mut self, lp: &LineParameters) {
        if lp.len > line_aa_basics::MAX_LENGTH {
            let (mut lp1, mut lp2) = (LineParameters::new_default(), LineParameters::new_default());
            lp.divide(&mut lp1, &mut lp2);
            self.line0_no_clip(&lp1);
            self.line0_no_clip(&lp2);
            return;
        }

        let mut li = LineIpAa0::<Self>::new(self, lp);
        if li.base.count() > 0 {
            if li.base.vertical() {
                while li.step_ver() {}
            } else {
                while li.step_hor() {}
            }
        }
    }

    fn line1_no_clip(&mut self, lp: &LineParameters, sx: i32, sy: i32) {
        let (mut sx, mut sy) = (sx, sy);
        if lp.len > line_aa_basics::MAX_LENGTH {
            let (lp1, lp2) = (LineParameters::new_default(), LineParameters::new_default());
            self.line1_no_clip(&lp1, (lp.x1 + sx) >> 1, (lp.y1 + sy) >> 1);
            self.line1_no_clip(&lp2, lp1.x2 + (lp1.y2 - lp1.y1), lp1.y2 - (lp1.x2 - lp1.x1));
            return;
        }

        fix_degenerate_bisectrix_start(lp, &mut sx, &mut sy);
        let mut li = LineIpAa1::<Self>::new(self, lp, sx, sy);
        if li.base.vertical() {
            while li.step_ver() {}
        } else {
            while li.step_hor() {}
        }
    }

    fn line2_no_clip(&mut self, lp: &LineParameters, ex: i32, ey: i32) {
        let (mut ex, mut ey) = (ex, ey);
        if lp.len > line_aa_basics::MAX_LENGTH {
            let (mut lp1, mut lp2) = (LineParameters::new_default(), LineParameters::new_default());
            lp.divide(&mut lp1, &mut lp2);
            self.line2_no_clip(&lp1, lp1.x2 + (lp1.y2 - lp1.y1), lp1.y2 - (lp1.x2 - lp1.x1));
            self.line2_no_clip(&lp2, (lp.x2 + ex) >> 1, (lp.y2 + ey) >> 1);
            return;
        }

        fix_degenerate_bisectrix_end(lp, &mut ex, &mut ey);
        let mut li = LineIpAa2::new(self, lp, ex, ey);
        if li.base.vertical() {
            while li.step_ver() {}
        } else {
            while li.step_hor() {}
        }
    }

    fn line3_no_clip(&mut self, lp: &LineParameters, sx: i32, sy: i32, ex: i32, ey: i32) {
        let (mut sx, mut sy, mut ex, mut ey) = (sx, sy, ex, ey);
        if lp.len > line_aa_basics::MAX_LENGTH {
            let (mut lp1, mut lp2) = (LineParameters::new_default(), LineParameters::new_default());
            lp.divide(&mut lp1, &mut lp2);
            let mx = lp1.x2 + (lp1.y2 - lp1.y1);
            let my = lp1.y2 - (lp1.x2 - lp1.x1);
            self.line3_no_clip(&lp1, (lp.x1 + sx) >> 1, (lp.y1 + sy) >> 1, mx, my);
            self.line3_no_clip(&lp2, mx, my, (lp.x2 + ex) >> 1, (lp.y2 + ey) >> 1);
            return;
        }

        fix_degenerate_bisectrix_start(lp, &mut sx, &mut sy);
        fix_degenerate_bisectrix_end(lp, &mut ex, &mut ey);
        let mut li = LineIpAa3::new(self, lp, sx, sy, ex, ey);
        if li.base.vertical() {
            while li.step_ver() {}
        } else {
            while li.step_hor() {}
        }
    }
}
