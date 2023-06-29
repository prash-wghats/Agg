use crate::platform::*;
use core::ptr::*;
use std::mem;
use std::{cell::RefCell, rc::Rc, str};

use libc::*;

use windows_sys::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::LibraryLoader::*,
    Win32::System::Performance::*, Win32::System::SystemServices::*,
    Win32::UI::Input::KeyboardAndMouse::*, Win32::UI::WindowsAndMessaging::*,
};

use agg::util::*;
use agg::RenderBuf;
use agg::RenderBuffer;
use agg::TransAffine;

use super::KeyCode;

mod bmp;
use bmp::*;

macro_rules! loword {
    ($v:expr) => {
        $v as usize & 0xffff
    };
}

macro_rules! hiword {
    ($v:expr) => {
        ($v as usize >> 16) & 0xffff
    };
}

#[derive(Clone, Copy)]
pub struct PlatSpecific {
    format: PixFormat,
    sys_format: PixFormat,
    flip_y: bool,
    //g_windows_instance: HMODULE,
    //g_windows_cmd_show: i32,
    bpp: u32,
    sys_bpp: u32,
    hwnd: HWND,
    pmap_window: PixelMap,
    pmap_img: [PixelMap; MAX_IMAGES as usize],
    keymap: [KeyCode; 256],
    last_translated_key: u32,
    cur_x: i32,
    cur_y: i32,
    input_flags: u32,
    redraw_flag: bool,
    current_dc: HDC,
    sw_freq: i64,
    sw_start: i64,
}

