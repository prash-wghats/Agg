//=============================================================scanline_p8
//
// This is a general purpose scaline container which supports the interface
// used in the rasterizer::render(). See description of scanline_u8
// for details.
//
//------------------------------------------------------------------------
use std::ptr::null_mut;
use crate::Scanline;
use crate::basics::Span;

#[derive(Clone)]
pub struct Span8 {
    pub x: i16,
    pub len: i16,
    pub icover: usize,
}
pub struct ScanlineP8 {
    last_x: i32,
    y: i32,
    spans: Vec<Span>,
    covers: Vec<u8>,
    cur_span: usize,
    cur_cover: usize,
}

impl ScanlineP8 {
    pub fn new() -> ScanlineP8 {
        ScanlineP8 {
            last_x: 0x7FFFFFF0,
            y: 0,
            spans: Vec::new(),
            covers: Vec::new(),
            cur_span: 0,
            cur_cover: 0,
        }
    }
}

impl Scanline for ScanlineP8 {
	type CoverType = u8;

    fn reset(&mut self, min_x: i32, max_x: i32) {
        let max_len = max_x - min_x + 3;
        if max_len > self.spans.len() as i32 {
			self.spans.clear();
            self.spans.resize(
                max_len as usize,
                Span {
                    x: 0,
                    len: 0,
                    covers: null_mut(),
                },
            );
			self.covers.clear();
            self.covers.resize(max_len as usize, 0);
        }
        self.last_x = 0x7FFFFFF0;
        self.cur_span = 0;
		self.cur_cover = 0;
    }

    fn add_cell(&mut self, x: i32, cover: u32) {
        self.covers[self.cur_cover] = cover as u8;
        if x == self.last_x + 1  && self.spans[self.cur_span].len > 0 {
            self.spans[self.cur_span].len += 1;
        } else {
			self.cur_span += 1;
            self.spans[self.cur_span].x = x;
            self.spans[self.cur_span].len = 1;
            self.spans[self.cur_span].covers = &self.covers[self.cur_cover] as *const u8 as *mut u8;
        }
		
        self.cur_cover += 1;
        self.last_x = x;
    }

    fn add_cells(&mut self, x: i32, len: u32, covers: &[u8]) {
        for i in 0..len as usize {
            self.covers[self.cur_cover + i] = covers[i];//unsafe { *covers.offset(i as isize) };
        }
        if x == self.last_x + 1 && self.spans[self.cur_span].len > 0 {
            self.spans[self.cur_span].len += len as i32;
        } else {
            self.cur_span += 1;
            self.spans[self.cur_span].x = x;
            self.spans[self.cur_span].len = len as i32;
            self.spans[self.cur_span].covers = &self.covers[self.cur_cover] as *const u8 as *mut u8;
        }
        self.cur_cover += len as usize;
        self.last_x = x + len as i32 - 1;
    }

    fn add_span(&mut self, x: i32, len: u32, cover: u32) {
        if x == self.last_x + 1
            && self.spans[self.cur_span].len < 0
            && unsafe { *self.spans[self.cur_span].covers} == cover as u8
        {
            self.spans[self.cur_span].len -= len as i32;
        } else {
            self.covers[self.cur_cover] = cover as u8;
            self.cur_span += 1;
            self.spans[self.cur_span].x = x;
            self.spans[self.cur_span].len = -(len as i32);
            self.spans[self.cur_span].covers = &self.covers[self.cur_cover] as *const u8 as *mut u8;
            self.cur_cover += 1;
        }
        self.last_x = x + len as i32 - 1;
    }

    fn finalize(&mut self, y: i32) {
        self.y = y;
    }

    fn reset_spans(&mut self) {
        self.last_x = 0x7FFFFFF0;
        self.cur_span = 0;
        self.cur_cover = 0;
		self.spans[self.cur_span].len = 0;
    }

    fn y(&self) -> i32 {
        self.y
    }

    fn num_spans(&self) -> u32 {
        self.cur_span as u32
    }

    fn begin(&self) -> &[Span] {
        &self.spans[1..=self.cur_span]
    }
}
/* 
/////////
#[derive(Clone)]
pub struct spanp32 {
    pub x: i16,
    pub len: i16,
    pub icover: usize,
}
pub struct Scanline32P8 {
    last_x: i32,
    y: i32,
    spans: pod_bvector<Span>,
    covers: Vec<u8>,
    cur_span: usize,
    cur_cover: usize,
}

impl Scanline32P8 {
    pub fn new() -> Scanline32P8 {
        Scanline32P8 {
            last_x: 0x7FFFFFF0,
            y: 0,
            spans: Vec::new(),
            covers: Vec::new(),
            cur_span: 0,
            cur_cover: 0,
        }
    }

    pub fn reset(&mut self, min_x: i32, max_x: i32) {
        let max_len = max_x - min_x + 3;
        if max_len > self.spans.len() as i32 {
			self.covers.resize(max_len as usize, 0);
        }
		self.spans.clear();
        self.last_x = 0x7FFFFFF0;
        self.cur_span = 0;
		self.cur_cover = 0;
    }

    pub fn add_cell(&mut self, x: i32, cover: u32) {
        self.covers[self.cur_cover] = cover as u8;
        if x == self.last_x + 1 && self.spans.len() > 0 && self.spans[self.spans.len() - 1].len > 0 {
            self.spans[self.spans.len() - 1].len += 1;
        } else {
            self.cur_span += 1;
            self.spans[self.cur_span].x = x;
            self.spans[self.cur_span].len = 1;
            self.spans[self.cur_span].covers = &self.covers[self.cur_cover] as *const u8 as *mut u8;
        }
        self.cur_cover += 1;
        self.last_x = x;
    }

    pub fn add_cells(&mut self, x: i32, len: u32, covers: *const u8) {
        for i in 0..len as usize {
            self.covers[self.cur_cover + i] = unsafe { *covers.offset(i as isize) };
        }
        if x == self.last_x + 1 && self.spans[self.cur_span].len > 0 {
            self.spans[self.cur_span].len += len as i16;
        } else {
            self.cur_span += 1;
            self.spans[self.cur_span].x = x as i16;
            self.spans[self.cur_span].len = len as i16;
            self.spans[self.cur_span].icover = self.cur_cover;
        }
        self.cur_cover += len as usize;
        self.last_x = x + len as i32 - 1;
    }

    pub fn add_span(&mut self, x: i32, len: u32, cover: u32) {
        if x == self.last_x + 1
            && self.cur_span > 0
            && self.spans[self.cur_span].len < 0
            && self.covers[self.spans[self.cur_span].icover] == cover as u8
        {
            self.spans[self.cur_span].len -= len as i16;
        } else {
            self.covers[self.cur_cover] = cover as u8;
            self.cur_span += 1;
            self.spans[self.cur_span].x = x as i16;
            self.spans[self.cur_span].len -= len as i16;
            self.spans[self.cur_span].icover = self.cur_cover;
            self.cur_cover += len as usize;
        }
        self.last_x = x + len as i32 - 1;
    }

    pub fn finalize(&mut self, y: i32) {
        self.y = y;
    }

    pub fn reset_spans(&mut self) {
        self.last_x = 0x7FFFFFF0;
        self.cur_span = 0;
        self.cur_cover = 0;
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn num_spans(&self) -> usize {
        self.cur_span
    }

    pub fn begin(&self) -> &[spanp32] {
        &self.spans[1..]
    }
}
*/