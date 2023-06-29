use crate::basics::{RectI, RowData};
use crate::color_rgba::{OrderBgra, Rgba8, Rgba16};
use crate::gamma_lut::GammaLut;
use crate::{
    slice_t_to_vt, slice_t_to_vt_mut, AggPrimitive, Args, BlenderPacked, Color, Equiv, Gamma,
    ImageSrc, Order, PixFmt, RenderBuf, RenderBuffer, RgbArgs,
};

use wrapping_arithmetic::wrappit;

pub type PixRgb555<'a> = AlphaBlendRgbPacked<'a, BlenderRgb555, RenderBuf>;
pub type PixRgb565<'a> = AlphaBlendRgbPacked<'a, BlenderRgb565, RenderBuf>;
pub type PixRgb555Pre<'a> = AlphaBlendRgbPacked<'a, BlenderRgb555Pre, RenderBuf>;
pub type PixRgb565Pre<'a> = AlphaBlendRgbPacked<'a, BlenderRgb565Pre, RenderBuf>;

pub type PixRgbAAA<'a> = AlphaBlendRgbPacked<'a, BlenderRgbAAA, RenderBuf>;
pub type PixBgrAAA<'a> = AlphaBlendRgbPacked<'a, BlenderBgrAAA, RenderBuf>;
pub type PixRgbBBA<'a> = AlphaBlendRgbPacked<'a, BlenderRgbBBA, RenderBuf>;
pub type PixBgrABB<'a> = AlphaBlendRgbPacked<'a, BlenderBgrABB, RenderBuf>;

pub type PixRgbAAAPre<'a> = AlphaBlendRgbPacked<'a, BlenderRgbAAAPre, RenderBuf>;
pub type PixBgrAAAPre<'a> = AlphaBlendRgbPacked<'a, BlenderBgrAAAPre, RenderBuf>;
pub type PixRgbBBAPre<'a> = AlphaBlendRgbPacked<'a, BlenderRgbBBAPre, RenderBuf>;
pub type PixBgrABBPre<'a> = AlphaBlendRgbPacked<'a, BlenderBgrABBPre, RenderBuf>;

pub type PixRgb555Gamma<'a> =
    AlphaBlendRgbPacked<'a, BlenderRgb555Gamma<'a, GammaLut<u8, u8>>, RenderBuf>;
pub type PixRgb565Gamma<'a> =
    AlphaBlendRgbPacked<'a, BlenderRgb565Gamma<'a, GammaLut<u8, u8>>, RenderBuf>;
pub type PixRgbAAAGamma<'a> =
    AlphaBlendRgbPacked<'a, BlenderRgbAAAGamma<'a, GammaLut<u8, u8>>, RenderBuf>;
pub type PixBgrAAAGamma<'a> =
    AlphaBlendRgbPacked<'a, BlenderBgrAAAGamma<'a, GammaLut<u8, u8>>, RenderBuf>;
pub type PixRgbBBAGamma<'a> =
    AlphaBlendRgbPacked<'a, BlenderRgbBBAGamma<'a, GammaLut<u8, u8>>, RenderBuf>;
pub type PixBgrABBGamma<'a> =
    AlphaBlendRgbPacked<'a, BlenderBgrABBGamma<'a, GammaLut<u8, u8>>, RenderBuf>;

// NOT TESTED
//=========================================================BlenderRgb555
pub struct BlenderRgb555;

impl BlenderPacked for BlenderRgb555 {
    type ColorType = Rgba8;
    type ValueType = u8;
    //type calc_type = i32;
    type PixelType = u16;

    fn new() -> Self {
        BlenderRgb555
    }

    #[wrappit]
    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p as u32;
        let r = (rgb >> 7) & 0xF8;
        let g = (rgb >> 2) & 0xF8;
        let b = (rgb << 3) & 0xF8;

        *p = (((((cr - r) * alpha + (r << 8)) >> 1) & 0x7C00)
            | ((((cg - g) * alpha + (g << 8)) >> 6) & 0x03E0)
            | (((cb - b) * alpha + (b << 8)) >> 11)
            | 0x8000) as u16
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xF8) << 7) | ((g & 0xF8) << 2) | (b >> 3) | 0x8000) as u16
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba8::new_params(
            ((p >> 7) & 0xF8) as u32,
            ((p >> 2) & 0xF8) as u32,
            ((p << 3) & 0xF8) as u32,
            255,
        )
    }
}

//=====================================================BlenderRgb555Gamma
pub struct BlenderRgb555Gamma<'a, G: Gamma<u8, u8>> {
    gamma: Equiv<'a, G>,
}

impl<'a, G: Gamma<u8, u8>> BlenderRgb555Gamma<'a, G> {
    pub fn new_borrowed(gamma: &'a mut G) -> Self {
        BlenderRgb555Gamma {
            gamma: Equiv::Brw(gamma),
        }
    }

    pub fn new_owned(gamma: G) -> Self {
        BlenderRgb555Gamma {
            gamma: Equiv::Own(gamma),
        }
    }

    pub fn set_gamma_borrowed(&mut self, g: &'a mut G) {
        self.gamma = Equiv::Brw(g);
    }

