#![allow(non_camel_case_types)]
#![allow(dead_code)]

use agg::trans_viewport::AspectRatio;
use agg::RenderBuf;

#[cfg(feature = "sdl")]
pub mod sdl;
#[cfg(feature = "sdl")]
pub use sdl::*;

#[cfg(all(target_os = "windows", feature = "win32"))]
mod win32;
#[cfg(all(target_os = "windows", feature = "win32"))]
pub use win32::*;

#[cfg(all(target_family = "unix", feature = "x11"))]
mod x11;
#[cfg(all(target_family = "unix", feature = "x11"))]
pub use self::x11::*;

#[cfg(not(any(
    all(target_os = "windows", feature = "win32"),
    feature = "sdl",
    all(target_family = "unix", feature = "x11")
)))]
compile_error!("unknown configuration!");

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
// class PlatSupport
//
// It's not a part of the AGG library, it's just a helper class to create
// interactive demo examples. Since the examples should not be too complex
// this class is provided to support some very basic interactive graphical
// funtionality, such as putting the rendered image to the window, simple
// keyboard and mouse input, window resizing, setting the window title,
// and catching the "idle" events.
//
// The idea is to have a single header file that does not depend on any
// platform (I hate these endless #ifdef/#elif/#elif.../#endif) and a number
// of different implementations depending on the concrete platform.
// The most popular platforms are:
//
// Windows-32 API
// X-Window API
// SDL library (see http://www.libsdl.org/)
// MacOS C/C++ API
//
// This file does not include any system dependent .h files such as
// windows.h or X11.h, so, your demo applications do not depend on the
// platform. The only file that can #include system dependend headers
// is the implementation file agg_platform_support.cpp. Different
// implementations are placed in different directories, such as
// ~/agg/src/platform/win32
// ~/agg/src/platform/sdl
// ~/agg/src/platform/X11
// and so on.
//
// All the system dependent stuff sits in the platform_specific
// class which is forward-declared here but not defined.
// The PlatSupport class has just a pointer to it and it's
// the responsibility of the implementation to create/delete it.
// This class being defined in the implementation file can have
// any platform dependent stuff such as HWND, X11 Window and so on.
//
//----------------------------------------------------------------------------

//use self::WindowFlag::*;
use crate::ctrl::Ctrl;
//use crate::ctrl::*;
use ::std::{cell::RefCell, rc::Rc};
//use agg::basics::*;
//use agg::RenderBuf;
use agg::RenderBuffer;
use agg::TransAffine;
use agg::TransViewport;

pub enum Draw {
    No,
    Yes,
    Update,
}

impl From<bool> for Draw {
    fn from(v: bool) -> Self {
        match v {
            true => Draw::Yes,
            false => Draw::No,
        }
    }
}

pub trait Interface {
    fn new(format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self;
    fn on_init(&mut self) {}
    fn on_ctrls(&mut self) -> &mut CtrlContainer;
    fn on_resize(&mut self, _sx: i32, _sy: i32) {}
    fn on_idle(&mut self) -> Draw {
        Draw::No
    }
    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No
    }
    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        Draw::No
    }
    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        Draw::No
    }
    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _key: u32, _flags: u32,
    ) -> Draw {
        Draw::No
    }
    fn on_ctrl_change(&mut self, _rb: &mut agg::RenderBuf) {}
    fn on_draw(&mut self, rb: &mut agg::RenderBuf);
    //fn on_post_draw(&self, raw_handler: *mut c_void);
}

//----------------------------------------------------------WindowFlag
// These are flags used in method init(). Not all of them are
// applicable on different platforms, for example the win32_api
// cannot use a hardware buffer (window_hw_buffer).
// The implementation should simply ignore unsupported flags.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowFlag {
    Resize = 1,
    HwBuffer = 2,
    KeepAspectRatio = 4,
    ProcessAllKeys = 8,
}

