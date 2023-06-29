#![allow(unused_parens)]

use std::ops::*;
use std::os::raw::c_long;

pub mod alpha_mask_u8;
pub mod arc;
pub mod array;
pub mod arrowhead;
pub mod basics;
pub mod bezier_arc;
pub mod bitset_iterator;
pub mod blur;
pub mod bounding_rect;
pub mod bspline;
pub mod clip_liang_barsky;
pub mod color_gray;
pub mod color_rgba;
pub mod conv_adaptor_vcgen;
pub mod conv_adaptor_vpgen;
pub mod conv_bspline;
pub mod conv_clip_polyline;
pub mod conv_concat;
pub mod conv_contour;
pub mod conv_curve;
pub mod conv_dash;
pub mod conv_gpc;
pub mod conv_marker;
pub mod conv_marker_adaptor;
pub mod conv_marker_concat;
pub mod conv_segmentator;
pub mod conv_smooth_poly1;
pub mod conv_stroke;
pub mod conv_transform;
pub mod curves;
pub mod dda_line;
pub mod ellipse;
pub mod ellipse_bresenham;
pub mod embedded_raster_fonts;
pub mod font_cache_manager;
pub mod font_freetype;
pub mod gamma_functions;
pub mod gamma_lut;
pub mod glyph_raster_bin;
pub mod gradient_lut;
pub mod gsv_text;
pub mod image_accessors;
pub mod image_filters;
pub mod line_aa_basics;
pub mod math;
pub mod math_stroke;
pub mod path_storage;
pub mod path_storage_integer;
pub mod pattern_filters_rgba;
pub mod pixfmt_amask_adaptor;
pub mod pixfmt_gray;
pub mod pixfmt_rgb;
pub mod pixfmt_rgb_packed;
pub mod pixfmt_rgba;
pub mod pixfmt_transposer;
mod rasterizer_cells_aa;
pub mod rasterizer_compound_aa;
pub mod rasterizer_outline;
pub mod rasterizer_outline_aa;
pub mod rasterizer_scanline_aa;
pub mod rasterizer_sl_clip;
pub mod renderer_base;
pub mod renderer_markers;
pub mod renderer_mclip;
pub mod renderer_outline_aa;
pub mod renderer_outline_image;
pub mod renderer_primitives;
pub mod renderer_raster_text;
pub mod renderer_scanline;
pub mod rendering_buffer;
pub mod rounded_rect;
pub mod scanline_bin;
pub mod scanline_boolean_algebra;
pub mod scanline_p;
pub mod scanline_storage_aa;
pub mod scanline_storage_bin;
pub mod scanline_u;
pub mod shorten_path;
pub mod simul_eq;
pub mod span_allocator;
pub mod span_converter;
pub mod span_gouraud;
pub mod span_gouraud_gray;
pub mod span_gouraud_rgba;
pub mod span_gradient;
pub mod span_gradient_alpha;
pub mod span_image_filter;
pub mod span_image_filter_gray;
pub mod span_image_filter_rgb;
pub mod span_image_filter_rgba;
pub mod span_interpolator_adaptor;
pub mod span_interpolator_linear;
pub mod span_interpolator_persp;
pub mod span_interpolator_trans;
pub mod span_pattern_gray;
pub mod span_pattern_rgb;
pub mod span_pattern_rgba;
pub mod span_solid;
pub mod span_subdiv_adaptor;
pub mod trans_affine;
pub mod trans_bilinear;
pub mod trans_double_path;
pub mod trans_perspective;
pub mod trans_single_path;
pub mod trans_viewport;
pub mod vcgen_bspline;
pub mod vcgen_contour;
pub mod vcgen_dash;
pub mod vcgen_markers_term;
pub mod vcgen_smooth_poly1;
pub mod vcgen_stroke;
pub mod vcgen_vertex_sequence;
pub mod vertex_sequence;
pub mod vpgen_clip_polygon;
pub mod vpgen_clip_polyline;
pub mod vpgen_segmentator;

pub mod util;

// Modules not tested. No Examples
pub mod conv_close_polygon;
pub mod conv_unclose_polygon;
pub mod path_length;
pub mod rendering_buffer_dynarrow;
pub mod trans_wrap_magnifier;

