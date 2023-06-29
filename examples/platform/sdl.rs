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
// Contact: mcseem@antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------
//
// class PlatSupport. SDL version.
//
//----------------------------------------------------------------------------

use crate::platform::*;
use core::ptr::*;

use libc::*;
use sdl2_sys::*;
use std::{cell::RefCell, rc::Rc, slice, str};
//use arr_macro::arr;
//use core::mem::MaybeUninit;
use agg::RenderBuf;
use agg::RenderBuffer;
use agg::TransAffine;

use super::KeyCode;
//------------------------------------------------------------------------
#[derive(Clone, Copy)]
pub struct PlatSpecific {
    format: PixFormat,
    sys_format: PixFormat,
    flip_y: bool,
    bpp: u32,
    sys_bpp: u32,
    rmask: u32,
    gmask: u32,
    bmask: u32,
    amask: u32,
    update_flag: bool,
    resize_flag: bool,
    initialized: bool,
    surf_screen: *mut SDL_Surface,
    surf_window: *mut SDL_Surface,
    surf_img: [*mut SDL_Surface; MAX_IMAGES as usize],
    main_window: *mut SDL_Window,
    wnd_renderer: *mut SDL_Renderer,
    wnd_texture: *mut SDL_Texture,
    cur_x: i32,
    cur_y: i32,
    sw_start: i32,
}

impl PlatSpecific {
    pub fn new(format: PixFormat, flip_y: bool) -> PlatSpecific {
        let mut plat_spec = PlatSpecific {
            format: format,
            sys_format: PixFormat::Undefined,
            flip_y: flip_y,
            bpp: 0,
            sys_bpp: 0,
            update_flag: true,
            resize_flag: true,
            initialized: false,
            surf_screen: null_mut(),
            surf_window: null_mut(),
            main_window: null_mut(),
            wnd_renderer: null_mut(),
            wnd_texture: null_mut(),
            cur_x: 0,
            cur_y: 0,
            surf_img: [null_mut(); MAX_IMAGES as usize],
            rmask: 0,
            gmask: 0,
            bmask: 0,
            amask: 0,
            sw_start: 0,
        };

        match format {
            PixFormat::Gray8 => {
                plat_spec.bpp = 8;
            }
            PixFormat::Rgb565 => {
                plat_spec.rmask = 0xF800;
                plat_spec.gmask = 0x7E0;
                plat_spec.bmask = 0x1F;
                plat_spec.amask = 0;
                plat_spec.bpp = 16;
            }
            PixFormat::Rgb555 => {
                plat_spec.rmask = 0x7C00;
                plat_spec.gmask = 0x3E0;
                plat_spec.bmask = 0x1F;
                plat_spec.amask = 0;
                plat_spec.bpp = 16;
            }
            PixFormat::Rgb24 => {
                plat_spec.rmask = 0xFF;
                plat_spec.gmask = 0xFF00;
                plat_spec.bmask = 0xFF0000;
                plat_spec.amask = 0;
                plat_spec.bpp = 24;
            }
            PixFormat::Bgr24 => {
                plat_spec.rmask = 0xFF0000;
                plat_spec.gmask = 0xFF00;
                plat_spec.bmask = 0xFF;
                plat_spec.amask = 0;
                plat_spec.bpp = 24;
            }
            PixFormat::Bgra32 => {
                plat_spec.rmask = 0xFF0000;
                plat_spec.gmask = 0xFF00;
                plat_spec.bmask = 0xFF;
                plat_spec.amask = 0xFF000000;
                plat_spec.bpp = 32;
            }
            PixFormat::Abgr32 => {
                plat_spec.rmask = 0xFF000000;
                plat_spec.gmask = 0xFF0000;
                plat_spec.bmask = 0xFF00;
                plat_spec.amask = 0xFF;
                plat_spec.bpp = 32;
            }
            PixFormat::Argb32 => {
                plat_spec.rmask = 0xFF00;
                plat_spec.gmask = 0xFF0000;
                plat_spec.bmask = 0xFF000000;
                plat_spec.amask = 0xFF;
                plat_spec.bpp = 32;
            }
            PixFormat::Rgba32 => {
                plat_spec.rmask = 0xFF;
                plat_spec.gmask = 0xFF00;
                plat_spec.bmask = 0xFF0000;
                plat_spec.amask = 0xFF000000;
                plat_spec.bpp = 32;
            }
            _ => (),
        }
        plat_spec
    }
}

