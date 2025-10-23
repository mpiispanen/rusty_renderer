//! wgpu backend implementation
//!
//! Provides cross-platform rendering using the wgpu library (WebGPU API).
//! wgpu automatically selects the appropriate backend:
//! - Windows: Direct3D 12
//! - macOS/iOS: Metal
//! - Linux/Android: Vulkan
//! - Web: WebGPU
//!
//! This backend uses WGSL shaders and provides a higher-level API than Vulkan.

use super::*;
use anyhow::Context;
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// wgpu backend implementation
pub struct WgpuBackend {
    // Core wgpu objects
    instance: Option<wgpu::Instance>,
    surface: Option<Arc<wgpu::Surface<'static>>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,

    // Surface configuration
    surface_config: Option<wgpu::SurfaceConfiguration>,

    // Rendering pipeline
    render_pipeline: Option<wgpu::RenderPipeline>,

    // Offscreen rendering target
    offscreen_texture: Option<wgpu::Texture>,
    offscreen_view: Option<wgpu::TextureView>,

    // Window size
    width: u32,
    height: u32,

    // Mode
    headless: bool,

    // Configuration
    enable_validation: bool,

    // Shader resource binding (M8.3)
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    bind_groups: Vec<wgpu::BindGroup>,
    temp_buffers: Vec<wgpu::Buffer>, // Temporary buffers for bind groups (e.g., push constants)
    
    // Default sampler for textures (M10 Phase 4)
    default_sampler: Option<wgpu::Sampler>,

    // Stub trait implementations (will be replaced)
    device_wrapper: WgpuDevice,
    swapchain_wrapper: WgpuSwapchain,
}

impl WgpuBackend {
    /// Create a new wgpu backend
    pub fn new(enable_validation: bool) -> Result<Self> {
        log::info!("Creating wgpu backend (validation: {enable_validation})");

        Ok(Self {
            instance: None,
            surface: None,
            adapter: None,
            device: None,
            queue: None,
            surface_config: None,
            render_pipeline: None,
            offscreen_texture: None,
            offscreen_view: None,
            width: 800,
            height: 600,
            headless: false,
            enable_validation,
            bind_group_layouts: vec![],
            bind_groups: vec![],
            temp_buffers: vec![],
            default_sampler: None,
            device_wrapper: WgpuDevice,
            swapchain_wrapper: WgpuSwapchain::new(),
        })
    }

