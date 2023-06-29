use crate::array::*;
use crate::basics::{iround, uceil, ufloor, uround, CoverScale, RectI};
use crate::clip_liang_barsky::*;
use crate::dda_line::*;
use crate::line_aa_basics::{LineMRSubpixel, LineParameters, LineSubpixel, MAX_LENGTH, *};
use crate::math::*;
use crate::rendering_buffer::*;
use crate::{
    Color, Coord, ImagePattern, PatternFilter, Pixel, RenderBuffer, Renderer, RendererOutline,
};

// NOT TESTED

//========================================================LineImageScale
pub struct LineImageScale<Src: Pixel> {
    source: Src,
    height: f64,
    scale: f64,
}

impl<Src: Pixel> LineImageScale<Src> {
    pub fn new(src: Src, height: f64) -> Self {
        let h = src.height();
        LineImageScale {
            source: src,
            height: height,
            scale: h as f64 / height,
        }
    }
}
impl<Src: Pixel> Pixel for LineImageScale<Src> {
    type ColorType = Src::ColorType;

    fn width(&self) -> f64 {
        self.source.width()
    }

    fn height(&self) -> f64 {
        self.height
    }

    fn pixel(&self, x: i32, y: i32) -> Src::ColorType {
        let src_y = (y as f64 + 0.5) * self.scale - 0.5;
        let h = self.source.height() as i32 - 1;
        let y1 = ufloor(src_y) as i32;
        let y2 = y1 + 1;
        let pix1 = if y1 < 0 {
            Src::ColorType::no_color()
        } else {
            self.source.pixel(x, y1 as i32)
        };
        let pix2 = if y2 > h {
            Src::ColorType::no_color()
        } else {
            self.source.pixel(x, y2 as i32)
        };
        pix1.gradient(&pix2, src_y - y1 as f64)
    }
}

//======================================================LineImagePattern
pub struct LineImagePattern<'a, Filter: PatternFilter> {
    filter: Filter,
    dilation: u32,
    dilation_hr: i32,
    data: PodArray<Filter::ColorType>,
    width: u32,
    height: u32,
    width_hr: i32,
    half_height_hr: i32,
    offset_y_hr: i32,
    //m_buf: RowAccessBuf<Filter::ColorType>,
    buf: RowPtrCacheBuf<'a, Filter::ColorType>,
}

impl<'a, Filter: PatternFilter> LineImagePattern<'a, Filter> {
    pub fn new(filter: Filter) -> Self {
        let dil = filter.dilation();
        LineImagePattern {
            filter: filter,
            dilation: dil + 1,
            dilation_hr: (dil as i32 + 1) << LineSubpixel::Shift as i32,
            data: Vec::new(),
            width: 0,
            height: 0,
            width_hr: 0,
            half_height_hr: 0,
            offset_y_hr: 0,
            buf: RowPtrCacheBuf::new_default(),
        }
    }

    pub fn width(&self) -> f64 {
        self.height as f64
    }

    pub fn filter(&self) -> &Filter {
        &self.filter
    }
}

impl<'a, Filter: PatternFilter> ImagePattern for LineImagePattern<'a, Filter> {
    type ColorType = Filter::ColorType;
    fn pixel(&self, p: &mut Self::ColorType, x: i32, y: i32) {
        self.filter.pixel_high_res(
            self.buf.rows(),
            p,
            x % self.width_hr + self.dilation_hr,
            y + self.offset_y_hr,
        );
    }

    fn pattern_width(&self) -> i32 {
        self.width_hr
    }

    fn line_width(&self) -> i32 {
        self.half_height_hr
    }

