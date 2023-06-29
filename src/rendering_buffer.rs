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
// class RenderBuf
//
//----------------------------------------------------------------------------
//========================================================RenderBuf
//
/// The definition of the main type for accessing the rows in the frame
/// buffer. It provides functionality to navigate to the rows in a
/// rectangular matrix, from top to bottom or from bottom to top depending
/// on stride.
///
/// RowAccessBuf is cheap to create/destroy, but performs one multiplication
/// when calling row_ptr().
///
/// RowPtrCacheBuf creates an array of pointers to rows, so, the access
/// via row_ptr() may be faster. But it requires memory allocation
/// when creating. For example, on typical Intel Pentium hardware
/// RowPtrCacheBuf speeds span_image_filter_rgb_nn up to 10%
///
/// It's used only in short hand typedefs like pixfmt_rgba32 and can be
/// redefined in agg_config.h
/// In real applications you can use both, depending on your needs
//------------------------------------------------------------------------
use crate::{basics::RowData, RenderBuffer};
use core::ptr::null_mut;

//typedef RowPtrCacheBuf<u8> RenderBuf;
pub type RenderBuf = RowAccessBuf<u8>;

//===========================================================RowAccessBuf
#[derive(Clone, Copy)]
pub struct RowAccessBuf<T: Copy> {
    buf: *mut T,   // Pointer (start of row) to rendering buffer
    start: *mut T, // Pointer to first (row) pixel depending on stride
    width: u32,    // Width in pixels
    height: u32,   // Height in pixels
    stride: i32,   // Number of bytes per row. Can be < 0
}

impl<T: Copy> RenderBuffer for RowAccessBuf<T> {
    type T = T;

    fn attach(&mut self, buf: *mut T, width: u32, height: u32, stride: i32) {
        self.buf = buf;
        self.start = buf;
        self.width = width;
        self.height = height;
        self.stride = stride;
        if self.stride < 0 {
            unsafe {
                self.start = self.buf.offset(-(((height as i32 - 1) * stride) as isize));
            }
        }
    }

    #[inline]
    fn width(&self) -> u32 {
        return self.width;
    }

    #[inline]
    fn height(&self) -> u32 {
        return self.height;
    }

    #[inline]
    fn stride(&self) -> i32 {
        return self.stride;
    }

    #[inline]
    fn stride_abs(&self) -> u32 {
        return (if self.stride < 0 {
            -self.stride
        } else {
            self.stride
        }) as u32;
    }

    #[inline]
    fn row(&self, y: i32) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.start.offset((y as i32 * self.stride) as isize),
                self.stride.abs() as usize,
            )
        }
    }

    #[inline]
    fn row_mut(&mut self, y: i32) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.start.offset((y as i32 * self.stride) as isize),
                self.stride.abs() as usize,
            )
        }
    }

    #[inline]
    fn row_data(&self, y: i32) -> RowData<T> {
        return RowData::new(0, self.width as i32 - 1, self.row_ptr(0, y, 0));
    }

    fn copy_from<RenBuf: crate::RenderBuffer<T = T>>(&mut self, src: &RenBuf) {
        let mut h = self.height();
        if src.height() < h {
            h = src.height();
        }

        for y in 0..h as i32 - 1 {
            //self.row_mut(y).copy_from_slice(src.row(y));
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.row(y).as_ptr(),
                    self.row_mut(y).as_mut_ptr(),
                    src.row(y).len(),
                );
            }
        }
    }
}

impl<T: Copy> RowAccessBuf<T> {
    pub fn new_default() -> Self {
        Self {
            buf: null_mut(),
            start: null_mut(),
            width: 0,
            height: 0,
            stride: 0,
        }
    }

    pub fn new(buf: *mut T, width: u32, height: u32, stride: i32) -> Self {
        let mut tmp = Self {
            buf: null_mut(),
            start: null_mut(),
            width: 0,
            height: 0,
            stride: 0,
        };
        tmp.attach(buf, width, height, stride);
        tmp
    }

    pub fn clear(&self, value: T) {
        let w = self.width();
        let stride = self.stride_abs();
        for y in 0..self.height() - 1 {
            let p = self.row_ptr_mut(0, y as i32, w);
            unsafe {
                for x in 0..stride as isize - 1 {
                    *p.offset(x) = value;
                }
            }
        }
    }
    #[inline]
    pub fn buf(&self) -> *const T {
        return self.buf;
    }

