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
// Viewport transformer - simple orthogonal conversions from world coordinates
//                        to screen (device) ones.
//
//----------------------------------------------------------------------------

use self::AspectRatio::*;
use crate::trans_affine::TransAffine;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AspectRatio {
    Stretch,
    Meet,
    Slice,
}

//----------------------------------------------------------TransViewport
#[derive(Clone, Copy)]
pub struct TransViewport {
    world_x1: f64,
    world_y1: f64,
    world_x2: f64,
    world_y2: f64,
    device_x1: f64,
    device_y1: f64,
    device_x2: f64,
    device_y2: f64,
    aspect: AspectRatio,
    is_valid: bool,
    align_x: f64,
    align_y: f64,
    wx1: f64,
    wy1: f64,
    wx2: f64,
    wy2: f64,
    dx1: f64,
    dy1: f64,
    kx: f64,
    ky: f64,
}
impl TransViewport {
    pub fn new() -> Self {
        Self {
            world_x1: 0.0,
            world_y1: 0.0,
            world_x2: 1.0,
            world_y2: 1.0,
            device_x1: 0.0,
            device_y1: 0.0,
            device_x2: 1.0,
            device_y2: 1.0,
            aspect: Stretch,
            is_valid: true,
            align_x: 0.5,
            align_y: 0.5,
            wx1: 0.0,
            wy1: 0.0,
            wx2: 1.0,
            wy2: 1.0,
            dx1: 0.0,
            dy1: 0.0,
            kx: 1.0,
            ky: 1.0,
        }
    }

    //-------------------------------------------------------------------
    pub fn set_preserve_aspect_ratio(&mut self, alignx: f64, aligny: f64, aspect: AspectRatio) {
        self.align_x = alignx;
        self.align_y = aligny;
        self.aspect = aspect;
        self.update();
    }

    //-------------------------------------------------------------------
    pub fn set_device_viewport(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.device_x1 = x1;
        self.device_y1 = y1;
        self.device_x2 = x2;
        self.device_y2 = y2;
        self.update();
    }

    //-------------------------------------------------------------------
    pub fn set_world_viewport(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.world_x1 = x1;
        self.world_y1 = y1;
        self.world_x2 = x2;
        self.world_y2 = y2;
        self.update();
    }

    //-------------------------------------------------------------------
    pub fn device_viewport(&self, x1: &mut f64, y1: &mut f64, x2: &mut f64, y2: &mut f64) {
        *x1 = self.device_x1;
        *y1 = self.device_y1;
        *x2 = self.device_x2;
        *y2 = self.device_y2;
    }

    //-------------------------------------------------------------------
    pub fn world_viewport(&self, x1: &mut f64, y1: &mut f64, x2: &mut f64, y2: &mut f64) {
        *x1 = self.world_x1;
        *y1 = self.world_y1;
        *x2 = self.world_x2;
        *y2 = self.world_y2;
    }
    //-------------------------------------------------------------------
    pub fn world_viewport_actual(
        &self, x1: &mut f64, y1: &mut f64, x2: &mut f64, y2: &mut f64,
    ) {
        *x1 = self.wx1;
        *y1 = self.wy1;
        *x2 = self.wx2;
        *y2 = self.wy2;
    }

    //-------------------------------------------------------------------
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
    pub fn align_x(&self) -> f64 {
        self.align_x
    }
    pub fn align_y(&self) -> f64 {
        self.align_y
    }
    pub fn aspect_ratio(&self) -> AspectRatio {
        self.aspect
    }
    pub fn transform(&self, x: &mut f64, y: &mut f64) {
        *x = (*x - self.wx1) * self.kx + self.dx1;
        *y = (*y - self.wy1) * self.ky + self.dy1;
    }
    pub fn transform_scale_only(&self, x: &mut f64, y: &mut f64) {
        *x *= self.kx;
        *y *= self.ky;
    }
    pub fn inverse_transform(&self, x: &mut f64, y: &mut f64) {
        *x = (*x - self.dx1) / self.kx + self.wx1;
        *y = (*y - self.dy1) / self.ky + self.wy1;
    }
    pub fn inverse_transform_scale_only(&self, x: &mut f64, y: &mut f64) {
        *x /= self.kx;
        *y /= self.ky;
    }
    pub fn device_dx(&self) -> f64 {
        self.dx1 - self.wx1 * self.kx
    }
    pub fn device_dy(&self) -> f64 {
        self.dy1 - self.wy1 * self.ky
    }

