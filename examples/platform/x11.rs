#![cfg_attr(not(feature = "xlib"), allow(dead_code))]
#![cfg_attr(not(feature = "xlib"), allow(unused_imports))]
use crate::platform::*;
use core::ptr::*;
use std::mem::*;

use time_clock::{clock, CLOCKS_PER_SEC};

use libc::*;

use std::{cell::RefCell, rc::Rc, slice, str};

//use arr_macro::arr;
//use core::mem::MaybeUninit;
use agg::util::*;
use agg::RenderBuf;
use agg::RenderBuffer;
use agg::TransAffine;

use super::KeyCode;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::mem;
use std::ptr;

extern crate x11;

use std::ffi::CString;
use x11::keysym::*;
use x11::{xinput2::*, xlib, xlib::*};

#[derive(Clone, Copy)]
pub struct PlatSpecific {
    format: PixFormat,
    sys_format: PixFormat,
    byte_order: c_int,
    flip_y: bool,
    bpp: c_uint,
    sys_bpp: c_uint,
    display: *mut Display,
    screen: c_int,
    depth: c_int,
    visual: *mut Visual,
    window: Window,
    gc: GC,
    ximg_window: *mut XImage,
    window_attributes: XSetWindowAttributes,
    close_atom: Atom,
    buf_window: *mut c_uchar,
    buf_img: [*mut c_uchar; MAX_IMAGES as usize],
    keymap: [KeyCode; 256],
    update_flag: bool,
    resize_flag: bool,
    initialized: bool,
    sw_start: u64,
}

impl PlatSpecific {
    pub fn new(format: PixFormat, flip_y: bool) -> PlatSpecific {
        let mut platform_specific = PlatSpecific {
            format: format,
            sys_format: PixFormat::Undefined,
            byte_order: LSBFirst,
            flip_y: flip_y,
            bpp: 0,
            sys_bpp: 0,
            display: ptr::null_mut(),
            screen: 0,
            depth: 0,
            visual: ptr::null_mut(),
            window: 0,
            gc: ptr::null_mut(),
            ximg_window: ptr::null_mut(),
            window_attributes: XSetWindowAttributes {
                background_pixmap: 0,
                background_pixel: 0,
                border_pixmap: 0,
                border_pixel: 0,
                bit_gravity: 0,
                win_gravity: 0,
                backing_store: 0,
                backing_planes: 0,
                backing_pixel: 0,
                save_under: 0,
                event_mask: 0,
                do_not_propagate_mask: 0,
                override_redirect: 0,
                colormap: 0,
                cursor: 0,
            },
            close_atom: 0,
            buf_window: ptr::null_mut(),
            buf_img: [ptr::null_mut(); MAX_IMAGES as usize],
            keymap: [KeyCode::Pause; 256],
            update_flag: true,
            resize_flag: true,
            initialized: false,
            sw_start: 0,
        };
        for i in 0..256 {
            platform_specific.keymap[i] = unsafe { ::std::mem::transmute(i as u16) };
        }
        platform_specific.keymap[(XK_Pause & 0xFF) as usize] = KeyCode::Pause;
        platform_specific.keymap[(XK_Clear & 0xFF) as usize] = KeyCode::Clear;
        platform_specific.keymap[(XK_KP_0 & 0xFF) as usize] = KeyCode::Kp0;
        platform_specific.keymap[(XK_KP_1 & 0xFF) as usize] = KeyCode::Kp1;
        match platform_specific.format {
            PixFormat::Gray8 => platform_specific.bpp = 8,
            PixFormat::Rgb565 | PixFormat::Rgb555 => platform_specific.bpp = 16,
            PixFormat::Rgb24 | PixFormat::Bgr24 => platform_specific.bpp = 24,
            PixFormat::Bgra32 | PixFormat::Abgr32 | PixFormat::Argb32 | PixFormat::Rgba32 => {
                platform_specific.bpp = 32
            }
            _ => panic!("Format Error!"),
        }
        platform_specific.sw_start = clock();
        platform_specific
    }

    pub fn set_caption(&mut self, capt: &str) {
        let mut tp: XTextProperty = XTextProperty {
            value: ptr::null_mut(),
            encoding: 0,
            format: 0,
            nitems: 0,
        };
        tp.value = capt.as_ptr() as *mut c_uchar;
        tp.encoding = XA_WM_NAME;
        tp.format = 8;
        tp.nitems = capt.len() as c_ulong;
        unsafe {
            XSetWMName(self.display, self.window, &mut tp);
            XStoreName(self.display, self.window, capt.as_ptr() as *mut c_char);
            XSetIconName(self.display, self.window, capt.as_ptr() as *mut c_char);
            XSetWMIconName(self.display, self.window, &mut tp);
        }
    }

