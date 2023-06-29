//----------------------------------------------------------------------------
// Anti-Grain Geometry - Version 2.4
// Copyright (C) 2002-2005 Maxim Shemanarev (http://www.antigrain.com)
//
// Permission to copy, use, modify, sell and distribute this software
// is granted provided this copyright notice appears in all copies.
// This software is provided "as is" without express or implied
// warranty, and with no claim as to its suitability for any purpose.
//
//----------------------------------------------------------------------------
// Contact: mcseem@antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------
//
// class RendererBase
//
//----------------------------------------------------------------------------

use crate::basics::RectI;
use crate::{Color, PixFmt, RenderBuffer, Renderer, Equiv};

pub struct RendererBase<'a, Pix: PixFmt> {
    ren: Equiv<'a, Pix>,
    clip_box: RectI,
}

impl<'a, Pix: PixFmt> RendererBase<'a, Pix> {
    pub fn new_borrowed(ren: &'a mut Pix) -> Self {
        let (w, h) = (ren.width(), ren.height());
        Self {
            ren: Equiv::Brw(ren),
            clip_box: RectI {
                x1: 0,
                y1: 0,
                x2: w as i32 - 1,
                y2: h as i32 - 1,
            },
        }
    }

	pub fn new_owned(ren: Pix) -> Self {
        let (w, h) = (ren.width(), ren.height());
        Self {
            ren: Equiv::Own(ren),
            clip_box: RectI {
                x1: 0,
                y1: 0,
                x2: w as i32 - 1,
                y2: h as i32 - 1,
            },
        }
    }

    pub fn attach_borrowed(&mut self, ren: &'a mut Pix) {
        let (w, h) = (ren.width(), ren.height());
        self.ren = Equiv::Brw(ren);
        self.clip_box = RectI {
            x1: 0,
            y1: 0,
            x2: w as i32 - 1,
            y2: h as i32 - 1,
        }
    }

	pub fn attach_owned(&mut self, ren: Pix) {
        let (w, h) = (ren.width(), ren.height());
        self.ren = Equiv::Own(ren);
        self.clip_box = RectI {
            x1: 0,
            y1: 0,
            x2: w as i32 - 1,
            y2: h as i32 - 1,
        }
    }

    pub fn ren_mut(&mut self) -> &mut Pix {
        return &mut self.ren;
    }

    pub fn ren(&self) -> &Pix {
        return &self.ren;
    }

    pub fn width(&self) -> u32 {
        return self.ren.width();
    }
    pub fn height(&self) -> u32 {
        return self.ren.height();
    }

    pub fn set_clip_box(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        let mut cb = RectI {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        };
        cb.normalize();
        if cb.clip(&RectI {
            x1: 0,
            y1: 0,
            x2: self.ren.width() as i32 - 1,
            y2: self.ren.height() as i32 - 1,
        }) {
            self.clip_box = cb;
            return true;
        }
        self.clip_box.x1 = 1;
        self.clip_box.y1 = 1;
        self.clip_box.x2 = 0;
        self.clip_box.y2 = 0;
        return false;
    }

    pub fn clip_box_naked(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.clip_box.x1 = x1;
        self.clip_box.y1 = y1;
        self.clip_box.x2 = x2;
        self.clip_box.y2 = y2;
    }

    pub fn inbox(&self, x: i32, y: i32) -> bool {
        return x >= self.clip_box.x1
            && y >= self.clip_box.y1
            && x <= self.clip_box.x2
            && y <= self.clip_box.y2;
    }

    pub fn clear(&mut self, c: &Pix::C) {
        let w = self.width();
        if w > 0 {
            for y in 0..self.height() as i32 {
                self.ren.copy_hline(0, y, w, c);
            }
        }
    }

    pub fn copy_bar(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: &Pix::C) {
        let mut rc = RectI::new(x1, y1, x2, y2);
        rc.normalize();
        if rc.clip(self.clip_box()) {
            for y in rc.y1..=rc.y2 {
                self.ren.copy_hline(rc.x1, y, (rc.x2 - rc.x1 + 1) as u32, c);
            }
        }
    }

