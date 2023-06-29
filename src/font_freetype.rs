use freetype_sys::{
    FT_Attach_File, FT_Bitmap, FT_Done_Face, FT_Done_FreeType, FT_Encoding, FT_Face,
    FT_Get_Char_Index, FT_Get_Kerning, FT_Init_FreeType, FT_Library, FT_Load_Glyph, FT_New_Face,
    FT_New_Memory_Face, FT_Outline, FT_Render_Glyph, FT_Select_Charmap, FT_Set_Char_Size,
    FT_Set_Pixel_Sizes, FT_Vector, FT_ENCODING_NONE, FT_HAS_KERNING, FT_IS_SCALABLE,
    FT_KERNING_DEFAULT, FT_LOAD_DEFAULT, FT_LOAD_NO_HINTING, FT_RENDER_MODE_MONO,
    FT_RENDER_MODE_NORMAL,
};
use std::marker::PhantomData;
use std::ops::Neg;
use std::os::raw::{c_long, c_ulong};
use std::ptr::null_mut;

use crate::basics::{CoverScale, RectI};
use crate::bitset_iterator::BitsetIterator;
use crate::conv_curve::ConvCurve;
use crate::font_cache_manager::{GlyphDataType, GlyphRender};
use crate::path_storage_integer::PathStorageInteger;
use crate::rasterizer_scanline_aa::{AaScale, RasterizerScanlineAa};
use crate::renderer_scanline::render_scanlines;
use crate::scanline_bin::ScanlineBin;
use crate::scanline_storage_aa::ScanlineStorageAA;
use crate::scanline_storage_bin::ScanlineStorageBin;
use crate::scanline_u::ScanlineU8;
use crate::trans_affine::TransAffine;
use crate::{
    AggInteger, AggPrimitive, GammaFn, PathStore, RasterScanLine, RendererScanline, Scanline,
    Transformer,
};
use crate::{FontEngine, FontEngineBase};

//------------------------------------------------font_engine_freetype_int32
// This class uses values of type int32 (26.6 format) for the vector cache.
// The vector cache is twice larger than in font_engine_freetype_int16,
// but it allows you to render glyphs of very large sizes.
impl<'a> FontEngine for FreetypeBase<'a, i32> {
    type PathAdaptorType = crate::SerializedIntegerPathAdaptor<i32>;

    fn new(max_faces: u32) -> Self {
        FreetypeBase::new_with_flags(true, max_faces)
    }

    fn path_adaptor(&self) -> Self::PathAdaptorType {
        Self::PathAdaptorType::new()
    }
}

//------------------------------------------------font_engine_freetype_int16
// This class uses values of type int16 (10.6 format) for the vector cache.
// The vector cache is compact, but when rendering glyphs of height
// more that 200 there integer overflow can occur.
impl<'a> FontEngine for FreetypeBase<'a, i16> {
    type PathAdaptorType = crate::SerializedIntegerPathAdaptor<i16>;

    fn new(max_faces: u32) -> Self {
        FreetypeBase::new_with_flags(false, max_faces)
    }

    fn path_adaptor(&self) -> Self::PathAdaptorType {
        Self::PathAdaptorType::new()
    }
}

//-----------------------------------------------FreetypeBase
pub struct FreetypeBase<'a, T: AggInteger + Neg> {
    m_flag32: bool,
    m_change_stamp: i32,
    m_last_error: i32,
    m_name: String,
    m_name_len: u32,
    m_face_index: u32,
    m_char_map: FT_Encoding,
    m_signature: String,
    m_height: u32,
    m_width: u32,
    m_hinting: bool,
    m_flip_y: bool,
    m_library_initialized: bool,
    m_library: FT_Library,
    m_faces: Vec<FT_Face>,
    m_face_names: Vec<String>,
    m_num_faces: u32,
    m_max_faces: u32,
    m_cur_face: FT_Face,
    m_resolution: u32,
    m_glyph_rendering: GlyphRender,
    m_glyph_index: u32,
    m_data_size: u32,
    m_data_type: GlyphDataType,
    m_bounds: RectI,
    m_advance_x: f64,
    m_advance_y: f64,
    m_affine: TransAffine,
    //m_path16: Rc<RefCell<PathStorageInteger<i16, 6>>>,
    //m_path32: Rc<RefCell<PathStorageInteger<i32, 6>>>,
    m_curves16: ConvCurve<'a,PathStorageInteger<i16, 6>>,
    m_curves32: ConvCurve<'a,PathStorageInteger<i32, 6>>,
    m_scanline_aa: ScanlineU8,
    m_scanline_bin: ScanlineBin,
    m_scanlines_aa: ScanlineStorageAA<i8>,
    m_scanlines_bin: ScanlineStorageBin,
    m_rasterizer: RasterizerScanlineAa,
    m_dum: PhantomData<T>,
}

impl<'a,T: AggInteger + Neg> FontEngineBase for FreetypeBase<'a,T> {
    type Gray8AdaptorType = crate::scanline_storage_aa::SerializedScanlinesAdaptorAa<u8>;
    type Gray8ScanlineType = crate::scanline_storage_aa::EmbeddedScanline;
    type MonoAdaptorType = crate::scanline_storage_bin::SerializedScanlinesAdaptorBin;
    type MonoScanlineType = crate::scanline_storage_bin::EmbeddedScanline;

    fn font_signature(&self) -> &str {
        &self.m_signature
    }
    fn change_stamp(&self) -> i32 {
        self.m_change_stamp
    }
    fn glyph_index(&self) -> u32 {
        self.m_glyph_index
    }
    fn data_size(&self) -> u32 {
        self.m_data_size
    }
    fn data_type(&self) -> GlyphDataType {
        self.m_data_type
    }
    fn bounds(&self) -> &RectI {
        &self.m_bounds
    }
    fn advance_x(&self) -> f64 {
        self.m_advance_x
    }
    fn advance_y(&self) -> f64 {
        self.m_advance_y
    }

    fn gray8_adaptor(&self) -> Self::Gray8AdaptorType {
        Self::Gray8AdaptorType::new()
    }

    fn gray8_scanline(&self) -> Self::Gray8ScanlineType {
        Self::Gray8ScanlineType::new()
    }

