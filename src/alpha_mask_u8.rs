use crate::basics::CoverType;
use crate::rendering_buffer::RenderBuf;
use crate::{AlphaMask, RenderBuffer};

pub type AlphaMaskGray8 = AlphaMaskU8<1, 0>;
pub type AlphaMaskRgb24R = AlphaMaskU8<3, 0>;
pub type AlphaMaskRgb24G = AlphaMaskU8<3, 1>;
pub type AlphaMaskRgb24B = AlphaMaskU8<3, 2>;
pub type AlphaMaskBgr24R = AlphaMaskU8<3, 2>;
pub type AlphaMaskBgr24G = AlphaMaskU8<3, 1>;
pub type AlphaMaskBgr24B = AlphaMaskU8<3, 0>;

pub type AlphaMaskRgb24Gray = AlphaMaskU8<3, 0, RgbtoGrayMaskU8<0, 1, 2>>;
pub type AlphaMaskBgr24Gray = AlphaMaskU8<3, 0, RgbtoGrayMaskU8<2, 1, 0>>;
pub type AlphaMaskRgba32Gray = AlphaMaskU8<4, 0, RgbtoGrayMaskU8<0, 1, 2>>;
pub type AlphaMaskArgb32Gray = AlphaMaskU8<4, 1, RgbtoGrayMaskU8<0, 1, 2>>;
pub type AlphaMaskBgra32Gray = AlphaMaskU8<4, 0, RgbtoGrayMaskU8<2, 1, 0>>;
pub type AlphaMaskAbgr32Gray = AlphaMaskU8<4, 1, RgbtoGrayMaskU8<2, 1, 0>>;

pub type AmaskNoClipGray8 = AmaskNoClipU8<1, 0>;
pub type AmaskNoClipRgb24R = AmaskNoClipU8<3, 0>;
pub type AmaskNoClipRgb24G = AmaskNoClipU8<3, 1>;
pub type AmaskNoClipRgb24B = AmaskNoClipU8<3, 2>;
pub type AmaskNoClipBgr24R = AmaskNoClipU8<3, 2>;
pub type AmaskNoClipBgr24G = AmaskNoClipU8<3, 1>;
pub type AmaskNoClipBgr24B = AmaskNoClipU8<3, 0>;

pub type AmaskNoClipRgb24Gray = AmaskNoClipU8<3, 0, RgbtoGrayMaskU8<0, 1, 2>>;
pub type AmaskNoClipBgr24Gray = AmaskNoClipU8<3, 0, RgbtoGrayMaskU8<2, 1, 0>>;
pub type AmaskNoClipRgba32Gray = AmaskNoClipU8<4, 0, RgbtoGrayMaskU8<0, 1, 2>>;
pub type AmaskNoClipArgb32Gray = AmaskNoClipU8<4, 1, RgbtoGrayMaskU8<0, 1, 2>>;
pub type AmaskNoClipBgra32Gray = AmaskNoClipU8<4, 0, RgbtoGrayMaskU8<2, 1, 0>>;
pub type AmaskNoClipAbgr32Gray = AmaskNoClipU8<4, 1, RgbtoGrayMaskU8<2, 1, 0>>;

pub trait MaskF {
    fn new() -> Self;
    fn calculate(&self, p: *mut CoverType) -> u32;
}

pub struct RgbtoGrayMaskU8<const R: u32, const G: u32, const B: u32>;

impl<const R: u32, const G: u32, const B: u32> MaskF for RgbtoGrayMaskU8<R, G, B> {
    fn new() -> Self {
        Self
    }
    fn calculate(&self, p: *mut CoverType) -> u32 {
        unsafe {
            (*p.offset(R as isize) as u32 * 77
                + *p.offset(G as isize) as u32 * 150
                + *p.offset(B as isize) as u32 * 29)
                >> 8
        }
    }
}
pub struct OneComponentMaskU8;
impl MaskF for OneComponentMaskU8 {
    fn new() -> Self {
        Self
    }
    fn calculate(&self, p: *mut CoverType) -> u32 {
        unsafe { (*p) as u32 }
    }
}

