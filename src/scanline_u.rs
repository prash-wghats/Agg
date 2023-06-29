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
// Adaptation for 32-bit screen coordinates (scanline32_u) has been sponsored by
// Liberty Technology Systems, Inc., visit http://lib-sys.com
//
// Liberty Technology Systems, Inc. is the provider of
// PostScript and PDF technology for software developers.
//
//----------------------------------------------------------------------------

//=============================================================ScanlineU8
//
// Unpacked scanline container class
//
// This class is used to transfer data from a scanline rasterizer
// to the rendering buffer. It's organized very simple. The class stores
// information of horizontal spans to render it into a pixel-map buffer.
// Each Span has staring X, length, and an array of bytes that determine the
// cover-values for each pixel.
// Before using this class you should know the minimal and maximal pixel
// coordinates of your scanline. The protocol of using is:
// 1. reset(min_x, max_x)
// 2. add_cell() / add_span() - accumulate scanline.
//    When forming one scanline the next X coordinate must be always greater
//    than the last stored one, i.e. it works only with ordered coordinates.
// 3. Call finalize(y) and render the scanline.
// 3. Call reset_spans() to prepare for the new scanline.
//
// 4. Rendering:
//
// Scanline provides an iterator class that allows you to extract
// the spans and the cover values for each pixel. Be aware that clipping
// has not been done yet, so you should perform it yourself.
// Use ScanlineU8::iterator to render spans:
//-------------------------------------------------------------------------
//
// int y = sl.y();                    // Y-coordinate of the scanline
//
// ************************************
// ...Perform vertical clipping here...
// ************************************
//
// ScanlineU8::const_iterator Span = sl.begin();
//
// unsigned char* row = m_rbuf->row(y); // The the address of the beginning
//                                      // of the current row
//
// unsigned num_spans = sl.num_spans(); // Number of spans. It's guaranteed that
//                                      // num_spans is always greater than 0.
//
// do
// {
//     const ScanlineU8::CoverType* covers =
//         Span->covers;                     // The array of the cover values
//
//     int num_pix = Span->len;              // Number of pixels of the Span.
//                                           // Always greater than 0, still it's
//                                           // better to use "int" instead of
//                                           // "unsigned" because it's more
//                                           // convenient for clipping
//     int x = Span->x;
//
//     **************************************
//     ...Perform horizontal clipping here...
//     ...you have x, covers, and pix_count..
//     **************************************
//
//     unsigned char* dst = row + x;  // Calculate the start address of the row.
//                                    // In this case we assume a simple
//                                    // grayscale image 1-byte per pixel.
//     do
//     {
//         *dst++ = *covers++;        // Hypotetical rendering.
//     }
//     while(--num_pix);
//
//     ++Span;
// }
// while(--num_spans);  // num_spans cannot be 0, so this loop is quite safe
//------------------------------------------------------------------------
//
// The question is: why should we accumulate the whole scanline when we
// could render just separate spans when they're ready?
// That's because using the scanline is generally faster. When is consists
// of more than one Span the conditions for the processor cash system
// are better, because switching between two different areas of memory
// (that can be very large) occurs less frequently.
//------------------------------------------------------------------------

use crate::basics::Span;
use crate::{ Scanline, AlphaMask };

pub struct ScanlineU8 {
    m_min_x: i32,
    m_last_x: i32,
    m_y: i32,
    m_covers: Vec<u8>,
    m_spans: Vec<Span>,
    m_cur_span: *mut Span,
}

//--------------------------------------------------------------------
impl ScanlineU8 {
    pub fn new() -> Self {
        Self {
            m_min_x: 0,
            m_last_x: 0x7FFFFFF0,
            m_y: 0,
            m_cur_span: std::ptr::null_mut(),
            m_covers: Vec::<u8>::new(),
            m_spans: Vec::<Span>::new(),
        }
    }
}

impl Scanline for ScanlineU8 {
    type CoverType = u8;
    fn y(&self) -> i32 {
        self.m_y
    }

    fn num_spans(&self) -> u32 {
        unsafe { self.m_cur_span.offset_from(self.m_spans.as_ptr()) as u32 }
    }

    fn begin(&self) -> &[Span] {
        &self.m_spans[0..(self.num_spans() as usize)]
    }

    fn reset(&mut self, min_x: i32, max_x: i32) {
        let max_len: usize = (max_x - min_x + 2) as usize;

        if max_len > self.m_spans.len() {
            self.m_spans.clear();
            self.m_spans.resize(
                max_len - self.m_spans.len(),
                Span {
                    x: 0,
                    len: 0,
                    covers: std::ptr::null_mut(),
                },
            );
            self.m_covers.clear();
            self.m_covers.resize(max_len - self.m_covers.len(), 0);
        }
        self.m_last_x = 0x7FFFFFF0;
        self.m_min_x = min_x;
        self.m_cur_span = self.m_spans.as_mut_ptr() as *mut Span; //&mut self.m_spans[0];
    }

