use crate::basics::{iround, Span};
use crate::{AggInteger, RasterScanLine, RendererScanline, Scanline};
use std::marker::PhantomData;
use std::{mem, ptr};
use wrapping_arithmetic::wrappit;

pub type ScanlineStorageAA8 = ScanlineStorageAA<u8>;
pub type ScanlineStorageAA16 = ScanlineStorageAA<u16>;
pub type ScanlineStorageAA32 = ScanlineStorageAA<u32>;

#[derive(Debug, Clone, Copy)]
pub struct SpanData {
    pub x: i32,
    pub len: i32,
    pub covers_id: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct ScanlineData {
    pub y: i32,
    pub num_spans: u32,
    pub start_span: u32,
}

pub struct ScanlineStorageAA<T: AggInteger> {
    pub covers: Vec<T>,
    pub spans: Vec<SpanData>,
    pub scanlines: Vec<ScanlineData>,
    pub fake_span: SpanData,
    pub fake_scanline: ScanlineData,
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub cur_scanline: usize,
}

impl<T: AggInteger> ScanlineStorageAA<T> {
    pub fn new() -> Self {
        Self {
            covers: Vec::new(),
            spans: Vec::with_capacity(256 - 2),
            scanlines: Vec::with_capacity(8),
            fake_span: SpanData {
                x: 0,
                len: 0,
                covers_id: 0,
            },
            fake_scanline: ScanlineData {
                y: 0,
                num_spans: 0,
                start_span: 0,
            },
            min_x: 0x7FFFFFFF,
            min_y: 0x7FFFFFFF,
            max_x: -0x7FFFFFFF,
            max_y: -0x7FFFFFFF,
            cur_scanline: 0,
        }
    }
    pub fn write_int32(p: *mut u8, val: i32) -> *mut u8 {
        let mut p = p;
        let dst = unsafe { std::slice::from_raw_parts_mut(p, 4) };
        dst[0] = ((val >> 0) & 0xFF) as u8;
        dst[1] = ((val >> 8) & 0xFF) as u8;
        dst[2] = ((val >> 16) & 0xFF) as u8;
        dst[3] = ((val >> 24) & 0xFF) as u8;
        unsafe {
            p = p.offset(4);
        }
        p
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

    pub fn covers_by_index(&self, i: usize) -> &[T] {
        let len = self.covers.len();
        &self.covers[i..len]
    }

    pub fn byte_size(&self) -> usize {
        let mut i = 0;
        let mut size = mem::size_of::<i32>() * 4; // min_x, min_y, max_x, max_y

        while i < self.scanlines.len() {
            size += mem::size_of::<i32>() * 3; // scanline size in bytes, Y, num_spans

            let sl_this = &self.scanlines[i];

            let mut num_spans = sl_this.num_spans;
            let mut span_idx = sl_this.start_span;
            loop {
                let sp = &self.spans[span_idx as usize];

                size += mem::size_of::<i32>() * 2; // X, span_len
                if sp.len < 0 {
                    size += mem::size_of::<T>(); // cover
                } else {
                    size += mem::size_of::<T>() * sp.len as usize; // covers
                }
                if num_spans == 1 {
                    break;
                }
                span_idx += 1;
                num_spans -= 1;
            }
            i += 1;
        }
        size
    }

    pub fn serialize(&self, data: &mut [u8]) {
        let mut data = data.as_mut_ptr();
        //let mut i = 0;

        data = Self::write_int32(data, self.min_x()); // min_x
        data = Self::write_int32(data, self.min_y()); // min_y
        data = Self::write_int32(data, self.max_x()); // max_x
        data = Self::write_int32(data, self.max_y()); // max_y

        for i in 0..self.scanlines.len() {
            let sl_this = &self.scanlines[i];

            let size_ptr = data;
            data = unsafe { data.offset(mem::size_of::<i32>() as isize) }; // Reserve space for scanline size in bytes
            data = Self::write_int32(data, sl_this.y); // Y
            data = Self::write_int32(data, sl_this.num_spans as i32); // num_spans

            let mut num_spans = sl_this.num_spans;
            let mut span_idx = sl_this.start_span;
            while num_spans > 0 {
                unsafe {
                    let sp = &self.spans[span_idx as usize];
                    let covers = self.covers.as_ptr().offset(sp.covers_id as isize);

                    data = Self::write_int32(data, sp.x); // X
                    data = Self::write_int32(data, sp.len); // span_len

                    if sp.len < 0 {
                        std::ptr::copy_nonoverlapping(covers as *mut u8, data, mem::size_of::<T>());
                        data = data.offset(mem::size_of::<T>() as isize);
                    } else {
                        std::ptr::copy_nonoverlapping(
                            covers as *mut u8,
                            data,
                            mem::size_of::<T>() * sp.len as usize,
                        );
                        data = data.offset((mem::size_of::<T>() * sp.len as usize) as isize);
                    }
                    span_idx += 1;
                    num_spans -= 1;
                }
            }
            Self::write_int32(size_ptr, unsafe { data.offset_from(size_ptr) as i32 });
        }
    }
}

impl<T: AggInteger> RasterScanLine for ScanlineStorageAA<T> {
    // Iterate scanlines interface
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