//-----------------------------------------------------------PixFormat
// Possible formats of the rendering buffer. Initially I thought that it's
// reasonable to create the buffer and the rendering functions in
// accordance with the native pixel format of the system because it
// would have no overhead for pixel format conersion.
// But eventually I came to a conclusion that having a possibility to
// convert pixel formats on demand is a good idea. First, it was X11 where
// there lots of different formats and visuals and it would be great to
// render everything in, say, RGB-24 and display it automatically without
// any additional efforts. The second reason is to have a possibility to
// debug renderers for different pixel formats and colorspaces having only
// one computer and one system.
//
// This stuff is not included into the basic AGG functionality because the
// number of supported pixel formats (and/or colorspaces) can be great and
// if one needs to add new format it would be good only to add new
// rendering files without having to modify any existing ones (a general
// principle of incapsulation and isolation).
//
// Using a particular pixel format doesn't obligatory mean the necessity
// of software conversion. For example, win32 API can natively display
// gray8, 15-bit RGB, 24-bit BGR, and 32-bit BGRA formats.
// This list can be (and will be!) extended in future.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PixFormat {
    Undefined = 0, // By default. No conversions are applied
    Bw,            // 1 bit per color B/W
    Gray8,         // Simple 256 level grayscale
    Gray16,        // Simple 65535 level grayscale
    Rgb555,        // 15 bit rgb. Depends on the byte ordering!
    Rgb565,        // 16 bit rgb. Depends on the byte ordering!
    RgbAAA,        // 30 bit rgb. Depends on the byte ordering!
    RgbBBA,        // 32 bit rgb. Depends on the byte ordering!
    BgrAAA,        // 30 bit bgr. Depends on the byte ordering!
    BgrABB,        // 32 bit bgr. Depends on the byte ordering!
    Rgb24,         // R-G-B, one byte per color component
    Bgr24,         // B-G-R, native win32 BMP format.
    Rgba32,        // R-G-B-A, one byte per color component
    Argb32,        // A-R-G-B, native MAC format
    Abgr32,        // A-B-G-R, one byte per color component
    Bgra32,        // B-G-R-A, native win32 BMP format
    Rgb48,         // R-G-B, 16 bits per color component
    Bgr48,         // B-G-R, native win32 BMP format.
    Rgba64,        // R-G-B-A, 16 bits byte per color component
    Argb64,        // A-R-G-B, native MAC format
    Abgr64,        // A-B-G-R, one byte per color component
    Bgra64,        // B-G-R-A, native win32 BMP format

    endOfPixFormats,
}

//-------------------------------------------------------------InputFlag
// Mouse and keyboard flags. They can be different on different platforms
// and the ways they are obtained are also different. But in any case
// the system dependent flags should be mapped into these ones. The meaning
// of that is as follows. For example, if kbd_ctrl is set it means that the
// ctrl key is pressed and being held at the moment. They are also used in
// the overridden methods such as on_mouse_move(), on_mouse_button_down(),
// on_mouse_button_dbl_click(), on_mouse_button_up(), on_key().
// In the method on_mouse_button_up() the mouse flags have different
// meaning. They mean that the respective button is being released, but
// the meaning of the keyboard flags remains the same.
// There's absolut minimal set of flags is used because they'll be most
// probably supported on different platforms. Even the mouse_right flag
// is restricted because Mac's mice have only one button, but AFAIK
// it can be simulated with holding a special key on the keydoard.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputFlag {
    MouseLeft = 1,
    MouseRight = 2,
    KbdShift = 4,
    KbdCtrl = 8,
}

//--------------------------------------------------------------KeyCode
// Keyboard codes. There's also a restricted set of codes that are most
// probably supported on different platforms. Any platform dependent codes
// should be converted into these ones. There're only those codes are
// defined that cannot be represented as printable ASCII-characters.
// All printable ASCII-set can be used in a regular C/C++ manner:
// ' ', 'A', '0' '+' and so on.
// Since the class is used for creating very simple demo-applications
// we don't need very rich possibilities here, just basic ones.
// Actually the numeric key codes are taken from the SDL library, so,
// the implementation of the SDL support does not require any mapping.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Undefined = 0,
    // ASCII set. Should be supported everywhere
    Backspace = 8,
    Tab = 9,
    Clear = 12,
    Return = 13,
    Pause = 19,
    Escape = 27,

    // Keypad
    Delete = 127,
    Kp0 = 256,
    Kp1 = 257,
    Kp2 = 258,
    Kp3 = 259,
    Kp4 = 260,
    Kp5 = 261,
    Kp6 = 262,
    Kp7 = 263,
    Kp8 = 264,
    Kp9 = 265,
    KpPeriod = 266,
    KpDivide = 267,
    KpMultiply = 268,
    KpMinus = 269,
    KpPlus = 270,
    KpEnter = 271,
    KpEquals = 272,

    // Arrow-keys and stuff
    Up = 273,
    Down = 274,
    Right = 275,
    Left = 276,
    Insert = 277,
    Home = 278,
    End = 279,
    PageUp = 280,
    PageDown = 281,

    // Functional keys. You'd better avoid using
    // f11...f15 in your applications if you want
    // the applications to be portable
    F1 = 282,
    F2 = 283,
    F3 = 284,
    F4 = 285,
    F5 = 286,
    F6 = 287,
    F7 = 288,
    F8 = 289,
    F9 = 290,
    F10 = 291,
    F11 = 292,
    F12 = 293,
    F13 = 294,
    F14 = 295,
    F15 = 296,

    // The possibility of using these keys is
    // very restricted. Actually it's guaranteed
    // only in win32Api and win32Sdl implementations
    Numlock = 300,
    Capslock = 301,
    Scrollock = 302,

    // Phew!
    EndOfKeyCodes,
}

