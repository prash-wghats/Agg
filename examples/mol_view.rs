use agg::basics::PathCmd;
use agg::{RasterScanLine, RenderBuffer, RendererScanlineColor, VertexSource};

use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::str;

mod misc;
use misc::pixel_formats::*;

const FLIP_Y: bool = true;
const FNAME: &str = "1.sdf";

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

trait ReadLine {
    fn readline(&mut self, buf: &mut String) -> bool;
}

impl ReadLine for BufReader<File> {
    fn readline(&mut self, buf: &mut String) -> bool {
        if let Ok(i) = self.read_line(buf) {
            if i == 0 {
                return false;
            }
            return true;
        }
        false
    }
}
const START_HEIGHT: f64 = 400.;
const START_WIDTH: f64 = 400.;

#[derive(Clone, Copy)]
enum AtomColor {
    ColorGeneral = 0,
    ColorN = 1,
    ColorO = 2,
    ColorS = 3,
    ColorP = 4,
    ColorHalogen = 5,
    End,
}

use AtomColor::*;

struct Atom {
    x: f64,
    y: f64,
    label: String,
    charge: i32,
    color_idx: AtomColor,
}

#[derive(Clone, Copy)]
struct Bond {
    idx1: u32,
    idx2: u32,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    order: u32,
    stereo: i32,
    topology: i32,
}

/*
MFCD00191150
  Mt7.00  09020210442D

 23 23  0  0  1  0  0  0  0  0999 V2000
   -2.6793   -0.2552    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
   . . .
  1  2  1  0  0  0  0
   . . .
M  END
> <MDLNUMBER>
MFCD00191150

$$$$
*/
struct Molecule {
    atoms: Vec<Atom>,
    bonds: Vec<Bond>,
    name: String,
    avr_len: f64,
}

impl Molecule {
    fn new() -> Molecule {
        Molecule {
            atoms: Vec::new(),
            bonds: Vec::new(),
            name: String::new(),
            avr_len: 0.0,
        }
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn atom(&self, i: u32) -> &Atom {
        &self.atoms[i as usize]
    }
    fn bond(&self, i: u32) -> &Bond {
        &self.bonds[i as usize]
    }
    fn num_atoms(&self) -> u32 {
        self.atoms.len() as u32
    }
    fn num_bonds(&self) -> u32 {
        self.bonds.len() as u32
    }
    fn average_bond_len(&self) -> f64 {
        self.avr_len
    }
    fn read(&mut self, fd: &mut std::io::BufReader<std::fs::File>) -> bool {
        let mut buf = String::new();
        if !fd.readline(&mut buf) {
            return false;
        }
        self.name = buf.trim().to_string();

        for _ in 0..3 {
            buf.clear();
            if !fd.readline(&mut buf) {
                return false;
            }
        }

        let num_atoms;
        let num_bonds;

        num_atoms = Self::get_int(&buf, 1, 3);
        num_bonds = Self::get_int(&buf, 4, 3);
        if num_atoms == 0 || num_bonds == 0 {
            return false;
        }

        for _ in 0..num_atoms {
            buf.clear();
            if !fd.readline(&mut buf) {
                return false;
            }

            let x = Self::get_dbl(&buf, 1, 10);
            let y = Self::get_dbl(&buf, 11, 10);
            let label = Self::get_str(&buf, 32, 3);

            let mut charge = Self::get_int(&buf, 39, 1);
            if charge > 0 {
                charge = 4 - charge;
            }
            let color_idx = match label.as_ref() {
                "N" => AtomColor::ColorN,
                "O" => AtomColor::ColorO,
                "S" => AtomColor::ColorS,
                "P" => AtomColor::ColorP,
                "F" | "Cl" | "Br" | "I" => AtomColor::ColorHalogen,
                _ => AtomColor::ColorGeneral,
            };
            self.atoms.push(Atom {
                x: x,
                y: y,
                label: label,
                charge: charge,
                color_idx: color_idx,
            });
        }

        for _i in 0..num_bonds {
            buf.clear();
            if !fd.readline(&mut buf) {
                return false;
            }

            let idx1 = Self::get_int(&buf, 1, 3) - 1;
            let idx2 = Self::get_int(&buf, 4, 3) - 1;
            if idx1 > num_atoms || idx2 > num_bonds {
                return false;
            }
            let x1 = self.atoms[idx1 as usize].x;
            let y1 = self.atoms[idx1 as usize].y;
            let x2 = self.atoms[idx2 as usize].x;
            let y2 = self.atoms[idx2 as usize].y;
            let order = Self::get_int(&buf, 7, 3);
            let stereo = Self::get_int(&buf, 10, 3);
            let topology = Self::get_int(&buf, 13, 3);
            self.bonds.push(Bond {
                idx1: idx1 as u32,
                idx2: idx2 as u32,
                x1: x1,
                y1: y1,
                x2: x2,
                y2: y2,
                order: order as u32,
                stereo: stereo,
                topology: topology,
            });
            self.avr_len += ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)).sqrt();
        }
        self.avr_len /= self.bonds.len() as f64;

