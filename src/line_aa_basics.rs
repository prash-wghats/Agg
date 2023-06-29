use crate::basics::{iround, Saturation};
use crate::Coord;

#[repr(i32)]
pub enum LineSubpixel {
    Shift = 8,                       //----SUBPIXEL_SHIFT
    Scale = 1 << Self::Shift as i32, //----SUBPIXEL_SCALE
    Mask = Self::Scale as i32 - 1,   //----SUBPIXEL_MASK
}

pub const MAX_COORD: i32 = (1 << 28) - 1; //----LineSubpixel::MaxCoord
pub const MAX_LENGTH: i32 = 1 << (LineSubpixel::Shift as i32 + 10); //----MAX_LENGTH

#[repr(i32)]
pub enum LineMRSubpixel {
    Shift = 4,                       //----SUBPIXEL_SHIFT
    Scale = 1 << Self::Shift as i32, //----SUBPIXEL_SCALE
    Mask = Self::Scale as i32 - 1,   //----SUBPIXEL_MASK
}

//------------------------------------------------------------------line_mr
pub fn line_mr(x: i32) -> i32 {
    x >> (LineSubpixel::Shift as i32 - LineMRSubpixel::Shift as i32)
}

//-------------------------------------------------------------------line_hr
pub fn line_hr(x: i32) -> i32 {
    x << (LineSubpixel::Shift as i32 - LineMRSubpixel::Shift as i32)
}

//---------------------------------------------------------------line_dbl_hr
pub fn line_dbl_hr(x: i32) -> i32 {
    x << LineSubpixel::Shift as i32
}

//---------------------------------------------------------------LineCoord
pub struct LineCoord;
impl Coord for LineCoord {
    fn conv(x: f64) -> i32 {
        iround(x * LineSubpixel::Scale as i32 as f64)
    }
}

//-----------------------------------------------------------LineCoordSat
pub struct LineCoordSat;
impl Coord for LineCoordSat {
    fn conv(x: f64) -> i32 {
        Saturation::<MAX_COORD>::iround(x * LineSubpixel::Scale as i32 as f64)
    }
}

//==========================================================LineParameters
#[derive(Copy, Clone)]
pub struct LineParameters {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub dx: i32,
    pub dy: i32,
    pub sx: i32,
    pub sy: i32,
    pub vertical: bool,
    pub inc: i32,
    pub len: i32,
    pub octant: i32,
}

impl LineParameters {
    pub fn new_default() -> Self {
        Self {
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
            dx: 0,
            dy: 0,
            sx: 0,
            sy: 0,
            vertical: false,
            inc: 0,
            len: 0,
            octant: 0,
        }
    }
    pub fn new(x1_: i32, y1_: i32, x2_: i32, y2_: i32, len_: i32) -> LineParameters {
        let dx = (x2_ - x1_).abs();
        let dy = (y2_ - y1_).abs();
        let sx = if x2_ > x1_ { 1 } else { -1 };
        let sy = if y2_ > y1_ { 1 } else { -1 };
        let vertical = dy >= dx;
        let inc = if vertical { sy } else { sx };
        let octant = (sy & 4) | (sx & 2) | if vertical { 1 } else { 0 };
        LineParameters {
            x1: x1_,
            y1: y1_,
            x2: x2_,
            y2: y2_,
            dx: dx,
            dy: dy,
            sx: sx,
            sy: sy,
            vertical: vertical,
            inc: inc,
            len: len_,
            octant: octant,
        }
    }

    pub fn orthogonal_quadrant(&self) -> u8 {
        Self::ORTHOGONAL_QUADRANT[self.octant as usize]
    }

    pub fn diagonal_quadrant(&self) -> u8 {
        Self::DIAGONAL_QUADRANT[self.octant as usize]
    }

    pub fn same_orthogonal_quadrant(&self, lp: &LineParameters) -> bool {
        Self::ORTHOGONAL_QUADRANT[self.octant as usize]
            == Self::ORTHOGONAL_QUADRANT[lp.octant as usize]
    }

    pub fn same_diagonal_quadrant(&self, lp: &LineParameters) -> bool {
        Self::DIAGONAL_QUADRANT[self.octant as usize] == Self::DIAGONAL_QUADRANT[lp.octant as usize]
    }