//------------------------------------------------------------------------
// A predeclaration of the platform dependent class. Since we do not
// know anything here the only we can have is just a pointer to this
// class as a data member. It should be created and destroyed explicitly
// in the constructor/destructor of the PlatSupport class.
// Although the pointer to platform_specific is public the application
// cannot have access to its members or methods since it does not know
// anything about them and it's a perfect incapsulation :-)
//class platform_specific;

//----------------------------------------------------------CtrlContainer
// A helper class that contains pointers to a number of controls.
// This class is used to ease the event handling with controls.
// The implementation should simply call the appropriate methods
// of this class when appropriate events occur.

const MAX_CTRL: u32 = 64;
#[derive(Clone)]
pub struct CtrlContainer {
    pub ctrl: Vec<Rc<RefCell<dyn Ctrl>>>,
    pub num_ctrl: u32,
    pub cur_ctrl: i32,
}

impl CtrlContainer {
    pub fn new() -> Self {
        CtrlContainer {
            ctrl: vec![],
            num_ctrl: 0,
            cur_ctrl: -1,
        }
    }

    pub fn set_transform(&mut self, mtx: &TransAffine) {
        for i in 0..self.num_ctrl {
            self.ctrl[i as usize].borrow_mut().set_transform(mtx);
        }
    }

    pub fn add(&mut self, c: Rc<RefCell<dyn Ctrl>>) {
        if self.num_ctrl < MAX_CTRL {
            self.ctrl[self.num_ctrl as usize] = c;
            self.num_ctrl += 1;
        }
    }

    pub fn in_rect(&self, x: f64, y: f64) -> bool {
        for i in 0..self.num_ctrl {
            if self.ctrl[i as usize].borrow_mut().in_rect(x, y) {
                return true;
            }
        }
        false
    }

    pub fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
        for i in 0..self.num_ctrl {
            if self.ctrl[i as usize]
                .borrow_mut()
                .on_mouse_button_down(x, y)
            {
                return true;
            }
        }
        false
    }

    pub fn on_mouse_button_up(&mut self, x: f64, y: f64) -> bool {
        let mut flag = false;
        for i in 0..self.num_ctrl {
            if self.ctrl[i as usize].borrow_mut().on_mouse_button_up(x, y) {
                flag = true;
            }
        }
        flag
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64, button_flag: bool) -> bool {
        for i in 0..self.num_ctrl {
            if self.ctrl[i as usize]
                .borrow_mut()
                .on_mouse_move(x, y, button_flag)
            {
                return true;
            }
        }
        false
    }

    pub fn on_arrow_keys(&mut self, left: bool, right: bool, down: bool, up: bool) -> bool {
        if self.cur_ctrl >= 0 {
            self.ctrl[self.cur_ctrl as usize]
                .borrow_mut()
                .on_arrow_keys(left, right, down, up)
        } else {
            false
        }
    }

    pub fn set_cur(&mut self, x: f64, y: f64) -> bool {
        for i in 0..self.num_ctrl {
            if self.ctrl[i as usize].borrow_mut().in_rect(x, y) {
                if self.cur_ctrl != i as i32 {
                    self.cur_ctrl = i as i32;
                    return true;
                }
                return false;
            }
        }
        if self.cur_ctrl != -1 {
            self.cur_ctrl = -1;
            return true;
        }
        false
    }
}

