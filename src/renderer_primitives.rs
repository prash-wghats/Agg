use crate::basics::*;
use crate::dda_line::*;
use crate::ellipse_bresenham::*;
use crate::{Color, Renderer, RenderPrim};

// NOT TESTED

pub struct RendererPrimitives<'a, R: Renderer> {
    pub(super) ren: &'a mut R,
    pub(super) fill_color: R::C,
    pub(super) line_color: R::C,
    curr_x: i32,
    curr_y: i32,
}

impl<'a, R: Renderer> RenderPrim for RendererPrimitives<'a, R> {
    type ColorType = R::C;
    fn move_to(&mut self, x: i32, y: i32) {
        self.curr_x = x;
        self.curr_y = y;
    }

    fn line_to(&mut self, x: i32, y: i32, last: bool) {
        self.line(self.curr_x, self.curr_y, x, y, last);
        self.curr_x = x;
        self.curr_y = y;
    }

    fn set_fill_color(&mut self, c: R::C) {
        self.fill_color = c;
    }

    fn set_line_color(&mut self, c: R::C) {
        self.line_color = c;
    }

	fn coord(&self, c: f64) -> i32 {
        (c * LineBresenhamIp::SUBPIXEL_SCALE as f64) as i32
    }
}

impl<'a, R: Renderer> RendererPrimitives<'a, R> {
    pub fn new(ren: &'a mut R) -> Self {
        RendererPrimitives {
            ren: ren,
            fill_color: R::C::new(),
            line_color: R::C::new(),
            curr_x: 0,
            curr_y: 0,
        }
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
	
    pub fn fill_color(&self) -> R::C {
        self.fill_color
    }

    pub fn line_color(&self) -> R::C {
        self.line_color
    }

    pub fn rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.ren
            .blend_hline(x1, y1, x2 - 1, &self.line_color, CoverScale::FULL as u8);
        self.ren
            .blend_vline(x2, y1, y2 - 1, &self.line_color, CoverScale::FULL as u8);
        self.ren
            .blend_hline(x1 + 1, y2, x2, &self.line_color, CoverScale::FULL as u8);
        self.ren
            .blend_vline(x1, y1 + 1, y2, &self.line_color, CoverScale::FULL as u8);
    }

    pub fn solid_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.ren
            .blend_bar(x1, y1, x2, y2, &self.fill_color, CoverScale::FULL as u8);
    }

    pub fn outlined_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.rectangle(x1, y1, x2, y2);
        self.ren.blend_bar(
            x1 + 1,
            y1 + 1,
            x2 - 1,
            y2 - 1,
            &self.fill_color,
            CoverScale::FULL as u8,
        );
    }

    pub fn ellipse(&mut self, x: i32, y: i32, rx: i32, ry: i32) {
        let mut ei = EllipseBresenhamIp::new(rx, ry);
        let mut dx = 0;
        let mut dy = -ry;
        loop {
            dx += ei.dx();
            dy += ei.dy();
            self.ren
                .blend_pixel(x + dx, y + dy, &self.line_color, CoverScale::FULL as u8);
            self.ren
                .blend_pixel(x + dx, y - dy, &self.line_color, CoverScale::FULL as u8);
            self.ren
                .blend_pixel(x - dx, y - dy, &self.line_color, CoverScale::FULL as u8);
            self.ren
                .blend_pixel(x - dx, y + dy, &self.line_color, CoverScale::FULL as u8);
            ei.inc();
            if dy >= 0 {
                break;
            }
        }
    }

    pub fn solid_ellipse(&mut self, x: i32, y: i32, rx: i32, ry: i32) {
        let mut ei = EllipseBresenhamIp::new(rx, ry);
        let mut dx = 0;
        let mut dy = -ry;
        let mut dy0 = dy;
        let mut dx0 = dx;

        loop {
            dx += ei.dx();
            dy += ei.dy();

            if dy != dy0 {
                self.ren.blend_hline(
                    x - dx0,
                    y + dy0,
                    x + dx0,
                    &self.fill_color,
                    CoverScale::FULL as u8,
                );
                self.ren.blend_hline(
                    x - dx0,
                    y - dy0,
                    x + dx0,
                    &self.fill_color,
                    CoverScale::FULL as u8,
                );
            }
            dx0 = dx;
            dy0 = dy;
            ei.inc();
            if dy >= 0 {
                break;
            }
        }
        self.ren.blend_hline(
            x - dx0,
            y + dy0,
            x + dx0,
            &self.fill_color,
            CoverScale::FULL as u8,
        );
    }

    pub fn outlined_ellipse(&mut self, x: i32, y: i32, rx: i32, ry: i32) {
        let mut ei = EllipseBresenhamIp::new(rx, ry);
        let mut dx = 0;
        let mut dy = -ry;

        loop {
            dx += ei.dx();
            dy += ei.dy();

            self.ren
                .blend_pixel(x + dx, y + dy, &self.line_color, CoverScale::FULL as u8);
            self.ren
                .blend_pixel(x + dx, y - dy, &self.line_color, CoverScale::FULL as u8);
            self.ren
                .blend_pixel(x - dx, y - dy, &self.line_color, CoverScale::FULL as u8);
            self.ren
                .blend_pixel(x - dx, y + dy, &self.line_color, CoverScale::FULL as u8);

            if ei.dy() != 0 && dx != 0 {
                self.ren.blend_hline(
                    x - dx + 1,
                    y + dy,
                    x + dx - 1,
                    &self.fill_color,
                    CoverScale::FULL as u8,
                );
                self.ren.blend_hline(
                    x - dx + 1,
                    y - dy,
                    x + dx - 1,
                    &self.fill_color,
                    CoverScale::FULL as u8,
                );
            }
            ei.inc();
            if dy >= 0 {
                break;
            }
        }
    }

    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, last: bool) {
        let mut li = LineBresenhamIp::new(x1, y1, x2, y2);

        let mut len = li.get_len();
        if len == 0 {
            if last {
                self.ren.blend_pixel(
                    LineBresenhamIp::line_lr(x1),
                    LineBresenhamIp::line_lr(y1),
                    &self.line_color,
                    CoverScale::FULL as u8,
                );
            }
            return;
        }

        if last {
            len += 1;
        }

        if li.is_ver() {
            for _ in 0..len {
                self.ren
                    .blend_pixel(li.x2(), li.y1(), &self.line_color, CoverScale::FULL as u8);
                li.vstep();
            }
        } else {
            for _ in 0..len {
                self.ren
                    .blend_pixel(li.x1(), li.y2(), &self.line_color, CoverScale::FULL as u8);
                li.hstep();
            }
        }
    }

}
