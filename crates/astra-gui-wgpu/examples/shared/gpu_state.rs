use astra_gui::{catppuccin::mocha, FullOutput};
use astra_gui_wgpu::Renderer;
use std::sync::Arc;
#[cfg(feature = "profiling")]
use std::time::Instant;
use winit::window::Window;

pub struct GpuState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub renderer: Renderer,
    last_shape_count: usize,

    // Micro-profiling of GPU submission phases (CPU-side timings).
    // These are especially useful to detect swapchain backpressure:
    // stalls often show up in `get_current_texture` on slow frames.
    #[cfg(feature = "profiling")]
    last_acquire_ms: f32,
    #[cfg(feature = "profiling")]
    last_clear_submit_ms: f32,
    #[cfg(feature = "profiling")]
    last_ui_submit_ms: f32,
    #[cfg(feature = "profiling")]
    last_present_ms: f32,
    #[cfg(feature = "profiling")]
    last_total_ms: f32,
}

impl GpuState {
    /// Create GPU state with AutoVsync present mode
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync, // No VSync for benchmarking
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, surface_format);

        Self {
            surface,
            device,
            queue,
            config,
            renderer,
            last_shape_count: 0,

            #[cfg(feature = "profiling")]
            last_acquire_ms: 0.0,
            #[cfg(feature = "profiling")]
            last_clear_submit_ms: 0.0,
            #[cfg(feature = "profiling")]
            last_ui_submit_ms: 0.0,
            #[cfg(feature = "profiling")]
            last_present_ms: 0.0,
            #[cfg(feature = "profiling")]
            last_total_ms: 0.0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn last_shape_count(&self) -> usize {
        self.last_shape_count
    }

    /// CPU-side micro timings of the swapchain/submit phases (milliseconds).
    /// These help pinpoint frame pacing stalls (usually in `get_current_texture`).
    #[cfg(feature = "profiling")]
    pub fn last_phase_timings_ms(&self) -> (f32, f32, f32, f32, f32) {
        (
            self.last_acquire_ms,
            self.last_clear_submit_ms,
            self.last_ui_submit_ms,
            self.last_present_ms,
            self.last_total_ms,
        )
    }

    #[cfg(not(feature = "profiling"))]
    pub fn last_phase_timings_ms(&self) -> (f32, f32, f32, f32, f32) {
        (0.0, 0.0, 0.0, 0.0, 0.0)
    }

    pub fn render(&mut self, ui_output: &FullOutput) -> Result<(), wgpu::SurfaceError> {
        #[cfg(feature = "profiling")]
        let total_start = Instant::now();

        self.last_shape_count = ui_output.shapes.len();

        // Acquire (this is commonly where swapchain backpressure blocks)
        #[cfg(feature = "profiling")]
        let acquire_start = Instant::now();
        let output = self.surface.get_current_texture()?;
        #[cfg(feature = "profiling")]
        {
            self.last_acquire_ms = acquire_start.elapsed().as_secs_f32() * 1000.0;
        }

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Clear pass submit timing
        #[cfg(feature = "profiling")]
        let clear_submit_start = Instant::now();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Clear the screen with mocha::BASE background
        {
            let background_color = mocha::BASE;
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: background_color.r as f64,
                            g: background_color.g as f64,
                            b: background_color.b as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        #[cfg(feature = "profiling")]
        {
            self.last_clear_submit_ms = clear_submit_start.elapsed().as_secs_f32() * 1000.0;
        }

        // UI submit timing (records + submits; may include CPU work such as write_buffer)
        #[cfg(feature = "profiling")]
        let ui_submit_start = Instant::now();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Astra GUI Encoder"),
            });

        self.renderer.render(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            self.config.width as f32,
            self.config.height as f32,
            ui_output,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        #[cfg(feature = "profiling")]
        {
            self.last_ui_submit_ms = ui_submit_start.elapsed().as_secs_f32() * 1000.0;
        }

        // Present timing (usually tiny; if it’s large it can indicate driver pacing)
        #[cfg(feature = "profiling")]
        let present_start = Instant::now();
        output.present();
        #[cfg(feature = "profiling")]
        {
            self.last_present_ms = present_start.elapsed().as_secs_f32() * 1000.0;
            self.last_total_ms = total_start.elapsed().as_secs_f32() * 1000.0;
        }

        Ok(())
    }
}
