use crate::basics::uround;
use crate::dda_line::Dda2LineIp;
use crate::trans_perspective::{IteratorX, TransPerspective};
use crate::{Interpolator, Transformer};

// NOT TESTED

//===========================================SpanIpPerspExact
pub struct SpanIpPerspExact<const SUBPIXEL_SHIFT: u32 = 8> {
    trans_dir: TransPerspective,
    trans_inv: TransPerspective,
    iterator: IteratorX,
    scale_x: Dda2LineIp,
    scale_y: Dda2LineIp,
}

impl<const SUBPIXEL_SHIFT: u32> SpanIpPerspExact<SUBPIXEL_SHIFT> {
    const SUBPIXEL_SCALE: u32 = 1 << SUBPIXEL_SHIFT;
    const SUBPIXEL_SHIFT: u32 = SUBPIXEL_SHIFT;
}

impl<const SUBPIXEL_SHIFT: u32> SpanIpPerspExact<SUBPIXEL_SHIFT> {
    // Arbitrary quadrangle transformations
    pub fn new() -> Self {
        Self {
            trans_dir: TransPerspective::new(),
            trans_inv: TransPerspective::new(),
            iterator: IteratorX::new_default(),
            scale_x: Dda2LineIp::new(),
            scale_y: Dda2LineIp::new(),
        }
    }
    // Arbitrary quadrangle transformations
    pub fn new_quad_to_quad(src: &[f64], dst: &[f64]) -> Self {
        let mut p = Self::new();
        p.quad_to_quad(src, dst);
        p
    }

    // Direct transformations
    pub fn new_rect_to_quad(x1: f64, y1: f64, x2: f64, y2: f64, quad: &[f64]) -> Self {
        let mut p = Self::new();
        p.rect_to_quad(x1, y1, x2, y2, quad);
        p
    }

    // Reverse transformations
    pub fn new_quad_to_rect(quad: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let mut p = Self::new();
        p.quad_to_rect(quad, x1, y1, x2, y2);
        p
    }

    // Set the transformations using two arbitrary quadrangles.
    pub fn quad_to_quad(&mut self, src: &[f64], dst: &[f64]) {
        self.trans_dir.quad_to_quad(src, dst);
        self.trans_inv.quad_to_quad(dst, src);
    }

    // Set the direct transformations, i.e., rectangle -> quadrangle
    pub fn rect_to_quad(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, quad: &[f64]) {
        let src = [x1, y1, x2, y1, x2, y2, x1, y2];
        self.quad_to_quad(&src, quad);
    }

    // Set the reverse transformations, i.e., quadrangle -> rectangle
    pub fn quad_to_rect(&mut self, quad: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) {
        let dst = [x1, y1, x2, y1, x2, y2, x1, y2];
        self.quad_to_quad(quad, &dst);
    }

    // Check if the equations were solved successfully
    pub fn is_valid(&self) -> bool {
        self.trans_dir.is_valid(crate::trans_affine::AFFINE_EPSILON)
    }
    pub fn transform(&self, x: &mut f64, y: &mut f64) {
        self.trans_dir.transform(x, y);
    }
}

impl<const SUBPIXEL_SHIFT: u32> Interpolator for SpanIpPerspExact<SUBPIXEL_SHIFT> {
    type Trf = TransPerspective;
    const SUBPIXEL_SHIFT: u32 = SUBPIXEL_SHIFT;

    //----------------------------------------------------------------
    fn begin(&mut self, x: f64, y: f64, len: u32) {
        self.iterator = self.trans_dir.begin(x, y, 1.0);
        let xt = self.iterator.x;
        let yt = self.iterator.y;

        let mut dx;
        let mut dy;
        let delta = 1.0 / Self::SUBPIXEL_SCALE as f64;
        dx = xt + delta;
        dy = yt;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= x;
        dy -= y;
        let sx1 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;
        dx = xt;
        dy = yt + delta;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= x;
        dy -= y;
        let sy1 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        let x = x + len as f64;
        let mut xt = x;
        let mut yt = y;
        self.trans_dir.transform(&mut xt, &mut yt);

        dx = xt + delta;
        dy = yt;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= x;
        dy -= y;
        let sx2 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;
        dx = xt;
        dy = yt + delta;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= x;
        dy -= y;
        let sy2 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        self.scale_x = Dda2LineIp::new_fwd(sx1, sx2, len as i32);
        self.scale_y = Dda2LineIp::new_fwd(sy1, sy2, len as i32);
    }

