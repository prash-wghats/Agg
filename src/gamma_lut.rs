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

use crate::{basics::uround, AggInteger, Gamma};

pub struct GammaLut<
    LoResT: AggInteger = u8,
    HiResT: AggInteger = u8,
    const GAMMA_SHIFT: u32 = 8,
    const HI_RES_SHIFT: u32 = 8,
> {
    gamma: f64,
    dir_gamma: Vec<HiResT>,
    inv_gamma: Vec<LoResT>,
}

impl<LoResT: AggInteger, HiResT: AggInteger, const GAMMA_SHIFT: u32, const HI_RES_SHIFT: u32>
    GammaLut<LoResT, HiResT, GAMMA_SHIFT, HI_RES_SHIFT>
{
    const GAMMA_SHIFT: u32 = GAMMA_SHIFT;
    const GAMMA_SIZE: u32 = 1 << GAMMA_SHIFT;
    const GAMMA_MASK: u32 = Self::GAMMA_SIZE - 1;

    const HI_RES_SHIFT: u32 = HI_RES_SHIFT;
    const HI_RES_SIZE: u32 = 1 << HI_RES_SHIFT;
    const HI_RES_MASK: u32 = Self::HI_RES_SIZE - 1;

    pub fn new_with_gamma(g: f64) -> GammaLut<LoResT, HiResT, GAMMA_SHIFT, HI_RES_SHIFT> {
        let mut lut = GammaLut::new();
        lut.set_gamma(g);
        lut
    }

    pub fn set_gamma(&mut self, g: f64) {
        self.gamma = g;

        for i in 0..Self::GAMMA_SIZE {
            self.dir_gamma[i as usize] = HiResT::from_u32(uround(
                (i as f64 / Self::GAMMA_MASK as f64).powf(self.gamma) * Self::HI_RES_MASK as f64,
            ) as u32);
        }

        let inv_g = 1.0 / g;
        for i in 0..Self::HI_RES_SIZE {
            self.inv_gamma[i as usize] = LoResT::from_u32(uround(
                (i as f64 / Self::HI_RES_MASK as f64).powf(inv_g) * Self::GAMMA_MASK as f64,
            ) as u32);
        }
    }

    pub fn gamma(&self) -> f64 {
        self.gamma
    }
}

impl<LoResT: AggInteger, HiResT: AggInteger, const GAMMA_SHIFT: u32, const HI_RES_SHIFT: u32>
    Gamma<LoResT, HiResT> for GammaLut<LoResT, HiResT, GAMMA_SHIFT, HI_RES_SHIFT>
{
	fn new() -> Self {
        let mut dir_gamma = Vec::<HiResT>::with_capacity(Self::GAMMA_SIZE as usize);
        let mut inv_gamma = Vec::<LoResT>::with_capacity(Self::HI_RES_SIZE as usize);

        for i in 0..Self::GAMMA_SIZE {
            dir_gamma.push(HiResT::from_u32(
                i << (Self::HI_RES_SHIFT - Self::GAMMA_SHIFT),
            ));
        }

        for i in 0..Self::HI_RES_SIZE {
            inv_gamma.push(LoResT::from_u32(
                i >> (Self::HI_RES_SHIFT - Self::GAMMA_SHIFT),
            ));
        }

        GammaLut {
            gamma: 1.0,
            dir_gamma: dir_gamma,
            inv_gamma: inv_gamma,
        }
    }

    fn dir(&self, v: LoResT) -> HiResT {
        self.dir_gamma[v.into_u32() as usize]
    }

    fn inv(&self, v: HiResT) -> LoResT {
        self.inv_gamma[v.into_u32() as usize]
    }
}