    fn create<Src: Pixel<ColorType = Filter::ColorType>>(&mut self, src: &Src) {
        self.height = uceil(src.height() as f64);
        self.width = uceil(src.width() as f64);
        self.width_hr = uround(src.width() as f64 * LineSubpixel::Scale as i32 as f64);
        self.half_height_hr = uround(src.height() as f64 * LineSubpixel::Scale as i32 as f64 / 2.);
        self.offset_y_hr = self.dilation_hr + self.half_height_hr - LineSubpixel::Scale as i32 / 2;
        self.half_height_hr += LineSubpixel::Scale as i32 / 2;

        self.data.resize(
            ((self.width + self.dilation * 2) * (self.height + self.dilation * 2)) as usize,
            Filter::ColorType::new(),
        );

        self.buf.attach(
            self.data.as_mut_ptr(),
            self.width + self.dilation * 2,
            self.height + self.dilation * 2,
            (self.width + self.dilation * 2) as i32,
        );

        let mut d1: _;

        for y in 0..self.height {
            d1 = self.buf.row_mut((y + self.dilation) as i32);

            for x in 0..self.width {
                d1[(self.dilation + x) as usize] = src.pixel(x as i32, y as i32);
            }
        }

        for y in 0..self.dilation {
            d1 = self.buf.row_mut((self.dilation + self.height + y) as i32);

            for x in 0..self.width {
                d1[(self.dilation + x) as usize] = Filter::ColorType::no_color();
            }
            d1 = self.buf.row_mut((self.dilation - y - 1) as i32);

            for x in 0..self.width {
                d1[(self.dilation + x) as usize] = Filter::ColorType::no_color();
            }
        }

        let h = self.height + self.dilation * 2;

        for y in 0..h as i32 {
            let s1 = self.dilation as usize;
            let s2 = (self.dilation + self.width) as usize;
            let d1 = (self.dilation + self.width) as usize;
            let d2 = self.dilation as usize;
            let buf = self.buf.row_mut(y);

            for x in 0..self.dilation as usize {
                buf[d1 + x] = buf[s1 + x];
                buf[d2 - x - 1] = buf[s2 - x - 1];
            }
        }
    }
}

pub struct LineImagePatternPow2<'a, F: PatternFilter> {
    base: LineImagePattern<'a, F>,
    mask: u32,
}

impl<'a, F: PatternFilter> LineImagePatternPow2<'a, F> {
    pub fn new(filter: F) -> Self {
        LineImagePatternPow2 {
            base: LineImagePattern::new(filter),
            mask: LineSubpixel::Mask as u32,
        }
    }
}
impl<'a, F: PatternFilter> ImagePattern for LineImagePatternPow2<'a, F> {
    type ColorType = F::ColorType;
    fn pixel(&self, p: &mut F::ColorType, x: i32, y: i32) {
        self.base.filter.pixel_high_res(
            self.base.buf.rows(),
            p,
            (x & self.mask as i32) + self.base.dilation_hr,
            y + self.base.offset_y_hr,
        );
    }

    fn pattern_width(&self) -> i32 {
        self.base.pattern_width()
    }

    fn line_width(&self) -> i32 {
        self.base.line_width()
    }

    fn create<S: Pixel<ColorType = F::ColorType>>(&mut self, src: &S) {
        self.base.create(src);
        self.mask = 1;
        while self.mask < self.base.width as u32 {
            self.mask <<= 1;
            self.mask |= 1;
        }
        self.mask <<= LineSubpixel::Shift as u32 - 1;
        self.mask |= LineSubpixel::Mask as u32;
        self.base.width_hr = self.mask as i32 + 1;
    }
}

//===================================================DistanceIp4
pub struct DistanceIp4 {
    dx: i32,
    dy: i32,
    dx_start: i32,
    dy_start: i32,
    dx_pict: i32,
    dy_pict: i32,
    dx_end: i32,
    dy_end: i32,
    dist: i32,
    dist_start: i32,
    dist_pict: i32,
    dist_end: i32,
    len: i32,
}