    fn mono_adaptor(&self) -> Self::MonoAdaptorType {
        Self::MonoAdaptorType::new()
    }

    fn mono_scanline(&self) -> Self::MonoScanlineType {
        Self::MonoScanlineType::new()
    }

    fn prepare_glyph(&mut self, glyph_code: u32) -> bool {
        self.m_glyph_index = unsafe { FT_Get_Char_Index(self.m_cur_face, glyph_code as c_ulong) };
        self.m_last_error = unsafe {
            FT_Load_Glyph(
                self.m_cur_face,
                self.m_glyph_index,
                if self.m_hinting {
                    FT_LOAD_DEFAULT
                } else {
                    FT_LOAD_NO_HINTING
                },
            )
        };
        if self.m_last_error == 0 {
            match self.m_glyph_rendering {
                GlyphRender::NativeMono => {
                    self.m_last_error =
                        unsafe { FT_Render_Glyph((*self.m_cur_face).glyph, FT_RENDER_MODE_MONO) };
                    if self.m_last_error == 0 {
                        unsafe {
                            decompose_ft_bitmap_mono(
                                &(*(*self.m_cur_face).glyph).bitmap,
                                (*(*self.m_cur_face).glyph).bitmap_left,
                                if self.m_flip_y {
                                    -(*(*self.m_cur_face).glyph).bitmap_top
                                } else {
                                    (*(*self.m_cur_face).glyph).bitmap_top
                                },
                                self.m_flip_y,
                                &mut self.m_scanline_bin,
                                &mut self.m_scanlines_bin,
                            );
                        }
                        self.m_bounds.x1 = self.m_scanlines_bin.min_x();
                        self.m_bounds.y1 = self.m_scanlines_bin.min_y();
                        self.m_bounds.x2 = self.m_scanlines_bin.max_x() + 1;
                        self.m_bounds.y2 = self.m_scanlines_bin.max_y() + 1;
                        self.m_data_size = self.m_scanlines_bin.byte_size() as u32;
                        self.m_data_type = GlyphDataType::Mono;
                        unsafe {
                            self.m_advance_x =
                                int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.x);
                            self.m_advance_y =
                                int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.y);
                        }
                        return true;
                    }
                }
                GlyphRender::NativeGray8 => {
                    self.m_last_error =
                        unsafe { FT_Render_Glyph((*self.m_cur_face).glyph, FT_RENDER_MODE_NORMAL) };
                    if self.m_last_error == 0 {
                        unsafe {
                            decompose_ft_bitmap_gray8(
                                &(*(*self.m_cur_face).glyph).bitmap,
                                (*(*self.m_cur_face).glyph).bitmap_left,
                                if self.m_flip_y {
                                    -(*(*self.m_cur_face).glyph).bitmap_top
                                } else {
                                    (*(*self.m_cur_face).glyph).bitmap_top
                                },
                                self.m_flip_y,
                                &mut self.m_rasterizer,
                                &mut self.m_scanline_aa,
                                &mut self.m_scanlines_aa,
                            );
                        }
                        self.m_bounds.x1 = self.m_scanlines_aa.min_x();
                        self.m_bounds.y1 = self.m_scanlines_aa.min_y();
                        self.m_bounds.x2 = self.m_scanlines_aa.max_x() + 1;
                        self.m_bounds.y2 = self.m_scanlines_aa.max_y() + 1;
                        self.m_data_size = self.m_scanlines_aa.byte_size() as u32;
                        self.m_data_type = GlyphDataType::Gray8;
                        unsafe {
                            self.m_advance_x =
                                int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.x);
                            self.m_advance_y =
                                int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.y);
                        }
                        return true;
                    }
                }
                GlyphRender::Outline => {
                    if self.m_last_error == 0 {
                        if self.m_flag32 {
                            self.m_curves32.source_mut().remove_all();
                            if unsafe {
                                decompose_ft_outline(
                                    &(*(*self.m_cur_face).glyph).outline,
                                    self.m_flip_y,
                                    &self.m_affine,
                                    &mut *self.m_curves32.source_mut(),
                                )
                            } {
                                let bnd = self.m_curves32.source_mut().bounding_rect();
                                self.m_data_size = self.m_curves32.source().byte_size() as u32;
                                self.m_data_type = GlyphDataType::Outline;
                                self.m_bounds.x1 = bnd.x1 as i32;
                                self.m_bounds.y1 = bnd.y1 as i32;
                                self.m_bounds.x2 = bnd.x2 as i32;
                                self.m_bounds.y2 = bnd.y2 as i32;
                                unsafe {
                                    self.m_advance_x =
                                        int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.x);
                                    self.m_advance_y =
                                        int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.y);
                                }
                                self.m_affine
                                    .transform(&mut self.m_advance_x, &mut self.m_advance_y);
                                return true;
                            }
                        } else {
                            self.m_curves16.source_mut().remove_all();
                            if unsafe {
                                decompose_ft_outline(
                                    &(*(*self.m_cur_face).glyph).outline,
                                    self.m_flip_y,
                                    &self.m_affine,
                                    &mut *self.m_curves16.source_mut(),
                                )
                            } {
                                let bnd = self.m_curves16.source_mut().bounding_rect();
                                self.m_data_size = self.m_curves16.source().byte_size() as u32;
                                self.m_data_type = GlyphDataType::Outline;
                                self.m_bounds.x1 = bnd.x1 as i32;
                                self.m_bounds.y1 = bnd.y1 as i32;
                                self.m_bounds.x2 = bnd.x2 as i32;
                                self.m_bounds.y2 = bnd.y2 as i32;
                                unsafe {
                                    self.m_advance_x =
                                        int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.x);
                                    self.m_advance_y =
                                        int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.y);
                                }
                                self.m_affine
                                    .transform(&mut self.m_advance_x, &mut self.m_advance_y);
                                return true;
                            }
                        }
                    }
                    return false;
                }
                GlyphRender::AggMono => {
                    if self.m_last_error == 0 {
                        self.m_rasterizer.reset();
                        if self.m_flag32 {
                            self.m_curves32.source_mut().remove_all();
                            unsafe {
                                decompose_ft_outline(
                                    &(*(*self.m_cur_face).glyph).outline,
                                    self.m_flip_y,
                                    &self.m_affine,
                                    &mut *self.m_curves32.source_mut(),
                                );
                            }
                            self.m_rasterizer.add_path(&mut self.m_curves32, 0);
                        } else {
                            self.m_curves16.source_mut().remove_all();
                            unsafe {
                                decompose_ft_outline(
                                    &(*(*self.m_cur_face).glyph).outline,
                                    self.m_flip_y,
                                    &self.m_affine,
                                    &mut *self.m_curves16.source_mut(),
                                );
                            }
                            self.m_rasterizer.add_path(&mut self.m_curves16, 0);
                        }
                        self.m_scanlines_bin.prepare(); // Remove all
                        render_scanlines(
                            &mut self.m_rasterizer,
                            &mut self.m_scanline_bin,
                            &mut self.m_scanlines_bin,
                        );
                        self.m_bounds.x1 = self.m_scanlines_bin.min_x();
                        self.m_bounds.y1 = self.m_scanlines_bin.min_y();
                        self.m_bounds.x2 = self.m_scanlines_bin.max_x() + 1;
                        self.m_bounds.y2 = self.m_scanlines_bin.max_y() + 1;
                        self.m_data_size = self.m_scanlines_bin.byte_size() as u32;
                        self.m_data_type = GlyphDataType::Mono;
                        unsafe {
                            self.m_advance_x =
                                int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.x);
                            self.m_advance_y =
                                int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.y);
                        }

                        self.m_affine
                            .transform(&mut self.m_advance_x, &mut self.m_advance_y);
                        return true;
                    }
                    return false;
                }
                GlyphRender::AggGray8 => {
                    if self.m_last_error == 0 {
                        self.m_rasterizer.reset();
                        if self.m_flag32 {
                            self.m_curves32.source_mut().remove_all();
                            unsafe {
                                decompose_ft_outline(
                                    &(*(*self.m_cur_face).glyph).outline,
                                    self.m_flip_y,
                                    &self.m_affine,
                                    &mut *self.m_curves32.source_mut(),
                                );
                            }
                            self.m_rasterizer.add_path(&mut self.m_curves32, 0);
                        } else {
                            self.m_curves16.source_mut().remove_all();
                            unsafe {
                                decompose_ft_outline(
                                    &(*(*self.m_cur_face).glyph).outline,
                                    self.m_flip_y,
                                    &self.m_affine,
                                    &mut *self.m_curves16.source_mut(),
                                );
                            }
                            self.m_rasterizer.add_path(&mut self.m_curves16, 0);
                        }
                        self.m_scanlines_aa.prepare(); // Remove all
                        render_scanlines(
                            &mut self.m_rasterizer,
                            &mut self.m_scanline_aa,
                            &mut self.m_scanlines_aa,
                        );
                        self.m_bounds.x1 = self.m_scanlines_aa.min_x();
                        self.m_bounds.y1 = self.m_scanlines_aa.min_y();
                        self.m_bounds.x2 = self.m_scanlines_aa.max_x() + 1;
                        self.m_bounds.y2 = self.m_scanlines_aa.max_y() + 1;
                        self.m_data_size = self.m_scanlines_aa.byte_size() as u32;
                        self.m_data_type = GlyphDataType::Gray8;
                        unsafe {
                            self.m_advance_x =
                                int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.x);
                            self.m_advance_y =
                                int26p6_to_dbl((*(*self.m_cur_face).glyph).advance.y);
                        }

                        self.m_affine
                            .transform(&mut self.m_advance_x, &mut self.m_advance_y);
                        return true;
                    }
                    return false;
                }
            }
        }
        return false;
    }

    fn write_glyph_to(&mut self, data: &mut [u8]) {
        if data.len() == 0 {
            return;
        }
        match self.m_data_type {
            GlyphDataType::Mono => self.m_scanlines_bin.serialize(data),
            GlyphDataType::Gray8 => self.m_scanlines_aa.serialize(data),
            GlyphDataType::Outline => {
                if self.m_flag32 {
                    self.m_curves32.source_mut().serialize(data);
                } else {
                    self.m_curves16.source_mut().serialize(data);
                }
            }
            GlyphDataType::Invalid => (),
        }
    }

    fn add_kerning(&self, first: u32, second: u32, x: &mut f64, y: &mut f64) -> bool {
        if !self.m_cur_face.is_null() {
            if first != 0 && second != 0 && FT_HAS_KERNING(self.m_cur_face) != false {
                let mut delta = FT_Vector { x: 0, y: 0 };
                let err = unsafe {
                    FT_Get_Kerning(
                        self.m_cur_face,
                        first,
                        second,
                        FT_KERNING_DEFAULT,
                        &mut delta,
                    )
                };
                if err != 0 {
                    panic!("FT_Get_Kerning failed: {}", err);
                }
                let mut dx = int26p6_to_dbl(delta.x);
                let mut dy = int26p6_to_dbl(delta.y);
                if self.m_glyph_rendering == GlyphRender::Outline
                    || self.m_glyph_rendering == GlyphRender::AggMono
                    || self.m_glyph_rendering == GlyphRender::AggGray8
                {
                    self.m_affine.transform_2x2(&mut dx, &mut dy);
                }
                *x += dx;
                *y += dy;
                return true;
            }
        }
        false
    }
}