    /// Load WGSL shader
    fn load_shader(&self) -> &'static str {
        include_str!("../../../shaders/wgsl/forward.wgsl")
    }

    /// Create render pipeline
    fn create_render_pipeline(&mut self) -> Result<()> {
        log::info!("Creating wgpu render pipeline");

        let device = self.device.as_ref().context("Device not initialized")?;
        let config = self
            .surface_config
            .as_ref()
            .context("Surface config not set")?;

        // Load shader
        let shader_source = self.load_shader();
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create pipeline layout with bind group layouts
        // Group 0: Camera, lighting, textures, materials
        // Group 1: Transform (emulates push constants)
        let bind_group_layouts_refs: Vec<&wgpu::BindGroupLayout> = 
            self.bind_group_layouts.iter().collect();
        
        log::info!("Creating pipeline layout with {} bind group layouts", bind_group_layouts_refs.len());
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Forward Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts_refs,
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Forward Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer layout matching VertexData
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress, // pos(3) + normal(3) + uv(2) + color(4)
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3, // position
                            1 => Float32x3, // normal
                            2 => Float32x2, // uv
                            3 => Float32x4, // color
                        ],
                    }
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        self.render_pipeline = Some(pipeline);
        log::info!("Render pipeline created successfully");

        Ok(())
    }

    /// Create bind group layouts for uniform buffers (M8.3 MVP + M10 Phase 4)
    ///
    /// Creates bind group layouts for:
    /// - Group 0, Binding 0: Camera uniforms
    /// - Group 0, Binding 1: Lighting uniforms
    /// - Group 0, Binding 2: Diffuse texture
    /// - Group 0, Binding 3: Material uniforms
    /// - Group 0, Binding 4: Texture sampler
    fn create_uniform_bind_group_layouts(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;

        // Group 0: Global uniforms + textures + materials
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global Uniforms + Textures Bind Group Layout"),
            entries: &[
                // Binding 0: Camera uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 1: Lighting uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 2: Diffuse texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Binding 3: Material uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 4: Texture sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        self.bind_group_layouts.push(bind_group_layout);

        // Create bind group layout 1 for transform (emulates push constants)
        let transform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Transform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        self.bind_group_layouts.push(transform_layout);

        log::info!("Bind group layouts created (groups 0 and 1)");
        Ok(())
    }

    /// Create default texture sampler (M10 Phase 4)
    fn create_default_sampler(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("default_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        
        self.default_sampler = Some(sampler);
        log::info!("Created default texture sampler");
        Ok(())
    }
}

impl GraphicsBackend for WgpuBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Wgpu
    }

    fn initialize(&mut self, window: &winit::window::Window) -> Result<()> {
        log::info!("Initializing wgpu backend");

        // Get window size
        let size = window.inner_size();
        self.width = size.width;
        self.height = size.height;

        // Create instance with validation if requested
        log::info!("Creating wgpu instance");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: if self.enable_validation {
                wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::DEBUG
            } else {
                wgpu::InstanceFlags::empty()
            },
            ..Default::default()
        });

        if self.enable_validation {
            log::info!("wgpu validation and debug enabled");
        }

        // Create surface
        // Safety: The surface must not outlive the window. We ensure this by
        // dropping the surface in cleanup() before the window is dropped.
        log::info!("Creating surface");
        let surface = Arc::new(unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window)?)?
        });
        // Request adapter
        log::info!("Requesting adapter");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .context("Failed to find appropriate adapter")?;

        log::info!("Adapter: {:?}", adapter.get_info());

        // Request device and queue
        log::info!("Requesting device and queue");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Primary Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))?;

        log::info!("Device and queue created");

        // Get surface capabilities and configure
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        log::info!("Surface format: {surface_format:?}");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.width,
            height: self.height,
            present_mode: wgpu::PresentMode::Fifo, // VSync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);
        log::info!("Surface configured: {}x{}", self.width, self.height);

        // Store everything
        self.instance = Some(instance);
        self.surface = Some(surface);
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface_config = Some(config);

        // Create default sampler
        self.create_default_sampler()?;

        // Create bind group layouts
        self.create_uniform_bind_group_layouts()?;

        // Create render pipeline
        self.create_render_pipeline()?;

        log::info!("wgpu backend initialized successfully");
        Ok(())
    }

    fn begin_frame(&mut self) -> Result<()> {
        // wgpu handles frame synchronization internally
        // No explicit begin_frame needed
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;
        let queue = self.queue.as_ref().context("Queue not initialized")?;
        let pipeline = self
            .render_pipeline
            .as_ref()
            .context("Pipeline not initialized")?;

        // Get render target (surface or offscreen)
        let (view, output) = if self.headless {
            // Headless mode: render to offscreen texture
            let view = self
                .offscreen_view
                .as_ref()
                .context("Offscreen view not initialized")?;
            (view, None)
        } else {
            // Window mode: render to surface
            let surface = self.surface.as_ref().context("Surface not initialized")?;
            let output = surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            // Store view in a temporary to ensure it lives long enough
            self.offscreen_view = Some(view);
            let view_ref = self.offscreen_view.as_ref().unwrap();
            (view_ref, Some(output))
        };

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.draw(0..3, 0..1); // Draw 3 vertices, 1 instance
        }

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));

        // Present if not headless
        if let Some(output) = output {
            output.present();
        }

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Ok(());
        }

        if self.width == width && self.height == height {
            return Ok(());
        }

        log::info!(
            "Resizing wgpu surface: {}x{} -> {}x{}",
            self.width,
            self.height,
            width,
            height
        );

        self.width = width;
        self.height = height;

        // Reconfigure surface
        if let (Some(surface), Some(device), Some(config)) =
            (&self.surface, &self.device, &mut self.surface_config)
        {
            config.width = width;
            config.height = height;
            surface.configure(device, config);
        }

        Ok(())
    }

    fn initialize_headless(&mut self, width: u32, height: u32) -> Result<()> {
        log::info!("Initializing wgpu backend in headless mode: {width}x{height}");

        self.width = width;
        self.height = height;
        self.headless = true;

        // Create instance
        log::info!("Creating wgpu instance");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: if self.enable_validation {
                wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::DEBUG
            } else {
                wgpu::InstanceFlags::empty()
            },
            ..Default::default()
        });

        // Request adapter (no surface needed)
        log::info!("Requesting adapter for headless mode");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None, // Headless!
            force_fallback_adapter: false,
        }))
        .context("Failed to find appropriate adapter")?;

        log::info!("Adapter: {:?}", adapter.get_info());

        // Request device and queue
        log::info!("Requesting device and queue");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Primary Device (Headless)"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))?;

        log::info!("Device and queue created");

        // Create offscreen render target
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Render Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        log::info!("Offscreen render target created: {width}x{height}");

        // Store everything
        self.instance = Some(instance);
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.offscreen_texture = Some(texture);
        self.offscreen_view = Some(view);

        // Create surface config (for pipeline compatibility)
        self.surface_config = Some(wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: self.width,
            height: self.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        });

        // Create default sampler
        self.create_default_sampler()?;

        // Create bind group layouts
        self.create_uniform_bind_group_layouts()?;

        // Create render pipeline
        self.create_render_pipeline()?;

        log::info!("wgpu backend initialized successfully in headless mode");
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<(u32, u32, Vec<u8>)> {
        let device = self.device.as_ref().context("Device not initialized")?;
        let queue = self.queue.as_ref().context("Queue not initialized")?;
        let texture = self
            .offscreen_texture
            .as_ref()
            .context("Offscreen texture not available (not in headless mode?)")?;

        let width = self.width;
        let height = self.height;

        // Calculate aligned bytes per row
        // wgpu requires COPY_BYTES_PER_ROW_ALIGNMENT (256 bytes)
        const ALIGNMENT: u32 = 256;
        let bytes_per_row_unaligned = 4 * width; // RGBA8
        let bytes_per_row = bytes_per_row_unaligned.div_ceil(ALIGNMENT) * ALIGNMENT;

        // Create buffer to copy texture data to
        let buffer_size = (bytes_per_row * height) as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Frame Capture Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create command encoder for copy
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frame Capture Encoder"),
        });

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .context("Failed to receive buffer mapping result")??;

        let data = buffer_slice.get_mapped_range();

        // If padded, we need to remove padding
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        if bytes_per_row != bytes_per_row_unaligned {
            // Remove padding from each row
            for y in 0..height {
                let start = (y * bytes_per_row) as usize;
                let end = start + bytes_per_row_unaligned as usize;
                pixels.extend_from_slice(&data[start..end]);
            }
        } else {
            // No padding, copy directly
            pixels.extend_from_slice(&data);
        }

        drop(data);
        buffer.unmap();

        log::info!("Frame captured: {width}x{height}, {} bytes", pixels.len());

        Ok((width, height, pixels))
    }

    fn wait_idle(&mut self) -> Result<()> {
        // wgpu handles synchronization automatically
        Ok(())
    }

    fn cleanup(&mut self) {
        log::info!("Cleaning up wgpu backend");

        // Drop in reverse order of creation
        self.render_pipeline = None;
        self.surface_config = None;
        self.queue = None;
        self.device = None;
        self.adapter = None;
        self.surface = None;
        self.instance = None;

        log::info!("wgpu backend cleaned up");
    }

    fn device(&self) -> &dyn Device {
        &self.device_wrapper
    }

    fn swapchain(&self) -> &dyn Swapchain {
        &self.swapchain_wrapper
    }

    fn execute_graph(
        &mut self,
        graph: &crate::render_graph::graph::RenderGraph,
        compiled: &crate::render_graph::graph::CompiledGraph,
    ) -> Result<()> {
        // Get raw pointer to backend early to avoid borrow checker issues
        let backend_ptr = self as *mut WgpuBackend;
        
        let device = self
            .device
            .as_ref()
            .context("Device not initialized for graph execution")?;
        let queue = self
            .queue
            .as_ref()
            .context("Queue not initialized for graph execution")?;

        log::debug!(
            "Executing render graph with {} passes, {} barriers",
            compiled.execution_order.len(),
            compiled.barriers.len()
        );

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Graph Encoder"),
        });

        // Get the render target (surface texture or offscreen)
        let surface_texture = if !self.headless {
            // Get surface texture
            let surface = self.surface.as_ref().context("Surface not initialized")?;
            Some(
                surface
                    .get_current_texture()
                    .context("Failed to get current surface texture")?,
            )
        } else {
            None
        };

        // Create a view from surface texture if windowed
        let surface_view = surface_texture.as_ref().map(|texture| {
            texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default())
        });

        // Get the view to render to
        let view = if let Some(ref view) = surface_view {
            view
        } else {
            self.offscreen_view
                .as_ref()
                .context("Offscreen view not initialized")?
        };

        // Clear previous frame's bind groups and temp buffers
        self.bind_groups.clear();
        self.temp_buffers.clear();

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Graph Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Bind pipeline
            if let Some(pipeline) = &self.render_pipeline {
                render_pass.set_pipeline(pipeline);
            }

            // Create execution context once for all passes
            let mut context = WgpuPassContext::new(&mut render_pass, backend_ptr);

            // Execute passes in order (M9)
            for pass_id in &compiled.execution_order {
                log::debug!("Executing pass: {pass_id:?}");

                // wgpu handles barriers automatically through resource state tracking
                // We don't need to insert explicit barriers like Vulkan

                // Execute pass callback (M9)
                if let Some(pass) = graph.get_pass(*pass_id) {
                    if let Some(callback) = &pass.callback {
                        // Execute the pass with the same context
                        callback.execute(&mut context);
                    }
                }
            }
            
            // Context (with bind_groups) stays alive until here!
        }

        // Submit commands
        queue.submit(Some(encoder.finish()));

        // Present if not headless
        if let Some(texture) = surface_texture {
            texture.present();
        }

        log::debug!("Render graph execution complete");
        Ok(())
    }

    // Resource Management (M8.1)

    fn create_buffer(&mut self, desc: &super::BufferDescriptor) -> Result<Box<dyn super::Buffer>> {
        log::debug!(
            "Creating wgpu buffer: {} bytes, usage: {:?}",
            desc.size,
            desc.usage
        );

        let device = self.device.as_ref().context("Device not initialized")?;

        // Convert usage flags
        let mut wgpu_usage = wgpu::BufferUsages::empty();
        if desc.usage.vertex {
            wgpu_usage |= wgpu::BufferUsages::VERTEX;
        }
        if desc.usage.index {
            wgpu_usage |= wgpu::BufferUsages::INDEX;
        }
        if desc.usage.uniform {
            wgpu_usage |= wgpu::BufferUsages::UNIFORM;
        }
        if desc.usage.transfer_src {
            wgpu_usage |= wgpu::BufferUsages::COPY_SRC;
        }
        if desc.usage.transfer_dst {
            wgpu_usage |= wgpu::BufferUsages::COPY_DST;
        }

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label.as_deref(),
            size: desc.size,
            usage: wgpu_usage,
            mapped_at_creation: false,
        });

        Ok(Box::new(WgpuBuffer {
            buffer,
            size: desc.size,
            usage: desc.usage,
            memory_location: desc.memory_location,
        }))
    }

    fn upload_to_buffer(
        &mut self,
        buffer: &dyn super::Buffer,
        data: &[u8],
        offset: u64,
    ) -> Result<()> {
        let queue = self.queue.as_ref().context("Queue not initialized")?;

        let wgpu_buffer = buffer
            .as_any()
            .downcast_ref::<WgpuBuffer>()
            .context("Buffer is not a WgpuBuffer")?;

        // Check bounds
        if offset + data.len() as u64 > buffer.size() {
            anyhow::bail!(
                "Upload out of bounds: offset {} + size {} > buffer size {}",
                offset,
                data.len(),
                buffer.size()
            );
        }

        queue.write_buffer(&wgpu_buffer.buffer, offset, data);
        Ok(())
    }

    fn create_texture(
        &mut self,
        desc: &super::TextureDescriptor,
    ) -> Result<Box<dyn super::Texture>> {
        let device = self.device.as_ref().context("Device not initialized")?;

        let wgpu_format = texture_format_to_wgpu(desc.format);

        // Convert usage flags
        let mut wgpu_usage = wgpu::TextureUsages::empty();
        if desc.usage.sampled {
            wgpu_usage |= wgpu::TextureUsages::TEXTURE_BINDING;
        }
        if desc.usage.render_target {
            wgpu_usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }
        if desc.usage.transfer_dst {
            wgpu_usage |= wgpu::TextureUsages::COPY_DST;
        }
        if desc.usage.transfer_src {
            wgpu_usage |= wgpu::TextureUsages::COPY_SRC;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: desc.label.as_deref(),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: desc.mip_levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu_usage,
            view_formats: &[],
        });

        // If initial data is provided, upload it
        if let Some(data) = desc.initial_data {
            let queue = self.queue.as_ref().context("Queue not initialized")?;
            let bytes_per_row = desc.width * desc.format.bytes_per_pixel();

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(desc.height),
                },
                wgpu::Extent3d {
                    width: desc.width,
                    height: desc.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        // Create texture view for binding
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        log::debug!(
            "Created wgpu texture: {}x{}, format: {:?}, mip_levels: {}",
            desc.width,
            desc.height,
            desc.format,
            desc.mip_levels
        );

        Ok(Box::new(WgpuTexture {
            texture,
            view,
            width: desc.width,
            height: desc.height,
            format: desc.format,
            usage: desc.usage,
            mip_levels: desc.mip_levels,
        }))
    }

    fn upload_to_texture(
        &mut self,
        texture: &dyn super::Texture,
        data: &[u8],
        mip_level: u32,
    ) -> Result<()> {
        let queue = self.queue.as_ref().context("Queue not initialized")?;

        // Downcast to WgpuTexture
        let wgpu_texture = texture
            .as_any()
            .downcast_ref::<WgpuTexture>()
            .context("Expected WgpuTexture")?;

        let bytes_per_row = wgpu_texture.width * wgpu_texture.format.bytes_per_pixel();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &wgpu_texture.texture,
                mip_level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(wgpu_texture.height),
            },
            wgpu::Extent3d {
                width: wgpu_texture.width,
                height: wgpu_texture.height,
                depth_or_array_layers: 1,
            },
        );

        log::debug!("Uploaded data to wgpu texture at mip level {mip_level}");

        Ok(())
    }

    fn create_sampler(
        &mut self,
        desc: &super::SamplerDescriptor,
    ) -> Result<Box<dyn super::Sampler>> {
        let device = self.device.as_ref().context("Device not initialized")?;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: desc.label.as_deref(),
            address_mode_u: address_mode_to_wgpu(desc.address_mode_u),
            address_mode_v: address_mode_to_wgpu(desc.address_mode_v),
            address_mode_w: address_mode_to_wgpu(desc.address_mode_w),
            mag_filter: filter_mode_to_wgpu(desc.mag_filter),
            min_filter: filter_mode_to_wgpu(desc.min_filter),
            mipmap_filter: filter_mode_to_wgpu(desc.mipmap_filter),
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        log::debug!(
            "Created wgpu sampler: mag={:?}, min={:?}",
            desc.mag_filter,
            desc.min_filter
        );

        Ok(Box::new(WgpuSampler { sampler }))
    }

    // Shader Resource Binding (M8.3)

    fn create_bind_group_layout(&mut self, layout: &BindGroupLayout) -> Result<usize> {
        let device = self.device.as_ref().context("Device not initialized")?;

        // Convert our BindGroupLayout to wgpu::BindGroupLayout
        let mut entries = Vec::new();

        for binding in layout.bindings() {
            let (binding_index, ty, visibility) = match binding {
                ShaderBinding::UniformBuffer { binding, stage, .. } => {
                    let vis = shader_stage_to_wgpu(*stage);
                    let ty = wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    };
                    (*binding, ty, vis)
                }
                ShaderBinding::StorageBuffer {
                    binding,
                    stage,
                    readonly,
                    ..
                } => {
                    let vis = shader_stage_to_wgpu(*stage);
                    let ty = wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: *readonly,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    };
                    (*binding, ty, vis)
                }
                ShaderBinding::Texture { binding, stage, .. } => {
                    let vis = shader_stage_to_wgpu(*stage);
                    let ty = wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    };
                    (*binding, ty, vis)
                }
                ShaderBinding::Sampler { binding, stage } => {
                    let vis = shader_stage_to_wgpu(*stage);
                    let ty = wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);
                    (*binding, ty, vis)
                }
            };

            entries.push(wgpu::BindGroupLayoutEntry {
                binding: binding_index,
                visibility,
                ty,
                count: None,
            });
        }

        let wgpu_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &entries,
        });

        self.bind_group_layouts.push(wgpu_layout);
        let handle = self.bind_group_layouts.len() - 1;

        log::debug!("Created wgpu bind group layout with handle {handle}");
        Ok(handle)
    }

    fn create_bind_group(&mut self, layout_handle: usize, bind_group: &BindGroup) -> Result<usize> {
        let device = self.device.as_ref().context("Device not initialized")?;

        let layout = self
            .bind_group_layouts
            .get(layout_handle)
            .context("Invalid bind group layout handle")?;

        let mut entries = Vec::new();

        for (binding, resource) in bind_group.resources() {
            match resource {
                BoundResource::UniformBuffer(buffer) | BoundResource::StorageBuffer(buffer) => {
                    // Downcast to WgpuBuffer
                    let wgpu_buffer = buffer
                        .as_any()
                        .downcast_ref::<WgpuBuffer>()
                        .context("Expected WgpuBuffer")?;

                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &wgpu_buffer.buffer,
                            offset: 0,
                            size: None,
                        }),
                    });
                }
                BoundResource::Texture(texture) => {
                    // Downcast to WgpuTexture
                    let wgpu_texture = texture
                        .as_any()
                        .downcast_ref::<WgpuTexture>()
                        .context("Expected WgpuTexture")?;

                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::TextureView(wgpu_texture.view()),
                    });
                }
                BoundResource::Sampler(sampler) => {
                    // Downcast to WgpuSampler
                    let wgpu_sampler = sampler
                        .as_any()
                        .downcast_ref::<WgpuSampler>()
                        .context("Expected WgpuSampler")?;

                    entries.push(wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::Sampler(&wgpu_sampler.sampler),
                    });
                }
            }
        }

        // Create bind group with actual entries
        let wgpu_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout,
            entries: &entries,
        });

        self.bind_groups.push(wgpu_bind_group);
        let handle = self.bind_groups.len() - 1;

        log::debug!("Created wgpu bind group with handle {handle}");
        Ok(handle)
    }
}