impl DistanceIp4 {
    pub fn new(
        x1: i32, y1: i32, x2: i32, y2: i32, sx: i32, sy: i32, ex: i32, ey: i32, len: i32,
        scale: f64, x: i32, y: i32,
    ) -> DistanceIp4 {
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

        let len_ = uround(len as f64 / scale);

        let d = len as f64 * scale;
        let dx0 = iround(((x2 - x1) << LineSubpixel::Shift as i32) as f64 / d);
        let dy0 = iround(((y2 - y1) << LineSubpixel::Shift as i32) as f64 / d);
        let dx_pict = -dy0;
        let dy_pict = dx0;
        let dist_pict = ((x + LineSubpixel::Scale as i32 / 2 - (x1 - dy0)) * dy_pict
            - (y + LineSubpixel::Scale as i32 / 2 - (y1 + dx0)) * dx_pict)
            >> LineSubpixel::Shift as i32;

        let dx = dx << LineSubpixel::Shift as i32;
        let dy = dy << LineSubpixel::Shift as i32;
        let dx_start = dx_start << LineMRSubpixel::Shift as i32;
        let dy_start = dy_start << LineMRSubpixel::Shift as i32;
        let dx_end = dx_end << LineMRSubpixel::Shift as i32;
        let dy_end = dy_end << LineMRSubpixel::Shift as i32;

        DistanceIp4 {
            dx,
            dy,
            dx_start,
            dy_start,
            dx_pict,
            dy_pict,
            dx_end,
            dy_end,
            dist,
            dist_start,
            dist_pict,
            dist_end,
            len: len_,
        }
    }

    pub fn inc_x(&mut self) {
        self.dist += self.dy;
        self.dist_start += self.dy_start;
        self.dist_pict += self.dy_pict;
        self.dist_end += self.dy_end;
    }

    pub fn dec_x(&mut self) {
        self.dist -= self.dy;
        self.dist_start -= self.dy_start;
        self.dist_pict -= self.dy_pict;
        self.dist_end -= self.dy_end;
    }

    pub fn inc_y(&mut self) {
        self.dist -= self.dx;
        self.dist_start -= self.dx_start;
        self.dist_pict -= self.dx_pict;
        self.dist_end -= self.dx_end;
    }

    pub fn dec_y(&mut self) {
        self.dist += self.dx;
        self.dist_start += self.dx_start;
        self.dist_pict += self.dx_pict;
        self.dist_end += self.dx_end;
    }

    pub fn inc_x_dy(&mut self, dy: i32) {
        self.dist += self.dy;
        self.dist_start += self.dy_start;
        self.dist_pict += self.dy_pict;
        self.dist_end += self.dy_end;
        if dy > 0 {
            self.dist -= self.dx;
            self.dist_start -= self.dx_start;
            self.dist_pict -= self.dx_pict;
            self.dist_end -= self.dx_end;
        }
        if dy < 0 {
            self.dist += self.dx;
            self.dist_start += self.dx_start;
            self.dist_pict += self.dx_pict;
            self.dist_end += self.dx_end;
        }
    }

    pub fn dec_x_dy(&mut self, dy: i32) {
        self.dist -= self.dy;
        self.dist_start -= self.dy_start;
        self.dist_pict -= self.dy_pict;
        self.dist_end -= self.dy_end;
        if dy > 0 {
            self.dist -= self.dx;
            self.dist_start -= self.dx_start;
            self.dist_pict -= self.dx_pict;
            self.dist_end -= self.dx_end;
        }
        if dy < 0 {
            self.dist += self.dx;
            self.dist_start += self.dx_start;
            self.dist_pict += self.dx_pict;
            self.dist_end += self.dx_end;
        }
    }

    pub fn inc_y_dx(&mut self, dx: i32) {
        self.dist -= self.dx;
        self.dist_start -= self.dx_start;
        self.dist_pict -= self.dx_pict;
        self.dist_end -= self.dx_end;
        if dx > 0 {
            self.dist += self.dy;
            self.dist_start += self.dy_start;
            self.dist_pict += self.dy_pict;
            self.dist_end += self.dy_end;
        }
        if dx < 0 {
            self.dist -= self.dy;
            self.dist_start -= self.dy_start;
            self.dist_pict -= self.dy_pict;
            self.dist_end -= self.dy_end;
        }
    }

    pub fn dec_y_dx(&mut self, dx: i32) {
        self.dist += self.dx;
        self.dist_start += self.dx_start;
        self.dist_pict += self.dx_pict;
        self.dist_end += self.dx_end;
        if dx > 0 {
            self.dist += self.dy;
            self.dist_start += self.dy_start;
            self.dist_pict += self.dy_pict;
            self.dist_end += self.dy_end;
        }
        if dx < 0 {
            self.dist -= self.dy;
            self.dist_start -= self.dy_start;
            self.dist_pict -= self.dy_pict;
            self.dist_end -= self.dy_end;
        }
    }