        buf.clear();
        while fd.readline(&mut buf) {
            if buf.starts_with("$") {
                return true;
            }
            buf.clear();
        }

        return false;
    }

    fn get_int(buf: &str, pos: usize, len: usize) -> i32 {
        let tmp = Self::get_str(buf, pos, len);
        tmp.parse::<i32>().unwrap()
    }

    fn get_dbl(buf: &str, pos: usize, len: usize) -> f64 {
        let tmp = Self::get_str(buf, pos, len);
        tmp.parse::<f64>().unwrap()
    }

    fn get_str(buf: &str, pos: usize, len: usize) -> String {
        let buf = buf.as_bytes();
        let mut len = len;
        let mut pos = pos;
        pos -= 1;
        let mut p = pos;
        let buf_len = buf.len();
        while pos + len < buf_len && buf[p].is_ascii_whitespace() {
            p += 1;
            len -= 1;
        }
        let mut i = 0;
        if len > 0 {
            while i < len && buf[p + i] != 0 && !buf[p + i].is_ascii_whitespace() {
                i += 1;
            }
        }
        return unsafe { str::from_utf8_unchecked(&buf[p..p + i]) }.to_string();
    }

    fn trim_cr_lf(buf: &mut String) -> usize {
        let mut buf = &mut buf[..];
        let mut len = buf.len();
        let mut pos = 0;
        while len > 0
            && (buf.chars().nth(0).unwrap() == '\n' || buf.chars().nth(0).unwrap() == '\r')
        {
            len -= 1;
            pos += 1;
        }
        if pos > 0 {
            buf = &mut buf[pos..];
        }
        while len > 0
            && (buf.chars().nth(len - 1).unwrap() == '\n'
                || buf.chars().nth(len - 1).unwrap() == '\r')
        {
            len -= 1;
        }
        //buf.truncate(len);
        len
    }
}

struct Line {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    thickness: f64,
    dx: f64,
    dy: f64,
    vertex: u32,
}

impl Line {
    fn new() -> Line {
        Line {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 0.0,
            thickness: 0.1,
            dx: 0.0,
            dy: 0.0,
            vertex: 0,
        }
    }

    fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.x1 = x1;
        self.y1 = y1;
        self.x2 = x2;
        self.y2 = y2;
    }

    fn thickness(&mut self, th: f64) {
        self.thickness = th;
    }
}

impl VertexSource for Line {
    fn rewind(&mut self, _start: u32) {
        calc_orthogonal(
            self.thickness * 0.5,
            self.x1,
            self.y1,
            self.x2,
            self.y2,
            &mut self.dx,
            &mut self.dy,
        );
        self.vertex = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        match self.vertex {
            0 => {
                *x = self.x1 - self.dx;
                *y = self.y1 - self.dy;
                self.vertex += 1;
                return PathCmd::MoveTo as u32;
            }
            1 => {
                *x = self.x2 - self.dx;
                *y = self.y2 - self.dy;
                self.vertex += 1;
                return PathCmd::LineTo as u32;
            }
            2 => {
                *x = self.x2 + self.dx;
                *y = self.y2 + self.dy;
                self.vertex += 1;
                return PathCmd::LineTo as u32;
            }
            3 => {
                *x = self.x1 + self.dx;
                *y = self.y1 + self.dy;
                self.vertex += 1;
                return PathCmd::LineTo as u32;
            }
            _ => return PathCmd::Stop as u32,
        }
    }
}

