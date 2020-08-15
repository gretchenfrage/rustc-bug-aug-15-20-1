/// Utility for tracking `winit::window::Window` input. 

use vek::*;
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
    borrow::Borrow,
    convert::identity,
};
use winit::{
    window::Window,
    event::{
        WindowEvent,
        DeviceEvent,
        KeyboardInput,
        MouseScrollDelta,
        ElementState,
    },
    dpi::{
        Position,
        PhysicalSize,
        PhysicalPosition,
    },
};
use smallvec::SmallVec;

/// Factory pattern for an `InputManager`. Used to bind keys. 
#[derive(Clone)]
pub struct InputManagerBuilder {
    bindings: Vec<Button>,
}

impl InputManagerBuilder {
    /// Start building an `InputManager`. 
    pub fn new() -> Self {
        InputManagerBuilder {
            bindings: Vec::new()
        }
    }

    /// Create a binding to a button. 
    pub fn bind<B: Into<Button>>(&mut self, bind_to: B) -> KeyBind {
        let bind_int = self.bindings.len();
        assert!(bind_int < u16::MAX as usize, "too many key bindings");
        self.bindings.push(bind_to.into());
        KeyBind(bind_int as u16)
    }

    /// Create the `InputManager`. 
    pub fn build(self, window: Arc<Window>) -> InputManager {
        let mut bindings = HashMap::new();
        let mut bindings_rev = HashMap::new();
        for (bind_int, button) in self.bindings.iter().copied().enumerate() {
            let bind_int = bind_int as u16;

            bindings
                .entry(button)
                .or_insert_with(Vec::new)
                .push(KeyBind(bind_int));
            bindings_rev.insert(KeyBind(bind_int), button);
        }

        InputManager {
            state: WindowState::Unfocused,
            bindings,
            bindings_rev,
            pressed: HashSet::new(),
            window_size: window.inner_size(),
            window_closing: false,
            window_scale_factor: window.scale_factor() as f64,
            cursor_location: [0.0; 2].into(),

            events: Vec::new(),
            mouse_captured_movement: [0.0; 2].into(),
            mouse_scroll: [0.0; 2].into(), 

            window,
        }
    }
}

/// Alias for `winit::event::Event<'static, ()>`.
pub type WinitEvent = winit::event::Event<'static, ()>;

/// A key on a keyboard. 
pub type Key = winit::event::VirtualKeyCode;

/// A button on a mouse. 
pub type MouseButton = winit::event::MouseButton;

/// A hardware button which can be pressed and released. 
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Button {
    Key(Key),
    Mouse(MouseButton),
}

impl From<Key> for Button {
    fn from(key: Key) -> Button {
        Button::Key(key)
    }
}

impl From<MouseButton> for Button {
    fn from(button: MouseButton) -> Button {
        Button::Mouse(button)
    }
}

/// A virtual "game" key, which may be bound to any `Button`. 
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyBind(u16);

/// Input state at a moment of time. 
#[derive(Clone, Debug)]
pub struct InputSnapshot {
    state: WindowState,
    cursor_location: Vec2<f64>,
    pressed: SmallVec<[KeyBind; 10]>,
}

impl InputSnapshot {
    /// Return the cursor location at this snapshot, normalized from 0 to 1. 
    ///
    /// If the mouse is captured, or if the window is unfocused, this may 
    /// return non-useful data. 
    pub fn cursor_location(&self) -> Vec2<f64> {
        self.cursor_location
    }

    /// Return whether a binded key is pressed at this snapshot. 
    pub fn is_pressed(&self, bind: KeyBind) -> bool {
        self.pressed.contains(&bind)
    }

    /// Return the window's `WindowState` at this snapshot.
    pub fn state(&self) -> WindowState {
        self.state
    }
}

/// Discrete input events which accumulate per-frame. 
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum InputEvent {
    /// A binded key was pressed. 
    Press(KeyBind),
    /// A binded key was unpressed. 
    Unpress(KeyBind),
    /// A mouse button was clicked. This is independent of whether it was 
    /// binded. We suggest to bind the mouse for captured input, but use 
    /// raw clicks for gui input. 
    Click(MouseButton),
    /// A mouse button was ybckucjed. This is independent of whether it was 
    /// binded. It's suggested to bind the mouse for captured input, but use 
    /// raw clicks for gui input. 
    Unclick(MouseButton),
}

/// A state the `InputManager` can be in.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum WindowState {
    /// The cursor is captured. 
    Captured,
    /// The window is focused but the cursor is not captured. 
    Focused,
    /// The window is unfocused. 
    Unfocused,
}

/// Utility for tracking `winit::window::Window` input. 
pub struct InputManager {
    window: Arc<Window>,

    // state which is tracking continually
    state: WindowState,
    bindings: HashMap<Button, Vec<KeyBind>>,
    bindings_rev: HashMap<KeyBind, Button>,
    pressed: HashSet<Button>,
    window_size: PhysicalSize<u32>,
    window_closing: bool,
    window_scale_factor: f64,
    cursor_location: Vec2<f64>,

