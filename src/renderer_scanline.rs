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
// Function render_ctrl
//
//----------------------------------------------------------------------------
use std::ops::Index;

use crate::basics::CoverScale;
use crate::{
    Color, Equiv, RasterScanLine, RasterStyle, Rasterizer0, Renderer, RendererScanline,
    RendererScanlineColor, Scanline, SpanAllocator, SpanGenerator, VertexSource,
};

//================================================render_scanline_aa_solid
fn render_scanline_aa_solid<S: Scanline, R: Renderer<C = C>, C: Color>(
    sl: &S, ren: &mut R, color: &C,
) {
    let span = sl.begin();
    let y = sl.y();
    for s in span {
        let x = s.x;
        if s.len > 0 {
            let slice = unsafe { std::slice::from_raw_parts(s.covers, s.len as usize) };
            ren.blend_solid_hspan(x, y, s.len, color, slice);
        } else {
            ren.blend_hline(x, y, x - s.len - 1, color, unsafe { *s.covers });
        }
    }
}

//===============================================render_scanlines_aa_solid
pub fn render_scanlines_aa_solid<Ras: RasterScanLine, S: Scanline, R: Renderer>(
    ras: &mut Ras, sl: &mut S, ren: &mut R, ren_color: &R::C,
) {
    if ras.rewind_scanlines() {
        /* let ren_color = R::C::new_from_rgba(&Rgba::new_params(
            color.r().into_f64() / 255.,
            color.g().into_f64() / 255.,
            color.b().into_f64() / 255.,
            color.a().into_f64() / 255.,
        ));*/

        sl.reset(ras.min_x(), ras.max_x());
        while ras.sweep_scanline(sl) {
            let y = sl.y();
            let span = sl.begin();

            for s in span {
                let x = s.x;
                if s.len > 0 {
                    let slice = unsafe { std::slice::from_raw_parts(s.covers, s.len as usize) };
                    ren.blend_solid_hspan(x, y, s.len, &ren_color, slice);
                } else {
                    ren.blend_hline(x, y, x - s.len - 1, &ren_color, unsafe { *s.covers });
                }
            }
        }
    }
}

//==============================================RendererScanlineAASolid

pub struct RendererScanlineAASolid<'a, R: Renderer> {
    m_ren: Equiv<'a, R>,
    m_color: R::C,
}

impl<'a, R: Renderer> RendererScanlineAASolid<'a, R> {
    pub fn new_borrowed(ren: &'a mut R) -> Self {
        Self {
            m_ren: Equiv::Brw(ren),
            m_color: R::C::new(),
        }
    }
    pub fn new_owned(ren: R) -> Self {
        Self {
            m_ren: Equiv::Own(ren),
            m_color: R::C::new(),
        }
    }
    pub fn ren_mut(&mut self) -> &mut R {
        return &mut self.m_ren;
    }

    pub fn ren(&self) -> &R {
        return &self.m_ren;
    }
}

impl<'a, R: Renderer> RendererScanlineColor for RendererScanlineAASolid<'a, R> {
    type C = R::C;
    fn set_color(&mut self, c: R::C) {
        self.m_color = c;
    }
    fn color(&self) -> R::C {
        self.m_color
    }
}

impl<'a, R: Renderer> RendererScanline for RendererScanlineAASolid<'a, R> {
    fn prepare(&mut self) {}
    fn render<Sl: Scanline>(&mut self, sl: &Sl) {
        render_scanline_aa_solid(sl, &mut *self.m_ren, &self.m_color);
    }
}

//====================================================RendererScanlineAA

pub struct RendererScanlineAA<'a, Ren: Renderer, Sa: SpanAllocator, Sg: SpanGenerator> {
    m_ren: Equiv<'a, Ren>,
    m_alloc: Sa,
    m_span_gen: Equiv<'a, Sg>,
}