struct SolidWedge {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    thickness: f64,
    dx: f64,
    dy: f64,
    vertex: u32,
}

impl SolidWedge {
    fn new() -> SolidWedge {
        SolidWedge {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 0.0,
            thickness: 0.1,
            dx: 0.0,
            dy: 0.0,
            vertex: 0,
        }
    }

    fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.x1 = x1;
        self.y1 = y1;
        self.x2 = x2;
        self.y2 = y2;
    }

    fn thickness(&mut self, th: f64) {
        self.thickness = th;
    }
}

impl VertexSource for SolidWedge {
    fn rewind(&mut self, _start: u32) {
        calc_orthogonal(
            self.thickness * 2.0,
            self.x1,
            self.y1,
            self.x2,
            self.y2,
            &mut self.dx,
            &mut self.dy,
        );
        self.vertex = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        match self.vertex {
            0 => {
                *x = self.x1;
                *y = self.y1;
                self.vertex += 1;
                return PathCmd::MoveTo as u32;
            }
            1 => {
                *x = self.x2 - self.dx;
                *y = self.y2 - self.dy;
                self.vertex += 1;
                return PathCmd::LineTo as u32;
            }
            2 => {
                *x = self.x2 + self.dx;
                *y = self.y2 + self.dy;
                self.vertex += 1;
                return PathCmd::LineTo as u32;
            }
            _ => return PathCmd::Stop as u32,
        }
    }
}

fn calc_orthogonal(len: f64, x1: f64, y1: f64, x2: f64, y2: f64, dx_: &mut f64, dy_: &mut f64) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let d = (dx * dx + dy * dy).sqrt();
    *dx_ = dy * len / d;
    *dy_ = -dx * len / d;
}

pub struct DashedWedge {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    xt2: f64,
    yt2: f64,
    xt3: f64,
    yt3: f64,
    xd: [f64; 4],
    yd: [f64; 4],
    thickness: f64,
    num_dashes: u32,
    vertex: usize,
}

impl DashedWedge {
    pub fn new() -> DashedWedge {
        DashedWedge {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 0.0,
            thickness: 0.1,
            num_dashes: 8,
            xt2: 0.0,
            yt2: 0.0,
            xt3: 0.0,
            yt3: 0.0,
            xd: [0.0; 4],
            yd: [0.0; 4],
            vertex: 0,
        }
    }

    pub fn new_with_params(
        x1: f64, y1: f64, x2: f64, y2: f64, thickness: f64, num_dashes: u32,
    ) -> DashedWedge {
        DashedWedge {
            x1: x2,
            y1: y2,
            x2: x1,
            y2: y1,
            thickness: thickness,
            num_dashes: num_dashes,
            xt2: 0.0,
            yt2: 0.0,
            xt3: 0.0,
            yt3: 0.0,
            xd: [0.0; 4],
            yd: [0.0; 4],
            vertex: 0,
        }
    }

    pub fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.x1 = x2;
        self.y1 = y2;
        self.x2 = x1;
        self.y2 = y1;
    }

    pub fn num_dashes(&mut self, nd: u32) {
        self.num_dashes = nd;
    }

    pub fn thickness(&mut self, th: f64) {
        self.thickness = th;
    }
}

