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
// Adaptation for high precision colors has been sponsored by
// Liberty Technology Systems, Inc., visit http://lib-sys.com
//
// Liberty Technology Systems, Inc. is the provider of
// PostScript and PDF technology for software developers.
//
//----------------------------------------------------------------------------

use crate::basics::{RectI, RowData};
use crate::color_rgba::{OrderBgr, OrderRgb, Rgba16, Rgba8};
use crate::gamma_lut::GammaLut;
use crate::rendering_buffer::RenderBuf;
use crate::{
    slice_t_to_vt, slice_t_to_vt_mut, AggPrimitive, Args, Blender, Color, Equiv, Gamma, ImageSrc,
    Order, PixFmt, RenderBuffer, RgbArgs,
};
use std::marker::PhantomData;
use wrapping_arithmetic::wrappit;

pub type PixRgb24<'a> = AlphaBlendRgb<'a, Rgba8, OrderRgb, BlenderRgb<Rgba8, OrderRgb>, RenderBuf>; //----PixRgb24
pub type PixBgr24<'a> = AlphaBlendRgb<'a, Rgba8, OrderBgr, BlenderRgb<Rgba8, OrderBgr>, RenderBuf>; //----PixBgr24
pub type PixRgb48<'a> =
    AlphaBlendRgb<'a, Rgba16, OrderRgb, BlenderRgb<Rgba16, OrderRgb>, RenderBuf>; //----PixRgb48
pub type PixBgr48<'a> =
    AlphaBlendRgb<'a, Rgba16, OrderBgr, BlenderRgb<Rgba16, OrderBgr>, RenderBuf>; //----PixBgr48

pub type BlenderBgr24 = BlenderRgb<Rgba8, OrderBgr>;
pub type BlenderBgr24Pre = BlenderRgbPre<Rgba8, OrderBgr>;

pub type PixRgb24Pre<'a> =
    AlphaBlendRgb<'a, Rgba8, OrderRgb, BlenderRgbPre<Rgba8, OrderRgb>, RenderBuf>; //----PixRgb24Pre
pub type PixBgr24Pre<'a> =
    AlphaBlendRgb<'a, Rgba8, OrderBgr, BlenderRgbPre<Rgba8, OrderBgr>, RenderBuf>; //----PixBgr24Pre
pub type PixRgb48Pre<'a> =
    AlphaBlendRgb<'a, Rgba16, OrderRgb, BlenderRgbPre<Rgba16, OrderRgb>, RenderBuf>; //----PixRgb48Pre
pub type PixBgr48Pre<'a> =
    AlphaBlendRgb<'a, Rgba16, OrderBgr, BlenderRgbPre<Rgba16, OrderBgr>, RenderBuf>; //----PixBgr48Pre
pub type PixBgr24Gamma<'a> = AlphaBlendRgb<
    'a,
    Rgba8,
    OrderBgr,
    BlenderRgbGamma<'a, Rgba8, OrderBgr, GammaLut<u8, u8, 8, 8>>,
    RenderBuf,
>;
pub type PixRgb24Gamma<'a> = AlphaBlendRgb<
    'a,
    Rgba8,
    OrderBgr,
    BlenderRgbGamma<'a, Rgba8, OrderRgb, GammaLut<u8, u8, 8, 8>>,
    RenderBuf,
>;

macro_rules! from_u32 {
    ($v:expr) => {
        C::ValueType::from_u32($v)
    };
}

//=====================================================ApplyGammaDirRgb
pub struct ApplyGammaDirRgb<
    'a,
    C: crate::Color,
    Order: crate::Order,
    GammaLut: crate::Gamma<C::ValueType, C::ValueType>,
> {
    gamma: &'a GammaLut,
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<Order>,
}