    pub fn set_gamma_owned(&mut self, g: G) {
        self.gamma = Equiv::Own(g);
    }

    pub fn gamma_mut(&mut self) -> &mut G {
        &mut self.gamma
    }

    pub fn gamma(&self) -> &G {
        &self.gamma
    }
}

impl<'a, G: Gamma<u8, u8>> BlenderPacked for BlenderRgb555Gamma<'a, G> {
    type ColorType = Rgba8;
    type ValueType = u8;
    type PixelType = u16;

    fn new() -> Self {
        BlenderRgb555Gamma {
            gamma: Equiv::Own(G::new()),
        }
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p;
        let r = self.gamma.dir(((rgb >> 7) & 0xF8) as u8) as u32;
        let g = self.gamma.dir(((rgb >> 2) & 0xF8) as u8) as u32;
        let b = self.gamma.dir(((rgb << 3) & 0xF8) as u8) as u32;

        *p = (((self
            .gamma
            .inv((((self.gamma.dir(cr as u8) as u32 - r) * alpha + (r << 8)) >> 8) as u8)
            as u16)
            << 7)
            & 0x7C00)
            | (((self
                .gamma
                .inv((((self.gamma.dir(cg as u8) as u32 - g) * alpha + (g << 8)) >> 8) as u8)
                as u16)
                << 2)
                & 0x03E0)
            | ((self
                .gamma
                .inv((((self.gamma.dir(cb as u8) as u32 - b) * alpha + (b << 8)) >> 8) as u8)
                as u16)
                >> 3)
            | 0x8000
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xF8) << 7) | ((g & 0xF8) << 2) | (b >> 3) | 0x8000) as u16
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba8::new_params(
            ((p >> 7) & 0xF8) as u32,
            ((p >> 2) & 0xF8) as u32,
            ((p << 3) & 0xF8) as u32,
            255,
        )
    }
}

//=====================================================blender_rgb555_pre
pub struct BlenderRgb555Pre;

impl BlenderPacked for BlenderRgb555Pre {
    type ColorType = Rgba8;
    type ValueType = u8;
    type PixelType = u16;

    fn new() -> Self {
        BlenderRgb555Pre
    }

	#[wrappit]
    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = Self::ColorType::BASE_MASK - alpha;
        let rgb = *p;
        let r = ((rgb as u32) >> 7) & 0xF8;
        let g = ((rgb as u32) >> 2) & 0xF8;
        let b = ((rgb as u32) << 3) & 0xF8;

        *p = (((r * alpha + cr * cover) >> 1) & 0x7C00
            | ((g * alpha + cg * cover) >> 6) & 0x03E0
            | ((b * alpha + cb * cover) >> 11)
            | 0x8000) as u16;
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        self.blend_pix_with_cover(p, cr, cg, cb, alpha, 0)
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xF8) << 7) | ((g & 0xF8) << 2) | (b >> 3) | 0x8000) as u16
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        let p = p as u32;
        Rgba8::new_params((p >> 7) & 0xF8, (p >> 2) & 0xF8, (p << 3) & 0xF8, 255)
    }
}

//=========================================================blender_rgb565
pub struct BlenderRgb565;

impl BlenderPacked for BlenderRgb565 {
    type ColorType = Rgba8;
    type ValueType = u8;
    //type calc_type = i32;
    type PixelType = u16;

    fn new() -> Self {
        BlenderRgb565
    }

	#[wrappit]
    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p as u32;
        let r = (rgb >> 8) & 0xF8;
        let g = (rgb >> 3) & 0xFC;
        let b = (rgb << 3) & 0xF8;

        *p = ((((cr - r) * alpha + (r << 8)) & 0xF800)
            | ((((cg - g) * alpha + (g << 8)) >> 5) & 0x07E0)
            | (((cb - b) * alpha + (b << 8)) >> 11)) as u16;
    }
    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }
    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)) as u16
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba8::new_params(
            ((p >> 8) & 0xF8) as u32,
            ((p >> 3) & 0xFC) as u32,
            ((p << 3) & 0xF8) as u32,
            255,
        )
    }
}

pub struct BlenderRgb565Pre;

impl BlenderPacked for BlenderRgb565Pre {
    type ColorType = Rgba8;
    type ValueType = u8;
    type PixelType = u16;

    fn new() -> Self {
        BlenderRgb565Pre
    }

	#[wrappit]
    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = Self::ColorType::BASE_MASK - alpha;
        let rgb = *p;
        let r = ((rgb as u32) >> 8) & 0xF8;
        let g = ((rgb as u32) >> 3) & 0xFC;
        let b = ((rgb as u32) << 3) & 0xF8;

        *p = (((r * alpha + cr * cover) >> 1) & 0xF800
            | ((g * alpha + cg * cover) >> 5) & 0x07E0
            | ((b * alpha + cb * cover) >> 11)) as u16;
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        self.blend_pix_with_cover(p, cr, cg, cb, alpha, 0)
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)) as u16
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        let p = p as u32;
        Rgba8::new_params((p >> 8) & 0xF8, (p >> 3) & 0xFC, (p << 3) & 0xF8, 255)
    }
}