impl VertexSource for DashedWedge {
    fn rewind(&mut self, _start: u32) {
        let (mut dx, mut dy) = (0., 0.);
        calc_orthogonal(
            self.thickness * 2.0,
            self.x1,
            self.y1,
            self.x2,
            self.y2,
            &mut dx,
            &mut dy,
        );
        self.xt2 = self.x2 - dx;
        self.yt2 = self.y2 - dy;
        self.xt3 = self.x2 + dx;
        self.yt3 = self.y2 + dy;
        self.vertex = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.vertex < self.num_dashes as usize * 4 {
            if self.vertex % 4 == 0 {
                let k1 = (self.vertex / 4) as f64 / self.num_dashes as f64;
                let k2 = k1 + 0.4 / self.num_dashes as f64;

                self.xd[0] = self.x1 + (self.xt2 - self.x1) * k1;
                self.yd[0] = self.y1 + (self.yt2 - self.y1) * k1;
                self.xd[1] = self.x1 + (self.xt2 - self.x1) * k2;
                self.yd[1] = self.y1 + (self.yt2 - self.y1) * k2;
                self.xd[2] = self.x1 + (self.xt3 - self.x1) * k2;
                self.yd[2] = self.y1 + (self.yt3 - self.y1) * k2;
                self.xd[3] = self.x1 + (self.xt3 - self.x1) * k1;
                self.yd[3] = self.y1 + (self.yt3 - self.y1) * k1;
                *x = self.xd[0];
                *y = self.yd[0];
                self.vertex += 1;
                return PathCmd::MoveTo as u32;
            } else {
                *x = self.xd[self.vertex % 4];
                *y = self.yd[self.vertex % 4];
                self.vertex += 1;
                return PathCmd::LineTo as u32;
            }
        }
        return PathCmd::Stop as u32;
    }
}

enum BondStyle {
    Single,
    WedgedSolid,
    WedgedDashed,
    Double,
    DoubleLeft,
    DoubleRight,
    Triple,
}

struct BondVertexGenerator<'a> {
    bond: &'a Bond,
    thickness: f64,
    style: BondStyle,
    line1: Line,
    line2: Line,
    line3: Line,
    solid_wedge: SolidWedge,
    dashed_wedge: DashedWedge,
    status: u32,
}

impl<'a> BondVertexGenerator<'a> {
    pub fn new(bond: &Bond, thickness: f64) -> BondVertexGenerator {
        let mut style = BondStyle::Single;
        if bond.order == 1 {
            if bond.stereo == 1 {
                style = BondStyle::WedgedSolid;
            }
            if bond.stereo == 6 {
                style = BondStyle::WedgedDashed;
            }
        }
        if bond.order == 2 {
            style = BondStyle::Double;
            if bond.topology == 1 {
                style = BondStyle::DoubleLeft;
            }
            if bond.topology == 2 {
                style = BondStyle::DoubleRight;
            }
        }
        if bond.order == 3 {
            style = BondStyle::Triple;
        }
        let mut line1 = Line::new();
        line1.thickness(thickness);
        let mut line2 = Line::new();
        line2.thickness(thickness);
        let mut line3 = Line::new();
        line3.thickness(thickness);
        let mut solid_wedge = SolidWedge::new();
        solid_wedge.thickness(thickness);
        let mut dashed_wedge = DashedWedge::new();
        dashed_wedge.thickness(thickness);
        BondVertexGenerator {
            bond: bond,
            thickness: thickness,
            style: style,
            line1: line1,
            line2: line2,
            line3: line3,
            solid_wedge: solid_wedge,
            dashed_wedge: dashed_wedge,
            status: 0,
        }
    }
}