// Namespace
pub use array::{PodBVector, VecPodB};
pub use arrowhead::Arrowhead;
pub use basics::{deg2rad, is_vertex, FillingRule, RectD, RectI, RowData, Span};
pub use blur::{stack_blur_gray8, stack_blur_rgb24, stack_blur_rgb32};
pub use bounding_rect::{bounding_rect, bounding_rect_single};
pub use bspline::Bspline;
pub use color_gray::{Gray16, Gray8};
pub use color_rgba::{Rgba, Rgba16, Rgba8};
pub use conv_bspline::ConvBspline;
pub use conv_concat::ConvConcat;
pub use conv_contour::ConvContour;
pub use conv_curve::ConvCurve;
pub use conv_dash::ConvDash;
pub use conv_gpc::{ConvGpc, GpcOp};
pub use conv_marker::ConvMarker;
pub use conv_marker_adaptor::ConvMarkerAdaptor;
pub use conv_marker_concat::ConvMarkerConcat;
pub use conv_segmentator::ConvSegmentator;
pub use conv_smooth_poly1::{ConvSmoothPoly1, ConvSmoothPoly1Curve};
pub use conv_stroke::ConvStroke;
pub use conv_transform::ConvTransform;
pub use curves::{Curve3, Curve4};
pub use ellipse::Ellipse;
pub use font_cache_manager::{FontCacheManager, GlyphCache, GlyphDataType, GlyphRender};
pub use font_freetype::FreetypeBase;
pub use gamma_functions::{GammaLinear, GammaMultiply, GammaNone, GammaPower, GammaThreshold};
pub use gamma_lut::GammaLut;
pub use gradient_lut::{ColorIp, GradientLut};
pub use gsv_text::{GsvText, GsvTextOutline};
pub use image_accessors::{
    ImageAccessorClip, ImageAccessorClone, ImageAccessorNoClip, ImageAccessorWrap, WrapModeReflect,
    WrapModeReflectAutoPow2, WrapModeRepeat, WrapModeRepeatAutoPow2, WrapModeRepeatPow2,
};
pub use image_filters::{
    ImageFilter, ImageFilterBessel, ImageFilterBicubic, ImageFilterBilinear, ImageFilterBlackman,
    ImageFilterCatrom, ImageFilterGaussian, ImageFilterHamming, ImageFilterHanning,
    ImageFilterHermite, ImageFilterKaiser, ImageFilterLanczos, ImageFilterLut, ImageFilterMitchell,
    ImageFilterQuadric, ImageFilterScale, ImageFilterSinc, ImageFilterSpline16,
    ImageFilterSpline36, ImageSubpixelScale,
};
pub use line_aa_basics::LineCoord;
pub use math::{calc_distance, point_in_triangle};
pub use math_stroke::{InnerJoin, LineCap, LineJoin, MathStroke};
pub use path_storage::{PathBase, PathStorage, PolyPlainAdaptor};
pub use path_storage_integer::{PathStorageInteger, SerializedIntegerPathAdaptor};
pub use pattern_filters_rgba::{PatternFilterBilinearRgba16, PatternFilterBilinearRgba8};
pub use pixfmt_gray::{
    AlphaBlendGray, BlenderGray, BlenderGray16, BlenderGray8, BlenderGrayPre, PixGray16, PixGray8,
};
pub use pixfmt_rgb::{
    BlenderRgb, BlenderRgbGamma, BlenderRgbPre, PixBgr24, PixBgr24Gamma, PixBgr24Pre, BlenderBgr24, BlenderBgr24Pre
};
pub use pixfmt_rgba::{BlenderBgra32, BlenderBgra32Pre, BlenderRgbaPre, PixBgra32, PixBgra32Pre};
pub use rasterizer_compound_aa::{LayerOrder, RasterizerCompoundAa};
pub use rasterizer_outline::RasterizerOutline;
pub use rasterizer_outline_aa::RasterizerOutlineAa;
pub use rasterizer_scanline_aa::RasterizerScanlineAa;
pub use rasterizer_sl_clip::{
    RasterizerSlClip, RasterizerSlClipDbl, RasterizerSlClipDbl3x, RasterizerSlClipInt,
    RasterizerSlClipInt3x, RasterizerSlClipIntSat,
};
pub use renderer_base::RendererBase;
pub use renderer_markers::{MarkerType, RendererMarkers};
pub use renderer_mclip::RendererMclip;
pub use renderer_outline_aa::{LineProfileAA, RendererOutlineAa};
pub use renderer_outline_image::{
    LineImagePattern, LineImagePatternPow2, LineImageScale, RendererOutlineImage,
};
pub use renderer_primitives::RendererPrimitives;
pub use renderer_raster_text::{
    RendererRasterHtext, RendererRasterHtextSolid, RendererRasterVtextSolid,
};
pub use renderer_scanline::{
    render_all_paths, render_scanline_aa, render_scanlines, render_scanlines_aa,
    render_scanlines_aa_solid, render_scanlines_bin_solid, render_scanlines_compound,
    render_scanlines_compound_layered, RendererScanlineAA, RendererScanlineAASolid,
    RendererScanlineBin, RendererScanlineBinSolid,
};
pub use rendering_buffer::RenderBuf;
pub use rounded_rect::RoundedRect;
pub use scanline_bin::{Scanline32Bin, ScanlineBin};
pub use scanline_boolean_algebra::{sbool_combine_shapes_aa, sbool_combine_shapes_bin, SBoolOp};
pub use scanline_p::ScanlineP8;
pub use scanline_storage_aa::{
    ScanlineStorageAA, ScanlineStorageAA16, ScanlineStorageAA32, ScanlineStorageAA8,
};
pub use scanline_storage_bin::ScanlineStorageBin;
pub use scanline_u::ScanlineU8;
pub use span_allocator::VecSpan;
pub use span_converter::SpanProcess;
pub use span_gouraud_gray::SpanGouraudGray;
pub use span_gouraud_rgba::SpanGouraudRgba;
pub use span_gradient::{
    GradientCircle, GradientConic, GradientDiamond, GradientLinearColor, GradientRadial,
    GradientRadialD, GradientRadialFocus, GradientReflectAdaptor, GradientRepeatAdaptor,
    GradientSqrtXY, GradientX, GradientXY, GradientY, SpanGradient,
};
pub use span_gradient_alpha::SpanGradientAlpha;
pub use span_image_filter_rgb::{
    SpanImageFilterRgb, SpanImageFilterRgb2x2, SpanImageFilterRgbBilinear,
    SpanImageFilterRgbBilinearClip, SpanImageFilterRgbNn, SpanImageResampleRgb,
    SpanImageResampleRgbAffine,
};
pub use span_image_filter_rgba::{
    SpanImageFilterRgba, SpanImageFilterRgba2x2, SpanImageFilterRgbaBilinear,
    SpanImageFilterRgbaBilinearClip, SpanImageFilterRgbaNn, SpanImageResampleRgba,
    SpanImageResampleRgbaAffine,
};
pub use span_interpolator_adaptor::SpanIpAdaptor;
pub use span_interpolator_linear::{SpanIpLinear, SpanIpLinearSubdiv};
pub use span_interpolator_persp::{SpanIpPerspExact, SpanIpPerspLerp};
pub use span_interpolator_trans::SpanIpTrans;
pub use span_subdiv_adaptor::SpanSubdivAdaptor;
pub use trans_affine::TransAffine;
pub use trans_bilinear::TransBilinear;
pub use trans_perspective::TransPerspective;
pub use trans_viewport::{AspectRatio, TransViewport};
pub use trans_wrap_magnifier::TransWarpMagnifier;
pub use vcgen_markers_term::VcgenMarkersTerm;
pub use vcgen_stroke::VcgenStroke;
pub use vcgen_vertex_sequence::VcgenVertexSequence;

use curves::CurveApproximationMethod;
use line_aa_basics::LineParameters;

// Traits
pub trait RenderBuffer {
    type T;

    fn attach(&mut self, buf: *mut Self::T, width: u32, height: u32, stride: i32);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn stride(&self) -> i32;
    fn stride_abs(&self) -> u32;
    fn row(&self, y: i32) -> &[Self::T];
    fn row_mut(&mut self, y: i32) -> &mut [Self::T];
    fn row_data(&self, y: i32) -> RowData<Self::T>;
    fn copy_from<RenBuf: crate::RenderBuffer<T = Self::T>>(&mut self, src: &RenBuf);
}

