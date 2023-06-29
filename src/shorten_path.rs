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

use crate::vertex_sequence::*;
use crate::{VertexSequence, VertexDistance};

//===========================================================shorten_path
    pub fn shorten_path<T: VertexSequence<ValueType = VertexDist>>(vs: &mut T, s_: f64, closed: u32) {

		let mut s = s_;
        if s > 0.0 && vs.size() > 1 {
            let mut d: f64;
            let mut n = vs.size() - 2;
            while n > 0 {
                d = vs.get(n as usize).dist;
                if d > s {
                    break;
                }
                vs.remove_last();
                s -= d;
                n -= 1;
            }
            if vs.size() < 2 {
                vs.remove_all();
            } else {
                n = vs.size() - 1;
				let sl = vs.get_mut_slice(0, n);
				let mut prev = sl[n-1];
				let mut last = sl[n];
                d = (prev.dist - s) / prev.dist;
                let x = prev.x + (last.x - prev.x) * d;
                let y = prev.y + (last.y - prev.y) * d;
                last.x = x;
                last.y = y;
                if !prev.calc_distance(&last) {
                    vs.remove_last();
                }
                vs.close(closed != 0);
            }
        }
    }

