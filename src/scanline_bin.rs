use crate::{basics::Span, Scanline};

//=============================================================scanline_bin
//
// This is binary scaline container which supports the interface
// used in the rasterizer::render(). See description of agg_scanline_u8
// for details.
//
//------------------------------------------------------------------------

pub struct ScanlineBin {
    last_x: i32,
    y: i32,
    spans: Vec<Span>,
    cur_span: usize,
}

impl ScanlineBin {
    pub fn new() -> ScanlineBin {
        ScanlineBin {
            last_x: 0x7FFFFFF0,
            y: 0,
            spans: Vec::new(),
            cur_span: 0,
        }
    }
}

impl Scanline for ScanlineBin {
    type CoverType = u8;

    fn reset(&mut self, min_x: i32, max_x: i32) {
        let max_len = max_x - min_x + 3;
        if max_len > self.spans.len() as i32 {
            self.spans.resize(
                max_len as usize,
                Span {
                    x: 0,
                    len: 0,
                    covers: std::ptr::null_mut(),
                },
            );
        }
        self.last_x = 0x7FFFFFF0;
        self.cur_span = 0;
    }

    fn add_cell(&mut self, x: i32, _: u32) {
        if x == self.last_x + 1 {
            self.spans[self.cur_span].len += 1;
        } else {
            self.cur_span += 1;
            self.spans[self.cur_span].x = x;
            self.spans[self.cur_span].len = 1;
        }
        self.last_x = x;
    }

    fn add_span(&mut self, x: i32, len: u32, _: u32) {
        if x == self.last_x + 1 {
            self.spans[self.cur_span].len += len as i32;
        } else {
            self.cur_span += 1;
            self.spans[self.cur_span].x = x;
            self.spans[self.cur_span].len = len as i32;
        }
        self.last_x = x + len as i32 - 1;
    }

    fn add_cells(&mut self, x: i32, len: u32, _: &[u8]) {
        self.add_span(x, len, 0);
    }

    fn finalize(&mut self, y: i32) {
        self.y = y;
    }

    fn reset_spans(&mut self) {
        self.last_x = 0x7FFFFFF0;
        self.cur_span = 0;
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

////////////
#[derive(Clone)]
pub struct SpanI32 {
    pub x: i32,
    pub len: i32,
}
pub struct Scanline32Bin {
    last_x: i32,
    y: i32,
    spans: Vec<SpanI32>,
    cur_span: usize,
}

impl Scanline32Bin {
    pub fn new() -> Scanline32Bin {
        Scanline32Bin {
            last_x: 0x7FFFFFF0,
            y: 0,
            spans: Vec::new(),
            cur_span: 0,
        }
    }

    pub fn reset(&mut self, min_x: i32, max_x: i32) {
        let max_len = max_x - min_x + 3;
        if max_len > self.spans.len() as i32 {
            self.spans
                .resize(max_len as usize, SpanI32 { x: 0, len: 0 });
        }
        self.last_x = 0x7FFFFFF0;
        self.cur_span = 0;
    }

    pub fn add_cell(&mut self, x: i32, _: u32) {
        if x == self.last_x + 1 {
            self.spans[self.cur_span].len += 1;
        } else {
            self.cur_span += 1;
            self.spans[self.cur_span].x = x;
            self.spans[self.cur_span].len = 1;
        }
        self.last_x = x;
    }

    pub fn add_span(&mut self, x: i32, len: u32, _: u32) {
        if x == self.last_x + 1 {
            self.spans[self.cur_span].len += len as i32;
        } else {
            self.cur_span += 1;
            self.spans[self.cur_span].x = x;
            self.spans[self.cur_span].len = len as i32;
        }
        self.last_x = x + len as i32 - 1;
    }

    pub fn add_cells(&mut self, x: i32, len: u32, _: *const SpanI32) {
        self.add_span(x, len, 0);
    }

    pub fn finalize(&mut self, y: i32) {
        self.y = y;
    }

    pub fn reset_spans(&mut self) {
        self.last_x = 0x7FFFFFF0;
        self.cur_span = 0;
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn num_spans(&self) -> usize {
        self.cur_span
    }

    pub fn begin(&self) -> &[SpanI32] {
        &self.spans[1..]
    }
}
