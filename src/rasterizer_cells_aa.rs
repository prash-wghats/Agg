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
//
// The author gratefully acknowleges the support of David Turner,
// Robert Wilhelm, and Werner Lemberg - the authors of the FreeType
// libray - in producing this work. See http://www.freetype.org for details.
//
//----------------------------------------------------------------------------
// Contact: mcseem@antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------
//
// Adaptation for 32-bit screen coordinates has been sponsored by
// Liberty Technology Systems, Inc., visit http://lib-sys.com
//
// Liberty Technology Systems, Inc. is the provider of
// PostScript and PDF technology for software developers.
//
//----------------------------------------------------------------------------

use crate::basics::{PolySubpixelScale, Span};
use crate::CellFn;
use crate::{Rasterizer, Scanline};

//-----------------------------------------------------RasterizerCellsAa
// An internal class that implements the main rasterization algorithm.
// Used in the rasterizer. Should not be used direcly.

const CELL_BLOCK_SHIFT: u32 = 12;
const CELL_BLOCK_SIZE: u32 = 1 << CELL_BLOCK_SHIFT;
const CELL_BLOCK_MASK: u32 = CELL_BLOCK_SIZE - 1;
const CELL_BLOCK_POOL: u32 = 256;
const CELL_BLOCK_LIMIT: u32 = 1024;

//-----------------------------------------------------------------cell_aa
// A pixel cell. There're no constructors defined and it was done
// intentionally in order to avoid extra overhead when allocating an
// array of cells.
#[derive(Clone, Copy)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub cover: i32,
    pub area: i32,
    pub left: i16,
    pub right: i16,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            x: 0x7FFFFFFF,
            y: 0x7FFFFFFF,
            cover: 0,
            area: 0,
            left: -1,
            right: -1,
        }
    }
    pub fn initial(&mut self) {
        self.x = 0x7FFFFFFF;
        self.y = 0x7FFFFFFF;
        self.cover = 0;
        self.area = 0;
        self.left = -1;
        self.right = -1;
    }
}

#[derive(Clone, Copy)]
struct SortedY {
    start: u32,
    num: u32,
}

pub struct RasterizerCellsAa<Ce: CellFn> {
    m_num_blocks: u32,
    m_max_blocks: u32,
    m_curr_block: u32,
    m_num_cells: u32,
    m_cells: Vec<Vec<Cell>>,
    m_curr_cell_ptr: *mut Cell,
    m_sorted_cells: Vec<*mut Cell>,
    m_sorted_y: Vec<SortedY>,
    m_curr_cell: Cell,
    m_style_cell: Cell,
    m_min_x: i32,
    m_min_y: i32,
    m_max_x: i32,
    m_max_y: i32,
    m_sorted: bool,
    m_dummy: Ce,
}

impl<Ce: CellFn> RasterizerCellsAa<Ce> {
    pub fn new(cell: Ce) -> Self {
        Self {
            m_num_blocks: 0,
            m_max_blocks: 0,
            m_curr_block: 0,
            m_num_cells: 0,
            m_cells: vec![],
            m_curr_cell_ptr: std::ptr::null_mut(),
            m_sorted_cells: Vec::<*mut Cell>::new(),
            m_sorted_y: Vec::<SortedY>::new(),
            m_min_x: 0x7FFFFFFF,
            m_min_y: 0x7FFFFFFF,
            m_max_x: -0x7FFFFFFF,
            m_max_y: -0x7FFFFFFF,
            m_sorted: false,
            m_curr_cell: Cell::new(),
            m_style_cell: Cell::new(),
            m_dummy: cell,
        }
    }

    pub fn min_x(&self) -> i32 {
        self.m_min_x
    }

    pub fn min_y(&self) -> i32 {
        self.m_min_y
    }

    pub fn max_x(&self) -> i32 {
        self.m_max_x
    }

    pub fn max_y(&self) -> i32 {
        self.m_max_y
    }

    pub fn total_cells(&self) -> u32 {
        self.m_num_cells
    }

    pub fn scanline_num_cells(&self, y: u32) -> u32 {
        self.m_sorted_y[(y.wrapping_sub(self.m_min_y as u32)) as usize].num
    }

    pub fn scanline_cells(&self, y: u32) -> &[*mut Cell] {
        &self.m_sorted_cells
            [self.m_sorted_y[(y.wrapping_sub(self.m_min_y as u32)) as usize].start as usize..]
    }

    pub fn sorted(&self) -> bool {
        self.m_sorted
    }

    pub fn style(&mut self, style_cell: &Cell) {
        self.m_dummy.style(&mut self.m_style_cell, style_cell);
    }