    fn reset_spans(&mut self) {
        self.m_last_x = 0x7FFFFFF0;
        self.m_cur_span = self.m_spans.as_mut_ptr() as *mut Span; //&mut self.m_spans[0];
    }

    fn add_cell(&mut self, x: i32, cover: u32) {
        let x = x - self.m_min_x;
        self.m_covers[x as usize] = cover as u8;

        unsafe {
            if x == self.m_last_x + 1 {
                (*self.m_cur_span.offset(-1)).len += 1;
            } else {
                //self.m_cur_span = self.m_cur_span.offset(1);
                (*self.m_cur_span).x = x + self.m_min_x;
                (*self.m_cur_span).len = 1;
                (*self.m_cur_span).covers = &mut self.m_covers[x as usize];
                self.m_cur_span = self.m_cur_span.offset(1);
            }
        }
        self.m_last_x = x;
    }

    fn add_cells(&mut self, x: i32, len: u32, covers: &[u8]) {
        let x = x - self.m_min_x;
        let covers = &covers[0];
        unsafe {
            std::ptr::copy_nonoverlapping(
                covers as *const u8,
                &mut self.m_covers[x as usize] as *mut u8,
                len as usize,
            );
        }
        unsafe {
            if x == self.m_last_x + 1 {
                (*self.m_cur_span.offset(-1)).len += len as i32;
            } else {
                //self.m_cur_span = self.m_cur_span.offset(1);
                (*self.m_cur_span).x = x + self.m_min_x;
                (*self.m_cur_span).len = len as i32;
                (*self.m_cur_span).covers = &mut self.m_covers[x as usize];
                self.m_cur_span = self.m_cur_span.offset(1);
            }
        }
        self.m_last_x = x + len as i32 - 1;
    }

    fn add_span(&mut self, x: i32, len: u32, cover: u32) {
        let x = x - self.m_min_x;
        for i in 0..len {
            self.m_covers[x as usize + i as usize] = cover as u8;
        }
        unsafe {
            if x == self.m_last_x + 1 {
                (*self.m_cur_span.offset(-1)).len += len as i32;
            } else {
                //self.m_cur_span = self.m_cur_span.offset(1);
                (*self.m_cur_span).x = x as i32 + self.m_min_x;
                (*self.m_cur_span).len = len as i32;
                (*self.m_cur_span).covers = &mut self.m_covers[x as usize];
                self.m_cur_span = self.m_cur_span.offset(1);
            }
        }
        self.m_last_x = x + len as i32 - 1;
    }

    fn finalize(&mut self, y: i32) {
        self.m_y = y;
    }
}


pub struct ScanlineU8AM<'a, AM: AlphaMask<CoverType = u8>> {
    base: ScanlineU8,
    alpha_mask: &'a mut AM,
}

impl<'a, AM: AlphaMask<CoverType = u8>> ScanlineU8AM<'a, AM> {
    pub fn new(am: &'a mut AM) -> Self {
        Self {
            base: ScanlineU8::new(),
            alpha_mask: am,
        }
    }
}
impl<'a, AM: AlphaMask<CoverType = u8>> Scanline for ScanlineU8AM<'a, AM> {
	// NOT TESTED
	//TODO - Deref to ScanlineU8
	type CoverType = u8;
    fn y(&self) -> i32 { self.base.y() }
    fn num_spans(&self) -> u32 { self.base.num_spans() }
    fn begin(&self) -> &[Span] { self.base.begin() }
    fn reset(&mut self, min_x: i32, max_x: i32) { self.base.reset(min_x, max_x) }
    fn reset_spans(&mut self) { self.base.reset_spans() }
    fn add_cell(&mut self, x: i32, cover: u32) { self.base.add_cell(x, cover) }
    fn add_span(&mut self, x: i32, len: u32, cover: u32) { self.base.add_span(x, len, cover) }
    fn add_cells(&mut self, x: i32, len: u32, covers: &[Self::CoverType]) { self.base.add_cells(x, len, covers) }

    fn finalize(&mut self, span_y: i32) {
        self.base.finalize(span_y);
        let span = self.base.begin();
        for s in span {
            self.alpha_mask
                .combine_hspan(s.x, self.base.y(), 
				unsafe { std::slice::from_raw_parts_mut(s.covers, s.len as usize) }, s.len);
        }
    }
}