    fn resynchronize(&mut self, xe: f64, ye: f64, len: u32) {
        let sx1 = self.scale_x.y();
        let sy1 = self.scale_y.y();

        let mut xt = xe;
        let mut yt = ye;
        self.trans_dir.transform(&mut xt, &mut yt);

        let delta = 1.0 / (Self::SUBPIXEL_SCALE as f64);
        let mut dx;
        let mut dy;

        dx = xt + delta;
        dy = yt;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= xe;
        dy -= ye;
        let sx2 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        dx = xt;
        dy = yt + delta;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= xe;
        dy -= ye;
        let sy2 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        self.scale_x = Dda2LineIp::new_fwd(sx1, sx2, len as i32);
        self.scale_y = Dda2LineIp::new_fwd(sy1, sy2, len as i32);
    }

    fn coordinates(&self, x: &mut i32, y: &mut i32) {
        *x = (self.iterator.x * Self::SUBPIXEL_SCALE as f64).round() as i32;
        *y = (self.iterator.y * Self::SUBPIXEL_SCALE as f64).round() as i32;
    }

    fn local_scale(&self, x: &mut i32, y: &mut i32) {
        *x = self.scale_x.y();
        *y = self.scale_y.y();
    }

    fn next(&mut self) {
        self.iterator.inc();
        self.scale_x.inc();
        self.scale_y.inc();
    }
}

//============================================SpanIpPerspLerp
pub struct SpanIpPerspLerp<const SUBPIXEL_SHIFT: u32 = 8> {
    trans_dir: TransPerspective,
    trans_inv: TransPerspective,
    coord_x: Dda2LineIp,
    coord_y: Dda2LineIp,
    scale_x: Dda2LineIp,
    scale_y: Dda2LineIp,
}

impl<const SUBPIXEL_SHIFT: u32> SpanIpPerspLerp<SUBPIXEL_SHIFT> {
    const SUBPIXEL_SCALE: u32 = 1 << SUBPIXEL_SHIFT;
    const SUBPIXEL_SHIFT: u32 = SUBPIXEL_SHIFT;
}

impl<const SUBPIXEL_SHIFT: u32> SpanIpPerspLerp<SUBPIXEL_SHIFT> {
    pub fn new() -> Self {
        SpanIpPerspLerp {
            trans_dir: TransPerspective::new(),
            trans_inv: TransPerspective::new(),
            coord_x: Dda2LineIp::new(),
            coord_y: Dda2LineIp::new(),
            scale_x: Dda2LineIp::new(),
            scale_y: Dda2LineIp::new(),
        }
    }

    // Arbitrary quadrangle transformations
    pub fn new_quad_to_quad(src: &[f64], dst: &[f64]) -> Self {
        let mut p = Self::new();
        p.quad_to_quad(src, dst);
        p
    }

    // Direct transformations
    pub fn new_rect_to_quad(x1: f64, y1: f64, x2: f64, y2: f64, quad: &[f64]) -> Self {
        let mut p = Self::new();
        p.rect_to_quad(x1, y1, x2, y2, quad);
        p
    }

    // Reverse transformations
    pub fn new_quad_to_rect(quad: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let mut p = Self::new();
        p.quad_to_rect(quad, x1, y1, x2, y2);
        p
    }
	
    pub fn quad_to_quad(&mut self, src: &[f64], dst: &[f64]) {
        self.trans_dir.quad_to_quad(src, dst);
        self.trans_inv.quad_to_quad(dst, src);
    }

    pub fn rect_to_quad(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, quad: &[f64]) {
        let src = [x1, y1, x2, y1, x2, y2, x1, y2];
        self.quad_to_quad(&src, quad);
    }

    pub fn quad_to_rect(&mut self, quad: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) {
        let dst = [x1, y1, x2, y1, x2, y2, x1, y2];
        self.quad_to_quad(quad, &dst);
    }