pub trait ImageSrc {}
pub trait PixFmtGray: PixFmt {
    const PIXEL_STEP: u32;
    const PIXEL_OFFSET: u32;
}

pub trait PixFmt: ImageSrc {
    type C: Color;
    type O: Order;
    type T: AggPrimitive;
    const PIXEL_WIDTH: u32;
    fn attach_pixfmt<Pix: PixFmt>(
        &mut self, pixf: &Pix, x1: i32, y1: i32, x2: i32, y2: i32,
    ) -> bool;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn stride(&self) -> i32;
    fn pix_ptr(&self, x: i32, y: i32) -> (&[u8], usize); //RB::T;
    fn pix_ptr_mut(&mut self, x: i32, y: i32) -> (&mut [u8], usize) {
        let (a, i) = self.pix_ptr(x, y);
        (
            unsafe { std::slice::from_raw_parts_mut(a.as_ptr() as *mut u8, a.len()) },
            i,
        )
    }
    fn row(&self, y: i32) -> &[Self::T];
    fn row_mut(&mut self, y: i32) -> &mut [Self::T];
    fn row_data(&self, y: i32) -> RowData<Self::T>;
    fn make_pix(&self, p: &mut [u8], c: &Self::C);
    fn copy_pixel(&mut self, x: i32, y: i32, c: &Self::C);
    fn blend_from<R: PixFmt<T = Self::T>>(
        &mut self, from: &R, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32, cover: u32,
    );
    fn blend_from_color<R: PixFmt<T = Self::T>>(
        &mut self, from: &R, color: &Self::C, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32,
        cover: u32,
    );
    fn blend_from_lut<R: PixFmt<T = Self::T>>(
        &mut self, from: &R, color_lut: &[Self::C], xdst: i32, ydst: i32, xsrc: i32, ysrc: i32,
        len: u32, cover: u32,
    );
    fn blend_pixel(&mut self, x: i32, y: i32, c: &Self::C, cover: u8);
    fn pixel(&self, x: i32, y: i32) -> Self::C;
    fn copy_hline(&mut self, x: i32, y: i32, len: u32, c: &Self::C);
    fn copy_vline(&mut self, x: i32, y: i32, len: u32, c: &Self::C);
    fn blend_hline(&mut self, x: i32, y: i32, len: u32, c: &Self::C, cover: u8);
    fn blend_vline(&mut self, x: i32, y: i32, len: u32, c: &Self::C, cover: u8);
    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: u32, c: &Self::C, covers: &[u8]);
    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: u32, c: &Self::C, covers: &[u8]);
    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[Self::C], covers: &[u8], cover: u8,
    );
    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[Self::C], covers: &[u8], cover: u8,
    );
    fn copy_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]);
    fn copy_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]);
    fn copy_from<Pix: RenderBuffer<T = Self::T>>(
        &mut self, from: &Pix, xdst: i32, ydst: i32, xsrc: i32, ysrc: i32, len: u32,
    );
}

pub trait Renderer {
    type C: Color;

    fn bounding_clip_box(&self) -> &RectI;
    fn blend_pixel(&mut self, x: i32, y: i32, c: &Self::C, cover: u8);
    fn pixel(&mut self, x: i32, y: i32) -> Self::C;
    fn copy_hline(&mut self, x1: i32, y: i32, x2: i32, c: &Self::C);
    fn copy_vline(&mut self, x: i32, y1: i32, y2: i32, c: &Self::C);
    fn blend_hline(&mut self, x1: i32, y: i32, x2: i32, c: &Self::C, cover: u8);
    fn blend_vline(&mut self, x: i32, y1: i32, y2: i32, c: &Self::C, cover: u8);
    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: i32, c: &Self::C, covers: &[u8]);
    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: i32, c: &Self::C, covers: &[u8]);
    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: i32, colors: &[Self::C], covers: &[u8], cover: u8,
    );
    fn blend_color_vspan(
        &mut self, x: i32, y: i32, len: i32, colors: &[Self::C], covers: &[u8], cover: u8,
    );
    fn copy_color_hspan(&mut self, x: i32, y: i32, len: i32, colors: &[Self::C]);
    fn copy_color_vspan(&mut self, x: i32, y: i32, len: i32, colors: &[Self::C]);
    fn blend_bar(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: &Self::C, cover: u8);
    /*fn copy_bar(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: &C);
    fn blend_color_hspan_no_clip(&mut self, x: i32, y: i32, len: i32, colors: &[C], covers: &[u8]);
    fn blend_color_vspan_no_clip(&mut self, x: i32, y: i32, len: i32, colors: &[C], covers: &[u8]);
    fn copy_from(&mut self, from: &R, rc: Option<&dyn Rect>, x_to: i32, y_to: i32);*/
}
pub trait Clip {
    fn reset_clipping(&mut self, visibility: bool);
    fn clip_box(&self) -> &RectI;
    fn xmin(&self) -> i32;
    fn ymin(&self) -> i32;
    fn xmax(&self) -> i32;
    fn ymax(&self) -> i32;
    fn bounding_clip_box(&self) -> &RectI;
    fn bounding_xmin(&self) -> i32;
    fn bounding_ymin(&self) -> i32;
    fn bounding_xmax(&self) -> i32;
    fn bounding_ymax(&self) -> i32;
}

pub trait RendererScanlineColor: RendererScanline {
    type C: Color;

    fn set_color(&mut self, _c: Self::C);
    fn color(&self) -> Self::C {
        todo!()
    }
}

pub trait RendererScanline {
    fn prepare(&mut self);
    fn render<Sl: Scanline>(&mut self, sl: &Sl);
}

pub trait RasterStyle<C: Color> {
    fn is_solid(&self, style: u32) -> bool;
    fn color(&self, style: u32) -> &C;
    fn generate_span(&mut self, span: &mut [C], x: i32, y: i32, len: u32, style: u32);
}

pub trait Rasterizer0 {
    fn sweep_styles(&mut self) -> u32;
    fn style(&self, idx: u32) -> u32;
    fn allocate_cover_buffer(&mut self, len: u32) -> &mut [u8]; // {&mut[]}
    fn scanline_start(&self) -> i32; // {0}
    fn scanline_length(&self) -> u32; // {0}
}

