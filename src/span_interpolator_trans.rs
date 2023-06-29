use crate::basics::iround;
use crate::Interpolator;
use crate::Transformer;

//=================================================span_interpolator_trans
pub struct SpanIpTrans<T: Transformer, const SUBPIXEL_SHIFT: u32 = 8> {
    trans: T,
    x: f64,
    y: f64,
    ix: i32,
    iy: i32,
}

impl<T: Transformer, const SUBPIXEL_SHIFT: u32> SpanIpTrans<T, SUBPIXEL_SHIFT> {
    const SUBPIXEL_SCALE: u32 = 1 << SUBPIXEL_SHIFT;
	pub fn new(trans: T) -> Self {
        SpanIpTrans {
            trans: trans,
            x: 0.0,
            y: 0.0,
            ix: 0,
            iy: 0,
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
    for SpanIpTrans<T, SUBPIXEL_SHIFT>
{
    type Trf = T;
    const SUBPIXEL_SHIFT: u32 = SUBPIXEL_SHIFT;

    

    fn begin(&mut self, x: f64, y: f64, _: u32) {
        let (mut x, mut y) = (x, y);
        self.x = x;
        self.y = y;
        self.trans.transform(&mut x, &mut y);
        self.ix = iround(x * Self::SUBPIXEL_SCALE as f64);
        self.iy = iround(y * Self::SUBPIXEL_SCALE as f64);
    }
	
    fn next(&mut self) {
        self.x += 1.0;
        let mut tx = self.x;
        let mut ty = self.y;
        self.trans.transform(&mut tx, &mut ty);
        self.ix = iround(tx * Self::SUBPIXEL_SCALE as f64);
        self.iy = iround(ty * Self::SUBPIXEL_SCALE as f64);
    }

    fn coordinates(&self, x: &mut i32, y: &mut i32) {
        *x = self.ix;
        *y = self.iy;
    }
}