    pub fn dist(&self) -> i32 {
        self.dist
    }

    pub fn dist_start(&self) -> i32 {
        self.dist_start
    }

    pub fn dist_pict(&self) -> i32 {
        self.dist_pict
    }

    pub fn dist_end(&self) -> i32 {
        self.dist_end
    }

    pub fn dx(&self) -> i32 {
        self.dx
    }

    pub fn dy(&self) -> i32 {
        self.dy
    }

    pub fn dx_start(&self) -> i32 {
        self.dx_start
    }

    pub fn dy_start(&self) -> i32 {
        self.dy_start
    }

    pub fn dx_pict(&self) -> i32 {
        self.dx_pict
    }

    pub fn dy_pict(&self) -> i32 {
        self.dy_pict
    }

    pub fn dx_end(&self) -> i32 {
        self.dx_end
    }

    pub fn dy_end(&self) -> i32 {
        self.dy_end
    }

    pub fn len(&self) -> i32 {
        self.len
    }
}

const MAX_HALF_WIDTH: usize = 64;
pub struct LineIpImage<'a, R: RendererOutline> {
    lp: LineParameters,
    li: Dda2LineIp,
    di: DistanceIp4,
    ren: &'a mut R,
    x: i32,
    y: i32,
    old_x: i32,
    old_y: i32,
    count: i32,
    width: i32,
    _max_extent: i32,
    start: i32,
    step: i32,
    dist_pos: [i32; MAX_HALF_WIDTH + 1],
    colors: [R::C; MAX_HALF_WIDTH * 2 + 4],
}

