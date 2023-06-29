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

use crate::basics::*;
use crate::dda_line::{Dda2LineIp};
use crate::Interpolator;
use crate::Transformer;

//NOT TESTED
//================================================SpanIpLinear
pub struct SpanIpLinear<T: Transformer, const SUBPIXEL_SHIFT: u32 = 8> {
    trans: T,
    li_x: Dda2LineIp,
    li_y: Dda2LineIp,
}

impl<T: Transformer, const SUBPIXEL_SHIFT: u32> SpanIpLinear<T, SUBPIXEL_SHIFT> {
    
    //const subpixel_shift: u32 = SUBPIXEL_SHIFT;
    const SUBPIXEL_SCALE: u32 = 1 << SUBPIXEL_SHIFT;

    pub fn resynchronize(&mut self, xe_: f64, ye_: f64, len: u32) {
        let (mut xe, mut ye) = (xe_, ye_);
        self.trans.transform(&mut xe, &mut ye);
        self.li_x = Dda2LineIp::new_fwd(self.li_x.y(), xe.round() as i32, len as i32);
        self.li_y = Dda2LineIp::new_fwd(self.li_y.y(), ye.round() as i32, len as i32);
    }

	pub fn new_begin(trans: T, x:f64, y:f64, len:u32) -> Self {
        let mut s = SpanIpLinear {
            trans: trans,
            li_x: Dda2LineIp::new_fwd(0, 0, 0),
            li_y: Dda2LineIp::new_fwd(0, 0, 0),
        };
		s.begin(x, y, len);
		s
    }
	pub fn new(trans: T) -> Self {
        SpanIpLinear {
            trans: trans,
            li_x: Dda2LineIp::new_fwd(0, 0, 0),
            li_y: Dda2LineIp::new_fwd(0, 0, 0),
        }
    }

	pub fn transformer(&self) -> &T {
		&self.trans
	}

	pub fn transformer_mut(&mut self) -> &mut T {
		&mut self.trans
	}

	pub fn set_transformer(&mut self, trans: T) {
		self.trans = trans;
	}

}

impl<T: Transformer, const SUBPIXEL_SHIFT: u32> Interpolator
    for SpanIpLinear<T, SUBPIXEL_SHIFT>
{
    type Trf = T;
    const SUBPIXEL_SHIFT: u32 = SUBPIXEL_SHIFT;

    fn begin(&mut self, x: f64, y: f64, len: u32) {
        let mut tx = x;
        let mut ty = y;
        self.trans.transform(&mut tx, &mut ty);
        let x1 = iround(tx * Self::SUBPIXEL_SCALE as f64);
        let y1 = iround(ty * Self::SUBPIXEL_SCALE as f64);

        let mut tx = x + len as f64;
        let mut ty = y;
        self.trans.transform(&mut tx, &mut ty);
        let x2 = iround(tx * Self::SUBPIXEL_SCALE as f64);
        let y2 = iround(ty * Self::SUBPIXEL_SCALE as f64);

        self.li_x = Dda2LineIp::new_fwd(x1, x2, len as i32);
        self.li_y = Dda2LineIp::new_fwd(y1, y2, len as i32);
    }

    fn next(&mut self) {
        self.li_x.inc();
        self.li_y.inc();
    }

    fn coordinates(&self, x: &mut i32, y: &mut i32) {
        *x = self.li_x.y();
		*y = self.li_y.y();
    }
}


 //=====================================span_interpolator_linear_subdiv
 pub struct SpanIpLinearSubdiv<T: Transformer, const SUBPIXEL_SHIFT: u32 = 8> {
	_subdiv_shift: u32,
	subdiv_size: u32,
	_subdiv_mask: u32,
	trans: T,
	li_x: Dda2LineIp,
	li_y: Dda2LineIp,
	src_x: i32,
	src_y: f64,
	pos: u32,
	len: u32,
}

impl<T: Transformer, const SUBPIXEL_SHIFT: u32> SpanIpLinearSubdiv<T, SUBPIXEL_SHIFT> {
	const SUBPIXEL_SCALE: u32 = 1 << SUBPIXEL_SHIFT;

