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
//
// The author gratefully acknowleges the support of David Turner,
// Robert Wilhelm, and Werner Lemberg - the authors of the FreeType
// libray - in producing this work. See http://www.freetype.org for details.
//
//----------------------------------------------------------------------------
// Contact: mcseem@antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------
//
// Adaptation for 32-bit screen coordinates has been sponsored by
// Liberty Technology Systems, Inc., visit http://lib-sys.com
//
// Liberty Technology Systems, Inc. is the provider of
// PostScript and PDF technology for software developers.
//
//----------------------------------------------------------------------------
use self::AaScale::*;
use self::Status::*;
use crate::basics::{FillingRule, PolySubpixelScale, *};
use crate::rasterizer_cells_aa::*;
use crate::rasterizer_sl_clip::*;
use crate::GammaFn;
use crate::RasClip;
use crate::RasConv;
use crate::Rasterizer;
use crate::{AggPrimitive, RasterScanLine, Scanline, VertexSource};

macro_rules! from_i32 {
    ($v:expr) => {
        <Clip::ConvType as RasConv>::CoordType::from_i32($v)
    };
}

struct CellDummy;

impl crate::CellFn for CellDummy {
    fn style(&mut self, _: &mut Cell, _: &Cell) {}

    fn not_equal(&self, me: &Cell, ex: i32, ey: i32, _: &Cell) -> i32 {
        (ex.wrapping_sub(me.x)) | (ey.wrapping_sub(me.y))
    }
}

//==================================================RasterizerScanlineAa
// Polygon rasterizer that is used to render filled polygons with
// high-quality Anti-Aliasing. Internally, by default, the class uses
// integer coordinates in format 24.8, i.e. 24 bits for integer part
// and 8 bits for fractional - see PolySubpixelScale::Shift. This class can be
// used in the following  way:
//
// 1. filling_rule(FillingRule ft) - optional.
//
// 2. gamma() - optional.
//
// 3. reset()
//
// 4. move_to(x, y) / line_to(x, y) - make the polygon. One can create
//    more than one contour, but each contour must consist of at least 3
//    vertices, i.e. move_to(x1, y1); line_to(x2, y2); line_to(x3, y3);
//    is the absolute minimum of vertices that define a triangle.
//    The algorithm does not check either the number of vertices nor
//    coincidence of their coordinates, but in the worst case it just
//    won't draw anything.
//    The orger of the vertices (clockwise or counterclockwise)
//    is important when using the non-zero filling rule (fill_non_zero).
//    In this case the vertex order of all the contours must be the same
//    if you want your intersecting polygons to be without "holes".
//    You actually can use different vertices order. If the contours do not
//    intersect each other the order is not important anyway. If they do,
//    contours with the same vertex order will be rendered without "holes"
//    while the intersecting contours with different orders will have "holes".
//
// filling_rule() and gamma() can be called anytime before "sweeping".
//------------------------------------------------------------------------

#[repr(u32)]
enum Status {
    Initial,
    MoveTo,
    LineTo,
    Closed,
}

#[repr(i32)]
pub enum AaScale {
    Shift = 8,
    Scale = 1 << Shift as i32,
    Mask = Scale as i32 - 1,
    Scale2 = Scale as i32 * 2,
    Mask2 = Scale2 as i32 - 1,
}

pub struct RasterizerScanlineAa<Clip: RasClip = RasterizerSlClipInt> {
    m_outline: RasterizerCellsAa<CellDummy>,
    m_clipper: Clip,
    m_gamma: [i32; AaScale::Scale as i32 as usize],
    m_filling_rule: FillingRule,
    m_auto_close: bool,
    m_start_x: <Clip::ConvType as RasConv>::CoordType,
    m_start_y: <Clip::ConvType as RasConv>::CoordType,
    m_status: u32,
    m_scan_y: i32,
}