impl<'a,T: AggInteger + Neg> FreetypeBase<'a,T> {
    pub fn new_with_flags(flag32: bool, max_faces: u32) -> Self {
        let pi16 = PathStorageInteger::<i16, 6>::new();
        let pi32 = PathStorageInteger::<i32, 6>::new();
        let mut f = FreetypeBase {
            m_flag32: flag32,
            m_change_stamp: 0,
            m_last_error: 0,
            m_name: "".to_string(),
            m_name_len: 256 - 16 - 1,
            m_face_index: 0,
            m_char_map: FT_ENCODING_NONE,
            m_signature: "".to_string(),
            m_height: 0,
            m_width: 0,
            m_hinting: true,
            m_flip_y: false,
            m_library_initialized: false,
            m_library: 0 as FT_Library,
            m_faces: vec![null_mut(); max_faces as usize],
            m_face_names: vec!["".to_string(); max_faces as usize],
            m_num_faces: 0,
            m_max_faces: max_faces,
            m_cur_face: 0 as FT_Face,
            m_resolution: 0,
            m_glyph_rendering: GlyphRender::NativeGray8,
            m_glyph_index: 0,
            m_data_size: 0,
            m_data_type: GlyphDataType::Invalid,
            m_bounds: RectI {
                x1: 1,
                y1: 1,
                x2: 0,
                y2: 0,
            },
            m_advance_x: 0.0,
            m_advance_y: 0.0,
            m_affine: TransAffine::new_default(),
            //m_path16: pi16.clone(),
            //m_path32: pi32.clone(),
            m_curves16: ConvCurve::<PathStorageInteger<i16, 6>>::new_owned(pi16),
            m_curves32: ConvCurve::<PathStorageInteger<i32, 6>>::new_owned(pi32),
            m_scanline_aa: ScanlineU8::new(),
            m_scanline_bin: ScanlineBin::new(),
            m_scanlines_aa: ScanlineStorageAA::new(),
            m_scanlines_bin: ScanlineStorageBin::new(),
            m_rasterizer: RasterizerScanlineAa::new(),
            m_dum: PhantomData,
        };
        f.m_curves16.set_approximation_scale(4.0);
        f.m_curves32.set_approximation_scale(4.0);
        f.m_last_error = unsafe { FT_Init_FreeType(&mut f.m_library as *mut FT_Library) };
        if f.m_last_error == 0 {
            f.m_library_initialized = true;
        }
        f
    }
    pub fn last_error(&self) -> i32 {
        self.m_last_error
    }
    pub fn resolution(&self) -> u32 {
        self.m_resolution
    }
    pub fn name(&self) -> &str {
        &self.m_name
    }
    pub fn char_map(&self) -> FT_Encoding {
        self.m_char_map
    }
    pub fn height(&self) -> f64 {
        (self.m_height as f64) / 64.0
    }
    pub fn width(&self) -> f64 {
        (self.m_width as f64) / 64.0
    }
    pub fn hinting(&self) -> bool {
        self.m_hinting
    }
    pub fn flip_y(&self) -> bool {
        self.m_flip_y
    }