//==========================================================AlphaMaskU8
pub struct AlphaMaskU8<const STEP: u32 = 1, const OFFSET: u32 = 0, M: MaskF = OneComponentMaskU8> {
    pub rbuf: RenderBuf,
    pub mask_function: M,
}

impl<const STEP: u32, const OFFSET: u32, M: MaskF> AlphaMaskU8<STEP, OFFSET, M> {
    const COVER_SHIFT: u32 = 8;
    const _COVER_NONE: u32 = 0;

    pub fn new(rb: RenderBuf) -> Self {
        AlphaMaskU8 {
            rbuf: rb,
            mask_function: MaskF::new(),
        }
    }

    pub fn attach(&mut self, rbuf: RenderBuf) {
        self.rbuf = rbuf;
    }

    pub fn rbuf_mut(&mut self) -> &mut RenderBuf {
        &mut self.rbuf
    }

    pub fn mask_function<'a>(&'a mut self) -> &'a mut M {
        &mut self.mask_function
    }
}

impl<const STEP: u32, const OFFSET: u32, M: MaskF> AlphaMask for AlphaMaskU8<STEP, OFFSET, M> {
    const COVER_FULL: u32 = 255;
    type CoverType = CoverType;

    fn pixel(&self, x: i32, y: i32) -> CoverType {
        if x >= 0 && y >= 0 && x < self.rbuf.width() as i32 && y < self.rbuf.height() as i32 {
            let mut p = self.rbuf.row_ptr(0, y, 0) as *mut CoverType;
            p = unsafe { p.offset((x * STEP as i32 + OFFSET as i32) as isize) };
            return self.mask_function.calculate(p) as CoverType;
        }
        return 0;
    }

    fn combine_pixel(&self, x: i32, y: i32, val: u8) -> u8 {
        if x >= 0 && y >= 0 && x < self.rbuf.width() as i32 && y < self.rbuf.height() as i32 {
            let mut p = self.rbuf.row_ptr(0, y, 0) as *mut CoverType;
            p = unsafe { p.offset((x * STEP as i32 + OFFSET as i32) as isize) };
            return (255 + val as u32 * self.mask_function.calculate(p) >> 8) as CoverType;
        }
        return 0;
    }

    fn fill_hspan(&self, x: i32, y: i32, dst: &mut [CoverType], num_pix: i32) {
        let (mut x, y) = (x, y);
        let xmax = self.rbuf.width() - 1;
        let ymax = self.rbuf.height() - 1;

        let mut count = num_pix;
        let covers = dst;
        let mut c = 0;
        if y < 0 || y > ymax as i32 {
            for i in 0..num_pix as usize {
                covers[i] = 0;
            }
            return;
        }

        if x < 0 {
            count += x;
            if count <= 0 {
                for i in 0..num_pix as usize {
                    covers[i] = 0;
                }
                return;
            }
            for i in 0..-x as usize {
                covers[i] = 0;
            }
            c -= x;
            x = 0;
        }

        if x + count > xmax as i32 {
            let rest = x + count - xmax as i32 - 1;
            count -= rest;
            if count <= 0 {
                for i in 0..num_pix as usize {
                    covers[i] = 0;
                }
                return;
            }
            for i in (count + c)..rest * std::mem::size_of::<CoverType>() as i32 {
                covers[i as usize] = 0;
            }
        }

        let mut mask = self.rbuf.row_ptr(0, y, 0) as *mut CoverType;
        mask = unsafe { mask.offset((x * STEP as i32 + OFFSET as i32) as isize) };
        for i in 0..count as u32 {
            covers[i as usize] = self
                .mask_function
                .calculate(unsafe { mask.offset((i * STEP) as isize) })
                as CoverType;
        }
    }