impl<'a> VertexSource for BondVertexGenerator<'a> {
    fn rewind(&mut self, _: u32) {
        let mut dx: f64 = 0.;
        let mut dy: f64 = 0.;
        let mut dx1: f64;
        let mut dy1: f64;
        let dx2: f64;
        let dy2: f64;
        match self.style {
            BondStyle::WedgedSolid => {
                self.solid_wedge
                    .init(self.bond.x1, self.bond.y1, self.bond.x2, self.bond.y2);
                self.solid_wedge.rewind(0);
            }
            BondStyle::WedgedDashed => {
                self.dashed_wedge
                    .init(self.bond.x1, self.bond.y1, self.bond.x2, self.bond.y2);
                self.dashed_wedge.rewind(0);
            }
            BondStyle::Double | BondStyle::DoubleLeft | BondStyle::DoubleRight => {
                calc_orthogonal(
                    self.thickness,
                    self.bond.x1,
                    self.bond.y1,
                    self.bond.x2,
                    self.bond.y2,
                    &mut dx,
                    &mut dy,
                );
                dx1 = dx;
                dx2 = dx;
                dy1 = dy;
                dy2 = dy;
                self.line1.init(
                    self.bond.x1 - dx1,
                    self.bond.y1 - dy1,
                    self.bond.x2 - dx1,
                    self.bond.y2 - dy1,
                );
                self.line1.rewind(0);
                self.line2.init(
                    self.bond.x1 + dx2,
                    self.bond.y1 + dy2,
                    self.bond.x2 + dx2,
                    self.bond.y2 + dy2,
                );
                self.line2.rewind(0);
                self.status = 0;
            }
            BondStyle::Triple => {}
            _ => {
                self.line1
                    .init(self.bond.x1, self.bond.y1, self.bond.x2, self.bond.y2);
                self.line1.rewind(0);
            }
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut flag = PathCmd::Stop as u32;
        match self.style {
            BondStyle::WedgedSolid => {
                return self.solid_wedge.vertex(x, y);
            }
            BondStyle::WedgedDashed => {
                return self.dashed_wedge.vertex(x, y);
            }
            BondStyle::DoubleLeft | BondStyle::DoubleRight | BondStyle::Double => {
                if self.status == 0 {
                    flag = self.line1.vertex(x, y);
                    if flag == PathCmd::Stop as u32 {
                        self.status = 1;
                    }
                }
                if self.status == 1 {
                    flag = self.line2.vertex(x, y);
                }
                return flag;
            }
            BondStyle::Triple => {}
            _ => {
                return self.line1.vertex(x, y);
            }
        }
        return self.line1.vertex(x, y);
    }
}

///////////
struct Application {
    molecules: Vec<Molecule>,
    num_molecules: u32,
    cur_molecule: u32,
    thickness: Ptr<Slider<'static, agg::Rgba8>>,
    text_size: Ptr<Slider<'static, agg::Rgba8>>,
    pdx: f64,
    pdy: f64,
    center_x: f64,
    center_y: f64,
    scale: f64,
    prev_scale: f64,
    angle: f64,
    prev_angle: f64,
    mouse_move: bool,
    atom_colors: [agg::Rgba8; End as usize],
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let mut molecules = Vec::new();
        let mut num_molecules = 0;
        let cur_molecule = 0;
        let thickness = ctrl_ptr(Slider::new(5., 5., START_WIDTH - 5., 12., !flip_y));
        let text_size = ctrl_ptr(Slider::new(5., 20., START_WIDTH - 5., 27., !flip_y));
        let pdx = 0.0;
        let pdy = 0.0;
        let center_x = START_WIDTH / 2.0;
        let center_y = START_HEIGHT / 2.0;
        let scale = 1.0;
        let prev_scale = 1.0;
        let angle = 0.0;
        let prev_angle = 0.0;
        let mouse_move = false;
        let mut atom_colors = [agg::Rgba8::new_params(0, 0, 0, 255); End as usize];
        atom_colors[ColorGeneral as usize] = agg::Rgba8::new_params(0, 0, 0, 255);
        atom_colors[ColorN as usize] = agg::Rgba8::new_params(0, 0, 120, 255);
        atom_colors[ColorO as usize] = agg::Rgba8::new_params(200, 0, 0, 255);
        atom_colors[ColorS as usize] = agg::Rgba8::new_params(120, 120, 0, 255);
        atom_colors[ColorP as usize] = agg::Rgba8::new_params(80, 50, 0, 255);
        atom_colors[ColorHalogen as usize] = agg::Rgba8::new_params(0, 200, 0, 255);

        thickness.borrow_mut().set_label("Thickness=%3.2f");
        text_size.borrow_mut().set_label("Label Size=%3.2f");

        let fd = File::open(FNAME);
        if fd.is_ok() {
            let mut buf_reader = BufReader::new(fd.unwrap());
            for _i in 0..100 {
                let mut molecule = Molecule::new();
                if !molecule.read(&mut buf_reader) {
                    break;
                }
                molecules.push(molecule);
                num_molecules += 1;
            }
        } else {
            let buf = format!(
                "File not found: '{}'. Download http://www.antigrain.com/{}\n",
                FNAME, FNAME
            );
            util.borrow_mut().message(&buf);
            panic!();
        }