    fn put_image(&self, src: &RenderBuf) {
        if self.ximg_window.is_null() {
            return;
        }
        unsafe {
            (*self.ximg_window).data = self.buf_window as *mut c_char;
        }
        if self.format == self.sys_format {
            unsafe {
                XPutImage(
                    self.display,
                    self.window,
                    self.gc,
                    self.ximg_window,
                    0,
                    0,
                    0,
                    0,
                    src.width(),
                    src.height(),
                );
            }
        } else {
            let row_len = (src.width() * self.sys_bpp / 8) as i32;
            let buf_tmp = unsafe {
                std::alloc::alloc(
                    std::alloc::Layout::from_size_align(
                        (row_len as u32 * src.height()) as usize,
                        1,
                    )
                    .unwrap(),
                )
            } as *mut u8;
            let mut rbuf_tmp = RenderBuf::new(
                buf_tmp,
                src.width(),
                src.height(),
                if self.flip_y { -row_len } else { row_len },
            );
            match self.sys_format {
                PixFormat::Rgb555 => match self.format {
                    PixFormat::Rgb555 => {
                        color_conv(&mut rbuf_tmp, src, Rgb555ToRgb555::convert);
                    }
                    PixFormat::Rgb565 => {
                        color_conv(&mut rbuf_tmp, src, Rgb565ToRgb555::convert);
                    }
                    PixFormat::Rgb24 => {
                        color_conv(&mut rbuf_tmp, src, Rgb24ToRgb555::convert);
                    }
                    PixFormat::Bgr24 => {
                        color_conv(&mut rbuf_tmp, src, Bgr24ToRgb555::convert);
                    }
                    PixFormat::Rgba32 => {
                        color_conv(&mut rbuf_tmp, src, Rgba32ToRgb555::convert);
                    }
                    PixFormat::Argb32 => {
                        color_conv(&mut rbuf_tmp, src, Argb32ToRgb555::convert);
                    }
                    PixFormat::Bgra32 => {
                        color_conv(&mut rbuf_tmp, src, Bgra32ToRgb555::convert);
                    }
                    PixFormat::Abgr32 => {
                        color_conv(&mut rbuf_tmp, src, Abgr32ToRgb555::convert);
                    }
                    _ => {}
                },
                PixFormat::Rgb565 => match self.format {
                    PixFormat::Rgb555 => {
                        color_conv(&mut rbuf_tmp, src, Rgb555ToRgb565::convert);
                    }
                    PixFormat::Rgb565 => {
                        color_conv(&mut rbuf_tmp, src, Rgb565ToRgb565::convert);
                    }
                    PixFormat::Rgb24 => {
                        color_conv(&mut rbuf_tmp, src, Rgb24ToRgb565::convert);
                    }
                    PixFormat::Bgr24 => {
                        color_conv(&mut rbuf_tmp, src, Bgr24ToRgb565::convert);
                    }
                    PixFormat::Rgba32 => {
                        color_conv(&mut rbuf_tmp, src, Rgba32ToRgb565::convert);
                    }
                    PixFormat::Argb32 => {
                        color_conv(&mut rbuf_tmp, src, Argb32ToRgb565::convert);
                    }
                    PixFormat::Bgra32 => {
                        color_conv(&mut rbuf_tmp, src, Bgra32ToRgb565::convert);
                    }
                    PixFormat::Abgr32 => {
                        color_conv(&mut rbuf_tmp, src, Abgr32ToRgb565::convert);
                    }
                    _ => {}
                },
                PixFormat::Rgba32 => match self.format {
                    PixFormat::Rgb555 => {
                        color_conv(&mut rbuf_tmp, src, Rgb555ToRgba32::convert);
                    }
                    PixFormat::Rgb565 => {
                        color_conv(&mut rbuf_tmp, src, Rgb565ToRgba32::convert);
                    }
                    PixFormat::Rgb24 => {
                        color_conv(&mut rbuf_tmp, src, Rgb24ToRgba32::convert);
                    }
                    PixFormat::Bgr24 => {
                        color_conv(&mut rbuf_tmp, src, Bgr24ToRgba32::convert);
                    }
                    PixFormat::Rgba32 => {
                        color_conv(&mut rbuf_tmp, src, Rgba32ToRgba32::convert);
                    }
                    PixFormat::Argb32 => {
                        color_conv(&mut rbuf_tmp, src, Argb32ToRgba32::convert);
                    }
                    PixFormat::Bgra32 => {
                        color_conv(&mut rbuf_tmp, src, Bgra32ToRgba32::convert);
                    }
                    PixFormat::Abgr32 => {
                        color_conv(&mut rbuf_tmp, src, Abgr32ToRgba32::convert);
                    }
                    _ => {}
                },
                PixFormat::Abgr32 => match self.format {
                    PixFormat::Rgb555 => color_conv(&mut rbuf_tmp, src, Rgb555ToAbgr32::convert),
                    PixFormat::Rgb565 => color_conv(&mut rbuf_tmp, src, Rgb565ToAbgr32::convert),
                    PixFormat::Rgb24 => color_conv(&mut rbuf_tmp, src, Rgb24ToAbgr32::convert),
                    PixFormat::Bgr24 => color_conv(&mut rbuf_tmp, src, Bgr24ToAbgr32::convert),
                    PixFormat::Abgr32 => color_conv(&mut rbuf_tmp, src, Abgr32ToAbgr32::convert),
                    PixFormat::Rgba32 => color_conv(&mut rbuf_tmp, src, Rgba32ToAbgr32::convert),
                    PixFormat::Argb32 => color_conv(&mut rbuf_tmp, src, Argb32ToAbgr32::convert),
                    PixFormat::Bgra32 => color_conv(&mut rbuf_tmp, src, Bgra32ToAbgr32::convert),
                    _ => {}
                },

                PixFormat::Argb32 => match self.format {
                    PixFormat::Rgb555 => color_conv(&mut rbuf_tmp, src, Rgb555ToArgb32::convert),
                    PixFormat::Rgb565 => color_conv(&mut rbuf_tmp, src, Rgb565ToArgb32::convert),
                    PixFormat::Rgb24 => color_conv(&mut rbuf_tmp, src, Rgb24ToArgb32::convert),
                    PixFormat::Bgr24 => color_conv(&mut rbuf_tmp, src, Bgr24ToArgb32::convert),
                    PixFormat::Rgba32 => color_conv(&mut rbuf_tmp, src, Rgba32ToArgb32::convert),
                    PixFormat::Argb32 => color_conv(&mut rbuf_tmp, src, Argb32ToArgb32::convert),
                    PixFormat::Abgr32 => color_conv(&mut rbuf_tmp, src, Abgr32ToArgb32::convert),
                    PixFormat::Bgra32 => color_conv(&mut rbuf_tmp, src, Bgra32ToArgb32::convert),
                    _ => {}
                },

                PixFormat::Bgra32 => match self.format {
                    PixFormat::Rgb555 => color_conv(&mut rbuf_tmp, src, Rgb555ToBgra32::convert),
                    PixFormat::Rgb565 => color_conv(&mut rbuf_tmp, src, Rgb565ToBgra32::convert),
                    PixFormat::Rgb24 => color_conv(&mut rbuf_tmp, src, Rgb24ToBgra32::convert),
                    PixFormat::Bgr24 => color_conv(&mut rbuf_tmp, src, Bgr24ToBgra32::convert),
                    PixFormat::Rgba32 => color_conv(&mut rbuf_tmp, src, Rgba32ToBgra32::convert),
                    PixFormat::Argb32 => color_conv(&mut rbuf_tmp, src, Argb32ToBgra32::convert),
                    PixFormat::Abgr32 => color_conv(&mut rbuf_tmp, src, Abgr32ToBgra32::convert),
                    PixFormat::Bgra32 => color_conv(&mut rbuf_tmp, src, Bgra32ToBgra32::convert),
                    _ => {}
                },

                _ => {}
            }
            unsafe {
                (*self.ximg_window).data = buf_tmp as *mut c_char;
            }
            unsafe {
                XPutImage(
                    self.display,
                    self.window,
                    self.gc,
                    self.ximg_window,
                    0,
                    0,
                    0,
                    0,
                    src.width(),
                    src.height(),
                );
            }
        }
    }
}