pub struct BlenderRgb565Gamma<'a, G: Gamma<u8, u8>> {
    gamma: Equiv<'a, G>,
}

impl<'a, G: Gamma<u8, u8>> BlenderRgb565Gamma<'a, G> {
    pub fn new_borrowed(gamma: &'a mut G) -> Self {
        BlenderRgb565Gamma {
            gamma: Equiv::Brw(gamma),
        }
    }

    pub fn new_owned(gamma: G) -> Self {
        BlenderRgb565Gamma {
            gamma: Equiv::Own(gamma),
        }
    }

    pub fn set_gamma_borrowed(&mut self, g: &'a mut G) {
        self.gamma = Equiv::Brw(g);
    }

    pub fn set_gamma_owned(&mut self, g: G) {
        self.gamma = Equiv::Own(g);
    }

    pub fn gamma_mut(&mut self) -> &mut G {
        &mut self.gamma
    }

    pub fn gamma(&self) -> &G {
        &self.gamma
    }
}

impl<'a, G: Gamma<u8, u8>> BlenderPacked for BlenderRgb565Gamma<'a, G> {
    type ColorType = Rgba8;
    type ValueType = u8;
    type PixelType = u16;

    fn new() -> Self {
        BlenderRgb565Gamma {
            gamma: Equiv::Own(G::new()),
        }
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p;
        let r = self.gamma.dir(((rgb >> 8) & 0xF8) as u8) as u32;
        let g = self.gamma.dir(((rgb >> 3) & 0xFC) as u8) as u32;
        let b = self.gamma.dir(((rgb << 3) & 0xF8) as u8) as u32;

        *p = (((self
            .gamma
            .inv((((self.gamma.dir(cr as u8) as u32 - r) * alpha + (r << 8)) >> 8) as u8)
            as u16)
            << 8)
            & 0xF800)
            | (((self
                .gamma
                .inv((((self.gamma.dir(cg as u8) as u32 - g) * alpha + (g << 8)) >> 8) as u8)
                as u16)
                << 3)
                & 0x07E0)
            | ((self
                .gamma
                .inv((((self.gamma.dir(cb as u8) as u32 - b) * alpha + (b << 8)) >> 8) as u8)
                as u16)
                >> 3);
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)) as u16
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba8::new_params(
            ((p >> 8) & 0xF8) as u32,
            ((p >> 3) & 0xFC) as u32,
            ((p << 3) & 0xF8) as u32,
            255,
        )
    }
}

pub struct BlenderRgbAAA;

impl BlenderPacked for BlenderRgbAAA {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderRgbAAA
    }

	#[wrappit]
    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p as u32;
        let r = (rgb >> 14) & 0xFFC0;
        let g = (rgb >> 4) & 0xFFC0;
        let b = (rgb << 6) & 0xFFC0;

        *p = (((((cr - r) * alpha + (r << 16)) >> 2) & 0x3FF00000)
            | ((((cg - g) * alpha + (g << 16)) >> 12) & 0x000FFC00)
            | (((cb - b) * alpha + (b << 16)) >> 22)
            | 0xC0000000) as u32;
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xFFC0) << 14) | ((g & 0xFFC0) << 4) | (b >> 6) | 0xC0000000) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p >> 14) & 0xFFC0) as u32,
            ((p >> 4) & 0xFFC0) as u32,
            ((p << 6) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderRgbAAAPre;

impl BlenderPacked for BlenderRgbAAAPre {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderRgbAAAPre
    }

	#[wrappit]
    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = Self::ColorType::BASE_MASK - alpha;
        let cover = (cover + 1) << (Self::ColorType::BASE_SHIFT - 8);
        let rgb = *p;
        let r = (rgb >> 14) & 0xFFC0;
        let g = (rgb >> 4) & 0xFFC0;
        let b = (rgb << 6) & 0xFFC0;

        *p = (((r * alpha + cr * cover) >> 2) & 0x3FF00000
            | ((g * alpha + cg * cover) >> 12) & 0x000FFC00
            | ((b * alpha + cb * cover) >> 22)
            | 0xC0000000) as u32;
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        self.blend_pix_with_cover(p, cr, cg, cb, alpha, 0)
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xFFC0) << 14) | ((g & 0xFFC0) << 4) | (b >> 6) | 0xC0000000) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p >> 14) & 0xFFC0) as u32,
            ((p >> 4) & 0xFFC0) as u32,
            ((p << 6) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderRgbAAAGamma<'a, G: Gamma<u8, u8>> {
    gamma: Equiv<'a, G>,
}

impl<'a, G: Gamma<u8, u8>> BlenderRgbAAAGamma<'a, G> {
    pub fn new_borrowed(gamma: &'a mut G) -> Self {
        BlenderRgbAAAGamma {
            gamma: Equiv::Brw(gamma),
        }
    }

    pub fn new_owned(gamma: G) -> Self {
        BlenderRgbAAAGamma {
            gamma: Equiv::Own(gamma),
        }
    }

    pub fn set_gamma_borrowed(&mut self, g: &'a mut G) {
        self.gamma = Equiv::Brw(g);
    }