impl<
        'a,
        C: crate::Color,
        Order: crate::Order,
        GammaLut: crate::Gamma<C::ValueType, C::ValueType>,
    > ApplyGammaDirRgb<'a, C, Order, GammaLut>
{
    pub fn new(gamma: &'a GammaLut) -> Self {
        ApplyGammaDirRgb {
            gamma: gamma,
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }

    #[inline]
    fn apply(&self, p: &mut [C::ValueType]) {
        p[Order::R] = self.gamma.dir(p[Order::R]);
        p[Order::G] = self.gamma.dir(p[Order::G]);
        p[Order::B] = self.gamma.dir(p[Order::B]);
    }
}

//=====================================================ApplyGammaInvRgb
pub struct ApplyGammaInvRgb<
    'a,
    C: crate::Color,
    Order: crate::Order,
    GammaLut: crate::Gamma<C::ValueType, C::ValueType>,
> {
    gamma: &'a GammaLut,
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<Order>,
}

impl<
        'a,
        C: crate::Color,
        Order: crate::Order,
        GammaLut: crate::Gamma<C::ValueType, C::ValueType>,
    > ApplyGammaInvRgb<'a, C, Order, GammaLut>
{
    pub fn new(gamma: &'a GammaLut) -> Self {
        ApplyGammaInvRgb {
            gamma: gamma,
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }

    #[inline]
    fn apply(&self, p: &mut [C::ValueType]) {
        p[Order::R] = self.gamma.inv(p[Order::R]);
        p[Order::G] = self.gamma.inv(p[Order::G]);
        p[Order::B] = self.gamma.inv(p[Order::B]);
    }
}

//=========================================================BlenderRgb
pub struct BlenderRgb<C: Color, O: Order> {
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<O>,
}

impl<C: Color, O: Order> Blender<C, O> for BlenderRgb<C, O> {
    fn new() -> Self {
        Self {
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }
    #[wrappit]
    #[inline]
    fn blend_pix(&self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32) {
        p[O::R] =
            p[O::R] + C::ValueType::from_u32(((cr - p[O::R].into_u32()) * alpha) >> C::BASE_SHIFT);
        p[O::G] =
            p[O::G] + C::ValueType::from_u32(((cg - p[O::G].into_u32()) * alpha) >> C::BASE_SHIFT);
        p[O::B] =
            p[O::B] + C::ValueType::from_u32(((cb - p[O::B].into_u32()) * alpha) >> C::BASE_SHIFT);
    }

    #[inline]
    fn blend_pix_with_cover(
        &self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }
}

//======================================================BlenderRgbPre
pub struct BlenderRgbPre<C: Color, O: Order> {
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<O>,
}

impl<C: Color, O: Order> crate::Blender<C, O> for BlenderRgbPre<C, O> {
    fn new() -> Self {
        Self {
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }
    #[inline]
    fn blend_pix_with_cover(
        &self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = C::BASE_MASK - alpha;
        let cover = (cover + 1) << (C::BASE_SHIFT - 8);

        p[O::R] = from_u32!((p[O::R].into_u32() * alpha + cr * cover) >> C::BASE_SHIFT);
        p[O::G] = from_u32!((p[O::G].into_u32() * alpha + cg * cover) >> C::BASE_SHIFT);
        p[O::B] = from_u32!((p[O::B].into_u32() * alpha + cb * cover) >> C::BASE_SHIFT);
    }

    #[inline]
    fn blend_pix(&self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32) {
        let alpha = C::BASE_MASK - alpha;

        p[O::R] = from_u32!(((p[O::R].into_u32() * alpha) >> C::BASE_SHIFT) + cr);
        p[O::G] = from_u32!(((p[O::G].into_u32() * alpha) >> C::BASE_SHIFT) + cg);
        p[O::B] = from_u32!(((p[O::B].into_u32() * alpha) >> C::BASE_SHIFT) + cb);
    }
}

//===================================================BlenderRgbGamma
pub struct BlenderRgbGamma<'a, C: Color, O: Order, GL: Gamma<C::ValueType, C::ValueType>> {
    gamma: Equiv<'a, GL>,
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<O>,
}

impl<'a, C: Color, O: Order, GL: Gamma<C::ValueType, C::ValueType>> BlenderRgbGamma<'a, C, O, GL> {
    pub fn new_owned(gamma: GL) -> Self {
        BlenderRgbGamma {
            gamma: Equiv::Own(gamma),
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }

    pub fn new_borrowed(gamma: &'a mut GL) -> Self {
        BlenderRgbGamma {
            gamma: Equiv::Brw(gamma),
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }

    pub fn set_gamma_borrowed(&mut self, g: &'a mut GL) {
        self.gamma = Equiv::Brw(g);
    }

    pub fn set_gamma_owned(&mut self, g: GL) {
        self.gamma = Equiv::Own(g);
    }

    pub fn gamma_mut(&mut self) -> &mut GL {
        &mut self.gamma
    }

    pub fn gamma(&self) -> &GL {
        &self.gamma
    }
}

impl<'a, C: Color, O: Order, GL: Gamma<C::ValueType, C::ValueType>> Blender<C, O>
    for BlenderRgbGamma<'a, C, O, GL>
{
    fn new() -> Self {
        BlenderRgbGamma {
            gamma: Equiv::Own(GL::new()),
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }

    #[wrappit]
    #[inline]
    fn blend_pix(&self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32) {
        let r = self.gamma.dir(p[O::R]).into_u32();
        let g = self.gamma.dir(p[O::G]).into_u32();
        let b = self.gamma.dir(p[O::B]).into_u32();

        p[O::R] = self.gamma.inv(C::ValueType::from_u32(
            (((self.gamma.dir(C::ValueType::from_u32(cr)).into_u32() - r) * alpha)
                >> C::BASE_SHIFT)
                + r,
        ));
        p[O::G] = self.gamma.inv(C::ValueType::from_u32(
            (((self.gamma.dir(C::ValueType::from_u32(cg)).into_u32() - g) * alpha)
                >> C::BASE_SHIFT)
                + g,
        ));
        p[O::B] = self.gamma.inv(C::ValueType::from_u32(
            (((self.gamma.dir(C::ValueType::from_u32(cb)).into_u32() - b) * alpha)
                >> C::BASE_SHIFT)
                + b,
        ));
    }

    fn blend_pix_with_cover(
        &self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }
}

pub struct AlphaBlendRgb<
    'a,
    C: Color + RgbArgs,
    O: Order,
    Blend: Blender<C, O>,
    RenBuf: RenderBuffer<T = u8>,
> {
    rbuf: Equiv<'a, RenBuf>,
    color: PhantomData<C>,
    order: PhantomData<O>,
    blender: Blend,
}

impl<'a, C: Color + RgbArgs, O: Order, Blend: Blender<C, O>, RenBuf: RenderBuffer<T = u8>>
    AlphaBlendRgb<'a, C, O, Blend, RenBuf>
{
    const PIXEL_WIDTH: usize = std::mem::size_of::<C::ValueType>() * 3;

    pub fn new_borrowed(rb: &'a mut RenBuf) -> Self {
        AlphaBlendRgb::<C, O, Blend, RenBuf> {
            rbuf: Equiv::Brw(rb),
            blender: Blend::new(),
            color: PhantomData,
            order: PhantomData,
        }
    }

    pub fn new_owned(rb: RenBuf) -> Self {
        AlphaBlendRgb::<C, O, Blend, RenBuf> {
            rbuf: Equiv::Own(rb),
            blender: Blend::new(),
            color: PhantomData,
            order: PhantomData,
        }
    }

    pub fn attach_borrowed(&mut self, rb: &'a mut RenBuf) {
        self.rbuf = Equiv::Brw(rb);
    }

    pub fn attach_owned(&mut self, rb: RenBuf) {
        self.rbuf = Equiv::Own(rb);
    }

    pub fn rbuf_mut(&mut self) -> &mut RenBuf {
        &mut self.rbuf
    }

    pub fn blender_mut(&mut self) -> &mut Blend {
        &mut self.blender
    }

    #[inline]
    fn copy_or_blend_pix_cover(p: &mut [C::ValueType], c: &C, cover: u32, blender: &Blend) {
        if c.a().into_u32() != 0 {
            let alpha = (c.a().into_u32() * (cover + 1)) >> 8;
            if alpha == C::BASE_MASK as u32 {
                p[O::R] = c.r();
                p[O::G] = c.g();
                p[O::B] = c.b();
            } else {
                blender.blend_pix_with_cover(
                    p,
                    c.r().into_u32(),
                    c.g().into_u32(),
                    c.b().into_u32(),
                    alpha,
                    cover,
                );
            }
        }
    }

    #[inline]
    fn copy_or_blend_pix(p: &mut [C::ValueType], c: &C, blender: &Blend) {
        if c.a().into_u32() != 0 {
            if c.a().into_u32() == C::BASE_MASK as u32 {
                p[O::R] = c.r();
                p[O::G] = c.g();
                p[O::B] = c.b();
            } else {
                blender.blend_pix(
                    p,
                    c.r().into_u32(),
                    c.g().into_u32(),
                    c.b().into_u32(),
                    c.a().into_u32(),
                );
            }
        }
    }

    pub fn for_each_pixel(&mut self, func: &dyn Fn(&mut [C::ValueType])) {
        for y in 0..self.height() as i32 {
            let r = self.rbuf.row_data(y);
            if !r.ptr.is_null() {
                let len = r.x2 - r.x1 + 1;
                let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), r.x1 * 3, C::ValueType);
                let mut i = 0;
                for _ in 0..len {
                    func(&mut p[i as usize..]);
                    i += 3;
                }
            }
        }
    }

    pub fn apply_gamma_dir<GammaLut: crate::Gamma<C::ValueType, C::ValueType>>(
        &mut self, g: &GammaLut,
    ) {
        let ag = ApplyGammaDirRgb::<C, O, GammaLut>::new(g);
        self.for_each_pixel(&|p| ag.apply(p));
    }

    pub fn apply_gamma_inv<GammaLut: crate::Gamma<C::ValueType, C::ValueType>>(
        &mut self, g: &GammaLut,
    ) {
        let ag = ApplyGammaInvRgb::<C, O, GammaLut>::new(g);
        self.for_each_pixel(&|p| ag.apply(p));
    }
}

impl<'a, C: Color + RgbArgs, O: Order, Blend: Blender<C, O>, RenBuf: RenderBuffer<T = u8>> ImageSrc
    for AlphaBlendRgb<'a, C, O, Blend, RenBuf>
{
}

impl<'a, C: Color + RgbArgs, O: Order, Blend: Blender<C, O>, RenBuf: RenderBuffer<T = u8>> PixFmt
    for AlphaBlendRgb<'a, C, O, Blend, RenBuf>
{
    type C = C;
    type O = O;
    type T = RenBuf::T;
    const PIXEL_WIDTH: u32 = std::mem::size_of::<C::ValueType>() as u32 * 3;

    fn width(&self) -> u32 {
        self.rbuf.width()
    }

    fn height(&self) -> u32 {
        self.rbuf.height()
    }

    fn attach_pixfmt<Pix: PixFmt>(
        &mut self, pixf: &Pix, x1: i32, y1: i32, x2: i32, y2: i32,
    ) -> bool {
        let mut r = RectI::new(x1, y1, x2, y2);
        if r.clip(&RectI::new(
            0,
            0,
            pixf.width() as i32 - 1,
            pixf.height() as i32 - 1,
        )) {
            let stride = pixf.stride();
            let (p, i) = pixf.pix_ptr(r.x1, if stride < 0 { r.y2 } else { r.y1 });

            self.rbuf.attach(
                &p[i] as *const u8 as *mut u8,
                ((r.x2 - r.x1) + 1) as u32,
                ((r.y2 - r.y1) + 1) as u32,
                stride,
            );
            return true;
        }
        return false;
    }

    fn stride(&self) -> i32 {
        self.rbuf.stride()
    }

    fn pix_ptr(&self, x: i32, y: i32) -> (&[u8], usize) {
        let p;
        let h = self.rbuf.height() as i32;
        let stride = self.rbuf.stride();
        let len;
        let off;
        if stride < 0 {
            p = self.rbuf.row(h - 1).as_ptr();
            len = (h - y) * stride.abs();
            off = (h - y - 1) * stride.abs() + x * Self::PIXEL_WIDTH as i32;
        } else {
            p = self.rbuf.row(y).as_ptr();
            len = (h - y) * stride.abs();
            off = x * Self::PIXEL_WIDTH as i32;
        }
        (
            unsafe { std::slice::from_raw_parts(p as *const u8, len as usize) },
            off as usize,
        )
    }

    fn make_pix(&self, p: &mut [u8], c: &C) {
        let p = p.as_mut_ptr() as *mut u8 as *mut C::ValueType;
        unsafe {
            *p.offset(O::R as isize) = c.r();
            *p.offset(O::G as isize) = c.g();
            *p.offset(O::B as isize) = c.b();
        }
    }

    fn copy_pixel(&mut self, x: i32, y: i32, c: &C) {
        unsafe {
            let p = self.rbuf.row_mut(y).as_mut_ptr().offset(3 * x as isize) as *mut C::ValueType;
            *p.offset(O::R as isize) = c.r();
            *p.offset(O::G as isize) = c.g();
            *p.offset(O::B as isize) = c.b();
        }
    }

    fn row(&self, y: i32) -> &[Self::T] {
        self.rbuf.row(y)
    }

    fn row_mut(&mut self, y: i32) -> &mut [Self::T] {
        self.rbuf.row_mut(y)
    }

    fn row_data(&self, y: i32) -> RowData<Self::T> {
        self.rbuf.row_data(y)
    }

    fn pixel(&self, x: i32, y: i32) -> C {
        unsafe {
            let p = self.rbuf.row(y).as_ptr().offset(3 * x as isize) as *const C::ValueType;
            C::new_init(
                (*p.offset(O::R as isize)),
                (*p.offset(O::G as isize)),
                (*p.offset(O::B as isize)),
                C::ValueType::from_u32(255),
            )
        }
    }

    #[inline]
    fn blend_pixel(&mut self, x: i32, y: i32, c: &C, cover: u8) {
        Self::copy_or_blend_pix_cover(
            slice_t_to_vt_mut!(self.rbuf.row_mut(y), x * 3, C::ValueType),
            c,
            cover as u32,
            &self.blender,
        )
    }

    #[inline]
    fn copy_hline(&mut self, x: i32, y: i32, len: u32, c: &C) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x * 3, C::ValueType);
        for i in 0..len as usize {
            p[(i * 3) + O::R] = c.r();
            p[(i * 3) + O::G] = c.g();
            p[(i * 3) + O::B] = c.b();
        }
    }

    #[inline]
    fn copy_vline(&mut self, x: i32, y: i32, len: u32, c: &C) {
        for i in 0..len as i32 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), x * 3, C::ValueType);

            p[O::R] = c.r();
            p[O::G] = c.g();
            p[O::B] = c.b();
        }
    }

    #[inline]
    fn blend_hline(&mut self, x: i32, y: i32, len: u32, c: &C, cover: u8) {
        if c.a().into_u32() != 0 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x * 3, C::ValueType);
            let alpha = (c.a().into_u32() * (cover as u32 + 1)) >> 8;
            if alpha == C::BASE_MASK {
                for i in 0..len as usize {
                    p[(i * 3) + O::R] = c.r();
                    p[(i * 3) + O::G] = c.g();
                    p[(i * 3) + O::B] = c.b();
                }
            } else {
                for i in 0..len as usize {
                    self.blender.blend_pix_with_cover(
                        &mut p[(i * 3)..],
                        c.r().into_u32(),
                        c.g().into_u32(),
                        c.b().into_u32(),
                        alpha,
                        cover as u32,
                    );
                }
            }
        }
    }

    #[inline]
    fn blend_vline(&mut self, x: i32, y: i32, len: u32, c: &C, cover: u8) {
        if c.a().into_u32() != 0 {
            let alpha = (c.a().into_u32() * (cover as u32 + 1)) >> 8;

            if alpha == C::BASE_MASK {
                for i in 0..len as i32 {
                    let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), x * 3, C::ValueType);

                    p[O::R] = c.r();
                    p[O::G] = c.g();
                    p[O::B] = c.b();
                }
            } else {
                for i in 0..len as i32 {
                    let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), x * 3, C::ValueType);
                    self.blender.blend_pix_with_cover(
                        p,
                        c.r().into_u32(),
                        c.g().into_u32(),
                        c.b().into_u32(),
                        alpha,
                        cover as u32,
                    );
                }
            }
        }
    }

    #[inline]
    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: u32, c: &C, covers: &[u8]) {
        if c.a().into_u32() != 0 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x * 3, C::ValueType);
            for i in 0..len as usize {
                let alpha = (c.a().into_u32() * (covers[i] as u32 + 1)) >> 8;
                if alpha == C::BASE_MASK {
                    p[(i * 3) + O::R] = c.r();
                    p[(i * 3) + O::G] = c.g();
                    p[(i * 3) + O::B] = c.b();
                } else {
                    self.blender.blend_pix_with_cover(
                        &mut p[(i * 3)..],
                        c.r().into_u32(),
                        c.g().into_u32(),
                        c.b().into_u32(),
                        alpha,
                        covers[i] as u32,
                    );
                }
            }
        }
    }

    #[inline]
    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: u32, c: &C, covers: &[u8]) {
        if c.a().into_u32() != 0 {
            for i in 0..len as usize {
                let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x * 3, C::ValueType);
                let alpha = (c.a().into_u32() * (covers[i] as u32 + 1)) >> 8;
                if alpha == C::BASE_MASK {
                    p[O::R] = c.r();
                    p[O::G] = c.g();
                    p[O::B] = c.b();
                } else {
                    self.blender.blend_pix_with_cover(
                        p,
                        c.r().into_u32(),
                        c.g().into_u32(),
                        c.b().into_u32(),
                        alpha,
                        covers[i] as u32,
                    );
                }
            }
        }
    }

    fn copy_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x * 3, C::ValueType);
        for i in 0..len as usize {
            p[(i * 3) + O::R] = colors[i].r();
            p[(i * 3) + O::G] = colors[i].g();
            p[(i * 3) + O::B] = colors[i].b();
        }
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]) {
        for i in 0..len as usize {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x * 3, C::ValueType);

            p[O::R] = colors[i].r();
            p[O::G] = colors[i].g();
            p[O::B] = colors[i].b();
        }
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[C], covers: &[u8], cover: u8,
    ) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x * 3, C::ValueType);

        if covers.len() > 0 {
            for i in 0..len as usize {
                Self::copy_or_blend_pix_cover(
                    &mut p[(i * 3)..],
                    &colors[i as usize],
                    covers[i as usize].into_u32(),
                    &self.blender,
                );
            }
        } else {
            if cover == 255 {
                for i in 0..len as usize {
                    Self::copy_or_blend_pix(&mut p[(i * 3)..], &colors[i as usize], &self.blender);
                }
            } else {
                for i in 0..len as usize {
                    Self::copy_or_blend_pix_cover(
                        &mut p[(i * 3)..],
                        &colors[i as usize],
                        cover as u32,
                        &self.blender,
                    );
                }
            }
        }
    }

    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[C], covers: &[u8], cover: u8,
    ) {
        let len = len as u32;
        if covers.len() > 0 {
            for i in 0..len as usize {
                let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x * 3, C::ValueType);

                Self::copy_or_blend_pix_cover(
                    p,
                    &colors[i as usize],
                    covers[i as usize].into_u32(),
                    &self.blender,
                );
            }
        } else {
            if cover == 255 {
                for i in 0..len as usize {
                    let p =
                        slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x * 3, C::ValueType);

                    Self::copy_or_blend_pix(p, &colors[i as usize], &self.blender);
                }
            } else {
                for i in 0..len as usize {
                    let p =
                        slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x * 3, C::ValueType);

                    Self::copy_or_blend_pix_cover(
                        p,
                        &colors[i as usize],
                        cover as u32,
                        &self.blender,
                    );
                }
            }
        }
    }

    fn copy_from<Pix: RenderBuffer<T = Self::T>>(
        &mut self, from: &Pix, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32,
    ) {
        let p = from.row(ysrc).as_ptr();
        if !p.is_null() {
            unsafe {
                let dst = self
                    .rbuf
                    .row_mut(ydst)
                    .as_mut_ptr()
                    .offset((xdst * Self::PIXEL_WIDTH as i32) as isize);
                let src = p.offset((xsrc * Self::PIXEL_WIDTH as i32) as isize);
                std::ptr::copy(src, dst, len as usize * Self::PIXEL_WIDTH);
            }
        }
    }

    fn blend_from<R: PixFmt<T = Self::T>>(
        &mut self, from: &R, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32, cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst * 3, C::ValueType);
            let psrc = slice_t_to_vt!(psrc, xsrc * 4, <<R as PixFmt>::C as Args>::ValueType);
            if cover == 255 {
                for i in 0..len as usize {
                    let alpha = psrc[i * 3 + R::O::A].into_u32();
                    if alpha != 0 {
                        if alpha == C::BASE_MASK {
                            pdst[i * 4 + O::R] =
                                C::ValueType::from_u32(psrc[i * 3 + R::O::R].into_u32());
                            pdst[i * 4 + O::G] =
                                C::ValueType::from_u32(psrc[i * 3 + R::O::G].into_u32());
                            pdst[i * 4 + O::B] =
                                C::ValueType::from_u32(psrc[i * 3 + R::O::B].into_u32());
                        } else {
                            self.blender.blend_pix(
                                &mut pdst[i * 4..],
                                psrc[i * 3 + R::O::R].into_u32(),
                                psrc[i * 3 + R::O::G].into_u32(),
                                psrc[i * 3 + R::O::B].into_u32(),
                                alpha,
                            );
                        }
                    }
                }
            } else {
                let mut color = C::new();
                for i in 0..len as usize {
                    *color.r_mut() = C::ValueType::from_u32(psrc[i * 3 + R::O::R].into_u32());
                    *color.g_mut() = C::ValueType::from_u32(psrc[i * 3 + R::O::G].into_u32());
                    *color.b_mut() = C::ValueType::from_u32(psrc[i * 3 + R::O::B].into_u32());
                    *color.a_mut() = C::ValueType::from_u32(psrc[i * 3 + R::O::A].into_u32());
                    Self::copy_or_blend_pix_cover(&mut pdst[i * 4..], &color, cover, &self.blender);
                }
            }
        }
    }

    fn blend_from_color<R: PixFmt>(
        &mut self, from: &R, color: &C, xdst: i32, ydst: i32, _xsrc: i32, ysrc: i32, len: u32,
        cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let psrc = slice_t_to_vt!(psrc, 0, <<R as PixFmt>::C as Args>::ValueType);
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst * 3, C::ValueType);

            for i in 0..len as usize {
                let cover: u32 = psrc[i].into_u32() * cover + C::BASE_MASK >> C::BASE_SHIFT;
                Self::copy_or_blend_pix_cover(&mut pdst[i * 3..], color, cover, &self.blender);
            }
        }
    }

    fn blend_from_lut<R: PixFmt>(
        &mut self, from: &R, color_lut: &[C], xdst: i32, ydst: i32, _xsrc: i32, ysrc: i32,
        len: u32, cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let psrc = slice_t_to_vt!(psrc, 0, <<R as PixFmt>::C as Args>::ValueType);
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst * 3, C::ValueType);

            if cover == 255 {
                for i in 0..len as usize {
                    let color = color_lut[psrc[i].into_u32() as usize];
                    self.blender.blend_pix(
                        &mut pdst[i * 3..],
                        color.r().into_u32(),
                        color.g().into_u32(),
                        color.b().into_u32(),
                        color.a().into_u32(),
                    );
                }
            } else {
                for i in 0..len as usize {
                    Self::copy_or_blend_pix_cover(
                        &mut pdst[i * 3..],
                        &color_lut[psrc[i].into_u32() as usize],
                        cover,
                        &self.blender,
                    );
                }
            }
        }
    }
}