impl PlatSpecific {
    pub fn new(format: PixFormat, flip_y: bool) -> PlatSpecific {
        let mut ps = Self {
            format: format,
            sys_format: PixFormat::Undefined,
            flip_y: flip_y,
            bpp: 0,
            sys_bpp: 0,
            hwnd: 0,
            pmap_window: PixelMap::new(),
            pmap_img: [PixelMap::new(); MAX_IMAGES as usize],
            last_translated_key: 0,
            cur_x: 0,
            cur_y: 0,
            input_flags: 0,
            redraw_flag: true,
            current_dc: 0,
            keymap: [KeyCode::Undefined; 256],
            sw_freq: 0,
            sw_start: 0,
        };

        ps.keymap[VK_PAUSE as usize] = KeyCode::Pause;
        ps.keymap[VK_CLEAR as usize] = KeyCode::Clear;

        ps.keymap[VK_NUMPAD0 as usize] = KeyCode::Kp0;
        ps.keymap[VK_NUMPAD1 as usize] = KeyCode::Kp1;
        ps.keymap[VK_NUMPAD2 as usize] = KeyCode::Kp2;
        ps.keymap[VK_NUMPAD3 as usize] = KeyCode::Kp3;
        ps.keymap[VK_NUMPAD4 as usize] = KeyCode::Kp4;
        ps.keymap[VK_NUMPAD5 as usize] = KeyCode::Kp5;
        ps.keymap[VK_NUMPAD6 as usize] = KeyCode::Kp6;
        ps.keymap[VK_NUMPAD7 as usize] = KeyCode::Kp7;
        ps.keymap[VK_NUMPAD8 as usize] = KeyCode::Kp8;
        ps.keymap[VK_NUMPAD9 as usize] = KeyCode::Kp9;
        ps.keymap[VK_DECIMAL as usize] = KeyCode::KpPeriod;
        ps.keymap[VK_DIVIDE as usize] = KeyCode::KpDivide;
        ps.keymap[VK_MULTIPLY as usize] = KeyCode::KpMultiply;
        ps.keymap[VK_SUBTRACT as usize] = KeyCode::KpMinus;
        ps.keymap[VK_ADD as usize] = KeyCode::KpPlus;

        ps.keymap[VK_UP as usize] = KeyCode::Up;
        ps.keymap[VK_DOWN as usize] = KeyCode::Down;
        ps.keymap[VK_RIGHT as usize] = KeyCode::Right;
        ps.keymap[VK_LEFT as usize] = KeyCode::Left;
        ps.keymap[VK_INSERT as usize] = KeyCode::Insert;
        ps.keymap[VK_DELETE as usize] = KeyCode::Delete;
        ps.keymap[VK_HOME as usize] = KeyCode::Home;
        ps.keymap[VK_END as usize] = KeyCode::End;
        ps.keymap[VK_PRIOR as usize] = KeyCode::PageUp;
        ps.keymap[VK_NEXT as usize] = KeyCode::PageDown;

        ps.keymap[VK_F1 as usize] = KeyCode::F1;
        ps.keymap[VK_F2 as usize] = KeyCode::F2;
        ps.keymap[VK_F3 as usize] = KeyCode::F3;
        ps.keymap[VK_F4 as usize] = KeyCode::F4;
        ps.keymap[VK_F5 as usize] = KeyCode::F5;
        ps.keymap[VK_F6 as usize] = KeyCode::F6;
        ps.keymap[VK_F7 as usize] = KeyCode::F7;
        ps.keymap[VK_F8 as usize] = KeyCode::F8;
        ps.keymap[VK_F9 as usize] = KeyCode::F9;
        ps.keymap[VK_F10 as usize] = KeyCode::F10;
        ps.keymap[VK_F11 as usize] = KeyCode::F11;
        ps.keymap[VK_F12 as usize] = KeyCode::F12;
        ps.keymap[VK_F13 as usize] = KeyCode::F13;
        ps.keymap[VK_F14 as usize] = KeyCode::F14;
        ps.keymap[VK_F15 as usize] = KeyCode::F15;

        ps.keymap[VK_NUMLOCK as usize] = KeyCode::Numlock;
        ps.keymap[VK_CAPITAL as usize] = KeyCode::Capslock;
        ps.keymap[VK_SCROLL as usize] = KeyCode::Scrollock;

        match ps.format {
            PixFormat::Bw => {
                ps.sys_format = PixFormat::Bw;
                ps.bpp = 1;
                ps.sys_bpp = 1;
            }
            PixFormat::Gray8 => {
                ps.sys_format = PixFormat::Gray8;
                ps.bpp = 8;
                ps.sys_bpp = 8;
            }
            PixFormat::Gray16 => {
                ps.sys_format = PixFormat::Gray8;
                ps.bpp = 16;
                ps.sys_bpp = 8;
            }
            PixFormat::Rgb565 | PixFormat::Rgb555 => {
                ps.sys_format = PixFormat::Rgb555;
                ps.bpp = 16;
                ps.sys_bpp = 16;
            }
            PixFormat::RgbAAA | PixFormat::BgrAAA | PixFormat::RgbBBA | PixFormat::BgrABB => {
                ps.sys_format = PixFormat::Bgr24;
                ps.bpp = 32;
                ps.sys_bpp = 24;
            }
            PixFormat::Rgb24 | PixFormat::Bgr24 => {
                ps.sys_format = PixFormat::Bgr24;
                ps.bpp = 24;
                ps.sys_bpp = 24;
            }
            PixFormat::Rgb48 | PixFormat::Bgr48 => {
                ps.sys_format = PixFormat::Bgr24;
                ps.bpp = 48;
                ps.sys_bpp = 24;
            }
            PixFormat::Bgra32 | PixFormat::Abgr32 | PixFormat::Argb32 | PixFormat::Rgba32 => {
                ps.sys_format = PixFormat::Bgra32;
                ps.bpp = 32;
                ps.sys_bpp = 32;
            }
            PixFormat::Bgra64 | PixFormat::Abgr64 | PixFormat::Argb64 | PixFormat::Rgba64 => {
                ps.sys_format = PixFormat::Bgra32;
                ps.bpp = 64;
                ps.sys_bpp = 32;
            }
            _ => {}
        }
        unsafe {
            QueryPerformanceFrequency(&mut ps.sw_freq);
            QueryPerformanceCounter(&mut ps.sw_start);
        }
        ps
    }

    fn create_pmap(&mut self, width: u32, height: u32, wnd: &mut RenderBuf) {
        self.pmap_window.create(
            width,
            height,
            unsafe { ::std::mem::transmute(self.bpp) },
            256,
        );
        wnd.attach(
            self.pmap_window.buf(),
            self.pmap_window.width(),
            self.pmap_window.height(),
            if self.flip_y {
                self.pmap_window.stride()
            } else {
                -self.pmap_window.stride()
            },
        );
    }