//------------------------------------------------------------------------
impl PlatUtil {
    pub fn drop(&mut self) {
        for i in 0..MAX_IMAGES as usize {
            if !self.specific.surf_img[i].is_null() {
                unsafe {
                    SDL_FreeSurface(self.specific.surf_img[i]);
                }
            }
        }
    }

    pub fn load_img(&mut self, idx: u32, file: &str)  -> bool {
        if idx < MAX_IMAGES {
            if self.specific.surf_img[idx as usize] != std::ptr::null_mut() {
                unsafe { SDL_FreeSurface(self.specific.surf_img[idx as usize]) };
            }

            let mut f = String::from(file);
            let len = f.len();
            if len < 4 || &f[len - 4..] != ".bmp" {
                f.push_str(".bmp");
            }

            let tmp_surf = unsafe {
                let s = std::ffi::CString::new(f.clone()).unwrap();
                let ops = SDL_RWFromFile(s.as_ptr(), "rb\u{0}".as_ptr() as *const i8);
                SDL_LoadBMP_RW(ops, 1)
            };
            if tmp_surf == std::ptr::null_mut() {
                println!(
                    "Couldn't load {}: {}",
                    f,
                    unsafe { std::ffi::CStr::from_ptr(SDL_GetError()) }
                        .to_str()
                        .unwrap()
                );
                return false;
            }

            let mut format = SDL_PixelFormat {
                palette: std::ptr::null_mut(),
                BitsPerPixel: self.specific.bpp as u8,
                BytesPerPixel: (self.specific.bpp >> 8) as u8,
                Rmask: self.specific.rmask,
                Gmask: self.specific.gmask,
                Bmask: self.specific.bmask,
                Amask: self.specific.amask,
                Rshift: 0,
                Gshift: 0,
                Bshift: 0,
                Ashift: 0,
                Rloss: 0,
                Gloss: 0,
                Bloss: 0,
                Aloss: 0,
                refcount: 0,
                padding: [0; 2],
                format: 0,
                next: std::ptr::null_mut(),
            };

            self.specific.surf_img[idx as usize] =
                unsafe { SDL_ConvertSurface(tmp_surf, &mut format, SDL_SWSURFACE) };

            unsafe { SDL_FreeSurface(tmp_surf) };

            if self.specific.surf_img[idx as usize] == std::ptr::null_mut() {
                return false;
            }

            let surf = self.specific.surf_img[idx as usize];
            unsafe {
                self.rbuf_img[idx as usize].attach(
                    (*surf).pixels as *mut u8,
                    (*surf).w as u32,
                    (*surf).h as u32,
                    if self.specific.flip_y {
                        -(*surf).pitch
                    } else {
                        (*surf).pitch
                    },
                );
            }
            return true;
        }
        return false;
    }

    //////////////
    pub fn save_img(&self, idx: u32, file: &str)  -> bool {
        if idx < MAX_IMAGES && self.specific.surf_img[idx as usize] != std::ptr::null_mut() {
            let mut f = file.to_string();
            let len = f.len();
            if len < 4 || &f[len - 4..] != ".bmp" {
                f.push_str(".bmp");
            }
            unsafe {
                let s = std::ffi::CString::new(f.clone()).unwrap();
                SDL_SaveBMP_RW(
                    self.specific.surf_img[idx as usize],
                    SDL_RWFromFile(s.as_ptr(), "wb\u{0}".as_ptr() as *const i8),
                    1,
                ) == 0
            }
        } else {
            false
        }
    }