    pub fn blend_from<R: PixFmt<T = Pix::T>>(
        &mut self, src: &R, rect_src_ptr: Option<&RectI>, dx: i32, dy: i32, cover: u32,
    ) {
        let mut rsrc = RectI::new(0, 0, src.width() as i32, src.height() as i32);
        if let Some(rect_src_ptr) = rect_src_ptr {
            rsrc.x1 = rect_src_ptr.x1;
            rsrc.y1 = rect_src_ptr.y1;
            rsrc.x2 = rect_src_ptr.x2 + 1;
            rsrc.y2 = rect_src_ptr.y2 + 1;
        }

        // Version with xdst, ydst (absolute positioning)
        //let rdist = RectI::new(xdst, ydst, xdst + rsrc.x2 - rsrc.x1, ydst + rsrc.y2 - rsrc.y1);

        // Version with dx, dy (relative positioning)
        let mut rdst = RectI::new(rsrc.x1 + dx, rsrc.y1 + dy, rsrc.x2 + dx, rsrc.y2 + dy);
        let mut rc = self.clip_rect_area(
            &mut rdst,
            &mut rsrc,
            src.width() as i32,
            src.height() as i32,
        );

        if rc.x2 > 0 {
            let mut incy = 1;
            if rdst.y1 > rsrc.y1 {
                rsrc.y1 += rc.y2 - 1;
                rdst.y1 += rc.y2 - 1;
                incy = -1;
            }
            while rc.y2 > 0 {
                let rw = src.row_data(rsrc.y1);
                if !rw.ptr.is_null() {
                    let mut x1src = rsrc.x1;
                    let mut x1dst = rdst.x1;
                    let mut len = rc.x2;
                    if rw.x1 > x1src {
                        x1dst += rw.x1 - x1src;
                        len -= rw.x1 - x1src;
                        x1src = rw.x1;
                    }
                    if len > 0 {
                        if x1src + len - 1 > rw.x2 {
                            len -= x1src + len - rw.x2 - 1;
                        }
                        if len > 0 {
                            self.ren
                                .blend_from(src, x1dst, rdst.y1, x1src, rsrc.y1, len as u32, cover);
                        }
                    }
                }
                rdst.y1 += incy;
                rsrc.y1 += incy;
                rc.y2 -= 1;
            }
        }
    }

    pub fn clip_rect_area(&self, dst: &mut RectI, src: &mut RectI, wsrc: i32, hsrc: i32) -> RectI {
        let mut rc = RectI::new(0, 0, 0, 0);
        let mut cb = *self.clip_box();
        cb.x2 += 1;
        cb.y2 += 1;

        if src.x1 < 0 {
            dst.x1 -= src.x1;
            src.x1 = 0;
        }
        if src.y1 < 0 {
            dst.y1 -= src.y1;
            src.y1 = 0;
        }

        if src.x2 > wsrc {
            src.x2 = wsrc;
        }
        if src.y2 > hsrc {
            src.y2 = hsrc;
        }

        if dst.x1 < cb.x1 {
            src.x1 += cb.x1 - dst.x1;
            dst.x1 = cb.x1;
        }
        if dst.y1 < cb.y1 {
            src.y1 += cb.y1 - dst.y1;
            dst.y1 = cb.y1;
        }

        if dst.x2 > cb.x2 {
            dst.x2 = cb.x2;
        }
        if dst.y2 > cb.y2 {
            dst.y2 = cb.y2;
        }

        rc.x2 = dst.x2 - dst.x1;
        rc.y2 = dst.y2 - dst.y1;

        if rc.x2 > src.x2 - src.x1 {
            rc.x2 = src.x2 - src.x1;
        }
        if rc.y2 > src.y2 - src.y1 {
            rc.y2 = src.y2 - src.y1;
        }
        return rc;
    }