/// Convert ShaderStage to wgpu::ShaderStages
fn shader_stage_to_wgpu(stage: ShaderStage) -> wgpu::ShaderStages {
    let mut stages = wgpu::ShaderStages::empty();

    if stage.contains(ShaderStage::VERTEX) {
        stages |= wgpu::ShaderStages::VERTEX;
    }
    if stage.contains(ShaderStage::FRAGMENT) {
        stages |= wgpu::ShaderStages::FRAGMENT;
    }
    if stage.contains(ShaderStage::COMPUTE) {
        stages |= wgpu::ShaderStages::COMPUTE;
    }

    stages
}

/// Convert TextureFormat to wgpu::TextureFormat
fn texture_format_to_wgpu(format: super::TextureFormat) -> wgpu::TextureFormat {
    match format {
        super::TextureFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        super::TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        super::TextureFormat::Bgra8Srgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        super::TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        super::TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
        super::TextureFormat::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
    }
}

/// Convert FilterMode to wgpu::FilterMode
fn filter_mode_to_wgpu(filter: super::FilterMode) -> wgpu::FilterMode {
    match filter {
        super::FilterMode::Nearest => wgpu::FilterMode::Nearest,
        super::FilterMode::Linear => wgpu::FilterMode::Linear,
    }
}

/// Convert AddressMode to wgpu::AddressMode
fn address_mode_to_wgpu(mode: super::AddressMode) -> wgpu::AddressMode {
    match mode {
        super::AddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        super::AddressMode::Repeat => wgpu::AddressMode::Repeat,
        super::AddressMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    }
}