    // accumulators which reset every frame
    events: Vec<(InputEvent, InputSnapshot)>,
    mouse_captured_movement: Vec2<f64>,
    mouse_scroll: Vec2<f64>,
}

impl InputManager {
    /// Update the input manager with the new frame's events. 
    /// 
    /// This will reset and re-accumulate some things, and be used to track 
    /// the latest state of other things. 
    pub fn update<I>(&mut self, winit_events: I)
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Borrow<WinitEvent>,
    {
        // reset accumulators
        self.events.clear();
        self.mouse_captured_movement = [0.0; 2].into();
        self.mouse_scroll = [0.0; 2].into();

        for event in winit_events
            .into_iter()
            .filter_map(|e| InterestEvent::option_from(e.borrow()))
        {
            match event {
                InterestEvent::KeyStateChange { 
                    state: ElementState::Pressed, 
                    key,
                } => {
                    if let Some(binds) = self.bindings.get(&key.into()) {
                        for &bind in binds {
                            self.events.push((
                                InputEvent::Press(bind),
                                self.snapshot(),
                            ));
                        }
                    }
                    self.pressed.insert(key.into());

                    // automatically un-capture the mouse when Esc is pressed
                    if key == Key::Escape {
                        self.uncapture_mouse();
                    }
                }
                InterestEvent::KeyStateChange {
                    state: ElementState::Released,
                    key,
                } => {
                    if let Some(binds) = self.bindings.get(&key.into()) {
                        for &bind in binds {
                            self.events.push((
                                InputEvent::Unpress(bind),
                                self.snapshot(),
                            ));
                        }
                    }
                    self.pressed.remove(&key.into());
                }
                InterestEvent::MouseStateChange {
                    state: ElementState::Pressed,
                    button,
                } => {
                    if let Some(binds) = self.bindings.get(&button.into()) {
                        for &bind in binds {
                            self.events.push((
                                InputEvent::Press(bind),
                                self.snapshot(),
                            ));
                        }
                    }
                    self.events.push((
                        InputEvent::Click(button),
                        self.snapshot(),
                    ));
                    self.pressed.insert(button.into());
                }
                InterestEvent::MouseStateChange {
                    state: ElementState::Released,
                    button,
                } => {
                    if let Some(binds) = self.bindings.get(&button.into()) {
                        for &bind in binds {
                            self.events.push((
                                InputEvent::Unpress(bind),
                                self.snapshot(),
                            ));
                        }
                    }
                    self.events.push((
                        InputEvent::Unclick(button),
                        self.snapshot(),
                    ));
                    self.pressed.remove(&button.into());
                }
                InterestEvent::NewCursorPosition { position } => {
                    let position = Vec2::new(
                        position.x as f64, 
                        position.y as f64,
                    );
                    let size = Vec2::new(
                        self.window_size.width as f64, 
                        self.window_size.height as f64,
                    );
                    self.cursor_location = size / position;
                }
                InterestEvent::MouseMovement { x, y } => {
                    if self.state == WindowState::Captured {
                        self.mouse_captured_movement += Vec2::new(
                            x as f64,
                            y as f64,
                        );
                    }
                }
                InterestEvent::MouseWheelScroll { delta } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(x, y) => Vec2 { 
                            x: x as f64,
                            y: y as f64,
                        },
                        MouseScrollDelta::PixelDelta(position) => Vec2 {
                            x: position.x,
                            y: position.y,
                        },
                    };
                    if self.state != WindowState::Unfocused {
                        self.mouse_scroll += delta;
                    }
                }
                InterestEvent::FocusChanged { focused: false } => {
                    self.state = WindowState::Unfocused;
                }
                InterestEvent::FocusChanged { focused: true } => {
                    self.state = WindowState::Focused;
                }
                InterestEvent::CloseRequested {} => {
                    self.window_closing = true;
                }
                InterestEvent::WindowDestroyed {} => {
                    self.window_closing = true;
                }
                InterestEvent::SizeChanged { new_size } => {
                    self.window_size = new_size;
                }
                InterestEvent::ScaleFactorChanged { 
                    scale_factor, 
                    new_inner_size,
                } => {
                    self.window_scale_factor = scale_factor as f64;
                    self.window_size = new_inner_size;
                }
            };
        }
    }

    /// Return the window's current `WindowState`.
    pub fn state(&self) -> WindowState {
        self.state
    }

    /// Access the input events that occured in the last frame. 
    ///
    /// These are a (event, snapshot) tuples, which store the exact
    /// input state when the event occured. This can be used to implement 
    /// modifiers.  
    pub fn events(&self) -> &[(InputEvent, InputSnapshot)] {
        &self.events
    }

    /// Return the cursor location, normalized from 0 to 1. 
    ///
    /// If the mouse is captured, or if the window is unfocused, this
    /// may return non-useful data. 
    pub fn cursor_location(&self) -> Vec2<f64> {
        self.cursor_location
    }

    /// Return whether a binded key is pressed at this snapshot. 
    pub fn is_pressed(&self, bind: KeyBind) -> bool {
        self.pressed.contains(&self.bindings_rev[&bind])
    }

    /// Return whether the window is being closed or destroyed. 
    pub fn is_closing(&self) -> bool {
        self.window_closing
    }

    /// Return the sum of captured mouse movement since the last frame. 
    ///
    /// Mouse movement will only be captured while the `InputManager` 
    /// is in the `Captured` state. 
    pub fn mouse_captured_movement(&self) -> Vec2<f64> {
        self.mouse_captured_movement
    }

    /// Return the sum of mouse scrolling since the last frame.
    pub fn mouse_scroll(&self) -> Vec2<f64> {
        self.mouse_scroll
    }

    /// Return the window's current scale factor. 
    pub fn scale_factor(&self) -> f64 {
        self.window_scale_factor
    }

    /// Attempt to capture the cursor, and place the window into the 
    /// `Captured` state. 
    ///
    /// This might not necessarily succeed. Also, this will automatically
    /// fail if the window is unfocused. 
    pub fn capture_mouse(&mut self) {
        if self.state == WindowState::Unfocused {
            warn!("attempt to capture un-focused window");
            return;
        }
        if self.window.set_cursor_grab(true).is_ok() {
            self.window.set_cursor_visible(false);
            self.state = WindowState::Captured;
        }
    }

    /// Release the cursor from the `Captured` state. 
    pub fn uncapture_mouse(&mut self) {
        if self.state == WindowState::Captured {
            let pos = Position::Physical(PhysicalPosition {
                x: self.window_size.width as i32 / 2,
                y: self.window_size.height as i32 / 2,
            });
            let _ = self.window.set_cursor_position(pos);

            self.state = WindowState::Focused;
        }
        let _ = self.window.set_cursor_grab(false);
        self.window.set_cursor_visible(true);
    }

    /// Capture the current state of input. 
    pub fn snapshot(&self) -> InputSnapshot {
        let pressed = self.pressed
            .iter()
            .filter_map(|button| self.bindings.get(button))
            .flat_map(identity)
            .copied()
            .collect();
        InputSnapshot {
            state: self.state,
            cursor_location: self.cursor_location,
            pressed,
        }
    }
}