    fn combine_hspan(&self, x: i32, y: i32, dst: &mut [u8], num_pix: i32) {
        let (mut x, y) = (x, y);
        let xmax = self.rbuf.width() - 1;
        let ymax = self.rbuf.height() - 1;

        let mut count = num_pix;
        let covers = dst;
        let mut c = 0;

        if y < 0 || y > ymax as i32 {
            for i in 0..num_pix as usize {
                covers[i] = 0;
            }
            return;
        }

        if x < 0 {
            count += x;
            if count <= 0 {
                for i in 0..num_pix as usize {
                    covers[i] = 0;
                }
                return;
            }
            for i in 0..-x as usize {
                covers[i] = 0;
            }
            c -= x;
            x = 0;
        }

        if x + count > xmax as i32 {
            let rest = x + count - xmax as i32 - 1;
            count -= rest;
            if count <= 0 {
                for i in 0..num_pix as usize {
                    covers[i] = 0;
                }
                return;
            }
            for i in (count + c)..rest * std::mem::size_of::<CoverType>() as i32 {
                covers[i as usize] = 0;
            }
        }

        let mut mask = self.rbuf.row_ptr(0, y, 0) as *mut CoverType;
        mask = unsafe { mask.offset((x * STEP as i32 + OFFSET as i32) as isize) };
        for i in 0..count as u32 {
            covers[i as usize] = ((Self::COVER_FULL
                + covers[i as usize] as u32
                    * self
                        .mask_function
                        .calculate(unsafe { mask.offset((i * STEP) as isize) }))
                >> Self::COVER_SHIFT) as CoverType;
        }
    }

    fn fill_vspan(&self, x: i32, y: i32, dst: &mut [u8], num_pix: i32) {
        let (x, mut y) = (x, y);
        let xmax = self.rbuf.width() - 1;
        let ymax = self.rbuf.height() - 1;

        let mut count = num_pix;
        let covers = dst;
        let mut c = 0;

        if x < 0 || x > xmax as i32 {
            for i in 0..num_pix as usize {
                covers[i] = 0;
            }
            return;
        }

        if y < 0 {
            count += y;
            if count <= 0 {
                for i in 0..num_pix as usize {
                    covers[i] = 0;
                }
                return;
            }
            for i in 0..-y as usize {
                covers[i] = 0;
            }
            c -= y;
            y = 0;
        }

        if y + count > ymax as i32 {
            let rest = y + count - ymax as i32 - 1;
            count -= rest;
            if count <= 0 {
                for i in 0..num_pix as usize {
                    covers[i] = 0;
                }
                return;
            }
            for i in (count + c)..rest * std::mem::size_of::<CoverType>() as i32 {
                covers[i as usize] = 0;
            }
        }

        let mut mask = self.rbuf.row_ptr(0, y, 0) as *mut CoverType;
        mask = unsafe { mask.offset((x * STEP as i32 + OFFSET as i32) as isize) };
        for i in 0..count {
            covers[i as usize] = self
                .mask_function
                .calculate(unsafe { mask.offset((i * self.rbuf.stride()) as isize) })
                as CoverType;
        }
    }

    fn combine_vspan(&self, x: i32, y: i32, dst: &mut [u8], num_pix: i32) {
        let (x, mut y) = (x, y);
        let xmax = self.rbuf.width() - 1;
        let ymax = self.rbuf.height() - 1;

        let mut count = num_pix;
        let covers = dst;
        let mut c = 0;

        if x < 0 || x > xmax as i32 {
            for i in 0..num_pix as usize {
                covers[i] = 0;
            }
            return;
        }

        if y < 0 {
            count += y;
            if count <= 0 {
                for i in 0..num_pix as usize {
                    covers[i] = 0;
                }
                return;
            }
            for i in 0..-y as usize {
                covers[i] = 0;
            }
            c -= y;
            y = 0;
        }

        if y + count > ymax as i32 {
            let rest = y + count - ymax as i32 - 1;
            count -= rest;
            if count <= 0 {
                for i in 0..num_pix as usize {
                    covers[i] = 0;
                }
                return;
            }
            for i in (count + c)..rest * std::mem::size_of::<CoverType>() as i32 {
                covers[i as usize] = 0;
            }
        }

        let mut mask = self.rbuf.row_ptr(0, y, 0) as *mut CoverType;
        mask = unsafe { mask.offset((x * STEP as i32 + OFFSET as i32) as isize) };
        for i in 0..count {
            covers[i as usize] = ((Self::COVER_FULL
                + covers[i as usize] as u32
                    * self
                        .mask_function
                        .calculate(unsafe { mask.offset((i * self.rbuf.stride()) as isize) }))
                >> Self::COVER_SHIFT) as u8;
        }
    }
}