    fn convert_pmap(dst: &mut RenderBuf, src: &RenderBuf, format: PixFormat) {
        match format {
            PixFormat::Gray8 => {}
            PixFormat::Gray16 => {
                color_conv(dst, src, Gray16ToGray8::convert);
            }
            PixFormat::Rgb565 => {
                color_conv(dst, src, Rgb565ToRgb555::convert);
            }
            PixFormat::RgbAAA => {
                color_conv(dst, src, RgbAAAToBgr24::convert);
            }
            PixFormat::BgrAAA => {
                color_conv(dst, src, BgrAAAToBgr24::convert);
            }
            PixFormat::RgbBBA => {
                color_conv(dst, src, RgbBBAToBgr24::convert);
            }
            PixFormat::BgrABB => {
                color_conv(dst, src, BgrABBToBgr24::convert);
            }
            PixFormat::Rgb24 => {
                color_conv(dst, src, Rgb24ToBgr24::convert);
            }
            PixFormat::Rgb48 => {
                color_conv(dst, src, Rgb48ToBgr24::convert);
            }
            PixFormat::Bgr48 => {
                color_conv(dst, src, Bgr48ToBgr24::convert);
            }
            PixFormat::Abgr32 => {
                color_conv(dst, src, Abgr32ToBgra32::convert);
            }
            PixFormat::Argb32 => {
                color_conv(dst, src, Argb32ToBgra32::convert);
            }
            PixFormat::Rgba32 => {
                color_conv(dst, src, Rgba32ToBgra32::convert);
            }
            PixFormat::Bgra64 => {
                color_conv(dst, src, Bgra64ToBgra32::convert);
            }
            PixFormat::Abgr64 => {
                color_conv(dst, src, Abgr64ToBgra32::convert);
            }
            PixFormat::Argb64 => {
                color_conv(dst, src, Argb64ToBgra32::convert);
            }
            PixFormat::Rgba64 => {
                color_conv(dst, src, Rgba64ToBgra32::convert);
            }
            _ => {}
        }
    }

    fn display_pmap(&mut self, dc: HDC, src: &RenderBuf) {
        if self.sys_format == self.format {
            self.pmap_window.draw(dc, None, None);
        } else {
            let mut pmap_tmp = PixelMap::new();
            pmap_tmp.create(
                self.pmap_window.width(),
                self.pmap_window.height(),
                unsafe { ::std::mem::transmute(self.sys_bpp) },
                256,
            );

            let mut rbuf_tmp = RenderBuf::new_default();
            rbuf_tmp.attach(
                pmap_tmp.buf(),
                pmap_tmp.width(),
                pmap_tmp.height(),
                if self.flip_y {
                    pmap_tmp.stride()
                } else {
                    -pmap_tmp.stride()
                },
            );

            Self::convert_pmap(&mut rbuf_tmp, src, self.format);
            pmap_tmp.draw(dc, None, None);
        }
    }

    fn save_pmap(&mut self, na: &str, idx: u32, src: &RenderBuf)  -> bool {
        if self.sys_format == self.format {
            return self.pmap_img[idx as usize].save_as_bmp_str(na);
        }

        let mut pmap_tmp = PixelMap::new();
        pmap_tmp.create(
            self.pmap_img[idx as usize].width(),
            self.pmap_img[idx as usize].height(),
            unsafe { ::std::mem::transmute(self.sys_bpp) },
            256,
        );

        let mut rbuf_tmp = RenderBuf::new_default();
        rbuf_tmp.attach(
            pmap_tmp.buf(),
            pmap_tmp.width(),
            pmap_tmp.height(),
            if self.flip_y {
                pmap_tmp.stride()
            } else {
                -pmap_tmp.stride()
            },
        );

        Self::convert_pmap(&mut rbuf_tmp, src, self.format);
        return pmap_tmp.save_as_bmp_str(na);
    }