const XEVENT_MASK: c_long = PointerMotionMask
    | ButtonPressMask
    | ButtonReleaseMask
    | ExposureMask
    | KeyPressMask
    | StructureNotifyMask;

impl PlatUtil {
    pub fn create_img(&mut self, idx: u32, width: u32, height: u32) -> bool {
        let (mut width, mut height) = (width, height);
        if idx < MAX_IMAGES {
            if width == 0 {
                width = self.width() as u32;
            }
            if height == 0 {
                height = self.height() as u32;
            }
            self.specific.buf_img[idx as usize] = unsafe {
                std::alloc::alloc(
                    std::alloc::Layout::from_size_align(
                        (width * height * (self.specific.bpp / 8)) as usize,
                        1,
                    )
                    .unwrap(),
                )
            } as *mut u8;

            self.rbuf_img[idx as usize].attach(
                self.specific.buf_img[idx as usize],
                width,
                height,
                if self.specific.flip_y {
                    -(width as i32 * (self.specific.bpp as i32 / 8))
                } else {
                    width as i32 * (self.specific.bpp as i32 / 8)
                },
            );
            true
        } else {
            false
        }
    }

    pub fn load_img(&mut self, id: u32, file: &str) -> bool {
        if id < MAX_IMAGES {
            let mut buf = [0u8; 1024];
            let mut f = String::from(file);
            let len = f.len();
            if len < 4 || &f[len - 4..] != ".ppm" {
                f.push_str(".ppm");
            }
            let mut fd = match File::open(f) {
                Ok(f) => f,
                Err(_) => return false,
            };
            let len = match fd.read(&mut buf[..1022]) {
                Ok(l) => l,
                Err(_) => return false,
            };
            buf[len] = 0;
            let mut idx = 0;
            if buf[0] != 'P' as u8 && buf[1] != '6' as u8 {
                return false;
            }
            idx += 2;
            while buf[idx] > 0 && !buf[idx].is_ascii_digit() {
                idx += 1;
            }

            if buf[idx] == 0 {
                return false;
            }

            fn getstr(buf: &[u8], idx: usize, len: usize) -> (&str, usize) {
                let mut i = 0;
                while idx + i < len && buf[idx + i].is_ascii_digit() {
                    i += 1;
                }
                (unsafe { str::from_utf8_unchecked(&buf[idx..idx + i]) }, i)
            }

            let (s, i) = getstr(&buf, idx, len);
            let st = s.parse::<u32>();
            let width = match st {
                Ok(w) => w,
                Err(_) => return false,
            };
            if width == 0 || width > 4096 {
                return false;
            }
            idx += i;
            while buf[idx] > 0 && !buf[idx].is_ascii_digit() {
                idx += 1;
            }
            if buf[idx] == 0 {
                return false;
            }
            let (s, i) = getstr(&buf, idx, len);
            let st = s.parse::<u32>();
            let height = match st {
                Ok(h) => h,
                Err(_) => return false,
            };
            if height == 0 || height > 4096 {
                return false;
            }
            idx += i;
            while buf[idx] > 0 && !buf[idx].is_ascii_digit() {
                idx += 1;
            }
            let (s, i) = getstr(&buf, idx, len);
            let st = s.parse::<u32>();
            if match st {
                Ok(v) => v,
                Err(_) => return false,
            } != 255
            {
                return false;
            }
            idx += i;
            if buf[idx] == 0 {
                return false;
            }
            idx += 1;
            match fd.seek(SeekFrom::Start(idx as u64)) {
                Ok(_) => (),
                Err(_) => return false,
            };
            self.create_img(id, width, height);
            let mut ret = true;
            let id = id as usize;
            let tmp = unsafe {
                std::slice::from_raw_parts_mut(
                    self.specific.buf_img[id],
                    (width * height * 3) as usize,
                )
            };
            if self.specific.format == PixFormat::Rgb24 {
                match fd.read(tmp) {
                    Ok(_) => (),
                    Err(_) => ret = false,
                };
            } else {
                let buf_img = unsafe {
                    std::alloc::alloc(
                        std::alloc::Layout::from_size_align((width * height * 3) as usize, 1)
                            .unwrap(),
                    )
                } as *mut u8;
                let rbuf_img = RenderBuf::new(
                    unsafe { &mut *buf_img },
                    width,
                    height,
                    if self.specific.flip_y {
                        -(width as i32 * 3)
                    } else {
                        width as i32 * 3
                    },
                );
                let tmp = unsafe {
                    std::slice::from_raw_parts_mut(buf_img, (width * height * 3) as usize)
                };
                match fd.read(tmp) {
                    Ok(_) => (),
                    Err(_) => ret = false,
                };
                match self.specific.format {
                    PixFormat::Rgb555 => {
                        color_conv(&mut self.rbuf_img[id], &rbuf_img, Rgb24ToRgb555::convert)
                    }
                    PixFormat::Rgb565 => {
                        color_conv(&mut self.rbuf_img[id], &rbuf_img, Rgb24ToRgb565::convert)
                    }
                    PixFormat::Bgr24 => {
                        color_conv(&mut self.rbuf_img[id], &rbuf_img, Rgb24ToBgr24::convert)
                    }
                    PixFormat::Rgba32 => {
                        color_conv(&mut self.rbuf_img[id], &rbuf_img, Rgb24ToRgba32::convert)
                    }
                    PixFormat::Argb32 => {
                        color_conv(&mut self.rbuf_img[id], &rbuf_img, Rgb24ToArgb32::convert)
                    }
                    PixFormat::Bgra32 => {
                        color_conv(&mut self.rbuf_img[id], &rbuf_img, Rgb24ToBgra32::convert)
                    }
                    PixFormat::Abgr32 => {
                        color_conv(&mut self.rbuf_img[id], &rbuf_img, Rgb24ToAbgr32::convert)
                    }
                    _ => ret = false,
                };
            }
            return ret;
        }
        return false;
    }