pub trait Rasterizer {
    fn reset(&mut self);
    fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32);
    fn render_hline(&mut self, ey: i32, x1: i32, y1: i32, x2: i32, y2: i32);
}
pub trait RasterScanLine {
    fn reset(&mut self) {}
    fn rewind_scanlines(&mut self) -> bool { false }
    fn add_path<VS: VertexSource>(&mut self, _vs: &mut VS, _path_id: u32) {}
    fn min_x(&self) -> i32;
    fn min_y(&self) -> i32;
    fn max_x(&self) -> i32;
    fn max_y(&self) -> i32;
    fn sweep_scanline<Sl: Scanline>(&mut self, _sl: &mut Sl) -> bool {
        false
    }
    fn sweep_scanline_with_style<Sl: Scanline>(&mut self, _sl: &mut Sl, _style_idx: i32) -> bool {
        false
    }
    fn apply_gamma(&self, _cover: usize) -> u32 {
        0
    }
}

pub trait Scanline {
    type CoverType: AggPrimitive;

    fn y(&self) -> i32;
    fn num_spans(&self) -> u32;
    fn begin(&self) -> &[Span];
    fn reset(&mut self, min_x: i32, max_x: i32);
    fn reset_spans(&mut self);
    fn add_cell(&mut self, x: i32, cover: u32);
    fn add_span(&mut self, x: i32, len: u32, cover: u32);
    fn add_cells(&mut self, x: i32, len: u32, covers: &[Self::CoverType]);
    fn finalize(&mut self, y: i32);
}

pub trait Gamma<LoResT: AggInteger, HiResT: AggInteger> {
	fn new() -> Self;
    fn dir(&self, v: LoResT) -> HiResT;
    fn inv(&self, v: HiResT) -> LoResT;
}

pub trait Order {
    const R: usize;
    const G: usize;
    const B: usize;
    const A: usize;
    const TAG: usize;
}

pub trait Args {
    type ValueType: AggInteger;

    fn a(&self) -> Self::ValueType;
    fn a_mut(&mut self) -> &mut Self::ValueType;
}

pub trait RgbArgs: Args {
    fn r(&self) -> Self::ValueType;
    fn g(&self) -> Self::ValueType;
    fn b(&self) -> Self::ValueType;
    fn r_mut(&mut self) -> &mut Self::ValueType;
    fn g_mut(&mut self) -> &mut Self::ValueType;
    fn b_mut(&mut self) -> &mut Self::ValueType;
    fn new_init(
        r: Self::ValueType, g: Self::ValueType, b: Self::ValueType, a: Self::ValueType,
    ) -> Self;
}

pub trait GrayArgs: Args {
    fn v(&self) -> Self::ValueType;
    fn v_mut(&mut self) -> &mut Self::ValueType;
    fn new_init(v: Self::ValueType, a: Self::ValueType) -> Self;
}

pub trait Color: Args + Copy {
    const BASE_SHIFT: u32;
    const BASE_SCALE: u32;
    const BASE_MASK: u32;

    fn new() -> Self;
    fn new_from_rgba(c: &Rgba) -> Self;
    fn clear(&mut self);
    fn transparent(&mut self) -> &Self;
    fn set_opacity(&mut self, a: f64) -> &Self;
    fn opacity(&self) -> f64;
    fn premultiply(&mut self) -> &Self;
    fn premultiply_a(&mut self, a_: u32) -> &Self;
    fn demultiply(&mut self) -> &Self;
    fn gradient(&self, c: &Self, k: f64) -> Self;
    fn add(&mut self, c: &Self, cover: u32);
    fn no_color() -> Self;
}

pub trait PathStore {
    type T: AggInteger;

    fn move_to(&mut self, x: Self::T, y: Self::T);
    fn line_to(&mut self, x: Self::T, y: Self::T);
    fn curve3(&mut self, x_ctrl: Self::T, y_ctrl: Self::T, x_to: Self::T, y_to: Self::T);
    fn curve4(
        &mut self, x_ctrl1: Self::T, y_ctrl1: Self::T, x_ctrl2: Self::T, y_ctrl2: Self::T,
        x_to: Self::T, y_to: Self::T,
    );
    fn close_polygon(&mut self);
}

pub trait Blender<C: Color, O: Order> {
	fn new() -> Self;
    fn blend_pix_with_cover(
        &self, _p: &mut [C::ValueType], _cr: u32, _cg: u32, _cb: u32, _alpha: u32, _cover: u32,
    ) {
        todo!()
    }
    fn blend_pix(&self, _p: &mut [C::ValueType], _cr: u32, _cg: u32, _cb: u32, _alpha: u32) {
        todo!()
    }
}

pub trait BlenderOp<C: Color, O: Order> {
	fn new() -> Self;
    fn blend_pix_with_cover(
        &self, _op: u32, _p: &mut [C::ValueType], _cr: u32, _cg: u32, _cb: u32, _alpha: u32,
        _cover: u32,
    ) {
        todo!()
    }
    fn blend_pix(
        &self, _op: u32, _p: &mut [C::ValueType], _cr: u32, _cg: u32, _cb: u32, _alpha: u32,
    ) {
        todo!()
    }
}

pub trait BlenderG<C: Color> {
	fn new() -> Self;
    fn blend_pix_with_cover(&self, _p: &mut C::ValueType, _v: u32, _alpha: u32, _cover: u32) {
        todo!()
    }
    fn blend_pix(&self, _p: &mut C::ValueType, _v: u32, _alpha: u32) {
        todo!()
    }
}

pub trait BlenderPacked {
    type PixelType: AggInteger;
    type ColorType: Color + RgbArgs;
    type ValueType: AggInteger;

	fn new() -> Self;

    fn blend_pix_with_cover(
        &self, _p: &mut Self::PixelType, _cr: u32, _cg: u32, _cb: u32, _alpha: u32, _cover: u32,
    ) {
        todo!()
    }
    fn blend_pix(&self, _p: &mut Self::PixelType, _cr: u32, _cg: u32, _cb: u32, _alpha: u32) {
        todo!()
    }
    fn make_pix(&self, r: u32, g: u32, b: u32) -> Self::PixelType;
    fn make_color(&self, p: Self::PixelType) -> Self::ColorType;
}

