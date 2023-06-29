
use crate::basics::*;
use crate::ellipse_bresenham::EllipseBresenhamIp;
use crate::renderer_primitives::{RendererPrimitives};
use crate::{Renderer, AggInteger, RenderPrim};



// NOT TESTED

//---------------------------------------------------------------MarkerType
#[repr(i32)]
pub enum MarkerType {
    Square,
    Diamond,
    Circle,
    CrossedCircle,
    SemiellipseLeft,
    SemiellipseRight,
    SemiellipseUp,
    SemiellipseDown,
    TriangleLeft,
    TriangleRight,
    TriangleUp,
    TriangleDown,
    FourRays,
    Cross,
    X,
    Dash,
    Dot,
    Pixel,
	//EndOfMarkers,
    //end_of_markers,
}

pub type RendererMarkers<'a, T> = RendererPrimitives<'a, T>;
//--------------------------------------------------------RendererMarkers
/*pub struct RendererMarkers<'a, R: 'a> {
    base: RendererPrimitives<'a, R>,
}*/

//impl<'a, R: RendererBase> RendererMarkers<'a, R> {
impl<'a, R: Renderer> RendererPrimitives<'a, R> {
    /*pub fn new(rbuf: &'a mut R) -> Self {
        RendererMarkers {
            base: RendererPrimitives::new(rbuf),
        }
    }*/


    pub fn visible(&self, x: i32, y: i32, r: i32) -> bool {
        let mut rc = RectI::new(x - r, y - r, x + y, y + r);
        rc.clip(self.ren.bounding_clip_box())
    }