/// Stub Device implementation
pub struct WgpuDevice;

impl Device for WgpuDevice {
    fn name(&self) -> &str {
        "wgpu-stub-device"
    }

    fn supports_feature(&self, _feature: &str) -> bool {
        // Stub: will be implemented in M4
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Stub CommandBuffer implementation
pub struct WgpuCommandBuffer;

impl CommandBuffer for WgpuCommandBuffer {
    fn begin(&mut self) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn end(&mut self) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn clear_color(&mut self, _r: f32, _g: f32, _b: f32, _a: f32) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn bind_pipeline(&mut self, _pipeline: &dyn Pipeline) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn draw(&mut self, _vertex_count: u32, _instance_count: u32) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }
}

/// Stub Pipeline implementation
pub struct WgpuPipeline {
    name: String,
}

impl WgpuPipeline {
    /// Create a new stub pipeline
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Pipeline for WgpuPipeline {
    fn name(&self) -> &str {
        &self.name
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Stub Resource implementation
pub struct WgpuResource {
    size: usize,
    resource_type: String,
}

impl WgpuResource {
    /// Create a new stub resource
    pub fn new(size: usize, resource_type: String) -> Self {
        Self {
            size,
            resource_type,
        }
    }
}

impl Resource for WgpuResource {
    fn size(&self) -> usize {
        self.size
    }

    fn resource_type(&self) -> &str {
        &self.resource_type
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Stub Swapchain implementation
pub struct WgpuSwapchain {
    width: u32,
    height: u32,
    current_frame: usize,
}

impl Default for WgpuSwapchain {
    fn default() -> Self {
        Self::new()
    }
}

impl WgpuSwapchain {
    /// Create a new stub swapchain
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            current_frame: 0,
        }
    }
}

impl Swapchain for WgpuSwapchain {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn current_frame(&self) -> usize {
        self.current_frame
    }

    fn acquire_next_image(&mut self) -> Result<()> {
        // Stub: will be implemented in M4
        self.current_frame = (self.current_frame + 1) % 3;
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn recreate(&mut self, width: u32, height: u32) -> Result<()> {
        // Stub: will be implemented in M4
        self.width = width;
        self.height = height;
        Ok(())
    }
}

// WgpuBuffer implementation
struct WgpuBuffer {
    buffer: wgpu::Buffer,
    size: u64,
    usage: super::BufferUsage,
    memory_location: super::MemoryLocation,
}

impl super::Buffer for WgpuBuffer {
    fn size(&self) -> u64 {
        self.size
    }

    fn usage(&self) -> super::BufferUsage {
        self.usage
    }

    fn memory_location(&self) -> super::MemoryLocation {
        self.memory_location
    }

    fn map(&mut self) -> Result<&mut [u8]> {
        anyhow::bail!("wgpu buffer mapping not yet implemented")
    }

    fn unmap(&mut self) {
        // No-op for now
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// WgpuTexture implementation
struct WgpuTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    format: super::TextureFormat,
    usage: super::TextureUsage,
    mip_levels: u32,
}

impl WgpuTexture {
    /// Get the texture view for binding
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

impl super::Texture for WgpuTexture {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn format(&self) -> super::TextureFormat {
        self.format
    }

    fn usage(&self) -> super::TextureUsage {
        self.usage
    }

    fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// WgpuSampler implementation
struct WgpuSampler {
    sampler: wgpu::Sampler,
}

impl super::Sampler for WgpuSampler {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Pass execution context for wgpu backend (M9)
/// Uses raw pointer to avoid borrow checker issues with render pass in loop
struct WgpuPassContext {
    render_pass: *mut (),
    backend: *mut WgpuBackend,
    // Collect uniform buffer data for bind group 0 (camera + lighting)
    uniform_buffers: Vec<(*const std::ffi::c_void, u32, u64, u64)>, // (ptr, binding, offset, size)
    // Collect texture bindings (M10 Phase 4)
    texture_bindings: Vec<(*const std::ffi::c_void, u32, u32)>, // (texture_ptr, set, binding)
    // Store push constant data (128 bytes)
    push_constant_data: Vec<u8>,
    // Store bind groups to keep them alive for render pass duration (Solution 1)
    bind_groups: Vec<wgpu::BindGroup>,
    // Store temporary buffers (for push constants emulation)
    temp_buffers: Vec<wgpu::Buffer>,
}

// Safety: The render pass pointer is only used during rendering within a single thread
unsafe impl Send for WgpuPassContext {}
unsafe impl Sync for WgpuPassContext {}

impl WgpuPassContext {
    fn new<'a>(render_pass: &mut wgpu::RenderPass<'a>, backend: *mut WgpuBackend) -> Self {
        Self {
            render_pass: render_pass as *mut _ as *mut (),
            backend,
            uniform_buffers: Vec::new(),
            texture_bindings: Vec::new(),
            push_constant_data: vec![0u8; 128],
            bind_groups: Vec::new(),
            temp_buffers: Vec::new(),
        }
    }

    fn render_pass<'a>(&mut self) -> &mut wgpu::RenderPass<'a> {
        unsafe { &mut *(self.render_pass as *mut wgpu::RenderPass<'a>) }
    }

    fn backend(&mut self) -> &mut WgpuBackend {
        unsafe { &mut *self.backend }
    }
    
    /// Set all pending bind groups on the render pass
    fn apply_bind_groups(&mut self) {
        if self.bind_groups.is_empty() {
            return;
        }
        
        let bind_groups_ptr = self.bind_groups.as_ptr();
        let num_bind_groups = self.bind_groups.len();
        
        log::info!("apply_bind_groups: Setting {} bind groups", num_bind_groups);
        
        // Get render pass directly from raw pointer
        let render_pass = unsafe { &mut *(self.render_pass as *mut wgpu::RenderPass) };
        
        // Set each bind group by index
        for i in 0..num_bind_groups {
            let bind_group = unsafe { &*bind_groups_ptr.add(i) };
            log::info!("apply_bind_groups: Setting bind group {} at index {}", i, i);
            render_pass.set_bind_group(i as u32, bind_group, &[]);
        }
        
        log::info!("apply_bind_groups: All bind groups set");
    }
}

impl crate::render_graph::PassExecutionContext for WgpuPassContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn bind_vertex_buffer(
        &mut self,
        binding: u32,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
    ) -> Result<()> {
        log::debug!("WgpuPassContext: Binding vertex buffer at binding {binding}, offset {offset}");

        // Cast the void pointer to WgpuBuffer
        let buffer_ref = unsafe { &*(buffer_ptr as *const WgpuBuffer) };

        // wgpu uses slot index starting from 0
        self.render_pass()
            .set_vertex_buffer(binding, buffer_ref.buffer.slice(offset..));

        log::debug!("WgpuPassContext: Vertex buffer bound successfully");
        Ok(())
    }

    fn bind_index_buffer(
        &mut self,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
        index_type: crate::render_graph::IndexType,
    ) -> Result<()> {
        // Cast the void pointer to WgpuBuffer
        let buffer_ref = unsafe { &*(buffer_ptr as *const WgpuBuffer) };

        let wgpu_index_format = match index_type {
            crate::render_graph::IndexType::U16 => wgpu::IndexFormat::Uint16,
            crate::render_graph::IndexType::U32 => wgpu::IndexFormat::Uint32,
        };

        self.render_pass()
            .set_index_buffer(buffer_ref.buffer.slice(offset..), wgpu_index_format);
        Ok(())
    }

    fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        // Create and store bind groups (but don't set them yet)
        let backend = unsafe { &mut *self.backend };  // Mutable to store bind groups
        
        log::info!("WgpuPassContext::draw - uniform_buffers: {}, texture_bindings: {}, push_constant_data: {}", 
                   self.uniform_buffers.len(), self.texture_bindings.len(), self.push_constant_data.len());
        
        // Need at least camera and lighting uniforms (bindings 0 and 1)
        if self.uniform_buffers.len() >= 2 {
            log::info!("WgpuPassContext::draw - Creating bind groups");
            if let Some(device) = &backend.device {
                log::info!("WgpuPassContext::draw - Device available");
                if !backend.bind_group_layouts.is_empty() {
                    log::info!("WgpuPassContext::draw - Bind group layouts available: {}", backend.bind_group_layouts.len());
                    let layout = &backend.bind_group_layouts[0];
                    
                    // Sort uniform buffers by binding number
                    let mut uniforms = self.uniform_buffers.clone();
                    uniforms.sort_by_key(|(_, binding, _, _)| *binding);
                    
                    let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
                    
                    // Add uniform buffer entries
                    for (ptr, binding, offset, size) in &uniforms {
                        let buffer_ref = unsafe { &*(*ptr as *const WgpuBuffer) };
                        log::info!("Adding uniform buffer at binding {}", binding);
                        entries.push(wgpu::BindGroupEntry {
                            binding: *binding,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &buffer_ref.buffer,
                                offset: *offset,
                                size: std::num::NonZeroU64::new(*size),
                            }),
                        });
                    }
                    
                    // Add texture entries (binding 2 for texture view, binding 4 for sampler)
                    for (tex_ptr, _set, binding) in &self.texture_bindings {
                        let texture = unsafe { &*(*tex_ptr as *const WgpuTexture) };
                        
                        log::info!("Adding texture at binding {}", binding);
                        // Add texture view at its binding (should be 2)
                        entries.push(wgpu::BindGroupEntry {
                            binding: *binding,
                            resource: wgpu::BindingResource::TextureView(texture.view()),
                        });
                    }
                    
                    // Add sampler if we have texture bindings (binding 4)
                    if !self.texture_bindings.is_empty() {
                        if let Some(ref sampler) = backend.default_sampler {
                            log::info!("Adding sampler at binding 4");
                            entries.push(wgpu::BindGroupEntry {
                                binding: 4, // Sampler at binding 4
                                resource: wgpu::BindingResource::Sampler(sampler),
                            });
                        }
                    }
                    
                    log::info!("WgpuPassContext::draw - Creating bind group 0 with {} entries", entries.len());
                    let bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Set 0: Uniforms + Textures"),
                        layout,
                        entries: &entries,
                    });
                    
                    // SOLUTION 1: Store bind group in BACKEND to keep it alive past render pass!
                    backend.bind_groups.push(bind_group_0);
                    
                    // Create bind group 1 for transform (emulating push constants)
                    if !self.push_constant_data.is_empty() && self.push_constant_data.len() == 128 {
                        // Create a temporary buffer for push constant data
                        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Transform Buffer (Push Constants)"),
                            contents: &self.push_constant_data,
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                        
                        // Get bind group layout 1 from backend
                        if backend.bind_group_layouts.len() > 1 {
                            let transform_layout = &backend.bind_group_layouts[1];
                            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Set 1: Transform"),
                                layout: transform_layout,
                                entries: &[wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: transform_buffer.as_entire_binding(),
                                }],
                            });
                            
                            // SOLUTION 1: Store both in BACKEND to keep them alive!
                            backend.temp_buffers.push(transform_buffer);
                            backend.bind_groups.push(bind_group_1);
                        }
                    }
                }
            }
        }
        
        // Now apply all bind groups and draw using the SAME render pass reference
        log::info!("WgpuPassContext::draw - Applying {} bind groups", backend.bind_groups.len());
        log::info!("WgpuPassContext::draw - render_pass pointer: {:p}", self.render_pass);
        
        // Get render pass ONCE and use it for everything
        let render_pass = unsafe { &mut *(self.render_pass as *mut wgpu::RenderPass) };
        log::info!("WgpuPassContext::draw - render_pass reference: {:p}", render_pass as *mut _);
        
        // Apply bind groups to THIS reference
        if !backend.bind_groups.is_empty() {
            let bind_groups_ptr = backend.bind_groups.as_ptr();
            let num_bind_groups = backend.bind_groups.len();
            
            log::info!("apply_bind_groups: Setting {} bind groups", num_bind_groups);
            
            for i in 0..num_bind_groups {
                let bind_group = unsafe { &*bind_groups_ptr.add(i) };
                log::info!("apply_bind_groups: Setting bind group {} at index {} (ptr: {:p})", i, i, bind_group);
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }
            
            log::info!("apply_bind_groups: All bind groups set");
        }
        
        log::info!("WgpuPassContext::draw - Drawing {} vertices", vertex_count);
        render_pass.draw(
            first_vertex..(first_vertex + vertex_count),
            first_instance..(first_instance + instance_count),
        );
        Ok(())
    }

    fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        _vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        // Note: wgpu doesn't have a direct vertex_offset parameter in draw_indexed
        // The vertex_offset would be baked into the indices themselves
        self.render_pass().draw_indexed(
            first_index..(first_index + index_count),
            0, // base_vertex - wgpu uses 0 here, vertex offset handled differently
            first_instance..(first_instance + instance_count),
        );
        Ok(())
    }

    fn bind_uniform_buffer(
        &mut self,
        set: u32,
        binding: u32,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
        size: u64,
    ) -> Result<()> {
        log::info!(
            "WgpuPassContext::bind_uniform_buffer - set {set}, binding {binding}, offset {offset}, size {size}"
        );

        // Only support set 0 for now (camera + lighting)
        if set == 0 {
            log::info!("WgpuPassContext: Collecting uniform buffer at set {set}, binding {binding}");
            self.uniform_buffers.push((buffer_ptr, binding, offset, size));
        } else {
            log::warn!("WgpuPassContext: Only set 0 supported, ignoring set {set}");
        }
        Ok(())
    }

    fn push_constants(
        &mut self,
        _stage_flags: u32,
        offset: u32,
        data: &[u8],
    ) -> Result<()> {
        log::debug!("WgpuPassContext: Storing push constants - {} bytes at offset {}", data.len(), offset);
        let start = offset as usize;
        let end = start + data.len();
        self.push_constant_data[start..end].copy_from_slice(data);
        Ok(())
    }