    /////////////
    pub fn create_img(&mut self, idx: u32, width: u32, height: u32)  -> bool {
        unsafe {
            if idx < MAX_IMAGES {
                if self.specific.surf_img[idx as usize] != std::ptr::null_mut() {
                    SDL_FreeSurface(self.specific.surf_img[idx as usize]);
                }

                self.specific.surf_img[idx as usize] = SDL_CreateRGBSurface(
                    SDL_SWSURFACE,
                    width as i32,
                    height as i32,
                    (*(*self.specific.surf_screen).format).BitsPerPixel as i32,
                    self.specific.rmask,
                    self.specific.gmask,
                    self.specific.bmask,
                    self.specific.amask,
                );
                if self.specific.surf_img[idx as usize] == std::ptr::null_mut() {
                    eprintln!(
                        "Couldn't create image: {}\n",
                        std::ffi::CStr::from_ptr(SDL_GetError()).to_str().unwrap()
                    );

                    return false;
                }

                self.rbuf_img[idx as usize].attach(
                    (*self.specific.surf_img[idx as usize]).pixels as *mut u8,
                    (*self.specific.surf_img[idx as usize]).w as u32,
                    (*self.specific.surf_img[idx as usize]).h as u32,
                    if self.specific.flip_y {
                        -((*self.specific.surf_img[idx as usize]).pitch as i32)
                    } else {
                        (*self.specific.surf_img[idx as usize]).pitch as i32
                    },
                );

                return true;
            }

            return false;
        }
    }

    //------------------------------------------------------------------------
    pub fn start_timer(&mut self) {
        self.specific.sw_start = unsafe { SDL_GetTicks() as i32 };
    }

    //------------------------------------------------------------------------
    pub fn elapsed_time(&self) -> f64 {
        let stop = unsafe { SDL_GetTicks() as i32 };
        return (stop - self.specific.sw_start) as f64;
    }

    //------------------------------------------------------------------------
    pub fn message(&self, msg: &str) {
        println!("{}", msg);
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
        if unsafe { SDL_Init(SDL_INIT_VIDEO) } < 0 {
            panic!("failed to initialize sdl2 with video");
        };
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
        plat.caption = "Anti-Grain Geometry Application".to_string();
        plat.demo.on_ctrls().set_transform(&plat.resize_mtx);
        plat
    }

    pub fn drop(&mut self) {
        for i in 0..MAX_IMAGES as usize {
            if !self.specific.surf_img[i].is_null() {
                unsafe {
                    SDL_FreeSurface(self.specific.surf_img[i]);
                }
            }
        }
    }

