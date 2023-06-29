use crate::color_gray::{Gray16, Gray8};
use crate::color_rgba::{OrderBgra};
use crate::{
    basics::{RectI, RowData},
    RenderBuf,
};
use crate::{
    slice_t_to_vt, slice_t_to_vt_mut, AggPrimitive, Args, BlenderG, Color, Equiv, GrayArgs,
    ImageSrc, PixFmt, PixFmtGray, RenderBuffer,
};
use std::marker::PhantomData;
use wrapping_arithmetic::wrappit;

pub type BlenderGray8 = BlenderGray<Gray8>;
pub type BlenderGray8Pre = BlenderGrayPre<Gray8>;
pub type BlenderGray16 = BlenderGray<Gray16>;
pub type BlenderGray16Pre = BlenderGrayPre<Gray16>;

pub type PixGray8<'a> = AlphaBlendGray<'a, Gray8, BlenderGray8, RenderBuf>;
pub type PixGray8Pre<'a> = AlphaBlendGray<'a, Gray8, BlenderGray8Pre, RenderBuf>;
pub type PixGray16<'a> = AlphaBlendGray<'a, Gray16, BlenderGray16, RenderBuf>;
pub type PixGray16Pre<'a> = AlphaBlendGray<'a, Gray16, BlenderGray16Pre, RenderBuf>;

pub struct BlenderGray<C> {
    pub color_type: PhantomData<C>,
}


impl<C: Color> BlenderG<C> for BlenderGray<C> {
	fn new() -> Self {
        BlenderGray {
            color_type: PhantomData,
        }
    }

    #[wrappit]
    #[inline]
    fn blend_pix_with_cover(&self, p: &mut C::ValueType, cv: u32, alpha: u32, _cover: u32) {
        *p = C::ValueType::from_u32(
            (((cv - (*p).into_u32()) * alpha) + ((*p).into_u32() << C::BASE_SHIFT))
                >> C::BASE_SHIFT,
        );
    }

    fn blend_pix(&self, p: &mut C::ValueType, cv: u32, alpha: u32) {
        self.blend_pix_with_cover(p, cv, alpha, 0);
    }
}

//======================================================blender_gray_pre
pub struct BlenderGrayPre<C> {
    pub color_type: PhantomData<C>,
}

impl<C: Color> BlenderG<C> for BlenderGrayPre<C> {
	fn new() -> Self {
        BlenderGrayPre {
            color_type: PhantomData,
        }
    }

    #[wrappit]
    #[inline]
    fn blend_pix_with_cover(&self, p: &mut C::ValueType, cv: u32, alpha: u32, cover: u32) {
        let alpha = C::BASE_MASK - alpha;
        let cover = (cover + 1) << (C::BASE_SHIFT - 8);

        *p = C::ValueType::from_u32(((*p).into_u32() * alpha + cv * cover) >> C::BASE_SHIFT);
    }
    fn blend_pix(&self, p: &mut C::ValueType, cv: u32, alpha: u32) {
        *p = C::ValueType::from_u32(
            (((*p).into_u32() * (C::BASE_MASK - alpha)) >> C::BASE_SHIFT) + cv,
        );
    }
}

//=====================================================apply_gamma_dir_gray
pub struct ApplyGammaDirGray<
    'a,
    C: crate::Color,
    GammaLut: crate::Gamma<C::ValueType, C::ValueType>,
> {
    gamma: &'a GammaLut,
    phantom_color: PhantomData<C>,
}

impl<'a, C: crate::Color, GammaLut: crate::Gamma<C::ValueType, C::ValueType>>
    ApplyGammaDirGray<'a, C, GammaLut>
{
    pub fn new(gamma: &'a GammaLut) -> Self {
        ApplyGammaDirGray {
            gamma: gamma,
            phantom_color: PhantomData,
        }
    }

    #[inline]
    fn apply(&self, p: &mut C::ValueType) {
        *p = self.gamma.dir(*p);
    }
}

//=====================================================apply_gamma_inv_gray
pub struct ApplyGammaInvGray<
    'a,
    C: crate::Color,
    GammaLut: crate::Gamma<C::ValueType, C::ValueType>,
> {
    gamma: &'a GammaLut,
    phantom_color: PhantomData<C>,
}