    pub fn set_gamma<F: GammaFn>(&mut self, gamma_function: F) {
        self.m_rasterizer.set_gamma(&gamma_function)
    }

    pub fn set_resolution(&mut self, dpi: u32) {
        self.m_resolution = dpi;
        self.update_char_size();
    }

    pub fn find_face(&self, face_name: &str) -> i32 {
        for i in 0..self.m_num_faces {
            if face_name == self.m_face_names[i as usize] {
                return i as i32;
            }
        }
        return -1;
    }

    pub fn ascender(&self) -> f64 {
        if !self.m_cur_face.is_null() {
            return unsafe {
                (*self.m_cur_face).ascender as f64 * self.height()
                    / (*self.m_cur_face).height as f64
            };
        }
        return 0.0;
    }

    pub fn descender(&self) -> f64 {
        if !self.m_cur_face.is_null() {
            return unsafe {
                (*self.m_cur_face).descender as f64 * self.height()
                    / (*self.m_cur_face).height as f64
            };
        }
        return 0.0;
    }

    pub fn load_font(
        &mut self, font_name: &str, face_index: u32, ren_type: GlyphRender, font_mem: &[u8],
        font_mem_size: u32,
    ) -> bool {
        let mut ret = false;
        //let mut f: FT_Face = null_mut();

        if self.m_library_initialized {
            self.m_last_error = 0;

            let idx = self.find_face(font_name);
            if idx >= 0 {
                self.m_cur_face = self.m_faces[idx as usize];
                self.m_name = self.m_face_names[idx as usize].clone();
            } else {
                if self.m_num_faces >= self.m_max_faces {
                    self.m_face_names[0] = "".to_string();
                    unsafe { FT_Done_Face(self.m_faces[0]) };
                    self.m_faces[0] = null_mut();
                    for i in 1..self.m_max_faces {
                        self.m_faces[i as usize - 1] = self.m_faces[i as usize];
                        self.m_face_names[i as usize - 1] = self.m_face_names[i as usize].clone();
                    }
                    self.m_num_faces = self.m_max_faces - 1;
                }

                if font_mem.len() > 0 && font_mem_size > 0 {
                    self.m_last_error = unsafe {
                        FT_New_Memory_Face(
                            self.m_library,
                            font_mem.as_ptr(),
                            font_mem_size as c_long,
                            face_index as c_long,
                            &mut self.m_faces[self.m_num_faces as usize],
                        )
                    };
                } else {
                    self.m_last_error = unsafe {
                        FT_New_Face(
                            self.m_library,
                            font_name.as_ptr() as *const i8,
                            face_index as std::os::raw::c_long,
                            &mut self.m_faces[self.m_num_faces as usize],
                        )
                    };
                }

                if self.m_last_error == 0 {
                    self.m_face_names[self.m_num_faces as usize] = font_name.to_string();
                    self.m_cur_face = self.m_faces[self.m_num_faces as usize];
                    self.m_name = self.m_face_names[self.m_num_faces as usize].clone();
                    self.m_num_faces += 1;
                } else {
                    self.m_face_names[self.m_num_faces as usize] = "".to_string();
                    self.m_cur_face = null_mut();
                    self.m_name = "".to_string();
                }
            }

            if self.m_last_error == 0 {
                ret = true;
                match ren_type {
                    GlyphRender::NativeMono => self.m_glyph_rendering = GlyphRender::NativeMono,
                    GlyphRender::NativeGray8 => self.m_glyph_rendering = GlyphRender::NativeGray8,
                    GlyphRender::Outline => {
                        if FT_IS_SCALABLE(self.m_cur_face) {
                            self.m_glyph_rendering = GlyphRender::Outline;
                        } else {
                            self.m_glyph_rendering = GlyphRender::NativeGray8;
                        }
                    }
                    GlyphRender::AggMono => {
                        if FT_IS_SCALABLE(self.m_cur_face) {
                            self.m_glyph_rendering = GlyphRender::AggMono;
                        } else {
                            self.m_glyph_rendering = GlyphRender::NativeMono;
                        }
                    }
                    GlyphRender::AggGray8 => {
                        if FT_IS_SCALABLE(self.m_cur_face) {
                            self.m_glyph_rendering = GlyphRender::AggGray8;
                        } else {
                            self.m_glyph_rendering = GlyphRender::NativeGray8;
                        }
                    }
                }
                self.update_signature();
            }
        }
        return ret;
    }

