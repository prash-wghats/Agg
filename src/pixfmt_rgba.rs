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

use std::marker::PhantomData;

use wrapping_arithmetic::wrappit;

use crate::basics::{uround, RectI, RowData};
use crate::color_rgba::{OrderAbgr, OrderArgb, OrderBgra, OrderRgba, Rgba16, Rgba8};
use crate::rendering_buffer::RenderBuf;
use crate::{
    slice_t_to_vt, slice_t_to_vt_mut, AggInteger, AggPrimitive, Blender, BlenderOp, Color, Equiv,
    ImageSrc, Order, PixFmt, RenderBuffer, RgbArgs, Args
};

macro_rules! from_u32 {
    ($v:expr) => {
        C::ValueType::from_u32($v)
    };
}

pub type BlenderRgba32 = BlenderRgba<Rgba8, OrderRgba>; //----blender_rgba32
pub type BlenderArgb32 = BlenderRgba<Rgba8, OrderArgb>; //----blender_argb32
pub type BlenderAbgr32 = BlenderRgba<Rgba8, OrderAbgr>; //----blender_abgr32
pub type BlenderBgra32 = BlenderRgba<Rgba8, OrderBgra>; //----blender_bgra32

pub type BlenderRgba32Pre = BlenderRgbaPre<Rgba8, OrderRgba>; //----blender_rgba32_pre
pub type BlenderArgb32Pre = BlenderRgbaPre<Rgba8, OrderArgb>; //----blender_argb32_pre
pub type BlenderAbgr32Pre = BlenderRgbaPre<Rgba8, OrderAbgr>; //----blender_abgr32_pre
pub type BlenderBgra32Pre = BlenderRgbaPre<Rgba8, OrderBgra>; //----blender_bgra32_pre

pub type BlenderRgba32Plain = BlenderRgbaPlain<Rgba8, OrderRgba>; //----blender_rgba32_plain
pub type BlenderArgb32Plain = BlenderRgbaPlain<Rgba8, OrderArgb>; //----blender_argb32_plain
pub type BlenderAbgr32Plain = BlenderRgbaPlain<Rgba8, OrderAbgr>; //----blender_abgr32_plain
pub type BlenderBgra32Plain = BlenderRgbaPlain<Rgba8, OrderBgra>; //----blender_bgra32_plain

pub type BlenderRgba64 = BlenderRgba<Rgba16, OrderRgba>; //----blender_rgba64
pub type BlenderArgb64 = BlenderRgba<Rgba16, OrderArgb>; //----blender_argb64
pub type BlenderAbgr64 = BlenderRgba<Rgba16, OrderAbgr>; //----blender_abgr64
pub type BlenderBgra64 = BlenderRgba<Rgba16, OrderBgra>; //----blender_bgra64

pub type BlenderRgba64Pre = BlenderRgbaPre<Rgba16, OrderRgba>; //----blender_rgba64_pre
pub type BlenderArgb64Pre = BlenderRgbaPre<Rgba16, OrderArgb>; //----blender_argb64_pre
pub type BlenderAbgr64Pre = BlenderRgbaPre<Rgba16, OrderAbgr>; //----blender_abgr64_pre
pub type BlenderBgra64Pre = BlenderRgbaPre<Rgba16, OrderBgra>; //----blender_bgra64_pre

pub type PixRgba32<'a> = AlphaBlendRgba<'a, Rgba8, OrderRgba, BlenderRgba32, RenderBuf>; //----pixfmt_rgba32
pub type PixArgb32<'a> = AlphaBlendRgba<'a, Rgba8, OrderArgb, BlenderArgb32, RenderBuf>; //----pixfmt_argb32
pub type PixAbgr32<'a> = AlphaBlendRgba<'a, Rgba8, OrderAbgr, BlenderAbgr32, RenderBuf>; //----pixfmt_abgr32
pub type PixBgra32<'a> = AlphaBlendRgba<'a, Rgba8, OrderBgra, BlenderBgra32, RenderBuf>; //----pixfmt_bgra32

pub type PixRgba32Pre<'a> = AlphaBlendRgba<'a, Rgba8, OrderRgba, BlenderRgba32, RenderBuf, u64>; //----pixfmt_rgba32_pre
pub type PixArgb32Pre<'a> = AlphaBlendRgba<'a, Rgba8, OrderArgb, BlenderArgb32, RenderBuf, u64>; //----pixfmt_argb32_pre
pub type PixAbgr32Pre<'a> = AlphaBlendRgba<'a, Rgba8, OrderAbgr, BlenderAbgr32, RenderBuf, u64>; //----pixfmt_abgr32_pre
pub type PixBgra32Pre<'a> = AlphaBlendRgba<'a, Rgba8, OrderBgra, BlenderBgra32, RenderBuf, u64>; //----pixfmt_bgra32_pre

pub type PixRgba32Plain<'a> = AlphaBlendRgba<'a, Rgba8, OrderRgba, BlenderRgba32, RenderBuf, u64>; //----pixfmt_rgba32_plain
pub type PixArgb32Plain<'a> = AlphaBlendRgba<'a, Rgba8, OrderArgb, BlenderArgb32, RenderBuf, u64>; //----pixfmt_argb32_plain
pub type PixAbgr32Plain<'a> = AlphaBlendRgba<'a, Rgba8, OrderAbgr, BlenderAbgr32, RenderBuf, u64>; //----pixfmt_abgr32_plain
pub type PixBgra32Plain<'a> = AlphaBlendRgba<'a, Rgba8, OrderBgra, BlenderBgra32, RenderBuf, u64>; //----pixfmt_bgra32_plain

pub type PixRgba64<'a> = AlphaBlendRgba<'a, Rgba16, OrderRgba, BlenderRgba64, RenderBuf, u64>; //----pixfmt_rgba64
pub type PixArgb64<'a> = AlphaBlendRgba<'a, Rgba16, OrderArgb, BlenderArgb64, RenderBuf, u64>; //----pixfmt_argb64
pub type PixAbgr64<'a> = AlphaBlendRgba<'a, Rgba16, OrderAbgr, BlenderAbgr64, RenderBuf, u64>; //----pixfmt_abgr64
pub type PixBgra64<'a> = AlphaBlendRgba<'a, Rgba16, OrderBgra, BlenderBgra64, RenderBuf, u64>; //----pixfmt_bgra64

pub type PixRgba64Pre<'a> = AlphaBlendRgba<'a, Rgba16, OrderRgba, BlenderRgba64, RenderBuf, u64>; //----pixfmt_rgba64_pre
pub type PixArgb64Pre<'a> = AlphaBlendRgba<'a, Rgba16, OrderArgb, BlenderArgb64, RenderBuf, u64>; //----pixfmt_argb64_pre
pub type PixAbgr64Pre<'a> = AlphaBlendRgba<'a, Rgba16, OrderAbgr, BlenderAbgr64, RenderBuf, u64>; //----pixfmt_abgr64_pre
pub type PixBgra64Pre<'a> = AlphaBlendRgba<'a, Rgba16, OrderBgra, BlenderBgra64, RenderBuf, u64>; //----pixfmt_bgra64_pre

//=========================================================MultiplierRgba
pub struct MultiplierRgba<C: Color, O: Order> {
    pub color_t: PhantomData<C>,
    pub order: PhantomData<O>,
}

