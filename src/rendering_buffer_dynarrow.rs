use crate::array::*;
use crate::basics::*;

//NOT TESTED

//===============================================RenderingBufferDynarrow
// Rendering buffer class with dynamic allocation of the rows.
// The rows are allocated as needed when requesting for span_ptr().
// The class automatically calculates min_x and max_x for each row.
// Generally it's more efficient to use this class as a temporary buffer
// for rendering a few lines and then to blend it with another buffer.

pub struct RenderingBufferDynarrow {
    rows: PodArray<RowData<u8>>,
    width: u32,
    height: u32,
    byte_width: u32,
}

impl RenderingBufferDynarrow {
    pub fn new() -> RenderingBufferDynarrow {
        RenderingBufferDynarrow {
            rows: PodArray::new(),
            width: 0,
            height: 0,
            byte_width: 0,
        }
    }

    // Allocate and clear the buffer
    pub fn init(&mut self, width: u32, height: u32, byte_width: u32) {
        self.rows.clear();
        if width != 0 && height != 0 {
            self.width = width;
            self.height = height;
            self.byte_width = byte_width;
            self.rows.resize(
                height as usize,
                RowData {
                    ptr: 0 as *mut u8,
                    x1: 0,
                    x2: 0,
                },
            );
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn byte_width(&self) -> u32 {
        self.byte_width
    }

    // The main function used for rendering. Returns pointer to the
    // pre-allocated span. Memory for the row is allocated as needed.
    pub fn row_ptr_xy(&mut self, x: i32, y: i32, len: u32) -> *const u8 {
        let r = &mut self.rows[y as usize];
        let x2 = x + len as i32 - 1;
        if !r.ptr.is_null() {
            if x < r.x1 {
                r.x1 = x;
            }
            if x2 > r.x2 {
                r.x2 = x2;
            }
        } else {
            let p = vec![0; self.byte_width as usize].as_mut_ptr();
            r.ptr = p;
            r.x1 = x;
            r.x2 = x2;
        }
        r.ptr
    }

    pub fn row_ptr_const(&self, y: i32) -> *const u8 {
        self.rows[y as usize].ptr
    }

    pub fn row_ptr(&mut self, y: i32) -> *mut u8 {
        self.row_ptr_xy(0, y, self.width) as *mut u8
    }

    pub fn row(&self, y: i32) -> RowData<u8> {
        self.rows[y as usize]
    }
}
