use crate::basics::{FillingRule, PolySubpixelScale, *};
use crate::rasterizer_cells_aa::{Cell, RasterizerCellsAa, ScanlineHitTest};
use crate::rasterizer_sl_clip::RasterizerSlClipInt;
use crate::RasClip;
use crate::RasConv;
use crate::Rasterizer;
//use crate::gamma_functions::*;
use self::AaScale::*;
use crate::{AggPrimitive, RasterScanLine, Rasterizer0, Scanline, VertexSource};

// NOT TESTED

//===========================================================LayerOrder
#[derive(PartialEq, Eq)]
pub enum LayerOrder {
    UnSorted, //------UnSorted
    Direct,   //------Direct
    Inverse,  //------Inverse
}

macro_rules! from_i32 {
    ($v:expr) => {
        <Clip::ConvType as RasConv>::CoordType::from_i32($v)
    };
}

/*
//-----------------------------------------------------------cell_style_aa
// A pixel cell. There're no constructors defined and it was done
// intentionally in order to avoid extra overhead when allocating an
// array of cells.
#[derive(Copy, Clone)]
pub struct cell_style_aa {
    pub x: i32,
    pub y: i32,
    pub cover: i32,
    pub area: i32,
    pub left: i16,
    pub right: i16,
}

impl cell_style_aa {
    pub fn initial(&mut self) {
        self.x = 0x7FFFFFFF;
        self.y = 0x7FFFFFFF;
        self.cover = 0;
        self.area = 0;
        self.left = -1;
        self.right = -1;
    }

    pub fn style(&mut self, c: &cell_style_aa) {
        self.left = c.left;
        self.right = c.right;
    }

    pub fn not_equal(&self, ex: i32, ey: i32, c: &cell_style_aa) -> i32 {
        (ex - self.x) | (ey - self.y) | (self.left - c.left) | (self.right - c.right)
    }
}
*/
struct CellDummy;

impl crate::CellFn for CellDummy {
    fn style(&mut self, me: &mut Cell, c: &Cell) {
        me.left = c.left;
        me.right = c.right;
    }

    fn not_equal(&self, me: &Cell, ex: i32, ey: i32, c: &Cell) -> i32 {
        (ex.wrapping_sub(me.x))
            | (ey.wrapping_sub(me.y))
            | (me.left.wrapping_sub(c.left)) as i32
            | (me.right.wrapping_sub(c.right)) as i32
    }
}

#[repr(i32)]
enum AaScale {
    Shift = 8,
    Scale = 1 << Shift as i32,
    Mask = Scale as i32 - 1,
    Scale2 = Scale as i32 * 2,
    Mask2 = Scale2 as i32 - 1,
}

#[derive(Default, Clone, Copy)]
struct StyleInfo {
    start_cell: u32,
    num_cells: u32,
    last_x: i32,
}

#[derive(Default, Clone, Copy)]
struct CellInfo {
    x: i32,
    area: i32,
    cover: i32,
}

//==================================================RasterizerCompoundAa
pub struct RasterizerCompoundAa<Clip: RasClip = RasterizerSlClipInt> {
    outline: RasterizerCellsAa<CellDummy>,
    clipper: Clip,
    filling_rule: FillingRule,
    layer_order: LayerOrder,
    styles: Vec<StyleInfo>, // Active Styles
    ast: Vec<u32>,          // Active Style Table (unique values)
    asm: Vec<u8>,           // Active Style Mask
    cells: Vec<CellInfo>,
    cover_buf: Vec<CoverType>,
    master_alpha: Vec<u32>,
    min_style: i32,
    max_style: i32,
    start_x: <Clip::ConvType as RasConv>::CoordType,
    start_y: <Clip::ConvType as RasConv>::CoordType,
    scan_y: i32,
    sl_start: i32,
    sl_len: u32,
}