    pub fn save_img(&mut self, idx: u32, file: &str) -> bool {
        if idx < MAX_IMAGES && self.rbuf_img(idx).buf() != ptr::null_mut() {
            let mut buf = file.to_string();
            let len = buf.len();
            if len < 4 || buf[len - 4..].to_lowercase() != ".ppm" {
                buf.push_str(".ppm");
            }

            let fd = File::create(buf);
            if fd.is_err() {
                return false;
            }

            let w = self.rbuf_img(idx).width();
            let h = self.rbuf_img(idx).height() as i32;

            let mut f = fd.unwrap();
            f.write_fmt(format_args!("P6\n{} {}\n255\n", w, h)).unwrap();

            let tmp_buf = unsafe {
                std::alloc::alloc(std::alloc::Layout::from_size_align((w * 3) as usize, 1).unwrap())
            } as *mut u8;
            let tmp_buf_ref = unsafe { std::slice::from_raw_parts_mut(tmp_buf, (w * 3) as usize) };
            for y in 0..self.rbuf_img(idx).height() as i32 {
                let src = self
                    .rbuf_img(idx)
                    .row(if self.specific.flip_y { h - 1 - y } else { y });
                match self.specific.format {
                    PixFormat::Rgb555 => {
                        color_conv_row(tmp_buf_ref, src, w, Rgb555ToRgb24::convert);
                    }

                    PixFormat::Rgb565 => {
                        color_conv_row(tmp_buf_ref, src, w, Rgb565ToRgb24::convert);
                    }

                    PixFormat::Bgr24 => {
                        color_conv_row(tmp_buf_ref, src, w, Bgr24ToRgb24::convert);
                    }

                    PixFormat::Rgb24 => {
                        color_conv_row(tmp_buf_ref, src, w, Rgb24ToRgb24::convert);
                    }

                    PixFormat::Rgba32 => {
                        color_conv_row(tmp_buf_ref, src, w, Rgba32ToRgb24::convert);
                    }

                    PixFormat::Argb32 => {
                        color_conv_row(tmp_buf_ref, src, w, Argb32ToRgb24::convert);
                    }

                    PixFormat::Bgra32 => {
                        color_conv_row(tmp_buf_ref, src, w, Bgra32ToRgb24::convert);
                    }

                    PixFormat::Abgr32 => {
                        color_conv_row(tmp_buf_ref, src, w, Abgr32ToRgb24::convert);
                    }
                    _ => {}
                }
                f.write(tmp_buf_ref).unwrap();
            }
            true
        } else {
            false
        }
    }

    pub fn message(&self, msg: &str) {
        println!("{}", msg);
    }

