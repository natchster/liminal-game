//! Window, input and frame loop: winit + a CAMetalLayer, keyboard, mouse,
//! pointer lock and fullscreen, and a vsync'd frame callback.

use crate::render::Renderer;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::NSView;
use objc2_metal::*;
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

/// Input events. Keys are named with the standard key-code names (`KeyW`,
/// `ShiftLeft`, `Space`, `Escape`, ...; see `key_code_name`).
#[derive(Clone, Debug)]
pub enum Input {
    /// key pressed: its key-code name (e.g. "KeyW", "ShiftLeft", "Space", "Escape") and whether it is an auto-repeat
    KeyDown(String, bool),
    KeyUp(String),
    /// raw mouse motion (relative x / y) while the pointer is locked
    MouseMove(f64, f64),
    /// cursor position in logical pixels (for menus)
    CursorPos(f64, f64),
    MouseDown(u8),
    MouseUp(u8),
    Wheel(f64),
    /// pointer lock was acquired (true) or released (false)
    PointerLock(bool),
    /// the window lost focus
    Blur,
    Resize,
    CloseRequested,
}

/// What the game can ask of the window.
pub struct AppCtx<'a> {
    pub window: &'a Window,
    pub renderer: &'a mut Renderer,
    pub drawable: Option<&'a ProtocolObject<dyn MTLTexture>>,
    /// logical (point) size and the backing scale factor
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
    pub locked: bool,
    pub fullscreen: bool,
    requests: &'a mut Requests,
}
#[derive(Default)]
struct Requests {
    lock: Option<bool>,
    fullscreen: Option<bool>,
    quit: bool,
    cursor: Option<bool>,
}
impl AppCtx<'_> {
    /// Lock (hide and capture) or release the pointer.
    pub fn set_pointer_lock(&mut self, on: bool) {
        self.requests.lock = Some(on);
    }
    pub fn set_fullscreen(&mut self, on: bool) {
        self.requests.fullscreen = Some(on);
    }
    pub fn set_cursor_visible(&mut self, on: bool) {
        self.requests.cursor = Some(on);
    }
    pub fn quit(&mut self) {
        self.requests.quit = true;
    }
    pub fn physical_size(&self) -> (usize, usize) {
        ((self.width * self.scale_factor).round() as usize, (self.height * self.scale_factor).round() as usize)
    }
}

/// The game side of the loop.
pub trait AppHandler {
    /// Once, after the window and renderer exist.
    fn init(&mut self, ctx: &mut AppCtx);
    /// Every frame (vsync). Draw into `ctx.drawable` with `ctx.renderer` and
    /// return the command buffer to present.
    fn frame(&mut self, ctx: &mut AppCtx, dt: f64) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>;
    fn input(&mut self, ctx: &mut AppCtx, ev: Input);
}

pub struct AppConfig {
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub fullscreen: bool,
}

struct Runner<H: AppHandler> {
    cfg: AppConfig,
    handler: H,
    window: Option<Window>,
    layer: Option<Retained<CAMetalLayer>>,
    renderer: Option<Renderer>,
    last: Instant,
    locked: bool,
    fullscreen: bool,
    started: bool,
    req: Requests,
}

pub fn run<H: AppHandler + 'static>(cfg: AppConfig, handler: H) {
    let el = EventLoop::new().expect("event loop");
    el.set_control_flow(ControlFlow::Poll);
    let fs = cfg.fullscreen;
    let mut r = Runner {
        cfg,
        handler,
        window: None,
        layer: None,
        renderer: None,
        last: Instant::now(),
        locked: false,
        fullscreen: fs,
        started: false,
        req: Requests::default(),
    };
    el.run_app(&mut r).expect("run");
}