impl<Clip: RasClip> RasterScanLine for RasterizerCompoundAa<Clip> {
    fn reset(&mut self) {
        self.outline.reset();
        self.min_style = 0x7FFFFFFF;
        self.max_style = -0x7FFFFFFF;
        self.scan_y = 0x7FFFFFFF;
        self.sl_start = 0;
        self.sl_len = 0;
    }
    fn apply_gamma(&self, _cover: usize) -> u32 {
        todo!()
    }

    fn add_path<VS: VertexSource>(&mut self, vs: &mut VS, path_id: u32) {
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;

        let mut cmd: u32;
        vs.rewind(path_id);
        if self.outline.sorted() {
            self.reset();
        }
        loop {
            cmd = vs.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            self.add_vertex(x, y, cmd);
        }
    }

    fn min_x(&self) -> i32 {
        self.outline.min_x()
    }
    fn min_y(&self) -> i32 {
        self.outline.min_y()
    }
    fn max_x(&self) -> i32 {
        self.outline.max_x()
    }
    fn max_y(&self) -> i32 {
        self.outline.max_y()
    }

    fn rewind_scanlines(&mut self) -> bool {
        self.outline.sort_cells();
        if self.outline.total_cells() == 0 {
            return false;
        }
        if self.max_style < self.min_style {
            return false;
        }
        self.scan_y = self.outline.min_y();
        self.styles.resize(
            (self.max_style - self.min_style + 2 + 128) as usize,
            StyleInfo {
                ..Default::default()
            },
        );
        self.allocate_master_alpha();
        true
    }

    // Sweeps one scanline with one style index. The style ID can be
    // determined by calling style().
    fn sweep_scanline_with_style<S: Scanline>(&mut self, sl: &mut S, style_idx: i32) -> bool {
        let mut style_idx = style_idx;
        let scan_y = self.scan_y - 1;
        if scan_y > self.outline.max_y() {
            return false;
        }

        sl.reset_spans();

        let mut master_alpha = Mask as u32;

        if style_idx < 0 {
            style_idx = 0;
        } else {
            style_idx += 1;
            master_alpha = self.master_alpha
                [(self.ast[style_idx as usize] + self.min_style as u32 - 1) as usize];
        }

        let st = &self.styles[self.ast[style_idx as usize] as usize];

        let mut num_cells = st.num_cells;
        let mut i = st.start_cell as usize;

        let mut cover = 0;
        while num_cells > 0 {
            num_cells -= 1;
            let mut alpha;
            let mut x = self.cells[i].x;
            let area = self.cells[i].area;

            cover += self.cells[i].cover;

            i += 1;

            if area != 0 {
                alpha = self.calculate_alpha(
                    (cover << (PolySubpixelScale::Shift as i32 + 1)) - area,
                    master_alpha,
                );
                sl.add_cell(x, alpha);
                x += 1;
            }

            if num_cells > 0 && self.cells[i].x > x {
                alpha = self
                    .calculate_alpha(cover << (PolySubpixelScale::Shift as i32 + 1), master_alpha);
                if alpha != 0 {
                    sl.add_span(x, (self.cells[i].x - x) as u32, alpha);
                }
            }
        }

        if sl.num_spans() == 0 {
            return false;
        }
        sl.finalize(scan_y);
        return true;
    }
}

