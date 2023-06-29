use crate::basics::RectI;
use crate::FontEngine;
use std::ptr::null;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GlyphDataType {
    Invalid = 0,
    Mono = 1,
    Gray8 = 2,
    Outline = 3,
}

#[derive(Clone)]
pub struct GlyphCache {
    pub glyph_index: u32,
    pub data: Vec<u8>,
    pub data_size: u32,
    pub data_type: GlyphDataType,
    pub bounds: RectI,
    pub advance_x: f64,
    pub advance_y: f64,
}

impl GlyphCache {
    pub fn new(
        glyph_index: u32, data: Vec<u8>, data_size: u32, data_type: GlyphDataType, bounds: RectI,
        advance_x: f64, advance_y: f64,
    ) -> Self {
        Self {
            glyph_index,
            data,
            data_size,
            data_type,
            bounds,
            advance_x,
            advance_y,
        }
    }
}

pub struct FontCache {
    glyphs: Vec<Vec<Option<GlyphCache>>>,
    font_signature: String,
}

impl FontCache {
    fn new() -> FontCache {
        FontCache {
            glyphs: vec![vec![None; 256]; 256],
            font_signature: String::new(),
        }
    }

    fn signature(&mut self, font_signature: &str) {
        self.font_signature = font_signature.to_string();
        self.glyphs = vec![vec![None; 256]; 256];
    }

    fn font_is(&self, font_signature: &str) -> bool {
        self.font_signature == font_signature
    }

    fn find_glyph(&self, glyph_code: u32) -> Option<&GlyphCache> {
        let msb = ((glyph_code >> 8) & 0xFF) as usize;
        if self.glyphs[msb].len() == 0 {
            return None;
        }
        let lsb = (glyph_code & 0xFF) as usize;
        return self.glyphs[msb][lsb].as_ref();
    }

    fn cache_glyph(
        &mut self, glyph_code: u32, glyph_index: u32, data_size: u32, data_type: GlyphDataType,
        bounds: RectI, advance_x: f64, advance_y: f64,
    ) -> Option<&GlyphCache> {
        let msb = ((glyph_code >> 8) & 0xFF) as usize;
        let lsb = (glyph_code & 0xFF) as usize;

        if self.glyphs[msb][lsb].is_none() {
            let data: Vec<u8> = vec![0; data_size as usize];
            self.glyphs[msb][lsb] = Some(GlyphCache::new(
                glyph_index,
                data,
                data_size,
                data_type,
                bounds,
                advance_x,
                advance_y,
            ));
        }
        self.glyphs[msb][lsb].as_ref()
    }
}

pub struct FontCachePool {
    fonts: Vec<FontCache>,
    max_fonts: usize,
    num_fonts: usize,
    cur_font: usize,
}

impl FontCachePool {
    pub fn new(max_fonts: u32) -> FontCachePool {
        FontCachePool {
            fonts: Vec::new(),
            max_fonts: max_fonts as usize,
            cur_font: 0,
            num_fonts: 0,
        }
    }

    pub fn font(&mut self, font_signature: &str, reset_cache: bool) {
        let idx = self.find_font(font_signature);
        if idx >= 0 {
            if reset_cache {
                self.fonts[idx as usize] = FontCache::new();
                self.fonts[idx as usize].signature(font_signature)
            }
            self.cur_font = idx as usize;
        } else {
            if self.num_fonts >= self.max_fonts {
                self.fonts.rotate_right(1);
                self.num_fonts = self.max_fonts - 1;
            }
            if self.num_fonts <= self.fonts.len() {
                self.fonts.push(FontCache::new())
            } else {
                self.fonts[self.num_fonts as usize] = FontCache::new();
            }
            self.fonts[self.num_fonts as usize].signature(font_signature);
            self.cur_font = self.num_fonts;
            self.num_fonts += 1;
        }
    }

    pub fn font_mut(&mut self) -> &mut FontCache {
        &mut self.fonts[self.cur_font]
    }

    pub fn find_glyph(&self, glyph_code: u32) -> Option<&GlyphCache> {
        self.fonts[self.cur_font].find_glyph(glyph_code)
    }

    pub fn cache_glyph(
        &mut self, glyph_code: u32, glyph_index: u32, data_size: u32, data_type: GlyphDataType,
        bounds: RectI, advance_x: f64, advance_y: f64,
    ) -> Option<&GlyphCache> {
        self.fonts[self.cur_font].cache_glyph(
            glyph_code,
            glyph_index,
            data_size,
            data_type,
            bounds,
            advance_x,
            advance_y,
        )
    }