    pub fn start_timer(&mut self) {
        self.specific.sw_start = clock();
    }

    pub fn elapsed_time(&self) -> f64 {
        let stop = clock();
        (stop - self.specific.sw_start) as f64 * 1000.0 / CLOCKS_PER_SEC as f64
    }

    pub fn img_ext(&self) -> &str {
        ".ppm"
    }
}

impl<App: Interface> PlatSupport<App> {
    pub fn new(demo: App, format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let ps = PlatSpecific::new(format, flip_y);
        let mbpp = ps.bpp;
        let mut plat = PlatSupport {
            //m_ctrls: ct,
            initial_height: 10,
            initial_width: 10,
            rbuf_img: [RenderBuf::new_default(); MAX_IMAGES as usize],
            rbuf_window: RenderBuf::new_default(),
            resize_mtx: TransAffine::new_default(),
            caption: "Anti-Grain Geometry Application".to_string(),
            specific: ps,
            format: format,
            bpp: mbpp,
            window_flags: 0,
            //m_wait_mode: true,
            flip_y: flip_y,
            demo: demo,
            util: util,
        };
        plat.demo.on_ctrls().set_transform(&plat.resize_mtx);
        plat
    }

    pub fn set_caption(&mut self, cap: &str) {
        self.caption = cap.to_string();
        if self.specific.initialized {
            self.specific.set_caption(cap);
        }
    }

