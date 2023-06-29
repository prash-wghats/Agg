use crate::{
    Color, GrayArgs, ImageAccessor, ImageAccessorGray, ImageAccessorRgb, ImageWrap, PixFmt, RgbArgs,
};

//-----------------------------------------------------ImageAccessorClip
pub struct ImageAccessorClip<P: PixFmt> {
    pixf: P,
    bk_buf: [u8; 4],
    x: i32,
    x0: i32,
    y: i32,
    pix_ptr: *const u8,
    pix_off: usize,
}

impl<P: PixFmt> ImageAccessorClip<P> {
    pub fn new(pix: P, bk: &P::C) -> Self {
        let mut p = Self {
            pixf: pix,
            bk_buf: [0; 4],
            x: 0,
            x0: 0,
            y: 0,
            pix_ptr: std::ptr::null(),
            pix_off: 0,
        };
        p.pixf.make_pix(&mut p.bk_buf, bk);
        p
    }

    pub fn attach(&mut self, pixf: P) {
        self.pixf = pixf;
    }

    pub fn background_color(&mut self, bk: &P::C) {
        self.pixf.make_pix(&mut self.bk_buf, bk);
    }

    fn pixel(&self) -> &[u8] {
        if self.y >= 0
            && self.y < self.pixf.height() as i32
            && self.x >= 0
            && self.x < self.pixf.width() as i32
        {
            let (p, i) = self.pixf.pix_ptr(self.x, self.y);
            return &p[i..];
        }
        return &self.bk_buf;
    }
}

//impl<P: PixFmt> ImageSrc for ImageAccessorClip<P> {}
impl<C: Color + RgbArgs, P: PixFmt<C = C>> ImageAccessorRgb for ImageAccessorClip<P> {
    type ColorType = P::C;
    type OrderType = P::O;
}

impl<C: Color + GrayArgs, P: PixFmt<C = C>> ImageAccessorGray for ImageAccessorClip<P> {
    type ColorType = P::C;
    type OrderType = P::O;
}

impl<P: PixFmt> ImageAccessor for ImageAccessorClip<P> {
    fn span(&mut self, x: i32, y: i32, len: u32) -> &[u8] {
        self.x = x;
        self.x0 = x;
        self.y = y;
        if y >= 0
            && y < self.pixf.height() as i32
            && x >= 0
            && x + (len as i32) <= self.pixf.width() as i32
        {
            let (p, i) = self.pixf.pix_ptr(self.x, self.y);
            (self.pix_ptr, self.pix_off) = (p.as_ptr(), i);
            return unsafe {
                std::slice::from_raw_parts(self.pix_ptr.offset(i as isize), len as usize)
            };
        }
        self.pix_ptr = std::ptr::null();
        self.pix_off = 0;
        return self.pixel();
    }

    fn next_x(&mut self) -> &[u8] {
        if !self.pix_ptr.is_null() {
            self.pix_off += P::PIXEL_WIDTH as usize;
            return unsafe {
                std::slice::from_raw_parts(self.pix_ptr.offset(self.pix_off as isize), 4 as usize)
            };
        }
        self.x += 1;
        return self.pixel();
    }

    fn next_y(&mut self) -> &[u8] {
        self.y += 1;
        self.x = self.x0;
        if !self.pix_ptr.is_null() && self.y >= 0 && self.y < self.pixf.height() as i32 {
            let (p, i) = self.pixf.pix_ptr(self.x, self.y);
            (self.pix_ptr, self.pix_off) = (p.as_ptr(), i);
            return unsafe { std::slice::from_raw_parts(self.pix_ptr.offset(i as isize), 4) };
        }
        self.pix_ptr = std::ptr::null();
        self.pix_off = 0;
        return self.pixel();
    }
}

//--------------------------------------------------ImageAccessorNoClip
pub struct ImageAccessorNoClip<P: PixFmt> {
    pixf: P,
    x: i32,
    y: i32,
    pix_ptr: *const u8,
    pix_off: usize,
}

impl<P: PixFmt> ImageAccessorNoClip<P> {
    pub fn new(pix: P) -> Self {
        Self {
            pixf: pix,
            x: 0,
            y: 0,
            pix_ptr: std::ptr::null(),
            pix_off: 0,
        }
    }

