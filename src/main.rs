use glium::{glutin, Surface};
use imgui::{Context, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use serde::Deserialize;
use std::{fs, path::PathBuf, time::Instant};

/// Represents a position or objective in the game world
#[derive(Deserialize)]
struct Position {
    name: String,
    #[serde(default)]
    hint: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    map: Option<i32>,
    #[serde(default)]
    pos: Option<[f32; 3]>
}

/// Represents a collection of positions/objectives
#[derive(Deserialize)]
struct LocationData {
    name: String,
    objectives: Vec<Position>
}

/// Handles a single objective
fn handle_objective(ui: &Ui, objective: Position) {
    if ui.button(&objective.name) {
        match objective.pos {
            Some(pos) => println!(
                "Selected position: {} at [{:.2}, {:.2}, {:.2}]", 
                objective.name, pos[0], pos[1], pos[2]
            ),
            None => println!("Selected: {} (no position data)", objective.name),
        }
    }
    
    if let Some(hint) = &objective.hint {
        if !hint.is_empty() {
            ui.same_line();
            ui.text(hint);
        }
    }
}

/// Handles a location data and its objectives
fn handle_location_data(ui: &Ui, location_data: LocationData) {
    if let Some(_node_token) = ui.tree_node(&location_data.name) {
        for objective in location_data.objectives {
            handle_objective(ui, objective);
        }
    }
}

/// Recursively handles directory contents and creates the UI tree structure
fn handle_directory(ui: &Ui, path: PathBuf) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");

            if path.is_dir() {
                if let Some(_token) = ui.tree_node(name) {
                    handle_directory(ui, path);
                }
            } else if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(json_content) = fs::read_to_string(&path) {
                    match serde_json::from_str::<LocationData>(&json_content) {
                        Ok(location_data) => handle_location_data(ui, location_data),
                        Err(e) => eprintln!("Error parsing {}: {}", name, e),
                    }
                }
            }
        }
    }
}

/// Creates the teleport window with the directory tree
fn teleport_window(ui: &Ui) {
    ui.window("Teleport")
        .size([400.0, 600.0], imgui::Condition::FirstUseEver)
        .build(|| {
            handle_directory(ui, PathBuf::from("data"));
        });
}

/// Sets up the window and returns the event loop and display
fn setup_window() -> (glutin::event_loop::EventLoop<()>, glium::Display) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("GW2 Teleport")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    (event_loop, display)
}

/// Sets up imgui and returns the imgui context, platform, and renderer
fn setup_imgui(display: &glium::Display) -> (Context, WinitPlatform, Renderer) {
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        display.gl_window().window(),
        HiDpiMode::Default,
    );

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                size_pixels: font_size,
                ..Default::default()
            }),
        },
    ]);

    let renderer = Renderer::init(&mut imgui, display).unwrap();
    (imgui, platform, renderer)
}

fn main() {
    let (event_loop, display) = setup_window();
    let (mut imgui, mut platform, mut renderer) = setup_imgui(&display);
    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::NewEvents(_) => {
                imgui.io_mut().update_delta_time(last_frame.elapsed());
                last_frame = Instant::now();
            }
            glutin::event::Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui.io_mut(), gl_window.window())
                    .unwrap();
                gl_window.window().request_redraw();
            }
            glutin::event::Event::RedrawRequested(_) => {
                let ui = imgui.frame();
                teleport_window(&ui);

                let gl_window = display.gl_window();
                let mut target = display.draw();
                target.clear_color_srgb(0.2, 0.2, 0.2, 1.0);
                
                platform.prepare_render(&ui, gl_window.window());
                let draw_data = imgui.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
            }
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
            }
        }
    });
} 