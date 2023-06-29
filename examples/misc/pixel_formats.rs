#![allow(dead_code)]

//use crate::platform::*;
use crate::PixFormat;
//use agg::BlenderRgbGamma;
use cfg_block::cfg_block;
cfg_block! {
#[cfg(feature = "agg_gray8")] {
    use agg::pixfmt_gray::{PixGray8, PixGray8Pre, BlenderGray8, BlenderGray8Pre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Gray8;
    pub type Pixfmt<'a> = PixGray8<'a>;
     pub type PixfmtPre<'a> = PixGray8Pre<'a>;
     pub type Blender = BlenderGray8;
     pub type BlenderPre = BlenderGray8Pre;
     pub type ColorType = agg::Gray8;
}

#[cfg(feature = "agg_gray16")] {
    use agg::pixfmt_gray::{PixGray16, PixGray16Pre, BlenderGray16, BlenderGray16Pre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Gray16;
    pub type Pixfmt<'a> = PixGray16<'a>;
     pub type PixfmtPre<'a> = PixGray16Pre<'a>;
     pub type Blender = BlenderGray16;
     pub type BlenderPre = BlenderGray16Pre;
     pub type ColorType = agg::Gray16;
}

#[cfg(feature = "agg_bgr24")] {
    use agg::pixfmt_rgb::{PixBgr24, PixBgr24Pre, BlenderRgb, BlenderRgbPre, PixBgr24Gamma, BlenderRgbGamma};
    pub const PIXEL_FORMAT: PixFormat = PixFormat::Bgr24;
    pub type Pixfmt<'a> = PixBgr24<'a>;
    pub type PixfmtPre<'a> = PixBgr24Pre<'a>;
    pub type PixfmtGamma<'a> = PixBgr24Gamma<'a>;
    pub type ColorType = agg::Rgba8;
    pub type OrderType = agg::color_rgba::OrderBgr;
    pub type Blender = BlenderRgb<ColorType, OrderType>;
    pub type BlenderPre = BlenderRgbPre<ColorType, OrderType>;
    pub type BlenderGamma<'a> = BlenderRgbGamma<'a, ColorType, OrderType, agg::GammaLut>;
}

#[cfg(feature = "agg_rgb24")] {
    use agg::pixfmt_rgb::{PixRgb24, PixRgb24Pre, BlenderRgb, BlenderRgbPre, PixRgb24Gamma, BlenderRgbGamma};
    pub const PIXEL_FORMAT: PixFormat = PixFormat::Rgb24;
    pub type Pixfmt<'a> = PixRgb24<'a>;
    pub type PixfmtPre<'a> = PixRgb24Pre<'a>;
	pub type PixfmtGamma<'a> = PixRgb24Gamma<'a>;
    pub type Blender = BlenderRgb<ColorType, OrderType>;
    pub type BlenderPre = BlenderRgbPre<ColorType, OrderType>;
	pub type BlenderGamma<'a> = BlenderRgbGamma<'a, ColorType, OrderType, agg::GammaLut>;
    pub type ColorType = agg::Rgba8;
    pub type OrderType = agg::color_rgba::OrderRgb;
}

#[cfg(feature = "agg_bgra32")] {
    use agg::pixfmt_rgba::{PixBgra32, PixBgra32Pre, BlenderRgba, BlenderRgbaPre};
    pub const PIXEL_FORMAT: PixFormat = PixFormat::Bgra32;
    pub type Pixfmt<'a> = PixBgra32<'a>;
    pub type PixfmtPre<'a> = PixBgra32Pre<'a>;
    pub type ColorType = agg::Rgba8;
    pub type OrderType = agg::color_rgba::OrderBgra;
    pub type Blender = BlenderRgba<ColorType, OrderType>;
    pub type BlenderPre = BlenderRgbaPre<ColorType, OrderType>;
}

#[cfg(feature = "agg_rgba32")] {
    use agg::pixfmt_rgba::{PixRgba32, PixRgba32Pre, BlenderRgba, BlenderRgbaPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Rgba32;
    pub type Pixfmt<'a> = PixRgba32<'a>;
     pub type PixfmtPre<'a> = PixRgba32Pre<'a>;
     pub type Blender = BlenderRgba<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbaPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba8;
     pub type OrderType = agg::color_rgba::OrderRgba;
}

#[cfg(feature = "agg_argb32")] {
    use agg::pixfmt_rgba::{PixArgb32, PixArgb32Pre, BlenderRgba, BlenderRgbaPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Argb32;
    pub type Pixfmt<'a> = PixArgb32<'a>;
     pub type PixfmtPre<'a> = PixArgb32Pre<'a>;
     pub type Blender = BlenderRgba<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbaPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba8;
     pub type OrderType = agg::color_rgba::OrderArgb;
}

#[cfg(feature = "agg_abgr32")] {
    use agg::pixfmt_rgba::{PixAbgr32, PixAbgr32Pre, BlenderRgba, BlenderRgbaPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Abgr32;
    pub type Pixfmt<'a> = PixAbgr32<'a>;
     pub type PixfmtPre<'a> = PixAbgr32Pre<'a>;
     pub type Blender = BlenderRgba<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbaPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba8;
     pub type OrderType = agg::color_rgba::OrderAbgr;
}

#[cfg(feature = "agg_bgr48")] {
    use agg::pixfmt_rgb::{PixBgr48, PixBgr48Pre, BlenderRgb, BlenderRgbPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Bgr48;
    pub type Pixfmt<'a> = PixBgr48<'a>;
     pub type PixfmtPre<'a> = PixBgr48Pre<'a>;
     pub type Blender = BlenderRgb<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba16;
     pub type OrderType = agg::color_rgba::OrderBgr;
}

