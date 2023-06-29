use crate::array::VecPodB;
use crate::basics::*;
use crate::renderer_base::RendererBase;
use crate::{Color, PixFmt, RenderBuffer, Renderer};

// NOT TESTED

//----------------------------------------------------------RendererMclip
pub struct RendererMclip<'a, P: PixFmt> {
    ren: RendererBase<'a, P>,
    curr_cb: usize,
    bounds: RectI,
    clip: VecPodB<RectI>,
}

impl<'a, P: PixFmt> RendererMclip<'a, P> {
    pub fn new(pixf: &'a mut P) -> Self {
        let ren = RendererBase::new_borrowed(pixf);
        let b = RectI::new(ren.xmin(), ren.ymin(), ren.xmax(), ren.ymax());
        let r = Self {
            ren: ren,
            curr_cb: 0,
            bounds: b,
            clip: VecPodB::new(),
        };
        r
    }

    pub fn attach(&mut self, pixf: &'a mut P) {
        self.ren.attach_borrowed(pixf);
        self.reset_clipping(true);
    }

    pub fn ren(&self) -> &P {
        self.ren.ren()
    }

    pub fn ren_mut(&mut self) -> &mut P {
        self.ren.ren_mut()
    }

    pub fn width(&self) -> u32 {
        self.ren.width()
    }

    pub fn height(&self) -> u32 {
        self.ren.height()
    }

    pub fn first_clip_box(&mut self) {
        self.curr_cb = 0;
        if !self.clip.is_empty() {
            let cb = &self.clip[0];
            self.ren.clip_box_naked(cb.x1, cb.y1, cb.x2, cb.y2);
        }
    }