impl<'a, Ren: Renderer, Sa: SpanAllocator<C = Ren::C>, Sg: SpanGenerator<C = Ren::C>>
    RendererScanlineAA<'a, Ren, Sa, Sg>
{
    pub fn new_borrowed(ren: &'a mut Ren, alloc: Sa, span_gen: &'a mut Sg) -> Self {
        Self {
            m_ren: Equiv::Brw(ren),
            m_alloc: alloc,
            m_span_gen: Equiv::Brw(span_gen),
        }
    }

    pub fn new_owned(ren: Ren, alloc: Sa, span_gen: Sg) -> Self {
        Self {
            m_ren: Equiv::Own(ren),
            m_alloc: alloc,
            m_span_gen: Equiv::Own(span_gen),
        }
    }

    pub fn span_gen_mut(&mut self) -> &mut Sg {
        &mut self.m_span_gen
    }

    pub fn span_gen(&self) -> &Sg {
        &self.m_span_gen
    }

    pub fn ren_mut(&mut self) -> &mut Ren {
        return &mut self.m_ren;
    }

    pub fn ren(&self) -> &Ren {
        return &self.m_ren;
    }
}

impl<'a, Ren: Renderer, Sa: SpanAllocator<C = Ren::C>, Sg: SpanGenerator<C = Ren::C>>
    RendererScanline for RendererScanlineAA<'a, Ren, Sa, Sg>
{
    fn prepare(&mut self) {
        self.m_span_gen.prepare();
    }
    fn render<S: Scanline>(&mut self, sl: &S) {
        render_scanline_aa(
            sl,
            &mut *self.m_ren,
            &mut self.m_alloc,
            &mut *self.m_span_gen,
        );
    }
}
/*
impl<'a, Ren: Renderer, Sa: SpanAllocator<C = Ren::C>, Sg: SpanGenerator<C = Ren::C>>
    RendererScanlineColor for RendererScanlineAA<'a, Ren, Sa, Sg>
{
}*/

//======================================================render_scanline_aa
pub fn render_scanline_aa<
    S: Scanline,
    Ren: Renderer,
    Sa: SpanAllocator<C = Ren::C>,
    Sg: SpanGenerator<C = Ren::C>,
>(
    sl: &S, ren: &mut Ren, alloc: &mut Sa, span_gen: &mut Sg,
) {
    let y = sl.y();
    let span = sl.begin();

    for s in span {
        let x = s.x;
        let mut len = s.len as i32; // XXXX span.len should be i32
        let covers = s.covers;

        if len < 0 {
            len = -len;
        }
        let colors = alloc.allocate(len as usize);
        span_gen.generate(colors, x, y, len as u32);
        let slice = unsafe {
            if s.len < 0 {
                std::slice::from_raw_parts(s.covers, 0)
            } else {
                std::slice::from_raw_parts(s.covers, s.len as usize)
            }
        };
        ren.blend_color_hspan(x, y, len, colors, slice, unsafe { *covers });
    }
}

//=====================================================render_scanlines_aa
pub fn render_scanlines_aa<
    R: RasterScanLine,
    S: Scanline,
    Ren: Renderer,
    Sa: SpanAllocator<C = Ren::C>,
    Sg: SpanGenerator<C = Ren::C>,
>(
    ras: &mut R, sl: &mut S, ren: &mut Ren, alloc: &mut Sa, span_gen: &mut Sg,
) {
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        span_gen.prepare();
        while ras.sweep_scanline(sl) {
            render_scanline_aa(sl, ren, alloc, span_gen);
        }
    }
}

//================================================render_scanline_bin_solid
pub fn render_scanline_bin_solid<S: Scanline, R: Renderer<C = C>, C: Color>(
    sl: &S, ren: &mut R, color: &C,
) {
    let span = sl.begin();

    for s in span {
        let x = s.x;
        ren.blend_hline(
            x,
            sl.y(),
            x - 1 + (if s.len < 0 { -s.len } else { s.len }),
            color,
            CoverScale::FULL as u8,
        );
    }
}

//==============================================render_scanlines_bin_solid
pub fn render_scanlines_bin_solid<Ras: RasterScanLine, S: Scanline, R: Renderer>(
    ras: &mut Ras, sl: &mut S, ren: &mut R, ren_color: &R::C,
) {
    if ras.rewind_scanlines() {
        /*let ren_color = R::C::new_from_rgba(&Rgba::new_params(
            color.r().into_f64() / 255.,
            color.g().into_f64() / 255.,
            color.b().into_f64() / 255.,
            color.a().into_f64() / 255.,
        ));*/

        sl.reset(ras.min_x(), ras.max_x());
        while ras.sweep_scanline(sl) {
            let span = sl.begin();
            for s in span {
                let x = s.x;
                ren.blend_hline(
                    x,
                    sl.y(),
                    x - 1 + (if s.len < 0 { -s.len } else { s.len }),
                    &ren_color,
                    CoverScale::FULL as u8,
                );
            }
        }
    }
}