impl<H: AppHandler> Runner<H> {
    fn ctx<'a>(
        window: &'a Window,
        renderer: &'a mut Renderer,
        drawable: Option<&'a ProtocolObject<dyn MTLTexture>>,
        locked: bool,
        fullscreen: bool,
        req: &'a mut Requests,
    ) -> AppCtx<'a> {
        let scale_factor = window.scale_factor();
        let sz = window.inner_size();
        AppCtx {
            window,
            renderer,
            drawable,
            width: sz.width as f64 / scale_factor,
            height: sz.height as f64 / scale_factor,
            scale_factor,
            locked,
            fullscreen,
            requests: req,
        }
    }
    fn apply_requests(&mut self, el: &ActiveEventLoop) {
        let Some(w) = &self.window else { return };
        if let Some(on) = self.req.lock.take() {
            if on {
                let ok = w.set_cursor_grab(CursorGrabMode::Locked).or_else(|_| w.set_cursor_grab(CursorGrabMode::Confined)).is_ok();
                w.set_cursor_visible(false);
                if ok && !self.locked {
                    self.locked = true;
                    self.send(Input::PointerLock(true));
                }
            } else {
                let _ = w.set_cursor_grab(CursorGrabMode::None);
                w.set_cursor_visible(true);
                if self.locked {
                    self.locked = false;
                    self.send(Input::PointerLock(false));
                }
            }
        }
        if let Some(c) = self.req.cursor.take() {
            if let Some(w) = &self.window {
                w.set_cursor_visible(c);
            }
        }
        if let Some(f) = self.req.fullscreen.take() {
            if let Some(w) = &self.window {
                w.set_fullscreen(if f { Some(Fullscreen::Borderless(None)) } else { None });
            }
            self.fullscreen = f;
        }
        if self.req.quit {
            el.exit();
        }
    }
    fn send(&mut self, ev: Input) {
        let (Some(w), Some(r)) = (&self.window, &mut self.renderer) else { return };
        let mut c = Self::ctx(w, r, None, self.locked, self.fullscreen, &mut self.req);
        self.handler.input(&mut c, ev);
    }
    fn update_layer_size(&self) {
        if let (Some(w), Some(l)) = (&self.window, &self.layer) {
            let s = w.inner_size();
            l.setDrawableSize(objc2_core_foundation::CGSize { width: s.width.max(1) as f64, height: s.height.max(1) as f64 });
            l.setContentsScale(w.scale_factor());
        }
    }
}

impl<H: AppHandler> ApplicationHandler for Runner<H> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let mut attrs = Window::default_attributes()
            .with_title(self.cfg.title.clone())
            .with_inner_size(LogicalSize::new(self.cfg.width, self.cfg.height));
        if self.cfg.fullscreen {
            attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        let window = el.create_window(attrs).expect("window");
        let device = MTLCreateSystemDefaultDevice().expect("no Metal device");
        let layer = CAMetalLayer::new();
        layer.setDevice(Some(&device));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        layer.setFramebufferOnly(true);
        layer.setMaximumDrawableCount(3);
        if let Ok(h) = window.window_handle() {
            if let RawWindowHandle::AppKit(h) = h.as_raw() {
                let view: &NSView = unsafe { h.ns_view.cast().as_ref() };
                view.setWantsLayer(true);
                view.setLayer(Some(&layer));
            }
        }
        self.renderer = Some(Renderer::new(device));
        self.layer = Some(layer);
        self.window = Some(window);
        self.update_layer_size();
        {
            let (w, r) = (self.window.as_ref().unwrap(), self.renderer.as_mut().unwrap());
            let mut c = Self::ctx(w, r, None, self.locked, self.fullscreen, &mut self.req);
            self.handler.init(&mut c);
        }
        self.started = true;
        self.apply_requests(el);
        self.last = Instant::now();
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => {
                self.send(Input::CloseRequested);
                el.exit();
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                self.update_layer_size();
                self.send(Input::Resize);
            }
            WindowEvent::Focused(false) => {
                if self.locked {
                    self.req.lock = Some(false);
                }
                self.send(Input::Blur);
                self.apply_requests(el);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let code = match event.physical_key {
                    PhysicalKey::Code(c) => key_code_name(c),
                    _ => String::new(),
                };
                if code.is_empty() {
                    return;
                }
                // Escape always releases pointer lock
                if code == "Escape" && event.state == ElementState::Pressed && self.locked {
                    self.req.lock = Some(false);
                    self.apply_requests(el);
                }
                match event.state {
                    ElementState::Pressed => self.send(Input::KeyDown(code, event.repeat)),
                    ElementState::Released => self.send(Input::KeyUp(code)),
                }
                self.apply_requests(el);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let b = match button {
                    MouseButton::Left => 0,
                    MouseButton::Middle => 1,
                    MouseButton::Right => 2,
                    _ => 3,
                };
                match state {
                    ElementState::Pressed => self.send(Input::MouseDown(b)),
                    ElementState::Released => self.send(Input::MouseUp(b)),
                }
                self.apply_requests(el);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);
                self.send(Input::CursorPos(position.x / scale, position.y / scale));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64 * 40.0,
                    MouseScrollDelta::PixelDelta(p) => p.y,
                };
                self.send(Input::Wheel(d));
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last).as_secs_f64();
                self.last = now;
                let Some(layer) = self.layer.clone() else { return };
                let drawable = layer.nextDrawable();
                if let (Some(w), Some(r)) = (&self.window, &mut self.renderer) {
                    let tex = drawable.as_ref().map(|d| d.texture());
                    let mut c = Self::ctx(w, r, tex.as_deref(), self.locked, self.fullscreen, &mut self.req);
                    let cb = self.handler.frame(&mut c, dt);
                    if let (Some(cb), Some(d)) = (cb, drawable.as_ref()) {
                        cb.presentDrawable(ProtocolObject::from_ref(&**d));
                        cb.commit();
                    }
                }
                self.apply_requests(el);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: DeviceId, ev: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = ev {
            if self.locked {
                self.send(Input::MouseMove(delta.0, delta.1));
            }
        }
    }
}