impl<Clip: RasClip> Rasterizer0 for RasterizerCompoundAa<Clip> {
    fn scanline_start(&self) -> i32 {
        self.sl_start
    }
    fn scanline_length(&self) -> u32 {
        self.sl_len
    }
    // Returns the number of styles
    fn sweep_styles(&mut self) -> u32 {
        loop {
            if self.scan_y > self.outline.max_y() {
                return 0;
            }
            let mut num_cells = self.outline.scanline_num_cells(self.scan_y as u32);
            let mut cells = self.outline.scanline_cells(self.scan_y as u32);
            let num_styles = self.max_style - self.min_style + 2;
            let mut curr_cell;
            let mut style_id;
            let mut style;
            let mut cell;

			self.cells.clear();
            self.cells.resize(
                num_cells as usize * 2,
                CellInfo {
                    ..Default::default()
                },
            ); // Each cell can have two styles
			self.cells.reserve(256);
            
            self.asm.clear();
			let sz = ((num_styles as usize + 7) >> 3);
            self.asm.resize(sz, 0);
			self.asm.reserve(8);
			
			self.ast.clear();
            self.ast.reserve(num_styles as usize + 64);
            
            //self.asm.zero();
            unsafe {
                if num_cells > 0 {
                    // Pre-add zero (for no-fill style, that is, -1).
                    // We need that to ensure that the "-1 style" would go first.
                    self.asm[0] |= 1;
                    self.ast.push(0);
                    style = &mut self.styles[0];
                    style.start_cell = 0;
                    style.num_cells = 0;
                    style.last_x = -0x7FFFFFFF;

                    self.sl_start = (*cells[0]).x;
                    self.sl_len = ((*cells[num_cells as usize - 1]).x - self.sl_start + 1) as u32;
                    let mut vec: Vec<(i16, i16)> = vec![];
                    for i in 0..num_cells as usize {
                        vec.push(((*cells[i]).left, (*cells[i]).right));
                        //vec.push();
                    }
                    for i in 0..num_cells as usize {
                        //{let curr_cell = cells[num_cells as usize];}
                        self.add_style(vec[i].0 as i32);
                        self.add_style(vec[i].1 as i32);
                    }

                    // Convert the Y-histogram into the array of starting indexes
                    let mut i = 0;
                    let mut start_cell = 0;
                    while i < self.ast.len() {
                        let st = &mut self.styles[self.ast[i] as usize];
                        let v = st.start_cell;
                        st.start_cell = start_cell;
                        start_cell += v;
                        i += 1;
                    }

                    cells = self.outline.scanline_cells(self.scan_y as u32);
                    num_cells = self.outline.scanline_num_cells(self.scan_y as u32);

                    for i in 0..num_cells as usize {
                        //num_cells -= 1;
                        curr_cell = cells[i];
                        style_id = if (*curr_cell).left < 0 {
                            0
                        } else {
                            (*curr_cell).left as i32 - self.min_style + 1
                        };

                        style = &mut self.styles[style_id as usize];
                        if (*curr_cell).x == style.last_x {
                            cell =
                                &mut self.cells[(style.start_cell + style.num_cells - 1) as usize];
                            cell.area += (*curr_cell).area;
                            cell.cover += (*curr_cell).cover;
                        } else {
                            cell = &mut self.cells[(style.start_cell + style.num_cells) as usize];
                            cell.x = (*curr_cell).x;
                            cell.area = (*curr_cell).area;
                            cell.cover = (*curr_cell).cover;
                            style.last_x = (*curr_cell).x;
                            style.num_cells += 1;
                        }

                        style_id = if (*curr_cell).right < 0 {
                            0
                        } else {
                            (*curr_cell).right as i32 - self.min_style + 1
                        };

                        style = &mut self.styles[style_id as usize];
                        if (*curr_cell).x == style.last_x {
                            cell =
                                &mut self.cells[(style.start_cell + style.num_cells - 1) as usize];
                            cell.area -= (*curr_cell).area;
                            cell.cover -= (*curr_cell).cover;
                        } else {
                            cell = &mut self.cells[(style.start_cell + style.num_cells) as usize];
                            cell.x = (*curr_cell).x;
                            cell.area = -(*curr_cell).area;
                            cell.cover = -(*curr_cell).cover;
                            style.last_x = (*curr_cell).x;
                            style.num_cells += 1;
                        }
                    }
                }
            }
            if self.ast.len() > 1 {
                break;
            }
            self.scan_y += 1;
        }
        self.scan_y += 1;

        if self.layer_order != LayerOrder::UnSorted {
            //let ra = RangeAdaptor::new(&self.ast, 1, self.ast.size() - 1);
            let ra = &mut self.ast[1..];
            if self.layer_order == LayerOrder::Direct {
                ra.sort_by(|a, b| b.cmp(a));
                //quick_sort(ra, unsigned_greater);
            } else {
                ra.sort_by(|a, b| a.cmp(b));
                //quick_sort(ra, unsigned_less);
            }
        }

        self.ast.len() as u32 - 1
    }