    pub fn clip_box(&self) -> &RectI {
        return &self.clip_box;
    }
    pub fn xmin(&self) -> i32 {
        return self.clip_box.x1;
    }
    pub fn ymin(&self) -> i32 {
        return self.clip_box.y1;
    }
    pub fn xmax(&self) -> i32 {
        return self.clip_box.x2;
    }
    pub fn ymax(&self) -> i32 {
        return self.clip_box.y2;
    }

    pub fn bounding_xmin(&self) -> i32 {
        return self.clip_box.x1;
    }
    pub fn bounding_ymin(&self) -> i32 {
        return self.clip_box.y1;
    }
    pub fn bounding_xmax(&self) -> i32 {
        return self.clip_box.x2;
    }
    pub fn bounding_ymax(&self) -> i32 {
        return self.clip_box.y2;
    }

    pub fn reset_clipping(&mut self, visibility: bool) {
        if visibility {
            self.clip_box.x1 = 0;
            self.clip_box.y1 = 0;
            self.clip_box.x2 = self.width() as i32 - 1;
            self.clip_box.y2 = self.height() as i32 - 1;
        } else {
            self.clip_box.x1 = 1;
            self.clip_box.y1 = 1;
            self.clip_box.x2 = 0;
            self.clip_box.y2 = 0;
        }
    }
}

impl<'a, Pix: PixFmt> Renderer for RendererBase<'a, Pix> {
    type C = Pix::C;

    /*fn copy_pixel(&self, x: i32, y: i32, c: &color_type)
    {
        if(self.inbox(x, y))
        {
            self.ren.copy_pixel(x, y, c);
        }
    }*/

    fn bounding_clip_box(&self) -> &RectI {
        return &self.clip_box;
    }

    fn blend_pixel(&mut self, x: i32, y: i32, c: &Self::C, cover: u8) {
        if self.inbox(x, y) {
            self.ren.blend_pixel(x, y, c, cover);
        }
    }

    fn pixel(&mut self, x: i32, y: i32) -> Self::C {
        return if self.inbox(x, y) {
            self.ren.pixel(x, y)
        } else {
            <Self::C as Color>::no_color()
        };
    }

    fn copy_vline(&mut self, x: i32, y1: i32, y2: i32, c: &Self::C) {
        let mut y1 = y1;
        let mut y2 = y2;
        if y1 > y2 {
            let t = y2;
            y2 = y1;
            y1 = t;
        }
        if x > self.xmax() {
            return;
        }
        if x < self.xmin() {
            return;
        }
        if y1 > self.ymax() {
            return;
        }
        if y2 < self.ymin() {
            return;
        }

        if y1 < self.ymin() {
            y1 = self.ymin();
        }
        if y2 > self.ymax() {
            y2 = self.ymax();
        }

        self.ren.copy_vline(x, y1, (y2 - y1 + 1) as u32, c);
    }

    fn copy_hline(&mut self, x1: i32, y: i32, x2: i32, c: &Pix::C) {
        let mut x1 = x1;
        let mut x2 = x2;
        if x1 > x2 {
            let t = x2;
            x2 = x1;
            x1 = t;
        }
        if y > self.ymax() {
            return;
        }
        if y < self.ymin() {
            return;
        }
        if x1 > self.xmax() {
            return;
        }
        if x2 < self.xmin() {
            return;
        }

        if x1 < self.xmin() {
            x1 = self.xmin();
        }
        if x2 > self.xmax() {
            x2 = self.xmax();
        }

        self.ren.copy_hline(x1, y, (x2 - x1 + 1) as u32, c);
    }

    fn blend_hline(&mut self, x1: i32, y: i32, x2: i32, c: &Self::C, cover: u8) {
        let mut x1 = x1;
        let mut x2 = x2;

        if x1 > x2 {
            let t = x2;
            x2 = x1;
            x1 = t;
        }
        if y > self.ymax() {
            return;
        }
        if y < self.ymin() {
            return;
        }
        if x1 > self.xmax() {
            return;
        }
        if x2 < self.xmin() {
            return;
        }

        if x1 < self.xmin() {
            x1 = self.xmin();
        }
        if x2 > self.xmax() {
            x2 = self.xmax();
        }

        self.ren.blend_hline(x1, y, (x2 - x1 + 1) as u32, c, cover);
    }

