use crate::basics::{iround, CoverScale, Span};
use crate::{RasterScanLine, RendererScanline, Scanline};
use wrapping_arithmetic::wrappit;

#[derive(Clone, Copy)]
pub struct SpanData {
    x: i32,
    len: i32,
}

impl SpanData {
    pub fn new() -> Self {
        Self { x: 0, len: 0 }
    }
}

#[derive(Clone, Copy)]
pub struct ScanlineData {
    y: i32,
    num_spans: u32,
    start_span: u32,
}
impl ScanlineData {
    pub fn new() -> Self {
        Self {
            y: 0,
            num_spans: 0,
            start_span: 0,
        }
    }
}

pub struct ScanlineStorageBin {
    spans: Vec<SpanData>,
    scanlines: Vec<ScanlineData>,
    fake_span: SpanData,
    fake_scanline: ScanlineData,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    cur_scanline: usize,
}

impl ScanlineStorageBin {
    pub fn new() -> ScanlineStorageBin {
        ScanlineStorageBin {
            spans: Vec::with_capacity(256 - 2),
            scanlines: Vec::new(),
            fake_span: SpanData::new(),
            fake_scanline: ScanlineData::new(),
            min_x: 0x7FFFFFFF,
            min_y: 0x7FFFFFFF,
            max_x: -0x7FFFFFFF,
            max_y: -0x7FFFFFFF,
            cur_scanline: 0,
        }
    }
    pub fn byte_size(&self) -> usize {
        let mut size = 4 * 4; // min_x, min_y, max_x, max_y

        for i in 0..self.scanlines.len() {
            size += 2 * 4 + // Y, num_spans
				self.scanlines[i].num_spans as usize * 2 * 4; // X, span_len
        }
        size
    }

    pub fn write_int32(dst: &mut [u8], val: i32) {
        dst[0] = ((val >> 0) & 0xFF) as u8;
        dst[1] = ((val >> 8) & 0xFF) as u8;
        dst[2] = ((val >> 16) & 0xFF) as u8;
        dst[3] = ((val >> 24) & 0xFF) as u8;
    }

    pub fn serialize(&self, data: &mut [u8]) {
        let mut i = 0;

        ScanlineStorageBin::write_int32(&mut data[i..], self.min_x()); // min_x
        i += 4;
        ScanlineStorageBin::write_int32(&mut data[i..], self.min_y()); // min_y
        i += 4;
        ScanlineStorageBin::write_int32(&mut data[i..], self.max_x()); // max_x
        i += 4;
        ScanlineStorageBin::write_int32(&mut data[i..], self.max_y()); // max_y
        i += 4;

        for j in 0..self.scanlines.len() {
            let sl_this = &self.scanlines[j];

            ScanlineStorageBin::write_int32(&mut data[i..], sl_this.y); // Y
            i += 4;

            ScanlineStorageBin::write_int32(&mut data[i..], sl_this.num_spans as i32); // num_spans
            i += 4;

            let mut num_spans = sl_this.num_spans;
            let mut span_idx = sl_this.start_span;
            loop {
                let sp = &self.spans[span_idx as usize];

                ScanlineStorageBin::write_int32(&mut data[i..], sp.x); // X
                i += 4;

                ScanlineStorageBin::write_int32(&mut data[i..], sp.len); // len
                i += 4;

                if num_spans == 1 {
                    break;
                }
                num_spans -= 1;
                span_idx += 1;
            }
        }
    }

    pub fn scanline_by_index(&self, i: usize) -> &ScanlineData {
        if i < self.scanlines.len() {
            &self.scanlines[i]
        } else {
            &self.fake_scanline
        }
    }

    pub fn span_by_index(&self, i: usize) -> &SpanData {
        if i < self.spans.len() {
            &self.spans[i]
        } else {
            &self.fake_span
        }
    }
}

impl RasterScanLine for ScanlineStorageBin {
    fn min_x(&self) -> i32 {
        self.min_x
    }

    fn min_y(&self) -> i32 {
        self.min_y
    }

    fn max_x(&self) -> i32 {
        self.max_x
    }

    fn max_y(&self) -> i32 {
        self.max_y
    }

    fn rewind_scanlines(&mut self) -> bool {
        self.cur_scanline = 0;
        self.scanlines.len() > 0
    }

    fn sweep_scanline<S: Scanline>(&mut self, sl: &mut S) -> bool {
        sl.reset_spans();
        loop {
            if self.cur_scanline >= self.scanlines.len() {
                return false;
            }
            let sl_this = &self.scanlines[self.cur_scanline];

            let mut num_spans = sl_this.num_spans;
            let mut span_idx = sl_this.start_span;
            loop {
                let sp = &self.spans[span_idx as usize];
                sl.add_span(sp.x, sp.len as u32, CoverScale::FULL as u32);
                span_idx += 1;
                if num_spans == 1 {
                    break;
                }
                num_spans -= 1;
            }

            self.cur_scanline += 1;
            if sl.num_spans() > 0 {
                sl.finalize(sl_this.y);
                break;
            }
        }
        true
    }
}


impl RendererScanline for ScanlineStorageBin {
    fn prepare(&mut self) {
        self.scanlines.clear();
        self.spans.clear();
        self.min_x = 0x7FFFFFFF;
        self.min_y = 0x7FFFFFFF;
        self.max_x = -0x7FFFFFFF;
        self.max_y = -0x7FFFFFFF;
        self.cur_scanline = 0;
    }

