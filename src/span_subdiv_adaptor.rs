use crate::basics::iround;
use crate::Interpolator;
// NOT TESTED

//=================================================SpanSubdivAdaptor
pub struct SpanSubdivAdaptor<I: Interpolator, const SUBPIXEL_SHIFT: u32 = 8> {
    subdiv_shift: u32,
    subdiv_size: u32,
    subdiv_mask: u32,
    interpolator: I,
    src_x: i32,
    src_y: f64,
    pos: u32,
    len: u32,
}

impl<I: Interpolator, const SUBPIXEL_SHIFT: u32> SpanSubdivAdaptor<I, SUBPIXEL_SHIFT> {
    const SUBPIXEL_SCALE: u32 = 1 << SUBPIXEL_SHIFT;
    pub fn new(interpolator: I, subdiv_shift: u32) -> Self {
        let subdiv_size = 1 << subdiv_shift;
        let subdiv_mask = subdiv_size - 1;
        SpanSubdivAdaptor {
            subdiv_shift,
            subdiv_size,
            subdiv_mask,
            interpolator,
            src_x: 0,
            src_y: 0.0,
            pos: 0,
            len: 0,
        }
    }

    pub fn subdiv_shift(&self) -> u32 {
        self.subdiv_shift
    }

    pub fn set_subdiv_shift(&mut self, shift: u32) {
        self.subdiv_shift = shift;
        self.subdiv_size = 1 << self.subdiv_shift;
        self.subdiv_mask = self.subdiv_size - 1;
    }

	pub fn interpolator(&self) -> &I {
		&self.interpolator
	}

	pub fn interpolator_mut(&mut self) -> &mut I {
		&mut self.interpolator
	}
}

impl<I: Interpolator, const SUBPIXEL_SHIFT: u32> Interpolator
    for SpanSubdivAdaptor<I, SUBPIXEL_SHIFT>
{
    type Trf = I::Trf;
    const SUBPIXEL_SHIFT: u32 = SUBPIXEL_SHIFT;

    fn begin(&mut self, x: f64, y: f64, len: u32) {
        let mut len = len;

        self.pos = 1;
        self.src_x = iround(x * Self::SUBPIXEL_SCALE as f64) + Self::SUBPIXEL_SCALE as i32;
        self.src_y = y;
        self.len = len;
        if len > self.subdiv_size {
            len = self.subdiv_size;
        }
        self.interpolator.begin(x, y, len);
    }

    fn next(&mut self) {
        self.interpolator.next();
        if self.pos >= self.subdiv_size {
            let mut len = self.len;
            if len > self.subdiv_size {
                len = self.subdiv_size;
            }
            self.interpolator.resynchronize(
                (self.src_x as f64 / (1 << 8) as f64) + len as f64,
                self.src_y,
                len,
            );
            self.pos = 0;
        }
        self.src_x += (1 << 8);
        self.pos += 1;
        self.len -= 1;
    }

    fn coordinates(&self, x: &mut i32, y: &mut i32) {
        self.interpolator.coordinates(x, y)
    }

    fn local_scale(&self, x: &mut i32, y: &mut i32) {
        self.interpolator.local_scale(x, y)
    }
}