    fn style(&self, idx: u32) -> u32 {
        self.ast[idx as usize + 1] + self.min_style as u32 - 1
    }

    fn allocate_cover_buffer(&mut self, len: u32) -> &mut [CoverType] {
        self.cover_buf.resize(len as usize + 256, 0);
        &mut self.cover_buf[0..len as usize]
    }
}

impl<Clip: RasClip> RasterizerCompoundAa<Clip> {
    pub fn move_to_d(&mut self, x: f64, y: f64) {
        if self.outline.sorted() {
            self.reset();
        }
        self.start_x = Clip::ConvType::upscale(x);
        self.start_y = Clip::ConvType::upscale(y);
        self.clipper.move_to(self.start_x, self.start_y);
    }

    pub fn line_to_d(&mut self, x: f64, y: f64) {
        self.clipper.line_to(
            &mut self.outline,
            Clip::ConvType::upscale(x),
            Clip::ConvType::upscale(y),
        );
    }

    pub fn new() -> Self {
        let dum = CellDummy;
        RasterizerCompoundAa {
            outline: RasterizerCellsAa::<CellDummy>::new(dum),
            clipper: Clip::new(),
            filling_rule: FillingRule::FillNonZero,
            layer_order: LayerOrder::Direct,
            styles: Vec::new(),
            ast: Vec::new(),
            asm: Vec::new(),
            cells: Vec::new(),
            cover_buf: Vec::new(),
            master_alpha: Vec::new(),
            min_style: 0x7FFFFFFF,
            max_style: -0x7FFFFFFF,
            start_x: from_i32!(0),
            start_y: from_i32!(0),
            scan_y: 0x7FFFFFFF,
            sl_start: 0,
            sl_len: 0,
        }
    }

    pub fn min_style(&self) -> i32 {
        self.min_style
    }
    pub fn max_style(&self) -> i32 {
        self.max_style
    }

    #[inline]
    fn calculate_alpha(&self, area: i32, master_alpha: u32) -> u32 {
        let mut cover = area >> (PolySubpixelScale::Shift as u32 * 2 + 1 - Shift as u32);
        cover = if cover < 0 { -cover } else { cover };
        if self.filling_rule == FillingRule::FillEvenOdd {
            cover &= Mask2 as i32;
            if cover > Scale as i32 {
                cover = Scale2 as i32 - cover
            }
        }

        if cover > Mask as i32 {
            cover = Mask as i32;
        }
        return (cover as u32 * master_alpha + Mask as u32) >> Shift as u32;
    }

    pub fn filling_rule(&mut self, filling_rule: FillingRule) {
        self.filling_rule = filling_rule;
    }

    pub fn set_layer_order(&mut self, order: LayerOrder) {
        self.layer_order = order;
    }