        Application {
            molecules,
            num_molecules,
            cur_molecule,
            thickness: thickness.clone(),
            text_size: text_size.clone(),
            pdx,
            pdy,
            center_x,
            center_y,
            scale,
            prev_scale,
            angle,
            prev_angle,
            mouse_move,
            atom_colors,
            ctrls: CtrlContainer {
                ctrl: vec![thickness, text_size],
                cur_ctrl: -1,
                num_ctrl: 2,
            },
            util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let width = self.util.borrow().initial_width();
        let height = self.util.borrow().initial_height();

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineP8::new();
        ras.clip_box(0.0, 0.0, rbuf.width() as f64, rbuf.height() as f64);
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        let mol = &self.molecules[self.cur_molecule as usize];
        let mut min_x = 1e100;
        let mut max_x = -1e100;
        let mut min_y = 1e100;
        let mut max_y = -1e100;

        for i in 0..mol.num_atoms() {
            if mol.atom(i).x < min_x {
                min_x = mol.atom(i).x;
            }
            if mol.atom(i).y < min_y {
                min_y = mol.atom(i).y;
            }
            if mol.atom(i).x > max_x {
                max_x = mol.atom(i).x;
            }
            if mol.atom(i).y > max_y {
                max_y = mol.atom(i).y;
            }
        }

        let mut mtx = agg::TransAffine::new_default();

        mtx *= agg::TransAffine::trans_affine_translation(
            -((max_x + min_x) * 0.5),
            -((max_y + min_y) * 0.5),
        );

        let scale = width / (max_x - min_x);
        let t = height / (max_y - min_y);
        let scale = if scale > t { t } else { scale };

        let mut text_size = mol.average_bond_len() * self.text_size.borrow().value() / 4.0;
        let thickness = mol.average_bond_len() / (self.scale * self.scale).sqrt() / 8.0;

        mtx *= agg::TransAffine::trans_affine_scaling(scale * 0.80, scale * 0.80);
        mtx *= agg::TransAffine::trans_affine_rotation(self.angle);
        mtx *= agg::TransAffine::trans_affine_scaling(self.scale, self.scale);
        mtx *= agg::TransAffine::trans_affine_translation(self.center_x, self.center_y);
        mtx *= *self.util.borrow().trans_affine_resizing();

        rs.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
        for i in 0..mol.num_bonds() {
            let bond =
                BondVertexGenerator::new(mol.bond(i), self.thickness.borrow().value() * thickness);
            let mut tr = agg::ConvTransform::new_owned(bond, mtx);
            ras.add_path(&mut tr, 0);
            agg::render_scanlines(&mut ras, &mut sl, &mut rs);
        }

        let ell = agg::Ellipse::new();
        let mut tr = agg::ConvTransform::new_owned(ell, mtx);
        for i in 0..mol.num_atoms() {
            if mol.atom(i).label != "C" {
                tr.source_mut().init(
                    mol.atom(i).x,
                    mol.atom(i).y,
                    text_size * 2.5,
                    text_size * 2.5,
                    20,
                    false,
                );
                ras.add_path(&mut tr, 0);
                rs.set_color(agg::Rgba8::new_params(255, 255, 255, 255));
                agg::render_scanlines(&mut ras, &mut sl, &mut rs);
            }
        }

        text_size *= 3.0;

        let label = agg::GsvText::new();
        let mut ls: agg::ConvStroke<_> = agg::ConvStroke::new_owned(label);

        ls.set_line_join(agg::LineJoin::Round);
        ls.set_line_cap(agg::LineCap::Round);
        ls.set_approximation_scale(mtx.scale());
        let mut lo = agg::ConvTransform::new_owned(ls, mtx);

        for i in 0..mol.num_atoms() {
            if mol.atom(i).label != "C" {
                lo.source_mut()
                    .set_width(self.thickness.borrow().value() * thickness);
                lo.source_mut().source_mut().set_text(&mol.atom(i).label);
                lo.source_mut().source_mut().set_start_point(
                    mol.atom(i).x - text_size / 2.,
                    mol.atom(i).y - text_size / 2.,
                );
                lo.source_mut().source_mut().set_size(text_size, 0.);
                ras.add_path(&mut lo, 0);
                rs.set_color(self.atom_colors[mol.atom(i).color_idx as usize]);
                agg::render_scanlines(&mut ras, &mut sl, &mut rs);
            }
        }

        //
        let mut label = agg::GsvText::new();
        label.set_text(mol.name());
        label.set_size(10.0, 0.);
        label.set_start_point(10.0, START_HEIGHT - 20.0);

        let mut ls: agg::ConvStroke<_> = agg::ConvStroke::new_owned(label);

        ls.set_line_join(agg::LineJoin::Round);
        ls.set_line_cap(agg::LineCap::Round);
        ls.set_approximation_scale(mtx.scale());
        //
        ls.set_approximation_scale(1.0);
        ls.set_width(1.5);
        let mut name =
            agg::ConvTransform::new_owned(ls, *self.util.borrow().trans_affine_resizing());

        ras.reset();
        ras.add_path(&mut name, 0);
        rs.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
        agg::render_scanlines(&mut ras, &mut sl, &mut rs);

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.thickness.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.text_size.borrow_mut(),
        );
    }