    fn load_pmap(&mut self, na: &str, idx: u32, dst: &mut RenderBuf)  -> bool {
        let mut pmap_tmp = PixelMap::new();
        if !pmap_tmp.load_from_bmp_str(na) {
            return false;
        }

        let mut rbuf_tmp = RenderBuf::new_default();
        rbuf_tmp.attach(
            pmap_tmp.buf(),
            pmap_tmp.width(),
            pmap_tmp.height(),
            if self.flip_y {
                pmap_tmp.stride()
            } else {
                -pmap_tmp.stride()
            },
        );

        self.pmap_img[idx as usize].create(
            pmap_tmp.width(),
            pmap_tmp.height(),
            unsafe { ::std::mem::transmute(self.bpp) },
            0,
        );

        dst.attach(
            self.pmap_img[idx as usize].buf(),
            self.pmap_img[idx as usize].width(),
            self.pmap_img[idx as usize].height(),
            if self.flip_y {
                self.pmap_img[idx as usize].stride()
            } else {
                -self.pmap_img[idx as usize].stride()
            },
        );

        match self.format {
            PixFormat::Gray8 => match pmap_tmp.bpp() {
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToGray8::convert),
                _ => (),
            },
            PixFormat::Gray16 => match pmap_tmp.bpp() {
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToGray16::convert),
                _ => (),
            },
            PixFormat::Rgb555 => match pmap_tmp.bpp() {
                16 => color_conv(dst, &rbuf_tmp, Rgb555ToRgb555::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToRgb555::convert),
                32 => color_conv(dst, &rbuf_tmp, Bgra32ToRgb555::convert),
                _ => (),
            },
            PixFormat::Rgb565 => match pmap_tmp.bpp() {
                16 => color_conv(dst, &rbuf_tmp, Rgb555ToRgb565::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToRgb565::convert),
                32 => color_conv(dst, &rbuf_tmp, Bgra32ToRgb565::convert),
                _ => (),
            },
            PixFormat::Rgb24 => match pmap_tmp.bpp() {
                16 => color_conv(dst, &rbuf_tmp, Rgb555ToRgb24::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToRgb24::convert),
                32 => color_conv(dst, &rbuf_tmp, Bgra32ToRgb24::convert),
                _ => (),
            },
            PixFormat::Bgr24 => match pmap_tmp.bpp() {
                16 => color_conv(dst, &rbuf_tmp, Rgb555ToBgr24::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToBgr24::convert),
                32 => color_conv(dst, &rbuf_tmp, Bgra32ToBgr24::convert),
                _ => (),
            },
            PixFormat::Rgb48 => {
                match pmap_tmp.bpp() {
                    //16 => color_conv(dst, &rbuf_tmp, color_conv_rgb555_to_rgb48()),
                    24 => color_conv(dst, &rbuf_tmp, Bgr24ToRgb48::convert),
                    //32 => color_conv(dst, &rbuf_tmp, color_conv_bgra32_to_rgb48()),
                    _ => {}
                }
            }
            PixFormat::Bgr48 => {
                match pmap_tmp.bpp() {
                    //16 => color_conv(dst, &rbuf_tmp, color_conv_rgb555_to_bgr48()),
                    24 => color_conv(dst, &rbuf_tmp, Bgr24ToBgr48::convert),
                    _ => {}
                }
            }
            PixFormat::Argb32 => match pmap_tmp.bpp() {
                16 => color_conv(dst, &rbuf_tmp, Rgb555ToArgb32::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToArgb32::convert),
                32 => color_conv(dst, &rbuf_tmp, Bgra32ToAbgr32::convert),
                _ => (),
            },

            PixFormat::Bgra32 => match pmap_tmp.bpp() {
                16 => color_conv(dst, &rbuf_tmp, Rgb555ToBgra32::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToBgra32::convert),
                32 => color_conv(dst, &rbuf_tmp, Bgra32ToBgra32::convert),
                _ => (),
            },
            PixFormat::Rgba32 => match pmap_tmp.bpp() {
                16 => color_conv(dst, &rbuf_tmp, Rgb555ToRgba32::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToRgba32::convert),
                32 => color_conv(dst, &rbuf_tmp, Bgra32ToRgba32::convert),
                _ => (),
            },
            PixFormat::Abgr64 => match pmap_tmp.bpp() {
                //16 => color_conv(dst, &rbuf_tmp, Rgb555ToAbgr64::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToAbgr64::convert),
                //32 => color_conv(dst, &rbuf_tmp, Bgra32ToAbgr64::convert),
                _ => (),
            },
            PixFormat::Argb64 => match pmap_tmp.bpp() {
                //16 => color_conv(dst, &rbuf_tmp, Rgb555ToArgb64::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToArgb64::convert),
                //32 => color_conv(dst, &rbuf_tmp, Bgra32ToArgb64::convert),
                _ => (),
            },
            PixFormat::Bgra64 => match pmap_tmp.bpp() {
                //16 => color_conv(dst, &rbuf_tmp, Rgb555ToBgra64::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToBgra64::convert),
                //32 => color_conv(dst, &rbuf_tmp, Bgra32ToBgra64::convert),
                _ => (),
            },
            PixFormat::Rgba64 => match pmap_tmp.bpp() {
                //16 => color_conv(dst, &rbuf_tmp, Rgb555ToRgba64::convert),
                24 => color_conv(dst, &rbuf_tmp, Bgr24ToRgba64::convert),
                //32 => color_conv(dst, &rbuf_tmp, Bgra32ToRgba64::convert),
                _ => (),
            },
            _ => {}
        }

        true
    }

    pub fn translate(&mut self, keycode: u32) -> u32 {
        self.last_translated_key = if keycode > 255 {
            0
        } else {
            self.keymap[keycode as usize] as u32
        };
        self.last_translated_key
    }
}

impl PlatUtil {
    pub fn message(&self, msg: &str) {
		println!("{}", msg);
        /*let msg = std::ffi::CString::new(msg).unwrap();
        unsafe {
            MessageBoxA(
                self.specific.hwnd,
                msg.as_ptr() as *const u8,
                "AGG Message".as_ptr(),
                MB_OK | MB_SYSTEMMODAL,
            );
        }*/
    }