//==============================================RendererScanlineBinSolid

pub struct RendererScanlineBinSolid<'a, R: Renderer> {
    m_ren: Equiv<'a, R>,
    m_color: R::C,
}

impl<'a, R: Renderer> RendererScanlineBinSolid<'a, R> {
    pub fn new_borrowed(ren: &'a mut R) -> Self {
        Self {
            m_ren: Equiv::Brw(ren),
            m_color: R::C::new(),
        }
    }

    pub fn new_owned(ren: R) -> Self {
        Self {
            m_ren: Equiv::Own(ren),
            m_color: R::C::new(),
        }
    }

    pub fn ren_mut(&mut self) -> &mut R {
        return &mut self.m_ren;
    }

    pub fn ren(&self) -> &R {
        return &self.m_ren;
    }
}

impl<'a, R: Renderer> RendererScanlineColor for RendererScanlineBinSolid<'a, R> {
    type C = R::C;

    fn set_color(&mut self, c: R::C) {
        self.m_color = c;
    }

    fn color(&self) -> R::C {
        self.m_color
    }
}

impl<'a, R: Renderer> RendererScanline for RendererScanlineBinSolid<'a, R> {
    fn prepare(&mut self) {}
    fn render<S: Scanline>(&mut self, sl: &S) {
        render_scanline_bin_solid(sl, &mut *self.m_ren, &self.m_color);
    }
}

//======================================================render_scanline_bin
pub fn render_scanline_bin<
    S: Scanline,
    Ren: Renderer,
    Sa: SpanAllocator<C = Ren::C>,
    Sg: SpanGenerator<C = Ren::C>,
>(
    sl: &S, ren: &mut Ren, alloc: &mut Sa, span_gen: &mut Sg,
) {
    let y = sl.y();
    let span = sl.begin();

    for s in span {
        let x = s.x;
        let mut len = s.len as i32;
        let covers = s.covers;

        if len < 0 {
            len = -len;
        }
        let colors = alloc.allocate(len as usize);
        span_gen.generate(colors, x, y, len as u32);
        let slice = unsafe { std::slice::from_raw_parts(s.covers, 0) };
        ren.blend_color_hspan(x, y, len, colors, slice, unsafe { *covers });
    }
}

//=====================================================render_scanlines_bin
pub fn render_scanlines_bin<
    R: RasterScanLine,
    S: Scanline,
    Ren: Renderer,
    Sa: SpanAllocator<C = Ren::C>,
    Sg: SpanGenerator<C = Ren::C>,
>(
    ras: &mut R, sl: &mut S, ren: &mut Ren, alloc: &mut Sa, span_gen: &mut Sg,
) {
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        span_gen.prepare();
        while ras.sweep_scanline(sl) {
            render_scanline_bin(sl, ren, alloc, span_gen);
        }
    }
}

//====================================================RendererScanlineBin
pub struct RendererScanlineBin<'a, Ren: Renderer, Sa: SpanAllocator, Sg: SpanGenerator> {
    m_ren: Equiv<'a, Ren>,
    m_alloc: Sa,
    m_span_gen: Equiv<'a, Sg>,
}

impl<'a, Ren: Renderer, Sa: SpanAllocator<C = Ren::C>, Sg: SpanGenerator<C = Ren::C>>
    RendererScanlineBin<'a, Ren, Sa, Sg>
{
    pub fn new_borrowed(alloc: Sa, ren: &'a mut Ren, span_gen: &'a mut Sg) -> Self {
        Self {
            m_ren: Equiv::Brw(ren),
            m_alloc: alloc,
            m_span_gen: Equiv::Brw(span_gen),
        }
    }

    pub fn new_owned(alloc: Sa, ren: Ren, span_gen: Sg) -> Self {
        Self {
            m_ren: Equiv::Own(ren),
            m_alloc: alloc,
            m_span_gen: Equiv::Own(span_gen),
        }
    }

    pub fn span_gen_mut(&mut self) -> &mut Sg {
        &mut self.m_span_gen
    }

    pub fn span_gen(&self) -> &Sg {
        &self.m_span_gen
    }

    pub fn ren_mut(&mut self) -> &mut Ren {
        return &mut self.m_ren;
    }

    pub fn ren(&self) -> &Ren {
        return &self.m_ren;
    }
}