    pub fn attach(&mut self, file_name: &str) -> bool {
        if !self.m_cur_face.is_null() {
            self.m_last_error =
                unsafe { FT_Attach_File(self.m_cur_face, file_name.as_ptr() as *const i8) };
            return self.m_last_error == 0;
        }
        return false;
    }

    pub fn num_faces(&self) -> u32 {
        if !self.m_cur_face.is_null() {
            return unsafe { (*self.m_cur_face).num_faces as u32 };
        }
        return 0;
    }

    pub fn set_char_map(&mut self, char_map: FT_Encoding) -> bool {
        if !self.m_cur_face.is_null() {
            self.m_last_error = unsafe {
                FT_Select_Charmap(self.m_cur_face, char_map /*self.m_char_map*/)
            }; // XXXX
            if self.m_last_error == 0 {
                self.update_signature();
                return true;
            }
        }
        return false;
    }

    pub fn set_height(&mut self, h: f64) -> bool {
        self.m_height = (h * 64.0) as u32;
        if !self.m_cur_face.is_null() {
            self.update_char_size();
            return true;
        }
        return false;
    }

    pub fn set_width(&mut self, w: f64) -> bool {
        self.m_width = (w * 64.0) as u32;
        if !self.m_cur_face.is_null() {
            self.update_char_size();
            return true;
        }
        return false;
    }

    pub fn set_hinting(&mut self, h: bool) {
        self.m_hinting = h;
        if !self.m_cur_face.is_null() {
            self.update_signature();
        }
    }

    pub fn set_flip_y(&mut self, f: bool) {
        self.m_flip_y = f;
        if !self.m_cur_face.is_null() {
            self.update_signature();
        }
    }

    pub fn transform(&mut self, affine: &TransAffine) {
        self.m_affine = *affine;
        if !self.m_cur_face.is_null() {
            self.update_signature();
        }
    }

    pub fn update_signature(&mut self) {
        if !self.m_cur_face.is_null() && self.m_name != "" {
            let name_len = self.m_name.len() as u32;
            if name_len > self.m_name_len {
                //self.m_signature = String::with_capacity(name_len + 32 + 256);
                self.m_name_len = name_len + 32 - 1;
            }

            let mut gamma_hash = 0;
            if self.m_glyph_rendering == GlyphRender::NativeGray8
                || self.m_glyph_rendering == GlyphRender::AggMono
                || self.m_glyph_rendering == GlyphRender::AggGray8
            {
                let mut gamma_table: [u8; AaScale::Scale as usize] = [0; AaScale::Scale as usize];

                for i in 0..AaScale::Scale as usize {
                    gamma_table[i as usize] = self.m_rasterizer.apply_gamma(i) as u8;
                }
                gamma_hash = calc_crc32(&gamma_table, gamma_table.len());
            }

            self.m_signature = format!(
                "{},{},{},{},{}:{}x{},{},{},{:08X}",
                self.m_name,
                self.m_char_map,
                self.m_face_index,
                self.m_glyph_rendering as i32,
                self.m_resolution,
                self.m_height,
                self.m_width,
                self.m_hinting as i32,
                self.m_flip_y as i32,
                gamma_hash
            );
            if self.m_glyph_rendering == GlyphRender::Outline
                || self.m_glyph_rendering == GlyphRender::AggMono
                || self.m_glyph_rendering == GlyphRender::AggGray8
            {
                let mut mtx: [f64; 6] = [0.0; 6];
                let buf; //: String = String::with_capacity(100);
                self.m_affine.store_to(&mut mtx);
                buf = format!(
                    ",{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}",
                    dbl_to_plain_fx(mtx[0]),
                    dbl_to_plain_fx(mtx[1]),
                    dbl_to_plain_fx(mtx[2]),
                    dbl_to_plain_fx(mtx[3]),
                    dbl_to_plain_fx(mtx[4]),
                    dbl_to_plain_fx(mtx[5])
                );
                self.m_signature.push_str(&buf);
            }
            self.m_change_stamp += 1;
        }
    }

    fn update_char_size(&mut self) {
        if !self.m_cur_face.is_null() {
            if self.m_resolution != 0 {
                let err = unsafe {
                    FT_Set_Char_Size(
                        self.m_cur_face,
                        self.m_width as c_long,
                        self.m_height as c_long,
                        self.m_resolution,
                        self.m_resolution,
                    )
                };
                if err != 0 {
                    panic!("FT_Set_Char_Size failed: {}", err);
                }
            } else {
                let err = unsafe {
                    FT_Set_Pixel_Sizes(self.m_cur_face, self.m_width >> 6, self.m_height >> 6)
                };
                if err != 0 {
                    panic!("FT_Set_Pixel_Sizes failed: {}", err);
                }
            }
            self.update_signature();
        }
    }
}

impl<'a,T: AggInteger + Neg> Drop for FreetypeBase<'a,T> {
    fn drop(&mut self) {
        unsafe {
            if self.m_library_initialized {
                FT_Done_FreeType(self.m_library);
            }
        }
    }
}

fn calc_crc32(buf: &[u8], size: usize) -> u32 {
    let mut crc: u32 = !0;
    //let mut len: usize = 0;
    let nr: usize = size;
    //let mut p: &u8 = buf;
    let mut i = 0;
    for _ in 0..nr {
        crc = (crc >> 8) ^ CRC32_TAB[(crc ^ buf[i] as u32) as usize & 0xff];
        i += 1
    }
    !crc
}

fn dbl_to_plain_fx(d: f64) -> i32 {
    (d * 65536.0) as i32
}

fn int26p6_to_dbl(p: c_long) -> f64 {
    p as f64 / 64.0
}