    pub fn attach(&mut self, pixf: P) {
        self.pixf = pixf;
    }
}
impl<C: Color + RgbArgs, P: PixFmt<C = C>> ImageAccessorRgb for ImageAccessorNoClip<P> {
    type ColorType = P::C;
    type OrderType = P::O;
}

impl<C: Color + GrayArgs, P: PixFmt<C = C>> ImageAccessorGray for ImageAccessorNoClip<P> {
    type ColorType = P::C;
    type OrderType = P::O;
}

impl<P: PixFmt> ImageAccessor for ImageAccessorNoClip<P> {
    fn span(&mut self, x: i32, y: i32, len: u32) -> &[u8] {
        self.x = x;
        self.y = y;
        let (p, i) = self.pixf.pix_ptr(self.x, self.y);
        (self.pix_ptr, self.pix_off) = (p.as_ptr(), i);
        return unsafe {
            std::slice::from_raw_parts(self.pix_ptr.offset(i as isize), len as usize)
        };
    }

    fn next_x(&mut self) -> &[u8] {
        if !self.pix_ptr.is_null() {
            self.pix_off += P::PIXEL_WIDTH as usize;
            return unsafe {
                std::slice::from_raw_parts(self.pix_ptr.offset(self.pix_off as isize), 4 as usize)
            };
        }
        &[]
    }

    fn next_y(&mut self) -> &[u8] {
        self.y += 1;
        let (p, i) = self.pixf.pix_ptr(self.x, self.y);
        (self.pix_ptr, self.pix_off) = (p.as_ptr(), i);
        return unsafe { std::slice::from_raw_parts(self.pix_ptr.offset(i as isize), 4 as usize) };
    }
}

//----------------------------------------------------ImageAccessorClone
pub struct ImageAccessorClone<P: PixFmt> {
    pixf: P,
    x: i32,
    x0: i32,
    y: i32,
    pix_ptr: *const u8,
    pix_off: usize,
}

impl<P: PixFmt> ImageAccessorClone<P> {
    pub fn new(pix: P) -> Self {
        Self {
            pixf: pix,
            x: 0,
            x0: 0,
            y: 0,
            pix_ptr: std::ptr::null(),
            pix_off: 0,
        }
    }

    pub fn attach(&mut self, pixf: P) {
        self.pixf = pixf;
    }

    fn pixel(&self) -> &[u8] {
        let mut x = self.x;
        let mut y = self.y;
        if x < 0 {
            x = 0;
        }
        if y < 0 {
            y = 0;
        }
        if x >= self.pixf.width() as i32 {
            x = self.pixf.width() as i32 - 1;
        }
        if y >= self.pixf.height() as i32 {
            y = self.pixf.height() as i32 - 1;
        }
        let (p, i) = self.pixf.pix_ptr(x, y);
        return &p[i..];
    }
}
impl<C: Color + RgbArgs, P: PixFmt<C = C>> ImageAccessorRgb for ImageAccessorClone<P> {
    type ColorType = P::C;
    type OrderType = P::O;
}

impl<C: Color + GrayArgs, P: PixFmt<C = C>> ImageAccessorGray for ImageAccessorClone<P> {
    type ColorType = P::C;
    type OrderType = P::O;
}

impl<P: PixFmt> ImageAccessor for ImageAccessorClone<P> {
    fn span(&mut self, x: i32, y: i32, len: u32) -> &[u8] {
        self.x = x;
        self.x0 = x;
        self.y = y;
        if y >= 0
            && y < self.pixf.height() as i32
            && x >= 0
            && x + len as i32 <= self.pixf.width() as i32
        {
            let (p, i) = self.pixf.pix_ptr(self.x, self.y);
            (self.pix_ptr, self.pix_off) = (p.as_ptr(), i);
            return unsafe {
                std::slice::from_raw_parts(self.pix_ptr.offset(i as isize), len as usize)
            };
        } else {
            self.pix_ptr = std::ptr::null();
            self.pix_off = 0;
            self.pixel()
        }
    }