pub trait Point {
    type T;

    fn new(x: Self::T, y: Self::T) -> Self;
}

pub trait VertexSource {
    fn rewind(&mut self, id: u32);
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32;
}

pub trait RasConv {
    type CoordType: AggPrimitive;

    fn mul_div(a: f64, b: f64, c: f64) -> Self::CoordType;
    fn xi(v: Self::CoordType) -> i32;
    fn yi(v: Self::CoordType) -> i32;
    fn upscale(v: f64) -> Self::CoordType;
    fn downscale(v: i32) -> Self::CoordType;
}

pub trait RasClip {
    type CoordType: AggPrimitive;
    type ConvType: RasConv;

    fn new() -> Self;
    fn reset_clipping(&mut self);
    fn move_to(
        &mut self, x1: <Self::ConvType as RasConv>::CoordType,
        y1: <Self::ConvType as RasConv>::CoordType,
    );
    fn line_to<R: Rasterizer>(
        &mut self, ras: &mut R, x2: <Self::ConvType as RasConv>::CoordType,
        y2: <Self::ConvType as RasConv>::CoordType,
    );
    fn clip_box(
        &mut self, x1: <Self::ConvType as RasConv>::CoordType,
        y1: <Self::ConvType as RasConv>::CoordType, x2: <Self::ConvType as RasConv>::CoordType,
        y2: <Self::ConvType as RasConv>::CoordType,
    );
}

pub trait CellFn {
    fn style(
        &mut self, me: &mut crate::rasterizer_cells_aa::Cell, c: &crate::rasterizer_cells_aa::Cell,
    );
    fn not_equal(
        &self, me: &crate::rasterizer_cells_aa::Cell, ex: i32, ey: i32,
        c: &crate::rasterizer_cells_aa::Cell,
    ) -> i32;
}

pub trait GammaFn {
    fn call(&self, x: f64) -> f64;
}

pub trait AlphaMask {
    type CoverType: AggInteger;
    const COVER_FULL: u32;

    fn pixel(&self, x: i32, y: i32) -> Self::CoverType;
    fn combine_pixel(&self, x: i32, y: i32, val: Self::CoverType) -> Self::CoverType;
    fn fill_hspan(&self, x: i32, y: i32, dst: &mut [Self::CoverType], num_pix: i32);
    fn combine_hspan(&self, x: i32, y: i32, dst: &mut [Self::CoverType], num_pix: i32);
    fn fill_vspan(&self, x: i32, y: i32, dst: &mut [Self::CoverType], num_pix: i32);
    fn combine_vspan(&self, x: i32, y: i32, dst: &mut [Self::CoverType], num_pix: i32);
}

pub trait VertexSequence<T = crate::vertex_sequence::VertexDist> {
    type ValueType;

    fn close(&mut self, closed: bool);
    fn size(&self) -> usize;
    fn remove_all(&mut self);
    fn remove_last(&mut self);
    fn get(&self, i: usize) -> &Self::ValueType;
    fn get_mut(&mut self, i: usize) -> &mut Self::ValueType;
    fn get_mut_slice(&mut self, s: usize, e: usize) -> &mut [Self::ValueType];
    fn add(&mut self, val: Self::ValueType);
    //fn prev(i: usize) -> &Self::ValueType;
    //fn modify_last(&mut self, val: &T);
}

pub trait VertexDistance {
    fn calc_distance(&mut self, v: &Self) -> bool;
}

pub trait VertexConsumer {
    type ValueType;

    fn remove_all(&mut self);
    fn add(&mut self, val: Self::ValueType);
}

pub trait SpanGenerator {
    type C: Color;

    fn prepare(&mut self);// {}
    fn generate(&mut self, span: &mut [Self::C], x: i32, y: i32, len: u32);
}

pub trait SpanAllocator {
    type C: Color;

    fn allocate(&mut self, span_len: usize) -> &mut [Self::C];
}

pub trait VertexSourceWithMarker: VertexSource {
    type Mrk: Markers;

    fn markers_mut(&mut self) -> &mut Self::Mrk;
}

pub trait Generator: VertexSource /* : VertexSource + Markers*/ {
    fn new() -> Self;
    fn remove_all(&mut self);
    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32);
}

pub trait Transformer {
    fn transform(&self, x: &mut f64, y: &mut f64);
    fn scaling_abs(&self, _x: &mut f64, _y: &mut f64) {}
}

pub trait VertexContainer {
    fn new() -> Self;
    fn remove_all(&mut self);
    fn free_all(&mut self);
    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32);
    fn modify_vertex(&mut self, idx: u32, x: f64, y: f64);
    fn modify_vertex_with_cmd(&mut self, idx: u32, x: f64, y: f64, cmd: u32);
    fn modify_command(&mut self, idx: u32, cmd: u32);
    fn swap_vertices(&mut self, v1: u32, v2: u32);
    fn last_command(&self) -> u32;
    fn last_vertex(&self, x: &mut f64, y: &mut f64) -> u32;
    fn prev_vertex(&self, x: &mut f64, y: &mut f64) -> u32;
    fn last_x(&self) -> f64;
    fn last_y(&self) -> f64;
    fn total_vertices(&self) -> u32;
    fn vertex(&self, idx: u32, x: &mut f64, y: &mut f64) -> u32;
    fn command(&self, idx: u32) -> u32;
}

pub trait Markers: Generator + VertexSource {}

pub trait ColorFn<C: Color> {
    fn size(&self) -> u32;
    fn get(&mut self, v: u32) -> C;
}

pub trait AlphaFn<V: AggPrimitive> {
    fn size(&self) -> u32;
    fn get(&mut self, v: u32) -> V;
}

pub trait GradientFunc {
    fn calculate(&self, x: i32, y: i32, d: i32) -> i32;
}

pub trait ColorInterpolator {
    type C: Color;
    fn new(c1: Self::C, c2: Self::C, len: u32) -> Self;
    fn next(&mut self);
    fn color(&self) -> Self::C;
}

pub trait Interpolator {
    type Trf: Transformer;
    const SUBPIXEL_SHIFT: u32;

    fn begin(&mut self, x: f64, y: f64, len: u32);
    fn next(&mut self);
    fn coordinates(&self, x: &mut i32, y: &mut i32);
    fn local_scale(&self, x: &mut i32, y: &mut i32) {
        *x = 0;
        *y = 0;
        todo!()
    }
    fn resynchronize(&mut self, _xe: f64, _ye: f64, _len: u32) {
        todo!()
    }
}

