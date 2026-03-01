// =============================================================================
// heyDM — Input Handler
//
// Processes keyboard and pointer events from the backend (winit or libinput).
// Routes input to the focused window, handles compositor keybindings
// (e.g., Super+Enter to open terminal, Super+D for launcher), and manages
// pointer-driven window interactions (move, resize, focus).
// =============================================================================

use smithay::backend::input::{
    AbsolutePositionEvent, Axis, ButtonState, Event, InputBackend, InputEvent,
    KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
    PointerMotionEvent,
};
use smithay::input::keyboard::{FilterResult, ModifiersState};
use smithay::input::pointer::{AxisFrame, ButtonEvent, MotionEvent};
use smithay::utils::SERIAL_COUNTER;

use tracing::info;

use crate::state::HeyDM;

/// Modifier key state tracked for compositor keybindings
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct ModifierState {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub logo: bool, // Super/Windows key
}

pub struct InputHandler;

impl InputHandler {
    /// Main input event dispatcher — routes backend input events
    pub fn handle_input<B: InputBackend>(state: &mut HeyDM, event: InputEvent<B>) {
        match event {
            InputEvent::Keyboard { event } => {
                Self::handle_keyboard::<B>(state, event);
            }
            InputEvent::PointerMotion { event } => {
                Self::handle_pointer_motion::<B>(state, event);
            }
            InputEvent::PointerMotionAbsolute { event } => {
                Self::handle_pointer_motion_absolute::<B>(state, event);
            }
            InputEvent::PointerButton { event } => {
                Self::handle_pointer_button::<B>(state, event);
            }
            InputEvent::PointerAxis { event } => {
                Self::handle_pointer_axis::<B>(state, event);
            }
            _ => {}
        }
    }

    /// Handle keyboard key press/release events
    fn handle_keyboard<B: InputBackend>(state: &mut HeyDM, event: B::KeyboardKeyEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time_msec();
        let key_code = event.key_code();
        let key_state = event.state();

        let keyboard = state.seat.get_keyboard().unwrap();

        keyboard.input::<(), _>(
            state,
            key_code,
            key_state,
            serial,
            time,
            |state, modifiers, keysym| {
                if key_state == KeyState::Pressed {
                    if let Some(action) =
                        Self::check_compositor_binding(modifiers, keysym.modified_sym())
                    {
                        Self::execute_action(state, action);
                        return FilterResult::Intercept(());
                    }
                }
                FilterResult::Forward
            },
        );
    }

    /// Check if the current key combination matches a compositor keybinding
    fn check_compositor_binding(
        modifiers: &ModifiersState,
        keysym: xkbcommon::xkb::Keysym,
    ) -> Option<CompositorAction> {
        use xkbcommon::xkb::Keysym as K;

        if modifiers.logo {
            match keysym {
                K::Return => Some(CompositorAction::SpawnTerminal),
                K::d | K::D => Some(CompositorAction::ToggleLauncher),
                K::q | K::Q => Some(CompositorAction::CloseWindow),
                K::f | K::F => Some(CompositorAction::ToggleFullscreen),
                K::Left => Some(CompositorAction::TileLeft),
                K::Right => Some(CompositorAction::TileRight),
                K::Tab => Some(CompositorAction::CycleFocus),
                _ if modifiers.shift && (keysym == K::e || keysym == K::E) => {
                    Some(CompositorAction::ExitCompositor)
                }
                _ => None,
            }
        } else if modifiers.alt && keysym == xkbcommon::xkb::Keysym::F4 {
            Some(CompositorAction::CloseWindow)
        } else {
            None
        }
    }

    /// Execute a compositor action
    fn execute_action(state: &mut HeyDM, action: CompositorAction) {
        match action {
            CompositorAction::SpawnTerminal => {
                info!("Action: Spawning terminal (alacritty)");
                if let Err(e) = std::process::Command::new("alacritty").spawn() {
                    tracing::warn!("Failed to spawn alacritty: {e}");
                }
            }
            CompositorAction::ToggleLauncher => {
                info!("Action: Toggling application launcher");
                state.launcher.toggle();
            }
            CompositorAction::CloseWindow => {
                info!("Action: Closing focused window");
                state.window_manager.close_focused();
            }
            CompositorAction::ToggleFullscreen => {
                info!("Action: Toggling fullscreen");
                state.window_manager.toggle_fullscreen(&state.output_size);
            }
            CompositorAction::TileLeft => {
                info!("Action: Tiling window left");
                state.window_manager.tile_left(&state.output_size);
            }
            CompositorAction::TileRight => {
                info!("Action: Tiling window right");
                state.window_manager.tile_right(&state.output_size);
            }
            CompositorAction::CycleFocus => {
                info!("Action: Cycling window focus");
                state.window_manager.cycle_focus();
            }
            CompositorAction::ExitCompositor => {
                info!("Action: Exiting compositor");
                state.loop_signal.stop();
            }
        }
    }