//---------------------------------------------------------PlatSupport
// This class is a base one to the apllication classes. It can be used
// as follows:
//
//  class the_application : public agg::PlatSupport
//  {
//  public:
//      the_application(unsigned bpp, bool flip_y) :
//          PlatSupport(bpp, flip_y)
//      . . .
//
//      //override stuff . . .
//      virtual void on_init()
//      {
//         . . .
//      }
//
//      virtual void on_draw()
//      {
//          . . .
//      }
//
//      virtual void on_resize(int sx, int sy)
//      {
//          . . .
//      }
//      // . . . and so on, see virtual functions
//
//
//      //any your own stuff . . .
//  };
//
//
//  int agg_main(int argc, char* argv[])
//  {
//      the_application app(pix_format_rgb24, true);
//      app.caption("AGG Example. Lion");
//
//      if(app.init(500, 400, agg::window_resize))
//      {
//          return app.run();
//      }
//      return 1;
//  }
//
// The reason to have agg_main() instead of just main() is that SDL
// for Windows requires including SDL.h if you define main(). Since
// the demo applications cannot rely on any platform/library specific
// stuff it's impossible to include SDL.h into the application files.
// The demo applications are simple and their use is restricted, so,
// this approach is quite reasonable.
//
pub const MAX_IMAGES: u32 = 16;
pub struct PlatUtil {
    specific: PlatSpecific,
    rbuf_img: [agg::RenderBuf; MAX_IMAGES as usize],
    width: u32,
    height: u32,
    initial_width: u32,
    initial_height: u32,
    wait_mode: bool,
    resize_mtx: TransAffine,
}

impl PlatUtil {
    pub fn new(format: PixFormat, flip_y: bool) -> Self {
        let rbuf = [RenderBuf::new_default(); MAX_IMAGES as usize];

        Self {
            specific: PlatSpecific::new(format, flip_y),
            rbuf_img: rbuf, //[Rc::new(RefCell::new(RenderBuffer::new_zero())); MAX_IMAGES as usize],
            width: 0,
            height: 0,
            initial_width: 0,
            initial_height: 0,
            wait_mode: true,
            resize_mtx: TransAffine::new_default(),
        }
    }

    pub fn plat_specific(&mut self, specific: &PlatSpecific, w: u32, h: u32) {
        self.specific = *specific;
        self.width = w;
        self.height = h;
    }

    pub fn width(&self) -> f64 {
        self.width as f64
    }
    pub fn height(&self) -> f64 {
        self.height as f64
    }
    pub fn initial_width(&self) -> f64 {
        self.initial_width as f64
    }
    pub fn initial_height(&self) -> f64 {
        self.initial_height as f64
    }

    pub fn set_trans_affine_resizing(&mut self, mtx: &TransAffine) {
        self.resize_mtx = *mtx;
    }

    pub fn trans_affine_resizing(&self) -> &TransAffine {
        &self.resize_mtx
    }

    pub fn set_initial(&mut self, w: u32, h: u32) {
        self.initial_width = w;
        self.initial_height = h;
    }

    // The following provides a very simple mechanism of doing someting
    // in background. It's not multithreading. When wait_mode is true
    // the class waits for the events and it does not ever call on_idle().
    // When it's false it calls on_idle() when the event queue is empty.
    // The mode can be changed anytime. This mechanism is satisfactory
    // to create very simple animations.
    pub fn wait_mode(&self) -> bool {
        self.wait_mode
    }
    pub fn set_wait_mode(&mut self, m: bool) {
        self.wait_mode = m;
    }

    pub fn rbuf_img(&self, idx: u32) -> &agg::RenderBuf {
        &self.rbuf_img[idx as usize]
    }

    pub fn rbuf_img_mut(&mut self, idx: u32) -> &mut agg::RenderBuf {
        &mut self.rbuf_img[idx as usize]
    }

    pub fn copy_img_to_window(&mut self, dst: &mut agg::RenderBuf, idx: u32) {
        if idx < MAX_IMAGES && self.rbuf_img(idx).buf() != std::ptr::null() {
            dst.copy_from(&*self.rbuf_img(idx));
        }
    }

    pub fn copy_window_to_img(&mut self, dst: &agg::RenderBuf, idx: u32) {
        let w = dst.width();
        let h = dst.height();
        if idx < MAX_IMAGES {
            self.create_img(idx, w, h);

            self.rbuf_img_mut(idx).copy_from(dst);
        }
    }

    pub fn copy_img_to_img(&mut self, idx_to: u32, idx_from: u32) {
        if idx_from < MAX_IMAGES && idx_to < MAX_IMAGES && !self.rbuf_img(idx_from).buf().is_null()
        {
            let w = self.rbuf_img(idx_from).width();
            let h = self.rbuf_img(idx_from).height();
            self.create_img(idx_to, w, h);
            let mut to = self.rbuf_img[idx_to as usize];
            let frm = self.rbuf_img[idx_from as usize];
            to.copy_from(&frm);
        }
    }
}

