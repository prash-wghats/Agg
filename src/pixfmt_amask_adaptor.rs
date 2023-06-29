use crate::array::PodArray;
use crate::basics::CoverScale;
use crate::{AggPrimitive, AlphaMask, ImageSrc, PixFmt, RenderBuffer};

//==================================================PixAmaskAdaptor
pub struct PixAmaskAdaptor<'a, P: PixFmt, A: AlphaMask<CoverType = u8>> {
    pixf: &'a mut P,
    mask: &'a mut A,
    span: PodArray<A::CoverType>,
}

impl<'a, P: PixFmt, A: AlphaMask<CoverType = u8>> PixAmaskAdaptor<'a, P, A> {
    const SPAN_EXTRA_TAIL: u32 = 256;

    pub fn new(pixf: &'a mut P, mask: &'a mut A) -> Self {
        PixAmaskAdaptor {
            pixf: pixf,
            mask: mask,
            span: PodArray::new(),
        }
    }

    fn realloc_span(&mut self, len: u32) {
        if len > self.span.len() as u32 {
            self.span.resize(
                len as usize + Self::SPAN_EXTRA_TAIL as usize,
                A::CoverType::from_u32(0),
            );
        }
    }

    fn init_span(&mut self, len: u32) {
        self.realloc_span(len);
        for i in 0..len {
            self.span[i as usize] = A::CoverType::from_u32(A::COVER_FULL);
        }
    }

    fn init_span_with_cover(&mut self, len: u32, covers: &[A::CoverType]) {
        self.realloc_span(len);
        for i in 0..len {
            self.span[i as usize] = covers[i as usize];
        }
    }

    pub fn attach_pixfmt2(&mut self, pixf: &'a mut P) {
		self.pixf = pixf;
    }

    pub fn attach_alpha_mask(&mut self, mask: &'a mut A) {
		self.mask = mask;
    }
}

impl<'a, P: PixFmt, A: AlphaMask<CoverType = u8>> ImageSrc for PixAmaskAdaptor<'a, P, A> {}

impl<'a, P: PixFmt, A: AlphaMask<CoverType = u8>> PixFmt for PixAmaskAdaptor<'a, P, A> {
    type C = P::C;
    type O = P::O;
    type T = P::T;
    const PIXEL_WIDTH: u32 = P::PIXEL_WIDTH;

    fn width(&self) -> u32 {
        self.pixf.width()
    }

    fn height(&self) -> u32 {
        self.pixf.height()
    }
	fn pix_ptr(&self, x: i32, y: i32) -> (&[u8],usize) {
		self.pixf.pix_ptr(x, y)
	}
	fn make_pix(&self, p: &mut [u8], c: &Self::C) {
		self.pixf.make_pix(p, c)
	}
    fn stride(&self) -> i32 {
        self.pixf.stride()
    }
    fn row_data(&self, y: i32) -> crate::basics::RowData<Self::T> {
        self.pixf.row_data(y)
    }
    fn row(&self, y: i32) -> &[Self::T] {
        self.pixf.row(y)
    }
    fn row_mut(&mut self, y: i32) -> &mut [Self::T] {
        self.pixf.row_mut(y)
    }
    fn blend_from<R: PixFmt<T = Self::T>>(
        &mut self, from: &R, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32, cover: u32,
    ) {
        self.pixf
            .blend_from(from, xdst, ydst, xsrc, ysrc, len, cover)
    }
    fn blend_from_color<R: PixFmt<T = Self::T>>(
        &mut self, from: &R, color: &Self::C, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32,
        cover: u32,
    ) {
        self.pixf
            .blend_from_color(from, color, xdst, ydst, xsrc, ysrc, len, cover)
    }
    fn blend_from_lut<R: PixFmt<T = Self::T>>(
        &mut self, from: &R, color_lut: &[Self::C], xdst: i32, ydst: i32, xsrc: i32, ysrc: i32,
        len: u32, cover: u32,
    ) {
        self.pixf
            .blend_from_lut(from, color_lut, xdst, ydst, xsrc, ysrc, len, cover)
    }
    fn attach_pixfmt<P2: PixFmt>(&mut self, pixf: &P2, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        self.pixf.attach_pixfmt(pixf, x1, y1, x2, y2)
    }

