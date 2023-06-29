use crate::Scanline;

use crate::basics::{CoverType, Span};
use crate::glyph_raster_bin::*;
use crate::{Color, GlyphGenerator, Renderer, RendererScanline};
//==============================================RendererRasterHtextSolid
pub struct RendererRasterHtextSolid<'a, R: Renderer, G: GlyphGenerator> {
    ren: &'a mut R,
    glyph: &'a mut G,
    color: R::C,
}

impl<'a, R: Renderer, G: GlyphGenerator> RendererRasterHtextSolid<'a, R, G> {
    pub fn new(ren: &'a mut R, glyph: &'a mut G) -> Self {
        RendererRasterHtextSolid {
            ren: ren,
            glyph: glyph,
            color: R::C::new(),
        }
    }

    pub fn glyph_gen(&self) -> &G {
        &*self.glyph
    }

    pub fn glyph_gen_mut(&mut self) -> &mut G {
        self.glyph
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
	
    pub fn color(&self) -> R::C {
        self.color
    }

    pub fn set_color(&mut self, c: R::C) {
        self.color = c;
    }

    pub fn render_text<T: AsRef<str>>(&mut self, x: f64, y: f64, text: T, flip: bool) {
        let mut r = GlyphRect::new();
        let mut x = x;
        let mut y = y;
        for c in text.as_ref().chars() {
            self.glyph.prepare(&mut r, x, y, c as u32, flip);
            if r.x2 >= r.x1 {


                if flip {
                    for i in r.y1..=r.y2 {
                        self.ren.blend_solid_hspan(
                            r.x1,
                            i,
                            r.x2 - r.x1 + 1,
                            &self.color,
                            self.glyph.span((r.y2 - i) as u32),
                        );
                    }
                } else {
                    for i in r.y1..=r.y2 {
                        self.ren.blend_solid_hspan(
                            r.x1,
                            i,
                            r.x2 - r.x1 + 1,
                            &self.color,
                            self.glyph.span((i - r.y1) as u32),
                        );
                    }
                }
            }

            x += r.dx;
            y += r.dy;
        }
    }
}

pub struct RendererRasterVtextSolid<'a, R: Renderer, G: GlyphGenerator> {
    ren: &'a mut R,
    glyph: &'a mut G,
    color: R::C,
}

impl<'a, R: Renderer, G: GlyphGenerator> RendererRasterVtextSolid<'a, R, G> {
    pub fn new(ren: &'a mut R, glyph: &'a mut G) -> Self {
        RendererRasterVtextSolid {
            ren: ren,
            glyph: glyph,
            color: R::C::new(),
        }
    }

    pub fn color(&self) -> &R::C {
        &self.color
    }

    pub fn set_color(&mut self, c: R::C) {
        self.color = c;
    }

    pub fn render_text(&mut self, x: f64, y: f64, str: &str, flip: bool) {
        let (mut x, mut y) = (x, y);
        let mut r = GlyphRect::new();
        for c in str.chars() {
            self.glyph.prepare(&mut r, x, y, c as u32, !flip);
            if r.x2 >= r.x1 {
                let mut i = r.y1;
                while i <= r.y2 {
                    if flip {
                        self.ren.blend_solid_vspan(
                            i,
                            r.x1,
                            r.x2 - r.x1 + 1,
                            &self.color,
                            self.glyph.span((i - r.y1) as u32),
                        );
                    } else {
                        self.ren.blend_solid_vspan(
                            i,
                            r.x1,
                            r.x2 - r.x1 + 1,
                            &self.color,
                            self.glyph.span((r.y2 - i) as u32),
                        );
                    }
                    i += 1;
                }
            }
            x += r.dx;
            y += r.dy;
        }
    }
}

//===================================================RendererRasterHtext
pub struct RendererRasterHtext<'a, R: RendererScanline, G: GlyphGenerator> {
    ren: &'a mut R,
    glyph: &'a mut G,
}

impl<'a, R: RendererScanline, G: GlyphGenerator> RendererRasterHtext<'a, R, G> {
    pub fn new(ren: &'a mut R, glyph: &'a mut G) -> Self {
        RendererRasterHtext { ren, glyph }
    }

    //--------------------------------------------------------------------
    pub fn render_text<T: AsRef<str>>(&mut self, x: f64, y: f64, str: T, flip: bool) {
        let (mut x, mut y) = (x, y);
        let mut r = GlyphRect::default();
        for c in str.as_ref().chars() {
            self.glyph.prepare(&mut r, x, y, c as u32, flip);
            if r.x2 >= r.x1 {
                self.ren.prepare();
                if flip {
                    for i in r.y1..=r.y2 {
                        let p = self.glyph.span((r.y2 - i) as u32).as_mut_ptr();
                        self.ren
                            .render(&ScanlineSingleSpan::new(r.x1, i, r.x2 - r.x1 + 1, p));
                    }
                } else {
                    for i in r.y1..=r.y2 {
                        self.ren.render(&ScanlineSingleSpan::new(
                            r.x1,
                            i,
                            r.x2 - r.x1 + 1,
                            self.glyph.span((i - r.y1) as u32).as_mut_ptr(),
                        ));
                    }
                }
            }
            x += r.dx;
            y += r.dy;
        }
    }
}

//------------------------------------------------------------------------
struct ScanlineSingleSpan {
    y: i32,
    span: [Span; 1],
}

impl ScanlineSingleSpan {
    pub fn new(x: i32, y: i32, len: i32, covers: *mut CoverType) -> ScanlineSingleSpan {
        ScanlineSingleSpan {
            y: y,
            span: [Span { x, len, covers }; 1],
        }
    }
}

impl Scanline for ScanlineSingleSpan {
    type CoverType = crate::basics::CoverType;
    fn y(&self) -> i32 {
        self.y
    }

    fn num_spans(&self) -> u32 {
        1
    }

    fn begin(&self) -> &[Span] {
        &self.span
    }
    fn reset(&mut self, _min_x: i32, _max_x: i32) {}
    fn reset_spans(&mut self) {}
    fn add_cell(&mut self, _x: i32, _cover: u32) {}
    fn add_span(&mut self, _x: i32, _len: u32, _cover: u32) {}
    fn add_cells(&mut self, _x: i32, _len: u32, _covers: &[Self::CoverType]) {}
    fn finalize(&mut self, _y: i32) {}
}