    #[inline]
    pub fn buf_mut(&self) -> *mut T {
        return self.buf;
    }

    #[inline]
    pub fn row_ptr(&self, _x: i32, y: i32, _s: u32) -> *const T {
        unsafe { self.start.offset((y as i32 * self.stride) as isize) }
    }
    #[inline]
    pub fn row_ptr_mut(&self, _x: i32, y: i32, _s: u32) -> *mut T {
        unsafe { self.start.offset((y as i32 * self.stride) as isize) }
    }
}

//==========================================================RowPtrCacheBuf
pub struct RowPtrCacheBuf<'a, T: Copy> {
    buf: *mut T,
    rows: Vec<&'a [T]>, // Pointers to each row of the buffer
    rows_mut: Vec<&'a mut [T]>,
    width: u32,
    height: u32,
    stride: i32,
}

impl<'a, T: Copy> RenderBuffer for RowPtrCacheBuf<'a, T> {
    type T = T;

    fn attach(&mut self, buf: *mut T, width: u32, height: u32, stride: i32) {
        self.buf = buf;
        self.width = width;
        self.height = height;
        self.stride = stride;

        let mut row_ptr = self.buf;
        if self.stride < 0 {
            unsafe {
                row_ptr = self.buf.offset(-(((height as i32 - 1) * stride) as isize));
            }
        }

        self.rows.clear();
        self.rows_mut.clear();

        for _i in 0..self.height as usize {
            unsafe {
                self.rows.push(std::slice::from_raw_parts(
                    row_ptr,
                    self.stride.abs() as usize,
                ));
                self.rows_mut.push(std::slice::from_raw_parts_mut(
                    row_ptr,
                    self.stride.abs() as usize,
                ));

                row_ptr = row_ptr.offset(self.stride as isize);
            }
        }
    }

    #[inline]
    fn width(&self) -> u32 {
        return self.width;
    }

    #[inline]
    fn height(&self) -> u32 {
        return self.height;
    }

    #[inline]
    fn stride(&self) -> i32 {
        return self.stride;
    }

    #[inline]
    fn stride_abs(&self) -> u32 {
        return (if self.stride < 0 {
            -self.stride
        } else {
            self.stride
        }) as u32;
    }

    #[inline]
    fn row(&self, y: i32) -> &[T] {
        self.rows[y as usize]
    }

    #[inline]
    fn row_mut(&mut self, y: i32) -> &mut [T] {
        self.rows_mut[y as usize]
    }

    #[inline]
    fn row_data(&self, y: i32) -> RowData<T> {
        return RowData::new(0, self.width as i32 - 1, self.rows[y as usize].as_ptr());
    }

    fn copy_from<RenBuf: crate::RenderBuffer<T = T>>(&mut self, src: &RenBuf) {
        let mut h = self.height();
        if src.height() < h {
            h = src.height();
        }

        for y in 0..h as i32 - 1 {
            self.row_mut(y).copy_from_slice(src.row(y));
        }
    }
}

impl<'a, T: Copy> RowPtrCacheBuf<'a, T> {
    pub fn new_default() -> Self {
        RowPtrCacheBuf {
            buf: 0 as *mut T,
            rows: Vec::new(),
            rows_mut: Vec::new(),
            width: 0,
            height: 0,
            stride: 0,
        }
    }

    pub fn new(buf: *mut T, width: u32, height: u32, stride: i32) -> Self {
        let mut tmp = Self::new_default();
        tmp.attach(buf, width, height, stride);
        tmp
    }

    pub fn rows(&self) -> &[&[T]] {
        &self.rows
    }

    pub fn clear(&mut self, value: T) {
        //let w = self.width();
        let stride = self.stride_abs();
        for y in 0..self.height() - 1 {
            let p = self.row_mut(y as i32);

            for x in 0..stride as usize {
                //XXXX -1?
                p[x] = value;
            }
        }
    }

    #[inline]
    pub fn buf(&self) -> *const T {
        return self.buf;
    }

    #[inline]
    pub fn buf_mut(&self) -> *mut T {
        return self.buf;
    }
}