    fn add_curr_cell(&mut self) {
        if (self.m_curr_cell.area | self.m_curr_cell.cover) != 0 {
            if (self.m_num_cells & CELL_BLOCK_MASK) == 0 {
                if self.m_num_blocks >= CELL_BLOCK_LIMIT {
                    return;
                }
                self.allocate_block();
            }
            unsafe {
                self.m_cells[self.m_num_blocks as usize - 1].push(self.m_curr_cell);
                //*self.m_curr_cell_ptr = self.m_curr_cell;
                self.m_curr_cell_ptr = self.m_curr_cell_ptr.offset(1);
            }
            self.m_num_cells += 1;
        }
    }

    pub fn allocate_block(&mut self) {
        if self.m_curr_block >= self.m_num_blocks {
            if self.m_num_blocks >= self.m_max_blocks {
                self.m_cells.reserve(CELL_BLOCK_POOL as usize);
                self.m_max_blocks += CELL_BLOCK_POOL;
            }

            //self.m_cells[self.m_num_blocks as usize] =
            self.m_cells
                .push(Vec::<Cell>::with_capacity(CELL_BLOCK_SIZE as usize));
            self.m_num_blocks += 1;
        }
        self.m_curr_cell_ptr = self.m_cells[self.m_curr_block as usize].as_mut_ptr();
        self.m_curr_block += 1;
    }

    fn set_curr_cell(&mut self, x: i32, y: i32) {
        if self
            .m_dummy
            .not_equal(&self.m_curr_cell, x, y, &self.m_style_cell)
            != 0
        {
            self.add_curr_cell();
            self.m_dummy
                .style(&mut self.m_curr_cell, &self.m_style_cell);
            (self.m_curr_cell.x, self.m_curr_cell.y) = (x, y);
            (self.m_curr_cell.area, self.m_curr_cell.cover) = (0, 0);
        }
    }

    pub fn sort_cells(&mut self) {
        if self.m_sorted {
            return;
        }

        self.add_curr_cell();
        (self.m_curr_cell.x, self.m_curr_cell.y) = (0x7FFFFFFF, 0x7FFFFFFF);
        (self.m_curr_cell.area, self.m_curr_cell.cover) = (0, 0);

        if self.m_num_cells == 0 {
            return;
        }

        // Allocate the array of cell pointers
        //self.m_sorted_cells.reserve(self.m_num_cells as usize);
        self.m_sorted_cells
            .resize(self.m_num_cells as usize + 16, std::ptr::null_mut());
        // Allocate and zero the Y array
        //self.m_sorted_y
        //    .reserve((self.m_max_y - self.m_min_y + 1) as usize);
        self.m_sorted_y.clear();
        self.m_sorted_y.resize(
            (self.m_max_y - self.m_min_y + 1) as usize,
            SortedY { start: 0, num: 0 },
        );

        // Create the Y-histogram (count the numbers of cells for each Y)
        for v in &self.m_cells {
            for c in v {
                self.m_sorted_y[(c.y - self.m_min_y) as usize].start += 1;
            }
        }

        // Convert the Y-histogram into the array of starting indexes
        let mut start = 0;
        for i in 0..self.m_sorted_y.len() {
            let v = self.m_sorted_y[i].start;
            self.m_sorted_y[i].start = start;
            start += v;
        }

        // Fill the cell pointer array sorted by Y
        for v in &self.m_cells {
            for c in v {
                let curr_y = &mut self.m_sorted_y[(c.y - self.m_min_y) as usize];
                self.m_sorted_cells[curr_y.start as usize + curr_y.num as usize] =
                    c as *const Cell as *mut Cell;
                curr_y.num += 1;
            }
        }

        // Finally arrange the X-arrays
        for i in 0..self.m_sorted_y.len() {
            let curr_y = &self.m_sorted_y[i];
            if curr_y.num > 0 {
                let slice = &mut self.m_sorted_cells
                    [curr_y.start as usize..(curr_y.start + curr_y.num) as usize];
                slice.sort_by(|a, b| {
                    let aa = *a;
                    let bb = *b;
                    unsafe { (*aa).x.cmp(&(*bb).x) }
                });
                let _i = slice[0];
                /*self.qsort_cells(
                    self.m_sorted_cells.as_mut_ptr().offset(curr_y.start as isize),
                    curr_y.num,
                );*/
            }
        }
        self.m_sorted = true;
    }
}