    fn on_idle(&mut self) -> Draw {
        self.angle += agg::deg2rad(0.1);
        Draw::Yes
    }

    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, _flags: u32,
    ) -> Draw {
        self.mouse_move = true;
        let mut x2 = x as f64;
        let mut y2 = y as f64;
        self.util
            .borrow_mut()
            .trans_affine_resizing()
            .inverse_transform(&mut x2, &mut y2);

        self.pdx = self.center_x - x2;
        self.pdy = self.center_y - y2;
        self.prev_scale = self.scale;
        self.prev_angle = self.angle + PI;
        Draw::Yes
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.mouse_move = false;
        Draw::No
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        let mut x2 = x as f64;
        let mut y2 = y as f64;
        self.util
            .borrow_mut()
            .trans_affine_resizing()
            .inverse_transform(&mut x2, &mut y2);

        if self.mouse_move && (flags & InputFlag::MouseLeft as u32) != 0 {
            let dx = x2 - self.center_x;
            let dy = y2 - self.center_y;
            self.scale = self.prev_scale * (dx * dx + dy * dy).sqrt()
                / (self.pdx * self.pdx + self.pdy * self.pdy).sqrt();

            self.angle = self.prev_angle + dy.atan2(dx) - self.pdy.atan2(self.pdx);
            return Draw::Yes;
        }

        if self.mouse_move && (flags & InputFlag::MouseRight as u32) != 0 {
            self.center_x = x2 + self.pdx;
            self.center_y = y2 + self.pdy;
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        const LEFT: u32 = KeyCode::Left as u32;
        const UP: u32 = KeyCode::Up as u32;
        const PAGEUP: u32 = KeyCode::PageUp as u32;

        match key {
            LEFT | UP | PAGEUP => {
                if self.cur_molecule > 0 {
                    self.cur_molecule -= 1;
                }
                return Draw::Yes;
            }
            x if x == KeyCode::Right as u32
                || x == KeyCode::Down as u32
                || x == KeyCode::PageDown as u32 =>
            {
                if self.cur_molecule < self.num_molecules - 1 {
                    self.cur_molecule += 1;
                }
                return Draw::Yes;
            }
            x if x == ' ' as u32 => {
                let b = !self.util.borrow_mut().wait_mode();
                self.util.borrow_mut().set_wait_mode(b);
            }
            _ => {}
        }
        Draw::No
    }
}

fn main() {
    if !std::path::Path::new(FNAME).exists() {
        let buf = format!(
            "File not found: '{}'. Download http://www.antigrain.com/{}\n",
            FNAME, FNAME
        );
        println!("{}", &buf);
        return;
    }
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG - A Simple SDF Molecular Viewer");

    if plat.init(
        START_WIDTH as u32,
        START_HEIGHT as u32,
        WindowFlag::Resize as u32,
    ) {
        plat.run();
    }
}
