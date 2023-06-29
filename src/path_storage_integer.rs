use std::marker::PhantomData;

use crate::AggInteger;
use crate::{
    basics::{is_move_to, PathCmd, PathFlag, RectD},
    PathStore, VertexSource,
};

//---------------------------------------------------------VertexInteger
#[derive(Copy, Clone)]
pub struct VertexInteger<T: AggInteger, const COORD_SHIFT: u32 = 6> {
    pub x: T,
    pub y: T,
}

impl<T: AggInteger, const COORD_SHIFT: u32> VertexInteger<T, COORD_SHIFT> {
    //const COORD_SHIFT: u32 = COORD_SHIFT;
    const COORD_SCALE: u32 = 1 << COORD_SHIFT;
    const MOVE_TO: u32 = 0;
    const LINE_TO: u32 = 1;
    const CURVE3: u32 = 2;
    const CURVE4: u32 = 3;

    pub fn new_default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
        }
    }

    pub fn new(x_: T, y_: T, flag: u32) -> Self {
        Self {
            x: T::from_u32(((x_ << T::from_u32(1)) & !T::from_u32(1)).into_u32() | (flag & 1)),
            y: T::from_u32(((y_ << T::from_u32(1)) & !T::from_u32(1)).into_u32() | (flag >> 1)),
        }
    }

    pub fn vertex(&self, x_: &mut f64, y_: &mut f64, dx: f64, dy: f64, scale: f64) -> u32 {
        *x_ = dx + ((self.x >> T::from_u32(1)).into_f64() / Self::COORD_SCALE as f64) * scale;
        *y_ = dy + ((self.y >> T::from_u32(1)).into_f64() / Self::COORD_SCALE as f64) * scale;

        match ((self.y.into_u32() & 1) << 1) | (self.x.into_u32() & 1) {
            x if x == Self::MOVE_TO => PathCmd::MoveTo as u32,
            x if x == Self::LINE_TO => PathCmd::LineTo as u32,
            x if x == Self::CURVE3 => PathCmd::Curve3 as u32,
            x if x == Self::CURVE4 => PathCmd::Curve4 as u32,
            _ => PathCmd::Stop as u32,
        }
    }
}

//---------------------------------------------------path_storage_integer
pub struct PathStorageInteger<T: AggInteger, const COORD_SHIFT: u32 = 6> {
    storage: Vec<VertexInteger<T>>,
    vertex_idx: usize,
    closed: bool,
}

impl<T: AggInteger, const COORD_SHIFT: u32> PathStorageInteger<T, COORD_SHIFT> {
    pub fn new() -> PathStorageInteger<T> {
        PathStorageInteger {
            storage: Vec::new(),
            vertex_idx: 0,
            closed: true,
        }
    }

    pub fn remove_all(&mut self) {
        self.storage.clear();
    }

    pub fn size(&self) -> usize {
        self.storage.len()
    }

    pub fn vertex_idx(&self, idx: usize, x: &mut f64, y: &mut f64) -> u32 {
        self.storage[idx].vertex(x, y, 0., 0., 1.)
    }

    pub fn byte_size(&self) -> usize {
        self.storage.len() * std::mem::size_of::<VertexInteger<T>>()
    }

    pub fn serialize(&self, ptr: &mut [u8]) {
        let mut p = ptr.as_mut_ptr();
        for i in 0..self.storage.len() {
            let v = self.storage[i];
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &v as *const VertexInteger<T> as *const u8,
                    p,
                    std::mem::size_of::<VertexInteger<T>>(),
                );
                p = p.offset(std::mem::size_of::<VertexInteger<T>>() as isize);
            }
        }
    }

    pub fn bounding_rect(&self) -> RectD {
        let mut bounds = RectD::new(1e100, 1e100, -1e100, -1e100);
        if self.storage.len() == 0 {
            bounds.x1 = 0.0;
            bounds.y1 = 0.0;
            bounds.x2 = 0.0;
            bounds.y2 = 0.0;
        } else {
            for i in 0..self.storage.len() {
                let mut x = 0.0;
                let mut y = 0.0;
                self.storage[i].vertex(&mut x, &mut y, 0., 0., 1.);
                if x < bounds.x1 {
                    bounds.x1 = x;
                }
                if y < bounds.y1 {
                    bounds.y1 = y;
                }
                if x > bounds.x2 {
                    bounds.x2 = x;
                }
                if y > bounds.y2 {
                    bounds.y2 = y;
                }
            }
        }
        bounds
    }
}