impl<'a, Ren: Renderer, Sa: SpanAllocator<C = Ren::C>, Sg: SpanGenerator<C = Ren::C>>
    RendererScanline for RendererScanlineBin<'a, Ren, Sa, Sg>
{
    fn prepare(&mut self) {
        self.m_span_gen.prepare();
    }
    fn render<S: Scanline>(&mut self, sl: &S) {
        render_scanline_bin(
            sl,
            &mut *self.m_ren,
            &mut self.m_alloc,
            &mut *self.m_span_gen,
        );
    }
}
/*
impl<'a, Ren: Renderer, Sa: SpanAllocator<C = Ren::C>, Sg: SpanGenerator<C = Ren::C>>
    RendererScanlineColor for RendererScanlineBin<'a, Ren, Sa, Sg>
{

}*/

//========================================================render_scanlines
pub fn render_scanlines<R: RasterScanLine, S: Scanline, RenSl: RendererScanline>(
    ras: &mut R, sl: &mut S, rensl: &mut RenSl,
) {
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        rensl.prepare();
        while ras.sweep_scanline(sl) {
            rensl.render(sl);
        }
    }
}

//========================================================render_all_paths
pub fn render_all_paths<
    R: RasterScanLine,
    S: Scanline,
    Rensl: RendererScanlineColor,
    VS: VertexSource,
    CS: Index<usize, Output = Rensl::C>,
    PathId: Index<usize, Output = u32>,
>(
    ras: &mut R, sl: &mut S, rensl: &mut Rensl, vs: &mut VS, colors: &CS, path_id: &PathId,
    num_paths: u32,
) {
    for i in 0..num_paths as usize {
        ras.reset();
        ras.add_path(vs, path_id[i]);
        rensl.set_color(colors[i]);
        render_scanlines(ras, sl, rensl);
    }
}

// NOT TESTED
//=============================================render_scanlines_compound
pub fn render_scanlines_compound<
    R: Rasterizer0 + RasterScanLine,
    SAA: Scanline,
    SB: Scanline,
    B: Renderer,
    A: SpanAllocator<C = B::C>,
    SH: RasterStyle<B::C>,