    fn next_x(&mut self) -> &[u8] {
        if !self.pix_ptr.is_null() {
            self.pix_off += P::PIXEL_WIDTH as usize;
            return unsafe {
                std::slice::from_raw_parts(self.pix_ptr.offset(self.pix_off as isize), 4 as usize)
            };
        } else {
            self.x += 1;
            self.pixel()
        }
    }

    fn next_y(&mut self) -> &[u8] {
        self.y += 1;
        self.x = self.x0;
        if !self.pix_ptr.is_null() && self.y >= 0 && self.y < self.pixf.height() as i32 {
            let (p, i) = self.pixf.pix_ptr(self.x, self.y);
            (self.pix_ptr, self.pix_off) = (p.as_ptr(), i);
            return unsafe {
                std::slice::from_raw_parts(self.pix_ptr.offset(i as isize), 4 as usize)
            };
        } else {
            self.pix_ptr = std::ptr::null();
            self.pix_off = 0;
            self.pixel()
        }
    }
}

pub struct ImageAccessorWrap<'a, P: PixFmt, WrapX: ImageWrap, WrapY: ImageWrap> {
    pixf: P,
    wrap_x: WrapX,
    wrap_y: WrapY,
    row: &'a [u8],
    x: i32,
}

impl<'a, C: Color + RgbArgs, P: PixFmt<C = C>, WrapX: ImageWrap, WrapY: ImageWrap> ImageAccessorRgb
    for ImageAccessorWrap<'a, P, WrapX, WrapY>
{
    type ColorType = P::C;
    type OrderType = P::O;
}

impl<'a, C: Color + GrayArgs, P: PixFmt<C = C>, WrapX: ImageWrap, WrapY: ImageWrap>
    ImageAccessorGray for ImageAccessorWrap<'a, P, WrapX, WrapY>
{
    type ColorType = P::C;
    type OrderType = P::O;
}

impl<'a, P: PixFmt, WrapX: ImageWrap, WrapY: ImageWrap> ImageAccessor
    for ImageAccessorWrap<'a, P, WrapX, WrapY>
{
    fn span(&mut self, x: i32, y: i32, _: u32) -> &[u8] {
        self.x = x;
        let r = self.pixf.row(self.wrap_y.get(y) as i32);
        self.row = unsafe {
            std::slice::from_raw_parts(
                r.as_ptr() as *const u8,
                r.len() * std::mem::size_of::<P::T>(),
            )
        };
        &self.row[(self.wrap_x.get(x) * P::PIXEL_WIDTH) as usize..]
    }

    fn next_x(&mut self) -> &[u8] {
        let x = self.wrap_x.inc();
        &self.row[(x * P::PIXEL_WIDTH) as usize..]
    }

    fn next_y(&mut self) -> &[u8] {
        let r = self.pixf.row(self.wrap_y.inc() as i32);
        self.row = unsafe {
            std::slice::from_raw_parts(
                r.as_ptr() as *const u8,
                r.len() * std::mem::size_of::<P::T>(),
            )
        };
        &self.row[(self.wrap_x.get(self.x) * P::PIXEL_WIDTH) as usize..]
    }
}

impl<'a, P: PixFmt, WrapX: ImageWrap, WrapY: ImageWrap> ImageAccessorWrap<'a, P, WrapX, WrapY> {
    pub fn new(pixf: P) -> Self {
        let w = pixf.width();
        let h = pixf.height();
        ImageAccessorWrap {
            pixf: pixf,
            wrap_x: WrapX::new(w),
            wrap_y: WrapY::new(h),
            row: &[],
            x: 0,
        }
    }

    pub fn attach(&mut self, pixf: P) {
        self.pixf = pixf;
    }
}

//--------------------------------------------------------WrapModeRepeat
pub struct WrapModeRepeat {
    size: u32,
    add: u32,
    value: u32,
}

impl ImageWrap for WrapModeRepeat {
    fn new(size: u32) -> WrapModeRepeat {
        WrapModeRepeat {
            size: size,
            add: size * (0x3FFFFFFF / size),
            value: 0,
        }
    }