impl<C: Color, O: Order> MultiplierRgba<C, O> {
    pub fn premultiply(p: &mut [C::ValueType]) {
        let a = p[O::A].into_u32();
        if a < C::BASE_MASK {
            if a == 0 {
                p[O::R] = from_u32!(0);
                p[O::G] = from_u32!(0);
                p[O::B] = from_u32!(0);
                return;
            }
            p[O::R] = from_u32!((p[O::R].into_u32() * a + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::G] = from_u32!((p[O::G].into_u32() * a + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::B] = from_u32!((p[O::B].into_u32() * a + C::BASE_MASK) >> C::BASE_SHIFT);
        }
    }

    pub fn demultiply(p: &mut [C::ValueType]) {
        let a = p[O::A].into_u32();
        if a < C::BASE_MASK {
            if a == 0 {
                p[O::R] = from_u32!(0);
                p[O::G] = from_u32!(0);
                p[O::B] = from_u32!(0);
                return;
            }
            let r = (p[O::R].into_u32() * C::BASE_MASK) / a;
            let g = (p[O::G].into_u32() * C::BASE_MASK) / a;
            let b = (p[O::B].into_u32() * C::BASE_MASK) / a;
            p[O::R] = from_u32!(if r > C::BASE_MASK { C::BASE_MASK } else { r });
            p[O::G] = from_u32!(if g > C::BASE_MASK { C::BASE_MASK } else { g });
            p[O::B] = from_u32!(if b > C::BASE_MASK { C::BASE_MASK } else { b });
        }
    }
}

//=====================================================ApplyGammaDirRgba
pub struct ApplyGammaDirRgba<
    'a,
    C: crate::Color,
    Order: crate::Order,
    Gamma: crate::Gamma<C::ValueType, C::ValueType>,
> {
    gamma: &'a Gamma,
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<Order>,
}

impl<'a, C: crate::Color, Order: crate::Order, Gamma: crate::Gamma<C::ValueType, C::ValueType>>
    ApplyGammaDirRgba<'a, C, Order, Gamma>
{
    pub fn new(gamma: &'a Gamma) -> Self {
        ApplyGammaDirRgba {
            gamma: gamma,
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }

    #[inline]
    fn apply(&self, p: &mut [C::ValueType]) {
        p[Order::R as usize] = self.gamma.dir(p[Order::R as usize]);
        p[Order::G as usize] = self.gamma.dir(p[Order::G as usize]);
        p[Order::B as usize] = self.gamma.dir(p[Order::B as usize]);
    }
}

//=====================================================ApplyGammaInvRgba
pub struct ApplyGammaInvRgba<
    'a,
    C: crate::Color,
    Order: crate::Order,
    Gamma: crate::Gamma<C::ValueType, C::ValueType>,
> {
    gamma: &'a Gamma,
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<Order>,
}

impl<'a, C: crate::Color, Order: crate::Order, Gamma: crate::Gamma<C::ValueType, C::ValueType>>
    ApplyGammaInvRgba<'a, C, Order, Gamma>
{
    pub fn new(gamma: &'a Gamma) -> Self {
        ApplyGammaInvRgba {
            gamma: gamma,
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }

    #[inline]
    fn apply(&self, p: &mut [C::ValueType]) {
        p[Order::R as usize] = self.gamma.inv(p[Order::R as usize]);
        p[Order::G as usize] = self.gamma.inv(p[Order::G as usize]);
        p[Order::B as usize] = self.gamma.inv(p[Order::B as usize]);
    }
}

///======================================================BlenderRgba

pub struct BlenderRgba<C: Color, O: Order> {
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<O>,
}

impl<C: Color, O: Order> Blender<C, O> for BlenderRgba<C, O> {
    fn new() -> Self {
        Self {
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }
    #[wrappit]
    #[inline]
    fn blend_pix(&self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32) {
        let r = (p[O::R]).into_u32();
        let g = (p[O::G]).into_u32();
        let b = (p[O::B]).into_u32();
        let a = (p[O::A]).into_u32();

        let r = (((cr - r) * alpha) + (r << C::BASE_SHIFT)) >> C::BASE_SHIFT;
        let g = (((cg - g) * alpha) + (g << C::BASE_SHIFT)) >> C::BASE_SHIFT;
        let b = (((cb - b) * alpha) + (b << C::BASE_SHIFT)) >> C::BASE_SHIFT;
        let a = (a + alpha) - ((alpha * a + C::BASE_MASK) >> C::BASE_SHIFT);
        p[O::R] = from_u32!(r);
        p[O::G] = from_u32!(g);
        p[O::B] = from_u32!(b);
        p[O::A] = from_u32!(a);
    }

    fn blend_pix_with_cover(
        &self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }
}

//======================================================BlenderRgbaPre
pub struct BlenderRgbaPre<C: Color, O: Order> {
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<O>,
}

impl<C: Color, O: Order> crate::Blender<C, O> for BlenderRgbaPre<C, O> {
    fn new() -> Self {
        Self {
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }
    #[wrappit]
    #[inline]
    fn blend_pix_with_cover(
        &self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = C::BASE_MASK - alpha;
        let cover = (cover + 1) << (C::BASE_SHIFT - 8);

        p[O::R] = from_u32!(((p[O::R]).into_u32() * alpha + cr * cover) >> C::BASE_SHIFT);
        p[O::G] = from_u32!(((p[O::G]).into_u32() * alpha + cg * cover) >> C::BASE_SHIFT);
        p[O::B] = from_u32!(((p[O::B]).into_u32() * alpha + cb * cover) >> C::BASE_SHIFT);
        p[O::A] = from_u32!(
            C::BASE_MASK - ((alpha * (C::BASE_MASK - p[O::A].into_u32())) >> C::BASE_SHIFT)
        );
    }

    #[wrappit]
    #[inline]
    fn blend_pix(&self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32) {
        let alpha = C::BASE_MASK - alpha;

        p[O::R] = from_u32!((((p[O::R]).into_u32() * alpha) >> C::BASE_SHIFT) + cr);
        p[O::G] = from_u32!((((p[O::G]).into_u32() * alpha) >> C::BASE_SHIFT) + cg);
        p[O::B] = from_u32!((((p[O::B]).into_u32() * alpha) >> C::BASE_SHIFT) + cb);
        p[O::A] = from_u32!(
            C::BASE_MASK - ((alpha * (C::BASE_MASK - p[O::A].into_u32())) >> C::BASE_SHIFT)
        );
    }
}

//======================================================BlenderRgbaPlain
pub struct BlenderRgbaPlain<C: Color, O: Order> {
    phantom_color: PhantomData<C>,
    phantom_order: PhantomData<O>,
}

impl<C: Color, O: Order> crate::Blender<C, O> for BlenderRgbaPlain<C, O> {
    fn new() -> Self {
        Self {
            phantom_color: PhantomData,
            phantom_order: PhantomData,
        }
    }
    #[wrappit]
    #[inline]
    fn blend_pix_with_cover(
        &self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = C::BASE_MASK - alpha;
        let cover = (cover + 1) << (C::BASE_SHIFT - 8);

        p[O::R] = from_u32!(((p[O::R]).into_u32() * alpha + cr * cover) >> C::BASE_SHIFT);
        p[O::G] = from_u32!(((p[O::G]).into_u32() * alpha + cg * cover) >> C::BASE_SHIFT);
        p[O::B] = from_u32!(((p[O::B]).into_u32() * alpha + cb * cover) >> C::BASE_SHIFT);
        p[O::A] = from_u32!(
            C::BASE_MASK - ((alpha * (C::BASE_MASK - p[O::A].into_u32())) >> C::BASE_SHIFT)
        );
    }

    #[wrappit]
    #[inline]
    fn blend_pix(&self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32) {
        let r = (p[O::R]).into_u32();
        let g = (p[O::G]).into_u32();
        let b = (p[O::B]).into_u32();
        let mut a = (p[O::A]).into_u32();

        a = ((alpha + a) << C::BASE_MASK) - alpha * a;
        p[O::A] = from_u32!(a >> C::BASE_SHIFT);
        p[O::R] = from_u32!((((cr << C::BASE_SHIFT) - r) * alpha + (r << C::BASE_SHIFT)) / a);
        p[O::G] = from_u32!((((cg << C::BASE_SHIFT) - g) * alpha + (g << C::BASE_SHIFT)) / a);
        p[O::B] = from_u32!((((cb << C::BASE_SHIFT) - b) * alpha + (b << C::BASE_SHIFT)) / a);
    }
}

//=========================================================CompOpRgbaClear
pub struct CompOpRgbaClear<C: Color, O: Order> {
    pub dummy: PhantomData<C>,
    pub order_type: PhantomData<O>,
}

impl<C: Color, O: Order> CompOpRgbaClear<C, O> {
    pub fn new() -> Self {
        CompOpRgbaClear {
            dummy: PhantomData,
            order_type: PhantomData,
        }
    }
}

impl<C: Color, O: Order> CompOpRgbaClear<C, O> {
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], _sr: u32, _sg: u32, _sb: u32, _sa: u32, cover: u32) {
        if cover < 255 {
            let cover = 255 - cover;
            p[O::R] = from_u32!((p[O::R].into_u32() * cover + 255) >> 8);
            p[O::G] = from_u32!((p[O::G].into_u32() * cover + 255) >> 8);
            p[O::B] = from_u32!((p[O::B].into_u32() * cover + 255) >> 8);
            p[O::A] = from_u32!((p[O::A].into_u32() * cover + 255) >> 8);
        } else {
            p[0] = from_u32!(0);
            p[1] = from_u32!(0);
            p[2] = from_u32!(0);
            p[3] = from_u32!(0);
        }
    }
}

//===========================================================CompOpRgbaSrc
pub struct CompOpRgbaSrc<C: Color, O: Order> {
    pub dummy: PhantomData<C>,
    pub order_type: PhantomData<O>,
}

impl<C: Color, O: Order> CompOpRgbaSrc<C, O> {
    pub fn new() -> Self {
        CompOpRgbaSrc {
            dummy: PhantomData,
            order_type: PhantomData,
        }
    }
}

impl<C: Color, O: Order> CompOpRgbaSrc<C, O> {
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        if cover < 255 {
            let alpha = 255 - cover;
            p[O::R] =
                from_u32!(((p[O::R].into_u32() * alpha + 255) >> 8) + ((sr * cover + 255) >> 8));
            p[O::G] =
                from_u32!(((p[O::G].into_u32() * alpha + 255) >> 8) + ((sg * cover + 255) >> 8));
            p[O::B] =
                from_u32!(((p[O::B].into_u32() * alpha + 255) >> 8) + ((sb * cover + 255) >> 8));
            p[O::A] =
                from_u32!(((p[O::A].into_u32() * alpha + 255) >> 8) + ((sa * cover + 255) >> 8));
        } else {
            p[O::R] = from_u32!(sr);
            p[O::G] = from_u32!(sg);
            p[O::B] = from_u32!(sb);
            p[O::A] = from_u32!(sa);
        }
    }
}

//===========================================================CompOpRgbaDst
pub struct CompOpRgbaDst<C: Color, O: Order> {
    pub dummy: PhantomData<C>,
    pub order_type: PhantomData<O>,
}

impl<C: Color, O: Order> CompOpRgbaDst<C, O> {
    pub fn new() -> Self {
        CompOpRgbaDst {
            dummy: PhantomData,
            order_type: PhantomData,
        }
    }
}

impl<C: Color, O: Order> CompOpRgbaDst<C, O> {
    #[inline]
    fn blend_pix(_p: &mut [C::ValueType], _sr: u32, _sg: u32, _sb: u32, _sa: u32, _cover: u32) {}
}

//======================================================CompOpRgbaSrcOver
pub struct CompOpRgbaSrcOver<C: Color, O: Order> {
    pub dummy: PhantomData<C>,
    pub order_type: PhantomData<O>,
}

impl<C: Color, O: Order> CompOpRgbaSrcOver<C, O> {
    pub fn new() -> Self {
        CompOpRgbaSrcOver {
            dummy: PhantomData,
            order_type: PhantomData,
        }
    }
}

impl<C: Color, O: Order> CompOpRgbaSrcOver<C, O> {
    //   Dca' = Sca + Dca.(1 - Sa)
    //   Da'  = Sa + Da - Sa.Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        let s1a = C::BASE_MASK - sa;
        p[O::R] = from_u32!(sr + ((p[O::R].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
        p[O::G] = from_u32!(sg + ((p[O::G].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
        p[O::B] = from_u32!(sb + ((p[O::B].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
        p[O::A] = from_u32!(
            sa + p[O::A].into_u32() - ((sa * p[O::A].into_u32() + C::BASE_MASK) >> C::BASE_SHIFT)
        );
    }
}

//======================================================CompOpRgbaDstOver
pub struct CompOpRgbaDstOver<C: Color, O: Order> {
    pub dummy: PhantomData<C>,
    pub order_type: PhantomData<O>,
}

impl<C: Color, O: Order> CompOpRgbaDstOver<C, O> {
    pub fn new() -> Self {
        CompOpRgbaDstOver {
            dummy: PhantomData,
            order_type: PhantomData,
        }
    }
}

impl<C: Color, O: Order> CompOpRgbaDstOver<C, O> {
    // Dca' = Dca + Sca.(1 - Da)
    // Da'  = Sa + Da - Sa.Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        let d1a = C::BASE_MASK - p[O::A].into_u32();
        p[O::R] = from_u32!(p[O::R].into_u32() + ((sr * d1a + C::BASE_MASK) >> C::BASE_SHIFT));
        p[O::G] = from_u32!(p[O::G].into_u32() + ((sg * d1a + C::BASE_MASK) >> C::BASE_SHIFT));
        p[O::B] = from_u32!(p[O::B].into_u32() + ((sb * d1a + C::BASE_MASK) >> C::BASE_SHIFT));
        p[O::A] = from_u32!(
            sa + p[O::A].into_u32() - ((sa * p[O::A].into_u32() + C::BASE_MASK) >> C::BASE_SHIFT)
        );
    }
}

//======================================================CompOpRgbaSrcIn
pub struct CompOpRgbaSrcIn<C: Color, O: Order> {
    pub dummy: PhantomData<C>,
    pub order_type: PhantomData<O>,
}

impl<C: Color, O: Order> CompOpRgbaSrcIn<C, O> {
    pub fn new() -> Self {
        CompOpRgbaSrcIn {
            dummy: PhantomData,
            order_type: PhantomData,
        }
    }
}

impl<C: Color, O: Order> CompOpRgbaSrcIn<C, O> {
    // Dca' = Sca.Da
    // Da'  = Sa.Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let da = p[O::A].into_u32();
        if cover < 255 {
            let alpha = 255 - cover;
            p[O::R] = from_u32!(
                ((p[O::R].into_u32() * alpha + 255) >> 8)
                    + ((((sr * da + C::BASE_MASK) >> C::BASE_SHIFT) * cover + 255) >> 8)
            );
            p[O::G] = from_u32!(
                ((p[O::G].into_u32() * alpha + 255) >> 8)
                    + ((((sg * da + C::BASE_MASK) >> C::BASE_SHIFT) * cover + 255) >> 8)
            );
            p[O::B] = from_u32!(
                ((p[O::B].into_u32() * alpha + 255) >> 8)
                    + ((((sb * da + C::BASE_MASK) >> C::BASE_SHIFT) * cover + 255) >> 8)
            );
            p[O::A] = from_u32!(
                ((p[O::A].into_u32() * alpha + 255) >> 8)
                    + ((((sa * da + C::BASE_MASK) >> C::BASE_SHIFT) * cover + 255) >> 8)
            );
        } else {
            p[O::R] = from_u32!((sr * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::G] = from_u32!((sg * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::B] = from_u32!((sb * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::A] = from_u32!((sa * da + C::BASE_MASK) >> C::BASE_SHIFT);
        }
    }
}

//======================================================CompOpRgbaDstIn
pub struct CompOpRgbaDstIn<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaDstIn<C, O> {
    pub fn new() -> Self {
        CompOpRgbaDstIn { dummy: PhantomData }
    }
}
impl<C: Color, O: Order> CompOpRgbaDstIn<C, O> {
    // Dca' = Dca.Sa
    // Da'  = Sa.Da
    #[inline(always)]
    fn blend_pix(p: &mut [C::ValueType], _: u32, _: u32, _: u32, sa: u32, cover: u32) {
        let mut sa = sa;

        if cover < 255 {
            sa = C::BASE_MASK - ((cover * (C::BASE_MASK - sa) + 255) >> 8);
        }
        p[O::R] = from_u32!((p[O::R].into_u32() * sa + C::BASE_MASK) >> C::BASE_SHIFT);
        p[O::G] = from_u32!((p[O::G].into_u32() * sa + C::BASE_MASK) >> C::BASE_SHIFT);
        p[O::B] = from_u32!((p[O::B].into_u32() * sa + C::BASE_MASK) >> C::BASE_SHIFT);
        p[O::A] = from_u32!((p[O::A].into_u32() * sa + C::BASE_MASK) >> C::BASE_SHIFT);
    }
}

//======================================================CompOpRgbaSrcOut
pub struct CompOpRgbaSrcOut<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaSrcOut<C, O> {
    pub fn new() -> Self {
        CompOpRgbaSrcOut { dummy: PhantomData }
    }
}
impl<C: Color, O: Order> CompOpRgbaSrcOut<C, O> {
    // Dca' = Sca.(1 - Da)
    // Da'  = Sa.(1 - Da)
    #[inline(always)]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let da = C::BASE_MASK - p[O::A].into_u32();
        if cover < 255 {
            let alpha = 255 - cover;
            p[O::R] = from_u32!(
                ((p[O::R].into_u32() * alpha + 255) >> 8)
                    + ((((sr * da + C::BASE_MASK) >> C::BASE_SHIFT) * cover + 255) >> 8)
            );
            p[O::G] = from_u32!(
                ((p[O::G].into_u32() * alpha + 255) >> 8)
                    + ((((sg * da + C::BASE_MASK) >> C::BASE_SHIFT) * cover + 255) >> 8)
            );
            p[O::B] = from_u32!(
                ((p[O::B].into_u32() * alpha + 255) >> 8)
                    + ((((sb * da + C::BASE_MASK) >> C::BASE_SHIFT) * cover + 255) >> 8)
            );
            p[O::A] = from_u32!(
                ((p[O::A].into_u32() * alpha + 255) >> 8)
                    + ((((sa * da + C::BASE_MASK) >> C::BASE_SHIFT) * cover + 255) >> 8)
            );
        } else {
            p[O::R] = from_u32!((sr * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::G] = from_u32!((sg * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::B] = from_u32!((sb * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::A] = from_u32!((sa * da + C::BASE_MASK) >> C::BASE_SHIFT);
        }
    }
}

//======================================================CompOpRgbaDstOut
pub struct CompOpRgbaDstOut<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaDstOut<C, O> {
    pub fn new() -> Self {
        CompOpRgbaDstOut { dummy: PhantomData }
    }
}
impl<C: Color, O: Order> CompOpRgbaDstOut<C, O> {
    // Dca' = Dca.(1 - Sa)
    // Da'  = Da.(1 - Sa)
    #[inline(always)]
    fn blend_pix(p: &mut [C::ValueType], _: u32, _: u32, _: u32, sa: u32, cover: u32) {
        let mut sa = sa;

        if cover < 255 {
            sa = (sa * cover + 255) >> 8;
        }
        sa = 255 - sa;
        p[O::R] = from_u32!((p[O::R].into_u32() * sa + C::BASE_SHIFT) >> C::BASE_SHIFT);
        p[O::G] = from_u32!((p[O::G].into_u32() * sa + C::BASE_SHIFT) >> C::BASE_SHIFT);
        p[O::B] = from_u32!((p[O::B].into_u32() * sa + C::BASE_SHIFT) >> C::BASE_SHIFT);
        p[O::A] = from_u32!((p[O::A].into_u32() * sa + C::BASE_SHIFT) >> C::BASE_SHIFT);
    }
}

//=====================================================CompOpRgbaSrcAtop
pub struct CompOpRgbaSrcAtop<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaSrcAtop<C, O> {
    pub fn new() -> Self {
        CompOpRgbaSrcAtop { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaSrcAtop<C, O> {
    // Dca' = Sca.Da + Dca.(1 - Sa)
    // Da'  = Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        let da = p[O::A].into_u32();
        sa = C::BASE_MASK - sa;
        p[O::R] = from_u32!((sr * da + p[O::R].into_u32() * sa + C::BASE_MASK) >> C::BASE_SHIFT);
        p[O::G] = from_u32!((sg * da + p[O::G].into_u32() * sa + C::BASE_MASK) >> C::BASE_SHIFT);
        p[O::B] = from_u32!((sb * da + p[O::B].into_u32() * sa + C::BASE_MASK) >> C::BASE_SHIFT);
    }
}

//=====================================================CompOpRgbaDstAtop
pub struct CompOpRgbaDstAtop<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaDstAtop<C, O> {
    pub fn new() -> Self {
        CompOpRgbaDstAtop { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaDstAtop<C, O> {
    // Dca' = Dca.Sa + Sca.(1 - Da)
    // Da'  = Sa
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb) = (sr, sg, sb);

        let da = C::BASE_MASK - p[O::A].into_u32();
        if cover < 255 {
            let alpha = 255 - cover;
            sr = (p[O::R].into_u32() * sa + sr * da + C::BASE_MASK) >> C::BASE_SHIFT;
            sg = (p[O::G].into_u32() * sa + sg * da + C::BASE_MASK) >> C::BASE_SHIFT;
            sb = (p[O::B].into_u32() * sa + sb * da + C::BASE_MASK) >> C::BASE_SHIFT;
            p[O::R] =
                from_u32!(((p[O::R].into_u32() * alpha + 255) >> 8) + ((sr * cover + 255) >> 8));
            p[O::G] =
                from_u32!(((p[O::G].into_u32() * alpha + 255) >> 8) + ((sg * cover + 255) >> 8));
            p[O::B] =
                from_u32!(((p[O::B].into_u32() * alpha + 255) >> 8) + ((sb * cover + 255) >> 8));
            p[O::A] =
                from_u32!(((p[O::A].into_u32() * alpha + 255) >> 8) + ((sa * cover + 255) >> 8));
        } else {
            p[O::R] =
                from_u32!((p[O::R].into_u32() * sa + sr * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::G] =
                from_u32!((p[O::G].into_u32() * sa + sg * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::B] =
                from_u32!((p[O::B].into_u32() * sa + sb * da + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::A] = from_u32!(sa);
        }
    }
}

//=========================================================CompOpRgbaXor
pub struct CompOpRgbaXor<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaXor<C, O> {
    pub fn new() -> Self {
        CompOpRgbaXor { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaXor<C, O> {
    // Dca' = Sca.(1 - Da) + Dca.(1 - Sa)
    // Da'  = Sa + Da - 2.Sa.Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let s1a = C::BASE_MASK - sa;
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            p[O::R] =
                from_u32!((p[O::R].into_u32() * s1a + sr * d1a + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::G] =
                from_u32!((p[O::G].into_u32() * s1a + sg * d1a + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::B] =
                from_u32!((p[O::B].into_u32() * s1a + sb * d1a + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::A] = from_u32!(
                sa + p[O::A].into_u32()
                    - ((sa * p[O::A].into_u32() + C::BASE_MASK / 2) >> (C::BASE_SHIFT - 1))
            );
        }
    }
}

//=========================================================CompOpRgbaPlus
pub struct CompOpRgbaPlus<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaPlus<C, O> {
    pub fn new() -> Self {
        CompOpRgbaPlus { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaPlus<C, O> {
    // Dca' = Sca + Dca
    // Da'  = Sa + Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let dr = p[O::R].into_u32() + sr;
            let dg = p[O::G].into_u32() + sg;
            let db = p[O::B].into_u32() + sb;
            let da = p[O::A].into_u32() + sa;
            p[O::R] = from_u32!(if dr > C::BASE_MASK { C::BASE_MASK } else { dr });
            p[O::G] = from_u32!(if dg > C::BASE_MASK { C::BASE_MASK } else { dg });
            p[O::B] = from_u32!(if db > C::BASE_MASK { C::BASE_MASK } else { db });
            p[O::A] = from_u32!(if da > C::BASE_MASK { C::BASE_MASK } else { da });
        }
    }
}

//========================================================CompOpRgbaMinus
pub struct CompOpRgbaMinus<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaMinus<C, O> {
    pub fn new() -> Self {
        CompOpRgbaMinus { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaMinus<C, O> {
    // Dca' = Dca - Sca
    // Da' = 1 - (1 - Sa).(1 - Da)
    #[wrappit]
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let dr = p[O::R].into_u32() - sr;
            let dg = p[O::G].into_u32() - sg;
            let db = p[O::B].into_u32() - sb;
            p[O::R] = from_u32!(if dr > C::BASE_MASK { 0 } else { dr });
            p[O::G] = from_u32!(if dg > C::BASE_MASK { 0 } else { dg });
            p[O::B] = from_u32!(if db > C::BASE_MASK { 0 } else { db });
            p[O::A] = from_u32!(
                sa + p[O::A].into_u32()
                    - ((sa * p[O::A].into_u32() + C::BASE_MASK) >> C::BASE_SHIFT)
            );
        }
    }
}

//=====================================================CompOpRgbaMultiply
pub struct CompOpRgbaMultiply<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaMultiply<C, O> {
    pub fn new() -> Self {
        CompOpRgbaMultiply { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaMultiply<C, O> {
    // Dca' = Sca.Dca + Sca.(1 - Da) + Dca.(1 - Sa)
    // Da'  = Sa + Da - Sa.Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let s1a = C::BASE_MASK - sa;
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            p[O::R] = from_u32!((sr * dr + sr * d1a + dr * s1a + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::G] = from_u32!((sg * dg + sg * d1a + dg * s1a + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::B] = from_u32!((sb * db + sb * d1a + db * s1a + C::BASE_MASK) >> C::BASE_SHIFT);
            p[O::A] = from_u32!(
                sa + p[O::A].into_u32()
                    - ((sa * p[O::A].into_u32() + C::BASE_MASK) >> C::BASE_SHIFT)
            );
        }
    }
}

//=====================================================CompOpRgbaScreen
pub struct CompOpRgbaScreen<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaScreen<C, O> {
    pub fn new() -> Self {
        CompOpRgbaScreen { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaScreen<C, O> {
    // Dca' = Sca + Dca - Sca.Dca
    // Da'  = Sa + Da - Sa.Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();
            p[O::R] = from_u32!(sr + dr - ((sr * dr + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::G] = from_u32!(sg + dg - ((sg * dg + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::B] = from_u32!(sb + db - ((sb * db + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=====================================================CompOpRgbaOverlay
pub struct CompOpRgbaOverlay<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaOverlay<C, O> {
    pub fn new() -> Self {
        CompOpRgbaOverlay { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaOverlay<C, O> {
    // if 2.Dca < Da
    //   Dca' = 2.Sca.Dca + Sca.(1 - Da) + Dca.(1 - Sa)
    // otherwise
    //   Dca' = Sa.Da - 2.(Da - Dca).(Sa - Sca) + Sca.(1 - Da) + Dca.(1 - Sa)
    //
    // Da' = Sa + Da - Sa.Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            let s1a = C::BASE_MASK - sa;
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();
            let sada = sa * p[O::A].into_u32();

            p[O::R] = from_u32!(
                if 2 * dr < da {
                    2 * sr * dr + sr * d1a + dr * s1a
                } else {
                    sada - 2 * (da - dr) * (sa - sr) + sr * d1a + dr * s1a + C::BASE_MASK
                } >> C::BASE_SHIFT
            );

            p[O::G] = from_u32!(
                if 2 * dg < da {
                    2 * sg * dg + sg * d1a + dg * s1a
                } else {
                    sada - 2 * (da - dg) * (sa - sg) + sg * d1a + dg * s1a + C::BASE_MASK
                } >> C::BASE_SHIFT
            );

            p[O::B] = from_u32!(
                if 2 * db < da {
                    2 * sb * db + sb * d1a + db * s1a
                } else {
                    sada - 2 * (da - db) * (sa - sb) + sb * d1a + db * s1a + C::BASE_MASK
                } >> C::BASE_SHIFT
            );

            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

pub fn sd_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

pub fn sd_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

//=====================================================CompOpRgbaDarken
pub struct CompOpRgbaDarken<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaDarken<C, O> {
    pub fn new() -> Self {
        CompOpRgbaDarken { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaDarken<C, O> {
    // Dca' = min(Sca.Da, Dca.Sa) + Sca.(1 - Da) + Dca.(1 - Sa)
    // Da'  = Sa + Da - Sa.Da
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            let s1a = C::BASE_MASK - sa;
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();

            p[O::R] = from_u32!(
                (sd_min(sr * da, dr * sa) + sr * d1a + dr * s1a + C::BASE_MASK) >> C::BASE_SHIFT
            );
            p[O::G] = from_u32!(
                (sd_min(sg * da, dg * sa) + sg * d1a + dg * s1a + C::BASE_MASK) >> C::BASE_SHIFT
            );
            p[O::B] = from_u32!(
                (sd_min(sb * da, db * sa) + sb * d1a + db * s1a + C::BASE_MASK) >> C::BASE_SHIFT
            );
            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=====================================================CompOpRgbaLighten
pub struct CompOpRgbaLighten<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaLighten<C, O> {
    pub fn new() -> Self {
        CompOpRgbaLighten { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaLighten<C, O> {
    // Dca' = max(Sca.Da, Dca.Sa) + Sca.(1 - Da) + Dca.(1 - Sa)
    // Da'  = Sa + Da - Sa.Da
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            let s1a = C::BASE_MASK - sa;
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();

            p[O::R] = from_u32!(
                (sd_max(sr * da, dr * sa) + sr * d1a + dr * s1a + C::BASE_MASK) >> C::BASE_SHIFT
            );
            p[O::G] = from_u32!(
                (sd_max(sg * da, dg * sa) + sg * d1a + dg * s1a + C::BASE_MASK) >> C::BASE_SHIFT
            );
            p[O::B] = from_u32!(
                (sd_max(sb * da, db * sa) + sb * d1a + db * s1a + C::BASE_MASK) >> C::BASE_SHIFT
            );
            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=====================================================CompOpRgbaDodge
pub struct CompOpRgbaDodge<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaDodge<C, O> {
    pub fn new() -> Self {
        CompOpRgbaDodge { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaDodge<C, O> {
    // if Sca.Da + Dca.Sa >= Sa.Da
    //   Dca' = Sa.Da + Sca.(1 - Da) + Dca.(1 - Sa)
    // otherwise
    //   Dca' = Dca.Sa/(1-Sca/Sa) + Sca.(1 - Da) + Dca.(1 - Sa)
    //
    // Da'  = Sa + Da - Sa.Da
    #[inline(always)]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            let s1a = C::BASE_MASK - sa;
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();
            let drsa = (dr) * (sa);
            let dgsa = (dg) * (sa);
            let dbsa = (db) * (sa);
            let srda = (sr) * (da);
            let sgda = (sg) * (da);
            let sbda = (sb) * (da);
            let sada = (sa) * (da);

            p[O::R] = from_u32!(if srda + drsa >= sada {
                (sada + (sr * d1a) + (dr * s1a) + C::BASE_MASK) >> C::BASE_SHIFT
            } else {
                drsa / (C::BASE_MASK - (sr << C::BASE_SHIFT) / sa)
                    + ((sr * d1a + dr * s1a + C::BASE_MASK) >> C::BASE_SHIFT)
            });

            p[O::G] = from_u32!(if sgda + dgsa >= sada {
                (sada + (sg * d1a) + (dg * s1a) + C::BASE_MASK) >> C::BASE_SHIFT
            } else {
                dgsa / (C::BASE_MASK - (sg << C::BASE_SHIFT) / (sa))
                    + ((sg * d1a + dg * s1a + C::BASE_MASK) >> C::BASE_SHIFT)
            });

            p[O::B] = from_u32!(if sbda + dbsa >= sada {
                (sada + (sb * d1a) + (db * s1a) + C::BASE_MASK) >> C::BASE_SHIFT
            } else {
                dbsa / (C::BASE_MASK - (sb << C::BASE_SHIFT) / (sa))
                    + ((sb * d1a + db * s1a + C::BASE_MASK) >> C::BASE_SHIFT)
            });

            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=====================================================CompOpRgbaBurn

pub struct CompOpRgbaBurn<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaBurn<C, O> {
    pub fn new() -> Self {
        CompOpRgbaBurn { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaBurn<C, O> {
    // if Sca.Da + Dca.Sa <= Sa.Da
    //   Dca' = Sca.(1 - Da) + Dca.(1 - Sa)
    // otherwise
    //   Dca' = Sa.(Sca.Da + Dca.Sa - Sa.Da)/Sca + Sca.(1 - Da) + Dca.(1 - Sa)
    //
    // Da'  = Sa + Da - Sa.Da
    #[inline(always)]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            let s1a = C::BASE_MASK - sa;
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();
            let drsa = (dr * sa) as u32;
            let dgsa = (dg * sa) as u32;
            let dbsa = (db * sa) as u32;
            let srda = (sr * da) as u32;
            let sgda = (sg * da) as u32;
            let sbda = (sb * da) as u32;
            let sada = (sa * da) as u32;

            p[O::R] = from_u32!(
                if srda + drsa <= sada {
                    sr * d1a + dr * s1a
                } else {
                    sa * (srda + drsa - sada) / sr + (sr * d1a) + (dr * s1a) + C::BASE_MASK
                } >> C::BASE_SHIFT
            );

            p[O::G] = from_u32!(
                if sgda + dgsa <= sada {
                    sg * d1a + dg * s1a
                } else {
                    sa * (sgda + dgsa - sada) / sg + (sg * d1a) + (dg * s1a) + C::BASE_MASK
                } >> C::BASE_SHIFT
            );

            p[O::B] = from_u32!(
                if sbda + dbsa <= sada {
                    sb * d1a + db * s1a
                } else {
                    sa * (sbda + dbsa - sada) / sb + (sb * d1a) + (db * s1a) + C::BASE_MASK
                } >> C::BASE_SHIFT
            );

            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=====================================================CompOpRgbaDifference

pub struct CompOpRgbaDifference<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaDifference<C, O> {
    pub fn new() -> Self {
        CompOpRgbaDifference { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaDifference<C, O> {
    // Dca' = Sca + Dca - 2.min(Sca.Da, Dca.Sa)
    // Da'  = Sa + Da - Sa.Da
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();
            p[O::R] = from_u32!(
                sr + dr - ((2 * sd_min(sr * da, dr * sa) + C::BASE_MASK) >> C::BASE_SHIFT)
            );
            p[O::G] = from_u32!(
                sg + dg - ((2 * sd_min(sg * da, dg * sa) + C::BASE_MASK) >> C::BASE_SHIFT)
            );
            p[O::B] = from_u32!(
                sb + db - ((2 * sd_min(sb * da, db * sa) + C::BASE_MASK) >> C::BASE_SHIFT)
            );
            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=====================================================CompOpRgbaHardLight

pub struct CompOpRgbaHardLight<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaHardLight<C, O> {
    pub fn new() -> Self {
        CompOpRgbaHardLight { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaHardLight<C, O> {
    // if 2.Sca < Sa
    //    Dca' = 2.Sca.Dca + Sca.(1 - Da) + Dca.(1 - Sa)
    // otherwise
    //    Dca' = Sa.Da - 2.(Da - Dca).(Sa - Sca) + Sca.(1 - Da) + Dca.(1 - Sa)
    //
    // Da'  = Sa + Da - Sa.Da
    #[inline(always)]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            let s1a = C::BASE_MASK - sa;
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();
            let sada = sa * da;

            p[O::R] = from_u32!(
                if 2 * sr < sa {
                    2 * sr * dr + sr * d1a + dr * s1a
                } else {
                    sada - 2 * (da - dr) * (sa - sr) + sr * d1a + dr * s1a + C::BASE_MASK
                } >> C::BASE_SHIFT
            );
            p[O::G] = from_u32!(
                if 2 * sg < sa {
                    2 * sg * dg + sg * d1a + dg * s1a
                } else {
                    sada - 2 * (da - dg) * (sa - sg) + sg * d1a + dg * s1a + C::BASE_MASK
                } >> C::BASE_SHIFT
            );
            p[O::B] = from_u32!(
                if 2 * sb < sa {
                    2 * sb * db + sb * d1a + db * s1a
                } else {
                    sada - 2 * (da - db) * (sa - sb) + sb * d1a + db * s1a + C::BASE_MASK
                } >> C::BASE_SHIFT
            );
            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=====================================================CompOpRgbaSoftLight
pub struct CompOpRgbaSoftLight<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaSoftLight<C, O> {
    pub fn new() -> Self {
        CompOpRgbaSoftLight { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaSoftLight<C, O> {
    // if 2.Sca < Sa
    //   Dca' = Dca.(Sa + (1 - Dca/Da).(2.Sca - Sa)) + Sca.(1 - Da) + Dca.(1 - Sa)
    // otherwise if 8.Dca <= Da
    //   Dca' = Dca.(Sa + (1 - Dca/Da).(2.Sca - Sa).(3 - 8.Dca/Da)) + Sca.(1 - Da) + Dca.(1 - Sa)
    // otherwise
    //   Dca' = (Dca.Sa + ((Dca/Da)^(0.5).Da - Dca).(2.Sca - Sa)) + Sca.(1 - Da) + Dca.(1 - Sa)
    //
    // Da'  = Sa + Da - Sa.Da
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], r: u32, g: u32, b: u32, _a: u32, cover: u32) {
        let mut a = _a;

        let sr = (r * cover) as f64 / (C::BASE_MASK as f64 * 255.0);
        let sg = (g * cover) as f64 / (C::BASE_MASK as f64 * 255.0);
        let sb = (b * cover) as f64 / (C::BASE_MASK as f64 * 255.0);
        let sa = (a * cover) as f64 / (C::BASE_MASK as f64 * 255.0);
        if sa > 0.0 {
            let mut dr = p[O::R].into_u32() as f64 / C::BASE_MASK as f64;
            let mut dg = p[O::G].into_u32() as f64 / C::BASE_MASK as f64;
            let mut db = p[O::B].into_u32() as f64 / C::BASE_MASK as f64;
            let da = if p[O::A].into_u32() == 0 {
                1.0 / C::BASE_MASK as f64
            } else {
                p[O::A].into_u32() as f64 / C::BASE_MASK as f64
            };
            if cover < 255 {
                a = (a * cover + 255) >> 8 as u32;
            }
            if 2.0 * sr < sa {
                dr = dr * (sa + (1.0 - dr / da) * (2.0 * sr - sa))
                    + sr * (1.0 - da)
                    + dr * (1.0 - sa);
            } else if 8.0 * dr <= da {
                dr = dr * (sa + (1.0 - dr / da) * (2.0 * sr - sa) * (3.0 - 8.0 * dr / da))
                    + sr * (1.0 - da)
                    + dr * (1.0 - sa);
            } else {
                dr = (dr * sa + ((dr / da).sqrt() * da) - dr * (2.0 * sr - sa))
                    + sr * (1.0 - da)
                    + dr * (1.0 - sa);
            }
            if 2.0 * sg < sa {
                dg = dg * (sa + (1.0 - dg / da) * (2.0 * sg - sa))
                    + sg * (1.0 - da)
                    + dg * (1.0 - sa);
            } else if 8.0 * dg <= da {
                dg = dg * (sa + (1.0 - dg / da) * (2.0 * sg - sa) * (3.0 - 8.0 * dg / da))
                    + sg * (1.0 - da)
                    + dg * (1.0 - sa);
            } else {
                dg = (dg * sa + ((dg / da).sqrt() * da - dg) * (2.0 * sg - sa))
                    + sg * (1.0 - da)
                    + dg * (1.0 - sa);
            }
            if 2.0 * sb < sa {
                db = db * (sa + (1.0 - db / da) * (2.0 * sb - sa))
                    + sb * (1.0 - da)
                    + db * (1.0 - sa);
            } else if 8.0 * db <= da {
                db = db * (sa + (1.0 - db / da) * (2.0 * sb - sa) * (3.0 - 8.0 * db / da))
                    + sb * (1.0 - da)
                    + db * (1.0 - sa);
            } else {
                db = (db * sa + ((db / da).sqrt() * da - db) * (2.0 * sb - sa))
                    + sb * (1.0 - da)
                    + db * (1.0 - sa);
            }
            p[O::R] = from_u32!(uround(dr * C::BASE_MASK as f64) as u32);
            p[O::G] = from_u32!(uround(dg * C::BASE_MASK as f64) as u32);
            p[O::B] = from_u32!(uround(db * C::BASE_MASK as f64) as u32);
            p[O::A] = from_u32!(
                a + p[O::A].into_u32()
                    - (((a * p[O::A].into_u32()) + C::BASE_MASK) >> C::BASE_SHIFT)
            );
        }
    }
}

//=====================================================CompOpRgbaExclusion
pub struct CompOpRgbaExclusion<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaExclusion<C, O> {
    pub fn new() -> Self {
        CompOpRgbaExclusion { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaExclusion<C, O> {
    // Dca' = (Sca.Da + Dca.Sa - 2.Sca.Dca) + Sca.(1 - Da) + Dca.(1 - Sa)
    // Da'  = Sa + Da - Sa.Da
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let d1a = C::BASE_MASK - p[O::A].into_u32();
            let s1a = C::BASE_MASK - sa;
            let dr = p[O::R].into_u32();
            let dg = p[O::G].into_u32();
            let db = p[O::B].into_u32();
            let da = p[O::A].into_u32();
            p[O::R] = from_u32!(
                (sr * da + dr * sa - 2 * sr * dr + sr * d1a + dr * s1a + C::BASE_MASK)
                    >> C::BASE_SHIFT
            );
            p[O::G] = from_u32!(
                (sg * da + dg * sa - 2 * sg * dg + sg * d1a + dg * s1a + C::BASE_MASK)
                    >> C::BASE_SHIFT
            );
            p[O::B] = from_u32!(
                (sb * da + db * sa - 2 * sb * db + sb * d1a + db * s1a + C::BASE_MASK)
                    >> C::BASE_SHIFT
            );
            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=====================================================CompOpRgbaContrast
pub struct CompOpRgbaContrast<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaContrast<C, O> {
    pub fn new() -> Self {
        CompOpRgbaContrast { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaContrast<C, O> {
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        let dr = p[O::R].into_u32() as i32;
        let dg = p[O::G].into_u32() as i32;
        let db = p[O::B].into_u32() as i32;
        let da = p[O::A].into_u32() as i32;
        let d2a = (da >> 1) as i32;
        let s2a = sa >> 1;

        let r = (((dr - d2a) * ((sr - s2a) * 2 + C::BASE_MASK) as i32) >> C::BASE_SHIFT) + d2a;
        let g = (((dg - d2a) * ((sg - s2a) * 2 + C::BASE_MASK) as i32) >> C::BASE_SHIFT) + d2a;
        let b = (((db - d2a) * ((sb - s2a) * 2 + C::BASE_MASK) as i32) >> C::BASE_SHIFT) + d2a;

        let r = if r < 0 { 0 } else { r };
        let g = if g < 0 { 0 } else { g };
        let b = if b < 0 { 0 } else { b };

        p[O::R] = from_u32!(if r > da { da as u32 } else { r as u32 });
        p[O::G] = from_u32!(if g > da { da as u32 } else { g as u32 });
        p[O::B] = from_u32!(if b > da { da as u32 } else { b as u32 });
    }
}

//=====================================================CompOpRgbaInvert
pub struct CompOpRgbaInvert<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaInvert<C, O> {
    pub fn new() -> Self {
        CompOpRgbaInvert { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaInvert<C, O> {
    // Dca' = (Da - Dca) * Sa + Dca.(1 - Sa)
    // Da'  = Sa + Da - Sa.Da
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], _sr: u32, _sg: u32, _sb: u32, sa: u32, cover: u32) {
        let mut sa = sa;

        sa = (sa * cover + 255) >> 8;
        if sa != 0 {
            let da = p[O::A].into_u32();
            let dr = ((da - p[O::R].into_u32()) * sa + C::BASE_MASK) >> C::BASE_SHIFT;
            let dg = ((da - p[O::G].into_u32()) * sa + C::BASE_MASK) >> C::BASE_SHIFT;
            let db = ((da - p[O::B].into_u32()) * sa + C::BASE_MASK) >> C::BASE_SHIFT;
            let s1a = 255 - sa;
            p[O::R] = from_u32!(dr + ((p[O::R].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::G] = from_u32!(dg + ((p[O::G].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::B] = from_u32!(db + ((p[O::B].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//=================================================CompOpRgbaInvertRgb
pub struct CompOpRgbaInvertRgb<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> CompOpRgbaInvertRgb<C, O> {
    pub fn new() -> Self {
        CompOpRgbaInvertRgb { dummy: PhantomData }
    }
}

impl<C: Color, O: Order> CompOpRgbaInvertRgb<C, O> {
    // Dca' = (Da - Dca) * Sca + Dca.(1 - Sa)
    // Da'  = Sa + Da - Sa.Da
    #[inline]
    fn blend_pix(p: &mut [C::ValueType], sr: u32, sg: u32, sb: u32, sa: u32, cover: u32) {
        let (mut sr, mut sg, mut sb, mut sa) = (sr, sg, sb, sa);

        if cover < 255 {
            sr = (sr * cover + 255) >> 8;
            sg = (sg * cover + 255) >> 8;
            sb = (sb * cover + 255) >> 8;
            sa = (sa * cover + 255) >> 8;
        }
        if sa != 0 {
            let da = p[O::A].into_u32();
            let dr = ((da - p[O::R].into_u32()) * sr + C::BASE_MASK) >> C::BASE_SHIFT;
            let dg = ((da - p[O::G].into_u32()) * sg + C::BASE_MASK) >> C::BASE_SHIFT;
            let db = ((da - p[O::B].into_u32()) * sb + C::BASE_MASK) >> C::BASE_SHIFT;
            let s1a = 255 - sa;
            p[O::R] = from_u32!(dr + ((p[O::R].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::G] = from_u32!(dg + ((p[O::G].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::B] = from_u32!(db + ((p[O::B].into_u32() * s1a + C::BASE_MASK) >> C::BASE_SHIFT));
            p[O::A] = from_u32!(sa + da - ((sa * da + C::BASE_MASK) >> C::BASE_SHIFT));
        }
    }
}

//======================================================CompOpTableRgba
//type CompOpFuncType = fn(p: &mut [C::ValueType], cr: u8, cg: u8, cb: u8, ca: u8, cover: u8);
pub struct CompOpTableRgba<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}
impl<C: Color, O: Order> CompOpTableRgba<C, O> {
    pub const COMP_OP_FUNC: [fn(
        p: &mut [C::ValueType],
        cr: u32,
        cg: u32,
        cb: u32,
        ca: u32,
        cover: u32,
    ); 28] = [
        CompOpRgbaClear::<C, O>::blend_pix,     // clear
        CompOpRgbaSrc::<C, O>::blend_pix,       // src
        CompOpRgbaDst::<C, O>::blend_pix,       // dst
        CompOpRgbaSrcOver::<C, O>::blend_pix,   // src_over
        CompOpRgbaDstOver::<C, O>::blend_pix,   // dst_over
        CompOpRgbaSrcIn::<C, O>::blend_pix,     // src_in
        CompOpRgbaDstIn::<C, O>::blend_pix,     // dst_in
        CompOpRgbaSrcOut::<C, O>::blend_pix,    // src_out
        CompOpRgbaDstOut::<C, O>::blend_pix,    // dst_out
        CompOpRgbaSrcAtop::<C, O>::blend_pix,   // src_atop
        CompOpRgbaDstAtop::<C, O>::blend_pix,   // dst_atop
        CompOpRgbaXor::<C, O>::blend_pix,       // xor
        CompOpRgbaPlus::<C, O>::blend_pix,      // plus
        CompOpRgbaMinus::<C, O>::blend_pix,     // minus
        CompOpRgbaMultiply::<C, O>::blend_pix,  // multiply
        CompOpRgbaScreen::<C, O>::blend_pix,    // screen
        CompOpRgbaOverlay::<C, O>::blend_pix,   // overlay
        CompOpRgbaDarken::<C, O>::blend_pix,    // darken
        CompOpRgbaLighten::<C, O>::blend_pix,   // lighten
        CompOpRgbaDodge::<C, O>::blend_pix,     // color_dodge
        CompOpRgbaBurn::<C, O>::blend_pix,      // color_burn
        CompOpRgbaHardLight::<C, O>::blend_pix, // hard_light
        CompOpRgbaSoftLight::<C, O>::blend_pix, // soft_light
        CompOpRgbaDifference::<C, O>::blend_pix,
        CompOpRgbaExclusion::<C, O>::blend_pix,
        CompOpRgbaContrast::<C, O>::blend_pix,
        CompOpRgbaInvert::<C, O>::blend_pix,
        CompOpRgbaInvertRgb::<C, O>::blend_pix,
    ];
}

//==============================================================CompOp
pub enum CompOp {
    CompOpClear,      //----CompOpClear
    CompOpSrc,        //----CompOpSrc
    CompOpDst,        //----CompOpDst
    CompOpSrcOver,    //----CompOpSrcOver
    CompOpDstOver,    //----CompOpDstOver
    CompOpSrcIn,      //----CompOpSrcIn
    CompOpDstIn,      //----CompOpDstIn
    CompOpSrcOut,     //----CompOpSrcOut
    CompOpDstOut,     //----CompOpDstOut
    CompOpSrcAtop,    //----CompOpSrcAtop
    CompOpDstAtop,    //----CompOpDstAtop
    CompOpXor,        //----CompOpXor
    CompOpPlus,       //----CompOpPlus
    CompOpMinus,      //----CompOpMinus
    CompOpMultiply,   //----CompOpMultiply
    CompOpScreen,     //----CompOpScreen
    CompOpOverlay,    //----CompOpOverlay
    CompOpDarken,     //----CompOpDarken
    CompOpLighten,    //----CompOpLighten
    CompOpColorDodge, //----CompOpColorDodge
    CompOpColorBurn,  //----CompOpColorBurn
    CompOpHardLight,  //----CompOpHardLight
    CompOpSoftLight,  //----CompOpSoftLight
    CompOpDifference, //----CompOpDifference
    CompOpExclusion,  //----CompOpExclusion
    CompOpContrast,   //----CompOpContrast
    CompOpInvert,     //----CompOpInvert
    CompOpInvertRgb,  //----CompOpInvertRgb

    EndofCompOp,
}

//====================================================CompOpRgbaAdaptor
pub struct CompOpRgbaAdaptor<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> BlenderOp<C, O> for CompOpRgbaAdaptor<C, O> {
    fn new() -> Self {
        CompOpRgbaAdaptor { dummy: PhantomData }
    }

    fn blend_pix_with_cover(
        &self, op: u32, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, ca: u32, cover: u32,
    ) {
        let func = CompOpTableRgba::<C, O>::COMP_OP_FUNC[op as usize];
        func(
            p,
            (cr * ca + C::BASE_MASK) >> C::BASE_SHIFT,
            (cg * ca + C::BASE_MASK) >> C::BASE_SHIFT,
            (cb * ca + C::BASE_MASK) >> C::BASE_SHIFT,
            ca,
            cover,
        );
    }
}

//=========================================comp_op_adaptor_clip_to_dst_rgba
pub struct CompOpAdaptorClipToDstRgba<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> BlenderOp<C, O> for CompOpAdaptorClipToDstRgba<C, O> {
    fn new() -> Self {
        CompOpAdaptorClipToDstRgba { dummy: PhantomData }
    }

    fn blend_pix_with_cover(
        &self, op: u32, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, ca: u32, cover: u32,
    ) {
        let cr = (cr * ca + C::BASE_MASK) >> C::BASE_SHIFT;
        let cg = (cg * ca + C::BASE_MASK) >> C::BASE_SHIFT;
        let cb = (cb * ca + C::BASE_MASK) >> C::BASE_SHIFT;
        let da = p[O::A].into_u32();
        CompOpTableRgba::<C, O>::COMP_OP_FUNC[op as usize](
            p,
            (cr * da + C::BASE_MASK) >> C::BASE_SHIFT,
            (cg * da + C::BASE_MASK) >> C::BASE_SHIFT,
            (cb * da + C::BASE_MASK) >> C::BASE_SHIFT,
            (ca * da + C::BASE_MASK) >> C::BASE_SHIFT,
            cover,
        );
    }
}

//================================================comp_op_adaptor_rgba_pre
pub struct CompOpAdaptorRgbaPre<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> BlenderOp<C, O> for CompOpAdaptorRgbaPre<C, O> {
    fn new() -> Self {
        CompOpAdaptorRgbaPre { dummy: PhantomData }
    }

    fn blend_pix_with_cover(
        &self, op: u32, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, ca: u32, cover: u32,
    ) {
        CompOpTableRgba::<C, O>::COMP_OP_FUNC[op as usize](p, cr, cg, cb, ca, cover);
    }
}

//=========================================comp_op_adaptor_clip_to_dst_rgba_pre
pub struct CompOpAdaptorClipToDstRgbaPre<C: Color, O: Order> {
    dummy: PhantomData<(C, O)>,
}

impl<C: Color, O: Order> BlenderOp<C, O> for CompOpAdaptorClipToDstRgbaPre<C, O> {
    fn new() -> Self {
        CompOpAdaptorClipToDstRgbaPre { dummy: PhantomData }
    }

    fn blend_pix_with_cover(
        &self, op: u32, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, ca: u32, cover: u32,
    ) {
        let da = p[O::A].into_u32();
        CompOpTableRgba::<C, O>::COMP_OP_FUNC[op as usize](
            p,
            (cr * da + C::BASE_MASK) >> C::BASE_SHIFT,
            (cg * da + C::BASE_MASK) >> C::BASE_SHIFT,
            (cb * da + C::BASE_MASK) >> C::BASE_SHIFT,
            (ca * da + C::BASE_MASK) >> C::BASE_SHIFT,
            cover,
        );
    }
}

//===============================================CopyOrBlendRgbaWrapper
pub struct CopyOrBlendRgbaWrapper<C: Color, O: Order, B: Blender<C, O>> {
    dummy: PhantomData<(C, O)>,
    pub blender: B,
}

impl<C: Color, O: Order, B: Blender<C, O>> CopyOrBlendRgbaWrapper<C, O, B> {
    pub fn new(blender: B) -> Self {
        CopyOrBlendRgbaWrapper {
            dummy: PhantomData,
            blender,
        }
    }

    pub fn blender_mut(&mut self) -> &mut B {
        &mut self.blender
    }

    pub fn copy_or_blend_pix(&self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32) {
        if alpha != 0 {
            if alpha == C::BASE_MASK {
                p[O::R] = from_u32!(cr);
                p[O::G] = from_u32!(cg);
                p[O::B] = from_u32!(cb);
                p[O::A] = from_u32!(C::BASE_MASK);
            } else {
                self.blender.blend_pix(p, cr, cg, cb, alpha);
            }
        }
    }

    pub fn copy_or_blend_pix_with_cover(
        &self, p: &mut [C::ValueType], cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        if cover == 255 {
            self.copy_or_blend_pix(p, cr, cg, cb, alpha);
        } else {
            if alpha != 0 {
                let alpha = (alpha * (cover + 1)) >> 8;
                if alpha == C::BASE_MASK {
                    p[O::R] = from_u32!(cr);
                    p[O::G] = from_u32!(cg);
                    p[O::B] = from_u32!(cb);
                    p[O::A] = from_u32!(C::BASE_MASK);
                } else {
                    self.blender
                        .blend_pix_with_cover(p, cr, cg, cb, alpha, cover);
                }
            }
        }
    }
}

pub struct AlphaBlendRgba<
    'a,
    C: Color + RgbArgs,
    O: Order,
    Blend: Blender<C, O>,
    RenBuf: RenderBuffer<T = u8>,
    PixelSize: AggInteger = u32,
> {
    rbuf: Equiv<'a, RenBuf>,
    color: PhantomData<C>,
    order: PhantomData<O>,
    cobtype: CopyOrBlendRgbaWrapper<C, O, Blend>,
    size: PhantomData<PixelSize>,
}

impl<
        'a,
        C: Color + RgbArgs,
        O: Order,
        Blend: Blender<C, O>,
        RenBuf: RenderBuffer<T = u8>,
        PixelSize: AggInteger,
    > AlphaBlendRgba<'a, C, O, Blend, RenBuf, PixelSize>
{
    const PIXEL_WIDTH: usize = std::mem::size_of::<PixelSize>();

    pub fn new_borrowed(rb: &'a mut RenBuf) -> Self {
        Self {
            rbuf: Equiv::Brw(rb),
            cobtype: CopyOrBlendRgbaWrapper::new(Blend::new()),
            color: PhantomData,
            order: PhantomData,
            size: PhantomData,
        }
    }
    pub fn new_owned(rb: RenBuf) -> Self {
        Self {
            rbuf: Equiv::Own(rb),
            cobtype: CopyOrBlendRgbaWrapper::new(Blend::new()),
            color: PhantomData,
            order: PhantomData,
            size: PhantomData,
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
        self.cobtype.blender_mut()
    }

    pub fn for_each_pixel(&mut self, func: &dyn Fn(&mut [C::ValueType])) {
        for y in 0..self.height() as i32 {
            let r = self.rbuf.row_data(y);
            if !r.ptr.is_null() {
                let len = r.x2 - r.x1 + 1;
                let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), (r.x1 << 2), C::ValueType);
                let mut i = 0;
                for _ in 0..len {
                    func(&mut p[i..]);
                    i += 4;
                }
            }
        }
    }

    pub fn apply_gamma_dir<GammaLut: crate::Gamma<C::ValueType, C::ValueType>>(
        &mut self, g: &GammaLut,
    ) {
        let ag = ApplyGammaDirRgba::<C, O, GammaLut>::new(g);
        self.for_each_pixel(&|p| ag.apply(p));
    }

    pub fn apply_gamma_inv<GammaLut: crate::Gamma<C::ValueType, C::ValueType>>(
        &mut self, g: &GammaLut,
    ) {
        let ag = ApplyGammaInvRgba::<C, O, GammaLut>::new(g);
        self.for_each_pixel(&|p| ag.apply(p));
    }

    pub fn demultiply(&mut self) {
        self.for_each_pixel(&MultiplierRgba::<C, O>::demultiply);
    }

    pub fn premultiply(&mut self) {
        self.for_each_pixel(&MultiplierRgba::<C, O>::premultiply);
    }
}

impl<
        'a,
        C: Color + RgbArgs,
        O: Order,
        Blend: Blender<C, O>,
        RenBuf: RenderBuffer<T = u8>,
        PixelSize: AggInteger,
    > ImageSrc for AlphaBlendRgba<'a, C, O, Blend, RenBuf, PixelSize>
{
}

impl<
        'a,
        C: Color + RgbArgs,
        O: Order,
        Blend: Blender<C, O>,
        RenBuf: RenderBuffer<T = u8>,
        PixelSize: AggInteger,
    > PixFmt for AlphaBlendRgba<'a, C, O, Blend, RenBuf, PixelSize>
{
    type C = C;
    type O = O;
    type T = RenBuf::T;
    const PIXEL_WIDTH: u32 = std::mem::size_of::<C::ValueType>() as u32 * 4;

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

    fn row(&self, y: i32) -> &[Self::T] {
        self.rbuf.row(y)
    }

    fn row_mut(&mut self, y: i32) -> &mut [Self::T] {
        self.rbuf.row_mut(y)
    }

    fn row_data(&self, y: i32) -> RowData<Self::T> {
        self.rbuf.row_data(y)
    }

    fn make_pix(&self, p: &mut [u8], c: &C) {
        let p = p.as_mut_ptr() as *mut u8 as *mut C::ValueType;
        unsafe {
            *p.offset(O::R as isize) = c.r();
            *p.offset(O::G as isize) = c.g();
            *p.offset(O::B as isize) = c.b();
            *p.offset(O::A as isize) = c.a();
        }
    }

    fn pixel(&self, x: i32, y: i32) -> C {
        let p = self.rbuf.row(y);
        if p.is_empty() {
            C::no_color()
        } else {
            let p = slice_t_to_vt!(p, x << 2, C::ValueType);
            C::new_init(p[O::R], p[O::G], p[O::B], p[O::A])
        }
    }

    fn copy_pixel(&mut self, x: i32, y: i32, c: &C) {
        unsafe {
            let p =
                self.rbuf.row_mut(y).as_mut_ptr().offset((x << 2) as isize) as *mut C::ValueType;
            *p.offset(O::R as isize) = c.r();
            *p.offset(O::G as isize) = c.g();
            *p.offset(O::B as isize) = c.b();
            *p.offset(O::A as isize) = c.a();
        }
    }

    fn blend_pixel(&mut self, x: i32, y: i32, c: &C, cover: u8) {
        self.cobtype.copy_or_blend_pix_with_cover(
            slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType),
            c.r().into_u32(),
            c.g().into_u32(),
            c.b().into_u32(),
            c.a().into_u32(),
            cover as u32,
        )
    }

    fn copy_hline(&mut self, x: i32, y: i32, len: u32, c: &C) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);

        for i in 0..len as usize {
            p[(i * 4) + O::R] = c.r();
            p[(i * 4) + O::G] = c.g();
            p[(i * 4) + O::B] = c.b();
            p[(i * 4) + O::A] = c.a();
        }
    }

    fn copy_vline(&mut self, x: i32, y: i32, len: u32, c: &C) {
        for i in 0..len as i32 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), x << 2, C::ValueType);

            p[O::R] = c.r();
            p[O::G] = c.g();
            p[O::B] = c.b();
            p[O::A] = c.a();
        }
    }

    fn blend_hline(&mut self, x: i32, y: i32, len: u32, c: &C, cover: u8) {
        if c.a().into_u32() != 0 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);
            let alpha = (c.a().into_u32() * (cover as u32 + 1)) >> 8;
            if alpha == C::BASE_MASK {
                for i in 0..len as usize {
                    p[(i * 4) + O::R] = c.r();
                    p[(i * 4) + O::G] = c.g();
                    p[(i * 4) + O::B] = c.b();
                    p[(i * 4) + O::A] = c.a();
                }
            } else {
                if cover == 255 {
                    for i in 0..len as usize {
                        self.cobtype.blender_mut().blend_pix(
                            &mut p[(i * 4)..],
                            c.r().into_u32(),
                            c.g().into_u32(),
                            c.b().into_u32(),
                            alpha,
                        );
                    }
                } else {
                    for i in 0..len as usize {
                        self.cobtype.blender_mut().blend_pix_with_cover(
                            &mut p[(i * 4)..],
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
    }

    fn blend_vline(&mut self, x: i32, y: i32, len: u32, c: &C, cover: u8) {
        if c.a().into_u32() != 0 {
            let alpha = (c.a().into_u32() * (cover as u32 + 1)) >> 8;
            if alpha == C::BASE_MASK {
                for i in 0..len as usize {
                    let p =
                        slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
                    p[O::R] = c.r();
                    p[O::G] = c.g();
                    p[O::B] = c.b();
                    p[O::A] = c.a();
                }
            } else {
                if cover == 255 {
                    for i in 0..len as usize {
                        let p = slice_t_to_vt_mut!(
                            self.rbuf.row_mut(y + i as i32),
                            x << 2,
                            C::ValueType
                        );
                        self.cobtype.blender_mut().blend_pix(
                            p,
                            c.r().into_u32(),
                            c.g().into_u32(),
                            c.b().into_u32(),
                            alpha,
                        );
                    }
                } else {
                    for i in 0..len as usize {
                        let p = slice_t_to_vt_mut!(
                            self.rbuf.row_mut(y + i as i32),
                            x << 2,
                            C::ValueType
                        );
                        self.cobtype.blender_mut().blend_pix_with_cover(
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
    }

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: u32, c: &C, covers: &[u8]) {
        if c.a().into_u32() != 0 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);
            for i in 0..len as usize {
                let alpha = (c.a().into_u32() * (covers[i] as u32 + 1)) >> 8;
                if alpha == C::BASE_MASK {
                    p[(i * 4) + O::R] = c.r();
                    p[(i * 4) + O::G] = c.g();
                    p[(i * 4) + O::B] = c.b();
                    p[(i * 4) + O::A] = c.a();
                } else {
                    self.cobtype.blender_mut().blend_pix_with_cover(
                        &mut p[(i * 4)..],
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

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: u32, c: &C, covers: &[u8]) {
        if c.a().into_u32() != 0 {
            for i in 0..len as usize {
                let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
                let alpha = (c.a().into_u32() * (covers[i] as u32 + 1)) >> 8;
                if alpha == C::BASE_MASK {
                    p[O::R] = c.r();
                    p[O::G] = c.g();
                    p[O::B] = c.b();
                    p[O::A] = c.a();
                } else {
                    self.cobtype.blender_mut().blend_pix_with_cover(
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
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);
        for i in 0..len as usize {
            p[(i * 4) + O::R] = colors[i].r();
            p[(i * 4) + O::G] = colors[i].g();
            p[(i * 4) + O::B] = colors[i].b();
            p[(i * 4) + O::A] = colors[i].a();
        }
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]) {
        for i in 0..len as usize {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);

            p[O::R] = colors[i].r();
            p[O::G] = colors[i].g();
            p[O::B] = colors[i].b();
            p[O::A] = colors[i].a();
        }
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[C], covers: &[u8], cover: u8,
    ) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);

        if covers.len() > 0 {
            for i in 0..len as usize {
                self.cobtype.copy_or_blend_pix_with_cover(
                    &mut p[(i * 4)..],
                    colors[i].r().into_u32(),
                    colors[i].g().into_u32(),
                    colors[i].b().into_u32(),
                    colors[i].a().into_u32(),
                    covers[i] as u32,
                );
            }
        } else {
            if cover == 255 {
                for i in 0..len as usize {
                    self.cobtype.copy_or_blend_pix(
                        &mut p[(i * 4)..],
                        colors[i].r().into_u32(),
                        colors[i].g().into_u32(),
                        colors[i].b().into_u32(),
                        colors[i].a().into_u32(),
                    );
                }
            } else {
                for i in 0..len as usize {
                    self.cobtype.copy_or_blend_pix_with_cover(
                        &mut p[(i * 4)..],
                        colors[i].r().into_u32(),
                        colors[i].g().into_u32(),
                        colors[i].b().into_u32(),
                        colors[i].a().into_u32(),
                        cover as u32,
                    );
                }
            }
        }
    }

    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[C], covers: &[u8], cover: u8,
    ) {
        if covers.len() > 0 {
            for i in 0..len as usize {
                let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
                self.cobtype.copy_or_blend_pix_with_cover(
                    p,
                    colors[i].r().into_u32(),
                    colors[i].g().into_u32(),
                    colors[i].b().into_u32(),
                    colors[i].a().into_u32(),
                    covers[i] as u32,
                );
            }
        } else {
            if cover == 255 {
                for i in 0..len as usize {
                    let p =
                        slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
                    self.cobtype.copy_or_blend_pix(
                        p,
                        colors[i].r().into_u32(),
                        colors[i].g().into_u32(),
                        colors[i].b().into_u32(),
                        colors[i].a().into_u32(),
                    );
                }
            } else {
                for i in 0..len as usize {
                    let p =
                        slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
                    self.cobtype.copy_or_blend_pix_with_cover(
                        p,
                        colors[i].r().into_u32(),
                        colors[i].g().into_u32(),
                        colors[i].b().into_u32(),
                        colors[i].a().into_u32(),
                        cover as u32,
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
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst << 2, C::ValueType);
            let psrc = slice_t_to_vt!(psrc, xsrc << 2, <<R as PixFmt>::C as Args>::ValueType);
            let mut incp = 4;
            let (mut os, mut od) = (0, 0);
            if xdst > xsrc {
                os = (len as i32 - 1) << 2;
                od = (len as i32 - 1) << 2;
                incp = -4;
            }
            if cover == 255 {
                for _i in 0..len as usize {
                    self.cobtype.copy_or_blend_pix(
                        &mut pdst[od as usize..],
                        psrc[os as usize + R::O::R].into_u32(),
                        psrc[os as usize + R::O::G].into_u32(),
                        psrc[os as usize + R::O::B].into_u32(),
                        psrc[os as usize + R::O::A].into_u32(),
                    );
                    os += incp;
                    od += incp;
                }
            } else {
                for _i in 0..len as usize {
                    self.cobtype.copy_or_blend_pix_with_cover(
                        &mut pdst[od as usize..],
                        psrc[os as usize + R::O::R].into_u32(),
                        psrc[os as usize + R::O::G].into_u32(),
                        psrc[os as usize + R::O::B].into_u32(),
                        psrc[os as usize + R::O::A].into_u32(),
                        cover as u32,
                    );
                    os += incp;
                    od += incp;
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
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst << 2, C::ValueType);

            for i in 0..len as usize {
                let cover: u32 = psrc[i].into_u32() * cover + C::BASE_MASK >> C::BASE_SHIFT;
                self.cobtype.copy_or_blend_pix_with_cover(
                    &mut pdst[i * 4..],
                    color.r().into_u32(),
                    color.g().into_u32(),
                    color.b().into_u32(),
                    color.a().into_u32(),
                    cover as u32,
                );
            }
        }
    }

    fn blend_from_lut<R: PixFmt>(
        &mut self, from: &R, color_lut: &[C], xdst: i32, ydst: i32, _xsrc: i32, ysrc: i32,
        len: u32, cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let psrc = slice_t_to_vt!(psrc, 0,  <<R as PixFmt>::C as Args>::ValueType);
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst << 2, C::ValueType);

            if cover == 255 {
                for i in 0..len as usize {
                    let color = color_lut[psrc[i].into_u32() as usize];
                    self.cobtype.copy_or_blend_pix(
                        &mut pdst[i * 4..],
                        color.r().into_u32(),
                        color.g().into_u32(),
                        color.b().into_u32(),
                        color.a().into_u32(),
                    );
                }
            } else {
                for i in 0..len as usize {
                    let color = color_lut[psrc[i].into_u32() as usize];
                    self.cobtype.copy_or_blend_pix_with_cover(
                        &mut pdst[i * 4..],
                        color.r().into_u32(),
                        color.g().into_u32(),
                        color.b().into_u32(),
                        color.a().into_u32(),
                        cover as u32,
                    );
                }
            }
        }
    }
}

//================================================CustomBlendRgba
pub struct CustomBlendRgba<
    'a,
    C: Color + RgbArgs,
    O: Order,
    Blend: BlenderOp<C, O>,
    RenBuf: RenderBuffer<T = u8>,
> {
    rbuf: Equiv<'a, RenBuf>,
    color: PhantomData<C>,
    order: PhantomData<O>,
    blender: Blend,
    comp_op: u32,
}

impl<'a, C: Color + RgbArgs, O: Order, Blend: BlenderOp<C, O>, RenBuf: RenderBuffer<T = u8>>
    CustomBlendRgba<'a, C, O, Blend, RenBuf>
{
    const PIXEL_WIDTH: usize = std::mem::size_of::<C::ValueType>() * 3;

    pub fn new_owned(rb: RenBuf) -> Self {
        Self {
            rbuf: Equiv::Own(rb),
            blender: Blend::new(),
            color: PhantomData,
            order: PhantomData,
            comp_op: 3,
        }
    }

    pub fn new_borrowed(rb: &'a mut RenBuf) -> Self {
        Self {
            rbuf: Equiv::Brw(rb),
            blender: Blend::new(),
            color: PhantomData,
            order: PhantomData,
            comp_op: 3,
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

    pub fn set_comp_op(&mut self, op: u32) {
        self.comp_op = op;
    }

    pub fn comp_op(&self) -> u32 {
        self.comp_op
    }

    pub fn for_each_pixel(&mut self, func: &dyn Fn(&mut [C::ValueType])) {
        for y in 0..self.height() as i32 {
            let r = self.rbuf.row_data(y);
            if !r.ptr.is_null() {
                let len = r.x2 - r.x1 + 1;

                let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), r.x1 << 2, C::ValueType);

                let mut i = 0;
                for _ in 0..len {
                    func(&mut p[i..]);
                    i += 4;
                }
            }
        }
    }

    pub fn apply_gamma_dir<GammaLut: crate::Gamma<C::ValueType, C::ValueType>>(
        &mut self, g: GammaLut,
    ) {
        let ag = ApplyGammaDirRgba::<C, O, GammaLut>::new(&g);
        self.for_each_pixel(&|p| ag.apply(p));
    }

    pub fn apply_gamma_inv<GammaLut: crate::Gamma<C::ValueType, C::ValueType>>(
        &mut self, g: GammaLut,
    ) {
        let ag = ApplyGammaInvRgba::<C, O, GammaLut>::new(&g);
        self.for_each_pixel(&|p| ag.apply(p));
    }

    pub fn demultiply(&mut self) {
        self.for_each_pixel(&MultiplierRgba::<C, O>::demultiply);
    }

    pub fn premultiply(&mut self) {
        self.for_each_pixel(&MultiplierRgba::<C, O>::premultiply);
    }
}

impl<'a, C: Color + RgbArgs, O: Order, Blend: BlenderOp<C, O>, RenBuf: RenderBuffer<T = u8>>
    ImageSrc for CustomBlendRgba<'a, C, O, Blend, RenBuf>
{
}

impl<'a, C: Color + RgbArgs, O: Order, Blend: BlenderOp<C, O>, RenBuf: RenderBuffer<T = u8>> PixFmt
    for CustomBlendRgba<'a, C, O, Blend, RenBuf>
{
    type C = C;
    type O = O;
    type T = RenBuf::T;
    const PIXEL_WIDTH: u32 = std::mem::size_of::<C::ValueType>() as u32 * 4;

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

    fn row(&self, y: i32) -> &[Self::T] {
        self.rbuf.row(y)
    }

    fn row_mut(&mut self, y: i32) -> &mut [Self::T] {
        self.rbuf.row_mut(y)
    }

    fn row_data(&self, y: i32) -> RowData<Self::T> {
        self.rbuf.row_data(y)
    }

    fn make_pix(&self, p: &mut [u8], c: &C) {
        let p = p.as_mut_ptr() as *mut u8 as *mut C::ValueType;
        unsafe {
            *p.offset(O::R as isize) = c.r();
            *p.offset(O::G as isize) = c.g();
            *p.offset(O::B as isize) = c.b();
            *p.offset(O::A as isize) = c.a();
        }
    }

    fn pixel(&self, x: i32, y: i32) -> C {
        let p = self.rbuf.row(y);
        if p.is_empty() {
            C::no_color()
        } else {
            let p = slice_t_to_vt!(p, x << 2, C::ValueType);
            C::new_init(p[O::R], p[O::G], p[O::B], p[O::A])
        }
    }

    fn copy_pixel(&mut self, x: i32, y: i32, c: &C) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);
        self.blender.blend_pix_with_cover(
            self.comp_op,
            p,
            c.r().into_u32(),
            c.g().into_u32(),
            c.b().into_u32(),
            c.a().into_u32(),
            255,
        );
    }

    fn blend_pixel(&mut self, x: i32, y: i32, c: &C, cover: u8) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);
        self.blender.blend_pix_with_cover(
            self.comp_op,
            p,
            c.r().into_u32(),
            c.g().into_u32(),
            c.b().into_u32(),
            c.a().into_u32(),
            cover as u32,
        );
    }

    fn copy_hline(&mut self, x: i32, y: i32, len: u32, c: &C) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);

        for i in 0..len as usize {
            self.blender.blend_pix_with_cover(
                self.comp_op,
                &mut p[i * 4..],
                c.r().into_u32(),
                c.g().into_u32(),
                c.b().into_u32(),
                c.a().into_u32(),
                255,
            );
        }
    }

    fn copy_vline(&mut self, x: i32, y: i32, len: u32, c: &C) {
        for i in 0..len as usize {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
            self.blender.blend_pix_with_cover(
                self.comp_op,
                p,
                c.r().into_u32(),
                c.g().into_u32(),
                c.b().into_u32(),
                c.a().into_u32(),
                255,
            );
        }
    }

    fn blend_hline(&mut self, x: i32, y: i32, len: u32, c: &C, cover: u8) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);

        for i in 0..len as usize {
            self.blender.blend_pix_with_cover(
                self.comp_op,
                &mut p[i * 4..],
                c.r().into_u32(),
                c.g().into_u32(),
                c.b().into_u32(),
                c.a().into_u32(),
                cover as u32,
            );
        }
    }

    fn blend_vline(&mut self, x: i32, y: i32, len: u32, c: &C, cover: u8) {
        for i in 0..len as usize {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
            self.blender.blend_pix_with_cover(
                self.comp_op,
                p,
                c.r().into_u32(),
                c.g().into_u32(),
                c.b().into_u32(),
                c.a().into_u32(),
                cover as u32,
            );
        }
    }

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: u32, c: &C, covers: &[u8]) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);

        for i in 0..len as usize {
            self.blender.blend_pix_with_cover(
                self.comp_op,
                &mut p[i * 4..],
                c.r().into_u32(),
                c.g().into_u32(),
                c.b().into_u32(),
                c.a().into_u32(),
                covers[i] as u32,
            );
        }
    }

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: u32, c: &C, covers: &[u8]) {
        for i in 0..len as usize {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
            self.blender.blend_pix_with_cover(
                self.comp_op,
                p,
                c.r().into_u32(),
                c.g().into_u32(),
                c.b().into_u32(),
                c.a().into_u32(),
                covers[i] as u32,
            );
        }
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[C], covers: &[u8], cover: u8,
    ) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);

        for i in 0..len as usize {
            self.blender.blend_pix_with_cover(
                self.comp_op,
                &mut p[i * 4..],
                colors[i].r().into_u32(),
                colors[i].g().into_u32(),
                colors[i].b().into_u32(),
                colors[i].a().into_u32(),
                (if covers.len() == 0 { cover } else { covers[i] }) as u32,
            );
        }
    }

    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[C], covers: &[u8], cover: u8,
    ) {
        for i in 0..len as usize {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);
            self.blender.blend_pix_with_cover(
                self.comp_op,
                p,
                colors[i].r().into_u32(),
                colors[i].g().into_u32(),
                colors[i].b().into_u32(),
                colors[i].a().into_u32(),
                (if covers.len() == 0 { cover } else { covers[i] }) as u32,
            );
        }
    }

    fn copy_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[C]) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x << 2, C::ValueType);
        for i in 0..len as usize {
            p[(i * 4) + O::R] = colors[i].r();
            p[(i * 4) + O::G] = colors[i].g();
            p[(i * 4) + O::B] = colors[i].b();
            p[(i * 4) + O::A] = colors[i].a();
        }
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[C]) {
        for i in 0..len as usize {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i as i32), x << 2, C::ValueType);

            p[O::R] = colors[i].r();
            p[O::G] = colors[i].g();
            p[O::B] = colors[i].b();
            p[O::A] = colors[i].a();
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

    fn blend_from<R: PixFmt>(
        &mut self, from: &R, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32, cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst << 2, C::ValueType);
            let psrc = slice_t_to_vt!(psrc, xsrc << 2, <<R as PixFmt>::C as Args>::ValueType);
            let mut incp = 4;
            let (mut os, mut od) = (0, 0);
            if xdst > xsrc {
                os = (len as i32 - 1) << 2;
                od = (len as i32 - 1) << 2;
                incp = -4;
            }
            for _i in 0..len as usize {
                self.blender.blend_pix_with_cover(
                    self.comp_op,
                    &mut pdst[od as usize..],
                    psrc[os as usize + R::O::R].into_u32(),
                    psrc[os as usize + 1 + R::O::G].into_u32(),
                    psrc[os as usize + 2 + R::O::B].into_u32(),
                    psrc[os as usize + 3 + R::O::A].into_u32(),
                    cover as u32,
                );
                os += incp;
                od += incp;
            }
        }
    }

    fn blend_from_color<R: PixFmt>(
        &mut self, from: &R, c: &C, xdst: i32, ydst: i32, _xsrc: i32, ysrc: i32, len: u32,
        cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let psrc = slice_t_to_vt!(psrc, 0, <<R as PixFmt>::C as Args>::ValueType);
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst << 2, C::ValueType);

            for i in 0..len as usize {
                let cover: u32 = psrc[i].into_u32() * cover + C::BASE_MASK >> C::BASE_SHIFT;
                self.blender.blend_pix_with_cover(
                    self.comp_op,
                    &mut pdst[i * 4..],
                    c.r().into_u32(),
                    c.g().into_u32(),
                    c.b().into_u32(),
                    c.a().into_u32(),
                    cover,
                );
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
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst << 2, C::ValueType);

            for i in 0..len as usize {
                let color = color_lut[psrc[i].into_u32() as usize];
                self.blender.blend_pix_with_cover(
                    self.comp_op,
                    &mut pdst[i * 4..],
                    color.r().into_u32(),
                    color.g().into_u32(),
                    color.b().into_u32(),
                    color.a().into_u32(),
                    cover,
                );
            }
        }
    }
}