	pub fn new_begin(trans: T, x:f64, y:f64, len:u32, _subdiv_shift: u32) -> Self {
		
		let subdiv_size = 1 << _subdiv_shift;
		let _subdiv_mask = subdiv_size - 1;
		let mut s = SpanIpLinearSubdiv {
			_subdiv_shift,
			subdiv_size,
			_subdiv_mask,
			trans,
			li_x: Dda2LineIp::new_fwd(0, 0, 0),
			li_y: Dda2LineIp::new_fwd(0, 0, 0),
			src_x: 0,
			src_y: 0.0,
			pos: 0,
			len: 0,
		};
		s.begin(x, y, len);
		s
	}

	pub fn new(trans: T) -> Self {
		let _subdiv_shift = 4;
		let subdiv_size = 1 << _subdiv_shift;
		let _subdiv_mask = subdiv_size - 1;
		SpanIpLinearSubdiv {
			_subdiv_shift,
			subdiv_size,
			_subdiv_mask,
			trans,
			li_x: Dda2LineIp::new_fwd(0, 0, 0),
			li_y: Dda2LineIp::new_fwd(0, 0, 0),
			src_x: 0,
			src_y: 0.0,
			pos: 0,
			len: 0,
		}
	}

	pub fn transformer(&self) -> &T {
		&self.trans
	}

	pub fn transformer_mut(&mut self) -> &mut T {
		&mut self.trans
	}

	pub fn set_transformer(&mut self, trans: T) {
		self.trans = trans;
	}
}

impl<T: Transformer, const SUBPIXEL_SHIFT: u32> Interpolator
    for SpanIpLinearSubdiv<T, SUBPIXEL_SHIFT>
{
    type Trf = T;
    const SUBPIXEL_SHIFT: u32 = SUBPIXEL_SHIFT;
	
	fn begin(&mut self, x: f64, y: f64, len: u32) {
		let mut len = len;
		let mut tx = x;
		let mut ty = y;
		self.pos = 1;
		self.src_x = iround(x * Self::SUBPIXEL_SCALE as f64) + Self::SUBPIXEL_SCALE as i32;
		self.src_y = y;
		self.len = len;

		if len > self.subdiv_size {
			len = self.subdiv_size;
		}

		self.trans.transform(&mut tx, &mut ty);
		let x1 = iround(tx * Self::SUBPIXEL_SCALE as f64);
		let y1 = iround(ty * Self::SUBPIXEL_SCALE as f64);

		tx = x + len as f64;
		ty = y;
		self.trans.transform(&mut tx, &mut ty);

		self.li_x = Dda2LineIp::new_fwd(x1, iround(tx * Self::SUBPIXEL_SCALE as f64), len as i32);
		self.li_y = Dda2LineIp::new_fwd(y1, iround(ty * Self::SUBPIXEL_SCALE as f64), len as i32);
	}
	
	fn next(&mut self) {
		self.li_x.inc();
		self.li_y.inc();
		if self.pos >= self.subdiv_size {
			let mut len = self.len;
			if len > self.subdiv_size {
				len = self.subdiv_size;
			}
			let mut tx = (self.src_x as f64) / (Self::SUBPIXEL_SCALE as f64) + len as f64;
			let mut ty = self.src_y;
			self.trans.transform(&mut tx, &mut ty);
			self.li_x = Dda2LineIp::new_fwd(self.li_x.y(), iround(tx * Self::SUBPIXEL_SCALE as f64), len as i32);
			self.li_y = Dda2LineIp::new_fwd(self.li_y.y(), iround(ty * Self::SUBPIXEL_SCALE as f64), len as i32);
			self.pos = 0;
		}
		self.src_x += Self::SUBPIXEL_SCALE as i32;
		self.pos += 1;
		self.len -= 1;
	}

	fn coordinates(&self, x: &mut i32, y: &mut i32) {
        *x = self.li_x.y();
		*y = self.li_y.y();
    }
}