fn dbl_to_int26p6(p: f64) -> c_long {
    (p * 64.0 + 0.5) as c_long
}

fn decompose_ft_bitmap_mono<Sl: Scanline, SlS: RendererScanline>(
    bitmap: &FT_Bitmap, x: i32, y: i32, flip_y: bool, sl: &mut Sl, storage: &mut SlS,
) {
    //let mut i: i32;
    let mut buf = bitmap.buffer;
    let mut pitch: i32 = bitmap.pitch;
    let mut y = y;

    sl.reset(x, x + bitmap.width);
    storage.prepare();
    if flip_y {
        unsafe {
            buf = buf.offset((bitmap.pitch * (bitmap.rows - 1)) as isize);
        }
        pitch = -pitch;
        y += bitmap.rows;
    }
    for i in 0..bitmap.rows {
        sl.reset_spans();
        let mut bits: BitsetIterator = BitsetIterator::new(buf, 0);
        //let mut j: i32;
        for j in 0..bitmap.width {
            if bits.bit() != 0 {
                sl.add_cell(x + j, CoverScale::FULL as u32);
            }
            bits.inc();
        }
        buf = unsafe { buf.offset(pitch as isize) };
        if sl.num_spans() > 0 {
            sl.finalize(y - i - 1);
            storage.render(sl);
        }
    }
}

fn decompose_ft_bitmap_gray8<Ras: RasterScanLine, Sl: Scanline, SlS: RendererScanline>(
    bitmap: &FT_Bitmap, x: i32, y: i32, flip_y: bool, ras: &mut Ras, sl: &mut Sl, storage: &mut SlS,
) {
    //let mut i: i32;
    let mut buf = bitmap.buffer;
    let mut pitch: i32 = bitmap.pitch;
    let mut y = y;

    sl.reset(x, x + bitmap.width);
    storage.prepare();
    if flip_y {
        unsafe {
            buf = buf.offset((bitmap.pitch * (bitmap.rows - 1)) as isize);
        }
        pitch = -pitch;
        y += bitmap.rows;
    }
    for i in 0..bitmap.rows {
        sl.reset_spans();
        let mut p = buf;
        //let mut j: i32;
        for j in 0..bitmap.width {
            if !p.is_null() {
                sl.add_cell(x + j, unsafe { ras.apply_gamma(*p as usize) });
            }
            unsafe {
                p = p.offset(1);
            }
        }
        buf = unsafe { buf.offset(pitch as isize) };
        if sl.num_spans() > 0 {
            sl.finalize(y - i - 1);
            storage.render(sl);
        }
    }
}
pub const FT_CURVE_TAG_ON: u8 = 1;
pub const FT_CURVE_TAG_CONIC: u8 = 0;
pub const FT_CURVE_TAG_CUBIC: u8 = 2;