    pub fn divide(&self, lp1: &mut LineParameters, lp2: &mut LineParameters) {
        let xmid = (self.x1 + self.x2) >> 1;
        let ymid = (self.y1 + self.y2) >> 1;
        let len2 = self.len >> 1;

        *lp1 = *self;
        *lp2 = *self;

        lp1.x2 = xmid;
        lp1.y2 = ymid;
        lp1.len = len2;
        lp1.dx = lp1.x2 - lp1.x1;
        lp1.dy = lp1.y2 - lp1.y1;

        lp2.x1 = xmid;
        lp2.y1 = ymid;
        lp2.len = len2;
        lp2.dx = lp2.x2 - lp2.x1;
        lp2.dy = lp2.y2 - lp2.y1;
    }

    // The number of the octant is determined as a 3-bit value as follows:
    // bit 0 = vertical flag
    // bit 1 = sx < 0
    // bit 2 = sy < 0
    //
    // [N] shows the number of the orthogonal quadrant
    // <M> shows the number of the diagonal quadrant
    //               <1>
    //   [1]          |          [0]
    //       . (3)011 | 001(1) .
    //         .      |      .
    //           .    |    .
    //             .  |  .
    //    (2)010     .|.     000(0)
    // <2> ----------.+.----------- <0>
    //    (6)110   .  |  .   100(4)
    //           .    |    .
    //         .      |      .
    //       .        |        .
    //         (7)111 | 101(5)
    //   [2]          |          [3]
    //               <3>
    //

    const ORTHOGONAL_QUADRANT: [u8; 8] = [0, 0, 1, 1, 3, 3, 2, 2];
    const DIAGONAL_QUADRANT: [u8; 8] = [0, 1, 2, 1, 0, 3, 2, 3];
}

//----------------------------------------------------------------bisectrix
pub fn bisectrix(l1: &LineParameters, l2: &LineParameters, x: &mut i32, y: &mut i32) {
    let k = (l2.len as f64) / (l1.len as f64);
    let mut tx = l2.x2 as f64 - ((l2.x1 - l1.x1) as f64) * k;
    let mut ty = l2.y2 as f64 - ((l2.y1 - l1.y1) as f64) * k;

    //All bisectrices must be on the right of the line
    //If the next point is on the left (l1 => l2.2)
    //then the bisectix should be rotated by 180 degrees.
    if ((l2.x2 - l2.x1) as f64) * ((l2.y1 - l1.y1) as f64)
        < ((l2.y2 - l2.y1) as f64) * ((l2.x1 - l1.x1) as f64) + 100.0
    {
        tx -= (tx - (l2.x1 as f64)) * 2.0;
        ty -= (ty - (l2.y1 as f64)) * 2.0;
    }

    // Check if the bisectrix is too short
    let dx = tx - (l2.x1 as f64);
    let dy = ty - (l2.y1 as f64);
    if ((dx * dx + dy * dy).sqrt() as i32) < LineSubpixel::Scale as i32 {
        *x = (l2.x1 + l2.x1 + (l2.y1 - l1.y1) + (l2.y2 - l2.y1)) >> 1;
        *y = (l2.y1 + l2.y1 - (l2.x1 - l1.x1) - (l2.x2 - l2.x1)) >> 1;
        return;
    }
    *x = iround(tx);
    *y = iround(ty);
}

//-------------------------------------------fix_degenerate_bisectrix_start
pub fn fix_degenerate_bisectrix_start(lp: &LineParameters, x: &mut i32, y: &mut i32) {
    let d = iround(
        ((*x - lp.x2) as f64 * (lp.y2 - lp.y1) as f64
            - (*y - lp.y2) as f64 * (lp.x2 - lp.x1) as f64)
            / lp.len as f64,
    );
    if d < LineSubpixel::Scale as i32 / 2 {
        *x = lp.x1 + (lp.y2 - lp.y1);
        *y = lp.y1 - (lp.x2 - lp.x1);
    }
}

//---------------------------------------------fix_degenerate_bisectrix_end
pub fn fix_degenerate_bisectrix_end(lp: &LineParameters, x: &mut i32, y: &mut i32) {
    let d = iround(
        ((*x - lp.x2) as f64 * (lp.y2 - lp.y1) as f64
            - (*y - lp.y2) as f64 * (lp.x2 - lp.x1) as f64)
            / lp.len as f64,
    );
    if d < LineSubpixel::Scale as i32 / 2 {
        *x = lp.x2 + (lp.y2 - lp.y1);
        *y = lp.y2 - (lp.x2 - lp.x1);
    }
}