    fn pixel(&self, x: i32, y: i32) -> P::C {
        self.pixf.pixel(x, y)
    }

    fn copy_pixel(&mut self, x: i32, y: i32, c: &P::C) {
        self.pixf
            .blend_pixel(x, y, &c, self.mask.pixel(x, y).into_u8());
    }

    fn blend_pixel(&mut self, x: i32, y: i32, c: &P::C, cover: A::CoverType) {
        self.pixf
            .blend_pixel(x, y, &c, self.mask.combine_pixel(x, y, cover).into_u8());
    }

    fn copy_hline(&mut self, x: i32, y: i32, len: u32, c: &P::C) {
        self.realloc_span(len);
        self.mask.fill_hspan(x, y, &mut self.span, len as i32);
        self.pixf.blend_solid_hspan(x, y, len, &c, &self.span[..]);
    }

    fn blend_hline(&mut self, x: i32, y: i32, len: u32, c: &P::C, _cover: A::CoverType) {
        self.init_span(len);
        self.mask.combine_hspan(x, y, &mut self.span, len as i32);
        self.pixf.blend_solid_hspan(x, y, len, &c, &self.span);
    }

    fn copy_vline(&mut self, x: i32, y: i32, len: u32, c: &P::C) {
        self.realloc_span(len);
        self.mask.fill_vspan(x, y, &mut self.span, len as i32);
        self.pixf.blend_solid_vspan(x, y, len, &c, &self.span[..]);
    }

    fn blend_vline(&mut self, x: i32, y: i32, len: u32, c: &P::C, _cover: A::CoverType) {
        self.init_span(len);
        self.mask.combine_vspan(x, y, &mut self.span, len as i32);
        self.pixf.blend_solid_vspan(x, y, len, &c, &self.span[..]);
    }

    fn copy_from<Ren: RenderBuffer<T = Self::T>>(
        &mut self, from: &Ren, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32,
    ) {
        self.pixf.copy_from(from, xdst, ydst, xsrc, ysrc, len);
    }

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: u32, c: &P::C, covers: &[A::CoverType]) {
        self.init_span_with_cover(len, covers);
        self.mask.combine_hspan(x, y, &mut self.span, len as i32);
        self.pixf.blend_solid_hspan(x, y, len, &c, &self.span);
    }

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: u32, c: &P::C, covers: &[A::CoverType]) {
        self.init_span_with_cover(len, covers);
        self.mask.combine_vspan(x, y, &mut self.span, len as i32);
        self.pixf.blend_solid_vspan(x, y, len, &c, &self.span[..]);
    }

    fn copy_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[P::C]) {
        self.realloc_span(len);
        self.mask.fill_hspan(x, y, &mut self.span, len as i32);
        self.pixf
            .blend_color_hspan(x, y, len, colors, &self.span[..], CoverScale::FULL as u8);
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[P::C]) {
        self.realloc_span(len);
        self.mask.fill_vspan(x, y, &mut self.span, len as i32);
        self.pixf
            .blend_color_vspan(x, y, len, colors, &self.span[..], CoverScale::FULL as u8);
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[P::C], covers: &[A::CoverType],
        cover: A::CoverType,
    ) {
        if covers.len() > 0 {
            self.init_span_with_cover(len, covers);
            self.mask.combine_hspan(x, y, &mut self.span, len as i32);
        } else {
            self.realloc_span(len);
            self.mask.fill_hspan(x, y, &mut self.span, len as i32);
        }
        self.pixf
            .blend_color_hspan(x, y, len, &colors, &self.span[..], cover);
    }

    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[P::C], covers: &[A::CoverType],
        cover: A::CoverType,
    ) {
        if covers.len() > 0 {
            self.init_span_with_cover(len, covers);
            self.mask.combine_vspan(x, y, &mut self.span, len as i32);
        } else {
            self.realloc_span(len);
            self.mask.fill_vspan(x, y, &mut self.span, len as i32);
        }
        self.pixf
            .blend_color_vspan(x, y, len, colors, &self.span[..], cover);
    }
}