/*

//-----------------------------------------------------pixfmt_rgb24_gamma
 template<class Gamma> class pixfmt_rgb24_gamma :
 public AlphaBlendRgb<BlenderRgbGamma<Rgba8, OrderRgb, Gamma>, RenderBuf>
 {
 public:
     pixfmt_rgb24_gamma(RenderBuf& rb, const Gamma& g) :
         AlphaBlendRgb<BlenderRgbGamma<Rgba8, OrderRgb, Gamma>, RenderBuf>(rb)
     {
         this->blender().gamma(g);
     }
 };

 //-----------------------------------------------------pixfmt_bgr24_gamma
 template<class Gamma> class pixfmt_bgr24_gamma :
 public AlphaBlendRgb<BlenderRgbGamma<Rgba8, OrderBgr, Gamma>, RenderBuf>
 {
 public:
     pixfmt_bgr24_gamma(RenderBuf& rb, const Gamma& g) :
         AlphaBlendRgb<BlenderRgbGamma<Rgba8, OrderBgr, Gamma>, RenderBuf>(rb)
     {
         this->blender().gamma(g);
     }
 };

 //-----------------------------------------------------pixfmt_rgb48_gamma
 template<class Gamma> class pixfmt_rgb48_gamma :
 public AlphaBlendRgb<BlenderRgbGamma<rgba16, OrderRgb, Gamma>, RenderBuf>
 {
 public:
     pixfmt_rgb48_gamma(RenderBuf& rb, const Gamma& g) :
         AlphaBlendRgb<BlenderRgbGamma<rgba16, OrderRgb, Gamma>, RenderBuf>(rb)
     {
         this->blender().gamma(g);
     }
 };

 //-----------------------------------------------------pixfmt_bgr48_gamma
 template<class Gamma> class pixfmt_bgr48_gamma :
 public AlphaBlendRgb<BlenderRgbGamma<rgba16, OrderBgr, Gamma>, RenderBuf>
 {
 public:
     pixfmt_bgr48_gamma(RenderBuf& rb, const Gamma& g) :
         AlphaBlendRgb<BlenderRgbGamma<rgba16, OrderBgr, Gamma>, RenderBuf>(rb)
     {
         this->blender().gamma(g);
     }
 };

 */
