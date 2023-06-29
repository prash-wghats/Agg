use crate::{ImageSrc, PixFmt, RenderBuffer};

pub struct PixfmtTransposer<'a, Pix: PixFmt> {
    pixf: &'a mut Pix,
}

impl<'a, Pix: PixFmt> PixfmtTransposer<'a, Pix> {
    pub fn new(pixf: &'a mut Pix) -> Self {
        Self { pixf: pixf }
    }
}

impl<'a, Pix: PixFmt> ImageSrc for PixfmtTransposer<'a, Pix> {}

impl<'a, Pix: PixFmt> PixFmt for PixfmtTransposer<'a, Pix> {
    type O = Pix::O;
    type C = Pix::C;
    type T = Pix::T;
    const PIXEL_WIDTH: u32 = Pix::PIXEL_WIDTH;

    fn width(&self) -> u32 {
        self.pixf.height()
    }

    fn height(&self) -> u32 {
        self.pixf.width()
    }
	fn attach_pixfmt<Pix_: PixFmt>(
        &mut self, pixf: &Pix_, x1: i32, y1: i32, x2: i32, y2: i32,
    ) -> bool {
		self.pixf.attach_pixfmt(pixf, x1, y1, x2, y2)
	}

    fn stride(&self) -> i32 {
        self.pixf.stride()
    }
    fn pix_ptr(&self, x: i32, y: i32) -> (&[u8], usize) {
        self.pixf.pix_ptr(x, y)
    }
    fn row(&self, y: i32) -> &[Self::T] {
        self.pixf.row(y)
    }
    fn row_data(&self, y: i32) -> crate::RowData<Self::T> {
        self.pixf.row_data(y)
    }
    fn row_mut(&mut self, y: i32) -> &mut [Self::T] {
        self.pixf.row_mut(y)
    }
    fn make_pix(&self, p: &mut [u8], c: &Self::C) {
        self.pixf.make_pix(p, c)
    }
    fn blend_from<R: PixFmt<T=Self::T>>(
        &mut self, from: &R, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32, cover: u32,
    ) {
        self.pixf
            .blend_from(from, xdst, ydst, xsrc, ysrc, len, cover)
    }
    fn blend_from_color<R: PixFmt<T=Self::T>>(
        &mut self, from: &R, color: &Self::C, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32,
        cover: u32,
    ) {
		self.pixf.blend_from_color(from, color, xdst, ydst, xsrc, ysrc, len, cover);
	}
    fn blend_from_lut<R: PixFmt<T=Self::T>>(
        &mut self, from: &R, color_lut: &[Self::C], xdst: i32, ydst: i32, xsrc: i32, ysrc: i32,
        len: u32, cover: u32,
    ) {
		self.pixf.blend_from_lut(from, color_lut, xdst, ydst, xsrc, ysrc, len, cover);
	}

    fn copy_from<Pix0: RenderBuffer<T=Self::T>>(
        &mut self, from: &Pix0, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32,
    ) {
        self.pixf.copy_from(from, xdst, ydst, xsrc, ysrc, len)
    }

    fn pixel(&self, x: i32, y: i32) -> Self::C {
        self.pixf.pixel(y, x)
    }

    fn copy_pixel(&mut self, x: i32, y: i32, c: &Self::C) {
        self.pixf.copy_pixel(y, x, &c);
    }
    fn blend_pixel(&mut self, x: i32, y: i32, c: &Self::C, cover: u8) {
        self.pixf.blend_pixel(y, x, &c, cover);
    }

    fn copy_hline(&mut self, x: i32, y: i32, len: u32, c: &Self::C) {
        self.pixf.copy_vline(y, x, len, &c);
    }

    fn copy_vline(&mut self, x: i32, y: i32, len: u32, c: &Self::C) {
        self.pixf.copy_hline(y, x, len, &c);
    }

    fn blend_hline(&mut self, x: i32, y: i32, len: u32, c: &Self::C, cover: u8) {
        self.pixf.blend_vline(y, x, len, &c, cover);
    }

    fn blend_vline(&mut self, x: i32, y: i32, len: u32, c: &Self::C, cover: u8) {
        self.pixf.blend_hline(y, x, len, &c, cover);
    }

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: u32, c: &Self::C, covers: &[u8]) {
        self.pixf.blend_solid_vspan(y, x, len, &c, covers);
    }

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: u32, c: &Self::C, covers: &[u8]) {
        self.pixf.blend_solid_hspan(y, x, len, &c, covers);
    }

    fn copy_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]) {
        self.pixf.copy_color_vspan(y, x, len, colors);
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]) {
        self.pixf.copy_color_hspan(y, x, len, colors);
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[Self::C], covers: &[u8], cover: u8,
    ) {
        self.pixf
            .blend_color_vspan(y, x, len, colors, covers, cover);
    }

    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[Self::C], covers: &[u8], cover: u8,
    ) {
        self.pixf
            .blend_color_hspan(y, x, len, colors, covers, cover);
    }
}
