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

use crate::GammaFn;

//===============================================================GammaNone
pub struct GammaNone;
impl GammaNone {
    pub fn new() -> Self {
        Self {}
    }
}
impl GammaFn for GammaNone {
    fn call(&self, x: f64) -> f64 {
        x
    }
}

//==============================================================GammaPower
pub struct GammaPower {
    m_gamma: f64,
}
impl GammaPower {
    pub fn new() -> GammaPower {
        GammaPower { m_gamma: 1.0 }
    }
    pub fn new_with_gamma(g: f64) -> GammaPower {
        GammaPower { m_gamma: g }
    }
    pub fn get_gamma(&self) -> f64 {
        self.m_gamma
    }
    pub fn gamma(&mut self, g: f64) {
        self.m_gamma = g
    }
}

impl GammaFn for GammaPower {
    fn call(&self, x: f64) -> f64 {
        x.powf(self.m_gamma)
    }
}

//==========================================================GammaThreshold
pub struct GammaThreshold {
    m_threshold: f64,
}

impl GammaThreshold {
    pub fn new() -> GammaThreshold {
        GammaThreshold { m_threshold: 0.5 }
    }

    pub fn new_with_threshold(t: f64) -> GammaThreshold {
        GammaThreshold { m_threshold: t }
    }

    pub fn threshold(&mut self, t: f64) {
        self.m_threshold = t;
    }

    pub fn get_threshold(&self) -> f64 {
        self.m_threshold
    }
}
impl GammaFn for GammaThreshold {
    fn call(&self, x: f64) -> f64 {
        if x < self.m_threshold {
            0.0
        } else {
            1.0
        }
    }
}

//============================================================GammaLinear
pub struct GammaLinear {
    m_start: f64,
    m_end: f64,
}

impl GammaLinear {
    pub fn new() -> GammaLinear {
        GammaLinear {
            m_start: 0.0,
            m_end: 1.0,
        }
    }

    pub fn new_with_start_end(s: f64, e: f64) -> GammaLinear {
        GammaLinear {
            m_start: s,
            m_end: e,
        }
    }

    pub fn set(&mut self, s: f64, e: f64) {
        self.m_start = s;
        self.m_end = e;
    }

    pub fn start(&mut self, s: f64) {
        self.m_start = s;
    }

    pub fn end(&mut self, e: f64) {
        self.m_end = e;
    }

    pub fn get_start(&self) -> f64 {
        self.m_start
    }

    pub fn get_end(&self) -> f64 {
        self.m_end
    }
}
impl GammaFn for GammaLinear {
    fn call(&self, x: f64) -> f64 {
        if x < self.m_start {
            0.0
        } else if x > self.m_end {
            1.0
        } else {
            (x - self.m_start) / (self.m_end - self.m_start)
        }
    }
}

//==========================================================GammaMultiply
pub struct GammaMultiply {
    m_mul: f64,
}

impl GammaMultiply {
    pub fn new() -> GammaMultiply {
        GammaMultiply { m_mul: 1.0 }
    }

    pub fn new_with_value(v: f64) -> GammaMultiply {
        GammaMultiply { m_mul: v }
    }

    pub fn value(&mut self, v: f64) {
        self.m_mul = v;
    }

    pub fn get_value(&self) -> f64 {
        self.m_mul
    }
}
impl GammaFn for GammaMultiply {
    fn call(&self, x: f64) -> f64 {
        let mut y = x * self.m_mul;
        if y > 1.0 {
            y = 1.0;
        }
        y
    }
}