    fn blend_vline(&mut self, x: i32, y1: i32, y2: i32, c: &Self::C, cover: u8) {
        let mut y1 = y1;
        let mut y2 = y2;

        if y1 > y2 {
            let t = y2;
            y2 = y1;
            y1 = t;
        }
        if x > self.xmax() {
            return;
        }
        if x < self.xmin() {
            return;
        }
        if y1 > self.ymax() {
            return;
        }
        if y2 < self.ymin() {
            return;
        }

        if y1 < self.ymin() {
            y1 = self.ymin();
        }
        if y2 > self.ymax() {
            y2 = self.ymax();
        }

        self.ren.blend_vline(x, y1, (y2 - y1 + 1) as u32, c, cover);
    }

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: i32, c: &Self::C, covers: &[u8]) {
        let mut x = x;
        let mut len = len;
        let mut co = covers;

        if y > self.ymax() {
            return;
        }
        if y < self.ymin() {
            return;
        }

        if x < self.xmin() {
            len -= self.xmin() - x;
            if len <= 0 {
                return;
            }
            //covers += self.xmin() - x;
            co = &covers[(self.xmin() - x) as usize..];
            x = self.xmin();
        }
        if x + len > self.xmax() {
            len = self.xmax() - x + 1;
            if len <= 0 {
                return;
            }
        }
        self.ren.blend_solid_hspan(x, y, len as u32, c, co);
    }

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: i32, c: &Self::C, covers: &[u8]) {
        let mut y = y;
        let mut len = len;
        let mut co = covers;

        if x > self.xmax() {
            return;
        }
        if x < self.xmin() {
            return;
        }

        if y < self.ymin() {
            len -= self.ymin() - y;
            if len <= 0 {
                return;
            }
            //covers += self.ymin() - y;
            co = &covers[(self.ymin() - y) as usize..];
            y = self.ymin();
        }
        if y + len > self.ymax() {
            len = self.ymax() - y + 1;
            if len <= 0 {
                return;
            }
        }
        self.ren.blend_solid_vspan(x, y, len as u32, c, co);
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: i32, colors: &[Self::C], covers: &[u8], cover: u8,
    ) {
        if y > self.ymax() {
            return;
        }
        if y < self.ymin() {
            return;
        }
        let mut covers = covers;
        let mut colors = colors;
        let mut x = x;
        let mut len = len;
        if x < self.xmin() {
            let d = self.xmin() - x;
            len -= d;
            if len <= d {
                return;
            }
            if covers.len() > 0 {
                covers = &covers[d as usize..];
            }
            colors = &colors[d as usize..];
            x = self.xmin();
        }
        if x + len > self.xmax() {
            len = self.xmax() - x + 1;
            if len <= 0 {
                return;
            }
        }
		
        self.ren.blend_color_hspan(x, y, len as u32, colors, covers, cover);
    }

    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: i32, colors: &[Self::C], covers: &[u8], cover: u8,
    ) {
        if x > self.xmax() {
            return;
        }
        if x < self.xmin() {
            return;
        }

        let mut covers = covers;
        let mut colors = colors;
        let mut y = y;
        let mut len = len;

        if y < self.ymin() {
            let d = self.ymin() - y;
            len -= d;
            if len <= 0 {
                return;
            }
            if !covers.is_empty() {
                covers = &covers[d as usize..];
            }
            colors = &colors[d as usize..];
            y = self.ymin();
        }

        if y + len > self.ymax() {
            len = self.ymax() - y + 1;
            if len <= 0 {
                return;
            }
        }
        self.ren
            .blend_color_vspan(x, y, len as u32, colors, covers, cover);
    }

    fn copy_color_hspan(&mut self, x: i32, y: i32, len: i32, colors: &[Self::C]) {
        if y > self.ymax() {
            return;
        }
        if y < self.ymin() {
            return;
        }

        let mut colors = colors;
        let mut x = x;
        let mut len = len;

        if x < self.xmin() {
            let d = self.xmin() - x;
            len -= d;
            if len <= 0 {
                return;
            }
            colors = &colors[d as usize..];
            x = self.xmin();
        }
        if x + len > self.xmax() {
            len = self.xmax() - x + 1;
            if len <= 0 {
                return;
            }
        }
        self.ren.copy_color_hspan(x, y, len as u32, colors);
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: i32, colors: &[Self::C]) {
        if x > self.xmax() {
            return;
        }
        if x < self.xmin() {
            return;
        }

        let mut colors = colors;
        let mut y = y;
        let mut len = len;

        if y < self.ymin() {
            let d = self.ymin() - y;
            len -= d;
            if len <= 0 {
                return;
            }
            colors = &colors[d as usize..];
            y = self.ymin();
        }
        if y + len > self.ymax() {
            len = self.ymax() - y + 1;
            if len <= 0 {
                return;
            }
        }
        self.ren.copy_color_vspan(x, y, len as u32, colors);
    }

    fn blend_bar(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: &Pix::C, cover: u8) {
        let mut rc = RectI::new(x1, y1, x2, y2);
        rc.normalize();
        if rc.clip(self.clip_box()) {
            for y in rc.y1..=rc.y2 {
                self.ren
                    .blend_hline(rc.x1, y, (rc.x2 - rc.x1 + 1) as u32, c, cover);
            }
        }
    }
}