impl<'a, C: crate::Color, GammaLut: crate::Gamma<C::ValueType, C::ValueType>>
    ApplyGammaInvGray<'a, C, GammaLut>
{
    pub fn new(gamma: &'a GammaLut) -> Self {
        ApplyGammaInvGray {
            gamma: gamma,
            phantom_color: PhantomData,
        }
    }

    #[inline]
    fn apply(&self, p: &mut C::ValueType) {
        *p = self.gamma.inv(*p);
    }
}

///=================================================AlphaBlendGray
pub struct AlphaBlendGray<
    'a,
    C: Color + GrayArgs,
    Blend: BlenderG<C>,
    RenBuf: RenderBuffer<T = u8>,
    const STEP: u32 = 1,
    const OFFSET: u32 = 0,
> {
    rbuf: Equiv<'a, RenBuf>,
    color: PhantomData<C>,
    blender: Blend,
}

impl<
        'a,
        C: Color + GrayArgs,
        Blend: BlenderG<C>,
        RenBuf: RenderBuffer<T = u8>,
        const STEP: u32,
        const OFFSET: u32,
    > ImageSrc for AlphaBlendGray<'a, C, Blend, RenBuf, STEP, OFFSET>
{
}

impl<
        'a,
        C: Color + GrayArgs,
        Blend: BlenderG<C>,
        RenBuf: RenderBuffer<T = u8>,
        const STEP: u32,
        const OFFSET: u32,
    > PixFmtGray for AlphaBlendGray<'a, C, Blend, RenBuf, STEP, OFFSET>
{
    const PIXEL_STEP: u32 = STEP;
    const PIXEL_OFFSET: u32 = OFFSET;
}