    fn render<S: Scanline>(&mut self, sl: &S) {
        let mut sl_this = ScanlineData::new();

        let y = sl.y();
        if y < self.min_y {
            self.min_y = y;
        }
        if y > self.max_y {
            self.max_y = y;
        }

        sl_this.y = y;
        sl_this.num_spans = sl.num_spans();
        sl_this.start_span = self.spans.len() as u32;
        let span_iterator = sl.begin();
        //let mut j = 0;
        //let mut num_spans = sl_this.num_spans;
        for s in span_iterator {
            let mut sp = SpanData::new();
            sp.x = s.x;
            sp.len = s.len.abs();
            self.spans.push(sp);
            let x1 = sp.x;
            let x2 = sp.x + sp.len - 1;
            if x1 < self.min_x {
                self.min_x = x1;
            }
            if x2 > self.max_x {
                self.max_x = x2;
            }
        }
        self.scanlines.push(sl_this);
    }
}

//---------------------------------------SerializedScanlinesAdaptorBin
pub struct SerializedScanlinesAdaptorBin {
    m_data: *const u8,
    m_end: *const u8,
    m_ptr: *const u8,
    m_dx: i32,
    m_dy: i32,
    m_min_x: i32,
    m_min_y: i32,
    m_max_x: i32,
    m_max_y: i32,
}

impl SerializedScanlinesAdaptorBin {
    pub fn new() -> SerializedScanlinesAdaptorBin {
        SerializedScanlinesAdaptorBin {
            m_data: 0 as *const u8,
            m_end: 0 as *const u8,
            m_ptr: 0 as *const u8,
            m_dx: 0,
            m_dy: 0,
            m_min_x: 0x7FFFFFFF,
            m_min_y: 0x7FFFFFFF,
            m_max_x: -0x7FFFFFFF,
            m_max_y: -0x7FFFFFFF,
        }
    }

    pub fn init(&mut self, data: *const u8, size: usize, dx: f64, dy: f64) {
        self.m_data = data;
        self.m_end = unsafe { data.offset(size as isize) };
        self.m_ptr = data;
        self.m_dx = iround(dx);
        self.m_dy = iround(dy);
        self.m_min_x = 0x7FFFFFFF;
        self.m_min_y = 0x7FFFFFFF;
        self.m_max_x = -0x7FFFFFFF;
        self.m_max_y = -0x7FFFFFFF;
    }

    fn read_int32(&mut self) -> i32 {
        let mut val: i32 = 0;
        unsafe {
            let ptr = &mut val as *mut i32 as *mut u8;
            *ptr = *self.m_ptr;
            *ptr.offset(1) = *self.m_ptr.offset(1);
            *ptr.offset(2) = *self.m_ptr.offset(2);
            *ptr.offset(3) = *self.m_ptr.offset(3);
            self.m_ptr = self.m_ptr.offset(4);
            //val = *(ptr as *const i32);
        }
        val
    }
}

// Iterate scanlines interface
impl RasterScanLine for SerializedScanlinesAdaptorBin {
    #[wrappit]
    fn rewind_scanlines(&mut self) -> bool {
        self.m_ptr = self.m_data;
        if self.m_ptr < self.m_end {
            self.m_min_x = self.read_int32() + self.m_dx;
            self.m_min_y = self.read_int32() + self.m_dy;
            self.m_max_x = self.read_int32() + self.m_dx;
            self.m_max_y = self.read_int32() + self.m_dy;
        }
        self.m_ptr < self.m_end
    }

    fn min_x(&self) -> i32 {
        self.m_min_x
    }
    fn min_y(&self) -> i32 {
        self.m_min_y
    }
    fn max_x(&self) -> i32 {
        self.m_max_x
    }
    fn max_y(&self) -> i32 {
        self.m_max_y
    }

    fn sweep_scanline<Sl: Scanline>(&mut self, sl: &mut Sl) -> bool {
        sl.reset_spans();
        loop {
            if self.m_ptr >= self.m_end {
                return false;
            }

            let y = self.read_int32() + self.m_dy;
            let mut num_spans = self.read_int32();

            loop {
                if num_spans == 0 {
                    break;
                }

                let x = self.read_int32() + self.m_dx;
                let mut len = self.read_int32();

                if len < 0 {
                    len = -len;
                }
                sl.add_span(x, len as u32, CoverScale::FULL as u32);
                num_spans -= 1;
            }

            if sl.num_spans() != 0 {
                sl.finalize(y);
                break;
            }
        }
        true
    }
}

#[derive(Clone)]
pub struct EmbeddedScanline {
    y: i32,
    spans: Vec<Span>,
}

impl EmbeddedScanline {
    pub fn new() -> Self {
        Self {
            y: 0,
            spans: Vec::new(),
        }
    }
}

impl Scanline for EmbeddedScanline {
    type CoverType = u8;

    fn reset(&mut self, _: i32, _: i32) {
        self.reset_spans();
    }
    fn num_spans(&self) -> u32 {
        self.spans.len() as u32
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn begin(&self) -> &[Span] {
        &self.spans
    }

    fn add_cell(&mut self, _x: i32, _cover: u32) {}
    fn reset_spans(&mut self) {
        self.spans.clear();
    }
    fn add_cells(&mut self, _x: i32, _len: u32, _covers: &[Self::CoverType]) {}
    fn add_span(&mut self, x: i32, len: u32, _cover: u32) {
        self.spans.push(Span {
            x: x,
            len: len as i32,
            covers: std::ptr::null_mut(),
        });
    }
    fn finalize(&mut self, y: i32) {
        self.y = y;
    }
}