    pub fn set_gamma_owned(&mut self, g: G) {
        self.gamma = Equiv::Own(g);
    }

    pub fn gamma_mut(&mut self) -> &mut G {
        &mut self.gamma
    }

    pub fn gamma(&self) -> &G {
        &self.gamma
    }
}

impl<'a, G: Gamma<u8, u8>> BlenderPacked for BlenderRgbAAAGamma<'a, G> {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderRgbAAAGamma {
            gamma: Equiv::Own(G::new()),
        }
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p;
        let r = self.gamma.dir(((rgb >> 14) & 0xFFC0) as u8) as u32;
        let g = self.gamma.dir(((rgb >> 4) & 0xFFC0) as u8) as u32;
        let b = self.gamma.dir(((rgb << 6) & 0xFFC0) as u8) as u32;

        *p = (((self
            .gamma
            .inv((((self.gamma.dir(cr as u8) as u32 - r) * alpha + (r << 16)) >> 16) as u8)
            as u32)
            << 14)
            & 0x3FF00000)
            | (((self
                .gamma
                .inv((((self.gamma.dir(cg as u8) as u32 - g) * alpha + (g << 16)) >> 16) as u8)
                as u32)
                << 4)
                & 0x000FFC00)
            | ((self
                .gamma
                .inv((((self.gamma.dir(cb as u8) as u32 - b) * alpha + (b << 16)) >> 16) as u8)
                as u32)
                >> 6)
            | 0xC0000000;
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xFFC0) << 14) | ((g & 0xFFC0) << 4) | (b >> 6) | 0xC0000000) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p >> 14) & 0xFFC0) as u32,
            ((p >> 4) & 0xFFC0) as u32,
            ((p << 6) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderBgrAAA;

impl BlenderPacked for BlenderBgrAAA {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderBgrAAA
    }

	#[wrappit]
    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p as u32;
        let b = (rgb >> 14) & 0xFFC0;
        let g = (rgb >> 4) & 0xFFC0;
        let r = (rgb << 6) & 0xFFC0;

        *p = (((((cb - b) * alpha + (b << 16)) >> 2) & 0x3FF00000)
            | ((((cg - g) * alpha + (g << 16)) >> 12) & 0x000FFC00)
            | (((cr - r) * alpha + (r << 16)) >> 22)
            | 0xC0000000) as u32;
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((b & 0xFFC0) << 14) | ((g & 0xFFC0) << 4) | (r >> 6) | 0xC0000000) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p << 6) & 0xFFC0) as u32,
            ((p >> 4) & 0xFFC0) as u32,
            ((p >> 14) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderBgrAAAPre;

impl BlenderPacked for BlenderBgrAAAPre {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderBgrAAAPre
    }

	#[wrappit]
    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = Self::ColorType::BASE_MASK - alpha;
        let cover = (cover + 1) << (Self::ColorType::BASE_SHIFT - 8);
        let rgb = *p;
        let b = (rgb >> 14) & 0xFFC0;
        let g = (rgb >> 4) & 0xFFC0;
        let r = (rgb << 6) & 0xFFC0;

        *p = (((b * alpha + cr * cover) >> 2) & 0x3FF00000
            | ((g * alpha + cg * cover) >> 12) & 0x000FFC00
            | ((r * alpha + cb * cover) >> 22)
            | 0xC0000000) as u32;
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        self.blend_pix_with_cover(p, cr, cg, cb, alpha, 0)
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((b & 0xFFC0) << 14) | ((g & 0xFFC0) << 4) | (r >> 6) | 0xC0000000) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p << 6) & 0xFFC0) as u32,
            ((p >> 4) & 0xFFC0) as u32,
            ((p >> 14) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderBgrAAAGamma<'a, G: Gamma<u8, u8>> {
    gamma: Equiv<'a, G>,
}

impl<'a, G: Gamma<u8, u8>> BlenderBgrAAAGamma<'a, G> {
    pub fn new_borrowed(gamma: &'a mut G) -> Self {
        BlenderBgrAAAGamma {
            gamma: Equiv::Brw(gamma),
        }
    }

    pub fn new_owned(gamma: G) -> Self {
        BlenderBgrAAAGamma {
            gamma: Equiv::Own(gamma),
        }
    }

    pub fn set_gamma_borrowed(&mut self, g: &'a mut G) {
        self.gamma = Equiv::Brw(g);
    }

    pub fn set_gamma_owned(&mut self, g: G) {
        self.gamma = Equiv::Own(g);
    }

    pub fn gamma_mut(&mut self) -> &mut G {
        &mut self.gamma
    }

    pub fn gamma(&self) -> &G {
        &self.gamma
    }
}

impl<'a, G: Gamma<u8, u8>> BlenderPacked for BlenderBgrAAAGamma<'a, G> {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderBgrAAAGamma {
            gamma: Equiv::Own(G::new()),
        }
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let bgr = *p;
        let b = self.gamma.dir(((bgr >> 14) & 0xFFC0) as u8) as u32;
        let g = self.gamma.dir(((bgr >> 4) & 0xFFC0) as u8) as u32;
        let r = self.gamma.dir(((bgr << 6) & 0xFFC0) as u8) as u32;