    pub fn start_timer(&mut self) {
        unsafe {
            QueryPerformanceCounter(&mut self.specific.sw_start);
        }
    }

    pub fn elapsed_time(&self) -> f64 {
        let mut stop: i64 = 0;
        unsafe {
            QueryPerformanceCounter(&mut stop);
        }
        (stop - self.specific.sw_start) as f64 * 1000.0 / self.specific.sw_freq as f64
    }

    pub fn load_img(&mut self, idx: u32, file: &str)  -> bool {
        if idx < MAX_IMAGES {
            let mut na = file.to_string();
            let len = na.len();
            if len < 4 || na[len - 4..].to_lowercase() != ".bmp" {
                na.push_str(".bmp");
            }
            self.specific
                .load_pmap(&na, idx, &mut self.rbuf_img[idx as usize])
        } else {
            true
        }
    }

    pub fn save_img(&mut self, idx: u32, file: &str)  -> bool {
        if idx < MAX_IMAGES {
            let mut na = file.to_string();
            let len = na.len();
            if len < 4 || na[len - 4..].to_lowercase() != ".bmp" {
                na.push_str(".bmp");
            }
            self.specific
                .save_pmap(&na, idx, &self.rbuf_img[idx as usize])
        } else {
            true
        }
    }

    pub fn create_img(&mut self, idx: u32, w: u32, h: u32)  -> bool {
        if idx < MAX_IMAGES {
            let width = if w == 0 {
                self.specific.pmap_window.width()
            } else {
                w
            };
            let height = if h == 0 {
                self.specific.pmap_window.height()
            } else {
                h
            };
            self.specific.pmap_img[idx as usize].create(
                width,
                height,
                unsafe { ::std::mem::transmute(self.specific.bpp) },
                256,
            );
            self.rbuf_img[idx as usize].attach(
                self.specific.pmap_img[idx as usize].buf(),
                self.specific.pmap_img[idx as usize].width(),
                self.specific.pmap_img[idx as usize].height(),
                if self.specific.flip_y {
                    self.specific.pmap_img[idx as usize].stride()
                } else {
                    -self.specific.pmap_img[idx as usize].stride()
                },
            );
            true
        } else {
            false
        }
    }

	pub fn img_ext(&self) -> &str {
        ".bmp"
    }

    pub fn full_file_name(&self, file_name: &str) -> String {
        file_name.to_string()
    }
}

impl<App: Interface> PlatSupport<App> {
    pub fn new(demo: App, format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let ps = PlatSpecific::new(format, flip_y);
        let mbpp = ps.bpp;

        let mut plat = PlatSupport {
            //m_ctrls: ct,
            initial_height: 0,
            initial_width: 0,
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
        self.caption = cap.to_string() + "\u{0}";
        if self.specific.hwnd != 0 {
            unsafe {
                SetWindowTextA(self.specific.hwnd, cap.as_ptr() as *const u8);
            }
        }
    }

    fn get_key_flags(wflags: usize) -> u32 {
        let wflags = wflags as u32;
        let mut flags: u32 = 0;
        if wflags & MK_LBUTTON != 0 {
            flags |= InputFlag::MouseLeft as u32;
        }
        if wflags & MK_RBUTTON != 0 {
            flags |= InputFlag::MouseRight as u32;
        }
        if wflags & MK_SHIFT != 0 {
            flags |= InputFlag::KbdShift as u32;
        }
        if wflags & MK_CONTROL != 0 {
            flags |= InputFlag::KbdCtrl as u32;
        }
        flags
    }

    pub fn raw_display_handler(&self) -> *mut c_void {
        self.specific.current_dc as *mut c_void
    }

    extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let mut ps: PAINTSTRUCT = unsafe { mem::zeroed() };
        let paint_dc;
        let user_data = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };

        if user_data == 0 {
            if msg == WM_DESTROY {
                unsafe {
                    PostQuitMessage(0);
                }
                return 0;
            }
            return unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) };
        }
        let app = unsafe { &mut *(user_data as *mut PlatSupport<App>) };

        let dc = unsafe { GetDC(app.specific.hwnd) };
        app.specific.current_dc = dc;
        let mut ret: LRESULT = 0;

		fn dr<App:Interface>(r: Draw, app: &mut PlatSupport<App>) {
			match r {
				Draw::Yes => app.force_redraw(),
				Draw::Update => app.update_window(),
				Draw::No => (),
			}
		}
        match msg {
            WM_CREATE => {}
            WM_SIZE => {
                app.specific.create_pmap(
                    loword!(lparam) as u32,
                    hiword!(lparam) as u32,
                    &mut app.rbuf_window,
                );
                app.set_trans_affine_resizing(loword!(lparam) as i32, hiword!(lparam) as i32);
                app.util
                    .borrow_mut()
                    .set_trans_affine_resizing(app.trans_affine_resizing());
                app.util.borrow_mut().plat_specific(
                    &app.specific,
                    loword!(lparam) as u32,
                    hiword!(lparam) as u32,
                );
                app.demo
                    .on_resize(loword!(lparam) as i32, hiword!(lparam) as i32);
                app.force_redraw();
            }
            WM_ERASEBKGND => {}
            WM_LBUTTONDOWN => {
                unsafe { SetCapture(app.specific.hwnd) };
                app.specific.cur_x = (loword!(lparam) as i32) as i32;
                if app.flip_y() {
                    app.specific.cur_y =
                        (app.rbuf_window().height() - (hiword!(lparam) as i32) as u32) as i32;
                } else {
                    app.specific.cur_y = (hiword!(lparam) as i32) as i32;
                }
                app.specific.input_flags =
                    InputFlag::MouseLeft as u32 | Self::get_key_flags(wparam);

                app.demo
                    .on_ctrls()
                    .set_cur(app.specific.cur_x as f64, app.specific.cur_y as f64);
                if app
                    .demo
                    .on_ctrls()
                    .on_mouse_button_down(app.specific.cur_x as f64, app.specific.cur_y as f64)
                {
                    app.demo.on_ctrl_change(&mut app.rbuf_window);
                    app.force_redraw();
                } else {
                    if app
                        .demo
                        .on_ctrls()
                        .in_rect(app.specific.cur_x as f64, app.specific.cur_y as f64)
                    {
                        if app
                            .demo
                            .on_ctrls()
                            .set_cur(app.specific.cur_x as f64, app.specific.cur_y as f64)
                        {
                            app.demo.on_ctrl_change(&mut app.rbuf_window);
                            app.force_redraw();
                        }
                    } else {
                        let r = app.demo.on_mouse_button_down(
                            &mut app.rbuf_window,
                            app.specific.cur_x,
                            app.specific.cur_y,
                            app.specific.input_flags,
                        );
						dr(r, app);
                    }
                }
            }
            WM_LBUTTONUP => {
                unsafe { ReleaseCapture() };
                app.specific.cur_x = loword!(lparam) as i32;
                if app.flip_y() {
                    app.specific.cur_y = app.rbuf_window().height() as i32 - hiword!(lparam) as i32;
                } else {
                    app.specific.cur_y = hiword!(lparam) as i32;
                }
                app.specific.input_flags =
                    InputFlag::MouseLeft as u32 | Self::get_key_flags(wparam);

                if app
                    .demo
                    .on_ctrls()
                    .on_mouse_button_up(app.specific.cur_x as f64, app.specific.cur_y as f64)
                {
                    app.demo.on_ctrl_change(&mut app.rbuf_window);
                    app.force_redraw();
                }
                let r = app.demo.on_mouse_button_up(
                    &mut app.rbuf_window,
                    app.specific.cur_x,
                    app.specific.cur_y,
                    app.specific.input_flags,
                );
				dr(r, app);
            }
            WM_RBUTTONDOWN => {
                unsafe { SetCapture(app.specific.hwnd) };
                app.specific.cur_x = loword!(lparam) as i32;
                if app.flip_y() {
                    app.specific.cur_y = app.rbuf_window().height() as i32 - hiword!(lparam) as i32;
                } else {
                    app.specific.cur_y = hiword!(lparam) as i32;
                }
                app.specific.input_flags =
                    InputFlag::MouseRight as u32 | Self::get_key_flags(wparam);
                let r = app.demo.on_mouse_button_down(
                    &mut app.rbuf_window,
                    app.specific.cur_x,
                    app.specific.cur_y,
                    app.specific.input_flags,
                );
				dr(r, app);
            }
            WM_RBUTTONUP => {
                unsafe { ReleaseCapture() };
                app.specific.cur_x = loword!(lparam) as i32;
                if app.flip_y() {
                    app.specific.cur_y = app.rbuf_window().height() as i32 - hiword!(lparam) as i32;
                } else {
                    app.specific.cur_y = hiword!(lparam) as i32;
                }
                app.specific.input_flags =
                    InputFlag::MouseRight as u32 | Self::get_key_flags(wparam);
                let r = app.demo.on_mouse_button_up(
                    &mut app.rbuf_window,
                    app.specific.cur_x,
                    app.specific.cur_y,
                    app.specific.input_flags,
                );
				dr(r, app);
            }
            WM_MOUSEMOVE => {
                app.specific.cur_x = loword!(lparam) as i32;
                if app.flip_y() {
                    app.specific.cur_y = app.rbuf_window().height() as i32 - hiword!(lparam) as i32;
                } else {
                    app.specific.cur_y = hiword!(lparam) as i32;
                }
                app.specific.input_flags = Self::get_key_flags(wparam);

                if app.demo.on_ctrls().on_mouse_move(
                    app.specific.cur_x as f64,
                    app.specific.cur_y as f64,
                    app.specific.input_flags & InputFlag::MouseLeft as u32 != 0,
                ) {
                    app.demo.on_ctrl_change(&mut app.rbuf_window);
                    app.force_redraw();
                } else {
                    if !app
                        .demo
                        .on_ctrls()
                        .in_rect(app.specific.cur_x as f64, app.specific.cur_y as f64)
                    {
                        let r =app.demo.on_mouse_move(
                            &mut app.rbuf_window,
                            app.specific.cur_x,
                            app.specific.cur_y,
                            app.specific.input_flags,
                        );
						dr(r, app);
                    }
                }
            }
            WM_SYSKEYDOWN | WM_KEYDOWN => {
                app.specific.last_translated_key = 0;
                match wparam {
                    x if x == VK_CONTROL as usize => {
                        app.specific.input_flags |= InputFlag::KbdCtrl as u32;
                    }
                    x if x == VK_SHIFT as usize => {
                        app.specific.input_flags |= InputFlag::KbdShift as u32;
                    }
                    _ => {
                        app.specific.translate(wparam as u32);
                    }
                }

                if app.specific.last_translated_key != 0 {
                    let mut left = false;
                    let mut up = false;
                    let mut right = false;
                    let mut down = false;

                    match app.specific.last_translated_key {
                        key_left if key_left == KeyCode::Left as u32 => left = true,
                        key_up if key_up == KeyCode::Up as u32 => up = true,
                        key_right if key_right == KeyCode::Right as u32 => right = true,
                        key_down if key_down == KeyCode::Down as u32 => down = true,
                        key_f2 if key_f2 == KeyCode::F2 as u32 => {
                            app.util
                                .borrow_mut()
                                .copy_window_to_img(&mut app.rbuf_window, MAX_IMAGES - 1);
                            app.util.borrow_mut().save_img(MAX_IMAGES - 1, "screenshot");
                        }
                        _ => {}
                    }

                    if app.window_flags() & WindowFlag::ProcessAllKeys as u32 != 0 {
                        let r = app.demo.on_key(
                            &mut app.rbuf_window,
                            app.specific.cur_x,
                            app.specific.cur_y,
                            app.specific.last_translated_key,
                            app.specific.input_flags,
                        );
						dr(r, app);
                    } else {
                        if app.demo.on_ctrls().on_arrow_keys(left, right, down, up) {
                            app.demo.on_ctrl_change(&mut app.rbuf_window);
                            app.force_redraw();
                        } else {
                            let r = app.demo.on_key(
                                &mut app.rbuf_window,
                                app.specific.cur_x,
                                app.specific.cur_y,
                                app.specific.last_translated_key,
                                app.specific.input_flags,
                            );
							dr(r, app);
                        }
                    }
                }
            }
            WM_SYSKEYUP | WM_KEYUP => {
                app.specific.last_translated_key = 0;
                match wparam {
                    x if x == VK_CONTROL as usize => {
                        app.specific.input_flags &= !(InputFlag::KbdCtrl as u32);
                    }
                    x if x == VK_SHIFT as usize => {
                        app.specific.input_flags &= !(InputFlag::KbdShift as u32);
                    }
                    _ => {}
                }
            }
            WM_CHAR | WM_SYSCHAR => {
                if app.specific.last_translated_key == 0 {
                    let r = app.demo.on_key(
                        &mut app.rbuf_window,
                        app.specific.cur_x,
                        app.specific.cur_y,
                        wparam as u32,
                        app.specific.input_flags,
                    );
					dr(r, app);
                }
            }
            WM_PAINT => {
                paint_dc = unsafe { BeginPaint(hwnd, &mut ps) };
                app.specific.current_dc = paint_dc;
                if app.specific.redraw_flag {
                    app.demo.on_draw(&mut app.rbuf_window);
                    app.specific.redraw_flag = false;
                }
                let rbuf = app.rbuf_window().clone();
                app.specific.display_pmap(paint_dc, &rbuf);
                //app.demo.on_post_draw(paint_dc);
                app.specific.current_dc = 0;
                unsafe { EndPaint(hwnd, &ps) };
            }
            WM_COMMAND => {}
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
            }
            _ => {
                ret = unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) };
            }
        }
        app.specific.current_dc = 0;
        unsafe { ReleaseDC(app.specific.hwnd, dc) };
        ret
    }

    pub fn init(&mut self, w: u32, h: u32, flags: u32)  -> bool {
        let instance = unsafe { GetModuleHandleA(std::ptr::null()) };
        debug_assert!(instance != 0);

        let (width, height) = (w as i32, h as i32);
        if self.specific.sys_format == PixFormat::Undefined {
            return false;
        }
        let window_class = s!("AGGAppClass");
        self.window_flags = flags;

        let wflags = CS_OWNDC | CS_VREDRAW | CS_HREDRAW;

        let wc = WNDCLASSA {
            lpszClassName: window_class,
            lpfnWndProc: Some(Self::wndproc),
            style: wflags,
            hInstance: instance,
            hIcon: 0, //unsafe { LoadIconA(0, IDI_APPLICATION) },
            hCursor: unsafe { LoadCursorW(0, IDC_ARROW) },
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: null(),
            cbClsExtra: 0,
            cbWndExtra: 0,
        };

        unsafe {
            RegisterClassA(&wc);
        }

        let mut wflags = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;

        if self.window_flags & WindowFlag::Resize as u32 != 0 {
            wflags |= WS_THICKFRAME | WS_MAXIMIZEBOX;
        }

        //let sss = b"This is a sample window".as_ptr() as _;
        self.specific.hwnd = unsafe {
            CreateWindowExA(
                0,
                window_class,
                self.caption(),
                wflags,
                100,
                100,
                width,
                height,
                0 as HWND,
                0 as HMENU,
                instance,
                std::ptr::null(),
            )
        };

        if self.specific.hwnd == 0 as HWND {
            let err = unsafe { GetLastError() };
            println!("Err {}", err);
            return false;
        }

        let mut rct = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        unsafe {
            GetClientRect(self.specific.hwnd, &mut rct);
        }

        unsafe {
            MoveWindow(
                self.specific.hwnd,
                100,
                100,
                width + (width - (rct.right - rct.left)),
                height + (height - (rct.bottom - rct.top)),
                FALSE,
            );
        }

        unsafe {
            SetWindowLongPtrA(
                self.specific.hwnd,
                GWL_USERDATA,
                self as *mut PlatSupport<App> as isize,
            );
        }

        self.specific.create_pmap(w, h, &mut self.rbuf_window);

        self.initial_width = w;
        self.initial_height = h;
        self.util
            .borrow_mut()
            .plat_specific(&self.specific, width as u32, height as u32);
        self.util
            .borrow_mut()
            .set_initial(self.initial_width, self.initial_height);
        self.demo.on_init();
        self.specific.redraw_flag = true;
        unsafe {
            ShowWindow(self.specific.hwnd, SW_SHOW);
        }
        true
    }

    pub fn run(&mut self) -> i32 {
        let mut msg: MSG = unsafe { mem::zeroed() };
		fn dr<App:Interface>(r: Draw, app: &mut PlatSupport<App>) {
			match r {
				Draw::Yes => app.force_redraw(),
				Draw::Update => app.update_window(),
				Draw::No => (),
			}
		}
        loop {
            if self.util.borrow().wait_mode() {
                if unsafe { GetMessageW(&mut msg, 0, 0, 0) } == 0 {
                    break;
                }
                unsafe { TranslateMessage(&mut msg) };
                unsafe { DispatchMessageW(&mut msg) };
            } else {
                if unsafe { PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) } != 0 {
                    unsafe { TranslateMessage(&mut msg) };
                    if msg.message == WM_QUIT {
                        break;
                    }
                    unsafe { DispatchMessageW(&mut msg) };
                } else {
                    let r = self.demo.on_idle();
					dr(r, self);
                }
            }
        }
        msg.wParam as i32
    }

    fn force_redraw(&mut self) {
        self.specific.redraw_flag = true;
        unsafe {
            InvalidateRect(self.specific.hwnd, std::ptr::null_mut(), 0);
        }
    }

    fn update_window(&mut self) {
        let dc = unsafe { GetDC(self.specific.hwnd) };
        self.specific.display_pmap(dc, &self.rbuf_window);
        unsafe { ReleaseDC(self.specific.hwnd, dc) };
    }
}