    /// Handle relative pointer motion
    fn handle_pointer_motion<B: InputBackend>(state: &mut HeyDM, event: B::PointerMotionEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let delta = (event.delta_x(), event.delta_y());

        let new_pos = state.window_manager.update_cursor_relative(
            delta.0,
            delta.1,
            state.output_size,
        );

        if state.window_manager.handle_pointer_motion(new_pos) {
            return;
        }

        if let Some((surface, surface_pos)) = state.window_manager.surface_under(new_pos) {
            let pointer = state.seat.get_pointer().unwrap();
            pointer.motion(
                state,
                Some((surface.clone(), surface_pos.into())),
                &MotionEvent {
                    location: new_pos.into(),
                    serial,
                    time: event.time_msec(),
                },
            );
        }
    }

    /// Handle absolute pointer motion (from winit backend)
    fn handle_pointer_motion_absolute<B: InputBackend>(
        state: &mut HeyDM,
        event: B::PointerMotionAbsoluteEvent,
    ) {
        let output_size = state.output_size;
        let pos = (
            event.x_transformed(output_size.w),
            event.y_transformed(output_size.h),
        );

        state.window_manager.set_cursor_position(pos.0, pos.1);

        let serial = SERIAL_COUNTER.next_serial();

        if let Some((surface, surface_pos)) =
            state.window_manager.surface_under((pos.0, pos.1))
        {
            let pointer = state.seat.get_pointer().unwrap();
            pointer.motion(
                state,
                Some((surface.clone(), surface_pos.into())),
                &MotionEvent {
                    location: pos.into(),
                    serial,
                    time: event.time_msec(),
                },
            );
        }
    }

    /// Handle pointer button press/release
    fn handle_pointer_button<B: InputBackend>(state: &mut HeyDM, event: B::PointerButtonEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let button = event.button_code();
        let button_state = event.state();

        let cursor_pos = state.window_manager.cursor_position();
        if button_state == ButtonState::Pressed {
            if cursor_pos.1 < 32.0 {
                state.panel.handle_click(cursor_pos.0, cursor_pos.1);
                return;
            }

            if state.launcher.is_visible() {
                if let Some(app) = state.launcher.handle_click(cursor_pos.0, cursor_pos.1, state.output_size.w as u32, state.output_size.h as u32) {
                    info!("Launching application: {}" , app);
                    if let Err(e) = std::process::Command::new(&app).spawn() {
                        tracing::warn!("Failed to launch {app}: {e}");
                    }
                    state.launcher.hide();
                    return;
                }
            }

            state.window_manager.focus_at(cursor_pos);
        }

        let pointer = state.seat.get_pointer().unwrap();
        pointer.button(
            state,
            &ButtonEvent {
                button,
                state: button_state,
                serial,
                time: event.time_msec(),
            },
        );
    }

    /// Handle pointer axis (scroll wheel) events
    fn handle_pointer_axis<B: InputBackend>(state: &mut HeyDM, event: B::PointerAxisEvent) {
        let pointer = state.seat.get_pointer().unwrap();
        let source = event.source();

        let mut frame = AxisFrame::new(event.time_msec()).source(source);

        if let Some(amount) = event.amount(Axis::Horizontal) {
            frame = frame.value(Axis::Horizontal, amount);
        }
        if let Some(amount) = event.amount(Axis::Vertical) {
            frame = frame.value(Axis::Vertical, amount);
        }

        pointer.axis(state, frame);
        pointer.frame(state);
    }
}

/// Compositor actions triggered by keybindings
#[derive(Debug, Clone)]
enum CompositorAction {
    SpawnTerminal,
    ToggleLauncher,
    CloseWindow,
    ToggleFullscreen,
    TileLeft,
    TileRight,
    CycleFocus,
    ExitCompositor,
}