pub struct PlatSupport<App: Interface> {
    specific: PlatSpecific,
    //ctrls: CtrlContainer,
    format: PixFormat,
    bpp: u32,
    rbuf_window: agg::RenderBuf,
    rbuf_img: [agg::RenderBuf; MAX_IMAGES as usize],
    window_flags: u32,

    flip_y: bool, // flip_y - true if you want to have the Y-axis flipped vertically.
    caption: String,
    initial_width: u32,
    initial_height: u32,
    resize_mtx: TransAffine,
    demo: App,
    util: Rc<RefCell<PlatUtil>>,
}

impl<App: Interface> PlatSupport<App> {
    pub fn create_plat(format: PixFormat, flip_y: bool) -> PlatSupport<App> {
        let util = Rc::new(RefCell::new(PlatUtil::new(format, flip_y)));
        let app = App::new(format, flip_y, util.clone());
        let plat = PlatSupport::new(app, format, flip_y, util);
        plat
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.demo
    }

    pub fn app(&mut self) -> &App {
        &self.demo
    }
    // Setting the windows caption (title). Should be able
    // to be called at least before calling init().
    // It's perfect if they can be called anytime.
    pub fn caption(&self) -> *const u8 {
        self.caption.as_ptr()
    }

    // The very same parameters that were used in the constructor
    pub fn format(&self) -> PixFormat {
        self.format
    }
    pub fn flip_y(&self) -> bool {
        self.flip_y
    }
    pub fn bpp(&self) -> u32 {
        self.bpp
    }

    // So, finally, how to draw anythig with AGG? Very simple.
    // rbuf_window() returns a reference to the main rendering
    // buffer which can be attached to any rendering class.
    // rbuf_img() returns a reference to the previously created
    // or loaded image buffer (see load_img()). The image buffers
    // are not displayed directly, they should be copied to or
    // combined somehow with the rbuf_window().borrow(). rbuf_window() is
    // the only buffer that can be actually displayed.
    pub fn rbuf_window(&mut self) -> &mut agg::RenderBuf {
        &mut self.rbuf_window
    }

    // Adding control elements. A control element once added will be
    // working and reacting to the mouse and keyboard events. Still, you
    // will have to render them in the on_draw() using function
    // render_ctrl() because PlatSupport doesn't know anything about
    // renderers you use. The controls will be also scaled automatically
    // if they provide a proper scaling mechanism (all the controls
    // included into the basic AGG package do).
    // If you don't need a particular control to be scaled automatically
    // call ctrl::no_transform() after adding.

    /*pub fn add_ctrl(&mut self, c: &mut ctrl) {
        self.ctrls.add(c);
        c.transform(&self.resize_mtx as *const _);
    }*/

    // Auxiliary functions. trans_affine_resizing() modifier sets up the resizing
    // matrix on the basis of the given width and height and the initial
    // width and height of the window. The implementation should simply
    // call this function every time when it catches the resizing event
    // passing in the new values of width and height of the window.
    // Nothing prevents you from "cheating" the scaling matrix if you
    // call this function from somewhere with wrong arguments.
    // trans_affine_resizing() accessor simply returns current resizing matrix
    // which can be used to apply additional scaling of any of your
    // stuff when the window is being resized.
    // width(), height(), initial_width(), and initial_height() must be
    // clear to understand with no comments :-)
    fn set_trans_affine_resizing(&mut self, width: i32, height: i32) {
        if self.window_flags & WindowFlag::KeepAspectRatio as u32 != 0 {
            let mut vp = TransViewport::new();
            vp.set_preserve_aspect_ratio(0.5, 0.5, AspectRatio::Meet);
            vp.set_device_viewport(0., 0., width as f64, height as f64);
            vp.set_world_viewport(
                0.,
                0.,
                self.initial_width as f64,
                self.initial_height as f64,
            );
            self.resize_mtx = vp.to_affine();
        } else {
            self.resize_mtx = TransAffine::trans_affine_scaling(
                (width as f64) / (self.initial_width as f64),
                (height as f64) / (self.initial_height as f64),
            );
        }
    }

    pub fn trans_affine_resizing(&self) -> &TransAffine {
        &self.resize_mtx
    }

    pub fn width(&self) -> f64 {
        self.rbuf_window.width() as f64
    }

    pub fn height(&self) -> f64 {
        self.rbuf_window.height() as f64
    }

    pub fn initial_width(&self) -> f64 {
        self.initial_width as f64
    }

    pub fn initial_height(&self) -> f64 {
        self.initial_height as f64
    }

    pub fn window_flags(&self) -> u32 {
        self.window_flags
    }
}
