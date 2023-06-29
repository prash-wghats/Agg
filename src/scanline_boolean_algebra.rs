use crate::basics::{intersect_rectangles, unite_rectangles, CoverScale, RectI, Span};
use crate::{AggPrimitive, RasterScanLine, RendererScanline, Scanline};
use std::marker::PhantomData;
use wrapping_arithmetic::wrappit;

trait FunctorOne<Sl1: Scanline, Sl: Scanline> {
    fn op(&mut self, iter: &[Span], x: i32, len: u32, sl: &mut Sl);
}

trait FunctorTwo<Sl1: Scanline, Sl2: Scanline, Sl: Scanline> {
    fn op(&mut self, iter1: &[Span], iter2: &[Span], x: i32, len: u32, sl: &mut Sl);
}

// Functor.
// Combine two binary encoded spans, i.e., when we don't have any
// anti-aliasing information, but only X and Length. The function
// is compatible with any type of scanlines.
//----------------
pub struct CombineSpansBin<Sl1: Scanline, Sl2: Scanline, Sl: Scanline> {
    sl1: PhantomData<Sl1>,
    sl2: PhantomData<Sl2>,
    sl: PhantomData<Sl>,
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline> CombineSpansBin<Sl1, Sl2, Sl> {
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl2: PhantomData,
            sl: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline> FunctorTwo<Sl1, Sl2, Sl>
    for CombineSpansBin<Sl1, Sl2, Sl>
{
    fn op(&mut self, _iter1: &[Span], _iter2: &[Span], x: i32, len: u32, sl: &mut Sl) {
        sl.add_span(x, len, CoverScale::FULL as u32);
    }
}

// Functor.
// Combine two spans as empty ones. The functor does nothing
// and is used to XOR binary spans.
//----------------
pub struct CombineSpansEmpty<Sl1: Scanline, Sl2: Scanline, Sl: Scanline> {
    sl1: PhantomData<Sl1>,
    sl2: PhantomData<Sl2>,
    sl: PhantomData<Sl>,
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline> CombineSpansEmpty<Sl1, Sl2, Sl> {
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl2: PhantomData,
            sl: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline> FunctorTwo<Sl1, Sl2, Sl>
    for CombineSpansEmpty<Sl1, Sl2, Sl>
{
    fn op(&mut self, _iter1: &[Span], _iter2: &[Span], _x: i32, _len: u32, _sl: &mut Sl) {}
}

// Functor.
// Add nothing. Used in conbine_shapes_sub
//----------------
pub struct AddSpanEmpty<Sl1: Scanline, Sl: Scanline> {
    sl1: PhantomData<Sl1>,
    sl: PhantomData<Sl>,
}

impl<Sl1: Scanline, Sl: Scanline> AddSpanEmpty<Sl1, Sl> {
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl: Scanline> FunctorOne<Sl1, Sl> for AddSpanEmpty<Sl1, Sl> {
    fn op(&mut self, _iter1: &[Span], _x: i32, _len: u32, _sl: &mut Sl) {}
}

// Functor.
// Add a binary span
//----------------
pub struct AddSpanBin<Sl1: Scanline, Sl: Scanline> {
    sl1: PhantomData<Sl1>,
    sl: PhantomData<Sl>,
}

impl<Sl1: Scanline, Sl: Scanline> AddSpanBin<Sl1, Sl> {
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl: Scanline> FunctorOne<Sl1, Sl> for AddSpanBin<Sl1, Sl> {
    fn op(&mut self, _iter1: &[Span], x: i32, len: u32, sl: &mut Sl) {
        sl.add_span(x, len, CoverScale::FULL as u32);
    }
}

// Functor.
// Add an anti-aliased span
// anti-aliasing information, but only X and Length. The function
// is compatible with any type of scanlines.
//----------------
pub struct AddSpanAa<Sl1: Scanline, Sl: Scanline> {
    sl1: PhantomData<Sl1>,
    sl: PhantomData<Sl>,
}

impl<Sl1: Scanline, Sl: Scanline> AddSpanAa<Sl1, Sl> {
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl: Scanline> FunctorOne<Sl1, Sl> for AddSpanAa<Sl1, Sl> {
    fn op(&mut self, span: &[Span], x: i32, len: u32, sl: &mut Sl) {
        if span[0].len < 0 {
            sl.add_span(x, len, unsafe { *span[0].covers } as u32);
        } else if span.len() > 0 {
            let mut covers = span[0].covers as *const Sl::CoverType;
            if span[0].x < x {
                covers = unsafe { covers.offset((x - span[0].x) as isize) };
            }
            sl.add_cells(x, len, unsafe {
                std::slice::from_raw_parts(covers, len as usize)
            });
        }
    }
}

//----------------------------------------------IntersectSpansAa
// Functor.
// Intersect two spans preserving the anti-aliasing information.
// The result is added to the "sl" scanline.
//------------------

pub struct IntersectSpansAa<
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    const CS: u32 = { CoverScale::Shift as u32 },
> {
    sl1: PhantomData<Sl1>,
    sl2: PhantomData<Sl2>,
    sl: PhantomData<Sl>,
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline> IntersectSpansAa<Sl1, Sl2, Sl> {
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl2: PhantomData,
            sl: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline, const CS: u32> FunctorTwo<Sl1, Sl2, Sl>
    for IntersectSpansAa<Sl1, Sl2, Sl, CS>
{
    fn op(&mut self, span1: &[Span], span2: &[Span], x_: i32, len: u32, sl: &mut Sl) {
        let cover_shift = CS;
        let cover_size = 1 << cover_shift;
        let cover_mask = cover_size - 1;
        let cover_full = cover_mask;
        let mut x = x_;
        let mut cover: u32;
        let mut covers1;
        let mut covers2;

        // Calculate the operation code and choose the
        // proper combination algorithm.
        // 0 = Both spans are of AA type
        // 1 = span1 is solid, span2 is AA
        // 2 = span1 is AA, span2 is solid
        // 3 = Both spans are of solid type
        //-----------------
        match (span1[0].len < 0) as u32 | (((span2[0].len < 0) as u32) << 1) {
            0 => {
                // Both are AA spans
                covers1 = span1[0].covers as *const Sl1::CoverType;
                covers2 = span2[0].covers as *const Sl2::CoverType;
                if span1[0].x < x {
                    covers1 = unsafe { covers1.offset((x - span1[0].x) as isize) };
                }
                if span2[0].x < x {
                    covers2 = unsafe { covers2.offset((x - span2[0].x) as isize) };
                }
                for i in 0..len as isize {
                    cover = unsafe { (*covers1.offset(i)).into_u32() * (*covers2.offset(i)).into_u32() };
                    sl.add_cell(
                        x,
                        if cover == cover_full * cover_full {
                            cover_full
                        } else {
                            cover >> cover_shift
                        },
                    );
                    x += 1;
                }
            }
            1 => {
                // span1 is solid, span2 is AA
                covers2 = span2[0].covers as *const Sl2::CoverType;
                if span2[0].x < x {
                    covers2 = unsafe { covers2.offset((x - span2[0].x) as isize) };
                }
                if unsafe { *(span1[0].covers) } as u32 == cover_full {
                    sl.add_cells(x, len, unsafe {
                        std::slice::from_raw_parts(covers2 as *const Sl::CoverType, len as usize)
                    });
                } else {
                    for i in 0..len as isize {
                        cover = unsafe { *span1[0].covers as u32 * (*covers2.offset(i)).into_u32() };
                        sl.add_cell(
                            x,
                            if cover == cover_full * cover_full {
                                cover_full
                            } else {
                                cover >> cover_shift
                            },
                        );
                        x += 1;
                    }
                }
            }
            2 => {
                // span1 is AA, span2 is solid
                covers1 = span1[0].covers as *const Sl1::CoverType;
                if span1[0].x < x {
                    covers1 = unsafe { covers1.offset((x - span1[0].x) as isize) };
                }
                if unsafe { *(span2[0].covers) } as u32 == cover_full {
                    sl.add_cells(x, len, unsafe {
                        std::slice::from_raw_parts(covers1 as *const Sl::CoverType, len as usize)
                    });
                } else {
                    for i in 0..len as isize {
                        cover = unsafe { (*covers1.offset(i)).into_u32() * *span2[0].covers as u32 };
                        sl.add_cell(
                            x,
                            if cover == cover_full * cover_full {
                                cover_full
                            } else {
                                cover >> cover_shift
                            },
                        );
                        x += 1;
                    }
                }
            }
            3 => {
                // Both are solid spans
                cover = unsafe { *span1[0].covers as u32 * *span2[0].covers as u32 };
                sl.add_span(
                    x,
                    len,
                    if cover == cover_full * cover_full {
                        cover_full
                    } else {
                        cover >> cover_shift
                    },
                );
            }
            _ => (),
        }
    }
}

//--------------------------------------------------sbool_unite_spans_aa
// Functor.
// Unite two spans preserving the anti-aliasing information.
// The result is added to the "sl" scanline.
//------------------

pub struct UniteSpansAa<
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    const CS: u32 = { CoverScale::Shift as u32 },
> {
    sl1: PhantomData<Sl1>,
    sl2: PhantomData<Sl2>,
    sl: PhantomData<Sl>,
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline, const CS: u32> UniteSpansAa<Sl1, Sl2, Sl, CS> {
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl2: PhantomData,
            sl: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline, const CS: u32> FunctorTwo<Sl1, Sl2, Sl>
    for UniteSpansAa<Sl1, Sl2, Sl, CS>
{
    fn op(&mut self, span1: &[Span], span2: &[Span], x_: i32, len: u32, sl: &mut Sl) {
        let mut cover;
        let mut covers1;
        let mut covers2;
        let cover_shift = CS;
        let cover_size = 1 << cover_shift;
        let cover_mask = cover_size - 1;
        let cover_full: u32 = cover_mask;
        let mut x = x_;
        // Calculate the operation code and choose the
        // proper combination algorithm.
        // 0 = Both spans are of AA type
        // 1 = span1 is solid, span2 is AA
        // 2 = span1 is AA, span2 is solid
        // 3 = Both spans are of solid type
        //-----------------
        match (span1[0].len < 0) as u32 | ((span2[0].len < 0) as u32) << 1 {
            0 => {
                // Both are AA spans
                covers1 = span1[0].covers;
                covers2 = span2[0].covers;
                if span1[0].x < x {
                    covers1 = unsafe { covers1.offset((x - span1[0].x) as isize) };
                }
                if span2[0].x < x {
                    covers2 = unsafe { covers2.offset((x - span2[0].x) as isize) };
                }
                for _ in 0..len {
                    cover = cover_mask * cover_mask
                        - unsafe {
                            (cover_mask - *covers1 as u32) * (cover_mask - *covers2 as u32)
                        };
                    sl.add_cell(
                        x,
                        if cover == cover_full * cover_full {
                            cover_full
                        } else {
                            cover >> cover_shift
                        },
                    );
                    x += 1;
                    covers1 = unsafe { covers1.offset(1) };
                    covers2 = unsafe { covers2.offset(1) };
                }
            }
            1 => {
                // span1 is solid, span2 is AA
                covers2 = span2[0].covers;
                if span2[0].x < x {
                    covers2 = unsafe { covers2.offset((x - span2[0].x) as isize) };
                }
                if unsafe { *span1[0].covers } as u32 == cover_full {
                    sl.add_span(x, len, cover_full);
                } else {
                    for _ in 0..len {
                        cover = cover_mask * cover_mask
                            - unsafe {
                                (cover_mask - (*span1[0].covers) as u32)
                                    * (cover_mask - *covers2 as u32)
                            };
                        sl.add_cell(
                            x,
                            if cover == cover_full * cover_full {
                                cover_full
                            } else {
                                cover >> cover_shift
                            },
                        );
                        x += 1;
                        covers2 = unsafe { covers2.offset(1) };
                    }
                }
            }
            2 => {
                // span1 is AA, span2 is solid
                covers1 = span1[0].covers;
                if span1[0].x < x {
                    covers1 = unsafe { covers1.offset((x - span1[0].x) as isize) };
                }
                if unsafe { *span2[0].covers } as u32 == cover_full {
                    sl.add_span(x, len, cover_full);
                } else {
                    for _ in 0..len {
                        cover = cover_mask * cover_mask
                            - unsafe {
                                (cover_mask - *covers1 as u32)
                                    * (cover_mask - (*span2[0].covers) as u32)
                            };
                        sl.add_cell(
                            x,
                            if cover == cover_full * cover_full {
                                cover_full
                            } else {
                                cover >> cover_shift
                            },
                        );
                        x += 1;
                        covers1 = unsafe { covers1.offset(1) };
                    }
                }
            }
            3 => {
                // Both are solid spans
                cover = cover_mask * cover_mask
                    - unsafe {
                        (cover_mask - (*(span1[0].covers) as u32))
                            * (cover_mask - (*(span2[0].covers)) as u32)
                    };
                sl.add_span(
                    x,
                    len,
                    if cover == cover_full * cover_full {
                        cover_full
                    } else {
                        cover >> cover_shift
                    },
                );
            }
            _ => {}
        }
    }
}

pub trait Formula {
    fn calculate(a: u32, b: u32) -> u32;
}

pub struct XorFormulaLinear<const CS: u32 = { CoverScale::Shift as u32 }>;
impl<const CS: u32> Formula for XorFormulaLinear<CS> {
    fn calculate(a: u32, b: u32) -> u32 {
        let cover_size = 1 << CS;
        let cover_mask = cover_size - 1;
        let mut cover = a + b;
        if cover > cover_mask {
            cover = cover_mask + cover_mask - cover;
        }
        cover
    }
}

pub struct XorFormulaSaddle<const CS: u32 = { CoverScale::Shift as u32 }>;

impl<const CS: u32> Formula for XorFormulaSaddle<CS> {
	#[wrappit]
    fn calculate(a: u32, b: u32) -> u32 {
        let cover_size:u32 = 1 << CS;
        let cover_mask = cover_size - 1;
        let k = a * b;
        if k == cover_mask * cover_mask {
            return 0;
        }
        let a = (cover_mask * cover_mask - (a << CS) + k) >> CS;
        let b = (cover_mask * cover_mask - (b << CS) + k) >> CS;
        cover_mask - ((a * b) >> CS)
    }
}

pub struct XorFormulaAbsDiff;
impl Formula for XorFormulaAbsDiff {
    fn calculate(a: u32, b: u32) -> u32 {
        (a as i32 - b as i32).abs() as u32
    }
}

//----------------------------------------------------sbool_xor_spans_aa
// Functor.
// XOR two spans preserving the anti-aliasing information.
// The result is added to the "sl" scanline.
//------------------
pub struct XorSpansAa<
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    XorFormula: Formula,
    const CS: u32 = { CoverScale::Shift as u32 },
> {
    sl1: PhantomData<Sl1>,
    sl2: PhantomData<Sl2>,
    sl: PhantomData<Sl>,
    f: PhantomData<XorFormula>,
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline, XorFormula: Formula, const CS: u32>
    XorSpansAa<Sl1, Sl2, Sl, XorFormula, CS>
{
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl2: PhantomData,
            sl: PhantomData,
            f: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline, XorFormula: Formula, const CS: u32>
    FunctorTwo<Sl1, Sl2, Sl> for XorSpansAa<Sl1, Sl2, Sl, XorFormula, CS>
{
    fn op(&mut self, span1: &[Span], span2: &[Span], x_: i32, len: u32, sl: &mut Sl) {
        let mut cover;
        let mut covers1;
        let mut covers2;
        let mut x = x_;
        match (span1[0].len < 0) as u32 | ((span2[0].len < 0) as u32) << 1 {
            0 => {
                covers1 = span1[0].covers;
                covers2 = span2[0].covers;
                if span1[0].x < x {
                    covers1 = unsafe { covers1.offset((x - span1[0].x) as isize) };
                }
                if span2[0].x < x {
                    covers2 = unsafe { covers2.offset((x - span2[0].x) as isize) };
                }
                for _ in 0..len {
                    cover = XorFormula::calculate(unsafe { *covers1 } as u32, unsafe { *covers2 }
                        as u32);
                    if cover != 0 {
                        sl.add_cell(x, cover);
                    }
                    x += 1;
                    covers1 = unsafe { covers1.offset(1) };
                    covers2 = unsafe { covers2.offset(1) };
                }
            }
            1 => {
                covers2 = span2[0].covers;
                if span2[0].x < x {
                    covers2 = unsafe { covers2.offset((x - span2[0].x) as isize) };
                }
                for _ in 0..len {
                    cover = XorFormula::calculate(unsafe { *(span1[0].covers) } as u32, unsafe {
                        *covers2
                    }
                        as u32);
                    if cover != 0 {
                        sl.add_cell(x, cover);
                    }
                    x += 1;
                    covers2 = unsafe { covers2.offset(1) };
                }
            }
            2 => {
                covers1 = span1[0].covers;
                if span1[0].x < x {
                    covers1 = unsafe { covers1.offset((x - span1[0].x) as isize) };
                }
                for _ in 0..len {
                    cover = XorFormula::calculate(unsafe { *covers1 } as u32, unsafe {
                        *(span2[0].covers)
                    }
                        as u32);
                    if cover != 0 {
                        sl.add_cell(x, cover);
                    }
                    x += 1;
                    covers1 = unsafe { covers1.offset(1) };
                }
            }
            3 => {
                cover = XorFormula::calculate(unsafe { *(span1[0].covers) } as u32, unsafe {
                    *(span2[0].covers)
                }
                    as u32);
                if cover != 0 {
                    sl.add_span(x, len, cover);
                }
            }
            _ => {}
        }
    }
}

// Functor.
// Unite two spans preserving the anti-aliasing information.
// The result is added to the "sl" scanline.
//------------------
pub struct SubtractSpansAa<
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    const CS: u32 = { CoverScale::Shift as u32 },
> {
    sl1: PhantomData<Sl1>,
    sl2: PhantomData<Sl2>,
    sl: PhantomData<Sl>,
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline, const CS: u32> SubtractSpansAa<Sl1, Sl2, Sl, CS> {
    pub fn new() -> Self {
        Self {
            sl1: PhantomData,
            sl2: PhantomData,
            sl: PhantomData,
        }
    }
}

impl<Sl1: Scanline, Sl2: Scanline, Sl: Scanline, const CS: u32> FunctorTwo<Sl1, Sl2, Sl>
    for SubtractSpansAa<Sl1, Sl2, Sl, CS>
{
    fn op(&mut self, span1: &[Span], span2: &[Span], x_: i32, len: u32, sl: &mut Sl) {
        let mut cover: u32;
        let mut covers1;
        let mut covers2;
        let cover_shift = CS;
        let cover_size = 1 << cover_shift;
        let cover_mask = cover_size - 1;
        let cover_full: u32 = cover_mask;
        let mut x = x_;
        // Calculate the operation code and choose the
        // proper combination algorithm.
        // 0 = Both spans are of AA type
        // 1 = span1 is solid, span2 is AA
        // 2 = span1 is AA, span2 is solid
        // 3 = Both spans are of solid type
        //-----------------
        match (span1[0].len < 0, span2[0].len < 0) {
            (false, false) => {
                // Both are AA spans
                covers1 = span1[0].covers;
                covers2 = span2[0].covers;
                if span1[0].x < x {
                    covers1 = unsafe { covers1.offset((x - span1[0].x) as isize) };
                }
                if span2[0].x < x {
                    covers2 = unsafe { covers2.offset((x - span2[0].x) as isize) };
                }
                for i in 0..len as isize {
                    cover = unsafe {
                        *covers1.offset(i) as u32 * (cover_mask - *covers2.offset(i) as u32)
                    };
                    if cover != 0 {
                        sl.add_cell(
                            x,
                            if cover == cover_full * cover_full {
                                cover_full
                            } else {
                                cover >> cover_shift
                            },
                        );
                    }
                    x += 1;
                }
            }
            (true, false) => {
                // span1 is solid, span2 is AA
                covers2 = span2[0].covers;
                if span2[0].x < x {
                    covers2 = unsafe { covers2.offset((x - span2[0].x) as isize) };
                }
                for i in 0..len as isize {
                    cover = unsafe {
                        *span1[0].covers as u32 * (cover_mask - *covers2.offset(i) as u32)
                    };
                    if cover != 0 {
                        sl.add_cell(
                            x,
                            if cover == cover_full * cover_full {
                                cover_full
                            } else {
                                cover >> cover_shift
                            },
                        );
                    }
                    x += 1;
                }
            }
            (false, true) => {
                // span1 is AA, span2 is solid
                covers1 = span1[0].covers;
                if span1[0].x < x {
                    covers1 = unsafe { covers1.offset((x - span1[0].x) as isize) };
                }
                if unsafe { *span2[0].covers } as u32 != cover_full {
                    for _ in 0..len {
                        cover = unsafe {
                            *covers1.offset(1) as u32 * (cover_mask - *span2[0].covers as u32)
                        };
                        if cover != 0 {
                            sl.add_cell(
                                x,
                                if cover == cover_full * cover_full {
                                    cover_full
                                } else {
                                    cover >> cover_shift
                                },
                            );
                        }
                        x += 1;
                    }
                }
            }
            (true, true) => {
                // Both are solid spans
                cover = unsafe { *span1[0].covers as u32 * (cover_mask - *span2[0].covers as u32) };
                if cover != 0 {
                    sl.add_span(
                        x,
                        len,
                        if cover == cover_full * cover_full {
                            cover_full
                        } else {
                            cover >> cover_shift
                        },
                    );
                }
            }
        }
    }
}

// Add spans and render
fn sbool_add_spans_and_render<
    Sl1: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
    Func: FunctorOne<Sl1, Sl>,
>(
    sl1: &Sl1, sl: &mut Sl, ren: &mut Ren, add_span: &mut Func,
) {
    sl.reset_spans();
    let spans = sl1.begin();
    let num_spans = sl1.num_spans();
	let mut i = 0;
    for span in spans {
        add_span.op(&spans[i..], span.x, span.len.abs() as u32, sl);
        if num_spans == 0 {
            break;
        }
		i += 1;
    }
    sl.finalize(sl1.y());
    ren.render(sl);
}

//---------------------------------------------sbool_intersect_scanlines
// Intersect two scanlines, "sl1" and "sl2" and generate a new "sl" one.
// The combine_spans functor can be of type sbool_combine_spans_bin or
// sbool_intersect_spans_aa. First is a general functor to combine
// two spans without Anti-Aliasing, the second preserves the AA
// information, but works slower
//
fn sbool_intersect_scanlines<
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Func: FunctorTwo<Sl1, Sl2, Sl>,
>(
    sl1: &Sl1, sl2: &Sl2, sl: &mut Sl, combine_spans: &mut Func,
) {
    sl.reset_spans();

    let num1 = sl1.num_spans() as usize;
    if num1 == 0 {
        return;
    }

    let num2 = sl2.num_spans() as usize;
    if num2 == 0 {
        return;
    }

    let span1 = sl1.begin();
    let span2 = sl2.begin();
    let (mut i1, mut i2): (usize, usize) = (0, 0);
    while i1 < num1 && i2 < num2 {
        let xb1 = span1[i1].x;
        let xb2 = span2[i2].x;
        let xe1 = xb1 + (span1[i1].len).abs() - 1;
        let xe2 = xb2 + (span2[i2].len).abs() - 1;

        // Determine what spans we should advance in the next step
        // The span with the least ending X should be advanced
        // advance_both is just an optimization when we ending
        // coordinates are the same and we can advance both
        //--------------
        let advance_span1 = xe1 < xe2;
        let advance_both = xe1 == xe2;

        // Find the intersection of the spans
        // and check if they intersect
        //--------------
        let mut xb = xb1;
        if xb1 < xb2 {
            xb = xb2;
        }
        let mut xe = xe1;
        if xe1 > xe2 {
            xe = xe2;
        }
        if xb <= xe {
            combine_spans.op(&span1[i1..], &span2[i2..], xb, (xe - xb + 1) as u32, sl);
        }

        // Advance the spans
        //--------------
        if advance_both {
            i1 += 1;
            i2 += 1;
        } else {
            if advance_span1 {
                i1 += 1;
            } else {
                i2 += 1;
            }
        }
    }
}

//------------------------------------------------sbool_intersect_shapes
// Intersect the scanline shapes. Here the "Scanline Generator"
// abstraction is used. ScanlineGen1 and ScanlineGen2 are
// the generators, and can be of type rasterizer_scanline_aa<>.
// There function requires three scanline containers that can be of
// different types.
// "sl1" and "sl2" are used to retrieve scanlines from the generators,
// "sl" is ised as the resulting scanline to render it.
// The external "sl1" and "sl2" are used only for the sake of
// optimization and reusing of the scanline objects.
// the function calls sbool_intersect_scanlines with CombineSpansFunctor
// as the last argument. See sbool_intersect_scanlines for details.
//----------
fn sbool_intersect_shapes<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
    CombineSpansFunctor: FunctorTwo<Sl1, Sl2, Sl>,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren, combine_spans: &mut CombineSpansFunctor,
) {
    // Prepare the scanline generators.
    // If anyone of them doesn't contain
    // any scanlines, then return.
    //-----------------
    if !sg1.rewind_scanlines() {
        return;
    }
    if !sg2.rewind_scanlines() {
        return;
    }

    // Get the bounding boxes
    //----------------
    let mut r1 = RectI::new(sg1.min_x(), sg1.min_y(), sg1.max_x(), sg1.max_y());
    let r2 = RectI::new(sg2.min_x(), sg2.min_y(), sg2.max_x(), sg2.max_y());

    // Calculate the intersection of the bounding
    // boxes and return if they don't intersect.
    //-----------------
    let ir = intersect_rectangles(&mut r1, &r2);
    if !ir.is_valid() {
        return;
    }

    // Reset the scanlines and get two first ones
    //-----------------
    sl.reset(ir.x1, ir.x2);
    sl1.reset(sg1.min_x(), sg1.max_x());
    sl2.reset(sg2.min_x(), sg2.max_x());
    if !sg1.sweep_scanline(sl1) {
        return;
    }
    if !sg2.sweep_scanline(sl2) {
        return;
    }

    ren.prepare();

    // The main loop
    // Here we synchronize the scanlines with
    // the same Y coordinate, ignoring all other ones.
    // Only scanlines having the same Y-coordinate
    // are to be combined.
    //-----------------
    loop {
        while sl1.y() < sl2.y() {
            if !sg1.sweep_scanline(sl1) {
                return;
            }
        }
        while sl2.y() < sl1.y() {
            if !sg2.sweep_scanline(sl2) {
                return;
            }
        }

        if sl1.y() == sl2.y() {
            // The Y coordinates are the same.
            // Combine the scanlines, render if they contain any spans,
            // and advance both generators to the next scanlines
            //----------------------
            sbool_intersect_scanlines(sl1, sl2, sl, combine_spans);
            if sl.num_spans() > 0 {
                sl.finalize(sl1.y());
                ren.render(sl);
            }
            if !sg1.sweep_scanline(sl1) {
                return;
            }
            if !sg2.sweep_scanline(sl2) {
                return;
            }
        }
    }
}

// Unite two scanlines, "sl1" and "sl2" and generate a new "sl" one.
// The combine_spans functor can be of type sbool_combine_spans_bin or
// sbool_intersect_spans_aa. First is a general functor to combine
// two spans without Anti-Aliasing, the second preserves the AA
// information, but works slower
fn sbool_unite_scanlines<
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    F1: FunctorOne<Sl1, Sl>,
    F2: FunctorOne<Sl2, Sl>,
    F: FunctorTwo<Sl1, Sl2, Sl>,
>(
    sl1: &Sl1, sl2: &Sl2, sl: &mut Sl, add_span1: &mut F1, add_span2: &mut F2,
    combine_spans: &mut F,
) {
    sl.reset_spans();

    let mut num1 = sl1.num_spans();
    let mut num2 = sl2.num_spans();

    let span1 = sl1.begin();
    let span2 = sl2.begin();
    let (mut i1, mut i2): (usize, usize) = (0, 0);
    const INVALID_B: i32 = 0xfffffff;
	const INVALID_E: i32 = 0xfffffff - 1;
	// Initialize the spans as invalid
    //---------------
    let mut xb1 = INVALID_B;
    let mut xb2 = INVALID_B;
    let mut xe1 = INVALID_E;
    let mut xe2 = INVALID_E;

    // Initialize span1 if there are spans
    //---------------
    if num1 != 0 {
        xb1 = span1[i1].x;
        xe1 = xb1 + (span1[i1].len as i32).abs() - 1;
        num1 -= 1;
    }

    // Initialize span2 if there are spans
    //---------------
    if num2 != 0 {
        xb2 = span2[i2].x;
        xe2 = xb2 + (span2[i2].len as i32).abs() - 1;
        num2 -= 1;
    }

    loop {
        // Retrieve a new span1 if it's invalid
        //----------------
        if (num1 != 0) && (xb1 > xe1) {
            num1 -= 1;
            i1 += 1;
            xb1 = span1[i1].x;
            xe1 = xb1 + (span1[i1].len as i32).abs() - 1;
        }

        // Retrieve a new span2 if it's invalid
        //----------------
        if (num2 != 0) && (xb2 > xe2) {
            num2 -= 1;
            i2 += 1;
            xb2 = span2[i2].x;
            xe2 = xb2 + (span2[i2].len as i32).abs() - 1;
        }

        if (xb1 > xe1) && (xb2 > xe2) {
            break;
        }

        // Calculate the intersection
        //----------------
        let mut xb = xb1;
        let mut xe = xe1;
        if xb < xb2 {
            xb = xb2;
        }
        if xe > xe2 {
            xe = xe2;
        }
        let len = xe - xb + 1; // The length of the intersection
        if len > 0 {
            // The spans intersect,
            // add the beginning of the span
            //----------------
            if xb1 < xb2 {
                add_span1.op(&span1[i1..], xb1, (xb2 - xb1) as u32, sl);
                xb1 = xb2;
            } else if xb2 < xb1 {
                add_span2.op(&span2[i2..], xb2, (xb1 - xb2) as u32, sl);
                xb2 = xb1;
            }

            // Add the combination part of the spans
            //----------------
            combine_spans.op(&span1[i1..], &span2[i2..], xb, len as u32, sl);

            // Invalidate the fully processed span or both
            //----------------
            if xe1 < xe2 {
                // Invalidate span1 and eat
                // the processed part of span2
                //--------------
                xb1 = INVALID_B;
                xe1 = INVALID_E;
                xb2 += len;
            } else if xe2 < xe1 {
                // Invalidate span2 and eat
                // the processed part of span1
                //--------------
                xb2 = INVALID_B;
                xe2 = INVALID_E;
                xb1 += len;
            } else {
                xb1 = INVALID_B; // Invalidate both
                xe1 = INVALID_E;
                xb2 = INVALID_B;
                xe2 = INVALID_E;
            }
        } else {
            // The spans do not intersect
            //--------------
            if xb1 < xb2 {
                // Advance span1
                //---------------
                if xb1 <= xe1 {
                    add_span1.op(&span1[i1..], xb1, (xe1 - xb1 + 1) as u32, sl);
                }
                xb1 = INVALID_B; // Invalidate
                xe1 = INVALID_E;
            } else {
                // Advance span2
                //---------------
                if xb2 <= xe2 {
                    add_span2.op(&span2[i2..], xb2, (xe2 - xb2 + 1) as u32, sl);
                }
                xb2 = INVALID_B; // Invalidate
                xe2 = INVALID_E;
            }
        }
    }
}

//----------------------------------------------------sbool_unite_shapes
// Unite the scanline shapes. Here the "Scanline Generator"
// abstraction is used. ScanlineGen1 and ScanlineGen2 are
// the generators, and can be of type rasterizer_scanline_aa<>.
// There function requires three scanline containers that can be
// of different type.
// "sl1" and "sl2" are used to retrieve scanlines from the generators,
// "sl" is ised as the resulting scanline to render it.
// The external "sl1" and "sl2" are used only for the sake of
// optimization and reusing of the scanline objects.
// the function calls sbool_unite_scanlines with CombineSpansFunctor
// as the last argument. See sbool_unite_scanlines for details.
//----------

fn sbool_unite_shapes<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
    AddSpanFunctor1: FunctorOne<Sl1, Sl>,
    AddSpanFunctor2: FunctorOne<Sl2, Sl>,
    CombineSpansFunctor: FunctorTwo<Sl1, Sl2, Sl>,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren, add_span1: &mut AddSpanFunctor1, add_span2: &mut AddSpanFunctor2,
    combine_spans: &mut CombineSpansFunctor,
) {
    // Prepare the scanline generators.
    // If anyone of them doesn't contain
    // any scanlines, then return.
    //-----------------
    let mut flag1 = sg1.rewind_scanlines();
    let mut flag2 = sg2.rewind_scanlines();
    if !flag1 && !flag2 {
        return;
    }

    // Get the bounding boxes
    //----------------
    let mut r1 = RectI::new(sg1.min_x(), sg1.min_y(), sg1.max_x(), sg1.max_y());
    let r2 = RectI::new(sg2.min_x(), sg2.min_y(), sg2.max_x(), sg2.max_y());

    // Calculate the union of the bounding boxes
    //-----------------
    let mut ur = RectI::new(1, 1, 0, 0);
    if flag1 && flag2 {
        ur = *unite_rectangles(&mut r1, &r2);
    } else if flag1 {
        ur = r1;
    } else if flag2 {
        ur = r2;
    }

    if !ur.is_valid() {
        return;
    }

    ren.prepare();

    // Reset the scanlines and get two first ones
    //-----------------
    sl.reset(ur.x1, ur.x2);
    if flag1 {
        sl1.reset(sg1.min_x(), sg1.max_x());
        flag1 = sg1.sweep_scanline(sl1);
    }

    if flag2 {
        sl2.reset(sg2.min_x(), sg2.max_x());
        flag2 = sg2.sweep_scanline(sl2);
    }

    // The main loop
    // Here we synchronize the scanlines with
    // the same Y coordinate.
    //-----------------
    while flag1 || flag2 {
        if flag1 && flag2 {
            if sl1.y() == sl2.y() {
                // The Y coordinates are the same.
                // Combine the scanlines, render if they contain any spans,
                // and advance both generators to the next scanlines
                //----------------------
                sbool_unite_scanlines(sl1, sl2, sl, add_span1, add_span2, combine_spans);
                if sl.num_spans() > 0 {
                    sl.finalize(sl1.y());
                    ren.render(sl);
                }
                flag1 = sg1.sweep_scanline(sl1);
                flag2 = sg2.sweep_scanline(sl2);
            } else {
                if sl1.y() < sl2.y() {
                    sbool_add_spans_and_render(sl1, sl, ren, add_span1);
                    flag1 = sg1.sweep_scanline(sl1);
                } else {
                    sbool_add_spans_and_render(sl2, sl, ren, add_span2);
                    flag2 = sg2.sweep_scanline(sl2);
                }
            }
        } else {
            if flag1 {
                sbool_add_spans_and_render(sl1, sl, ren, add_span1);
                flag1 = sg1.sweep_scanline(sl1);
            }
            if flag2 {
                sbool_add_spans_and_render(sl2, sl, ren, add_span2);
                flag2 = sg2.sweep_scanline(sl2);
            }
        }
    }
}

//-------------------------------------------------sbool_subtract_shapes
// Subtract the scanline shapes, "sg1-sg2". Here the "Scanline Generator"
// abstraction is used. ScanlineGen1 and ScanlineGen2 are
// the generators, and can be of type rasterizer_scanline_aa<>.
// There function requires three scanline containers that can be of
// different types.
// "sl1" and "sl2" are used to retrieve scanlines from the generators,
// "sl" is ised as the resulting scanline to render it.
// The external "sl1" and "sl2" are used only for the sake of
// optimization and reusing of the scanline objects.
// the function calls sbool_intersect_scanlines with CombineSpansFunctor
// as the last argument. See combine_scanlines_sub for details.
//----------
fn sbool_subtract_shapes<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
    AddSpanFunctor1: FunctorOne<Sl1, Sl>,
    CombineSpansFunctor: FunctorTwo<Sl1, Sl2, Sl>,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren, add_span1: &mut AddSpanFunctor1, combine_spans: &mut CombineSpansFunctor,
) {
    if !sg1.rewind_scanlines() {
        return;
    }
    let mut flag2 = sg2.rewind_scanlines();

    let _r1 = RectI::new(sg1.min_x(), sg1.min_y(), sg1.max_x(), sg1.max_y());

    sl.reset(sg1.min_x(), sg1.max_x());
    sl1.reset(sg1.min_x(), sg1.max_x());
    sl2.reset(sg2.min_x(), sg2.max_x());
    if !sg1.sweep_scanline(sl1) {
        return;
    }

    if flag2 {
        flag2 = sg2.sweep_scanline(sl2);
    }

    ren.prepare();

    let mut add_span2 = AddSpanEmpty::<Sl2, Sl>::new();
    let mut flag1;

    loop {
        while flag2 && sl2.y() < sl1.y() {
            flag2 = sg2.sweep_scanline(sl2);
        }

        if flag2 && sl2.y() == sl1.y() {
            sbool_unite_scanlines(sl1, sl2, sl, add_span1, &mut add_span2, combine_spans);
            if sl.num_spans() > 0 {
                sl.finalize(sl1.y());
                ren.render(sl);
            }
        } else {
            sbool_add_spans_and_render(sl1, sl, ren, add_span1);
        }

        flag1 = sg1.sweep_scanline(sl1);
        if !flag1 {
            break;
        }
    }
}

//---------------------------------------------sbool_intersect_shapes_aa
// Intersect two anti-aliased scanline shapes.
// Here the "Scanline Generator" abstraction is used.
// ScanlineGen1 and ScanlineGen2 are the generators, and can be of
// type rasterizer_scanline_aa<>. There function requires three
// scanline containers that can be of different types.
// "sl1" and "sl2" are used to retrieve scanlines from the generators,
// "sl" is ised as the resulting scanline to render it.
// The external "sl1" and "sl2" are used only for the sake of
// optimization and reusing of the scanline objects.
//----------
fn sbool_intersect_shapes_aa<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut combine_functor = IntersectSpansAa::<Sl1, Sl2, Sl>::new();
    sbool_intersect_shapes(sg1, sg2, sl1, sl2, sl, ren, &mut combine_functor);
}

//--------------------------------------------sbool_intersect_shapes_bin
// Intersect two binary scanline shapes (without anti-aliasing).
// See intersect_shapes_aa for more comments
//----------
fn sbool_intersect_shapes_bin<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut combine_functor = CombineSpansBin::<Sl1, Sl2, Sl>::new();
    sbool_intersect_shapes(sg1, sg2, sl1, sl2, sl, ren, &mut combine_functor);
}

//-------------------------------------------------sbool_unite_shapes_aa
// Unite two anti-aliased scanline shapes
// See intersect_shapes_aa for more comments
//----------
fn sbool_unite_shapes_aa<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut add_functor1 = AddSpanAa::<Sl1, Sl>::new();
    let mut add_functor2 = AddSpanAa::<Sl2, Sl>::new();
    let mut combine_functor = UniteSpansAa::<Sl1, Sl2, Sl>::new();
    sbool_unite_shapes(
        sg1,
        sg2,
        sl1,
        sl2,
        sl,
        ren,
        &mut add_functor1,
        &mut add_functor2,
        &mut combine_functor,
    );
}

//------------------------------------------------sbool_unite_shapes_bin
// Unite two binary scanline shapes (without anti-aliasing).
// See intersect_shapes_aa for more comments
//----------
fn sbool_unite_shapes_bin<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut add_functor1 = AddSpanBin::<Sl1, Sl>::new();
    let mut add_functor2 = AddSpanBin::<Sl2, Sl>::new();
    let mut combine_functor = CombineSpansBin::<Sl1, Sl2, Sl>::new();

    sbool_unite_shapes(
        sg1,
        sg2,
        sl1,
        sl2,
        sl,
        ren,
        &mut add_functor1,
        &mut add_functor2,
        &mut combine_functor,
    );
}

//------------------------------------------sbool_xor_shapes_saddle_aa
// Apply eXclusive OR to two anti-aliased scanline shapes.
// There's the classical "Saddle" used to calculate the
// Anti-Aliasing values, that is:
// a XOR b : 1-((1-a+a*b)*(1-b+a*b))
// See intersect_shapes_aa for more comments
//----------
fn sbool_xor_shapes_saddle_aa<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut add_functor1 = AddSpanAa::<Sl1, Sl>::new();
    let mut add_functor2 = AddSpanAa::<Sl2, Sl>::new();
    let mut combine_functor = XorSpansAa::<Sl1, Sl2, Sl, XorFormulaSaddle>::new();
    sbool_unite_shapes(
        sg1,
        sg2,
        sl1,
        sl2,
        sl,
        ren,
        &mut add_functor1,
        &mut add_functor2,
        &mut combine_functor,
    );
}

//--------------------------------------sbool_xor_shapes_abs_diff_aa
// Apply eXclusive OR to two anti-aliased scanline shapes.
// There's the absolute difference used to calculate
// Anti-Aliasing values, that is:
// a XOR b : abs(a-b)
// See intersect_shapes_aa for more comments
//----------

fn sbool_xor_shapes_abs_diff_aa<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut add_functor1 = AddSpanAa::<Sl1, Sl>::new();
    let mut add_functor2 = AddSpanAa::<Sl2, Sl>::new();
    let mut combine_functor = XorSpansAa::<Sl1, Sl2, Sl, XorFormulaAbsDiff>::new();
    sbool_unite_shapes(
        sg1,
        sg2,
        sl1,
        sl2,
        sl,
        ren,
        &mut add_functor1,
        &mut add_functor2,
        &mut combine_functor,
    );
}

//--------------------------------------------------sbool_xor_shapes_bin
// Apply eXclusive OR to two binary scanline shapes (without anti-aliasing).
// See intersect_shapes_aa for more comments
//----------
fn sbool_xor_shapes_bin<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut add_functor1 = AddSpanBin::<Sl1, Sl>::new();
    let mut add_functor2 = AddSpanBin::<Sl2, Sl>::new();
    let mut combine_functor = CombineSpansEmpty::<Sl1, Sl2, Sl>::new();
    sbool_unite_shapes(
        sg1,
        sg2,
        sl1,
        sl2,
        sl,
        ren,
        &mut add_functor1,
        &mut add_functor2,
        &mut combine_functor,
    );
}

//---------------------------------------------sbool_subtract_shapes_bin
// Subtract binary shapes "sg1-sg2" without anti-aliasing
// See intersect_shapes_aa for more comments
//----------

fn sbool_subtract_shapes_bin<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut add_functor = AddSpanBin::<Sl1, Sl>::new();
    let mut combine_functor = CombineSpansEmpty::<Sl1, Sl2, Sl>::new();
    sbool_subtract_shapes(
        sg1,
        sg2,
        sl1,
        sl2,
        sl,
        ren,
        &mut add_functor,
        &mut combine_functor,
    );
}

//---------------------------------------------------sbool_xor_shapes_aa
// Apply eXclusive OR to two anti-aliased scanline shapes. There's
// a modified "Linear" XOR used instead of classical "Saddle" one.
// The reason is to have the result absolutely conststent with what
// the scanline rasterizer produces.
// See intersect_shapes_aa for more comments
//----------
fn sbool_xor_shapes_aa<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut add_functor1 = AddSpanAa::<Sl1, Sl>::new();
    let mut add_functor2 = AddSpanAa::<Sl2, Sl>::new();
    let mut combine_functor = XorSpansAa::<Sl1, Sl2, Sl, XorFormulaLinear>::new();
    sbool_unite_shapes(
        sg1,
        sg2,
        sl1,
        sl2,
        sl,
        ren,
        &mut add_functor1,
        &mut add_functor2,
        &mut combine_functor,
    );
}

//----------------------------------------------sbool_subtract_shapes_aa
// Subtract shapes "sg1-sg2" with anti-aliasing
// See intersect_shapes_aa for more comments
//----------
fn sbool_subtract_shapes_aa<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2, sl: &mut Sl,
    ren: &mut Ren,
) {
    let mut add_functor = AddSpanAa::<Sl1, Sl> {
        sl: PhantomData,
        sl1: PhantomData,
    };
    let mut combine_functor = SubtractSpansAa::<Sl1, Sl2, Sl>::new();
    sbool_subtract_shapes(
        sg1,
        sg2,
        sl1,
        sl2,
        sl,
        ren,
        &mut add_functor,
        &mut combine_functor,
    );
}

//------------------------------------------------------------sbool_op_e
#[derive(Clone, Copy)]
pub enum SBoolOp {
    Or,         //----sbool_or
    And,        //----sbool_and
    Xor,        //----sbool_xor
    XorSaddle,  //----sbool_xor_saddle
    XorAbsDiff, //----sbool_xor_abs_diff
    AMinusB,    //----sbool_a_minus_b
    BMinusA,    //----sbool_b_minus_a
}

//----------------------------------------------sbool_combine_shapes_bin
pub fn sbool_combine_shapes_bin<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    op: SBoolOp, sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2,
    sl: &mut Sl, renderer: &mut Ren,
) {
    match op {
        SBoolOp::Or => sbool_unite_shapes_bin(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::And => sbool_intersect_shapes_bin(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::Xor | SBoolOp::XorSaddle | SBoolOp::XorAbsDiff => {
            sbool_xor_shapes_bin(sg1, sg2, sl1, sl2, sl, renderer)
        }
        SBoolOp::AMinusB => sbool_subtract_shapes_bin(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::BMinusA => sbool_subtract_shapes_bin(sg2, sg1, sl2, sl1, sl, renderer),
    }
}

//-----------------------------------------------sbool_combine_shapes_aa
pub fn sbool_combine_shapes_aa<
    ScanlineGen1: RasterScanLine,
    ScanlineGen2: RasterScanLine,
    Sl1: Scanline,
    Sl2: Scanline,
    Sl: Scanline,
    Ren: RendererScanline,
>(
    op: SBoolOp, sg1: &mut ScanlineGen1, sg2: &mut ScanlineGen2, sl1: &mut Sl1, sl2: &mut Sl2,
    sl: &mut Sl, renderer: &mut Ren,
) {
    match op {
        SBoolOp::Or => sbool_unite_shapes_aa(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::And => sbool_intersect_shapes_aa(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::Xor => sbool_xor_shapes_aa(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::XorSaddle => sbool_xor_shapes_saddle_aa(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::XorAbsDiff => sbool_xor_shapes_abs_diff_aa(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::AMinusB => sbool_subtract_shapes_aa(sg1, sg2, sl1, sl2, sl, renderer),
        SBoolOp::BMinusA => sbool_subtract_shapes_aa(sg2, sg1, sl2, sl1, sl, renderer),
    }
}
