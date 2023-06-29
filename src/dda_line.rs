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
// classes DdaLineIp, Dda2LineIp
//
//----------------------------------------------------------------------------

//===================================================DdaLineIp
pub struct DdaLineIp<const FRAC_SHIFT: i32, const YSHIFT: i32 = 0> {
    m_y: i32,
    m_inc: i32,
    m_dy: i32,
}

impl<const FRAC_SHIFT: i32, const YSHIFT: i32> DdaLineIp<FRAC_SHIFT, YSHIFT> {
    pub fn new(y1: i32, y2: i32, count: u32) -> Self {
        Self {
            m_y: y1,
            m_inc: ((y2 - y1) << FRAC_SHIFT) / count as i32,
            m_dy: 0,
        }
    }

    pub fn y(&self) -> i32 {
        self.m_y + (self.m_dy >> (FRAC_SHIFT - YSHIFT))
    }

    pub fn dy(&self) -> i32 {
        self.m_dy
    }

    pub fn inc(&mut self) {
        self.m_dy += self.m_inc
    }

    pub fn dec(&mut self) {
        self.m_dy -= self.m_inc
    }

    pub fn inc_by(&mut self, n: u32) {
        self.m_dy += self.m_inc * n as i32;
    }

    pub fn dec_by(&mut self, n: u32) {
        self.m_dy -= self.m_inc * n as i32;
    }
}

//=================================================Dda2LineIp
#[derive(Copy, Clone)]
pub struct Dda2LineIp {
    pub m_cnt: i32,
    pub m_lft: i32,
    pub m_rem: i32,
    pub m_mod: i32,
    pub m_y: i32,
}

impl Dda2LineIp {
    pub fn new() -> Dda2LineIp {
        Dda2LineIp {
            m_cnt: 0,
            m_lft: 0,
            m_rem: 0,
            m_mod: 0,
            m_y: 0,
        }
    }

    //-------------------------------------------- Forward-adjusted line
    pub fn new_fwd(y1: i32, y2: i32, count: i32) -> Dda2LineIp {
        let mut ret = Dda2LineIp::new();
        ret.m_cnt = if count <= 0 { 1 } else { count };
        ret.m_lft = (y2 - y1) / ret.m_cnt;
        ret.m_rem = (y2 - y1) % ret.m_cnt;
        ret.m_mod = ret.m_rem;
        ret.m_y = y1;
        if ret.m_mod <= 0 {
            ret.m_mod += count;
            ret.m_rem += count;
            ret.m_lft -= 1;
        }
        ret.m_mod -= count;
        ret
    }

    //-------------------------------------------- Backward-adjusted line
    pub fn new_bwd(y1: i32, y2: i32, count: i32, _: i32) -> Dda2LineIp {
        let mut ret = Dda2LineIp::new();
        ret.m_cnt = if count <= 0 { 1 } else { count };
        ret.m_lft = (y2 - y1) / ret.m_cnt;
        ret.m_rem = (y2 - y1) % ret.m_cnt;
        ret.m_mod = ret.m_rem;
        ret.m_y = y1;
        if ret.m_mod <= 0 {
            ret.m_mod += count;
            ret.m_rem += count;
            ret.m_lft -= 1;
        }
        ret
    }

    //-------------------------------------------- Backward-adjusted line
    pub fn new_bwd_y(y: i32, count: i32) -> Dda2LineIp {
        let mut ret = Dda2LineIp::new();
        ret.m_cnt = if count <= 0 { 1 } else { count };
        ret.m_lft = y / ret.m_cnt;
        ret.m_rem = y % ret.m_cnt;
        ret.m_mod = ret.m_rem;
        ret.m_y = 0;
        if ret.m_mod <= 0 {
            ret.m_mod += count;
            ret.m_rem += count;
            ret.m_lft -= 1;
        }
        ret
    }

    //--------------------------------------------------------------------
    pub fn save(&self, data: &mut [i32]) {
        data[0] = self.m_mod;
        data[1] = self.m_y;
    }

    //--------------------------------------------------------------------
    pub fn load(&mut self, data: &[i32]) {
        self.m_mod = data[0];
        self.m_y = data[1];
    }