    //-------------------------------------------------------------------
    pub fn scale_x(&self) -> f64 {
        self.kx
    }
    pub fn scale_y(&self) -> f64 {
        self.ky
    }
    pub fn scale(&self) -> f64 {
        (self.kx + self.ky) * 0.5
    }

    //-------------------------------------------------------------------
    pub fn to_affine(&self) -> TransAffine {
        let mut mtx = TransAffine::trans_affine_translation(-self.wx1, -self.wy1);
        mtx *= TransAffine::trans_affine_scaling(self.kx, self.ky);
        mtx *= TransAffine::trans_affine_translation(self.dx1, self.dy1);
        return mtx;
    }

    //-------------------------------------------------------------------
    pub fn to_affine_scale_only(&self) -> TransAffine {
        return TransAffine::trans_affine_scaling(self.kx, self.ky);
    }

    //-------------------------------------------------------------------
    pub fn byte_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
    pub fn serialize(&self, ptr: &mut [u8]) {
        ptr.copy_from_slice(unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        });
    }
    pub fn deserialize(&mut self, ptr: &[u8]) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                ptr.as_ptr(),
                (self as *mut Self) as *mut u8,
                std::mem::size_of::<Self>(),
            );
        }
    }
    /*
        pub fn serialize(int8u* ptr)
        {
            memcpy(ptr, this, sizeof(*this));
        }

        pub fn deserialize( int8u* ptr)
        {
            memcpy(this,  ptr, sizeof(*this));
        }
    */
    //-----------------------------------------------------------------------
    pub fn update(&mut self) {
        let epsilon = 1e-30;
        if (self.world_x1 - self.world_x2).abs() < epsilon
            || (self.world_y1 - self.world_y2).abs() < epsilon
            || (self.device_x1 - self.device_x2).abs() < epsilon
            || (self.device_y1 - self.device_y2).abs() < epsilon
        {
            self.wx1 = self.world_x1;
            self.wy1 = self.world_y1;
            self.wx2 = self.world_x1 + 1.0;
            self.wy2 = self.world_y2 + 1.0;
            self.dx1 = self.device_x1;
            self.dy1 = self.device_y1;
            self.kx = 1.0;
            self.ky = 1.0;
            self.is_valid = false;
            return;
        }

        let mut world_x1 = self.world_x1;
        let mut world_y1 = self.world_y1;
        let mut world_x2 = self.world_x2;
        let mut world_y2 = self.world_y2;
        let device_x1 = self.device_x1;
        let device_y1 = self.device_y1;
        let device_x2 = self.device_x2;
        let device_y2 = self.device_y2;
        if self.aspect != Stretch {
            let d;
            self.kx = (device_x2 - device_x1) / (world_x2 - world_x1);
            self.ky = (device_y2 - device_y1) / (world_y2 - world_y1);

            if (self.aspect == Meet) == (self.kx < self.ky) {
                d = (world_y2 - world_y1) * self.ky / self.kx;
                world_y1 += (world_y2 - world_y1 - d) * self.align_y;
                world_y2 = world_y1 + d;
            } else {
                d = (world_x2 - world_x1) * self.kx / self.ky;
                world_x1 += (world_x2 - world_x1 - d) * self.align_x;
                world_x2 = world_x1 + d;
            }
        }
        self.wx1 = world_x1;
        self.wy1 = world_y1;
        self.wx2 = world_x2;
        self.wy2 = world_y2;
        self.dx1 = device_x1;
        self.dy1 = device_y1;
        self.kx = (device_x2 - device_x1) / (world_x2 - world_x1);
        self.ky = (device_y2 - device_y1) / (world_y2 - world_y1);
        self.is_valid = true;
    }
}
