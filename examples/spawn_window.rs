//! Helper binary for spawning test windows
//!
//! Usage: cargo run --example spawn_window -- --title "Test Window" [--icon path/to/icon.png]

use std::env;
use std::path::Path;
use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct App {
    window: Option<Window>,
    title: String,
    icon_path: Option<String>,
}

impl App {
    fn new(title: String, icon_path: Option<String>) -> Self {
        Self {
            window: None,
            title,
            icon_path,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attributes = WindowAttributes::default().with_title(&self.title);

        // Load icon if provided
        if let Some(icon_path) = &self.icon_path {
            if let Ok(icon) = load_icon(icon_path) {
                window_attributes = window_attributes.with_window_icon(Some(icon));
            } else {
                eprintln!("Warning: Failed to load icon from {}", icon_path);
            }
        }

        let window = event_loop.create_window(window_attributes).unwrap();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Handle redraw if needed
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn load_icon(path: &str) -> Result<winit::window::Icon, Box<dyn std::error::Error>> {
    let image = image::open(path)?;
    let image = image.to_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Ok(winit::window::Icon::from_rgba(rgba, width, height)?)
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let mut title = "Test Window".to_string();
    let mut icon_path: Option<String> = None;

    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--title" => {
                if i + 1 < args.len() {
                    title = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --title requires a value");
                    std::process::exit(1);
                }
            }
            "--icon" => {
                if i + 1 < args.len() {
                    let path = args[i + 1].clone();
                    if Path::new(&path).exists() {
                        icon_path = Some(path);
                    } else {
                        eprintln!("Warning: Icon file does not exist: {}", path);
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --icon requires a path");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("Usage: {} [OPTIONS]", args[0]);
                println!("Options:");
                println!("  --title <TITLE>    Set window title (default: 'Test Window')");
                println!("  --icon <PATH>      Set window icon from image file");
                println!("  --help, -h         Show this help message");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    println!("Creating window with title: '{}'", title);
    if let Some(ref icon) = icon_path {
        println!("Using icon: {}", icon);
    }

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(title, icon_path);

    // Handle SIGTERM gracefully
    #[cfg(unix)]
    {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            println!("Received SIGTERM, shutting down...");
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }

    event_loop.run_app(&mut app).unwrap();
}