    fn bind_texture(
        &mut self,
        set: u32,
        binding: u32,
        texture_ptr: *const std::ffi::c_void,
    ) -> Result<()> {
        log::debug!("WgpuPassContext: Collecting texture binding at set {}, binding {}", set, binding);
        
        // Only support set 0 for now
        if set == 0 {
            self.texture_bindings.push((texture_ptr, set, binding));
        } else {
            log::warn!("WgpuPassContext: Only set 0 supported, ignoring set {}", set);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgpu_backend_creation() {
        let backend = WgpuBackend::new(false);
        assert!(backend.is_ok(), "Failed to create wgpu backend");
    }

    #[test]
    fn test_wgpu_backend_type() {
        let backend = WgpuBackend::new(false).unwrap();
        assert_eq!(backend.backend_type(), BackendType::Wgpu);
    }

    #[test]
    fn test_wgpu_device_name() {
        let device = WgpuDevice;
        assert_eq!(device.name(), "wgpu-stub-device");
    }

    #[test]
    fn test_wgpu_device_features() {
        let device = WgpuDevice;
        assert!(!device.supports_feature("any_feature"));
    }

    #[test]
    fn test_wgpu_device_as_any() {
        let device = WgpuDevice;
        let any = device.as_any();
        assert!(any.downcast_ref::<WgpuDevice>().is_some());
    }

    #[test]
    fn test_wgpu_command_buffer_methods() {
        let mut cmd = WgpuCommandBuffer;
        assert!(cmd.begin().is_ok());
        assert!(cmd.clear_color(0.0, 0.0, 0.0, 1.0).is_ok());

        let pipeline = WgpuPipeline::new("test".to_string());
        assert!(cmd.bind_pipeline(&pipeline).is_ok());
        assert!(cmd.draw(3, 1).is_ok());
        assert!(cmd.end().is_ok());
    }

    #[test]
    fn test_wgpu_pipeline_creation() {
        let pipeline = WgpuPipeline::new("test_pipeline".to_string());
        assert_eq!(pipeline.name(), "test_pipeline");
    }

    #[test]
    fn test_wgpu_pipeline_as_any() {
        let pipeline = WgpuPipeline::new("test".to_string());
        let any = pipeline.as_any();
        assert!(any.downcast_ref::<WgpuPipeline>().is_some());
    }

    #[test]
    fn test_wgpu_resource_creation() {
        let resource = WgpuResource::new(1024, "buffer".to_string());
        assert_eq!(resource.size(), 1024);
        assert_eq!(resource.resource_type(), "buffer");
    }

    #[test]
    fn test_wgpu_resource_as_any() {
        let resource = WgpuResource::new(1024, "buffer".to_string());
        let any = resource.as_any();
        assert!(any.downcast_ref::<WgpuResource>().is_some());
    }

    #[test]
    fn test_wgpu_swapchain_creation() {
        let swapchain = WgpuSwapchain::new();
        assert_eq!(swapchain.width(), 800);
        assert_eq!(swapchain.height(), 600);
        assert_eq!(swapchain.current_frame(), 0);
    }

    #[test]
    fn test_wgpu_swapchain_acquire_present() {
        let mut swapchain = WgpuSwapchain::new();
        assert_eq!(swapchain.current_frame(), 0);

        assert!(swapchain.acquire_next_image().is_ok());
        assert_eq!(swapchain.current_frame(), 1);

        assert!(swapchain.present().is_ok());
    }

    #[test]
    fn test_wgpu_swapchain_recreate() {
        let mut swapchain = WgpuSwapchain::new();
        assert_eq!(swapchain.width(), 800);
        assert_eq!(swapchain.height(), 600);

        assert!(swapchain.recreate(1920, 1080).is_ok());
        assert_eq!(swapchain.width(), 1920);
        assert_eq!(swapchain.height(), 1080);
    }

    #[test]
    fn test_wgpu_backend_device_access() {
        let backend = WgpuBackend::new(false).unwrap();
        let device = backend.device();
        assert_eq!(device.name(), "wgpu-stub-device");
    }

    #[test]
    fn test_wgpu_backend_swapchain_access() {
        let backend = WgpuBackend::new(false).unwrap();
        let swapchain = backend.swapchain();
        assert_eq!(swapchain.width(), 800);
        assert_eq!(swapchain.height(), 600);
    }

    // Note: test_wgpu_backend_initialize is not included because creating a window
    // in unit tests with winit 0.30 requires an event loop which is not suitable
    // for unit tests. The initialize method is tested in integration tests.

    #[test]
    fn test_wgpu_backend_frame_lifecycle() {
        let mut backend = WgpuBackend::new(false).unwrap();
        assert!(backend.begin_frame().is_ok());
        // Note: end_frame() requires initialization with a window, which is not
        // suitable for unit tests. This is tested in integration tests.
        // assert!(backend.end_frame().is_ok());
    }

    #[test]
    fn test_wgpu_backend_resize() {
        let mut backend = WgpuBackend::new(false).unwrap();
        assert!(backend.resize(1920, 1080).is_ok());
    }

    #[test]
    fn test_wgpu_backend_cleanup() {
        let mut backend = WgpuBackend::new(false).unwrap();
        backend.cleanup(); // Should not panic
    }
}
