use wayland_client::{
    protocol::{
        wl_callback, wl_compositor, wl_output, wl_pointer, wl_region, wl_registry, wl_seat,
        wl_surface,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1,
};

pub enum InputState {
    Capturing,   // full_region - захватываем всё
    Passthrough, // empty_region - пропускаем всё
}

pub struct WaylandState {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub seat: Option<wl_seat::WlSeat>,
    pub outputs: Vec<wl_output::WlOutput>,
    pub surface: Option<wl_surface::WlSurface>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub pointer: Option<wl_pointer::WlPointer>,
    pub width: i32,
    pub height: i32,
    pub configured: bool,
    pub input_state: InputState,
    pub drawing: bool,
    pub current_stroke: Vec<(f32, f32, f32)>,
    pub strokes: Vec<Vec<(f32, f32, f32)>>,
    pub start_time: std::time::Instant,
    pub empty_region: Option<wl_region::WlRegion>,
    pub full_region: Option<wl_region::WlRegion>,
    pub last_poll: std::time::Instant,
    pub last_scroll: Option<std::time::Instant>,
}

impl WaylandState {
    pub fn new() -> Self {
        Self {
            compositor: None,
            layer_shell: None,
            seat: None,
            outputs: Vec::new(),
            surface: None,
            layer_surface: None,
            pointer: None,
            width: 0,
            height: 0,
            configured: false,
            input_state: InputState::Capturing,
            drawing: false,
            current_stroke: Vec::new(),
            strokes: Vec::new(),
            start_time: std::time::Instant::now(),
            empty_region: None,
            full_region: None,
            last_poll: std::time::Instant::now(),
            last_scroll: None,
        }
    }

    pub fn set_input_passthrough(&mut self, passthrough: bool) {
        if let Some(surface) = &self.surface {
            let region = if passthrough {
                self.empty_region.as_ref()
            } else {
                self.full_region.as_ref()
            };
            if let Some(r) = region {
                surface.set_input_region(Some(r));
                surface.commit();
            }
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<WaylandState>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor =
                        Some(registry.bind::<wl_compositor::WlCompositor, _, _>(name, 4, qh, ()));
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell = Some(
                        registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                            name,
                            1,
                            qh,
                            (),
                        ),
                    );
                }
                "wl_seat" => {
                    state.seat = Some(registry.bind::<wl_seat::WlSeat, _, _>(name, 5, qh, ()));
                }
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, 3, qh, ());
                    state.outputs.push(output);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            state.width = width as i32;
            state.height = height as i32;
            layer_surface.ack_configure(serial);
            state.configured = true;
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<WaylandState>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(caps),
        } = event
        {
            if caps.contains(wl_seat::Capability::Pointer) && state.pointer.is_none() {
                state.pointer = Some(seat.get_pointer(qh, ()));
            }
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
        match event {
            wl_pointer::Event::Button {
                button,
                state: WEnum::Value(btn_state),
                ..
            } => {
                if button == 0x110 {
                    if btn_state == wl_pointer::ButtonState::Pressed {
                        state.input_state = InputState::Capturing;
                        state.set_input_passthrough(false);
                        state.drawing = true;
                        state.current_stroke.clear();
                    } else {
                        state.drawing = false;
                        if !state.current_stroke.is_empty() {
                            state.strokes.push(state.current_stroke.clone());
                            state.current_stroke.clear();
                        }
                    }
                }
            }
            wl_pointer::Event::Axis { .. } => {
                state.input_state = InputState::Passthrough;
                state.set_input_passthrough(true);
                state.drawing = false;
                state.last_scroll = Some(std::time::Instant::now());
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                if state.drawing {
                    let t = state.start_time.elapsed().as_secs_f32();
                    state
                        .current_stroke
                        .push((surface_x as f32, surface_y as f32, t));
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &wl_callback::WlCallback,
        _: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
    }
}

impl Dispatch<wl_region::WlRegion, ()> for WaylandState {
    fn event(
        _: &mut Self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WaylandState>,
    ) {
    }
}

pub fn setup_wayland(
    state: &mut WaylandState,
    conn: &Connection,
    event_queue: &mut wayland_client::EventQueue<WaylandState>,
) {
    let display = conn.display();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    event_queue.roundtrip(state).unwrap();
    event_queue.roundtrip(state).unwrap();

    let compositor = state.compositor.as_ref().unwrap();
    let layer_shell = state.layer_shell.as_ref().unwrap();

    let surface = compositor.create_surface(&qh, ());
    let output = state.outputs.first().cloned();

    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        output.as_ref(),
        Layer::Overlay,
        "annotate".to_string(),
        &qh,
        (),
    );

    layer_surface.set_anchor(zwlr_layer_surface_v1::Anchor::all());
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);

    let empty_region = compositor.create_region(&qh, ());
    let full_region = compositor.create_region(&qh, ());
    full_region.add(0, 0, i32::MAX, i32::MAX);

    surface.set_input_region(Some(&full_region));
    surface.commit();

    state.surface = Some(surface.clone());
    state.layer_surface = Some(layer_surface);
    state.empty_region = Some(empty_region);
    state.full_region = Some(full_region);

    event_queue.roundtrip(state).unwrap();

    while !state.configured {
        event_queue.blocking_dispatch(state).unwrap();
    }
}

pub fn create_egl_window(
    surface: &wl_surface::WlSurface,
    width: i32,
    height: i32,
) -> wayland_egl::WlEglSurface {
    wayland_egl::WlEglSurface::new(surface.id(), width, height).unwrap()
}
