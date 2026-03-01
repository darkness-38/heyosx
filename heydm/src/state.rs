// =============================================================================
// heyDM — Core Compositor State
// =============================================================================


use std::sync::Arc;
use std::time::Duration;

use calloop::{EventLoop, LoopHandle, LoopSignal};
use smithay::backend::renderer::glow::GlowRenderer;
use smithay::backend::renderer::{Frame, Renderer as SmithayRenderer};
use smithay::backend::winit::{self, WinitEvent};

use smithay::delegate_compositor;
use smithay::delegate_data_device;
use smithay::delegate_output;
use smithay::delegate_seat;
use smithay::delegate_shm;
use smithay::delegate_xdg_shell;

use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::reexports::wayland_server::backend::ClientData;
use smithay::reexports::wayland_server::protocol::wl_buffer;
use smithay::reexports::wayland_server::protocol::wl_seat::WlSeat;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Display, DisplayHandle, Resource};
use smithay::utils::{Clock, Monotonic, Size, Transform};
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::compositor::{
    CompositorClientState, CompositorHandler, CompositorState,
};
use smithay::wayland::output::{OutputHandler, OutputManagerState};
use smithay::wayland::selection::data_device::{
    DataDeviceHandler, DataDeviceState, WaylandDndGrabHandler,
};
use smithay::wayland::selection::SelectionHandler;
use smithay::wayland::shell::xdg::{
    PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::wayland::socket::ListeningSocketSource;

use tracing::{error, info};

use crate::input::InputHandler;
use crate::launcher::AppLauncher;
use crate::panel::StatusPanel;
use crate::window::{WindowElement, WindowManager};

/// Client-specific state tracked by the Wayland display
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: smithay::reexports::wayland_server::backend::ClientId) {}
    fn disconnected(
        &self,
        _client_id: smithay::reexports::wayland_server::backend::ClientId,
        _reason: smithay::reexports::wayland_server::backend::DisconnectReason,
    ) {
    }
}

/// The main compositor state struct.
#[allow(dead_code)]
pub struct HeyDM {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, Self>,
    pub loop_signal: LoopSignal,
    pub clock: Clock<Monotonic>,

    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,
    pub output_manager_state: OutputManagerState,

    pub seat: Seat<Self>,
    pub seat_name: String,

    pub window_manager: WindowManager,
    pub panel: StatusPanel,
    pub launcher: AppLauncher,

    pub output_size: Size<i32, smithay::utils::Physical>,
}

impl HeyDM {
    /// Main entry point: sets up the compositor and runs the event loop.
    pub fn run(use_winit: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut display = Display::<Self>::new()?;
        let display_handle = display.handle();

        let mut event_loop: EventLoop<Self> = EventLoop::try_new()?;
        let loop_handle = event_loop.handle();
        let loop_signal = event_loop.get_signal();
        let clock = Clock::new();

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);

        let seat_name = "seat0".to_string();
        let mut seat = seat_state.new_wl_seat(&display_handle, seat_name.clone());

        seat.add_keyboard(Default::default(), 200, 25)?;
        seat.add_pointer();

        info!("Wayland protocols initialized, seat '{seat_name}' created");

        let panel = StatusPanel::new();
        let launcher = AppLauncher::new();
        let window_manager = WindowManager::new();
        let output_size = Size::from((1920, 1080));

        let mut state = Self {
            display_handle: display_handle.clone(),
            loop_handle: loop_handle.clone(),
            loop_signal,
            clock,
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            data_device_state,
            output_manager_state,
            seat,
            seat_name,
            window_manager,
            panel,
            launcher,
            output_size,
        };

        // Add the Wayland display socket to the event loop
        let listening_socket = ListeningSocketSource::new_auto()?;
        let socket_name = listening_socket.socket_name().to_os_string();
        info!("Wayland socket: {:?}", socket_name);
        
        // Save the original display for nested mode before we potentially overwrite it
        let original_wayland_display = std::env::var("WAYLAND_DISPLAY").ok();

        // ListeningSocketSource implements calloop 0.14 EventSource natively
        loop_handle.insert_source(listening_socket, |client_stream, _, state| {
            if let Err(e) = state
                .display_handle
                .insert_client(client_stream, Arc::new(ClientState::default()))
            {
                tracing::warn!("Failed to insert client: {e}");
            }
        })?;

        // Poll the Wayland display fd for client requests
        // Clone the fd so we don't hold a borrow on `display`
        let poll_fd = display.backend().poll_fd().try_clone_to_owned()?;
        loop_handle.insert_source(
            calloop::generic::Generic::new(poll_fd, calloop::Interest::READ, calloop::Mode::Level),
            |_, _, state| {
                state.display_handle.flush_clients()?;
                Ok(calloop::PostAction::Continue)
            },
        )?;