macro_rules! match_simplifier {
    (
        $(#[$($attr:tt)*])*
        $from:ty => $into:ident {$(
            $pattern:pat => $variant:ident {$(
                $field:ident: $field_ty:ty,
            )*},
        )*}
    )=>{
        $(#[$($attr)*])*
        enum $into {$(
            $variant {$(
                $field: $field_ty,
            )*},
        )*}

        impl $into {
            fn option_from(from: &$from) -> Option<$into> {
                match from {
                    $(
                        $pattern => Some($into::$variant {$(
                            $field,
                        )*}),
                    )*
                    _ => None,
                }
            }
        }
    };
}

match_simplifier! {
    /// Subset of `WinitEvent` data that we are actually interested in. 
    WinitEvent => InterestEvent {
        &WinitEvent::WindowEvent {
            event: WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            },
            ..
        } => KeyStateChange {
            state: ElementState,
            key: Key,
        },
        &WinitEvent::WindowEvent {
            event: WindowEvent::MouseInput {
                state,
                button,
                ..
            },
            ..
        } => MouseStateChange {
            state: ElementState,
            button: MouseButton,
        },
        &WinitEvent::WindowEvent {
            event: WindowEvent::CursorMoved {
                position,
                ..
            },
            ..
        } => NewCursorPosition {
            position: PhysicalPosition<f64>,
        },
        &WinitEvent::DeviceEvent {
            event: DeviceEvent::MouseMotion {
                delta: (x, y),
            },
            ..
        } => MouseMovement {
            x: f64,
            y: f64,
        },
        &WinitEvent::WindowEvent {
            event: WindowEvent::MouseWheel {
                delta,
                ..
            },
            ..
        } => MouseWheelScroll {
            delta: MouseScrollDelta,
        },
        &WinitEvent::WindowEvent {
            event: WindowEvent::Focused(focused),
            ..
        } => FocusChanged {
            focused: bool,
        },
        &WinitEvent::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => CloseRequested {},
        &WinitEvent::WindowEvent {
            event: WindowEvent::Destroyed,
            ..
        } => WindowDestroyed {},
        &WinitEvent::WindowEvent {
            event: WindowEvent::Resized(new_size),
            ..
        } => SizeChanged {
            new_size: PhysicalSize<u32>,
        },
        &WinitEvent::WindowEvent {
            event: WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size: &mut new_inner_size
            },
            ..
        } => ScaleFactorChanged {
            scale_factor: f64,
            new_inner_size: PhysicalSize<u32>,
        },
    }
}