impl<Ce: CellFn> Rasterizer for RasterizerCellsAa<Ce> {
    fn reset(&mut self) {
        self.m_num_cells = 0;
        self.m_curr_block = 0;
        for i in 0..self.m_num_blocks as usize {
            self.m_cells[i].clear()
        }
        self.m_curr_cell = Cell::new();
        self.m_style_cell = Cell::new();
        self.m_sorted = false;
        self.m_min_x = 0x7FFFFFFF;
        self.m_min_y = 0x7FFFFFFF;
        self.m_max_x = -0x7FFFFFFF;
        self.m_max_y = -0x7FFFFFFF;
    }

    #[inline]
    fn render_hline(&mut self, ey: i32, x1: i32, y1_: i32, x2: i32, y2: i32) {
        let mut ex1 = x1 >> PolySubpixelScale::Shift as i32;
        let ex2 = x2 >> PolySubpixelScale::Shift as i32;
        let fx1 = x1 & PolySubpixelScale::Mask as i32;
        let fx2 = x2 & PolySubpixelScale::Mask as i32;
        let mut y1 = y1_;
        let mut delta: i32;
        let mut p: i32;
        let mut first: i32;
        let mut dx: i32;
        let mut incr: i32;
        let mut lift: i32;
        let mut mod_: i32;
        let mut rem: i32;

        //trivial case. Happens often
        if y1 == y2 {
            self.set_curr_cell(ex2, ey);
            return;
        }

        //everything is located in a single cell.  That is easy!
        if ex1 == ex2 {
            delta = y2 - y1;
            self.m_curr_cell.cover += delta;
            self.m_curr_cell.area += (fx1 + fx2) * delta;
            return;
        }

        //ok, we'll have to render a run of adjacent cells on the same
        //hline...
        p = (PolySubpixelScale::Scale as i32 - fx1) * (y2 - y1);
        first = PolySubpixelScale::Scale as i32;
        incr = 1;

        dx = x2.wrapping_sub(x1);

        if dx < 0 {
            p = fx1 * (y2 - y1);
            first = 0;
            incr = -1;
            dx = -dx;
        }

        delta = p / dx;
        mod_ = p % dx;

        if mod_ < 0 {
            delta -= 1;
            mod_ += dx;
        }

        self.m_curr_cell.cover += delta;
        self.m_curr_cell.area += (fx1 + first) * delta;

        ex1 += incr;
        self.set_curr_cell(ex1, ey);
        y1 += delta;

        if ex1 != ex2 {
            p = PolySubpixelScale::Scale as i32 * (y2 - y1 + delta);
            lift = p / dx;
            rem = p % dx;

            if rem < 0 {
                lift -= 1;
                rem += dx;
            }

            mod_ -= dx;

            while ex1 != ex2 {
                delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dx;
                    delta += 1;
                }

                self.m_curr_cell.cover += delta;
                self.m_curr_cell.area += PolySubpixelScale::Scale as i32 * delta;
                y1 = y1.wrapping_add(delta);
                ex1 += incr;
                self.set_curr_cell(ex1, ey);
            }
        }
        delta = y2 - y1;
        self.m_curr_cell.cover += delta;
        self.m_curr_cell.area += (fx2 + PolySubpixelScale::Scale as i32 - first) * delta;
    }

    fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let dx = x2.wrapping_sub(x1);

        const DX_LIMIT: i32 = 16384 << PolySubpixelScale::Shift as i32;

        if dx >= DX_LIMIT || dx <= -DX_LIMIT {
            let cx = (x1 + x2) >> 1;
            let cy = (y1 + y2) >> 1;
            self.line(x1, y1, cx, cy);
            self.line(cx, cy, x2, y2);
        }

        let mut dy = y2.wrapping_sub(y1);
        let ex1 = x1 >> PolySubpixelScale::Shift as i32;
        let ex2 = x2 >> PolySubpixelScale::Shift as i32;
        let mut ey1 = y1 >> PolySubpixelScale::Shift as i32;
        let ey2 = y2 >> PolySubpixelScale::Shift as i32;
        let fy1 = y1 & PolySubpixelScale::Mask as i32;
        let fy2 = y2 & PolySubpixelScale::Mask as i32;

        let mut x_from;
        let mut x_to;
        let mut p;
        let mut rem;
        let mut mod_;
        let mut lift;
        let mut delta;
        let mut first;
        let mut incr;

        if ex1 < self.m_min_x {
            self.m_min_x = ex1;
        }
        if ex1 > self.m_max_x {
            self.m_max_x = ex1;
        }
        if ey1 < self.m_min_y {
            self.m_min_y = ey1;
        }
        if ey1 > self.m_max_y {
            self.m_max_y = ey1;
        }
        if ex2 < self.m_min_x {
            self.m_min_x = ex2;
        }
        if ex2 > self.m_max_x {
            self.m_max_x = ex2;
        }
        if ey2 < self.m_min_y {
            self.m_min_y = ey2;
        }
        if ey2 > self.m_max_y {
            self.m_max_y = ey2;
        }

        self.set_curr_cell(ex1, ey1);

        //everything is on a single hline
        if ey1 == ey2 {
            self.render_hline(ey1, x1, fy1, x2, fy2);
            return;
        }

        //Vertical line - we have to calculate start and end cells,
        //and then - the common values of the area and coverage for
        //all cells of the line. We know exactly there's only one
        //cell, so, we don't have to call render_hline().
        incr = 1;
        if dx == 0 {
            let ex = x1 >> PolySubpixelScale::Shift as i32;
            let two_fx = (x1 - (ex << PolySubpixelScale::Shift as i32)) << 1;
            let area;

            first = PolySubpixelScale::Scale as i32;
            if dy < 0 {
                first = 0;
                incr = -1;
            }

            //x_from = x1;

            //render_hline(ey1, x_from, fy1, x_from, first);
            delta = first - fy1;
            self.m_curr_cell.cover += delta;
            self.m_curr_cell.area += two_fx * delta;

            ey1 += incr;
            self.set_curr_cell(ex, ey1);

            delta = first + first - PolySubpixelScale::Scale as i32;
            area = two_fx * delta;
            while ey1 != ey2 {
                //render_hline(ey1, x_from, PolySubpixelScale::Scale as i32 - first, x_from, first);
                self.m_curr_cell.cover = delta;
                self.m_curr_cell.area = area;
                ey1 += incr;
                self.set_curr_cell(ex, ey1);
            }
            //render_hline(ey1, x_from, PolySubpixelScale::Scale as i32 - first, x_from, fy2);
            delta = fy2 - PolySubpixelScale::Scale as i32 + first;
            self.m_curr_cell.cover += delta;
            self.m_curr_cell.area += two_fx * delta;
            return;
        }

        //ok, we have to render several hlines
        p = (PolySubpixelScale::Scale as i32 - fy1) * dx;
        first = PolySubpixelScale::Scale as i32;

        if dy < 0 {
            p = fy1 * dx;
            first = 0;
            incr = -1;
            dy = -dy;
        }

        delta = p / dy;
        mod_ = p % dy;

        if mod_ < 0 {
            delta -= 1;
            mod_ += dy;
        }

        x_from = x1.wrapping_add(delta);
        self.render_hline(ey1, x1, fy1, x_from, first);

        ey1 += incr;
        self.set_curr_cell(x_from >> PolySubpixelScale::Shift as i32, ey1);

        if ey1 != ey2 {
            p = PolySubpixelScale::Scale as i32 * dx;
            lift = p / dy;
            rem = p % dy;

            if rem < 0 {
                lift -= 1;
                rem += dy;
            }
            mod_ -= dy;

            while ey1 != ey2 {
                delta = lift;
                mod_ += rem;
                if mod_ >= 0 {
                    mod_ -= dy;
                    delta += 1;
                }

                x_to = x_from + delta;
                self.render_hline(
                    ey1,
                    x_from,
                    PolySubpixelScale::Scale as i32 - first,
                    x_to,
                    first,
                );
                x_from = x_to;

                ey1 += incr;
                self.set_curr_cell(x_from >> PolySubpixelScale::Shift as i32, ey1);
            }
        }
        self.render_hline(
            ey1,
            x_from,
            PolySubpixelScale::Scale as i32 - first,
            x2,
            fy2,
        );
    }
}


//------------------------------------------------------ScanlineHitTest
pub struct ScanlineHitTest {
    x: i32,
    hit: bool,
}

impl ScanlineHitTest {
	pub fn new(x: i32) -> Self {
        Self { x, hit: false }
    }
	pub fn hit(&self) -> bool {
        self.hit
    }
}

impl Scanline for ScanlineHitTest {
	type CoverType = u32;

	fn begin(&self) -> &[Span] {&[]}
	fn add_cells(&mut self, _x: i32, _len: u32, _covers: &[Self::CoverType]) {}
	fn y(&self) -> i32 {0}
	fn reset(&mut self, _min_x: i32, _max_x: i32) {}
    fn reset_spans(&mut self) {}
    fn finalize(&mut self, _: i32) {}
    fn add_cell(&mut self, x: i32, _: u32) {
        if self.x == x {
            self.hit = true;
        }
    }
    fn add_span(&mut self, x: i32, len: u32, _: u32) {
        if self.x >= x && self.x < x + len as i32 {
            self.hit = true;
        }
    }
    fn num_spans(&self) -> u32 {
        1
    }

}

