use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

use naze_ir::RenderTree;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::manifest::Manifest;
use crate::native_renderer;

struct App {
    render_tree: RenderTree,
    font: fontdue::Font,
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(&self.render_tree.title)
            .with_inner_size(LogicalSize::new(1024.0f64, 768.0));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        self.window = Some(window.clone());
        self.surface = Some(surface);
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }
}

impl App {
    fn render(&mut self) {
        let window = match &self.window {
            Some(w) => w,
            None => return,
        };
        let size = window.inner_size();
        let w = size.width;
        let h = size.height;
        if w == 0 || h == 0 {
            return;
        }

        let layout = naze_layout::compute_layout(&self.render_tree, w as f32, h as f32);

        let mut pixmap = match tiny_skia::Pixmap::new(w, h) {
            Some(p) => p,
            None => return,
        };
        pixmap.fill(tiny_skia::Color::WHITE);
        native_renderer::draw_tree(&mut pixmap, &layout, &self.font);

        let surface = match &mut self.surface {
            Some(s) => s,
            None => return,
        };
        surface
            .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
            .unwrap();
        let mut buffer = surface.buffer_mut().unwrap();

        let pixels = pixmap.data();
        for i in 0..(w * h) as usize {
            let r = pixels[i * 4] as u32;
            let g = pixels[i * 4 + 1] as u32;
            let b = pixels[i * 4 + 2] as u32;
            buffer[i] = (r << 16) | (g << 8) | b;
        }
        buffer.present().unwrap();
    }
}

/// Run the native desktop preview for the current project.
pub fn run(manifest: &Manifest) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new(&manifest.build.output);
    let bin_path = output_dir.join("app_data.bin");

    if !bin_path.exists() {
        return Err(format!(
            "no build output found at {}. Run `nazec build` first.",
            bin_path.display()
        )
        .into());
    }

    let bytes = std::fs::read(&bin_path)?;
    let render_tree = naze_ir::deserialize(&bytes)
        .map_err(|e| format!("failed to deserialize {}: {}", bin_path.display(), e))?;

    let font_bytes = include_bytes!("../fonts/DejaVuSans.ttf");
    let font = fontdue::Font::from_bytes(font_bytes as &[u8], fontdue::FontSettings::default())
        .map_err(|e| format!("failed to load font: {}", e))?;

    eprintln!("running {} (native preview)", render_tree.title);

    let event_loop = EventLoop::new()?;
    let mut app = App {
        render_tree,
        font,
        window: None,
        surface: None,
    };
    event_loop.run_app(&mut app)?;

    Ok(())
}