#[cfg(feature = "agg_rgb48")] {
    use agg::pixfmt_rgb::{PixRgb48, PixRgb48Pre, BlenderRgb, BlenderRgbPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Rgb48;
    pub type Pixfmt<'a> = PixRgb48<'a>;
     pub type PixfmtPre<'a> = PixRgb48Pre<'a>;
     pub type Blender = BlenderRgb<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba16;
     pub type OrderType = agg::color_rgba::OrderRgb;
}

#[cfg(feature = "agg_bgra64")] {
    use agg::pixfmt_rgba::{PixBgra64, PixBgra64Pre, BlenderRgba, BlenderRgbaPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Bgra64;
    pub type Pixfmt<'a> = PixBgra64<'a>;
     pub type PixfmtPre<'a> = PixBgra64Pre<'a>;
     pub type Blender = BlenderRgba<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbaPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba16;
     pub type OrderType = agg::color_rgba::OrderBgra;
}

#[cfg(feature = "agg_rgba64")] {
    use agg::pixfmt_rgba::{PixRgba64, PixRgba64Pre, BlenderRgba, BlenderRgbaPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Rgba64;
    pub type Pixfmt<'a> = PixRgba64<'a>;
     pub type PixfmtPre<'a> = PixRgba64Pre<'a>;
     pub type Blender = BlenderRgba<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbaPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba16;
     pub type OrderType = agg::color_rgba::OrderRgba;
}

#[cfg(feature = "agg_argb64")] {
    use agg::pixfmt_rgba::{PixArgb64, PixArgb64Pre, BlenderRgba, BlenderRgbaPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Argb64;
    pub type Pixfmt<'a> = PixArgb64<'a>;
     pub type PixfmtPre<'a> = PixArgb64Pre<'a>;
     pub type Blender = BlenderRgba<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbaPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba16;
     pub type OrderType = agg::color_rgba::OrderArgb;
}

#[cfg(feature = "agg_abgr64")] {
    use agg::pixfmt_rgba::{PixAbgr64, PixAbgr64Pre, BlenderRgba, BlenderRgbaPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Abgr64;
    pub type Pixfmt<'a> = PixAbgr64<'a>;
     pub type PixfmtPre<'a> = PixAbgr64Pre<'a>;
     pub type Blender = BlenderRgba<ColorType, OrderType>;
     pub type BlenderPre = BlenderRgbaPre<ColorType, OrderType>;
     pub type ColorType = agg::Rgba16;
     pub type OrderType = agg::color_rgba::OrderAbgr;
}

#[cfg(feature = "agg_rgb555")] {
    use agg::pixfmt_rgb_packed::{PixRgb555, PixRgb555Pre, BlenderRgb555, BlenderRgb555Pre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Rgb555;
    pub type Pixfmt<'a> = PixRgb555<'a>;
     pub type PixfmtPre<'a> = PixRgb555Pre<'a>;
     pub type Blender = BlenderRgb555;
     pub type BlenderPre = BlenderRgb555Pre;
     pub type ColorType = agg::Rgba8;
}

#[cfg(feature = "agg_rgb565")] {
    use agg::pixfmt_rgb_packed::{PixRgb565, PixRgb565Pre, BlenderRgb565, BlenderRgb565Pre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::Rgb565;
    pub type Pixfmt<'a> = PixRgb565<'a>;
     pub type PixfmtPre<'a> = PixRgb565Pre<'a>;
     pub type Blender = BlenderRgb565;
     pub type BlenderPre = BlenderRgb565Pre;
     pub type ColorType = agg::Rgba8;
}

#[cfg(feature = "agg_rgbAAA")] {
    use agg::pixfmt_rgb_packed::{PixRgbAAA, PixRgbAAAPre, BlenderRgbAAA, BlenderRgbAAAPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::RgbAAA;
    pub type Pixfmt<'a> = PixRgbAAA<'a>;
     pub type PixfmtPre<'a> = PixRgbAAAPre<'a>;
     pub type Blender = BlenderRgbAAA;
     pub type BlenderPre = BlenderRgbAAAPre;
     pub type ColorType = agg::Rgba16;
}

#[cfg(feature = "agg_bgrAAA")] {
    use agg::pixfmt_rgb_packed::{PixBgrAAA, PixBgrAAAPre, BlenderBgrAAA, BlenderBgrAAAPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::BgrAAA;
    pub type Pixfmt<'a> = PixBgrAAA<'a>;
     pub type PixfmtPre<'a> = PixBgrAAAPre<'a>;
     pub type Blender = BlenderBgrAAA;
     pub type BlenderPre = BlenderBgrAAAPre;
     pub type ColorType = agg::Rgba16;
}

#[cfg(feature = "agg_rgbBBA")] {
    use agg::pixfmt_rgb_packed::{PixRgbBBA, PixRgbBBAPre, BlenderRgbBBA, BlenderRgbBBAPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::RgbBBA;
    pub type Pixfmt<'a> = PixRgbBBA<'a>;
     pub type PixfmtPre<'a> = PixRgbBBAPre<'a>;
     pub type Blender = BlenderRgbBBA;
     pub type BlenderPre = BlenderRgbBBAPre;
     pub type ColorType = agg::Rgba16;
}

#[cfg(feature = "agg_bgrABB")] {
    use agg::pixfmt_rgb_packed::{PixBgrABB, PixBgrABBPre, BlenderBgrABB, BlenderBgrABBPre};
     pub const PIXEL_FORMAT: PixFormat = PixFormat::BgrABB;
    pub type Pixfmt<'a> = PixBgrABB<'a>;
     pub type PixfmtPre<'a> = PixBgrABBPre<'a>;
     pub type Blender = BlenderBgrABB;
     pub type BlenderPre = BlenderBgrABBPre;
     pub type ColorType = agg::Rgba16;
}

}