    pub fn init(&mut self, width: u32, height: u32, flags: u32)  -> bool {
        self.window_flags = flags;
        let mut wflags = SDL_SWSURFACE;

        if self.window_flags & WindowFlag::HwBuffer as u32 != 0 {
            wflags = 0; //SDL_HWSURFACE;
        }

        if self.window_flags & WindowFlag::Resize as u32 != 0 {
            wflags |= SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32;
        }

        if !self.specific.surf_screen.is_null() {
            unsafe {
                SDL_FreeSurface(self.specific.surf_screen);
            }
        }

        let caption = self.caption.clone() + "\u{0}";
        unsafe {
            if self.specific.main_window.is_null() {
                self.specific.main_window = SDL_CreateWindow(
                    caption.as_ptr() as *const std::os::raw::c_char,
                    SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                    SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                    width as i32,
                    height as i32,
                    wflags,
                );
                if self.specific.main_window.is_null() {
                    let c_s = SDL_GetError();
                    let s = str::from_utf8_unchecked(slice::from_raw_parts(
                        c_s as *const u8,
                        strlen(c_s) + 1,
                    ));
                    eprint!(
                        "Unable to set {}x{} {} bpp video: {}\n",
                        width, height, self.bpp, s
                    );
                    return false;
                }
            }
            let _pixfmt = SDL_GetWindowPixelFormat(self.specific.main_window);
            self.specific.surf_screen = SDL_GetWindowSurface(self.specific.main_window);
            //SDL_PixelFormat *fmt=surface->format;
            //bpp = fmt->BitsPerPixel
            self.specific.wnd_renderer = SDL_CreateRenderer(self.specific.main_window, -1, 0);

            //bpp = SDL_BITSPERPIXEL(pixfrm) //(format >> 8) & 0xFF
            //self.specific.surf_screen = SDL_SetVideoMode(width, height, self.bpp, wflags);
            if self.specific.surf_screen.is_null() {
                let c_s = SDL_GetError();
                let s = str::from_utf8_unchecked(slice::from_raw_parts(
                    c_s as *const u8,
                    strlen(c_s) + 1,
                ));
                eprint!(
                    "Unable to set {}x{} {} bpp video: {}\n",
                    width, height, self.bpp, s
                );
                return false;
            }

            //SDL_WM_SetCaption(self.caption, 0);

            if !self.specific.surf_window.is_null() {
                SDL_FreeSurface(self.specific.surf_window);
            }

            self.specific.surf_window = SDL_CreateRGBSurface(
                0,
                (*self.specific.surf_screen).w,
                (*self.specific.surf_screen).h,
                self.specific.bpp as i32,
                self.specific.rmask,
                self.specific.gmask,
                self.specific.bmask,
                self.specific.amask,
            );

            if self.specific.surf_window.is_null() {
                let c_s = SDL_GetError();
                let s = str::from_utf8_unchecked(slice::from_raw_parts(
                    c_s as *const u8,
                    strlen(c_s) + 1,
                ));
                eprint!(
                    "Unable to create image buffer {}x{} {} bpp video: {}\n",
                    width, height, self.bpp, s
                );
                return false;
            }

            self.specific.wnd_texture = SDL_CreateTexture(
                self.specific.wnd_renderer,
                SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32,
                SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32,
                width as i32,
                height as i32,
            );

            self.rbuf_window.attach(
                (*self.specific.surf_window).pixels as *mut u8,
                (*self.specific.surf_window).w as u32,
                (*self.specific.surf_window).h as u32,
                if self.flip_y {
                    -(*self.specific.surf_window).pitch
                } else {
                    (*self.specific.surf_window).pitch
                },
            );
        }

        self.util
            .borrow_mut()
            .plat_specific(&self.specific, width, height);

        if !self.specific.initialized {
            self.initial_width = width;
            self.initial_height = height;
            self.util
                .borrow_mut()
                .set_initial(self.initial_width, self.initial_height);
            self.demo.on_init();
            self.specific.initialized = true;
        }
        self.demo.on_resize(
            self.rbuf_window.width() as i32,
            self.rbuf_window.height() as i32,
        );
        self.specific.update_flag = true;

        return true;
    }

    fn update_window(&mut self) {
        unsafe {
            /*SDL_UpdateTexture(
                self.specific.wnd_texture,
                null(),
                (*self.specific.surf_window).pixels,
                (*self.specific.surf_window).pitch,
            );
            SDL_RenderClear(self.specific.wnd_renderer);
            SDL_RenderCopy(
                self.specific.wnd_renderer,
                self.specific.wnd_texture,
                null(),
                null(),
            );
            SDL_RenderPresent(self.specific.wnd_renderer);*/
            /*SDL_FillRect(
                self.specific.surf_screen,
                null_mut(),
                SDL_MapRGB((*self.specific.surf_screen).format, 0xFF, 0xFF, 0x00),
            );
            SDL_UpdateWindowSurface(self.specific.main_window);*/

            SDL_UpperBlit(
                self.specific.surf_window,
                null_mut() as *mut _,
                self.specific.surf_screen,
                null_mut() as *mut _,
            );
            SDL_UpdateWindowSurface(self.specific.main_window);
        }
    }