        *p = (((self
            .gamma
            .inv((((self.gamma.dir(cb as u8) as u32 - r) * alpha + (b << 16)) >> 16) as u8)
            as u32)
            << 14)
            & 0x3FF00000)
            | (((self
                .gamma
                .inv((((self.gamma.dir(cg as u8) as u32 - g) * alpha + (g << 16)) >> 16) as u8)
                as u32)
                << 4)
                & 0x000FFC00)
            | ((self
                .gamma
                .inv((((self.gamma.dir(cr as u8) as u32 - b) * alpha + (r << 16)) >> 16) as u8)
                as u32)
                >> 6)
            | 0xC0000000;
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((b & 0xFFC0) << 14) | ((g & 0xFFC0) << 4) | (r >> 6) | 0xC0000000) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p << 6) & 0xFFC0) as u32,
            ((p >> 4) & 0xFFC0) as u32,
            ((p >> 14) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderRgbBBA;

impl BlenderPacked for BlenderRgbBBA {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderRgbBBA
    }

	#[wrappit]
    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p as u32;
        let r = (rgb >> 16) & 0xFFE0;
        let g = (rgb >> 5) & 0xFFE0;
        let b = (rgb << 6) & 0xFFC0;

        *p = ((((cr - r) * alpha + (r << 16)) & 0xFFE00000)
            | ((((cg - g) * alpha + (g << 16)) >> 11) & 0x001FFC00)
            | (((cb - b) * alpha + (b << 16)) >> 22)) as u32;
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xFFE0) << 16) | ((g & 0xFFE0) << 5) | (b >> 6)) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p >> 16) & 0xFFE0) as u32,
            ((p >> 5) & 0xFFE0) as u32,
            ((p << 6) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderRgbBBAPre;

impl BlenderPacked for BlenderRgbBBAPre {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderRgbBBAPre
    }

	#[wrappit]
    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = Self::ColorType::BASE_MASK - alpha;
        let cover = (cover + 1) << (Self::ColorType::BASE_SHIFT - 8);
        let rgb = *p;
        let r = (rgb >> 16) & 0xFFE0;
        let g = (rgb >> 5) & 0xFFE0;
        let b = (rgb << 6) & 0xFFC0;

        *p = ((r * alpha + cr * cover) & 0xFFE00000
            | ((g * alpha + cg * cover) >> 11) & 0x001FFC00
            | ((b * alpha + cb * cover) >> 22)) as u32;
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        self.blend_pix_with_cover(p, cr, cg, cb, alpha, 0)
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xFFE0) << 16) | ((g & 0xFFE0) << 5) | (b >> 6)) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p >> 16) & 0xFFE0) as u32,
            ((p >> 5) & 0xFFE0) as u32,
            ((p << 6) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderRgbBBAGamma<'a, G: Gamma<u8, u8>> {
    gamma: Equiv<'a, G>,
}

impl<'a, G: Gamma<u8, u8>> BlenderRgbBBAGamma<'a, G> {
	pub fn new_borrowed(gamma: &'a mut G) -> Self {
        BlenderRgbBBAGamma {
            gamma: Equiv::Brw(gamma),
        }
    }

    pub fn new_owned(gamma: G) -> Self {
        BlenderRgbBBAGamma {
            gamma: Equiv::Own(gamma),
        }
    }

    pub fn set_gamma_borrowed(&mut self, g: &'a mut G) {
        self.gamma = Equiv::Brw(g);
    }

    pub fn set_gamma_owned(&mut self, g: G) {
        self.gamma = Equiv::Own(g);
    }

    pub fn gamma_mut(&mut self) -> &mut G {
        &mut self.gamma
    }

    pub fn gamma(&self) -> &G {
        &self.gamma
    }
}

impl<'a, G: Gamma<u8, u8>> BlenderPacked for BlenderRgbBBAGamma<'a, G> {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderRgbBBAGamma {
            gamma: Equiv::Own(G::new()),
        }
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let rgb = *p;
        let r = self.gamma.dir(((rgb >> 16) & 0xFFE0) as u8) as u32;
        let g = self.gamma.dir(((rgb >> 5) & 0xFFE0) as u8) as u32;
        let b = self.gamma.dir(((rgb << 6) & 0xFFC0) as u8) as u32;