impl<T: AggInteger, const COORD_SHIFT: u32> VertexSource for PathStorageInteger<T, COORD_SHIFT> {
    fn rewind(&mut self, _: u32) {
        self.vertex_idx = 0;
        self.closed = true;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.storage.len() < 2 || self.vertex_idx > self.storage.len() {
            *x = 0.0;
            *y = 0.0;
            return PathCmd::Stop as u32;
        }
        if self.vertex_idx == self.storage.len() {
            *x = 0.0;
            *y = 0.0;
            self.vertex_idx += 1;
            return PathCmd::EndPoly as u32 | PathFlag::Close as u32;
        }
        let cmd = self.storage[self.vertex_idx].vertex(x, y, 0., 0., 1.);
        if is_move_to(cmd) && !self.closed {
            *x = 0.0;
            *y = 0.0;
            self.closed = true;
            return PathCmd::EndPoly as u32 | PathFlag::Close as u32;
        }
        self.closed = false;
        self.vertex_idx += 1;
        cmd
    }
}
impl<T: AggInteger, const COORD_SHIFT: u32> PathStore for PathStorageInteger<T, COORD_SHIFT> {
    type T = T;

    fn move_to(&mut self, x: T, y: T) {
        self.storage
            .push(VertexInteger::new(x, y, VertexInteger::<T>::MOVE_TO));
    }

    fn line_to(&mut self, x: T, y: T) {
        self.storage
            .push(VertexInteger::new(x, y, VertexInteger::<T>::LINE_TO));
    }

    fn curve3(&mut self, x_ctrl: T, y_ctrl: T, x_to: T, y_to: T) {
        self.storage.push(VertexInteger::new(
            x_ctrl,
            y_ctrl,
            VertexInteger::<T>::CURVE3,
        ));
        self.storage
            .push(VertexInteger::new(x_to, y_to, VertexInteger::<T>::CURVE3));
    }

    fn curve4(&mut self, x_ctrl1: T, y_ctrl1: T, x_ctrl2: T, y_ctrl2: T, x_to: T, y_to: T) {
        self.storage.push(VertexInteger::new(
            x_ctrl1,
            y_ctrl1,
            VertexInteger::<T>::CURVE4,
        ));
        self.storage.push(VertexInteger::new(
            x_ctrl2,
            y_ctrl2,
            VertexInteger::<T>::CURVE4,
        ));
        self.storage
            .push(VertexInteger::new(x_to, y_to, VertexInteger::<T>::CURVE4));
    }

    fn close_polygon(&mut self) {}
}

//-----------------------------------------serialized_integer_path_adaptor
pub struct SerializedIntegerPathAdaptor<T: AggInteger, const COORD_SHIFT: u32 = 6> {
    data: *const u8,
    end: *const u8,
    ptr: *const u8,
    dx: f64,
    dy: f64,
    scale: f64,
    vertices: u32,
    dummy: PhantomData<T>,
}

impl<T: AggInteger, const COORD_SHIFT: u32> SerializedIntegerPathAdaptor<T, COORD_SHIFT> {
    pub fn new() -> Self {
        SerializedIntegerPathAdaptor {
            data: 0 as *const u8,
            end: 0 as *const u8,
            ptr: 0 as *const u8,
            dx: 0.0,
            dy: 0.0,
            scale: 1.0,
            vertices: 0,
            dummy: PhantomData,
        }
    }

    pub fn init(&mut self, data: *const u8, size: usize, dx: f64, dy: f64, scale: f64) {
        self.data = data;
        self.end = unsafe { data.offset(size as isize) };
        self.ptr = data;
        self.dx = dx;
        self.dy = dy;
        self.scale = scale;
        self.vertices = 0;
    }
}

impl<T: AggInteger, const COORD_SHIFT: u32> VertexSource
    for SerializedIntegerPathAdaptor<T, COORD_SHIFT>
{
    fn rewind(&mut self, _: u32) {
        self.ptr = self.data;
        self.vertices = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.data == 0 as *const u8 || self.ptr > self.end {
            *x = 0.0;
            *y = 0.0;
            return PathCmd::Stop as u32;
        }

        if self.ptr == self.end {
            *x = 0.0;
            *y = 0.0;
            self.ptr = unsafe {
                self.ptr
                    .offset(std::mem::size_of::<VertexInteger<T, COORD_SHIFT>>() as isize)
            };
            return PathCmd::EndPoly as u32 | PathFlag::Close as u32;
        }
		
        let mut v = VertexInteger::new_default();
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.ptr,
                &mut v as *mut VertexInteger<T> as *mut u8,
                std::mem::size_of::<VertexInteger<T>>(),
            );
        }
        let cmd = v.vertex(x, y, self.dx, self.dy, self.scale);
        if is_move_to(cmd) && self.vertices > 2 {
            *x = 0.0;
            *y = 0.0;
            self.vertices = 0;
            return PathCmd::EndPoly as u32 | PathFlag::Close as u32;
        }
        self.vertices += 1;
        self.ptr = unsafe {
            self.ptr
                .offset(std::mem::size_of::<VertexInteger<T>>() as isize)
        };
        return cmd;
    }
}