impl<
        'a,
        C: Color + GrayArgs,
        Blend: BlenderG<C>,
        RenBuf: RenderBuffer<T = u8>,
        const STEP: u32,
        const OFFSET: u32,
    > AlphaBlendGray<'a, C, Blend, RenBuf, STEP, OFFSET>
{
    pub fn new_borrowed(rbuf: &'a mut RenBuf) -> Self {
        AlphaBlendGray {
            rbuf: Equiv::Brw(rbuf),
            blender: Blend::new(),
            color: PhantomData,
        }
    }

    pub fn new_owned(rbuf: RenBuf) -> Self {
        AlphaBlendGray {
            rbuf: Equiv::Own(rbuf),
            blender: Blend::new(),
            color: PhantomData,
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
    fn copy_or_blend_pix(p: &mut C::ValueType, c: &C, blender: &Blend) {
        if c.a().into_u32() != 0 {
            if c.a().into_u32() == C::BASE_MASK {
                *p = c.v();
            } else {
                blender.blend_pix(p, c.v().into_u32(), c.a().into_u32());
            }
        }
    }

    #[inline]
    fn copy_or_blend_pix_cover(p: &mut C::ValueType, c: &C, cover: u32, blender: &Blend) {
        if c.a().into_u32() != 0 {
            let alpha = (c.a().into_u32() * (cover + 1)) >> 8;
            if alpha == C::BASE_MASK {
                *p = c.v();
            } else {
                blender.blend_pix_with_cover(p, c.v().into_u32(), alpha, cover);
            }
        }
    }

    pub fn for_each_pixel(&mut self, func: &dyn Fn(&mut C::ValueType)) {
        for y in 0..self.height() as i32 {
            let r = self.rbuf.row_data(y);
            if !r.ptr.is_null() {
                let len = r.x2 - r.x1 + 1;
                let p = slice_t_to_vt_mut!(
                    self.rbuf.row_mut(y),
                    r.x1 as u32 * STEP + OFFSET,
                    C::ValueType
                );

                let mut i = 0;
                for _ in 0..len {
                    func(&mut p[i as usize]);
                    i += STEP;
                }
            }
        }
    }

    pub fn apply_gamma_dir<GammaLut: crate::Gamma<C::ValueType, C::ValueType>>(
        &mut self, g: &GammaLut,
    ) {
        let ag = ApplyGammaDirGray::<C, GammaLut>::new(g);
        self.for_each_pixel(&|p| ag.apply(p));
    }

    pub fn apply_gamma_inv<GammaLut: crate::Gamma<C::ValueType, C::ValueType>>(
        &mut self, g: &GammaLut,
    ) {
        let ag = ApplyGammaInvGray::<C, GammaLut>::new(g);
        self.for_each_pixel(&|p| ag.apply(p));
    }
}

impl<
        'a,
        C: Color + GrayArgs,
        Blend: BlenderG<C>,
        RenBuf: RenderBuffer<T = u8>,
        const STEP: u32,
        const OFFSET: u32,
    > PixFmt for AlphaBlendGray<'a, C, Blend, RenBuf, STEP, OFFSET>
{
    type C = C;
    type O = OrderBgra;
    type T = RenBuf::T;
    const PIXEL_WIDTH: u32 = std::mem::size_of::<C::ValueType>() as u32; // * 2?

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
            *p = c.v();
        }
    }

    fn pixel(&self, x: i32, y: i32) -> C {
        let p = slice_t_to_vt!(self.rbuf.row(y), x as u32 * STEP + OFFSET, C::ValueType);

        //C::new_from_rgba(&Rgba::new_params(v * 0.299, v * 0.587, v * 0.114, 1.0))
        C::new_init(p[0], C::ValueType::from_u32(255))
    }

    fn pix_ptr(&self, x: i32, y: i32) -> (&[u8], usize) {
        let p;
        let h = self.rbuf.height() as i32;
        let stride = self.rbuf.stride();
        let len;
        let off;
        let pw = (x * STEP as i32 + OFFSET as i32);
        if stride < 0 {
            p = self.rbuf.row(h - 1).as_ptr();
            len = (h - y) * stride.abs();
            off = (h - y - 1) * stride.abs() + pw;
        } else {
            off = pw;
            p = self.rbuf.row(y).as_ptr();
            len = (h - y) * stride.abs();
        }
        (
            unsafe { std::slice::from_raw_parts(p as *const u8, len as usize) },
            off as usize,
        )
    }

    fn copy_pixel(&mut self, x: i32, y: i32, c: &C) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x as u32 * STEP + OFFSET, C::ValueType);
        p[0] = c.v();
    }

    fn blend_pixel(&mut self, x: i32, y: i32, c: &C, cover: u8) {
        Self::copy_or_blend_pix_cover(
            &mut slice_t_to_vt_mut!(self.rbuf.row_mut(y), x as u32 * STEP + OFFSET, C::ValueType)
                [0],
            c,
            cover as u32,
            &self.blender,
        )
    }

    fn copy_hline(&mut self, x: i32, y: i32, len: u32, c: &C) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x as u32 * STEP + OFFSET, C::ValueType);
        for i in 0..len {
            p[i as usize] = c.v();
        }
    }

    fn copy_vline(&mut self, x: i32, y: i32, len: u32, c: &C) {
        for i in 0..len as i32 {
            let p = slice_t_to_vt_mut!(
                self.rbuf.row_mut(y + i),
                x as u32 * STEP + OFFSET,
                C::ValueType
            );
            p[0] = c.v();
        }
    }

    fn blend_hline(&mut self, x: i32, y: i32, len: u32, c: &C, cover: u8) {
        if c.a().into_u32() != 0 {
            let p =
                slice_t_to_vt_mut!(self.rbuf.row_mut(y), x as u32 * STEP + OFFSET, C::ValueType);
            let alpha = (c.a().into_u32() * (cover as u32 + 1)) >> 8;
            if alpha == C::BASE_MASK {
                for i in (0..len * STEP).step_by(STEP as usize) {
                    p[i as usize] = c.v();
                }
            } else {
                for i in (0..len * STEP).step_by(STEP as usize) {
                    self.blender.blend_pix_with_cover(
                        &mut p[i as usize],
                        c.v().into_u32(),
                        alpha,
                        cover as u32,
                    );
                }
            }
        }
    }

    fn blend_vline(&mut self, x: i32, y: i32, len: u32, c: &C, cover: u8) {
        if c.a().into_u32() != 0 {
            let alpha = (c.a().into_u32() * (cover as u32 + 1)) >> 8;
            if alpha == C::BASE_MASK {
                for i in 0..len as i32 {
                    slice_t_to_vt_mut!(
                        self.rbuf.row_mut(y+i),
                        x as u32 * STEP + OFFSET,
                        C::ValueType
                    )[0] = c.v();
                }
            } else {
                for i in 0..len as i32 {
                    self.blender.blend_pix_with_cover(
                        &mut slice_t_to_vt_mut!(
                            self.rbuf.row_mut(y+i),
                            x as u32 * STEP + OFFSET,
                            C::ValueType
                        )[0],
                        c.v().into_u32(),
                        alpha,
                        cover as u32,
                    );
                }
            }
        }
    }

    fn blend_solid_hspan(&mut self, x: i32, y: i32, len: u32, c: &C, covers: &[u8]) {
        if c.a().into_u32() != 0 {
            let p =
                slice_t_to_vt_mut!(self.rbuf.row_mut(y), x as u32 * STEP + OFFSET, C::ValueType);
            for i in 0..len {
                let alpha = (c.a().into_u32() * (covers[i as usize] as u32 + 1)) >> 8;
                if alpha == C::BASE_MASK {
                    p[(i * STEP) as usize] = c.v();
                } else {
                    self.blender.blend_pix_with_cover(
                        &mut p[(i * STEP) as usize],
                        c.v().into_u32(),
                        alpha,
                        covers[i as usize] as u32,
                    );
                }
            }
        }
    }

    fn blend_solid_vspan(&mut self, x: i32, y: i32, len: u32, c: &C, covers: &[u8]) {
        if c.a().into_u32() != 0 {
            for i in 0..len as i32 {
                let alpha = (c.a().into_u32() * (covers[i as usize] as u32 + 1)) >> 8;
                if alpha == C::BASE_MASK {
                    slice_t_to_vt_mut!(
                        self.rbuf.row_mut(y+i),
                        x as u32 * STEP + OFFSET,
                        C::ValueType
                    )[0] = c.v();
                } else {
                    self.blender.blend_pix_with_cover(
                        &mut slice_t_to_vt_mut!(
                            self.rbuf.row_mut(y+i),
                            x as u32 * STEP + OFFSET,
                            C::ValueType
                        )[0],
                        c.v().into_u32(),
                        alpha,
                        covers[i as usize] as u32,
                    );
                }
            }
        }
    }

    fn blend_color_hspan(
        &mut self, x: i32, y: i32, len: u32, colors: &[C], covers: &[u8], cover: u8,
    ) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x as u32 * STEP + OFFSET, C::ValueType);
        if covers.len() > 0 {
            for i in 0..len {
                Self::copy_or_blend_pix_cover(
                    &mut p[(i * STEP) as usize],
                    &colors[i as usize],
                    covers[i as usize] as u32,
                    &self.blender,
                );
            }
        } else {
            if cover == 255 {
                for i in 0..len {
                    if colors[i as usize].a().into_u32() == C::BASE_MASK {
                        p[(i * STEP) as usize] = colors[i as usize].v();
                    } else {
                        Self::copy_or_blend_pix(
                            &mut p[(i * STEP) as usize],
                            &colors[i as usize],
                            &self.blender,
                        );
                    }
                }
            } else {
                for i in 0..len {
                    Self::copy_or_blend_pix_cover(
                        &mut p[(i * STEP) as usize],
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
        if covers.len() > 0 {
            for i in 0..len as i32 {
                Self::copy_or_blend_pix_cover(
                    &mut slice_t_to_vt_mut!(
                        self.rbuf.row_mut(y + i),
                        x as u32 * STEP + OFFSET,
                        C::ValueType
                    )[0],
                    &colors[i as usize],
                    covers[i as usize] as u32,
                    &self.blender,
                );
            }
        } else {
            if cover == 255 {
                for i in 0..len as i32 {
                    let p = slice_t_to_vt_mut!(
                        self.rbuf.row_mut(y + i),
                        x as u32 * STEP + OFFSET,
                        C::ValueType
                    );
                    if colors[i as usize].a().into_u32() == C::BASE_MASK {
                        p[0] = colors[i as usize].v();
                    } else {
                        Self::copy_or_blend_pix(&mut p[0], &colors[i as usize], &self.blender);
                    }
                }
            } else {
                for i in 0..len as i32 {
                    Self::copy_or_blend_pix_cover(
                        &mut slice_t_to_vt_mut!(
                            self.rbuf.row_mut(y + i),
                            x as u32 * STEP + OFFSET,
                            C::ValueType
                        )[0],
                        &colors[i as usize],
                        cover as u32,
                        &self.blender,
                    );
                }
            }
        }
    }

    fn copy_color_hspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]) {
        let p = slice_t_to_vt_mut!(self.rbuf.row_mut(y), x as u32 * STEP + OFFSET, C::ValueType);

        for i in 0..len {
            p[(i * STEP) as usize] = colors[i as usize].v();
        }
    }

    fn copy_color_vspan(&mut self, x: i32, y: i32, len: u32, colors: &[Self::C]) {
        for i in 0..len as i32 {
            slice_t_to_vt_mut!(
                self.rbuf.row_mut(y + i),
                x as u32 * STEP + OFFSET,
                C::ValueType
            )[0] = colors[i as usize].v();
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

    // Blend from a source with pixel format conversion

    fn blend_from<R: PixFmt>(
        &mut self, _from: &R, _xdst: i32, _ydst: i32, _xsrc: i32, _ysrc: i32, _len: u32,
        _cover: u32,
    ) {
    }

    fn blend_from_color<R: PixFmt>(
        &mut self, from: &R, color: &C, xdst: i32, ydst: i32, _xsrc: i32, ysrc: i32, len: u32,
        cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let psrc = slice_t_to_vt!(psrc, 0, <<R as PixFmt>::C as Args>::ValueType);
            let pdst =
                slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst, <Self::C as Args>::ValueType);
            for i in 0..len as usize {
                Self::copy_or_blend_pix_cover(
                    &mut pdst[i],
                    &color,
                    (psrc[i].into_u32() * cover + C::BASE_MASK) >> C::BASE_SHIFT,
                    &self.blender,
                );
            }
        }
        /*let mut psrc = from.row_ptr(ysrc) as *mut <<R as PixFmt>::C as Args>::ValueType;
        if !psrc.is_null() {
            unsafe {
                let mut pdst = self.rbuf.row_ptr(xdst, ydst, len) as *mut C::ValueType;
                pdst = pdst.offset(xdst as isize);
                let mut i: u32 = 0;
                while i < len {
                    let cover: u32 = (*psrc).into_u32() * cover + C::BASE_MASK >> C::BASE_SHIFT;
                    Self::copy_or_blend_pix_cover(pdst, &color, cover);
                    psrc = psrc.offset(1);
                    pdst = pdst.offset(1);
                    i = i + 1;
                }
            }
        }*/
    }

    fn blend_from_lut<R: PixFmt>(
        &mut self, from: &R, color_lut: &[C], xdst: i32, ydst: i32, _xsrc: i32, ysrc: i32,
        len: u32, cover: u32,
    ) {
        let psrc = from.row(ysrc);
        if !psrc.is_empty() {
            let psrc = slice_t_to_vt!(psrc, 0, <<R as PixFmt>::C as Args>::ValueType);
            let pdst =
                slice_t_to_vt_mut!(self.rbuf.row_mut(ydst), xdst, <Self::C as Args>::ValueType);
            for i in 0..len as usize {
                Self::copy_or_blend_pix_cover(
                    &mut pdst[i],
                    &color_lut[psrc[i].into_u32() as usize],
                    cover,
                    &self.blender,
                );
            }
        }
        /*let mut psrc = from.row_ptr(ysrc) as *mut <<R as PixFmt>::C as Args>::ValueType;
        if !psrc.is_null() {
            unsafe {
                let mut pdst = self.rbuf.row_ptr(xdst, ydst, len) as *mut C::ValueType;
                pdst = pdst.offset((xdst) as isize);
                let mut i: u32 = 0;
                while i < len {
                    let cover: u32 = (*psrc).into_u32() * cover + C::BASE_MASK >> C::BASE_SHIFT;
                    Self::copy_or_blend_pix_cover(
                        pdst,
                        &color_lut[(*psrc).into_u32() as usize],
                        cover,
                    );
                    psrc = psrc.offset(1);
                    pdst = pdst.offset(1);
                    i = i + 1;
                }
            }
        }*/
    }
}