//==========================================================AmaskNoClipU8
pub struct AmaskNoClipU8<const STEP: u32 = 1, const OFFSET: u32 = 0, M: MaskF = OneComponentMaskU8>
{
    pub rbuf: RenderBuf,
    pub mask_function: M,
}

impl<const STEP: u32, const OFFSET: u32, M: MaskF> AmaskNoClipU8<STEP, OFFSET, M> {
    const COVER_SHIFT: u32 = 8;
    const _COVER_NONE: u32 = 0;

    pub fn new(rb: RenderBuf) -> Self {
        AmaskNoClipU8 {
            rbuf: rb,
            mask_function: MaskF::new(),
        }
    }

    pub fn attach(&mut self, rbuf: RenderBuf) {
        self.rbuf = rbuf;
    }

    pub fn rbuf_mut(&mut self) -> &mut RenderBuf {
        &mut self.rbuf
    }

    pub fn mask_function_mut<'a>(&'a mut self) -> &'a mut M {
        &mut self.mask_function
    }
}

impl<const STEP: u32, const OFFSET: u32, M: MaskF> AlphaMask for AmaskNoClipU8<STEP, OFFSET, M> {
    const COVER_FULL: u32 = 255;
    type CoverType = CoverType;

    fn pixel(&self, x: i32, y: i32) -> CoverType {
        self.mask_function.calculate(unsafe {
            self.rbuf
                .row_ptr(0, y, 0)
                .offset((x * STEP as i32 + OFFSET as i32) as isize) as *mut CoverType
        }) as CoverType
    }

    fn combine_pixel(&self, x: i32, y: i32, val: CoverType) -> CoverType {
        (Self::COVER_FULL
            + val as u32
                * self.mask_function.calculate(unsafe {
                    self.rbuf
                        .row_ptr(0, y, 0)
                        .offset((x * STEP as i32 + OFFSET as i32) as isize)
                        as *mut CoverType
                })
            >> Self::COVER_SHIFT) as CoverType
    }

    fn fill_hspan(&self, x: i32, y: i32, dst: &mut [CoverType], num_pix: i32) {
        let mask = unsafe {
            self.rbuf
                .row_ptr(0, y, 0)
                .offset((x * STEP as i32 + OFFSET as i32) as isize)
        } as *mut CoverType;
        for i in 0..num_pix {
            dst[i as usize] = self
                .mask_function
                .calculate(unsafe { mask.offset((i as u32 * STEP) as isize) })
                as CoverType;
        }
    }

    fn combine_hspan(&self, x: i32, y: i32, dst: &mut [CoverType], num_pix: i32) {
        let mut mask = self.rbuf.row_ptr(0, y, 0) as *mut CoverType;
        mask = unsafe { mask.offset((x * STEP as i32 + OFFSET as i32) as isize) };
        for i in 0..num_pix {

            dst[i as usize] = ((Self::COVER_FULL
                + dst[i as usize] as u32 * self.mask_function.calculate(mask))
                >> Self::COVER_SHIFT) as CoverType;
            mask = unsafe { mask.offset(1) };

        }

    }

    fn fill_vspan(&self, x: i32, y: i32, dst: &mut [CoverType], num_pix: i32) {
        let mask = unsafe {
            self.rbuf
                .row_ptr(0, y, 0)
                .offset((x * STEP as i32 + OFFSET as i32) as isize)
        } as *mut CoverType;
        for i in 0..num_pix {
            dst[i as usize] = self
                .mask_function
                .calculate(unsafe { mask.offset((i * self.rbuf.stride()) as isize) })
                as CoverType;
        }
    }

    fn combine_vspan(&self, x: i32, y: i32, dst: &mut [CoverType], num_pix: i32) {
        let mask = unsafe {
            self.rbuf
                .row_ptr(0, y, 0)
                .offset((x * STEP as i32 + OFFSET as i32) as isize)
        } as *mut CoverType;
        for i in 0..num_pix {
            dst[i as usize] = ((Self::COVER_FULL
                + dst[i as usize] as u32
                    * self
                        .mask_function
                        .calculate(unsafe { mask.offset((i * self.rbuf.stride()) as isize) }))
                >> Self::COVER_SHIFT) as CoverType;
        }
    }
}