    pub fn next_clip_box(&mut self) -> bool {
		self.curr_cb += 1;
		if self.curr_cb < self.clip.len() {
            let cb = &self.clip[self.curr_cb];
            self.ren.clip_box_naked(cb.x1, cb.y1, cb.x2, cb.y2);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self, c: &P::C) {
        self.ren.clear(c);
    }

    pub fn clip_box(&self) -> &RectI {
        self.ren.clip_box()
    }

    pub fn xmin(&self) -> i32 {
        self.ren.xmin()
    }

    pub fn ymin(&self) -> i32 {
        self.ren.ymin()
    }

    pub fn xmax(&self) -> i32 {
        self.ren.xmax()
    }

    pub fn ymax(&self) -> i32 {
        self.ren.ymax()
    }

    pub fn bounding_xmin(&self) -> i32 {
        self.bounds.x1
    }

    pub fn bounding_ymin(&self) -> i32 {
        self.bounds.y1
    }

    pub fn bounding_xmax(&self) -> i32 {
        self.bounds.x2
    }

    pub fn bounding_ymax(&self) -> i32 {
        self.bounds.y2
    }

    pub fn reset_clipping(&mut self, visibility: bool) {
        self.ren.reset_clipping(visibility);
        self.clip.clear();
        self.curr_cb = 0;
        self.bounds = *self.ren.clip_box();
    }

    pub fn add_clip_box(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut cb = RectI::new(x1, y1, x2, y2);
        cb.normalize();
        if cb.clip(&RectI::new(
            0,
            0,
            self.width() as i32 - 1,
            self.height() as i32 - 1,
        )) {
            self.clip.push(cb);
            if cb.x1 < self.bounds.x1 {
                self.bounds.x1 = cb.x1;
            }
            if cb.y1 < self.bounds.y1 {
                self.bounds.y1 = cb.y1;
            }
            if cb.x2 > self.bounds.x2 {
                self.bounds.x2 = cb.x2;
            }
            if cb.y2 > self.bounds.y2 {
                self.bounds.y2 = cb.y2;
            }
        }
    }

    pub fn copy_pixel(&mut self, x: i32, y: i32, c: &P::C) {
        self.first_clip_box();
        loop {
            if self.ren.inbox(x, y) {
                self.ren.ren_mut().copy_pixel(x, y, c);
                break;
            }
            if !self.next_clip_box() {
                break;
            }
        }
    }

    pub fn copy_from<RenBuf: RenderBuffer<T = P::T>>(
        &mut self, from: &RenBuf, rc: Option<&RectI>, x_to: i32, y_to: i32,
    ) {
        self.first_clip_box();
        loop {
            self.ren.copy_from(from, rc, x_to, y_to);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    pub fn blend_from<R: PixFmt<T = P::T>>(
        &mut self, src: &R, rect_src_ptr: &RectI, dx: i32, dy: i32, cover: u32,
    ) {
        self.first_clip_box();
        loop {
            self.ren.blend_from(src, Some(rect_src_ptr), dx, dy, cover);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    pub fn copy_bar(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: &P::C) {
        self.first_clip_box();
        loop {
            self.ren.copy_bar(x1, y1, x2, y2, c);
            if !self.next_clip_box() {
                break;
            }
        }
    }
}

impl<'a, P: PixFmt> Renderer for RendererMclip<'a, P> {
    type C = P::C;

    fn bounding_clip_box(&self) -> &RectI {
        &self.bounds
    }

    fn blend_pixel(&mut self, x: i32, y: i32, c: &P::C, cover: CoverType) {
        self.first_clip_box();
        loop {
            if self.ren.inbox(x, y) {
                self.ren.ren_mut().blend_pixel(x, y, c, cover);
                break;
            }
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn pixel(&mut self, x: i32, y: i32) -> P::C {
        self.first_clip_box();
        loop {
            if self.ren.inbox(x, y) {
                return self.ren.ren().pixel(x, y);
            }
            if !self.next_clip_box() {
                break;
            }
        }
        P::C::no_color()
    }

    fn copy_hline(&mut self, x1: i32, y: i32, x2: i32, c: &P::C) {
        self.first_clip_box();
        loop {
            self.ren.copy_hline(x1, y, x2, c);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn copy_vline(&mut self, x: i32, y1: i32, y2: i32, c: &P::C) {
        self.first_clip_box();
        loop {
            self.ren.copy_vline(x, y1, y2, c);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn blend_hline(&mut self, x1: i32, y: i32, x2: i32, c: &P::C, cover: CoverType) {
        self.first_clip_box();
        loop {
            self.ren.blend_hline(x1, y, x2, c, cover);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn blend_vline(&mut self, x: i32, y1: i32, y2: i32, c: &P::C, cover: CoverType) {
        self.first_clip_box();
        loop {
            self.ren.blend_vline(x, y1, y2, c, cover);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: i32, c: &P::C, covers: &[CoverType]) {
        self.first_clip_box();
        loop {
            self.ren.blend_solid_hspan(x, y, len, c, covers);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: i32, c: &P::C, covers: &[CoverType]) {
        self.first_clip_box();
        loop {
            self.ren.blend_solid_vspan(x, y, len, c, covers);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn copy_color_hspan(&mut self, x: i32, y: i32, len: i32, colors: &[P::C]) {
        self.first_clip_box();
        loop {
            self.ren.copy_color_hspan(x, y, len, colors);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: i32, colors: &[P::C]) {
        self.first_clip_box();
        loop {
            self.ren.copy_color_vspan(x, y, len, colors);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: i32, colors: &[P::C], covers: &[CoverType],
        cover: CoverType,
    ) {
        self.first_clip_box();
        loop {
            self.ren.blend_color_hspan(x, y, len, colors, covers, cover);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: i32, colors: &[P::C], covers: &[CoverType],
        cover: CoverType,
    ) {
        self.first_clip_box();
        loop {
            self.ren.blend_color_vspan(x, y, len, colors, covers, cover);
            if !self.next_clip_box() {
                break;
            }
        }
    }

    fn blend_bar(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: &P::C, cover: CoverType) {
        self.first_clip_box();
        loop {
            self.ren.blend_bar(x1, y1, x2, y2, c, cover);
            if !self.next_clip_box() {
                break;
            }
        }
    }
}