        *p = (((self
            .gamma
            .inv((((self.gamma.dir(cr as u8) as u32 - r) * alpha + (r << 16)) >> 16) as u8)
            as u32)
            << 16)
            & 0xFFE00000)
            | (((self
                .gamma
                .inv((((self.gamma.dir(cg as u8) as u32 - g) * alpha + (g << 16)) >> 16) as u8)
                as u32)
                << 5)
                & 0x001FFC00)
            | ((self
                .gamma
                .inv((((self.gamma.dir(cb as u8) as u32 - b) * alpha + (b << 16)) >> 16) as u8)
                as u32)
                >> 6);
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((r & 0xFFE0) << 14) | ((g & 0xFFE0) << 5) | (b >> 6)) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p >> 16) & 0xFFE0) as u32,
            ((p >> 5) & 0xFFE0) as u32,
            ((p << 6) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderBgrABB;

impl BlenderPacked for BlenderBgrABB {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderBgrABB
    }

	#[wrappit]
    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let bgr = *p as u32;
        let b = (bgr >> 16) & 0xFFC0;
        let g = (bgr >> 6) & 0xFFE0;
        let r = (bgr << 5) & 0xFFE0;

        *p = ((((cb - b) * alpha + (b << 16)) & 0xFFC00000)
            | ((((cg - g) * alpha + (g << 16)) >> 10) & 0x003FF800)
            | (((cr - r) * alpha + (r << 16)) >> 21)) as u32;
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((b & 0xFFC0) << 16) | ((g & 0xFFE0) << 6) | (r >> 5)) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p << 5) & 0xFFE0) as u32,
            ((p >> 6) & 0xFFE0) as u32,
            ((p >> 16) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderBgrABBPre;

impl BlenderPacked for BlenderBgrABBPre {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderBgrABBPre
    }

	#[wrappit]
    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, cover: u32,
    ) {
        let alpha = Self::ColorType::BASE_MASK - alpha;
        let cover = (cover + 1) << (Self::ColorType::BASE_SHIFT - 8);
        let bgr = *p;
        let b = (bgr >> 16) & 0xFFC0;
        let g = (bgr >> 6) & 0xFFE0;
        let r = (bgr << 5) & 0xFFE0;

        *p = ((b * alpha + cb * cover) & 0xFFC00000
            | ((g * alpha + cg * cover) >> 10) & 0x003FF800
            | ((r * alpha + cr * cover) >> 21)) as u32;
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        self.blend_pix_with_cover(p, cr, cg, cb, alpha, 0)
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((b & 0xFFC0) << 16) | ((g & 0xFFE0) << 6) | (r >> 5)) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p << 5) & 0xFFE0) as u32,
            ((p >> 6) & 0xFFE0) as u32,
            ((p >> 16) & 0xFFC0) as u32,
            255,
        )
    }
}

pub struct BlenderBgrABBGamma<'a, G: Gamma<u8, u8>> {
    gamma: Equiv<'a, G>,
}

impl<'a, G: Gamma<u8, u8>> BlenderBgrABBGamma<'a, G> {
	pub fn new_borrowed(gamma: &'a mut G) -> Self {
        BlenderBgrABBGamma {
            gamma: Equiv::Brw(gamma),
        }
    }

    pub fn new_owned(gamma: G) -> Self {
        BlenderBgrABBGamma {
            gamma: Equiv::Own(gamma),
        }
    }

    pub fn set_gamma_borrowed(&mut self, g: &'a mut G) {
        self.gamma = Equiv::Brw(g);
    }

    pub fn set_gamma_owned(&mut self, g: G) {
        self.gamma = Equiv::Own(g);
    }

    pub fn gamma_mut(&mut self) -> &mut G {
        &mut self.gamma
    }

    pub fn gamma(&self) -> &G {
        &self.gamma
    }
}

impl<'a, G: Gamma<u8, u8>> BlenderPacked for BlenderBgrABBGamma<'a, G> {
    type ColorType = Rgba16;
    type ValueType = u16;
    type PixelType = u32;

    fn new() -> Self {
        BlenderBgrABBGamma {
            gamma: Equiv::Own(G::new()),
        }
    }

    fn blend_pix(&self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32) {
        let bgr = *p;
        let b = self.gamma.dir(((bgr >> 16) & 0xFFC0) as u8) as u32;
        let g = self.gamma.dir(((bgr >> 6) & 0xFFE0) as u8) as u32;
        let r = self.gamma.dir(((bgr << 5) & 0xFFE0) as u8) as u32;

        *p = (((self
            .gamma
            .inv((((self.gamma.dir(cb as u8) as u32 - b) * alpha + (b << 16)) >> 16) as u8)
            as u32)
            << 16)
            & 0xFFC00000)
            | (((self
                .gamma
                .inv((((self.gamma.dir(cg as u8) as u32 - g) * alpha + (g << 16)) >> 16) as u8)
                as u32)
                << 6)
                & 0x003FF800)
            | ((self
                .gamma
                .inv((((self.gamma.dir(cr as u8) as u32 - r) * alpha + (r << 16)) >> 16) as u8)
                as u32)
                >> 5);
    }

    fn blend_pix_with_cover(
        &self, p: &mut Self::PixelType, cr: u32, cg: u32, cb: u32, alpha: u32, _cover: u32,
    ) {
        self.blend_pix(p, cr, cg, cb, alpha);
    }

    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType {
        (((b & 0xFFC0) << 16) | ((g & 0xFFE0) << 6) | (r >> 5)) as u32
    }

    fn make_color(&self, p: Self::PixelType) -> Self::ColorType {
        Rgba16::new_params(
            ((p << 5) & 0xFFE0) as u32,
            ((p >> 6) & 0xFFE0) as u32,
            ((p >> 16) & 0xFFC0) as u32,
            255,
        )
    }
}