// NOT TESTED
impl<'a, Pix: PixFmt> RendererBase<'a, Pix> {
    pub fn copy_from<RenBuf: RenderBuffer<T = Pix::T>>(
        &mut self, src: &RenBuf, rect_src_ptr: Option<&RectI>, dx: i32, dy: i32,
    ) {
        let mut rsrc = RectI::new(0, 0, src.width() as i32, src.height() as i32);
        if let Some(r) = rect_src_ptr {
            rsrc.x1 = r.x1;
            rsrc.y1 = r.y1;
            rsrc.x2 = r.x2 + 1;
            rsrc.y2 = r.y2 + 1;
        }

        // Version with xdst, ydst (absolute(x: i32. positioning)
        //RectI rdst(xdst, ydst, xdst + rsrc.x2 - rsrc.x1, ydst + rsrc.y2 - rsrc.y1);

        // Version with dx, dy (relative positioning)
        let mut rdst = RectI::new(rsrc.x1 + dx, rsrc.y1 + dy, rsrc.x2 + dx, rsrc.y2 + dy);

        let mut rc = self.clip_rect_area(
            &mut rdst,
            &mut rsrc,
            src.width() as i32,
            src.height() as i32,
        );

        if rc.x2 > 0 {
            let mut incy = 1;
            if rdst.y1 > rsrc.y1 {
                rsrc.y1 += rc.y2 - 1;
                rdst.y1 += rc.y2 - 1;
                incy = -1;
            }
            while rc.y2 > 0 {
                self.ren
                    .copy_from(src, rdst.x1, rdst.y1, rsrc.x1, rsrc.y1, rc.x2 as u32);
                rdst.y1 += incy;
                rsrc.y1 += incy;
                rc.y2 -= 1;
            }
        }
    }