    pub fn clip_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.reset();
        self.clipper.clip_box(
            Clip::ConvType::upscale(x1),
            Clip::ConvType::upscale(y1),
            Clip::ConvType::upscale(x2),
            Clip::ConvType::upscale(y2),
        );
    }
    /*

        pub fn reset_clipping(&mut self) {
            self.reset();
            self.clipper.reset_clipping();
        }
    */

    pub fn set_styles(&mut self, left: i32, right: i32) {
        let mut cell = Cell::new();
        cell.initial();
        cell.left = left as i16;
        cell.right = right as i16;
        self.outline.style(&cell);
        if left >= 0 && left < self.min_style {
            self.min_style = left;
        }
        if left >= 0 && left > self.max_style {
            self.max_style = left;
        }
        if right >= 0 && right < self.min_style {
            self.min_style = right;
        }
        if right >= 0 && right > self.max_style {
            self.max_style = right;
        }
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        if self.outline.sorted() {
            self.reset();
        }
        self.start_x = Clip::ConvType::downscale(x);
        self.start_y = Clip::ConvType::downscale(y);
        self.clipper.move_to(self.start_x, self.start_y);
    }

    pub fn line_to(&mut self, x: i32, y: i32) {
        self.clipper.line_to(
            &mut self.outline,
            Clip::ConvType::downscale(x),
            Clip::ConvType::downscale(y),
        );
    }

    pub fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        if is_move_to(cmd) {
            self.move_to_d(x, y);
        } else if is_vertex(cmd) {
            self.line_to_d(x, y);
        } else if is_close(cmd) {
            self.clipper
                .line_to(&mut self.outline, self.start_x, self.start_y);
        }
    }

    pub fn edge(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if self.outline.sorted() {
            self.reset();
        }
        self.clipper
            .move_to(Clip::ConvType::downscale(x1), Clip::ConvType::downscale(y1));
        self.clipper.line_to(
            &mut self.outline,
            Clip::ConvType::downscale(x2),
            Clip::ConvType::downscale(y2),
        );
    }

    pub fn edge_d(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        if self.outline.sorted() {
            self.reset();
        }
        self.clipper
            .move_to(Clip::ConvType::upscale(x1), Clip::ConvType::upscale(y1));
        self.clipper.line_to(
            &mut self.outline,
            Clip::ConvType::upscale(x2),
            Clip::ConvType::upscale(y2),
        );
    }

    pub fn sort(&mut self) {
        self.outline.sort_cells();
    }

    pub fn add_style(&mut self, style_id: i32) {
        let mut style_id = style_id;
        if style_id < 0 {
            style_id = 0;
        } else {
            style_id -= self.min_style - 1;
        }

        let nbyte = style_id >> 3;
        let mask = 1 << (style_id & 7);

        let style = &mut self.styles[style_id as usize];
        if (self.asm[nbyte as usize] & mask) == 0 {
            self.ast.push(style_id as u32);
            self.asm[nbyte as usize] |= mask;
            style.start_cell = 0;
            style.num_cells = 0;
            style.last_x = -0x7FFFFFFF;
        }
        style.start_cell += 1;
    }

    pub fn navigate_scanline(&mut self, y: i32) -> bool {
        self.outline.sort_cells();
        if self.outline.total_cells() == 0 {
            return false;
        }
        if self.max_style < self.min_style {
            return false;
        }
        if y < self.outline.min_y() || y > self.outline.max_y() {
            return false;
        }
        self.scan_y = y;
        self.styles.resize(
            (self.max_style - self.min_style + 2 + 128) as usize,
            StyleInfo {
                ..Default::default()
            },
        );
        self.allocate_master_alpha();
        return true;
    }

    pub fn hit_test(&mut self, tx: i32, ty: i32) -> bool {
        if !self.navigate_scanline(ty) {
            return false;
        }

        let num_styles = self.sweep_styles();
        if num_styles <= 0 {
            return false;
        }

        let mut sl = ScanlineHitTest::new(tx);
        self.sweep_scanline_with_style(&mut sl, -1);
        return sl.hit();
    }

    fn allocate_master_alpha(&mut self) {
        while (self.master_alpha.len() as i32) <= self.max_style {
            self.master_alpha.push(Mask as u32);
        }
    }

    pub fn set_master_alpha(&mut self, style: i32, alpha: f64) {
        if style >= 0 {
            while (self.master_alpha.len() as i32) <= style {
                self.master_alpha.push(Mask as u32);
            }
            self.master_alpha[style as usize] = uround(alpha * Mask as u32 as f64) as u32;
        }
    }
}