//===========================================AlphaBlendRgbPacked
pub struct AlphaBlendRgbPacked<'a, B: BlenderPacked, R: RenderBuffer<T = u8>> {
    pub rbuf: Equiv<'a, R>,
    pub blender: B,
}

impl<'a, B: BlenderPacked, R: RenderBuffer<T = u8>> AlphaBlendRgbPacked<'a, B, R> {
    pub const BASE_SHIFT: u32 = B::ColorType::BASE_SHIFT;
    pub const BASE_SCALE: u32 = B::ColorType::BASE_SCALE;
    pub const BASE_MASK: u32 = B::ColorType::BASE_MASK;

    pub fn new_borrowed(rb: &'a mut R) -> Self {
        AlphaBlendRgbPacked {
            rbuf: Equiv::Brw(rb),
            blender: B::new(),
        }
    }

    pub fn new_owned(rb: R) -> Self {
        AlphaBlendRgbPacked {
            rbuf: Equiv::Own(rb),
            blender: B::new(),
        }
    }

    pub fn attach_borrowed(&mut self, rb: &'a mut R) {
        self.rbuf = Equiv::Brw(rb);
    }

    pub fn attach_owned(&mut self, rb: R) {
        self.rbuf = Equiv::Own(rb);
    }

    pub fn rbuf_mut(&mut self) -> &mut R {
        &mut self.rbuf
    }

    pub fn blender_mut(&mut self) -> &mut B {
        &mut self.blender
    }

    pub fn copy_or_blend_pix(p: &mut B::PixelType, c: &B::ColorType, cover: u32, blender: &B) {
        if c.a().into_u32() != 0 {
            let alpha = (c.a().into_u32() * (cover + 1) as u32) >> 8;
            if alpha == B::ColorType::BASE_MASK {
                *p = blender.make_pix(c.r().into_u32(), c.g().into_u32(), c.b().into_u32());
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
}

impl<'a, B: BlenderPacked, R: RenderBuffer<T = u8>> ImageSrc for AlphaBlendRgbPacked<'a, B, R> {}
impl<'a, B: BlenderPacked, R: RenderBuffer<T = u8>> PixFmt for AlphaBlendRgbPacked<'a, B, R> {
    type C = B::ColorType;
    type O = OrderBgra;
    type T = R::T;
    const PIXEL_WIDTH: u32 = std::mem::size_of::<B::PixelType>() as u32;

    fn width(&self) -> u32 {
        self.rbuf.width()
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

    fn height(&self) -> u32 {
        self.rbuf.height()
    }

    fn stride(&self) -> i32 {
        self.rbuf.stride()
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

    fn make_pix(&self, p: &mut [u8], c: &B::ColorType) {
        let p = p.as_mut_ptr() as *mut B::PixelType;
        unsafe {
            *p = self
                .blender
                .make_pix(c.r().into_u32(), c.g().into_u32(), c.b().into_u32());
        }
    }

    fn pixel(&self, x: i32, y: i32) -> B::ColorType {
        let p = slice_t_to_vt!(self.rbuf.row(y), 0, B::PixelType);

        self.blender.make_color(p[x as usize])
    }

    fn copy_pixel(&mut self, x: i32, y: i32, c: &B::ColorType) {
        //let mut p = self.rbuf.row_ptr(x, y, 0) as *mut B::PixelType;
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), 0, B::PixelType);
        p[x as usize] = self
            .blender
            .make_pix(c.r().into_u32(), c.g().into_u32(), c.b().into_u32());
    }

    fn blend_pixel(&mut self, x: i32, y: i32, c: &B::ColorType, cover: u8) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), 0, B::PixelType);

        Self::copy_or_blend_pix(&mut p[x as usize], c, cover as u32, &self.blender);
    }

