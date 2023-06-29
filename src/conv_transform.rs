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
// class ConvTransform
//
//----------------------------------------------------------------------------

use crate::basics::is_vertex;
use crate::{VertexSource, Transformer};
use crate::Equiv;

pub struct ConvTransform<'a, VS: VertexSource, Trf: Transformer> {
	source: Equiv<'a, VS>,
	trans: Trf,
}
impl<'a, VS: VertexSource, Trf: Transformer> ConvTransform<'a, VS, Trf> {
	pub fn new_owned(source: VS, tr: Trf) -> Self {
		ConvTransform {
			source: Equiv::Own(source),
			trans: tr,
		}
	}

	pub fn new_borrowed(source: &'a mut VS, tr: Trf) -> Self {
		ConvTransform {
			source: Equiv::Brw(source),
			trans: tr,
		}
	}

	pub fn set_source_owned(&mut self, source: VS) {
		self.source = Equiv::Own(source);
    }

	pub fn set_source_borrowed(&mut self, source: &'a mut VS) {
		self.source = Equiv::Brw(source);
    }

	pub fn set_transformer(&mut self, tr: Trf) -> Trf {
		let tmp = std::mem::replace(&mut self.trans, tr);
		tmp
	}

	pub fn source_mut(&mut self) -> &mut VS {
        &mut self.source
    }

    pub fn source(&self) -> &VS {
        & self.source
    }

	pub fn trans_mut(&mut self) -> &mut Trf {
        &mut self.trans
    }
    pub fn trans(&self) -> &Trf {
        & self.trans
    }
}

impl<'a, VS: VertexSource, Trf: Transformer>  VertexSource for ConvTransform<'a, VS, Trf> {
	fn rewind(&mut self, path_id: u32) {
		self.source.rewind(path_id);
	}

	fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
		let cmd = self.source.vertex(x, y);
		if is_vertex(cmd) {
			self.trans.transform(x, y);
		}
		cmd
	}
}