    pub fn find_font(&self, font_signature: &str) -> isize {
        for (i, font) in self.fonts.iter().enumerate() {
            if font.font_is(font_signature) {
                return i as isize;
            }
        }
        -1
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GlyphRender {
    NativeMono,
    NativeGray8,
    Outline,
    AggMono,
    AggGray8,
}

pub struct FontCacheManager<F: FontEngine> {
    fonts: FontCachePool,
    engine: F,
    change_stamp: i32,
    prev_glyph: *const GlyphCache,
    last_glyph: *const GlyphCache,
    /*path_adaptor: PathAdaptor,
    gray8_adaptor: Gray8Adaptor,
    gray8_scanline: Gray8Scanline,
    mono_adaptor: MonoAdaptor,
    mono_scanline: MonoScanline,*/
}

impl<F: FontEngine> FontCacheManager<F> {
    pub fn new(engine: F, max_fonts: u32) -> Self {
        Self {
            fonts: FontCachePool::new(max_fonts),
            engine: engine,
            change_stamp: -1,
            prev_glyph: null(),
            last_glyph: null(),
            /*path_adaptor: PathAdaptor::new(),
            gray8_adaptor: Gray8Adaptor::new(),
            gray8_scanline: Gray8Scanline::new(),
            mono_adaptor: MonoAdaptor::new(),
            mono_scanline: MonoScanline::new(),*/
        }
    }

    pub fn reset_last_glyph(&mut self) {
        self.prev_glyph = null();
        self.last_glyph = null();
    }

	pub fn engine(&self) -> &F {
		&self.engine
	}

	pub fn engine_mut(&mut self) -> &mut F {
		&mut self.engine
	}

    pub fn glyph(&mut self, glyph_code: u32) -> *const GlyphCache {
        self.synchronize();

        let gl = self.fonts.find_glyph(glyph_code);
        if gl.is_some() {
            self.prev_glyph = self.last_glyph;
            self.last_glyph = gl.unwrap() as *const GlyphCache;
            return self.last_glyph;
        }

        if self.engine.prepare_glyph(glyph_code) {
            self.prev_glyph = self.last_glyph;
            self.last_glyph = (self.fonts.cache_glyph(
                glyph_code,
                self.engine.glyph_index(),
                self.engine.data_size(),
                self.engine.data_type(),
                *self.engine.bounds(),
                self.engine.advance_x(),
                self.engine.advance_y(),
            ))
            .unwrap() as *const GlyphCache;
            unsafe {
                self.engine
                    .write_glyph_to(&mut (*(self.last_glyph as *mut GlyphCache)).data);
            }
            return self.last_glyph;
        }

        null()
    }
/* 
    pub fn init_embedded_adaptors(&mut self, gl: Option<&GlyphCache>, x: f64, y: f64, scale: f64) {
        if let Some(gl) = gl {
            match &gl.data_type {
                GlyphDataType::Mono => {
                    self.mono_adaptor
                        .init(gl.data.as_ptr(), gl.data_size as usize, x, y);
                }
                GlyphDataType::Gray8 => {
                    self.gray8_adaptor
                        .init(gl.data.as_ptr(), gl.data_size as usize, x, y);
                }
                GlyphDataType::Outline => {
                    self.path_adaptor.init(
                        gl.data.as_ptr(),
                        gl.data_size as usize,
                        x,
                        y,
                        scale,
                    );
                }
                _ => {}
            }
        }
    }
*/
    pub fn path_adaptor(&self) -> F::PathAdaptorType {
		self.engine.path_adaptor()
    }

    pub fn gray8_adaptor(&self) -> F::Gray8AdaptorType {
        self.engine.gray8_adaptor()
    }

    pub fn gray8_scanline(& self) -> F::Gray8ScanlineType {
        self.engine.gray8_scanline()
    }

    pub fn mono_adaptor(& self) -> F::MonoAdaptorType {
        self.engine().mono_adaptor()
    }

    pub fn mono_scanline(& self) -> F::MonoScanlineType {
        self.engine.mono_scanline()
    }

    pub fn perv_glyph(&self) -> *const GlyphCache {
        self.prev_glyph
    }

    pub fn last_glyph(&self) -> *const GlyphCache {
        self.last_glyph
    }

    pub fn add_kerning(&mut self, x: &mut f64, y: &mut f64) -> bool {
        if !self.prev_glyph.is_null() && !self.last_glyph.is_null() {
            unsafe {
                self.engine.add_kerning(
                    (*self.prev_glyph).glyph_index,
                    (*self.last_glyph).glyph_index,
                    x,
                    y,
                )
            }
        } else {
            false
        }
    }

    pub fn precache(&mut self, from: u32, to: u32) {
        for i in from..to + 1 {
            self.glyph(i);
        }
    }

    pub fn reset_cache(&mut self) {
        self.fonts
            .font(&self.engine.font_signature(), true);
        self.change_stamp = self.engine.change_stamp();
        self.prev_glyph = null();
        self.last_glyph = null();
    }

    fn synchronize(&mut self) {
        if self.change_stamp != self.engine.change_stamp() {
            self.fonts
                .font(&self.engine.font_signature(), true);
            self.change_stamp = self.engine.change_stamp();
            self.prev_glyph = null();
            self.last_glyph = null();
        }
    }
}