    pub fn run(&mut self) -> i32 {
        let mut event: SDL_Event = unsafe { std::mem::zeroed() }; //SDL_Event::default();
        let mut ev_flag;
        fn dr<App: Interface>(r: Draw, app: &mut PlatSupport<App>) {
            match r {
                Draw::Yes => app.force_redraw(),
                Draw::Update => app.update_window(),
                Draw::No => (),
            }
        }
        loop {
            if self.specific.update_flag {
                self.demo.on_draw(&mut self.rbuf_window);
                self.update_window();
                self.specific.update_flag = false;
            }

            ev_flag = false;
            if self.util.borrow().wait_mode() {
                unsafe { SDL_WaitEvent(&mut event) };
                ev_flag = true;
            } else {
                if unsafe { SDL_PollEvent(&mut event) } == 1 {
                    ev_flag = true;
                } else {
                    dr(self.demo.on_idle(), self);
                }
            }
            unsafe {
                if ev_flag {
                    if event.type_ == SDL_EventType::SDL_QUIT as u32 {
                        break;
                    }

                    let y: i32;
                    let mut flags: u32;

                    if event.type_ == SDL_EventType::SDL_WINDOWEVENT as u32 {
                        if event.window.event == SDL_WindowEventID::SDL_WINDOWEVENT_RESIZED as u8 {
                            if !self.init(
                                event.window.data1 as u32,
                                event.window.data2 as u32,
                                self.window_flags,
                            ) {
                                return 0;
                            }
                            self.demo.on_resize(
                                self.rbuf_window.width() as i32,
                                self.rbuf_window.height() as i32,
                            );
                            self.set_trans_affine_resizing(event.window.data1, event.window.data2);
                            self.util
                                .borrow_mut()
                                .set_trans_affine_resizing(self.trans_affine_resizing());
                            //self.demo.on_ctrls().set_transform(self.resize_mtx);
                            self.specific.update_flag = true;
                        }
                    } else if event.type_ == SDL_EventType::SDL_KEYDOWN as u32 {
                        flags = 0;
                        if event.key.keysym.mod_ & SDL_Keymod::KMOD_LSHIFT as u16 != 0 {
                            flags |= InputFlag::KbdShift as u32;
                        }
                        if event.key.keysym.mod_ & SDL_Keymod::KMOD_LCTRL as u16 != 0 {
                            flags |= InputFlag::KbdCtrl as u32;
                        }

                        let mut left: bool = false;
                        let mut up: bool = false;
                        let mut right: bool = false;
                        let mut down: bool = false;

                        match event.key.keysym.sym {
                            x if x == KeyCode::Left as i32 => {
                                left = true;
                            }
                            x if x == KeyCode::Up as i32 => {
                                up = true;
                            }
                            x if x == KeyCode::Right as i32 => {
                                right = true;
                            }
                            x if x == KeyCode::Down as i32 => {
                                down = true;
                            }
                            _ => {}
                        }

                        if self.demo.on_ctrls().on_arrow_keys(left, right, down, up) {
                            self.demo.on_ctrl_change(&mut self.rbuf_window);
                            self.force_redraw();
                        } else {
                            let r = self.demo.on_key(
                                &mut self.rbuf_window,
                                self.specific.cur_x,
                                self.specific.cur_y,
                                event.key.keysym.sym as u32,
                                flags,
                            );
                            dr(r, self);
                        }
                    } else if event.type_ == SDL_EventType::SDL_MOUSEMOTION as u32 {
                        y = if self.flip_y {
                            self.rbuf_window.height() as i32 - event.motion.y
                        } else {
                            event.motion.y
                        };

                        self.specific.cur_x = event.motion.x;
                        self.specific.cur_y = y;
                        flags = 0;
                        if event.motion.state & 1/*SDL_BUTTON_LMASK*/ != 0 {
                            flags |= InputFlag::MouseLeft as u32;
                        }
                        if event.motion.state & 4/*SDL_BUTTON_RMASK*/ != 0 {
                            flags |= InputFlag::MouseRight as u32;
                        }

                        if self.demo.on_ctrls().on_mouse_move(
                            self.specific.cur_x as f64,
                            self.specific.cur_y as f64,
                            (flags & InputFlag::MouseLeft as u32) != 0,
                        ) {
                            self.demo.on_ctrl_change(&mut self.rbuf_window);
                            self.force_redraw();
                        } else {
                            let r = self.demo.on_mouse_move(
                                &mut self.rbuf_window,
                                self.specific.cur_x,
                                self.specific.cur_y,
                                flags,
                            );
                            dr(r, self);
                        }
                        let mut eventtrash: SDL_Event = std::mem::zeroed();
                        while SDL_PeepEvents(
                            &mut eventtrash,
                            1,
                            SDL_eventaction::SDL_GETEVENT,
                            SDL_EventType::SDL_MOUSEMOTION as u32,
                            SDL_EventType::SDL_MOUSEMOTION as u32,
                        ) != 0
                        {}
                    } else if event.type_ == SDL_EventType::SDL_MOUSEBUTTONDOWN as u32 {
                        y = if self.flip_y {
                            self.rbuf_window.height() as i32 - event.button.y
                        } else {
                            event.button.y
                        };

                        self.specific.cur_x = event.button.x;
                        self.specific.cur_y = y;
                        //flags = 0;
                        match event.button.button as u32 {
                            SDL_BUTTON_LEFT => {
                                flags = InputFlag::MouseLeft as u32;

                                if self.demo.on_ctrls().on_mouse_button_down(
                                    self.specific.cur_x as f64,
                                    self.specific.cur_y as f64,
                                ) {
                                    self.demo.on_ctrls().set_cur(
                                        self.specific.cur_x as f64,
                                        self.specific.cur_y as f64,
                                    );
                                    self.demo.on_ctrl_change(&mut self.rbuf_window);
                                    self.force_redraw();
                                } else {
                                    if self.demo.on_ctrls().in_rect(
                                        self.specific.cur_x as f64,
                                        self.specific.cur_y as f64,
                                    ) {
                                        if self.demo.on_ctrls().set_cur(
                                            self.specific.cur_x as f64,
                                            self.specific.cur_y as f64,
                                        ) {
                                            self.demo.on_ctrl_change(&mut self.rbuf_window);
                                            self.force_redraw();
                                        }
                                    } else {
                                        let r = self.demo.on_mouse_button_down(
                                            &mut self.rbuf_window,
                                            self.specific.cur_x,
                                            self.specific.cur_y,
                                            flags,
                                        );
                                        dr(r, self);
                                    }
                                }
                            }
                            SDL_BUTTON_RIGHT => {
                                flags = InputFlag::MouseRight as u32;
                                let r = self.demo.on_mouse_button_down(
                                    &mut self.rbuf_window,
                                    self.specific.cur_x,
                                    self.specific.cur_y,
                                    flags,
                                );
                                dr(r, self);
                            }
                            _ => {}
                        } //match(event.button.button)
                    } else if event.type_ == SDL_EventType::SDL_MOUSEBUTTONUP as u32 {
                        y = if self.flip_y {
                            self.rbuf_window.height() as i32 - event.button.y
                        } else {
                            event.button.y
                        };
                        if self.demo.on_ctrls().on_mouse_button_up(
                            self.specific.cur_x as f64,
                            self.specific.cur_y as f64,
                        ) {
                            self.demo.on_ctrl_change(&mut self.rbuf_window);
                            self.force_redraw();
                        }

                        self.specific.cur_x = event.button.x;
                        self.specific.cur_y = y;
                        //flags = 0;
                    }
                }
            }
        }
        0
    }

    pub fn set_caption(&mut self, cap: &str) {
        self.caption = cap.to_string() + "\u{0}";
        if self.specific.initialized {
            unsafe {
                //let s = std::ffi::CString::new(cap).unwrap();
                SDL_SetWindowTitle(self.specific.main_window, self.caption() as *const i8);
            }
        }
    }

    pub fn force_redraw(&mut self) {
        self.specific.update_flag = true;
    }
}