/// winit key → its standard key-code name ("KeyW", "Digit1", "ShiftLeft", "ArrowUp", ...).
pub fn key_code_name(c: KeyCode) -> String {
    use KeyCode::*;
    let s = match c {
        KeyA => "KeyA",
        KeyB => "KeyB",
        KeyC => "KeyC",
        KeyD => "KeyD",
        KeyE => "KeyE",
        KeyF => "KeyF",
        KeyG => "KeyG",
        KeyH => "KeyH",
        KeyI => "KeyI",
        KeyJ => "KeyJ",
        KeyK => "KeyK",
        KeyL => "KeyL",
        KeyM => "KeyM",
        KeyN => "KeyN",
        KeyO => "KeyO",
        KeyP => "KeyP",
        KeyQ => "KeyQ",
        KeyR => "KeyR",
        KeyS => "KeyS",
        KeyT => "KeyT",
        KeyU => "KeyU",
        KeyV => "KeyV",
        KeyW => "KeyW",
        KeyX => "KeyX",
        KeyY => "KeyY",
        KeyZ => "KeyZ",
        Digit0 => "Digit0",
        Digit1 => "Digit1",
        Digit2 => "Digit2",
        Digit3 => "Digit3",
        Digit4 => "Digit4",
        Digit5 => "Digit5",
        Digit6 => "Digit6",
        Digit7 => "Digit7",
        Digit8 => "Digit8",
        Digit9 => "Digit9",
        ShiftLeft => "ShiftLeft",
        ShiftRight => "ShiftRight",
        ControlLeft => "ControlLeft",
        ControlRight => "ControlRight",
        AltLeft => "AltLeft",
        AltRight => "AltRight",
        SuperLeft => "MetaLeft",
        SuperRight => "MetaRight",
        Space => "Space",
        Enter => "Enter",
        Escape => "Escape",
        Tab => "Tab",
        Backspace => "Backspace",
        ArrowUp => "ArrowUp",
        ArrowDown => "ArrowDown",
        ArrowLeft => "ArrowLeft",
        ArrowRight => "ArrowRight",
        F1 => "F1",
        F2 => "F2",
        F3 => "F3",
        F4 => "F4",
        F5 => "F5",
        F6 => "F6",
        F7 => "F7",
        F8 => "F8",
        F9 => "F9",
        F10 => "F10",
        F11 => "F11",
        F12 => "F12",
        Minus => "Minus",
        Equal => "Equal",
        BracketLeft => "BracketLeft",
        BracketRight => "BracketRight",
        Backslash => "Backslash",
        Semicolon => "Semicolon",
        Quote => "Quote",
        Comma => "Comma",
        Period => "Period",
        Slash => "Slash",
        Backquote => "Backquote",
        CapsLock => "CapsLock",
        Delete => "Delete",
        Home => "Home",
        End => "End",
        PageUp => "PageUp",
        PageDown => "PageDown",
        _ => return format!("{:?}", c),
    };
    s.to_string()
}