macro_rules! FT_CURVE_TAG {
    ($flags:expr) => {
        $flags as u8 & 3
    };
}
macro_rules! from_i32 {
    ($v:expr) => {
        <<PS as PathStore>::T as AggPrimitive>::from_clong($v)
    };
}
fn decompose_ft_outline<PS: PathStore>(
    outline: &FT_Outline, flip_y: bool, mtx: &TransAffine, path: &mut PS,
) -> bool {
    let mut v_last; //: FT_Vector = FT_Vector { x: 0, y: 0 };
    let mut v_control; //: FT_Vector = FT_Vector { x: 0, y: 0 };
    let mut v_start; //: FT_Vector = FT_Vector { x: 0, y: 0 };
    let mut x1: f64;
    let mut y1: f64;
    let mut x2: f64;
    let mut y2: f64;
    let mut x3: f64;
    let mut y3: f64;

    let mut point: *const FT_Vector;
    let mut limit: *const FT_Vector;
    let mut tags;

    //let mut n: i32;
    let mut first: i32;
    let mut tag: u8;
    let mut go = 0;
    first = 0;

    for n in 0..outline.n_contours {
        let last: i32;
        unsafe {
            last = *outline.contours.offset(n as isize) as i32;
            limit = outline.points.offset(last as isize);

            v_start = *outline.points.offset(first as isize);
            v_last = *outline.points.offset(last as isize);

            v_control = v_start;

            point = outline.points.offset(first as isize);
            tags = outline.tags.offset(first as isize);
            tag = FT_CURVE_TAG!(*tags);
        }
        if tag == FT_CURVE_TAG_CUBIC {
            return false;
        }

        if tag == FT_CURVE_TAG_CONIC {
            unsafe {
                if FT_CURVE_TAG!(*outline.tags.offset(last as isize)) == FT_CURVE_TAG_ON {
                    v_start = v_last;
                    limit = limit.offset(-1);
                } else {
                    v_start.x = (v_start.x + v_last.x) / 2;
                    v_start.y = (v_start.y + v_last.y) / 2;

                    //v_last = v_start; XXX
                }
                point = point.offset(-1);
                tags = tags.offset(-1);
            }
        }

        x1 = int26p6_to_dbl(v_start.x);
        y1 = int26p6_to_dbl(v_start.y);
        if flip_y {
            y1 = -y1;
        }
        mtx.transform(&mut x1, &mut y1);
        path.move_to(from_i32!(dbl_to_int26p6(x1)), from_i32!(dbl_to_int26p6(y1)));

        'wouter: while point < limit {
            unsafe {
                point = point.offset(1);
                tags = tags.offset(1);

                tag = FT_CURVE_TAG!(*tags);
            }
            match tag {
                FT_CURVE_TAG_ON => {
                    unsafe {
                        x1 = int26p6_to_dbl((*point).x);
                        y1 = int26p6_to_dbl((*point).y);
                    }
                    if flip_y {
                        y1 = -y1;
                    }
                    mtx.transform(&mut x1, &mut y1);
                    path.line_to(from_i32!(dbl_to_int26p6(x1)), from_i32!(dbl_to_int26p6(y1)));
                }
                FT_CURVE_TAG_CONIC => {
                    // consume conic arcs
                    unsafe {
                        v_control.x = (*point).x;
                        v_control.y = (*point).y;
                    }
                    while point < limit {
                        let mut vec: FT_Vector = FT_Vector { x: 0, y: 0 };
                        let mut v_middle: FT_Vector = FT_Vector { x: 0, y: 0 };
                        unsafe {
                            point = point.offset(1);
                            tags = tags.offset(1);
                            tag = FT_CURVE_TAG!(*tags.offset(0));

                            vec.x = (*point).x;
                            vec.y = (*point).y;
                        }
                        if tag == FT_CURVE_TAG_ON {
                            x1 = int26p6_to_dbl(v_control.x);
                            y1 = int26p6_to_dbl(v_control.y);
                            x2 = int26p6_to_dbl(vec.x);
                            y2 = int26p6_to_dbl(vec.y);
                            if flip_y {
                                y1 = -y1;
                                y2 = -y2;
                            }
                            mtx.transform(&mut x1, &mut y1);
                            mtx.transform(&mut x2, &mut y2);
                            path.curve3(
                                from_i32!(dbl_to_int26p6(x1)),
                                from_i32!(dbl_to_int26p6(y1)),
                                from_i32!(dbl_to_int26p6(x2)),
                                from_i32!(dbl_to_int26p6(y2)),
                            );
                            continue 'wouter;
                        }

                        if tag != FT_CURVE_TAG_CONIC {
                            return false;
                        }

                        v_middle.x = (v_control.x + vec.x) / 2;
                        v_middle.y = (v_control.y + vec.y) / 2;

                        x1 = int26p6_to_dbl(v_control.x);
                        y1 = int26p6_to_dbl(v_control.y);
                        x2 = int26p6_to_dbl(v_middle.x);
                        y2 = int26p6_to_dbl(v_middle.y);
                        if flip_y {
                            y1 = -y1;
                            y2 = -y2;
                        }
                        mtx.transform(&mut x1, &mut y1);
                        mtx.transform(&mut x2, &mut y2);
                        path.curve3(
                            from_i32!(dbl_to_int26p6(x1)),
                            from_i32!(dbl_to_int26p6(y1)),
                            from_i32!(dbl_to_int26p6(x2)),
                            from_i32!(dbl_to_int26p6(y2)),
                        );

                        v_control = vec;
                    }

                    x1 = int26p6_to_dbl(v_control.x);
                    y1 = int26p6_to_dbl(v_control.y);
                    x2 = int26p6_to_dbl(v_start.x);
                    y2 = int26p6_to_dbl(v_start.y);
                    if flip_y {
                        y1 = -y1;
                        y2 = -y2;
                    }
                    mtx.transform(&mut x1, &mut y1);
                    mtx.transform(&mut x2, &mut y2);
                    path.curve3(
                        from_i32!(dbl_to_int26p6(x1)),
                        from_i32!(dbl_to_int26p6(y1)),
                        from_i32!(dbl_to_int26p6(x2)),
                        from_i32!(dbl_to_int26p6(y2)),
                    );
                    go = 1;
                    break 'wouter;
                }
                _ => {
                    let mut vec1: FT_Vector = FT_Vector { x: 0, y: 0 };
                    let mut vec2: FT_Vector = FT_Vector { x: 0, y: 0 };

                    if unsafe {
                        point.offset(1) > limit
                            || FT_CURVE_TAG!(*tags.offset(1)) != FT_CURVE_TAG_CUBIC
                    } {
                        return false;
                    }
                    unsafe {
                        vec1.x = (*point.offset(0)).x;
                        vec1.y = (*point.offset(0)).y;
                        vec2.x = (*point.offset(1)).x;
                        vec2.y = (*point.offset(2)).y;

                        point = point.offset(2);
                        tags = tags.offset(2);
                    }
                    if point <= limit {
                        let mut vec: FT_Vector = FT_Vector { x: 0, y: 0 };
                        unsafe {
                            vec.x = (*point).x;
                            vec.y = (*point).y;
                        }

                        x1 = int26p6_to_dbl(vec1.x);
                        y1 = int26p6_to_dbl(vec1.y);
                        x2 = int26p6_to_dbl(vec2.x);
                        y2 = int26p6_to_dbl(vec2.y);
                        x3 = int26p6_to_dbl(vec.x);
                        y3 = int26p6_to_dbl(vec.y);
                        if flip_y {
                            y1 = -y1;
                            y2 = -y2;
                            y3 = -y3;
                        }
                        mtx.transform(&mut x1, &mut y1);
                        mtx.transform(&mut x2, &mut y2);
                        mtx.transform(&mut x3, &mut y3);
                        path.curve4(
                            from_i32!(dbl_to_int26p6(x1)),
                            from_i32!(dbl_to_int26p6(y1)),
                            from_i32!(dbl_to_int26p6(x2)),
                            from_i32!(dbl_to_int26p6(y2)),
                            from_i32!(dbl_to_int26p6(x3)),
                            from_i32!(dbl_to_int26p6(y3)),
                        );

                        continue;
                    }

                    x1 = int26p6_to_dbl(vec1.x);
                    y1 = int26p6_to_dbl(vec1.y);
                    x2 = int26p6_to_dbl(vec2.x);
                    y2 = int26p6_to_dbl(vec2.y);
                    x3 = int26p6_to_dbl(v_start.x);
                    y3 = int26p6_to_dbl(v_start.y);
                    if flip_y {
                        y1 = -y1;
                        y2 = -y2;
                        y3 = -y3;
                    }
                    mtx.transform(&mut x1, &mut y1);
                    mtx.transform(&mut x2, &mut y2);
                    mtx.transform(&mut x3, &mut y3);
                    path.curve4(
                        from_i32!(dbl_to_int26p6(x1)),
                        from_i32!(dbl_to_int26p6(y1)),
                        from_i32!(dbl_to_int26p6(x2)),
                        from_i32!(dbl_to_int26p6(y2)),
                        from_i32!(dbl_to_int26p6(x3)),
                        from_i32!(dbl_to_int26p6(y3)),
                    );

                    go = 1;
                    break 'wouter;
                }
            }
        }
        if go == 0 {
            path.close_polygon();
        }
        go = 0;

        first = last + 1;
    }

    true
}