pub trait BlurCalcRgb: RgbArgs {
    fn new() -> Self;
    fn clear(&mut self);
    fn add<A: RgbArgs>(&mut self, v: &A);
    fn add_k<A: RgbArgs>(&mut self, v: &A, k: u32);
    fn sub<A: RgbArgs>(&mut self, v: &A);
    fn calc_pix<A: RgbArgs>(&mut self, v: &mut A, div: u32);
    fn calc_pix_mul<A: RgbArgs>(&mut self, v: &mut A, mul: u32, shr: u32);
}

pub trait BlurCalcGray: GrayArgs {
    fn new() -> Self;
    fn clear(&mut self);
    fn add<A: GrayArgs>(&mut self, v: &A);
    fn add_k<A: GrayArgs>(&mut self, v: &A, k: u32);
    fn sub<A: GrayArgs>(&mut self, v: &A);
    fn calc_pix<A: GrayArgs>(&mut self, v: &mut A, div: u32);
    fn calc_pix_mul<A: GrayArgs>(&mut self, v: &mut A, mul: u32, shr: u32);
}

pub trait BlurCalcRecuRgb: Clone {
    type ValueType: AggPrimitive;

    fn new() -> Self;
    fn from_pix<C: RgbArgs>(&mut self, c: &C);
    fn to_pix<C: RgbArgs>(&self, c: &mut C);
    fn calc(
        &self, b1: Self::ValueType, b2: Self::ValueType, b3: Self::ValueType, b4: Self::ValueType,
        c1: &Self, c2: &Self, c3: &Self, c4: &Self,
    );
}

pub trait BlurCalcRecuGray: Clone {
    type ValueType: AggPrimitive;

    fn new() -> Self;
    fn from_pix<C: GrayArgs>(&mut self, c: &C);
    fn to_pix<C: GrayArgs>(&self, c: &mut C);
    fn calc(
        &self, b1: Self::ValueType, b2: Self::ValueType, b3: Self::ValueType, b4: Self::ValueType,
        c1: &Self, c2: &Self, c3: &Self, c4: &Self,
    );
}

pub trait CurveBase {
    fn new() -> Self;
    fn reset(&mut self);
    fn set_cusp_limit(&mut self, v: f64);
    fn cusp_limit(&self) -> f64;
    fn set_angle_tolerance(&mut self, v: f64);
    fn angle_tolerance(&self) -> f64;
    fn set_approximation_scale(&mut self, s: f64);
    fn approximation_scale(&self) -> f64;
    fn set_approximation_method(&mut self, v: CurveApproximationMethod);
    fn approximation_method(&self) -> CurveApproximationMethod;
}

pub trait CurveType3: CurveBase + VertexSource {
    fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64);
}

pub trait CurveType4: CurveBase + VertexSource {
    fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64);
}
pub trait VpGenerator {
    fn new() -> Self;
    fn auto_close(&self) -> bool;
    fn auto_unclose(&self) -> bool;
    fn reset(&mut self);
    fn move_to(&mut self, x: f64, y: f64);
    fn line_to(&mut self, x: f64, y: f64);
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32;
}

pub trait Coord {
    fn conv(x: f64) -> i32;
}
pub trait FilterF {
    fn radius(&self) -> f64;
    fn set_radius(&mut self, _r: f64) {}
    fn calc_weight(&self, x: f64) -> f64;
}
pub trait ImageWrap {
    fn new(size: u32) -> Self;
    fn get(&mut self, v: i32) -> u32;
    fn inc(&mut self) -> u32;
}
pub trait FontEngine: FontEngineBase {
    type PathAdaptorType: VertexSource;

    fn path_adaptor(&self) -> Self::PathAdaptorType;
    fn new(max_faces: u32) -> Self;
}

pub trait FontEngineBase {
    type Gray8AdaptorType: RasterScanLine;
    type MonoAdaptorType: RasterScanLine;
    type Gray8ScanlineType: Scanline;
    type MonoScanlineType: Scanline;

    fn add_kerning(&self, first: u32, second: u32, x: &mut f64, y: &mut f64) -> bool;
    fn prepare_glyph(&mut self, glyph_code: u32) -> bool;
    fn write_glyph_to(&mut self, data: &mut [u8]);
    fn glyph_index(&self) -> u32;
    fn data_size(&self) -> u32;
    fn data_type(&self) -> crate::font_cache_manager::GlyphDataType;
    fn bounds(&self) -> &RectI;
    fn advance_x(&self) -> f64;
    fn advance_y(&self) -> f64;
    fn font_signature(&self) -> &str;
    fn change_stamp(&self) -> i32;
    fn gray8_adaptor(&self) -> Self::Gray8AdaptorType;
    fn gray8_scanline(&self) -> Self::Gray8ScanlineType;
    fn mono_adaptor(&self) -> Self::MonoAdaptorType;
    fn mono_scanline(&self) -> Self::MonoScanlineType;
}

pub trait GlyphGenerator {
    fn prepare(
        &mut self, r: &mut glyph_raster_bin::GlyphRect, x: f64, y: f64, glyph: u32, flip: bool,
    );
    fn span(&mut self, i: u32) -> &mut [u8];
}

impl<K: ImageAccessor> ImageSrc for K {}

pub trait ImageAccessor: ImageSrc {
    fn span(&mut self, x: i32, y: i32, len: u32) -> &[u8];
    fn next_x(&mut self) -> &[u8];
    fn next_y(&mut self) -> &[u8];
}

pub trait ImageAccessorGray: ImageAccessor {
    type ColorType: Color + GrayArgs;
    type OrderType: Order;
}

pub trait ImageAccessorRgb: ImageAccessor {
    type ColorType: Color + RgbArgs;
    type OrderType: Order;
}

pub trait PatternFilter {
    type ColorType: Color;

    fn new() -> Self;
    fn dilation(&self) -> u32;
    fn pixel_high_res(&self, buf: &[&[Self::ColorType]], p: &mut Self::ColorType, x: i32, y: i32);
    fn pixel_low_res(&self, buf: &[&[Self::ColorType]], p: &mut Self::ColorType, x: i32, y: i32);
}