    fn copy_hline(&mut self, x: i32, y: i32, len: u32, c: &B::ColorType) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), 0, B::PixelType);

        let v = self
            .blender
            .make_pix(c.r().into_u32(), c.g().into_u32(), c.b().into_u32());
        for i in 0..len as usize {
            p[x as usize + i] = v;
        }
    }

    fn copy_vline(&mut self, x: i32, y: i32, len: u32, c: &B::ColorType) {
        let v = self
            .blender
            .make_pix(c.r().into_u32(), c.g().into_u32(), c.b().into_u32());
        for i in 0..len as i32 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), 0, B::PixelType);
            p[x as usize] = v;
        }
    }

    fn blend_hline(&mut self, x: i32, y: i32, len: u32, c: &B::ColorType, cover: u8) {
        if c.a().into_u32() != 0 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), 0, B::PixelType);
            let alpha = (c.a().into_u32() as u32 * (cover as u32 + 1)) >> 8;
            if alpha == Self::BASE_MASK {
                let v = self
                    .blender
                    .make_pix(c.r().into_u32(), c.g().into_u32(), c.b().into_u32());
                for i in 0..len as usize {
                    p[x as usize + i] = v;
                }
            } else {
                for i in 0..len as usize {
                    self.blender.blend_pix_with_cover(
                        &mut p[x as usize + i],
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

    fn blend_vline(&mut self, x: i32, y: i32, len: u32, c: &B::ColorType, cover: u8) {
        if c.a().into_u32() != 0 {
            let alpha = (c.a().into_u32() as u32 * (cover as u32 + 1)) >> 8;
            if alpha == Self::BASE_MASK {
                let v = self
                    .blender
                    .make_pix(c.r().into_u32(), c.g().into_u32(), c.b().into_u32());
                for i in 0..len as i32 {
                    let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), 0, B::PixelType);
                    p[x as usize] = v;
                }
            } else {
                for i in 0..len as i32 {
                    let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), 0, B::PixelType);
                    self.blender.blend_pix_with_cover(
                        &mut p[x as usize],
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

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: u32, c: &B::ColorType, covers: &[u8]) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), 0, B::PixelType);
        for i in 0..len as usize {
            Self::copy_or_blend_pix(&mut p[x as usize + i], c, covers[i] as u32, &self.blender);
        }
    }

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: u32, c: &B::ColorType, covers: &[u8]) {
        for i in 0..len as i32 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), 0, B::PixelType);
            Self::copy_or_blend_pix(
                &mut p[x as usize],
                c,
                covers[i as usize] as u32,
                &self.blender,
            );
        }
    }

    fn copy_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[B::ColorType]) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), 0, B::PixelType);
        for i in 0..len as usize {
            p[x as usize + i] = self.blender.make_pix(
                colors[i].r().into_u32(),
                colors[i].g().into_u32(),
                colors[i].b().into_u32(),
            );
        }
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[B::ColorType]) {
        for i in 0..len as i32 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), 0, B::PixelType);

            p[x as usize] = self.blender.make_pix(
                colors[i as usize].r().into_u32(),
                colors[i as usize].g().into_u32(),
                colors[i as usize].b().into_u32(),
            );
        }
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[B::ColorType], covers: &[u8], cover: u8,
    ) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), 0, B::PixelType);

        for i in 0..len as usize {
            Self::copy_or_blend_pix(
                &mut p[x as usize + i],
                &colors[i as usize],
                if covers.len() > 0 { covers[i] } else { cover } as u32,
                &self.blender,
            );
        }
    }

    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[B::ColorType], covers: &[u8], cover: u8,
    ) {
        for i in 0..len as i32 {
            let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y + i), 0, B::PixelType);
            Self::copy_or_blend_pix(
                &mut p[x as usize],
                &colors[i as usize],
                (if covers.len() > 0 {
                    covers[i as usize]
                } else {
                    cover
                }) as u32,
                &self.blender,
            );
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
                std::ptr::copy(src, dst, (len * Self::PIXEL_WIDTH) as usize);
            }
        }
    }

    fn blend_from<P: PixFmt<T = Self::T>>(
        &mut self, from: &P, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32, cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst, B::PixelType);
            let psrc = slice_t_to_vt!(psrc, xsrc * 4, <<P as PixFmt>::C as Args>::ValueType);

            for i in 0..len as usize {
                let alpha = psrc[(i * 4) + P::O::A].into_u32();
                if alpha > 0 {
                    if alpha == Self::BASE_MASK && cover == 255 {
                        pdst[i] = self.blender.make_pix(
                            psrc[(i * 4) + P::O::R].into_u32(),
                            psrc[(i * 4) + P::O::G].into_u32(),
                            psrc[(i * 4) + P::O::B].into_u32(),
                        );
                    } else {
                        self.blender.blend_pix_with_cover(
                            &mut pdst[i],
                            psrc[(i * 4) + P::O::R].into_u32(),
                            psrc[(i * 4) + P::O::G].into_u32(),
                            psrc[(i * 4) + P::O::B].into_u32(),
                            alpha,
                            cover,
                        );
                    }
                }
            }
        }
    }

    fn blend_from_color<Ren: PixFmt>(
        &mut self, from: &Ren, c: &Self::C, xdst: i32, ydst: i32, _xsrc: i32, ysrc: i32, len: u32,
        cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let _psrc = slice_t_to_vt!(psrc, 0, <<Ren as PixFmt>::C as Args>::ValueType);
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst, B::PixelType);

            for i in 0..len as usize {
                self.blender.blend_pix_with_cover(
                    &mut pdst[i],
                    c.r().into_u32(),
                    c.g().into_u32(),
                    c.b().into_u32(),
                    c.a().into_u32(),
                    cover,
                );
            }
        }
    }

    fn blend_from_lut<Ren: PixFmt>(
        &mut self, from: &Ren, color_lut: &[Self::C], xdst: i32, ydst: i32, _xsrc: i32, ysrc: i32,
        len: u32, cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let psrc = slice_t_to_vt!(psrc, 0, <<Ren as PixFmt>::C as Args>::ValueType);
            let pdst = slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst, B::PixelType);
            for i in 0..len as usize {
                let c = color_lut[psrc[i].into_u32() as usize];
                self.blender.blend_pix_with_cover(
                    &mut pdst[i],
                    c.r().into_u32(),
                    c.g().into_u32(),
                    c.b().into_u32(),
                    c.a().into_u32(),
                    cover,
                );
            }
        }
    }
}