const CRC32_TAB: [u32; 256] = [
    0x00000000, 0x77073096, 0xee0e612c, 0x990951ba, 0x076dc419, 0x706af48f, 0xe963a535, 0x9e6495a3,
    0x0edb8832, 0x79dcb8a4, 0xe0d5e91e, 0x97d2d988, 0x09b64c2b, 0x7eb17cbd, 0xe7b82d07, 0x90bf1d91,
    0x1db71064, 0x6ab020f2, 0xf3b97148, 0x84be41de, 0x1adad47d, 0x6ddde4eb, 0xf4d4b551, 0x83d385c7,
    0x136c9856, 0x646ba8c0, 0xfd62f97a, 0x8a65c9ec, 0x14015c4f, 0x63066cd9, 0xfa0f3d63, 0x8d080df5,
    0x3b6e20c8, 0x4c69105e, 0xd56041e4, 0xa2677172, 0x3c03e4d1, 0x4b04d447, 0xd20d85fd, 0xa50ab56b,
    0x35b5a8fa, 0x42b2986c, 0xdbbbc9d6, 0xacbcf940, 0x32d86ce3, 0x45df5c75, 0xdcd60dcf, 0xabd13d59,
    0x26d930ac, 0x51de003a, 0xc8d75180, 0xbfd06116, 0x21b4f4b5, 0x56b3c423, 0xcfba9599, 0xb8bda50f,
    0x2802b89e, 0x5f058808, 0xc60cd9b2, 0xb10be924, 0x2f6f7c87, 0x58684c11, 0xc1611dab, 0xb6662d3d,
    0x76dc4190, 0x01db7106, 0x98d220bc, 0xefd5102a, 0x71b18589, 0x06b6b51f, 0x9fbfe4a5, 0xe8b8d433,
    0x7807c9a2, 0x0f00f934, 0x9609a88e, 0xe10e9818, 0x7f6a0dbb, 0x086d3d2d, 0x91646c97, 0xe6635c01,
    0x6b6b51f4, 0x1c6c6162, 0x856530d8, 0xf262004e, 0x6c0695ed, 0x1b01a57b, 0x8208f4c1, 0xf50fc457,
    0x65b0d9c6, 0x12b7e950, 0x8bbeb8ea, 0xfcb9887c, 0x62dd1ddf, 0x15da2d49, 0x8cd37cf3, 0xfbd44c65,
    0x4db26158, 0x3ab551ce, 0xa3bc0074, 0xd4bb30e2, 0x4adfa541, 0x3dd895d7, 0xa4d1c46d, 0xd3d6f4fb,
    0x4369e96a, 0x346ed9fc, 0xad678846, 0xda60b8d0, 0x44042d73, 0x33031de5, 0xaa0a4c5f, 0xdd0d7cc9,
    0x5005713c, 0x270241aa, 0xbe0b1010, 0xc90c2086, 0x5768b525, 0x206f85b3, 0xb966d409, 0xce61e49f,
    0x5edef90e, 0x29d9c998, 0xb0d09822, 0xc7d7a8b4, 0x59b33d17, 0x2eb40d81, 0xb7bd5c3b, 0xc0ba6cad,
    0xedb88320, 0x9abfb3b6, 0x03b6e20c, 0x74b1d29a, 0xead54739, 0x9dd277af, 0x04db2615, 0x73dc1683,
    0xe3630b12, 0x94643b84, 0x0d6d6a3e, 0x7a6a5aa8, 0xe40ecf0b, 0x9309ff9d, 0x0a00ae27, 0x7d079eb1,
    0xf00f9344, 0x8708a3d2, 0x1e01f268, 0x6906c2fe, 0xf762575d, 0x806567cb, 0x196c3671, 0x6e6b06e7,
    0xfed41b76, 0x89d32be0, 0x10da7a5a, 0x67dd4acc, 0xf9b9df6f, 0x8ebeeff9, 0x17b7be43, 0x60b08ed5,
    0xd6d6a3e8, 0xa1d1937e, 0x38d8c2c4, 0x4fdff252, 0xd1bb67f1, 0xa6bc5767, 0x3fb506dd, 0x48b2364b,
    0xd80d2bda, 0xaf0a1b4c, 0x36034af6, 0x41047a60, 0xdf60efc3, 0xa867df55, 0x316e8eef, 0x4669be79,
    0xcb61b38c, 0xbc66831a, 0x256fd2a0, 0x5268e236, 0xcc0c7795, 0xbb0b4703, 0x220216b9, 0x5505262f,
    0xc5ba3bbe, 0xb2bd0b28, 0x2bb45a92, 0x5cb36a04, 0xc2d7ffa7, 0xb5d0cf31, 0x2cd99e8b, 0x5bdeae1d,
    0x9b64c2b0, 0xec63f226, 0x756aa39c, 0x026d930a, 0x9c0906a9, 0xeb0e363f, 0x72076785, 0x05005713,
    0x95bf4a82, 0xe2b87a14, 0x7bb12bae, 0x0cb61b38, 0x92d28e9b, 0xe5d5be0d, 0x7cdcefb7, 0x0bdbdf21,
    0x86d3d2d4, 0xf1d4e242, 0x68ddb3f8, 0x1fda836e, 0x81be16cd, 0xf6b9265b, 0x6fb077e1, 0x18b74777,
    0x88085ae6, 0xff0f6a70, 0x66063bca, 0x11010b5c, 0x8f659eff, 0xf862ae69, 0x616bffd3, 0x166ccf45,
    0xa00ae278, 0xd70dd2ee, 0x4e048354, 0x3903b3c2, 0xa7672661, 0xd06016f7, 0x4969474d, 0x3e6e77db,
    0xaed16a4a, 0xd9d65adc, 0x40df0b66, 0x37d83bf0, 0xa9bcae53, 0xdebb9ec5, 0x47b2cf7f, 0x30b5ffe9,
    0xbdbdf21c, 0xcabac28a, 0x53b39330, 0x24b4a3a6, 0xbad03605, 0xcdd70693, 0x54de5729, 0x23d967bf,
    0xb3667a2e, 0xc4614ab8, 0x5d681b02, 0x2a6f2b94, 0xb40bbe37, 0xc30c8ea1, 0x5a05df1b, 0x2d02ef8d,
];