    fn get(&mut self, v: i32) -> u32 {
        self.value = (v as u32 + self.add) % self.size;
        self.value
    }

    fn inc(&mut self) -> u32 {
        self.value += 1;
        if self.value >= self.size {
            self.value = 0;
        }
        self.value
    }
}

//---------------------------------------------------WrapModeRepeatPow2
pub struct WrapModeRepeatPow2 {
    mask: u32,
    value: u32,
}

impl ImageWrap for WrapModeRepeatPow2 {
    fn new(size: u32) -> WrapModeRepeatPow2 {
        let mut mask = 1;
        while mask < size {
            mask = (mask << 1) | 1;
        }
        mask >>= 1;
        WrapModeRepeatPow2 {
            mask: mask,
            value: 0,
        }
    }

    fn get(&mut self, v: i32) -> u32 {
        self.value = v as u32 & self.mask;
        self.value
    }

    fn inc(&mut self) -> u32 {
        self.value += 1;
        if self.value > self.mask {
            self.value = 0;
        }
        self.value
    }
}

//----------------------------------------------WrapModeRepeatAutoPow2
pub struct WrapModeRepeatAutoPow2 {
    size: u32,
    add: u32,
    mask: u32,
    value: u32,
}

impl ImageWrap for WrapModeRepeatAutoPow2 {
    fn new(size: u32) -> WrapModeRepeatAutoPow2 {
        let mask = if size & (size - 1) == 0 { size - 1 } else { 0 };
        WrapModeRepeatAutoPow2 {
            size: size,
            add: size * (0x3FFFFFFF / size),
            mask: mask,
            value: 0,
        }
    }

    fn get(&mut self, v: i32) -> u32 {
        if self.mask != 0 {
            self.value = v as u32 & self.mask;
        } else {
            self.value = (v as u32 + self.add) % self.size;
        }
        self.value
    }

    fn inc(&mut self) -> u32 {
        self.value += 1;
        if self.value >= self.size {
            self.value = 0;
        }
        self.value
    }
}

//-------------------------------------------------------WrapModeReflect
pub struct WrapModeReflect {
    size: u32,
    size2: u32,
    add: u32,
    value: u32,
}

impl ImageWrap for WrapModeReflect {
    fn new(size: u32) -> WrapModeReflect {
        WrapModeReflect {
            size: size,
            size2: size * 2,
            add: size * 2 * (0x3FFFFFFF / size * 2),
            value: 0,
        }
    }

    fn get(&mut self, v: i32) -> u32 {
        self.value = (v as u32 + self.add) % self.size2;
        if self.value >= self.size {
            self.size2 - self.value - 1
        } else {
            self.value
        }
    }

    fn inc(&mut self) -> u32 {
        self.value += 1;
        if self.value >= self.size2 {
            self.value = 0;
        }
        if self.value >= self.size {
            self.size2 - self.value - 1
        } else {
            self.value
        }
    }
}

//---------------------------------------------WrapModeReflectAutoPow2
pub struct WrapModeReflectAutoPow2 {
    size: u32,
    size2: u32,
    add: u32,
    mask: u32,
    value: u32,
}

impl ImageWrap for WrapModeReflectAutoPow2 {
    fn new(size: u32) -> WrapModeReflectAutoPow2 {
        let size2 = size * 2;
        let add = size2 * (0x3FFFFFFF / size2);
        let mask = if size2 & (size2 - 1) == 0 {
            size2 - 1
        } else {
            0
        };
        WrapModeReflectAutoPow2 {
            size: size,
            size2: size2,
            add: add,
            mask: mask,
            value: 0,
        }
    }

    fn get(&mut self, v: i32) -> u32 {
        self.value = if self.mask != 0 {
            v as u32 & self.mask
        } else {
            (self.add.wrapping_add(v as u32)) % self.size2
        };
        if self.value >= self.size {
            self.size2 - self.value - 1
        } else {
            self.value
        }
    }

    fn inc(&mut self) -> u32 {
        self.value += 1;
        if self.value >= self.size2 {
            self.value = 0;
        }
        if self.value >= self.size {
            self.size2 - self.value - 1
        } else {
            self.value
        }
    }
}