impl<'a, R: RendererOutline> LineIpImage<'a, R> {
    const MAX_HALF_WIDTH: u32 = 64;
    pub fn new(
        ren: &'a mut R, lp: LineParameters, sx: i32, sy: i32, ex: i32, ey: i32, pattern_start: i32,
        scale_x: f64,
    ) -> Self {
        let mut li = Dda2LineIp::new_bwd_y(
            if lp.vertical {
                line_dbl_hr(lp.x2 - lp.x1)
            } else {
                line_dbl_hr(lp.y2 - lp.y1)
            },
            if lp.vertical {
                (lp.y2 - lp.y1).abs()
            } else {
                (lp.x2 - lp.x1).abs() + 1
            },
        );
        let mut di = DistanceIp4::new(
            lp.x1,
            lp.y1,
            lp.x2,
            lp.y2,
            sx,
            sy,
            ex,
            ey,
            lp.len,
            scale_x,
            lp.x1 & !(LineSubpixel::Mask as i32),
            lp.y1 & !(LineSubpixel::Mask as i32),
        );
        let mut x = lp.x1 >> LineSubpixel::Shift as i32;
        let mut y = lp.y1 >> LineSubpixel::Shift as i32;
        let mut old_x = x;
        let mut old_y = y;
        let count = if lp.vertical {
            ((lp.y2 >> LineSubpixel::Shift as i32) - y).abs()
        } else {
            ((lp.x2 >> LineSubpixel::Shift as i32) - x).abs()
        };
        let width = ren.subpixel_width();
        let max_extent = (width + LineSubpixel::Scale as i32) >> LineSubpixel::Shift as i32;
        let start = pattern_start + (max_extent + 2) * ren.pattern_width();
        let mut step = 0;

        let mut dist_pos = [0; MAX_HALF_WIDTH + 1];

        let mut li0 = Dda2LineIp::new_fwd(
            0,
            if lp.vertical {
                lp.dy << LineSubpixel::Shift as i32
            } else {
                lp.dx << LineSubpixel::Shift as i32
            },
            lp.len,
        );
        let mut i: usize = 0;
        let stop = width + LineSubpixel::Scale as i32 * 2;
        while i < MAX_HALF_WIDTH {
            dist_pos[i] = li0.y();
            if dist_pos[i] >= stop {
                break;
            }
            li0.inc();
            i += 1;
        }
        dist_pos[i] = 0x7FFF0000;

        let mut npix = 1;

        if lp.vertical {
            loop {
                li.dec();
                y -= lp.inc;
                x = (lp.x1 + li.y()) >> LineSubpixel::Shift as i32;

                if lp.inc > 0 {
                    di.dec_y_dx(x - old_x);
                } else {
                    di.inc_y_dx(x - old_x);
                }

                old_x = x;

                let mut dist1_start = di.dist_start();
                let mut dist2_start = dist1_start;

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
                    if dist_pos[dx] > width {
                        break;
                    }
                }
                if npix == 0 {
                    break;
                }

                npix = 0;
                step -= 1;
                if step < -max_extent {
                    break;
                }
            }
        } else {
            loop {
                li.dec();

                x -= lp.inc;
                y = (lp.y1 + li.y()) >> LineSubpixel::Shift as i32;

                if lp.inc > 0 {
                    di.dec_x_dy(y - old_y);
                } else {
                    di.inc_x_dy(y - old_y);
                }

                old_y = y;

                let mut dist1_start = di.dist_start();
                let mut dist2_start = dist1_start;

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
                    if dist_pos[dy] > width {
                        break;
                    }
                }
                if npix == 0 {
                    break;
                }

                npix = 0;
                step -= 1;
                if step < -max_extent {
                    break;
                }
            }
        }
        li.adjust_forward();
        step -= max_extent;

        Self {
            lp: lp,
            li: li,
            di: di,
            ren: ren,
            x: x,
            y: y,
            old_x: x,
            old_y: y,
            count: count,
            width: width,
            _max_extent: max_extent,
            start: start,
            step: step,
            dist_pos: dist_pos,
            colors: [R::C::new(); MAX_HALF_WIDTH * 2 + 4],
        }
    }

    pub fn step_hor(&mut self) -> bool {
        self.li.inc();
        self.x += self.lp.inc;
        self.y = (self.lp.y1 + self.li.y()) >> LineSubpixel::Shift as i32;

        if self.lp.inc > 0 {
            self.di.inc_x_dy(self.y - self.old_y);
        } else {
            self.di.dec_x_dy(self.y - self.old_y);
        }

        self.old_y = self.y;

        let mut s1 = self.di.dist() / self.lp.len;
        let s2 = -s1;

        if self.lp.inc < 0 {
            s1 = -s1;
        }

        let mut dist_start;
        let mut dist_pict;
        let mut dist_end;
        let mut dy;
        let mut dist;

        dist_start = self.di.dist_start();
        dist_pict = self.di.dist_pict() + self.start;
        dist_end = self.di.dist_end();
        let mut p0 = Self::MAX_HALF_WIDTH as usize + 2;
        let mut p1 = p0;

        let mut npix = 0;
        self.colors[p1].clear();
        if dist_end > 0 {
            if dist_start <= 0 {
                self.ren.pixel(&mut self.colors[p1], dist_pict, s2);
            }
            npix += 1;
        }
        p1 += 1;

        dy = 1;

        loop {
            dist = self.dist_pos[dy];
            if dist - s1 > self.width {
                break;
            }
            dist_start -= self.di.dx_start();
            dist_pict -= self.di.dx_pict();
            dist_end -= self.di.dx_end();
            self.colors[p1].clear();
            if dist_end > 0 && dist_start <= 0 {
                if self.lp.inc > 0 {
                    dist = -dist;
                }
                self.ren.pixel(&mut self.colors[p1], dist_pict, s2 - dist);
                npix += 1;
            }
            p1 += 1;
            dy += 1;
        }

        dy = 1;
        dist_start = self.di.dist_start();
        dist_pict = self.di.dist_pict() + self.start;
        dist_end = self.di.dist_end();
        loop {
            dist = self.dist_pos[dy];
            if dist + s1 > self.width {
                break;
            }
            dist_start += self.di.dx_start();
            dist_pict += self.di.dx_pict();
            dist_end += self.di.dx_end();
            p0 -= 1;
            self.colors[p0].clear();
            if dist_end > 0 && dist_start <= 0 {
                if self.lp.inc > 0 {
                    dist = -dist;
                }
                self.ren.pixel(&mut self.colors[p0], dist_pict, s2 + dist);
                npix += 1;
            }
            dy += 1;
        }
        self.ren.blend_color_vspan(
            self.x,
            self.y - dy as i32 + 1,
            (p1 - p0) as u32,
            // std::mem::size_of::<R::C>(),
            &self.colors[p0..],
        );
        self.step += 1;
        npix != 0 && self.step < self.count
    }

    pub fn step_ver(&mut self) -> bool {
        self.li.inc();
        self.y += self.lp.inc;
        self.x = (self.lp.x1 + self.li.y()) >> LineSubpixel::Shift as i32;

        if self.lp.inc > 0 {
            self.di.inc_y_dx(self.x - self.old_x);
        } else {
            self.di.dec_y_dx(self.x - self.old_x);
        }

        self.old_x = self.x;

        let mut s1 = self.di.dist() / self.lp.len;
        let s2 = -s1;

        if self.lp.inc > 0 {
            s1 = -s1;
        }

        let mut dist_start;
        let mut dist_pict;
        let mut dist_end;
        let mut dist;
        let mut dx;

        dist_start = self.di.dist_start();
        dist_pict = self.di.dist_pict() + self.start;
        dist_end = self.di.dist_end();
        let mut p0 = Self::MAX_HALF_WIDTH as usize + 2;
        let mut p1 = p0;

        let mut npix = 0;
        self.colors[p1].clear();
        if dist_end > 0 {
            if dist_start <= 0 {
                self.ren.pixel(&mut self.colors[p1], dist_pict, s2);
            }
            npix += 1;
        }
        p1 += 1;

        dx = 1;
        loop {
            dist = self.dist_pos[dx];
            if dist - s1 > self.width {
                break;
            }
            dist_start += self.di.dy_start();
            dist_pict += self.di.dy_pict();
            dist_end += self.di.dy_end();
            self.colors[p1].clear();
            if dist_end > 0 && dist_start <= 0 {
                if self.lp.inc > 0 {
                    dist = -dist;
                }
                self.ren.pixel(&mut self.colors[p1], dist_pict, s2 + dist);
                npix += 1;
            }
            p1 += 1;
            dx += 1;
        }

        dx = 1;
        dist_start = self.di.dist_start();
        dist_pict = self.di.dist_pict() + self.start;
        dist_end = self.di.dist_end();
        loop {
            dist = self.dist_pos[dx];
            if dist + s1 > self.width {
                break;
            }
            dist_start -= self.di.dy_start();
            dist_pict -= self.di.dy_pict();
            dist_end -= self.di.dy_end();
            p0 -= 1;
            self.colors[p0].clear();
            if dist_end > 0 && dist_start <= 0 {
                if self.lp.inc > 0 {
                    dist = -dist;
                }
                self.ren.pixel(&mut self.colors[p0], dist_pict, s2 - dist);
                npix += 1;
            }
            dx += 1;
        }
        self.ren.blend_color_hspan(
            self.x - dx as i32 + 1,
            self.y,
            (p1 - p0) as u32,
            &self.colors[p0..],
        );
        self.step += 1;
        return npix != 0 && self.step < self.count;
    }

    pub fn pattern_end(&self) -> i32 {
        return self.start + self.di.len();
    }

    pub fn vertical(&self) -> bool {
        return self.lp.vertical;
    }
    pub fn width(&self) -> i32 {
        return self.width;
    }
    pub fn count(&self) -> i32 {
        return self.count;
    }
}