    fn sweep_scanline<Sl: Scanline>(&mut self, sl: &mut Sl) -> bool {
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
                let covers = self.covers_by_index(sp.covers_id as usize);
                if sp.len < 0 {
                    sl.add_span(sp.x, -sp.len as u32, covers[0].into_u32());
                } else {
                    let len = (mem::size_of::<Sl::CoverType>() as f64 / mem::size_of::<T>() as f64)
                        as usize
                        * covers.len();
                    let tmp = unsafe {
                        std::slice::from_raw_parts(
                            covers.as_ptr() as *const Sl::CoverType,
                            len,
                        )
                    };
                    sl.add_cells(sp.x, sp.len as u32, tmp);
                }
                if num_spans == 1 {
                    break;
                }
                span_idx += 1;
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

impl<T: AggInteger> RendererScanline for ScanlineStorageAA<T> {
    // Renderer Interface

    fn prepare(&mut self) {
        self.covers.clear();
        self.scanlines.clear();
        self.spans.clear();
        self.min_x = 0x7FFFFFFF;
        self.min_y = 0x7FFFFFFF;
        self.max_x = -0x7FFFFFFF;
        self.max_y = -0x7FFFFFFF;
        self.cur_scanline = 0;
    }

    fn render<Sl: Scanline>(&mut self, sl: &Sl) {
        let mut sl_this = ScanlineData {
            y: 0,
            num_spans: 0,
            start_span: 0,
        };

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

        let spans = sl.begin();
        for span in spans {
            let mut sp = SpanData {
                x: 0,
                len: 0,
                covers_id: 0,
            };

            sp.x = span.x;
            sp.len = span.len as i32;
            let len = sp.len.abs() as usize;
            sp.covers_id = self.covers.len() as i32;
            self.covers.extend_from_slice(unsafe {
                std::slice::from_raw_parts_mut(span.covers as *mut T, len)
            }); // WRONG XXX need to make span generic

            self.spans.push(sp);
            let x1 = sp.x;
            let x2 = sp.x + len as i32 - 1;
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

pub struct SerializedScanlinesAdaptorAa<T> {
    data: *const u8,
    end: *const u8,
    ptr: *const u8,
    dx: i32,
    dy: i32,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    phantom: PhantomData<T>,
}

impl<T> SerializedScanlinesAdaptorAa<T> {
    pub fn new() -> Self {
        SerializedScanlinesAdaptorAa {
            data: ptr::null(),
            end: ptr::null(),
            ptr: ptr::null(),
            dx: 0,
            dy: 0,
            min_x: 0x7FFFFFFF,
            min_y: 0x7FFFFFFF,
            max_x: -0x7FFFFFFF,
            max_y: -0x7FFFFFFF,
            phantom: PhantomData,
        }
    }

    pub fn init(&mut self, data: *const u8, size: usize, dx: f64, dy: f64) {
        self.data = data;
        self.end = unsafe { data.offset(size as isize) };
        self.ptr = data;
        self.dx = iround(dx);
        self.dy = iround(dy);
        self.min_x = 0x7FFFFFFF;
        self.min_y = 0x7FFFFFFF;
        self.max_x = -0x7FFFFFFF;
        self.max_y = -0x7FFFFFFF;
    }

    fn read_int32(&mut self) -> i32 {
        let mut val: i32 = 0;
        unsafe {
            let ptr = &mut val as *mut i32 as *mut u8;
            *ptr = *self.ptr;
            *ptr.offset(1) = *self.ptr.offset(1);
            *ptr.offset(2) = *self.ptr.offset(2);
            *ptr.offset(3) = *self.ptr.offset(3);
            self.ptr = self.ptr.offset(4);
            //val = *(ptr as *const i32);
        }
        val
    }
}
// Iterate scanlines interface
impl<T> RasterScanLine for SerializedScanlinesAdaptorAa<T> {
    #[wrappit]
    fn rewind_scanlines(&mut self) -> bool {
        self.ptr = self.data;
        if self.ptr < self.end {
            self.min_x = self.read_int32() + self.dx;
            self.min_y = self.read_int32() + self.dy;
            self.max_x = self.read_int32() + self.dx;
            self.max_y = self.read_int32() + self.dy;
        }
        self.ptr < self.end
    }

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

    fn sweep_scanline<S: Scanline>(&mut self, sl: &mut S) -> bool {
        sl.reset_spans();
        loop {
            if self.ptr >= self.end {
                return false;
            }

            self.read_int32(); // Skip scanline size in bytes
            let y = self.read_int32() + self.dy;
            let num_spans = self.read_int32();

            for _ in 0..num_spans {
                let x = self.read_int32() + self.dx;
                let len = self.read_int32();

                if len < 0 {
                    sl.add_span(x, (-len) as u32, unsafe { (*self.ptr).into() });
                    self.ptr = unsafe { self.ptr.offset(mem::size_of::<T>() as isize) };
                } else {
                    let tmp = unsafe {
                        std::slice::from_raw_parts(self.ptr as *const S::CoverType, len as usize)
                    };
                    sl.add_cells(x, len as u32, tmp);
                    self.ptr =
                        unsafe { self.ptr.offset(len as isize * mem::size_of::<T>() as isize) };
                }
            }

            if sl.num_spans() > 0 {
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