    pub fn is_valid(&self) -> bool {
        self.trans_dir.is_valid(crate::trans_affine::AFFINE_EPSILON)
    }

    pub fn transform(&self, x: &mut f64, y: &mut f64) {
        self.trans_dir.transform(x, y);
    }
}

impl<const SUBPIXEL_SHIFT: u32> Interpolator for SpanIpPerspLerp<SUBPIXEL_SHIFT> {
    type Trf = TransPerspective;
    const SUBPIXEL_SHIFT: u32 = SUBPIXEL_SHIFT;

    fn begin(&mut self, x: f64, y: f64, len: u32) {
        let mut xt = x;
        let mut yt = y;
        self.trans_dir.transform(&mut xt, &mut yt);
        let x1 = xt * Self::SUBPIXEL_SCALE as f64;
        let y1 = yt * Self::SUBPIXEL_SCALE as f64;

        let mut dx;
        let mut dy;
        let delta = 1.0 / Self::SUBPIXEL_SCALE as f64;

        dx = xt + delta;
        dy = yt;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= x;
        dy -= y;
        let sx1 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        dx = xt;
        dy = yt + delta;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= x;
        dy -= y;
        let sy1 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        let xe = x + len as f64;
        let ye = y;
        xt = xe;
        yt = ye;
        self.trans_dir.transform(&mut xt, &mut yt);
        let x2 = xt * Self::SUBPIXEL_SCALE as f64;
        let y2 = yt * Self::SUBPIXEL_SCALE as f64;

        dx = xt + delta;
        dy = yt;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= xe;
        dy -= ye;
        let sx2 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        dx = xt;
        dy = yt + delta;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= xe;
        dy -= ye;
        let sy2 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        self.coord_x = Dda2LineIp::new_fwd(x1 as i32, x2 as i32, len as i32);
        self.coord_y = Dda2LineIp::new_fwd(y1 as i32, y2 as i32, len as i32);
        self.scale_x = Dda2LineIp::new_fwd(sx1 as i32, sx2 as i32, len as i32);
        self.scale_y = Dda2LineIp::new_fwd(sy1 as i32, sy2 as i32, len as i32);
    }

    fn resynchronize(&mut self, xe: f64, ye: f64, len: u32) {
        let x1 = self.coord_x.y();
        let y1 = self.coord_y.y();
        let sx1 = self.scale_x.y();
        let sy1 = self.scale_y.y();

        let mut xt = xe;
        let mut yt = ye;
        self.trans_dir.transform(&mut xt, &mut yt);
        let x2 = xt * Self::SUBPIXEL_SCALE as f64;
        let y2 = yt * Self::SUBPIXEL_SCALE as f64;

        let delta = 1.0 / Self::SUBPIXEL_SCALE as f64;
        let mut dx;
        let mut dy;

        dx = xt + delta;
        dy = yt;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= xe;
        dy -= ye;
        let sx2 = uround(Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt())
            >> Self::SUBPIXEL_SHIFT;

        dx = xt;
        dy = yt + delta;
        self.trans_inv.transform(&mut dx, &mut dy);
        dx -= xe;
        dy -= ye;
        let sy2 = uround((Self::SUBPIXEL_SCALE as f64 / (dx * dx + dy * dy).sqrt()))
            >> Self::SUBPIXEL_SHIFT;

        self.coord_x = Dda2LineIp::new_fwd(x1, x2 as i32, len as i32);
        self.coord_y = Dda2LineIp::new_fwd(y1, y2 as i32, len as i32);
        self.scale_x = Dda2LineIp::new_fwd(sx1, sx2 as i32, len as i32);
        self.scale_y = Dda2LineIp::new_fwd(sy1, sy2 as i32, len as i32);
    }

    fn next(&mut self) {
        self.coord_x.inc();
        self.coord_y.inc();
        self.scale_x.inc();
        self.scale_y.inc();
    }

    fn coordinates(&self, x: &mut i32, y: &mut i32) {
        *x = self.coord_x.y();
        *y = self.coord_y.y();
    }

    fn local_scale(&self, x: &mut i32, y: &mut i32) {
        *x = self.scale_x.y();
        *y = self.scale_y.y();
    }
}