    pub fn square(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                self.outlined_rectangle(x - r, y - r, x + r, y + r);
            } else {
                self
                    .ren
                    .blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    pub fn diamond(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                let mut dy = -r;
                let mut dx = 0;
                loop {
                    self
                        .ren
                        .blend_pixel(x - dx, y + dy, &self.line_color, CoverScale::FULL as u8);
                    self
                        .ren
                        .blend_pixel(x + dx, y + dy, &self.line_color, CoverScale::FULL as u8);
                    self
                        .ren
                        .blend_pixel(x - dx, y - dy, &self.line_color, CoverScale::FULL as u8);
                    self
                        .ren
                        .blend_pixel(x + dx, y - dy, &self.line_color, CoverScale::FULL as u8);

                    if dx != 0 {
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
                    dy += 1;
                    dx += 1;
                    if dy > 0 {
                        break;
                    }
                }
            } else {
                self
                    .ren
                    .blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    pub fn circle(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                self.outlined_ellipse(x, y, r, r);
            } else {
                self
                    .ren
                    .blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    pub fn crossed_circle(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                self.outlined_ellipse(x, y, r, r);
                let mut r6 = r + (r >> 1);
                if r <= 2 {
                    r6 += 1;
                }
                let r = r >> 1;
                self
                    .ren
                    .blend_hline(x - r6, y, x - r, &self.line_color, CoverScale::FULL as u8);
                self
                    .ren
                    .blend_hline(x + r, y, x + r6, &self.line_color, CoverScale::FULL as u8);
                self
                    .ren
                    .blend_vline(x, y - r6, y - r, &self.line_color, CoverScale::FULL as u8);
                self
                    .ren
                    .blend_vline(x, y + r, y + r6, &self.line_color, CoverScale::FULL as u8);
            } else {
                self
                    .ren
                    .blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }

    pub fn semiellipse_left(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                let r8 = r * 4 / 5;
                let mut dy = -r;
                let mut dx = 0;
                let mut ei = EllipseBresenhamIp::new(r * 3 / 5, r + r8);
                loop {
                    dx += ei.dx();
                    dy += ei.dy();

                    self
                        .ren
                        .blend_pixel(x + dy, y + dx, &self.line_color, CoverScale::FULL as u8);
                    self
                        .ren
                        .blend_pixel(x + dy, y - dx, &self.line_color, CoverScale::FULL as u8);

                    if ei.dy() != 0 && dx != 0 {
                        self.ren.blend_vline(
                            x + dy,
                            y - dx + 1,
                            y + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                    }
                    ei.inc();
                    if dy >= r8 {
                        break;
                    }
                }
                self.ren.blend_vline(
                    x + dy,
                    y - dx,
                    y + dx,
                    &self.line_color,
                    CoverScale::FULL as u8,
                );
            } else {
                self
                    .ren
                    .blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    pub fn semiellipse_right(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                let r8 = r * 4 / 5;
                let mut dy = -r;
                let mut dx = 0;
                let mut ei = EllipseBresenhamIp::new(r * 3 / 5, r + r8);
                loop {
                    dx += ei.dx();
                    dy += ei.dy();

                    self
                        .ren
                        .blend_pixel(x - dy, y + dx, &self.line_color, CoverScale::FULL as u8);
                    self
                        .ren
                        .blend_pixel(x - dy, y - dx, &self.line_color, CoverScale::FULL as u8);

                    if ei.dy() != 0 && dx != 0 {
                        self.ren.blend_vline(
                            x - dy,
                            y - dx + 1,
                            y + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                    }
                    ei.inc();
                    if dy >= r8 {
                        break;
                    }
                }
                self.ren.blend_vline(
                    x - dy,
                    y - dx,
                    y + dx,
                    &self.line_color,
                    CoverScale::FULL as u8,
                );
            } else {
                self
                    .ren
                    .blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn semiellipse_up(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                let r8 = r * 4 / 5;
                let mut dy = -r;
                let mut dx = 0;
                let mut ei = EllipseBresenhamIp::new(r * 3 / 5, r + r8);
                loop {
                    dx += ei.dx();
                    dy += ei.dy();

                    self.ren
                        .blend_pixel(x + dx, y - dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x - dx, y - dy, &self.line_color, CoverScale::FULL as u8);

                    if ei.dy() != 0 && dx != 0 {
                        self.ren.blend_hline(
                            x - dx + 1,
                            y - dy,
                            x + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                    }
                    ei.inc();
                    if dy >= r8 {
                        break;
                    }
                }
                self.ren
                    .blend_hline(x - dx, y - dy - 1, x + dx, &self.line_color, CoverScale::FULL as u8);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn semiellipse_down(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                let r8 = r * 4 / 5;
                let mut dy = -r;
                let mut dx = 0;
                let mut ei = EllipseBresenhamIp::new(r * 3 / 5, r + r8);
                loop {
                    dx += ei.dx();
                    dy += ei.dy();

                    self.ren
                        .blend_pixel(x + dx, y + dy, &self.line_color, CoverScale::FULL as u8);
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
                    }
                    ei.inc();
                    if dy >= r8 {
                        break;
                    }
                }
                self.ren
                    .blend_hline(x - dx, y + dy + 1, x + dx, &self.line_color, CoverScale::FULL as u8);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn triangle_left(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                let mut dy = -r;
                let mut dx = 0;
                let mut flip = 0;
                let r6 = r * 3 / 5;
                loop {
                    self.ren
                        .blend_pixel(x + dy, y - dx, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x + dy, y + dx, &self.line_color, CoverScale::FULL as u8);

                    if dx != 0 {
                        self.ren.blend_vline(
                            x + dy,
                            y - dx + 1,
                            y + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                    }
                    dy += 1;
                    dx += flip;
                    flip ^= 1;
                    if dy >= r6 {
                        break;
                    }
                }
                self.ren
                    .blend_vline(x + dy, y - dx, y + dx, &self.line_color, CoverScale::FULL as u8);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn triangle_right(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                let mut dy = -r;
                let mut dx = 0;
                let mut flip = 0;
                let r6 = r * 3 / 5;
                loop {
                    self.ren
                        .blend_pixel(x - dy, y - dx, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x - dy, y + dx, &self.line_color, CoverScale::FULL as u8);

                    if dx != 0 {
                        self.ren.blend_vline(
                            x - dy,
                            y - dx + 1,
                            y + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                    }
                    dy += 1;
                    dx += flip;
                    flip ^= 1;
                    if dy >= r6 {
                        break;
                    }
                }
                self.ren
                    .blend_vline(x - dy, y - dx, y + dx, &self.line_color, CoverScale::FULL as u8);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn triangle_up(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r != 0 {
                let mut dy = -r;
                let mut dx = 0;
                let mut flip = 0;
                let r6 = r * 3 / 5;
                loop {
                    self.ren
                        .blend_pixel(x - dx, y - dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x + dx, y - dy, &self.line_color, CoverScale::FULL as u8);

                    if dx != 0 {
                        self.ren.blend_hline(
                            x - dx + 1,
                            y - dy,
                            x + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                    }
                    dy += 1;
                    dx += flip;
                    flip ^= 1;
                    if dy >= r6 {
                        break;
                    }
                }
                self.ren
                    .blend_hline(x - dx, y - dy, x + dx, &self.line_color, CoverScale::FULL as u8);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn triangle_down(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r > 0 {
                let mut dy = -r;
                let mut dx = 0;
                let mut flip = 0;
                let r6 = r * 3 / 5;
                loop {
                    self.ren
                        .blend_pixel(x - dx, y + dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x + dx, y + dy, &self.line_color, CoverScale::FULL as u8);

                    if dx > 0 {
                        self.ren.blend_hline(
                            x - dx + 1,
                            y + dy,
                            x + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                    }
                    dy += 1;
                    dx += flip;
                    flip ^= 1;
                    if dy >= r6 {
                        break;
                    }
                }
                self.ren
                    .blend_hline(x - dx, y + dy, x + dx, &self.line_color, CoverScale::FULL as u8);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn four_rays(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r > 0 {
                let mut dy = -r;
                let mut dx = 0;
                let mut flip = 0;
                let r3 = -(r / 3);
                loop {
                    self.ren
                        .blend_pixel(x - dx, y + dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x + dx, y + dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x - dx, y - dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x + dx, y - dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x + dy, y - dx, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x + dy, y + dx, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x - dy, y - dx, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x - dy, y + dx, &self.line_color, CoverScale::FULL as u8);

                    if dx > 0 {
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
                        self.ren.blend_vline(
                            x + dy,
                            y - dx + 1,
                            y + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                        self.ren.blend_vline(
                            x - dy,
                            y - dx + 1,
                            y + dx - 1,
                            &self.fill_color,
                            CoverScale::FULL as u8,
                        );
                    }
                    dy += 1;
                    dx += flip;
                    flip ^= 1;
                    if dy > r3 {
                        break;
                    }
                }
                self.solid_rectangle(x + r3 + 1, y + r3 + 1, x - r3 - 1, y - r3 - 1);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn cross(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r > 0 {
                self.ren
                    .blend_vline(x, y - r, y + r, &self.line_color, CoverScale::FULL as u8);
                self.ren
                    .blend_hline(x - r, y, x + r, &self.line_color, CoverScale::FULL as u8);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn xing(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r > 0 {
                let mut dy = -r * 7 / 10;
                loop {
                    self.ren
                        .blend_pixel(x + dy, y + dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x - dy, y + dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x + dy, y - dy, &self.line_color, CoverScale::FULL as u8);
                    self.ren
                        .blend_pixel(x - dy, y - dy, &self.line_color, CoverScale::FULL as u8);
                    dy += 1;
                    if dy >= 0 {
                        break;
                    }
                }
            }
            self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
        }
    }


    fn dash(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r > 0 {
                self.ren
                    .blend_hline(x - r, y, x + r, &self.line_color, CoverScale::FULL as u8);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn dot(&mut self, x: i32, y: i32, r: i32) {
        if self.visible(x, y, r) {
            if r > 0 {
                self.solid_ellipse(x, y, r, r);
            } else {
                self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
            }
        }
    }


    fn pixel(&mut self, x: i32, y: i32, _: i32) {
        self.ren.blend_pixel(x, y, &self.fill_color, CoverScale::FULL as u8);
    }


    pub fn marker(&mut self, x: i32, y: i32, r: i32, type_: MarkerType) {
        match type_ {
            MarkerType::Square => self.square(x, y, r),
            MarkerType::Diamond => self.diamond(x, y, r),
            MarkerType::Circle => self.circle(x, y, r),
            MarkerType::CrossedCircle => self.crossed_circle(x, y, r),
            MarkerType::SemiellipseLeft => self.semiellipse_left(x, y, r),
            MarkerType::SemiellipseRight => self.semiellipse_right(x, y, r),
            MarkerType::SemiellipseUp => self.semiellipse_up(x, y, r),
            MarkerType::SemiellipseDown => self.semiellipse_down(x, y, r),
            MarkerType::TriangleLeft => self.triangle_left(x, y, r),
            MarkerType::TriangleRight => self.triangle_right(x, y, r),
            MarkerType::TriangleUp => self.triangle_up(x, y, r),
            MarkerType::TriangleDown => self.triangle_down(x, y, r),
            MarkerType::FourRays => self.four_rays(x, y, r),
            MarkerType::Cross => self.cross(x, y, r),
            MarkerType::X => self.xing(x, y, r),
            MarkerType::Dash => self.dash(x, y, r),
            MarkerType::Dot => self.dot(x, y, r),
            MarkerType::Pixel => self.pixel(x, y, r),
        }
    }


    pub fn markers<T: AggInteger>(&mut self, n: i32, x: &[T], y: &[T], r: T, type_: MarkerType) {
        if n <= 0 {
            return;
        }
        if r.into_u32() == 0 {
            for i in 0..n as usize as usize {
                self.ren
                    .blend_pixel(x[i].into_i32(), y[i].into_i32(), &self.fill_color, CoverScale::FULL as u8);
            }
            return;
        }

        match type_ {
            MarkerType::Square => {
                for i in 0..n as usize {
                    self.square(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::Diamond => {
                for i in 0..n as usize {
                    self.diamond(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::Circle => {
                for i in 0..n as usize {
                    self.circle(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::CrossedCircle => {
                for i in 0..n as usize {
                    self.crossed_circle(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::SemiellipseLeft => {
                for i in 0..n as usize {
                    self.semiellipse_left(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::SemiellipseRight => {
                for i in 0..n as usize {
                    self.semiellipse_right(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::SemiellipseUp => {
                for i in 0..n as usize {
                    self.semiellipse_up(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::SemiellipseDown => {
                for i in 0..n as usize {
                    self.semiellipse_down(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::TriangleLeft => {
                for i in 0..n as usize {
                    self.triangle_left(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::TriangleRight => {
                for i in 0..n as usize {
                    self.triangle_right(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::TriangleUp => {
                for i in 0..n as usize {
                    self.triangle_up(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::TriangleDown => {
                for i in 0..n as usize {
                    self.triangle_down(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::FourRays => {
                for i in 0..n as usize {
                    self.four_rays(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::Cross => {
                for i in 0..n as usize {
                    self.cross(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::X => {
                for i in 0..n as usize {
                    self.xing(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::Dash => {
                for i in 0..n as usize {
                    self.dash(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::Dot => {
                for i in 0..n as usize {
                    self.dot(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
            MarkerType::Pixel => {
                for i in 0..n as usize {
                    self.pixel(x[i].into_i32(), y[i].into_i32(), r.into_i32());
                }
            }
        }
    }

    pub fn markers_array(&mut self, n: i32, x: &[i32], y: &[i32], r: &[i32], type_: MarkerType) {
        if n <= 0 {
            return;
        }
        match type_ {
            MarkerType::Square => {
                for i in 0..n as usize {
                    self.square(x[i], y[i], r[i]);
                }
            }
            MarkerType::Diamond => {
                for i in 0..n as usize {
                    self.diamond(x[i], y[i], r[i]);
                }
            }
            MarkerType::Circle => {
                for i in 0..n as usize {
                    self.circle(x[i], y[i], r[i]);
                }
            }
            MarkerType::CrossedCircle => {
                for i in 0..n as usize {
                    self.crossed_circle(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseLeft => {
                for i in 0..n as usize {
                    self.semiellipse_left(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseRight => {
                for i in 0..n as usize {
                    self.semiellipse_right(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseUp => {
                for i in 0..n as usize {
                    self.semiellipse_up(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseDown => {
                for i in 0..n as usize {
                    self.semiellipse_down(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleLeft => {
                for i in 0..n as usize {
                    self.triangle_left(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleRight => {
                for i in 0..n as usize {
                    self.triangle_right(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleUp => {
                for i in 0..n as usize {
                    self.triangle_up(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleDown => {
                for i in 0..n as usize {
                    self.triangle_down(x[i], y[i], r[i]);
                }
            }
            MarkerType::FourRays => {
                for i in 0..n as usize {
                    self.four_rays(x[i], y[i], r[i]);
                }
            }
            MarkerType::Cross => {
                for i in 0..n as usize {
                    self.cross(x[i], y[i], r[i]);
                }
            }
            MarkerType::X => {
                for i in 0..n as usize {
                    self.xing(x[i], y[i], r[i]);
                }
            }
            MarkerType::Dash => {
                for i in 0..n as usize {
                    self.dash(x[i], y[i], r[i]);
                }
            }
            MarkerType::Dot => {
                for i in 0..n as usize {
                    self.dot(x[i], y[i], r[i]);
                }
            }
            MarkerType::Pixel => {
                for i in 0..n as usize {
                    self.pixel(x[i], y[i], r[i]);
                }
            }
        }
    }


    pub fn markers_with_fill(
        &mut self, n: i32, x: &[i32], y: &[i32], r: &[i32], fc: &[R::C], type_: MarkerType,
    ) {
        if n <= 0 {
            return;
        }
        match type_ {
            MarkerType::Square => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.square(x[i], y[i], r[i]);
                }
            }
            MarkerType::Diamond => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.diamond(x[i], y[i], r[i]);
                }
            }
            MarkerType::Circle => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.circle(x[i], y[i], r[i]);
                }
            }
            MarkerType::CrossedCircle => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.crossed_circle(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseLeft => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.semiellipse_left(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseRight => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.semiellipse_right(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseUp => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.semiellipse_up(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseDown => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.semiellipse_down(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleLeft => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.triangle_left(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleRight => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.triangle_right(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleUp => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.triangle_up(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleDown => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.triangle_down(x[i], y[i], r[i]);
                }
            }
            MarkerType::FourRays => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.four_rays(x[i], y[i], r[i]);
                }
            }
            MarkerType::Cross => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.cross(x[i], y[i], r[i]);
                }
            }
            MarkerType::X => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.xing(x[i], y[i], r[i]);
                }
            }
            MarkerType::Dash => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.dash(x[i], y[i], r[i]);
                }
            }
            MarkerType::Dot => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.dot(x[i], y[i], r[i]);
                }
            }
            MarkerType::Pixel => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.pixel(x[i], y[i], r[i]);
                }
            }
        }
    }


    pub fn markers_with_fill_line(
        &mut self, n: i32, x: &[i32], y: &[i32], r: &[i32], fc: &[R::C], lc: &[R::C],
        type_: MarkerType,
    ) {
        if n <= 0 {
            return;
        }
        match type_ {
            MarkerType::Square => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.square(x[i], y[i], r[i]);
                }
            }
            MarkerType::Diamond => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.diamond(x[i], y[i], r[i]);
                }
            }
            MarkerType::Circle => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.circle(x[i], y[i], r[i]);
                }
            }
            MarkerType::CrossedCircle => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.crossed_circle(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseLeft => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.semiellipse_left(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseRight => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.semiellipse_right(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseUp => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.semiellipse_up(x[i], y[i], r[i]);
                }
            }
            MarkerType::SemiellipseDown => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.semiellipse_down(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleLeft => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.triangle_left(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleRight => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.triangle_right(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleUp => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.triangle_up(x[i], y[i], r[i]);
                }
            }
            MarkerType::TriangleDown => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.triangle_down(x[i], y[i], r[i]);
                }
            }
            MarkerType::FourRays => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.four_rays(x[i], y[i], r[i]);
                }
            }
            MarkerType::Cross => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.cross(x[i], y[i], r[i]);
                }
            }
            MarkerType::X => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.xing(x[i], y[i], r[i]);
                }
            }
            MarkerType::Dash => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.dash(x[i], y[i], r[i]);
                }
            }
            MarkerType::Dot => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.dot(x[i], y[i], r[i]);
                }
            }
            MarkerType::Pixel => {
                for i in 0..n as usize {
                    self.set_fill_color(fc[i]);
                    self.set_line_color(lc[i]);
                    self.pixel(x[i], y[i], r[i]);
                }
            }
        }
    }
}