impl<Clip: RasClip> RasterizerScanlineAa<Clip> {
    pub fn new() -> Self {
        let dum = CellDummy;
        let mut r = RasterizerScanlineAa {
            m_outline: RasterizerCellsAa::<CellDummy>::new(dum),
            m_clipper: Clip::new(),
            m_gamma: [0; AaScale::Scale as i32 as usize],
            m_filling_rule: FillingRule::FillNonZero,
            m_auto_close: true,
            m_start_x: from_i32!(0),
            m_start_y: from_i32!(0),
            m_status: Initial as u32,
            m_scan_y: 0,
        };
        for i in 0..Scale as usize {
            r.m_gamma[i] = i as i32;
        }
        r
    }

    pub fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        if is_move_to(cmd) {
            self.move_to_d(x, y);
        } else if is_vertex(cmd) {
            self.line_to_d(x, y);
        } else if is_close(cmd) {
            self.close_polygon();
        }
    }

    pub fn set_gamma<F: GammaFn>(&mut self, gamma_function: &F) {
        for i in 0..AaScale::Scale as usize {
            self.m_gamma[i] =
                uround(gamma_function.call(i as f64 / Mask as i32 as f64) * Mask as i32 as f64);
        }
    }

    #[inline]
    fn calculate_alpha(&self, area: i32) -> u32 {
        let mut cover = area >> (PolySubpixelScale::Shift as u32 * 2 + 1 - Shift as u32);
        cover = if cover < 0 { -cover } else { cover };
        if self.m_filling_rule == FillingRule::FillEvenOdd {
            cover &= Mask2 as i32;
            if cover > Scale as i32 {
                cover = Scale2 as i32 - cover
            }
        }

        if cover > Mask as i32 {
            cover = Mask as i32;
        }

        self.m_gamma[cover as usize] as u32
    }

    pub fn clip_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.reset();
        self.m_clipper.clip_box(
            Clip::ConvType::upscale(x1),
            Clip::ConvType::upscale(y1),
            Clip::ConvType::upscale(x2),
            Clip::ConvType::upscale(y2),
        );
    }

    pub fn close_polygon(&mut self) {
        if self.m_status == LineTo as u32 {
            self.m_clipper
                .line_to(&mut self.m_outline, self.m_start_x, self.m_start_y);
            self.m_status = Closed as u32;
        }
    }

    pub fn set_filling_rule(&mut self, filling_rule: FillingRule) {
        self.m_filling_rule = filling_rule;
    }

    pub fn reset_clipping(&mut self) {
        self.reset();
        self.m_clipper.reset_clipping();
    }

    pub fn set_auto_close(&mut self, flag: bool) {
        self.m_auto_close = flag;
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        if self.m_outline.sorted() {
            self.reset();
        }
        if self.m_auto_close {
            self.close_polygon();
        }
        self.m_start_x = Clip::ConvType::downscale(x);
        self.m_start_y = Clip::ConvType::downscale(y);
        self.m_clipper.move_to(self.m_start_x, self.m_start_y);
        self.m_status = MoveTo as u32;
    }

    pub fn line_to(&mut self, x: i32, y: i32) {
        self.m_clipper.line_to(
            &mut self.m_outline,
            Clip::ConvType::downscale(x),
            Clip::ConvType::downscale(y),
        );

        self.m_status = LineTo as u32;
    }

    pub fn edge(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if self.m_outline.sorted() {
            self.reset();
        }
        self.m_clipper
            .move_to(Clip::ConvType::downscale(x1), Clip::ConvType::downscale(y1));
        self.m_clipper.line_to(
            &mut self.m_outline,
            Clip::ConvType::downscale(x2),
            Clip::ConvType::downscale(y2),
        );
        self.m_status = MoveTo as u32;
    }

    pub fn sort(&mut self) {
        if self.m_auto_close {
            self.close_polygon();
        }
        self.m_outline.sort_cells();
    }

    pub fn navigate_scanline(&mut self, y: i32) -> bool {
        if self.m_auto_close {
            self.close_polygon();
        }
        self.m_outline.sort_cells();
        if self.m_outline.total_cells() == 0
            || y < self.m_outline.min_y()
            || y > self.m_outline.max_y()
        {
            return false;
        }
        self.m_scan_y = y;
        true
    }

    pub fn hit_test(&mut self, tx: i32, ty: i32) -> bool {
        if !self.navigate_scanline(ty) {
            return false;
        }
        let mut sl = ScanlineHitTest::new(tx);
        self.sweep_scanline(&mut sl);
        sl.hit()
    }

    pub fn move_to_d(&mut self, x: f64, y: f64) {
        if self.m_outline.sorted() {
            self.reset();
        }
        if self.m_auto_close {
            self.close_polygon();
        }
        self.m_start_x = Clip::ConvType::upscale(x);
        self.m_start_y = Clip::ConvType::upscale(y);
        self.m_clipper.move_to(self.m_start_x, self.m_start_y);
        self.m_status = MoveTo as u32;
    }

    pub fn line_to_d(&mut self, x: f64, y: f64) {
        self.m_clipper.line_to(
            &mut self.m_outline,
            Clip::ConvType::upscale(x),
            Clip::ConvType::upscale(y),
        );
        self.m_status = LineTo as u32;
    }
}