        if use_winit {
            // Restore original display for winit to connect to parent compositor
            if let Some(display_env) = original_wayland_display {
                std::env::set_var("WAYLAND_DISPLAY", display_env);
            }
            Self::run_winit(&mut event_loop, &mut display, &mut state, socket_name)?;
        } else {
            std::env::set_var("WAYLAND_DISPLAY", &socket_name);
            Self::run_udev(&mut event_loop, &mut display, &mut state)?;
        }

        Ok(())
    }

    /// Run using the winit backend (nested compositor for development/testing)
    fn run_winit(
        event_loop: &mut EventLoop<Self>,
        display: &mut Display<Self>,
        state: &mut Self,
        socket_name: std::ffi::OsString,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing winit backend with Glow (OpenGL) renderer");
        let (mut backend, mut winit_evt) = winit::init::<GlowRenderer>()?;
        
        // Set the variable for any future children we spawn (alacritty, etc.)
        std::env::set_var("WAYLAND_DISPLAY", socket_name);
        
        // winit 0.30: window_size() returns Size<i32, Physical> directly
        let output_size = backend.window_size();
        state.output_size = output_size;

        let output = smithay::output::Output::new(
            "heydm-winit".to_string(),
            smithay::output::PhysicalProperties {
                size: (0, 0).into(),
                subpixel: smithay::output::Subpixel::Unknown,
                make: "heyOS".into(),
                model: "winit".into(),
                serial_number: String::new(),
            },
        );

        let mode = smithay::output::Mode {
            size: state.output_size,
            refresh: 60_000,
        };

        output.change_current_state(
            Some(mode),
            Some(Transform::Flipped180),
            None,
            Some((0, 0).into()),
        );
        output.set_preferred(mode);
        output.create_global::<Self>(&state.display_handle);

        info!(
            "Winit backend started, output size: {}x{}",
            state.output_size.w, state.output_size.h
        );

        let mut running = true;
        while running {
            winit_evt.dispatch_new_events(|event| match event {
                WinitEvent::Resized { size, .. } => {
                    state.output_size = size;
                    let mode = smithay::output::Mode {
                        size,
                        refresh: 60_000,
                    };
                    output.change_current_state(Some(mode), None, None, None);
                }
                WinitEvent::Input(input_event) => {
                    InputHandler::handle_input(state, input_event);
                }
                WinitEvent::Focus(_) => {}
                WinitEvent::Redraw => {}
                WinitEvent::CloseRequested => {
                    info!("Window close requested — shutting down");
                    running = false;
                    state.loop_signal.stop();
                }
            });

            if !running {
                break;
            }

            // Winit backend render path
            {
                let (renderer, mut target) = backend.bind()?;
                let mut frame = renderer
                    .render(&mut target, state.output_size, smithay::utils::Transform::Normal)?;
                
                crate::render::Renderer::render_frame(state, &mut frame, &output, state.output_size)?;
                
                let _ = frame.finish()?;
            }
            backend.submit(None)?;

            display.flush_clients()?;
            event_loop.dispatch(Some(Duration::from_millis(16)), state)?;
        }

        Ok(())
    }

    /// Run using udev/DRM backend (direct hardware — production path)
    fn run_udev(
        _event_loop: &mut EventLoop<Self>,
        _display: &mut Display<Self>,
        _state: &mut Self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        error!("Direct DRM/udev backend is not fully implemented for rendering.");
        error!("Please run heydm via a Wayland compositor like cage (which provides WAYLAND_DISPLAY).");
        std::process::exit(1);
    }
}

// =============================================================================
// Smithay Delegate Implementations
// =============================================================================

impl CompositorHandler for HeyDM {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a smithay::reexports::wayland_server::Client,
    ) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        tracing::debug!("Surface commit: {:?}", surface.id());
        self.window_manager.handle_commit(surface);
    }
}

delegate_compositor!(HeyDM);

impl XdgShellHandler for HeyDM {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        info!("New toplevel window created");
        self.window_manager
            .add_window(WindowElement::new(surface), &self.output_size);

        let window = self.window_manager.windows().last().unwrap();
        window.toplevel().send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        tracing::debug!("New popup surface created");
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        info!("Toplevel window destroyed");
        self.window_manager.remove_window(&surface);
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: smithay::utils::Serial) {}

    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {}
}

delegate_xdg_shell!(HeyDM);

impl ShmHandler for HeyDM {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for HeyDM {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

delegate_shm!(HeyDM);

impl SeatHandler for HeyDM {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }

    fn focus_changed(
        &mut self,
        _seat: &Seat<Self>,
        _focused: Option<&WlSurface>,
    ) {
    }
}

delegate_seat!(HeyDM);

impl DataDeviceHandler for HeyDM {
    fn data_device_state(&mut self) -> &mut DataDeviceState {
        &mut self.data_device_state
    }
}

impl SelectionHandler for HeyDM {
    type SelectionUserData = ();
}

impl WaylandDndGrabHandler for HeyDM {}

delegate_data_device!(HeyDM);

impl OutputHandler for HeyDM {}

delegate_output!(HeyDM);