    pub fn blend_from_color<R: PixFmt<T = Pix::T>>(
        &mut self, src: &R, color: &Pix::C, rect_src_ptr: Option<&RectI>, dx: i32, dy: i32,
        cover: u32,
    ) {
        let mut rsrc = RectI::new(0, 0, src.width() as i32, src.height() as i32);
        if let Some(r) = rect_src_ptr {
            rsrc.x1 = r.x1;
            rsrc.y1 = r.y1;
            rsrc.x2 = r.x2 + 1;
            rsrc.y2 = r.y2 + 1;
        }

        // Version with xdst, ydst (absolute positioning)
        //let rdist = RectI::new(xdst, ydst, xdst + rsrc.x2 - rsrc.x1, ydst + rsrc.y2 - rsrc.y1);

        // Version with dx, dy (relative positioning)
        let mut rdst = RectI::new(rsrc.x1 + dx, rsrc.y1 + dy, rsrc.x2 + dx, rsrc.y2 + dy);
        let mut rc = self.clip_rect_area(
            &mut rdst,
            &mut rsrc,
            src.width() as i32,
            src.height() as i32,
        );

        if rc.x2 > 0 {
            let mut incy = 1;
            if rdst.y1 > rsrc.y1 {
                rsrc.y1 += rc.y2 - 1;
                rdst.y1 += rc.y2 - 1;
                incy = -1;
            }
            while rc.y2 > 0 {
                let rw = src.row_data(rsrc.y1);
                if !rw.ptr.is_null() {
                    let mut x1src = rsrc.x1;
                    let mut x1dst = rdst.x1;
                    let mut len = rc.x2;
                    if rw.x1 > x1src {
                        x1dst += rw.x1 - x1src;
                        len -= rw.x1 - x1src;
                        x1src = rw.x1;
                    }
                    if len > 0 {
                        if x1src + len - 1 > rw.x2 {
                            len -= x1src + len - rw.x2 - 1;
                        }
                        if len > 0 {
                            self.ren.blend_from_color(
                                src, color, x1dst, rdst.y1, x1src, rsrc.y1, len as u32, cover,
                            );
                        }
                    }
                }
                rdst.y1 += incy;
                rsrc.y1 += incy;
                rc.y2 -= 1;
            }
        }
    }

    pub fn blend_from_lut<R: PixFmt<T = Pix::T>>(
        &mut self, src: &R, color_lut: &[Pix::C], rect_src_ptr: Option<&RectI>, dx: i32, dy: i32,
        cover: u32,
    ) {
        let mut rsrc = RectI::new(0, 0, src.width() as i32, src.height() as i32);
        if let Some(r) = rect_src_ptr {
            rsrc.x1 = r.x1;
            rsrc.y1 = r.y1;
            rsrc.x2 = r.x2 + 1;
            rsrc.y2 = r.y2 + 1;
        }

        // Version with xdst, ydst (absolute positioning)
        //let rdist = RectI::new(xdst, ydst, xdst + rsrc.x2 - rsrc.x1, ydst + rsrc.y2 - rsrc.y1);

        // Version with dx, dy (relative positioning)
        let mut rdst = RectI::new(rsrc.x1 + dx, rsrc.y1 + dy, rsrc.x2 + dx, rsrc.y2 + dy);
        let mut rc = self.clip_rect_area(
            &mut rdst,
            &mut rsrc,
            src.width() as i32,
            src.height() as i32,
        );

        if rc.x2 > 0 {
            let mut incy = 1;
            if rdst.y1 > rsrc.y1 {
                rsrc.y1 += rc.y2 - 1;
                rdst.y1 += rc.y2 - 1;
                incy = -1;
            }
            while rc.y2 > 0 {
                let rw = src.row_data(rsrc.y1);
                if !rw.ptr.is_null() {
                    let mut x1src = rsrc.x1;
                    let mut x1dst = rdst.x1;
                    let mut len = rc.x2;
                    if rw.x1 > x1src {
                        x1dst += rw.x1 - x1src;
                        len -= rw.x1 - x1src;
                        x1src = rw.x1;
                    }
                    if len > 0 {
                        if x1src + len - 1 > rw.x2 {
                            len -= x1src + len - rw.x2 - 1;
                        }
                        if len > 0 {
                            self.ren.blend_from_lut(
                                src, color_lut, x1dst, rdst.y1, x1src, rsrc.y1, len as u32, cover,
                            );
                        }
                    }
                }
                rdst.y1 += incy;
                rsrc.y1 += incy;
                rc.y2 -= 1;
            }
        }
    }
}