pub trait SpanConverter {
    type C: Color;

    fn prepare(&mut self) {}
    fn generate(&mut self, span: &mut [Self::C], x: i32, y: i32, len: u32);
}

pub trait Distortion {
    fn calculate(&self, x: &mut i32, y: &mut i32);
}

pub trait RenderPrim {
    type ColorType: Color;

    fn coord(&self, c: f64) -> i32;
    fn set_line_color(&mut self, c: Self::ColorType);
    fn set_fill_color(&mut self, c: Self::ColorType);
    fn line_to(&mut self, x: i32, y: i32, last: bool);
    fn move_to(&mut self, x: i32, y: i32);
}

pub trait RendererOutline {
    type C: Color;

    fn pixel(&self, p: &mut Self::C, x: i32, y: i32);
    fn pattern_width(&self) -> i32;
    fn set_color(&mut self, col: Self::C);
    fn cover(&self, d: i32) -> i32;
    fn blend_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]);
    fn blend_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]);
    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: i32, covers: &[u8]);
    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: i32, covers: &[u8]);
    fn subpixel_width(&self) -> i32;
    fn accurate_join_only(&self) -> bool;
    fn line0(&mut self, lp: &LineParameters);
    fn line1(&mut self, lp: &LineParameters, sx: i32, sy: i32);
    fn line2(&mut self, lp: &LineParameters, ex: i32, ey: i32);
    fn line3(&mut self, lp: &LineParameters, sx: i32, sy: i32, ex: i32, ey: i32);
    fn pie(&mut self, xc: i32, yc: i32, x1: i32, y1: i32, x2: i32, y2: i32);
    fn semidot<Cmp>(&mut self, cmp: Cmp, xc1: i32, yc1: i32, xc2: i32, yc2: i32)
    where
        Cmp: Fn(i32) -> bool;
}

pub trait Pixel {
    type ColorType: Color;
    fn width(&self) -> f64;
    fn height(&self) -> f64;
    fn pixel(&self, x: i32, y: i32) -> Self::ColorType;
}

pub trait ImagePattern {
    type ColorType: Color;

    fn pattern_width(&self) -> i32;
    fn line_width(&self) -> i32;
    fn pixel(&self, p: &mut Self::ColorType, x: i32, y: i32);
    fn create<Src: Pixel<ColorType = Self::ColorType>>(&mut self, src: &Src);
}

pub trait AggPrimitive:
    Copy
    + Default
    + PartialOrd
    + PartialEq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + Default
{
    fn into_u32(&self) -> u32;
    fn into_i32(&self) -> i32;
    fn into_f64(&self) -> f64;
	fn into_u64(&self) -> u64;
    fn into_u8(&self) -> u8;
    fn from_i32(i: i32) -> Self;
    fn from_u32(i: u32) -> Self;
    fn from_u8(i: u8) -> Self;
    fn from_clong(i: c_long) -> Self;
    fn from_f64(i: f64) -> Self;
	fn from_u64(i: u64) -> Self;
    fn wrapping_mul(&self, a: Self) -> Self;
    fn wrapping_add(&self, a: Self) -> Self;
}

impl AggPrimitive for i8 {
    fn into_u32(&self) -> u32 {
        *self as u32
    }
    fn into_u8(&self) -> u8 {
        *self as u8
    }
    fn into_f64(&self) -> f64 {
        *self as f64
    }
    fn from_i32(i: i32) -> Self {
        i as Self
    }
    fn from_u32(i: u32) -> Self {
        i as Self
    }
    fn from_u8(i: u8) -> Self {
        i as Self
    }
    fn from_clong(i: c_long) -> Self {
        i as Self
    }
    fn into_i32(&self) -> i32 {
        *self as i32
    }
    fn from_f64(i: f64) -> Self {
        i as Self
    }

	fn into_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(i: u64) -> Self {
        i as Self
    }

    fn wrapping_mul(&self, a: Self) -> Self {
        (*self as i8).wrapping_mul(a)
    }
    fn wrapping_add(&self, a: Self) -> Self {
        (*self as i8).wrapping_add(a)
    }
}

impl AggPrimitive for u8 {
    fn into_u32(&self) -> u32 {
        *self as u32
    }
    fn into_u8(&self) -> u8 {
        *self as u8
    }
    fn into_f64(&self) -> f64 {
        *self as f64
    }
    fn from_i32(i: i32) -> Self {
        i as Self
    }
    fn from_u32(i: u32) -> Self {
        i as Self
    }
    fn from_u8(i: u8) -> Self {
        i as Self
    }
    fn from_clong(i: c_long) -> Self {
        i as Self
    }
    fn into_i32(&self) -> i32 {
        *self as i32
    }
    fn from_f64(i: f64) -> Self {
        i as Self
    }
	fn into_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(i: u64) -> Self {
        i as Self
    }
    fn wrapping_mul(&self, a: Self) -> Self {
        (*self as u8).wrapping_mul(a)
    }
    fn wrapping_add(&self, a: Self) -> Self {
        (*self as u8).wrapping_add(a)
    }
}

impl AggPrimitive for i16 {
    fn into_u32(&self) -> u32 {
        *self as u32
    }
    fn into_u8(&self) -> u8 {
        *self as u8
    }
    fn from_i32(i: i32) -> Self {
        i as Self
    }
    fn from_u32(i: u32) -> Self {
        i as Self
    }
    fn from_u8(i: u8) -> Self {
        i as Self
    }
    fn from_clong(i: c_long) -> Self {
        i as Self
    }
    fn into_f64(&self) -> f64 {
        *self as f64
    }
    fn into_i32(&self) -> i32 {
        *self as i32
    }
    fn from_f64(i: f64) -> Self {
        i as Self
    }
	fn into_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(i: u64) -> Self {
        i as Self
    }
    fn wrapping_mul(&self, a: Self) -> Self {
        (*self as i16).wrapping_mul(a)
    }
    fn wrapping_add(&self, a: Self) -> Self {
        (*self as i16).wrapping_add(a)
    }
}