    pub fn init(&mut self, width: u32, height: u32, flags: u32) -> bool {
        self.window_flags = flags;

        self.specific.display = unsafe { XOpenDisplay(ptr::null()) };
        if self.specific.display == 0 as *mut Display {
            println!("Unable to open DISPLAY!");
            return false;
        }

        self.specific.screen = unsafe { XDefaultScreen(self.specific.display) };
        self.specific.depth = unsafe { XDefaultDepth(self.specific.display, self.specific.screen) };
        self.specific.visual =
            unsafe { XDefaultVisual(self.specific.display, self.specific.screen) };
        let r_mask = unsafe { *self.specific.visual }.red_mask;
        let g_mask = unsafe { *self.specific.visual }.green_mask;
        let b_mask = unsafe { *self.specific.visual }.blue_mask;

        if self.specific.depth < 15 || r_mask == 0 || g_mask == 0 || b_mask == 0 {
            println!("There's no Visual compatible with minimal AGG requirements:\nAt least 15-bit color depth and True- or DirectColor class.\n\n");
            unsafe { XCloseDisplay(self.specific.display) };
            return false;
        }

        let t: u32 = 1;
        let mut hw_byte_order = LSBFirst;
        if unsafe { *(&t as *const u32 as *const u8) } == 0 {
            hw_byte_order = MSBFirst;
        }

        // Perceive SYS-format by mask
        match self.specific.depth {
            15 => {
                self.specific.sys_bpp = 16;
                if r_mask == 0x7C00 && g_mask == 0x3E0 && b_mask == 0x1F {
                    self.specific.sys_format = PixFormat::Rgb555;
                    self.specific.byte_order = hw_byte_order;
                }
            }

            16 => {
                self.specific.sys_bpp = 16;
                if r_mask == 0xF800 && g_mask == 0x7E0 && b_mask == 0x1F {
                    self.specific.sys_format = PixFormat::Rgb565;
                    self.specific.byte_order = hw_byte_order;
                }
            }

            24 | 32 => {
                self.specific.sys_bpp = 32;
                if g_mask == 0xFF00 {
                    if r_mask == 0xFF && b_mask == 0xFF0000 {
                        match self.specific.format {
                            PixFormat::Rgba32 => {
                                self.specific.sys_format = PixFormat::Rgba32;
                                self.specific.byte_order = LSBFirst;
                            }

                            PixFormat::Abgr32 => {
                                self.specific.sys_format = PixFormat::Abgr32;
                                self.specific.byte_order = MSBFirst;
                            }

                            _ => {
                                self.specific.byte_order = hw_byte_order;
                                self.specific.sys_format = if hw_byte_order == LSBFirst {
                                    PixFormat::Rgba32
                                } else {
                                    PixFormat::Abgr32
                                };
                            }
                        }
                    }

                    if r_mask == 0xFF0000 && b_mask == 0xFF {
                        match self.specific.format {
                            PixFormat::Argb32 => {
                                self.specific.sys_format = PixFormat::Argb32;
                                self.specific.byte_order = MSBFirst;
                            }

                            PixFormat::Bgra32 => {
                                self.specific.sys_format = PixFormat::Bgra32;
                                self.specific.byte_order = LSBFirst;
                            }

                            _ => {
                                self.specific.byte_order = hw_byte_order;
                                self.specific.sys_format = if hw_byte_order == MSBFirst {
                                    PixFormat::Argb32
                                } else {
                                    PixFormat::Bgra32
                                };
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if self.specific.sys_format == PixFormat::Undefined {
            println!("RGB masks are not compatible with AGG pixel formats:\nR={:08x}, R={:08x}, B={:08x}", r_mask, g_mask, b_mask);
            unsafe { XCloseDisplay(self.specific.display) };
            return false;
        }

        unsafe {
            std::ptr::write_bytes(
                &mut self.specific.window_attributes as *mut _ as *mut u8,
                0 as u8,
                mem::size_of::<XSetWindowAttributes>(),
            );
        }
        self.specific.window_attributes.border_pixel =
            unsafe { XBlackPixel(self.specific.display, self.specific.screen) };

        self.specific.window_attributes.background_pixel =
            unsafe { XWhitePixel(self.specific.display, self.specific.screen) };

        self.specific.window_attributes.override_redirect = 0;

        let window_mask = CWBackPixel | CWBorderPixel;

        self.specific.window = unsafe {
            XCreateWindow(
                self.specific.display,
                XDefaultRootWindow(self.specific.display),
                0,
                0,
                width,
                height,
                0,
                self.specific.depth,
                InputOutput as u32,
                null_mut(), //CopyFromParent,
                window_mask,
                (&mut self.specific.window_attributes) as *mut _,
            )
        };

        self.specific.gc = unsafe {
            XCreateGC(
                self.specific.display,
                self.specific.window,
                0,
                ptr::null_mut(),
            )
        };
        self.specific.buf_window = unsafe {
            std::alloc::alloc(
                std::alloc::Layout::from_size_align((width * height * (self.bpp / 8)) as usize, 1)
                    .unwrap(),
            )
        } as *mut u8;

        unsafe {
            std::ptr::write_bytes(
                self.specific.buf_window,
                255 as u8,
                (width * height * (self.bpp / 8)) as usize,
            );
        };

        self.rbuf_window.attach(
            unsafe { &mut *self.specific.buf_window },
            width,
            height,
            if self.flip_y {
                -((width * (self.bpp / 8)) as i32)
            } else {
                (width * (self.bpp / 8)) as i32
            },
        );

        self.specific.ximg_window = unsafe {
            XCreateImage(
                self.specific.display,
                self.specific.visual, //CopyFromParent,
                self.specific.depth as u32,
                ZPixmap,
                0,
                self.specific.buf_window as *mut c_char,
                width,
                height,
                self.specific.sys_bpp as i32,
                (width * (self.specific.sys_bpp / 8)) as i32,
            )
        };
        unsafe { *self.specific.ximg_window }.byte_order = self.specific.byte_order;

        self.specific.set_caption(&self.caption);
        self.initial_width = width;
        self.initial_height = height;
        self.util
            .borrow_mut()
            .plat_specific(&self.specific, width, height);
        if !self.specific.initialized {
            self.util
                .borrow_mut()
                .set_initial(self.initial_width, self.initial_height);
            self.demo.on_init();
            self.specific.initialized = true;
        }

        self.set_trans_affine_resizing(width as i32, height as i32);
        self.demo.on_resize(width as i32, height as i32);
        self.specific.update_flag = true;

        let hints = unsafe { XAllocSizeHints() };
        if !hints.is_null() {
            if flags & WindowFlag::Resize as u32 != 0 {
                unsafe {
                    (*hints).min_width = 32;
                    (*hints).min_height = 32;
                    (*hints).max_width = 4096;
                    (*hints).max_height = 4096;
                }
            } else {
                unsafe {
                    (*hints).min_width = width as i32;
                    (*hints).min_height = height as i32;
                    (*hints).max_width = width as i32;
                    (*hints).max_height = height as i32;
                }
            }
            unsafe {
                (*hints).flags = PMaxSize | PMinSize;

                XSetWMNormalHints(self.specific.display, self.specific.window, hints);
            }

            unsafe { XFree(hints as *mut c_void) };
        }

        unsafe {
            XMapWindow(self.specific.display, self.specific.window);
        }

        unsafe {
            XSelectInput(self.specific.display, self.specific.window, XEVENT_MASK);
        }

        self.specific.close_atom = unsafe {
            XInternAtom(
                self.specific.display,
                "WM_DELETE_WINDOW\0".as_ptr() as *const c_char,
                0,
            )
        };

        unsafe {
            XSetWMProtocols(
                self.specific.display,
                self.specific.window,
                (&mut self.specific.close_atom) as *mut _,
                1,
            );
        }

        true
    }

    pub fn update_window(&mut self) {
        self.specific.put_image(&self.rbuf_window);

        // When wait_mode is true we can discard all the events
        // came while the image is being drawn. In this case
        // the X server does not accumulate mouse motion events.
        // When wait_mode is false, i.e. we have some idle drawing
        // we cannot afford to miss any events
        unsafe { XSync(self.specific.display, self.util.borrow().wait_mode() as i32) };
    }

    #[allow(non_upper_case_globals)]
    pub fn run(&mut self) -> i32 {
        unsafe {
            XFlush(self.specific.display);
        }

        let mut quit = false;
        let mut flags: u32;
        let mut cur_x: i32;
        let mut cur_y: i32;
        fn dr<App: Interface>(r: Draw, app: &mut PlatSupport<App>) {
            match r {
                Draw::Yes => app.force_redraw(),
                Draw::Update => app.update_window(),
                Draw::No => (),
            }
        }

        while !quit {
            if self.specific.update_flag {
                self.demo.on_draw(&mut self.rbuf_window);
                self.update_window();
                self.specific.update_flag = false;
            }

            if !self.util.borrow().wait_mode() {
                if unsafe { XPending(self.specific.display) } == 0 {
                    self.demo.on_idle();
                    continue;
                }
            }

            let mut x_event: XEvent = unsafe { mem::zeroed() };
            unsafe {
                XNextEvent(self.specific.display, &mut x_event);
            }

            // In the Idle mode discard all intermediate MotionNotify events
            if !self.util.borrow().wait_mode() && unsafe { x_event.type_ } == MotionNotify {
                let mut te = x_event;
                loop {
                    if unsafe { XPending(self.specific.display) } == 0 {
                        break;
                    }
                    unsafe {
                        XNextEvent(self.specific.display, &mut te);
                    }
                    if unsafe { te.type_ } != MotionNotify {
                        break;
                    }
                }
                x_event = te;
            }

            match unsafe { x_event.type_ } {
                ConfigureNotify => {
                    if unsafe { x_event.configure.width } != self.rbuf_window.width() as i32
                        || unsafe { x_event.configure.height } != self.rbuf_window.height() as i32
                    {
                        let width = unsafe { x_event.configure.width };
                        let height = unsafe { x_event.configure.height };

                        unsafe {
                            self.specific.buf_window = std::alloc::alloc(
                                std::alloc::Layout::from_size_align(
                                    (width * height * (self.bpp as i32 / 8)) as usize,
                                    1,
                                )
                                .unwrap(),
                            ) as *mut u8;
                        }

                        unsafe {
                            (*self.specific.ximg_window).data = ptr::null_mut();
                        }
                        unsafe {
                            XDestroyImage(self.specific.ximg_window);
                        }

                        self.rbuf_window.attach(
                            self.specific.buf_window,
                            width as u32,
                            height as u32,
                            if self.flip_y {
                                -(width * (self.bpp as i32 / 8))
                            } else {
                                width * (self.bpp as i32 / 8)
                            },
                        );

                        self.specific.ximg_window = unsafe {
                            XCreateImage(
                                self.specific.display,
                                self.specific.visual, //CopyFromParent,
                                self.specific.depth as u32,
                                ZPixmap,
                                0,
                                self.specific.buf_window as *mut c_char,
                                width as u32,
                                height as u32,
                                self.specific.sys_bpp as i32,
                                width * (self.specific.sys_bpp as i32 / 8),
                            )
                        };
                        unsafe {
                            (*self.specific.ximg_window).byte_order = self.specific.byte_order;
                        }

                        self.set_trans_affine_resizing(width as i32, height as i32);
                        self.demo.on_resize(width as i32, height as i32);
                        self.demo.on_draw(&mut self.rbuf_window);
                        self.update_window();
                    }
                }

                Expose => {
                    self.specific.put_image(&self.rbuf_window);
                    unsafe {
                        XFlush(self.specific.display);
                        XSync(self.specific.display, false as c_int);
                    }
                }

                KeyPress => {
                    let key = unsafe { XLookupKeysym((&mut x_event.key) as *mut _, 0) };
                    flags = 0;
                    if unsafe { x_event.key.state } & Button1Mask != 0 {
                        flags |= InputFlag::MouseLeft as u32;
                    }
                    if unsafe { x_event.key.state } & Button3Mask != 0 {
                        flags |= InputFlag::MouseRight as u32;
                    }
                    if unsafe { x_event.key.state } & ShiftMask != 0 {
                        flags |= InputFlag::KbdShift as u32;
                    }
                    if unsafe { x_event.key.state } & ControlMask != 0 {
                        flags |= InputFlag::KbdCtrl as u32;
                    }

                    let mut left = false;
                    let mut up = false;
                    let mut right = false;
                    let mut down = false;

                    match self.specific.keymap[(key & 0xFF) as usize] {
                        KeyCode::Left => {
                            left = true;
                        }

                        KeyCode::Up => {
                            up = true;
                        }

                        KeyCode::Right => {
                            right = true;
                        }

                        KeyCode::Down => {
                            down = true;
                        }

                        KeyCode::F2 => {
                            self.util
                                .borrow_mut()
                                .copy_window_to_img(&mut self.rbuf_window, MAX_IMAGES - 1);
                            self.util
                                .borrow_mut()
                                .save_img(MAX_IMAGES - 1, "screenshot");
                        }

                        _ => {}
                    }

                    if self.demo.on_ctrls().on_arrow_keys(left, right, down, up) {
                        self.demo.on_ctrl_change(&mut self.rbuf_window);
                        self.force_redraw();
                    } else {
                        let h = self.rbuf_window.height() as i32;
                        let k = self.specific.keymap[(key & 0xFF) as usize] as u32;
                        let r = self.demo.on_key(
                            &mut self.rbuf_window,
                            unsafe { x_event.key.x },
                            if self.flip_y {
                                h - unsafe { x_event.key.y }
                            } else {
                                unsafe { x_event.key.y }
                            },
                            k,
                            flags,
                        );
                        dr(r, self);
                    }
                }

                ButtonPress => {
                    flags = 0;
                    if unsafe { x_event.button.state } & ShiftMask != 0 {
                        flags |= InputFlag::KbdShift as u32;
                    }
                    if unsafe { x_event.button.state } & ControlMask != 0 {
                        flags |= InputFlag::KbdCtrl as u32;
                    }
                    if unsafe { x_event.button.button } == Button1 {
                        flags |= InputFlag::MouseLeft as u32;
                    }
                    if unsafe { x_event.button.button } == Button3 {
                        flags |= InputFlag::MouseRight as u32;
                    }

                    cur_x = unsafe { x_event.button.x };
                    cur_y = if self.flip_y {
                        self.rbuf_window.height() as i32 - unsafe { x_event.button.y }
                    } else {
                        unsafe { x_event.button.y }
                    };

                    if flags & InputFlag::MouseLeft as u32 != 0 {
                        if self
                            .demo
                            .on_ctrls()
                            .on_mouse_button_down(cur_x as f64, cur_y as f64)
                        {
                            self.demo.on_ctrls().set_cur(cur_x as f64, cur_y as f64);
                            self.demo.on_ctrl_change(&mut self.rbuf_window);
                            self.force_redraw();
                        } else {
                            if self.demo.on_ctrls().in_rect(cur_x as f64, cur_y as f64) {
                                if self.demo.on_ctrls().set_cur(cur_x as f64, cur_y as f64) {
                                    self.demo.on_ctrl_change(&mut self.rbuf_window);
                                    self.force_redraw();
                                }
                            } else {
                                let r = self.demo.on_mouse_button_down(
                                    &mut self.rbuf_window,
                                    cur_x,
                                    cur_y,
                                    flags,
                                );
                                dr(r, self);
                            }
                        }
                    }
                    if flags & InputFlag::MouseRight as u32 != 0 {
                        self.demo
                            .on_mouse_button_down(&mut self.rbuf_window, cur_x, cur_y, flags);
                    }
                    //self.specific->m_wait_mode = wait_mode;
                    //m_wait_mode = true;
                }
                xlib::MotionNotify => {
                    flags = 0;
                    if unsafe { x_event.motion.state } & xlib::Button1Mask != 0 {
                        flags |= InputFlag::MouseLeft as u32;
                    }
                    if unsafe { x_event.motion.state } & xlib::Button3Mask != 0 {
                        flags |= InputFlag::MouseRight as u32;
                    }
                    if unsafe { x_event.motion.state } & xlib::ShiftMask != 0 {
                        flags |= InputFlag::KbdShift as u32;
                    }
                    if unsafe { x_event.motion.state } & xlib::ControlMask != 0 {
                        flags |= InputFlag::KbdCtrl as u32;
                    }

                    cur_x = unsafe { x_event.button.x };
                    cur_y = if self.flip_y {
                        self.rbuf_window.height() as i32 - unsafe { x_event.button.y }
                    } else {
                        unsafe { x_event.button.y }
                    };

                    if self.demo.on_ctrls().on_mouse_move(
                        cur_x as f64,
                        cur_y as f64,
                        flags & InputFlag::MouseLeft as u32 != 0,
                    ) {
                        self.demo.on_ctrl_change(&mut self.rbuf_window);
                        self.force_redraw();
                    } else {
                        if !self.demo.on_ctrls().in_rect(cur_x as f64, cur_y as f64) {
                            let r =
                                self.demo
                                    .on_mouse_move(&mut self.rbuf_window, cur_x, cur_y, flags);
                            dr(r, self);
                        }
                    }
                }
                xlib::ButtonRelease => {
                    flags = 0;
                    if unsafe { x_event.button.state } & xlib::ShiftMask != 0 {
                        flags |= InputFlag::KbdShift as u32;
                    }
                    if unsafe { x_event.button.state } & xlib::ControlMask != 0 {
                        flags |= InputFlag::KbdCtrl as u32;
                    }
                    if unsafe { x_event.button.button } == xlib::Button1 {
                        flags |= InputFlag::MouseLeft as u32;
                    }
                    if unsafe { x_event.button.button } == xlib::Button3 {
                        flags |= InputFlag::MouseRight as u32;
                    }

                    cur_x = unsafe { x_event.button.x };
                    cur_y = if self.flip_y {
                        self.rbuf_window.height() as i32 - unsafe { x_event.button.y }
                    } else {
                        unsafe { x_event.button.y }
                    };

                    if flags & InputFlag::MouseLeft as u32 != 0 {
                        if self
                            .demo
                            .on_ctrls()
                            .on_mouse_button_up(cur_x as f64, cur_y as f64)
                        {
                            self.demo.on_ctrl_change(&mut self.rbuf_window);
                            self.force_redraw();
                        }
                    }
                    if flags & (InputFlag::MouseLeft as u32 | InputFlag::MouseRight as u32) != 0 {
                        let r = self.demo.on_mouse_button_up(
                            &mut self.rbuf_window,
                            cur_x,
                            cur_y,
                            flags,
                        );
                        dr(r, self);
                    }
                }
                xlib::ClientMessage => {
                    if unsafe { x_event.client_message.format } == 32
                        && unsafe { x_event.client_message.data.get_long(0) }
                            == self.specific.close_atom as c_long
                    {
                        quit = true;
                    }
                }
                _ => {}
            }
        }

        let mut i = MAX_IMAGES as usize;
        /*while i > 0 {
            i -= 1;
            if self.specific.buf_img[i] != ptr::null_mut() {
                unsafe {
                    libc::free(self.specific.buf_img[i] as *mut libc::c_void);
                }
            }
        }*/

        unsafe {
            //libc::free(self.specific.buf_window as *mut libc::c_void);

            (*self.specific.ximg_window).data = ptr::null_mut();

            xlib::XDestroyImage(self.specific.ximg_window);

            xlib::XFreeGC(self.specific.display, self.specific.gc);

            xlib::XDestroyWindow(self.specific.display, self.specific.window);

            xlib::XCloseDisplay(self.specific.display);
        }
        0
    }

    /*pub fn full_file_name(&self, file_name: &str) -> &str {
        file_name
    }*/

    pub fn force_redraw(&mut self) {
        self.specific.update_flag = true;
    }
}