pub struct RendererOutlineImage<'a, Ren: Renderer<C = ImgPat::ColorType>, ImgPat: ImagePattern> {
    ren: &'a mut Ren,
    pattern: ImgPat,
    start: i32,
    scale_x: f64,
    clip_box: RectI,
    clipping: bool,
}

impl<'a, Ren: Renderer<C = ImgPat::ColorType>, ImgPat: ImagePattern>
    RendererOutlineImage<'a, Ren, ImgPat>
{
    pub fn new(ren: &'a mut Ren, patt: ImgPat) -> Self {
        Self {
            ren: ren,
            pattern: patt,
            start: 0,
            scale_x: 1.0,
            clip_box: RectI {
                x1: 0,
                y1: 0,
                x2: 0,
                y2: 0,
            },
            clipping: false,
        }
    }

    pub fn attach(&mut self, ren: &'a mut Ren) {
        self.ren = ren;
    }

    pub fn ren_mut(&mut self) -> &mut Ren {
        return self.ren;
    }

    pub fn ren(&self) -> &Ren {
        return &*self.ren;
    }

    pub fn set_pattern(&mut self, p: ImgPat) {
        self.pattern = p;
    }

    pub fn pattern(&self) -> &ImgPat {
        &self.pattern
    }

    pub fn pattern_mut(&mut self) -> &mut ImgPat {
        &mut self.pattern
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

    pub fn set_scale_x(&mut self, s: f64) {
        self.scale_x = s;
    }

    pub fn set_start_x(&mut self, s: f64) {
        self.start = iround(s * LineSubpixel::Scale as i32 as f64);
    }

    pub fn width(&self) -> f64 {
        (self.subpixel_width() / LineSubpixel::Scale as i32) as f64
    }

    pub fn line3_no_clip(&mut self, lp: &LineParameters, sx: i32, sy: i32, ex: i32, ey: i32) {
        let (mut sx, mut sy) = (sx, sy);
        let (mut ex, mut ey) = (ex, ey);
        if lp.len > MAX_LENGTH {
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

        let mut li = LineIpImage::new(self, *lp, sx, sy, ex, ey, self.start, self.scale_x);

        if li.vertical() {
            while li.step_ver() {}
        } else {
            while li.step_hor() {}
        }

        self.start += uround(lp.len as f64 / self.scale_x);
    }
}

impl<'a, Ren: Renderer<C = ImgPat::ColorType>, ImgPat: ImagePattern> RendererOutline
    for RendererOutlineImage<'a, Ren, ImgPat>
{
    type C = Ren::C;

    fn pixel(&self, p: &mut Ren::C, x: i32, y: i32) {
        self.pattern.pixel(p, x, y);
    }

    fn pattern_width(&self) -> i32 {
        self.pattern.pattern_width()
    }

    fn set_color(&mut self, _c: Ren::C) {
        todo!()
    }

    fn cover(&self, _d: i32) -> i32 {
        todo!()
    }

    fn blend_solid_hspan(&mut self, _x: i32, _y: i32, _len: i32, _covers: &[u8]) {
        todo!()
    }

    fn blend_solid_vspan(&mut self, _x: i32, _y: i32, _len: i32, _covers: &[u8]) {
        todo!()
    }

    fn blend_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[Ren::C]) {
        self.ren
            .blend_color_hspan(x, y, len as i32, colors, &[], CoverScale::FULL as u8);
    }

    fn blend_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[Ren::C]) {
        self.ren
            .blend_color_vspan(x, y, len as i32, colors, &[], CoverScale::FULL as u8);
    }

    fn subpixel_width(&self) -> i32 {
        self.pattern.line_width()
    }

    fn accurate_join_only(&self) -> bool {
        true
    }

    fn semidot<Cmp>(&mut self, _cmp: Cmp, _x1: i32, _y1: i32, _x2: i32, _y2: i32) {}

    fn pie(&mut self, _x1: i32, _y1: i32, _x2: i32, _y2: i32, _x3: i32, _y3: i32) {}

    fn line0(&mut self, _params: &LineParameters) {}

    fn line1(&mut self, _params: &LineParameters, _x1: i32, _y1: i32) {}

    fn line2(&mut self, _params: &LineParameters, _x1: i32, _y1: i32) {}

    fn line3(&mut self, lp: &LineParameters, sx: i32, sy: i32, ex: i32, ey: i32) {
        let (mut sx, mut sy, mut ex, mut ey) = (sx, sy, ex, ey);

        if self.clipping {
            let mut x1 = lp.x1;
            let mut y1 = lp.y1;
            let mut x2 = lp.x2;
            let mut y2 = lp.y2;
            let flags = clip_line_segment(&mut x1, &mut y1, &mut x2, &mut y2, &self.clip_box);
            let start = self.start;
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
                        self.start += uround(
                            calc_distance(lp.x1 as f64, lp.y1 as f64, x1 as f64, y1 as f64)
                                / self.scale_x,
                        );
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
            self.start = start + uround(lp.len as f64 / self.scale_x);
        } else {
            self.line3_no_clip(lp, sx, sy, ex, ey);
        }
    }
}