impl AggPrimitive for i32 {
    fn into_u32(&self) -> u32 {
        *self as u32
    }
    fn into_u8(&self) -> u8 {
        *self as u8
    }
    fn from_i32(i: i32) -> Self {
        i as Self
    }
    fn from_u32(i: u32) -> Self {
        i as Self
    }
    fn from_u8(i: u8) -> Self {
        i as Self
    }
    fn from_clong(i: c_long) -> Self {
        i as Self
    }
    fn into_f64(&self) -> f64 {
        *self as f64
    }
    fn into_i32(&self) -> i32 {
        *self as i32
    }
    fn from_f64(i: f64) -> Self {
        i as Self
    }
	fn into_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(i: u64) -> Self {
        i as Self
    }
    fn wrapping_mul(&self, a: Self) -> Self {
        (*self as i32).wrapping_mul(a)
    }
    fn wrapping_add(&self, a: Self) -> Self {
        (*self as i32).wrapping_add(a)
    }
}

impl AggPrimitive for u16 {
    fn into_u32(&self) -> u32 {
        *self as u32
    }
    fn into_u8(&self) -> u8 {
        *self as u8
    }
    fn from_i32(i: i32) -> Self {
        i as Self
    }
    fn from_u32(i: u32) -> Self {
        i as Self
    }
    fn from_u8(i: u8) -> Self {
        i as Self
    }
    fn from_clong(i: c_long) -> Self {
        i as Self
    }
    fn into_f64(&self) -> f64 {
        *self as f64
    }
    fn into_i32(&self) -> i32 {
        *self as i32
    }
    fn from_f64(i: f64) -> Self {
        i as Self
    }
	fn into_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(i: u64) -> Self {
        i as Self
    }
    fn wrapping_mul(&self, a: Self) -> Self {
        (*self as u16).wrapping_mul(a)
    }
    fn wrapping_add(&self, a: Self) -> Self {
        (*self as u16).wrapping_add(a)
    }
}

impl AggPrimitive for u32 {
    fn into_u32(&self) -> u32 {
        *self
    }
    fn into_u8(&self) -> u8 {
        *self as u8
    }
    fn from_i32(i: i32) -> Self {
        i as Self
    }
    fn from_u32(i: u32) -> Self {
        i as Self
    }
    fn from_u8(i: u8) -> Self {
        i as Self
    }
    fn from_clong(i: c_long) -> Self {
        i as Self
    }
    fn into_f64(&self) -> f64 {
        *self as f64
    }
    fn into_i32(&self) -> i32 {
        *self as i32
    }
    fn from_f64(i: f64) -> Self {
        i as Self
    }
	fn into_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(i: u64) -> Self {
        i as Self
    }
    fn wrapping_mul(&self, a: Self) -> Self {
        (*self as u32).wrapping_mul(a)
    }
    fn wrapping_add(&self, a: Self) -> Self {
        (*self as u32).wrapping_add(a)
    }
}

impl AggPrimitive for u64 {
    fn into_u32(&self) -> u32 {
        *self as u32
    }
    fn into_u8(&self) -> u8 {
        *self as u8
    }
    fn from_i32(i: i32) -> Self {
        i as Self
    }
    fn from_u32(i: u32) -> Self {
        i as Self
    }
    fn from_u8(i: u8) -> Self {
        i as Self
    }
    fn from_clong(i: c_long) -> Self {
        i as Self
    }
    fn into_i32(&self) -> i32 {
        *self as i32
    }
	fn from_f64(i: f64) -> Self {
        i as Self
    }
	fn into_f64(&self) -> f64 {
        *self as f64
    }
    fn into_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(i: u64) -> Self {
        i as Self
    }
    fn wrapping_mul(&self, a: Self) -> Self {
        self * a
    }
    fn wrapping_add(&self, a: Self) -> Self {
        self + a
    }
}

impl AggPrimitive for f64 {
    fn into_u32(&self) -> u32 {
        *self as u32
    }
    fn into_u8(&self) -> u8 {
        *self as u8
    }
    fn from_i32(i: i32) -> Self {
        i as Self
    }
    fn from_u32(i: u32) -> Self {
        i as Self
    }
    fn from_u8(i: u8) -> Self {
        i as Self
    }
    fn from_clong(i: c_long) -> Self {
        i as Self
    }
    fn into_f64(&self) -> f64 {
        *self as f64
    }
    fn into_i32(&self) -> i32 {
        *self as i32
    }
    fn from_f64(i: f64) -> Self {
        i as Self
    }
	fn into_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(i: u64) -> Self {
        i as Self
    }

    fn wrapping_mul(&self, a: Self) -> Self {
        self * a
    }
    fn wrapping_add(&self, a: Self) -> Self {
        self + a
    }
}

pub trait AggInteger:
    AggPrimitive + Shl<Output = Self> + Shr<Output = Self> + Not<Output = Self> + BitAnd<Output = Self>
{
}

impl AggInteger for i8 {}
impl AggInteger for u8 {}
impl AggInteger for i16 {}
impl AggInteger for i32 {}
impl AggInteger for u16 {}
impl AggInteger for u32 {}
impl AggInteger for u64 {}

pub enum Equiv<'a, B: Sized> {
    Own(B),
    Brw(&'a mut B),
}

impl<'a, B: Sized> Deref for Equiv<'a, B> {
    type Target = B;
    fn deref(&self) -> &B {
        match self {
            Equiv::Brw(borrowed) => borrowed,
            Equiv::Own(ref owned) => owned,
        }
    }
}

impl<'a, B: Sized> DerefMut for Equiv<'a, B> {
    fn deref_mut(&mut self) -> &mut B {
        match self {
            Equiv::Brw(borrowed) => borrowed,
            Equiv::Own(ref mut owned) => owned,
        }
    }
}

macro_rules! slice_t_to_vt {
    ($arr: expr, $off: expr, $totype: ty) => {
        unsafe {
            std::slice::from_raw_parts(
                ($arr.as_ptr() as *const $totype).offset($off as isize),
                $arr.len() / std::mem::size_of::<$totype>(),
            )
        }
    };
}

macro_rules! slice_t_to_vt_mut {
    ($arr: expr, $off: expr, $totype: ty) => {
        unsafe {
            std::slice::from_raw_parts_mut(
                ($arr.as_mut_ptr() as *mut $totype).offset($off as isize),
                $arr.len() / std::mem::size_of::<$totype>(),
            )
        }
    };
}
pub(crate) use {slice_t_to_vt, slice_t_to_vt_mut};