    //--------------------------------------------------------------------
    pub fn inc(&mut self) {
        self.m_mod += self.m_rem;
        self.m_y += self.m_lft;
        if self.m_mod > 0 {
            self.m_mod -= self.m_cnt;
            self.m_y += 1;
        }
    }

    //--------------------------------------------------------------------
    pub fn dec(&mut self) {
        if self.m_mod <= self.m_rem {
            self.m_mod += self.m_cnt;
            self.m_y -= 1;
        }
        self.m_mod -= self.m_rem;
        self.m_y -= self.m_lft;
    }

    //--------------------------------------------------------------------
    pub fn adjust_forward(&mut self) {
        self.m_mod -= self.m_cnt;
    }

    //--------------------------------------------------------------------
    pub fn adjust_backward(&mut self) {
        self.m_mod += self.m_cnt;
    }

    //--------------------------------------------------------------------
    pub fn mod_(&self) -> i32 {
        self.m_mod
    }
    pub fn rem(&self) -> i32 {
        self.m_rem
    }
    pub fn lft(&self) -> i32 {
        self.m_lft
    }

    //--------------------------------------------------------------------
    pub fn y(&self) -> i32 {
        self.m_y
    }
}

//---------------------------------------------LineBresenhamIp
pub struct LineBresenhamIp {
    m_x1_lr: i32,
    m_y1_lr: i32,
    _m_x2_lr: i32,
    _m_y2_lr: i32,
    m_ver: bool,
    m_len: u32,
    m_inc: i32,
    m_interpolator: Dda2LineIp,
}

impl LineBresenhamIp {
    pub const SUBPIXEL_SHIFT: u32 = 8;
    pub const SUBPIXEL_SCALE: u32 = 1 << Self::SUBPIXEL_SHIFT;
    pub const SUBPIXEL_MASK: u32 = Self::SUBPIXEL_SCALE - 1;

    pub fn line_lr(v: i32) -> i32 {
        return v >> Self::SUBPIXEL_SHIFT as i32;
    }
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> LineBresenhamIp {
        let x1_lr = Self::line_lr(x1);
        let y1_lr = Self::line_lr(y1);
        let x2_lr = Self::line_lr(x2);
        let y2_lr = Self::line_lr(y2);

        let ver = (x2_lr - x1_lr).abs() < (y2_lr - y1_lr).abs();
        let len = if ver {
            (y2_lr - y1_lr).abs()
        } else {
            (x2_lr - x1_lr).abs()
        };
        let inc = if ver {
            if y2 > y1 {
                1
            } else {
                -1
            }
        } else {
            if x2 > x1 {
                1
            } else {
                -1
            }
        };
        LineBresenhamIp {
            m_x1_lr: x1_lr,
            m_y1_lr: y1_lr,
            _m_x2_lr: x2_lr,
            _m_y2_lr: y2_lr,
            m_ver: ver,
            m_len: len as u32,
            m_inc: inc,
            m_interpolator: Dda2LineIp::new_fwd(
                if ver { x1 } else { y1 },
                if ver { x2 } else { y2 },
                len,
            ),
        }
    }

    pub fn is_ver(&self) -> bool {
        self.m_ver
    }
    pub fn get_len(&self) -> u32 {
        self.m_len
    }
    pub fn get_inc(&self) -> i32 {
        self.m_inc
    }

    pub fn hstep(&mut self) {
        self.m_interpolator.inc();
        self.m_x1_lr += self.m_inc;
    }

    pub fn vstep(&mut self) {
        self.m_interpolator.inc();
        self.m_y1_lr += self.m_inc;
    }

    pub fn x1(&self) -> i32 {
        self.m_x1_lr
    }
    pub fn y1(&self) -> i32 {
        self.m_y1_lr
    }
    pub fn x2(&self) -> i32 {
        self.m_interpolator.y() >> 8
    }
    pub fn y2(&self) -> i32 {
        self.m_interpolator.y() >> 8
    }
    pub fn x2_hr(&self) -> i32 {
        self.m_interpolator.y()
    }
    pub fn y2_hr(&self) -> i32 {
        self.m_interpolator.y()
    }
}