>(
    ras: &mut R, sl_aa: &mut SAA, sl_bin: &mut SB, ren: &mut B, alloc: &mut A, sh: &mut SH,
) {
    if ras.rewind_scanlines() {
        let min_x = ras.min_x();
        let mut len = ras.max_x() - min_x + 2;
        sl_aa.reset(min_x, ras.max_x());
        sl_bin.reset(min_x, ras.max_x());

        let mut color_span = alloc.allocate(len as usize * 2);
        let cs_mix_buffer = len;
        let mut num_styles;
        let mut style;
        let mut solid;

        loop {
            num_styles = ras.sweep_styles();
            if num_styles <= 0 {
                break;
            }
            let mut span_aa;
            if num_styles == 1 {
                // Optimization for a single style. Happens often
                //-------------------------
                if ras.sweep_scanline_with_style(sl_aa, 0) {
                    style = ras.style(0);
                    if sh.is_solid(style) {
                        // Just solid fill
                        //-----------------------
                        render_scanline_aa_solid(sl_aa, ren, sh.color(style));
                    } else {
                        // Arbitrary span generator
                        //-----------------------
                        span_aa = sl_aa.begin();
                        for s in span_aa {
                            len = s.len;
                            sh.generate_span(&mut color_span, s.x, sl_aa.y(), len as u32, style);
                            let slice =
                                unsafe { std::slice::from_raw_parts(s.covers, s.len as usize) };
                            ren.blend_color_hspan(
                                s.x,
                                sl_aa.y(),
                                s.len,
                                &color_span,
                                slice,
                                CoverScale::FULL as u8,
                            );
                        }
                    }
                }
            } else {
                if ras.sweep_scanline_with_style(sl_bin, -1) {
                    // Clear the spans of the mix_buffer
                    //--------------------
                    let span_bin = sl_bin.begin();
                    for s in span_bin {
                        let st = (cs_mix_buffer + s.x - min_x) as usize;
                        color_span[st..st + s.len as usize].fill(B::C::new());
                    }

                    for i in 0..num_styles {
                        style = ras.style(i);
                        solid = sh.is_solid(style);

                        if ras.sweep_scanline_with_style(sl_aa, i as i32) {
                            let mut cs_colors;
                            //let mut cspan;
                            let mut covers;
                            span_aa = sl_aa.begin();
                            if solid {
                                // Just solid fill
                                //-----------------------
                                for s in span_aa {
                                    let c = sh.color(style);
                                    len = s.len;
                                    cs_colors = cs_mix_buffer + s.x - min_x;
                                    covers = s.covers;
                                    for _i in 0..len {
                                        if unsafe { *covers } == CoverScale::FULL as u8 {
                                            color_span[cs_colors as usize] = *c;
                                        } else {
                                            color_span[cs_colors as usize]
                                                .add(c, unsafe { *covers } as u32);
                                        }
                                        cs_colors += 1;
                                        unsafe {
                                            covers = covers.offset(1);
                                        }
                                        len -= 1;
										if len == 0 {
                                            break;
                                        }
                                    }
                                }
                            } else {
                                // Arbitrary span generator
                                //-----------------------
                                for s in span_aa {
                                    len = s.len;
                                    cs_colors = cs_mix_buffer + s.x - min_x;
                                    //cspan = color_span;
                                    sh.generate_span(
                                        &mut color_span,
                                        s.x,
                                        sl_aa.y(),
                                        len as u32,
                                        style,
                                    );
                                    covers = s.covers;
                                    for i in 0..len as usize {
                                        if unsafe { *covers } == CoverScale::FULL as u8 {
                                            color_span[cs_colors as usize] = color_span[i];
                                        } else {
                                            let c = color_span[i];
                                            color_span[cs_colors as usize]
                                                .add(&c, unsafe { *covers } as u32);
                                        }
                                        cs_colors += 1;
                                        unsafe {
                                            covers = covers.offset(1);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Emit the blended result as a color hspan
                    //-------------------------
                    let span_bin = sl_bin.begin();
                    for s in span_bin {
                        ren.blend_color_hspan(
                            s.x,
                            sl_bin.y(),
                            s.len,
                            &color_span[(cs_mix_buffer + s.x - min_x) as usize..],
                            &[],
                            CoverScale::FULL as u8,
                        );
                    }
                } // if(ras.sweep_scanline(sl_bin, -1))
            } // if(num_styles == 1) ... else
        } // while((num_styles = ras.sweep_styles()) > 0)
    } // if(ras.rewind_scanlines())
}

//=======================================render_scanlines_compound_layered
pub fn render_scanlines_compound_layered<
    R: Rasterizer0 + RasterScanLine,
    SAA: Scanline,
    B: Renderer,
    A: SpanAllocator<C = B::C>,
    SH: RasterStyle<B::C>,
>(
    ras: &mut R, sl_aa: &mut SAA, ren: &mut B, alloc: &mut A, sh: &mut SH,
) {
    if ras.rewind_scanlines() {
        let min_x = ras.min_x();
        let mut len = ras.max_x() - min_x + 2;
        sl_aa.reset(min_x, ras.max_x());

        let mut color_span = alloc.allocate(len as usize * 2);
        let cs_mix_buffer = len;
        let mut cover_buffer = vec![]; //ras.allocate_cover_buffer(len as u32);
        cover_buffer.resize(len as usize, 0);
        let cb_cover_buffer = 0;

        let mut num_styles;
        let mut style;
        let mut solid;
        loop {
            num_styles = ras.sweep_styles();
            if num_styles == 0 {
                break;
            }
            let mut span_aa;
            if num_styles == 1 {
                // Optimization for a single style. Happens often
                //-------------------------
                if ras.sweep_scanline_with_style(sl_aa, 0) {
                    style = ras.style(0);
                    if sh.is_solid(style) {
                        // Just solid fill
                        //-----------------------
                        render_scanline_aa_solid(sl_aa, ren, sh.color(style));
                    } else {
                        // Arbitrary span generator
                        //-----------------------
                        span_aa = sl_aa.begin();
                        for s in span_aa {
                            len = s.len;
                            sh.generate_span(&mut color_span, s.x, sl_aa.y(), len as u32, style);
                            let slice =
                                unsafe { std::slice::from_raw_parts(s.covers, s.len as usize) };
                            ren.blend_color_hspan(
                                s.x,
                                sl_aa.y(),
                                s.len,
                                &color_span,
                                slice,
                                CoverScale::FULL as u8,
                            );
                        }
                    }
                }
            } else {
                let sl_start = ras.scanline_start();
                let sl_len = ras.scanline_length();

                if sl_len != 0 {
					let i_cs = (cs_mix_buffer + sl_start - min_x) as usize;
                    color_span[i_cs..i_cs + sl_len as usize]
                        .fill(B::C::new());
                    let i_cb = (cb_cover_buffer + sl_start - min_x) as usize;
						cover_buffer[i_cb..i_cb + sl_len as usize]
                        .fill(0);
                    let mut sl_y = 0x7FFFFFFF;
                    for i in 0..num_styles {
                        style = ras.style(i);
                        solid = sh.is_solid(style);

                        if ras.sweep_scanline_with_style(sl_aa, i as i32) {
                            let mut cover;
                            let mut cs_colors;
                            let mut src_covers;
                            let mut dst_covers;
                            span_aa = sl_aa.begin();
                            sl_y = sl_aa.y();
                            if solid {
                                // Just solid fill
                                //-----------------------
                                for s in span_aa {
                                    let c = sh.color(style);
                                    len = s.len;
                                    cs_colors = cs_mix_buffer + s.x - min_x;
                                    src_covers = s.covers;
                                    dst_covers = (cb_cover_buffer + s.x - min_x) as usize;
                                    loop {
                                        cover = unsafe { *src_covers } as u32;
                                        if cover_buffer[dst_covers] as u32 + cover > CoverScale::FULL as u32
                                        {
                                            cover =
                                                CoverScale::FULL as u32 - cover_buffer[dst_covers] as u32;
                                        }
                                        if cover != 0 {
                                            color_span[cs_colors as usize].add(c, cover);
                                            cover_buffer[dst_covers] += cover as u8;
                                        }
                                        src_covers = unsafe { src_covers.offset(1) };
                                        dst_covers += 1;
                                        cs_colors += 1;
                                        len -= 1;
										if len == 0 {
                                            break;
                                        }
                                    }
                                }
                            } else {
                                // Arbitrary span generator
                                //-----------------------
                                for s in span_aa {
                                    len = s.len;
                                    cs_colors = cs_mix_buffer + s.x - min_x;
                                    //cspan = color_span;
                                    sh.generate_span(
                                        &mut color_span,
                                        s.x,
                                        sl_aa.y(),
                                        len as u32,
                                        style,
                                    );
                                    src_covers = s.covers;
                                    dst_covers = (cb_cover_buffer + s.x - min_x) as usize;
                                    for i in 0..len as usize {
                                        cover = unsafe { *src_covers } as u32;
                                        if cover_buffer[dst_covers] as u32 + cover > CoverScale::FULL as u32
                                        {
                                            cover =
                                                CoverScale::FULL as u32 - cover_buffer[dst_covers] as u32;
                                        }
                                        if cover != 0 {
                                            let c = color_span[i];
                                            color_span[cs_colors as usize].add(&c, cover);
                                            cover_buffer[dst_covers] += cover as u8;
                                        }
                                        src_covers = unsafe { src_covers.offset(1) };
                                        dst_covers += 1;
                                    }
                                }
                            }
                        }
                    }
                    ren.blend_color_hspan(
                        sl_start,
                        sl_y,
                        sl_len as i32,
                        &color_span[(cs_mix_buffer + sl_start - min_x) as usize..],
                        &[],
                        CoverScale::FULL as u8,
                    );
                } //if(sl_len)
            } //if(num_styles == 1) ... else
        } //while((num_styles = ras.sweep_styles()) > 0)
    } //if(ras.rewind_scanlines())
}