impl<Clip: RasClip> RasterScanLine for RasterizerScanlineAa<Clip> {
    fn reset(&mut self) {
        self.m_outline.reset();
        self.m_status = Initial as u32;
    }

    fn apply_gamma(&self, cover: usize) -> u32 {
        self.m_gamma[cover] as u32
    }

    fn add_path<VS: VertexSource>(&mut self, vs: &mut VS, path_id: u32) {
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;
        let mut cmd: u32;
        vs.rewind(path_id);
        if self.m_outline.sorted() {
            self.reset();
        }
        loop {
            cmd = vs.vertex(&mut x, &mut y);

            if is_stop(cmd) {
                break;
            }

            self.add_vertex(x, y, cmd);
        }
    }

    fn rewind_scanlines(&mut self) -> bool {
        if self.m_auto_close {
            self.close_polygon();
        }
        self.m_outline.sort_cells();
        if self.m_outline.total_cells() == 0 {
            return false;
        }
        self.m_scan_y = self.m_outline.min_y();
        return true;
    }
	
    fn min_x(&self) -> i32 {
        self.m_outline.min_x()
    }

    fn min_y(&self) -> i32 {
        self.m_outline.min_y()
    }

    fn max_x(&self) -> i32 {
        self.m_outline.max_x()
    }

    fn max_y(&self) -> i32 {
        self.m_outline.max_y()
    }

    fn sweep_scanline<Sl: Scanline>(&mut self, sl: &mut Sl) -> bool {
        loop {
            if self.m_scan_y > self.m_outline.max_y() {
                return false;
            }
            sl.reset_spans();
            let mut num_cells = self.m_outline.scanline_num_cells(self.m_scan_y as u32);
            let cells = self.m_outline.scanline_cells(self.m_scan_y as u32);
            let mut cover = 0;
            let mut ci = 0;
            unsafe {
                while num_cells > 0 {
                    let mut cur_cell = cells[ci];
                    let mut x = (*cur_cell).x;
                    let mut area = (*cur_cell).area;
                    let mut alpha;

                    cover += (*cur_cell).cover;
                    num_cells -= 1;
                    ci += 1;
                    //accumulate all cells with the same X
                    while num_cells > 0 {
                        cur_cell = cells[ci];

                        if (*cur_cell).x != x {
                            break;
                        }
                        area += (*cur_cell).area;
                        cover += (*cur_cell).cover;
                        num_cells -= 1;
                        ci += 1;
                    }

                    if area != 0 {
                        alpha = self.calculate_alpha(
                            (cover << (PolySubpixelScale::Shift as i32 + 1)).wrapping_sub(area),
                        );
                        if alpha != 0 {
                            sl.add_cell(x, alpha);
                        }
                        x += 1;
                    }

                    if num_cells > 0 && (*cur_cell).x > x {
                        alpha =
                            self.calculate_alpha(cover << (PolySubpixelScale::Shift as i32 + 1));
                        if alpha != 0 {
                            sl.add_span(x, ((*cur_cell).x - x) as u32, alpha);
                        }
                    }
                    //cells = &cells[1..];
                }
            }
            if sl.num_spans() > 0 {
                break;
            }
            self.m_scan_y += 1;
        }

        sl.finalize(self.m_scan_y);
        self.m_scan_y += 1;
        return true;
    }
}
