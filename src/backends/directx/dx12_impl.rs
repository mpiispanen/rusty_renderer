//! DirectX 12 implementation details (Windows-only)
//!
//! This module contains the actual DirectX 12 implementation using the `windows` crate.
//! It's only compiled on Windows platforms.

use super::*;
use crate::render_graph::{PassExecutionContext, PassPreparationContext, ResourceId};
use anyhow::{Context, Result};
use std::io::Write;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::System::Threading::*,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

// HLSL shader source code - embedded forward rendering shader
const HLSL_SHADER_SOURCE: &str = include_str!("../../../shaders/hlsl/forward.hlsl");

/// DirectX 12 backend implementation
pub struct DirectXBackendImpl {
    // Core D3D12 objects
    device: Option<ID3D12Device>,
    command_queue: Option<ID3D12CommandQueue>,
    command_allocator: Option<ID3D12CommandAllocator>,
    command_list: Option<ID3D12GraphicsCommandList>,

    // Separate command objects for uploads/screenshots (to avoid state conflicts)
    upload_command_allocator: Option<ID3D12CommandAllocator>,
    upload_command_list: Option<ID3D12GraphicsCommandList>,

    // Swap chain
    dxgi_factory: Option<IDXGIFactory4>,
    swap_chain: Option<IDXGISwapChain3>,
    frame_index: u32,
    render_targets: Vec<ID3D12Resource>,
    rtv_heap: Option<ID3D12DescriptorHeap>,
    rtv_descriptor_size: u32,

    // Depth stencil
    depth_stencil: Option<ID3D12Resource>,
    dsv_heap: Option<ID3D12DescriptorHeap>,

    // Pipeline (will be created with embedded HLSL bytecode)
    pipeline_state: Option<ID3D12PipelineState>,
    root_signature: Option<ID3D12RootSignature>,

    // Shader resource binding (M8.3)
    cbv_srv_uav_heap: Option<ID3D12DescriptorHeap>, // Descriptor heap for CBV/SRV/UAV
    cbv_srv_uav_descriptor_size: u32,
    descriptor_heap_offset: u32, // Current offset in descriptor heap
    root_signatures: Vec<ID3D12RootSignature>, // Root signatures for bind group layouts
    descriptor_tables: Vec<u32>, // Descriptor table offsets for bind groups

    // Synchronization
    fence: Option<ID3D12Fence>,
    fence_value: u64,
    fence_event: HANDLE,

    // Fence values tracking when each allocator was last used
    main_allocator_fence_value: u64,
    upload_allocator_fence_value: u64,

    // Configuration
    width: u32,
    height: u32,
    frame_count: u32,
    use_warp: bool,
    enable_validation: bool,
    headless: bool,

    // Offscreen rendering (headless mode)
    offscreen_resource: Option<ID3D12Resource>,
    readback_buffer: Option<ID3D12Resource>,

    // Trait implementations
    device_wrapper: DirectXDevice,
    swapchain_wrapper: DirectXSwapchain,

    // Render graph resource management (M10 Phase 1)
    resource_buffers: std::collections::HashMap<crate::render_graph::ResourceId, Box<dyn Buffer>>,
    resource_textures: std::collections::HashMap<crate::render_graph::ResourceId, Box<dyn Texture>>,

    // Pipeline cache for render graph compiled pipelines
    pipeline_cache: std::collections::HashMap<crate::render_graph::PassId, ID3D12PipelineState>,
    root_signature_cache:
        std::collections::HashMap<crate::render_graph::PassId, ID3D12RootSignature>,

    // Per-pass render target tracking (Issue #94)
    pass_rtv_cache: std::collections::HashMap<u64, D3D12_CPU_DESCRIPTOR_HANDLE>,
    pass_dsv_cache: std::collections::HashMap<u64, D3D12_CPU_DESCRIPTOR_HANDLE>,
}

// SAFETY: DirectX 12 objects are thread-safe once created
// HANDLE is just a pointer that we manage carefully
unsafe impl Send for DirectXBackendImpl {}
unsafe impl Sync for DirectXBackendImpl {}

impl DirectXBackendImpl {
    pub fn new(enable_validation: bool) -> Result<Self> {
        let use_warp = std::env::var("RUSTY_RENDERER_USE_WARP").is_ok();

        log::info!(
            "Creating DirectX 12 backend (WARP: {}, validation: {})",
            use_warp,
            enable_validation
        );

        Ok(Self {
            device: None,
            command_queue: None,
            command_allocator: None,
            command_list: None,
            upload_command_allocator: None,
            upload_command_list: None,
            dxgi_factory: None,
            swap_chain: None,
            frame_index: 0,
            render_targets: Vec::new(),
            rtv_heap: None,
            rtv_descriptor_size: 0,
            depth_stencil: None,
            dsv_heap: None,
            pipeline_state: None,
            root_signature: None,
            cbv_srv_uav_heap: None,
            cbv_srv_uav_descriptor_size: 0,
            descriptor_heap_offset: 0,
            root_signatures: Vec::new(),
            descriptor_tables: Vec::new(),
            fence: None,
            fence_value: 0,
            fence_event: HANDLE::default(),
            main_allocator_fence_value: 0,
            upload_allocator_fence_value: 0,
            width: 800,
            height: 600,
            frame_count: 2, // Double buffering
            use_warp,
            enable_validation,
            headless: false,
            offscreen_resource: None,
            readback_buffer: None,
            device_wrapper: DirectXDevice,
            swapchain_wrapper: DirectXSwapchain {
                width: 800,
                height: 600,
                frame_index: 0,
            },
            resource_buffers: std::collections::HashMap::new(),
            resource_textures: std::collections::HashMap::new(),
            pipeline_cache: std::collections::HashMap::new(),
            root_signature_cache: std::collections::HashMap::new(),
            pass_rtv_cache: std::collections::HashMap::new(),
            pass_dsv_cache: std::collections::HashMap::new(),
        })
    }

    pub fn initialize(&mut self, window: &winit::window::Window) -> Result<()> {
        // Debug logging - write to file immediately
        log::info!("Initializing DirectX 12 backend");

        // Set camera backend for proper projection matrices
        crate::camera::set_camera_backend(crate::camera::CameraBackend::DirectX);

        let size = window.inner_size();
        self.width = size.width;
        self.height = size.height;
        // Enable debug layer if requested
        if self.enable_validation {
            unsafe {
                let mut debug: Option<ID3D12Debug> = None;
                if D3D12GetDebugInterface(&mut debug).is_ok() {
                    if let Some(debug) = debug {
                        debug.EnableDebugLayer();
                        log::info!("DirectX 12 debug layer enabled");
                    }
                } else {
                    log::warn!("DirectX 12 debug layer requested but not available");
                }
            }
        }
        // Create DXGI factory
        self.create_factory()?;
        // Create device
        self.create_device()?;
        // Create command queue
        self.create_command_queue()?;
        // Create swap chain
        self.create_swap_chain(window)?;
        // Create render target views
        self.create_render_targets()?;
        // Create depth stencil buffer
        self.create_depth_stencil()?;
        // Create SRV descriptor heap for textures
        self.create_srv_heap()?;
        // Create command objects
        self.create_command_objects()?;
        // Create fence
        self.create_fence()?;
        // Note: Pipeline creation now happens via render graph compilation
        // The old create_pipeline() is no longer needed

        log::info!("DirectX 12 backend initialized successfully");
        Ok(())
    }

    fn create_factory(&mut self) -> Result<()> {
        log::info!("Creating DXGI factory");

        unsafe {
            let flags = if self.enable_validation {
                DXGI_CREATE_FACTORY_DEBUG
            } else {
                DXGI_CREATE_FACTORY_FLAGS(0)
            };

            let factory: IDXGIFactory4 = CreateDXGIFactory2(flags)?;
            self.dxgi_factory = Some(factory);
        }

        Ok(())
    }

    fn create_device(&mut self) -> Result<()> {
        log::info!("Creating D3D12 device");

        unsafe {
            let factory = self.dxgi_factory.as_ref().context("Factory not created")?;

            let adapter: IDXGIAdapter1 = if self.use_warp {
                log::info!("Using WARP software renderer");
                factory.EnumWarpAdapter()?
            } else {
                factory.EnumAdapters1(0)?
            };

            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device)?;

            self.device = device;
            log::info!("D3D12 device created");
        }

        Ok(())
    }

    fn create_command_queue(&mut self) -> Result<()> {
        log::info!("Creating command queue");

        unsafe {
            let device = self.device.as_ref().context("Device not created")?;

            let desc = D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
                Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                NodeMask: 0,
            };

            self.command_queue = Some(device.CreateCommandQueue(&desc)?);
        }

        Ok(())
    }

    fn create_swap_chain(&mut self, window: &winit::window::Window) -> Result<()> {
        log::info!("Creating swap chain");

        unsafe {
            let factory = self.dxgi_factory.as_ref().context("Factory not created")?;
            let command_queue = self
                .command_queue
                .as_ref()
                .context("Command queue not created")?;

            // Get HWND from window
            let window_handle = window.window_handle()?;
            let hwnd = match window_handle.as_raw() {
                RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as *mut std::ffi::c_void),
                _ => anyhow::bail!("Not a Windows window"),
            };

            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: self.width,
                Height: self.height,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                Stereo: BOOL::from(false),
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: self.frame_count,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
                Flags: 0,
            };

            let swap_chain: IDXGISwapChain1 = factory.CreateSwapChainForHwnd(
                command_queue,
                hwnd,
                &swap_chain_desc,
                None,
                None,
            )?;

            let swap_chain: IDXGISwapChain3 = swap_chain.cast()?;
            self.frame_index = swap_chain.GetCurrentBackBufferIndex();
            self.swap_chain = Some(swap_chain);

            log::info!("Swap chain created: {}x{}", self.width, self.height);
        }

        Ok(())
    }

    fn create_render_targets(&mut self) -> Result<()> {
        log::info!("Creating render target views");

        unsafe {
            let device = self.device.as_ref().context("Device not created")?;
            let swap_chain = self.swap_chain.as_ref().context("Swap chain not created")?;

            // Create descriptor heap for RTVs
            let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                NumDescriptors: self.frame_count,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            };

            let rtv_heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&heap_desc)?;
            self.rtv_descriptor_size =
                device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

            // Create RTVs for each frame
            let mut rtv_handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();

            for i in 0..self.frame_count {
                let render_target: ID3D12Resource = swap_chain.GetBuffer(i)?;
                device.CreateRenderTargetView(&render_target, None, rtv_handle);

                rtv_handle.ptr += self.rtv_descriptor_size as usize;
                self.render_targets.push(render_target);
            }

            self.rtv_heap = Some(rtv_heap);
            log::info!("Created {} render target views", self.frame_count);
        }

        Ok(())
    }

    fn create_depth_stencil(&mut self) -> Result<()> {
        log::info!("Creating depth stencil buffer");

        unsafe {
            let device = self.device.as_ref().context("Device not created")?;

            // Create DSV descriptor heap
            let dsv_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
                NumDescriptors: 1,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            };

            let dsv_heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&dsv_heap_desc)?;

            // Create depth stencil resource
            let depth_desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Alignment: 0,
                Width: self.width as u64,
                Height: self.height,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_D32_FLOAT,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                Flags: D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
            };

            let clear_value = D3D12_CLEAR_VALUE {
                Format: DXGI_FORMAT_D32_FLOAT,
                Anonymous: D3D12_CLEAR_VALUE_0 {
                    DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                        Depth: 1.0,
                        Stencil: 0,
                    },
                },
            };

            let heap_properties = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let mut depth_stencil: Option<ID3D12Resource> = None;
            device.CreateCommittedResource(
                &heap_properties,
                D3D12_HEAP_FLAG_NONE,
                &depth_desc,
                D3D12_RESOURCE_STATE_DEPTH_WRITE,
                Some(&clear_value),
                &mut depth_stencil,
            )?;

            let depth_stencil = depth_stencil.context("Failed to create depth stencil resource")?;

            // Create DSV
            let dsv_desc = D3D12_DEPTH_STENCIL_VIEW_DESC {
                Format: DXGI_FORMAT_D32_FLOAT,
                ViewDimension: D3D12_DSV_DIMENSION_TEXTURE2D,
                Flags: D3D12_DSV_FLAG_NONE,
                Anonymous: D3D12_DEPTH_STENCIL_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_DSV { MipSlice: 0 },
                },
            };

            let dsv_handle = dsv_heap.GetCPUDescriptorHandleForHeapStart();
            device.CreateDepthStencilView(&depth_stencil, Some(&dsv_desc), dsv_handle);

            self.depth_stencil = Some(depth_stencil);
            self.dsv_heap = Some(dsv_heap);

            log::info!(
                "Created depth stencil buffer: {}x{}",
                self.width,
                self.height
            );
        }

        Ok(())
    }

    fn create_srv_heap(&mut self) -> Result<()> {
        log::info!("Creating SRV/CBV/UAV descriptor heap for textures");

        unsafe {
            let device = self.device.as_ref().context("Device not created")?;

            // Create descriptor heap for shader resource views (textures)
            // Start with 256 descriptors - should be enough for most scenes
            let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                NumDescriptors: 256,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                NodeMask: 0,
            };

            let heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&heap_desc)?;

            // Get descriptor size for this heap type
            let descriptor_size =
                device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);

            self.cbv_srv_uav_heap = Some(heap);
            self.cbv_srv_uav_descriptor_size = descriptor_size;
            self.descriptor_heap_offset = 0; // Start at beginning

            log::info!(
                "Created SRV descriptor heap with 256 descriptors (size: {} bytes each)",
                descriptor_size
            );
        }

        Ok(())
    }

    fn create_command_objects(&mut self) -> Result<()> {
        log::info!("Creating command allocator and list");

        unsafe {
            let device = self.device.as_ref().context("Device not created")?;

            // Main rendering command allocator and list
            let allocator: ID3D12CommandAllocator =
                device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;

            let command_list: ID3D12GraphicsCommandList =
                device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)?;

            // Close the command list initially
            command_list.Close()?;

            self.command_allocator = Some(allocator);
            self.command_list = Some(command_list);

            // Upload command allocator and list (for resource uploads/screenshots)
            let upload_allocator: ID3D12CommandAllocator =
                device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;

            let upload_command_list: ID3D12GraphicsCommandList = device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &upload_allocator,
                None,
            )?;

            // Close the upload command list initially
            upload_command_list.Close()?;

            self.upload_command_allocator = Some(upload_allocator);
            self.upload_command_list = Some(upload_command_list);
        }

        Ok(())
    }

    fn create_fence(&mut self) -> Result<()> {
        log::info!("Creating fence for synchronization");

        unsafe {
            let device = self.device.as_ref().context("Device not created")?;

            let fence: ID3D12Fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            self.fence = Some(fence);
            self.fence_value = 1;

            // Create event for fence signaling
            self.fence_event = CreateEventA(None, false, false, None)?;
        }

        Ok(())
    }

    fn create_pipeline(&mut self) -> Result<()> {
        log::info!("Creating pipeline state and root signature");

        unsafe {
            let device = self.device.as_ref().context("Device not created")?;

            // Compile shaders at runtime
            let vs_bytecode = self.compile_shader("VSMain", "vs_5_0")?;
            let ps_bytecode = self.compile_shader("PSMain", "ps_5_0")?;

            // Create root signature with CBV parameters, root constants, and texture
            // Root parameter 0: Camera uniforms (CBV b0)
            // Root parameter 1: Lighting uniforms (CBV b1)
            // Root parameter 2: Push constants for model/normal matrices (32 DWORDs b2)
            // Root parameter 3: Material uniforms (CBV b3)
            // Root parameter 4: Texture descriptor table (SRV t0)
            // Static sampler 0: Texture sampler (s0)
            let mut root_parameters = vec![
                D3D12_ROOT_PARAMETER {
                    ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                    Anonymous: D3D12_ROOT_PARAMETER_0 {
                        Descriptor: D3D12_ROOT_DESCRIPTOR {
                            ShaderRegister: 0, // b0 in HLSL
                            RegisterSpace: 0,
                        },
                    },
                    ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
                },
                D3D12_ROOT_PARAMETER {
                    ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                    Anonymous: D3D12_ROOT_PARAMETER_0 {
                        Descriptor: D3D12_ROOT_DESCRIPTOR {
                            ShaderRegister: 1, // b1 in HLSL
                            RegisterSpace: 0,
                        },
                    },
                    ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
                },
                D3D12_ROOT_PARAMETER {
                    ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
                    Anonymous: D3D12_ROOT_PARAMETER_0 {
                        Constants: D3D12_ROOT_CONSTANTS {
                            ShaderRegister: 2, // b2 in HLSL
                            RegisterSpace: 0,
                            Num32BitValues: 48, // 192 bytes / 4 = 48 DWORDs (viewProj + model + normal)
                        },
                    },
                    ShaderVisibility: D3D12_SHADER_VISIBILITY_VERTEX,
                },
                D3D12_ROOT_PARAMETER {
                    ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                    Anonymous: D3D12_ROOT_PARAMETER_0 {
                        Descriptor: D3D12_ROOT_DESCRIPTOR {
                            ShaderRegister: 3, // b3 in HLSL
                            RegisterSpace: 0,
                        },
                    },
                    ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
                },
            ];

            // Add descriptor table for texture (t0)
            let descriptor_range = D3D12_DESCRIPTOR_RANGE {
                RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
                NumDescriptors: 1,
                BaseShaderRegister: 0, // t0
                RegisterSpace: 0,
                OffsetInDescriptorsFromTableStart: 0,
            };
            let mut descriptor_ranges = vec![descriptor_range];

            root_parameters.push(D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                        NumDescriptorRanges: 1,
                        pDescriptorRanges: descriptor_ranges.as_ptr(),
                    },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
            });

            // Static sampler for s0
            let static_sampler = D3D12_STATIC_SAMPLER_DESC {
                Filter: D3D12_FILTER_MIN_MAG_MIP_LINEAR,
                AddressU: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
                AddressV: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
                AddressW: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
                MipLODBias: 0.0,
                MaxAnisotropy: 0,
                ComparisonFunc: D3D12_COMPARISON_FUNC_NEVER,
                BorderColor: D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK,
                MinLOD: 0.0,
                MaxLOD: f32::MAX,
                ShaderRegister: 0, // s0
                RegisterSpace: 0,
                ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
            };
            let static_samplers = vec![static_sampler];

            let root_signature_desc = D3D12_ROOT_SIGNATURE_DESC {
                NumParameters: root_parameters.len() as u32,
                pParameters: root_parameters.as_ptr(),
                NumStaticSamplers: static_samplers.len() as u32,
                pStaticSamplers: static_samplers.as_ptr(),
                Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
            };

            let mut signature_blob: Option<ID3DBlob> = None;
            let mut error_blob: Option<ID3DBlob> = None;

            let result = D3D12SerializeRootSignature(
                &root_signature_desc,
                D3D_ROOT_SIGNATURE_VERSION_1,
                &mut signature_blob,
                Some(&mut error_blob),
            );

            if result.is_err() {
                if let Some(error) = error_blob {
                    let error_msg = std::slice::from_raw_parts(
                        error.GetBufferPointer() as *const u8,
                        error.GetBufferSize(),
                    );
                    let error_str = String::from_utf8_lossy(error_msg);
                    anyhow::bail!("Root signature serialization failed: {}", error_str);
                }
                result?;
            }

            let signature_blob = signature_blob.context("No signature blob created")?;

            let root_signature: ID3D12RootSignature = device.CreateRootSignature(
                0,
                std::slice::from_raw_parts(
                    signature_blob.GetBufferPointer() as *const u8,
                    signature_blob.GetBufferSize(),
                ),
            )?;

            // Define vertex input layout matching our Vertex struct
            // Vertex has: position (vec3), normal (vec3), uv (vec2), color (vec4)
            let input_elements = vec![
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR::from_raw(b"POSITION\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR::from_raw(b"NORMAL\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 12, // After 3 floats (position)
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR::from_raw(b"TEXCOORD\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 24, // After position + normal
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D12_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR::from_raw(b"COLOR\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 32, // After position + normal + uv
                    InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
            ];

            // Create PSO
            let pso_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                pRootSignature: std::mem::ManuallyDrop::new(Some(root_signature.clone())),
                VS: D3D12_SHADER_BYTECODE {
                    pShaderBytecode: vs_bytecode.GetBufferPointer(),
                    BytecodeLength: vs_bytecode.GetBufferSize(),
                },
                PS: D3D12_SHADER_BYTECODE {
                    pShaderBytecode: ps_bytecode.GetBufferPointer(),
                    BytecodeLength: ps_bytecode.GetBufferSize(),
                },
                DS: D3D12_SHADER_BYTECODE::default(),
                HS: D3D12_SHADER_BYTECODE::default(),
                GS: D3D12_SHADER_BYTECODE::default(),
                StreamOutput: D3D12_STREAM_OUTPUT_DESC::default(),
                BlendState: D3D12_BLEND_DESC {
                    AlphaToCoverageEnable: FALSE,
                    IndependentBlendEnable: FALSE,
                    RenderTarget: [
                        D3D12_RENDER_TARGET_BLEND_DESC {
                            BlendEnable: FALSE,
                            LogicOpEnable: FALSE,
                            SrcBlend: D3D12_BLEND_ONE,
                            DestBlend: D3D12_BLEND_ZERO,
                            BlendOp: D3D12_BLEND_OP_ADD,
                            SrcBlendAlpha: D3D12_BLEND_ONE,
                            DestBlendAlpha: D3D12_BLEND_ZERO,
                            BlendOpAlpha: D3D12_BLEND_OP_ADD,
                            LogicOp: D3D12_LOGIC_OP_NOOP,
                            RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
                        },
                        D3D12_RENDER_TARGET_BLEND_DESC::default(),
                        D3D12_RENDER_TARGET_BLEND_DESC::default(),
                        D3D12_RENDER_TARGET_BLEND_DESC::default(),
                        D3D12_RENDER_TARGET_BLEND_DESC::default(),
                        D3D12_RENDER_TARGET_BLEND_DESC::default(),
                        D3D12_RENDER_TARGET_BLEND_DESC::default(),
                        D3D12_RENDER_TARGET_BLEND_DESC::default(),
                    ],
                },
                SampleMask: u32::MAX,
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: D3D12_CULL_MODE_BACK,
                    FrontCounterClockwise: TRUE,
                    DepthBias: 0,
                    DepthBiasClamp: 0.0,
                    SlopeScaledDepthBias: 0.0,
                    DepthClipEnable: TRUE,
                    MultisampleEnable: FALSE,
                    AntialiasedLineEnable: FALSE,
                    ForcedSampleCount: 0,
                    ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
                },
                DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                    DepthEnable: TRUE,
                    DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
                    DepthFunc: D3D12_COMPARISON_FUNC_LESS,
                    StencilEnable: FALSE,
                    StencilReadMask: D3D12_DEFAULT_STENCIL_READ_MASK as u8,
                    StencilWriteMask: D3D12_DEFAULT_STENCIL_WRITE_MASK as u8,
                    FrontFace: D3D12_DEPTH_STENCILOP_DESC::default(),
                    BackFace: D3D12_DEPTH_STENCILOP_DESC::default(),
                },
                InputLayout: D3D12_INPUT_LAYOUT_DESC {
                    pInputElementDescs: input_elements.as_ptr(),
                    NumElements: input_elements.len() as u32,
                },
                IBStripCutValue: D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED,
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: 1,
                RTVFormats: [
                    DXGI_FORMAT_R8G8B8A8_UNORM,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_FORMAT_UNKNOWN,
                ],
                DSVFormat: DXGI_FORMAT_D32_FLOAT,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                NodeMask: 0,
                CachedPSO: D3D12_CACHED_PIPELINE_STATE::default(),
                Flags: D3D12_PIPELINE_STATE_FLAG_NONE,
            };

            let pipeline_state: ID3D12PipelineState =
                device.CreateGraphicsPipelineState(&pso_desc)?;

            self.root_signature = Some(root_signature);
            self.pipeline_state = Some(pipeline_state);

            log::info!("Pipeline created successfully");
        }

        Ok(())
    }

    fn load_shader_source(&self) -> Result<String> {
        // Try to load forward.hlsl (full forward rendering with textures)
        if let Ok(source) = std::fs::read_to_string("shaders/hlsl/forward.hlsl") {
            log::info!("Loaded forward.hlsl shader (with textures)");
            Ok(source)
        } else if let Ok(source) = std::fs::read_to_string("shaders/hlsl/forward_simple.hlsl") {
            log::info!("Loaded forward_simple.hlsl shader (no textures)");
            Ok(source)
        } else {
            log::warn!("Could not load forward shaders, using embedded triangle shader");
            Ok(HLSL_SHADER_SOURCE.to_string())
        }
    }

    fn load_precompiled_shader(&self, entry_point: &str, target: &str) -> Option<Vec<u8>> {
        // Determine shader type from entry point and target
        let shader_type = if entry_point == "VSMain" || target.starts_with("vs_") {
            "vert"
        } else if entry_point == "PSMain" || target.starts_with("ps_") {
            "frag"
        } else {
            return None;
        };

        // Try different shader variants (DXIL files from build.rs)
        let shader_names = vec!["forward_simple", "forward", "triangle"];

        for name in shader_names {
            // Try .dxil files in shaders/ directory (from build.rs)
            let path = format!("shaders/{}.{}.dxil", name, shader_type);
            if let Ok(bytecode) = std::fs::read(&path) {
                log::info!("Loaded pre-compiled DXIL shader: {}", path);
                return Some(bytecode);
            }

            // Also try .cso files for compatibility
            let cso_path = format!(
                "shaders/compiled/{}_{}.cso",
                name,
                if shader_type == "vert" { "vs" } else { "ps" }
            );
            if let Ok(bytecode) = std::fs::read(&cso_path) {
                log::info!("Loaded pre-compiled CSO shader: {}", cso_path);
                return Some(bytecode);
            }
        }

        None
    }

    fn compile_shader(&self, entry_point: &str, target: &str) -> Result<ID3DBlob> {
        unsafe {
            // WORKAROUND: Skip pre-compiled DXIL for now due to vkd3d-proton compatibility issues
            // vkd3d-proton's DXIL->SPIR-V converter fails with our DXIL bytecode (vkd3d result -3)
            // TODO: Investigate proper DXIL format or use alternative shader compilation path
            // For native Windows, we could re-enable DXIL loading here
            log::warn!("Skipping pre-compiled DXIL due to vkd3d-proton compatibility issues");

            // Fall back to runtime compilation
            log::info!(
                "Attempting runtime compilation for {} ({})",
                entry_point,
                target
            );

            // Load shader source
            let shader_source_string = self.load_shader_source()?;

            let entry_cstr = format!("{}\0", entry_point);
            let target_cstr = format!("{}\0", target);

            let shader_source = PCSTR::from_raw(shader_source_string.as_ptr());
            let entry = PCSTR::from_raw(entry_cstr.as_ptr());
            let target_pcstr = PCSTR::from_raw(target_cstr.as_ptr());

            let mut shader_blob: Option<ID3DBlob> = None;
            let mut error_blob: Option<ID3DBlob> = None;

            let result = D3DCompile(
                shader_source.as_ptr() as *const _,
                shader_source_string.len(),
                None,
                None,
                None,
                entry,
                target_pcstr,
                0, // No flags
                0,
                &mut shader_blob,
                Some(&mut error_blob),
            );

            if result.is_err() {
                if let Some(error) = error_blob {
                    let error_msg = std::slice::from_raw_parts(
                        error.GetBufferPointer() as *const u8,
                        error.GetBufferSize(),
                    );
                    let error_str = String::from_utf8_lossy(error_msg);
                    anyhow::bail!(
                        "Shader compilation failed for {} ({}): {}",
                        entry_point,
                        target,
                        error_str
                    );
                }
                result?;
            }
            shader_blob.context(format!(
                "No shader blob created for {} ({})",
                entry_point, target
            ))
        }
    }

    pub fn begin_frame(&mut self) -> Result<()> {
        // Wait for GPU to finish with the command allocator before resetting it
        // This ensures we don't reset while commands are still in flight
        unsafe {
            if let (Some(fence), Some(_command_queue)) = (&self.fence, &self.command_queue) {
                // Wait for the main allocator's last submitted commands to complete
                if self.main_allocator_fence_value > 0 {
                    let completed = fence.GetCompletedValue();
                    if completed < self.main_allocator_fence_value {
                        fence.SetEventOnCompletion(
                            self.main_allocator_fence_value,
                            self.fence_event,
                        )?;
                        WaitForSingleObject(self.fence_event, INFINITE);
                    }
                }
            }

            // Now it's safe to reset the command allocator
            if let Some(allocator) = &self.command_allocator {
                allocator.Reset()?;
            }

            if let (Some(command_list), Some(allocator)) =
                (&self.command_list, &self.command_allocator)
            {
                command_list.Reset(allocator, None)?;
            }
        }

        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<()> {
        unsafe {
            // Close the command list (it was left open after execute_graph)
            // Ignore error if already closed (can happen during resource allocation)
            if let Some(command_list) = &self.command_list {
                let _ = command_list.Close(); // Ignore error
            }

            // Execute commands
            if let (Some(command_queue), Some(command_list)) =
                (&self.command_queue, &self.command_list)
            {
                let command_lists = [Some(command_list.cast()?)];
                command_queue.ExecuteCommandLists(&command_lists);
            }

            // Signal fence for this frame and track it for the main allocator
            if let (Some(fence), Some(command_queue)) = (&self.fence, &self.command_queue) {
                let fence_value = self.fence_value;
                command_queue.Signal(fence, fence_value)?;
                self.main_allocator_fence_value = fence_value;
                self.fence_value += 1;
            }

            // Present (only for windowed mode)
            if !self.headless {
                if let Some(swap_chain) = &self.swap_chain {
                    swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
                }

                // Update frame index for next frame
                if let Some(swap_chain) = &self.swap_chain {
                    self.frame_index = swap_chain.GetCurrentBackBufferIndex();
                }
            }
        }

        Ok(())
    }

    fn wait_for_previous_frame(&mut self) -> Result<()> {
        unsafe {
            if let (Some(fence), Some(command_queue)) = (&self.fence, &self.command_queue) {
                let fence_value = self.fence_value;
                command_queue.Signal(fence, fence_value)?;
                self.fence_value += 1;

                if fence.GetCompletedValue() < fence_value {
                    fence.SetEventOnCompletion(fence_value, self.fence_event)?;
                    WaitForSingleObject(self.fence_event, INFINITE);
                }

                if let Some(swap_chain) = &self.swap_chain {
                    self.frame_index = swap_chain.GetCurrentBackBufferIndex();
                }
            }
        }

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Ok(());
        }

        if self.width == width && self.height == height {
            return Ok(());
        }

        log::info!(
            "Resizing DirectX swap chain: {}x{} -> {}x{}",
            self.width,
            self.height,
            width,
            height
        );

        self.width = width;
        self.height = height;

        // TODO: Recreate swap chain

        Ok(())
    }

    pub fn initialize_headless(&mut self, width: u32, height: u32) -> Result<()> {
        log::info!("Initializing DirectX 12 backend in headless mode: {width}x{height}");

        // Set camera backend for proper projection matrices
        crate::camera::set_camera_backend(crate::camera::CameraBackend::DirectX);

        self.headless = true;
        self.width = width;
        self.height = height;

        unsafe {
            // Create DXGI factory
            self.create_factory()?;

            // Create device
            self.create_device()?;

            // Create command queue
            self.create_command_queue()?;

            // Create offscreen render target
            self.create_offscreen_render_target()?;

            // Create RTV heap
            self.create_rtv_heap_headless()?;

            // Create depth stencil buffer (includes DSV heap creation)
            self.create_depth_stencil()?;

            // Create SRV descriptor heap for textures
            self.create_srv_heap()?;

            // Create command objects
            self.create_command_objects()?;

            // Create fence
            self.create_fence()?;

            // Note: Pipeline creation now happens via render graph compilation
            // The old create_pipeline() is no longer needed

            log::info!("DirectX 12 backend initialized successfully in headless mode");
            Ok(())
        }
    }

    pub fn capture_frame(&mut self) -> Result<(u32, u32, Vec<u8>)> {
        if !self.headless {
            anyhow::bail!("Frame capture is only available in headless mode");
        }

        let device = self.device.as_ref().context("Device not initialized")?;
        let command_queue = self
            .command_queue
            .as_ref()
            .context("Command queue not initialized")?;
        let offscreen = self
            .offscreen_resource
            .as_ref()
            .context("Offscreen resource not initialized")?;

        let width = self.width;
        let height = self.height;
        let row_pitch = ((width * 4 + 255) / 256) * 256; // Align to 256 bytes
        let buffer_size = (row_pitch * height) as u64;

        log::info!("Capturing frame: {width}x{height}");

        unsafe {
            // Create readback buffer if not exists
            if self.readback_buffer.is_none() {
                let heap_props = D3D12_HEAP_PROPERTIES {
                    Type: D3D12_HEAP_TYPE_READBACK,
                    CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                    MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                    CreationNodeMask: 1,
                    VisibleNodeMask: 1,
                };

                let desc = D3D12_RESOURCE_DESC {
                    Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                    Alignment: 0,
                    Width: buffer_size,
                    Height: 1,
                    DepthOrArraySize: 1,
                    MipLevels: 1,
                    Format: DXGI_FORMAT_UNKNOWN,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                    Flags: D3D12_RESOURCE_FLAG_NONE,
                };

                let mut readback: Option<ID3D12Resource> = None;
                device.CreateCommittedResource(
                    &heap_props,
                    D3D12_HEAP_FLAG_NONE,
                    &desc,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    None,
                    &mut readback,
                )?;

                self.readback_buffer = readback;
            }

            let readback = self.readback_buffer.as_ref().unwrap().clone();

            // Wait for upload allocator's previous work to complete
            if let Some(fence) = &self.fence {
                if self.upload_allocator_fence_value > 0 {
                    let completed = fence.GetCompletedValue();
                    if completed < self.upload_allocator_fence_value {
                        fence.SetEventOnCompletion(
                            self.upload_allocator_fence_value,
                            self.fence_event,
                        )?;
                        WaitForSingleObject(self.fence_event, INFINITE);
                    }
                }
            }

            // Use UPLOAD command allocator/list for screenshot to avoid conflicts
            let upload_allocator = self
                .upload_command_allocator
                .as_ref()
                .context("Upload command allocator not initialized")?;
            let upload_command_list = self
                .upload_command_list
                .as_ref()
                .context("Upload command list not initialized")?;

            // Reset upload command allocator and list for screenshot
            upload_allocator.Reset()?;
            upload_command_list.Reset(upload_allocator, None)?;

            // Transition offscreen resource to COPY_SOURCE
            let barrier_to_copy = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::ManuallyDrop::new(Some(offscreen.clone())),
                        StateBefore: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        StateAfter: D3D12_RESOURCE_STATE_COPY_SOURCE,
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    }),
                },
            };

            upload_command_list.ResourceBarrier(&[barrier_to_copy]);

            // Copy texture to buffer
            let src_location = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::ManuallyDrop::new(Some(offscreen.clone())),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: 0,
                },
            };

            let dst_location = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::ManuallyDrop::new(Some(readback.clone())),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                        Offset: 0,
                        Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                            Width: width,
                            Height: height,
                            Depth: 1,
                            RowPitch: row_pitch,
                        },
                    },
                },
            };

            upload_command_list.CopyTextureRegion(&dst_location, 0, 0, 0, &src_location, None);

            // Transition back to RENDER_TARGET
            let barrier_to_rt = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: std::mem::ManuallyDrop::new(Some(offscreen.clone())),
                        StateBefore: D3D12_RESOURCE_STATE_COPY_SOURCE,
                        StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    }),
                },
            };

            upload_command_list.ResourceBarrier(&[barrier_to_rt]);

            // Close and execute upload command list
            upload_command_list.Close()?;
            command_queue.ExecuteCommandLists(&[Some(upload_command_list.cast()?)]);

            // Wait for screenshot copy to complete and track upload allocator usage
            let screenshot_fence_value = self.fence_value;
            self.fence_value += 1;

            if let Some(fence) = &self.fence {
                command_queue.Signal(fence, screenshot_fence_value)?;
                self.upload_allocator_fence_value = screenshot_fence_value;
                if fence.GetCompletedValue() < screenshot_fence_value {
                    fence.SetEventOnCompletion(screenshot_fence_value, self.fence_event)?;
                    WaitForSingleObject(self.fence_event, INFINITE);
                }
            }

            // Map and read buffer
            let mut data_ptr = std::ptr::null_mut();
            readback.Map(0, None, Some(&mut data_ptr))?;

            let mut pixels = Vec::with_capacity((width * height * 4) as usize);

            // Copy data, removing row padding if necessary
            for y in 0..height {
                let src_offset = (y * row_pitch) as isize;
                let src_ptr = (data_ptr as *const u8).offset(src_offset);
                let row_data = std::slice::from_raw_parts(src_ptr, (width * 4) as usize);
                pixels.extend_from_slice(row_data);
            }

            readback.Unmap(0, None);

            log::info!("Frame captured: {width}x{height}, {} bytes", pixels.len());

            Ok((width, height, pixels))
        }
    }

    pub fn wait_idle(&mut self) -> Result<()> {
        if self.fence.is_some() {
            self.wait_for_previous_frame()?;
        }
        Ok(())
    }

    pub fn cleanup(&mut self) {
        log::info!("Cleaning up DirectX 12 backend");

        // Wait for GPU to finish
        if self.fence.is_some() {
            let _ = self.wait_for_previous_frame();
        }

        // Close fence event
        if !self.fence_event.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.fence_event);
            }
        }

        // Release resources in reverse order
        self.pipeline_state = None;
        self.root_signature = None;
        self.command_list = None;
        self.command_allocator = None;
        self.upload_command_list = None;
        self.upload_command_allocator = None;
        self.render_targets.clear();
        self.rtv_heap = None;

        // Headless-specific cleanup
        if self.headless {
            self.readback_buffer = None;
            self.offscreen_resource = None;
        } else {
            self.swap_chain = None;
        }

        self.command_queue = None;
        self.fence = None;
        self.device = None;
        self.dxgi_factory = None;

        log::info!("DirectX 12 backend cleaned up");
    }

    pub fn device(&self) -> &dyn Device {
        &self.device_wrapper
    }

    pub fn swapchain(&self) -> &dyn Swapchain {
        &self.swapchain_wrapper
    }

    /// Allocate resources from the render graph
    fn allocate_graph_resources(
        &mut self,
        graph: &crate::render_graph::graph::RenderGraph,
        compiled: &crate::render_graph::graph::CompiledGraph,
    ) -> Result<()> {
        use crate::render_graph::ResourceDescriptor;

        log::info!(
            "Allocating {} resources from render graph",
            compiled.resources_to_allocate.len()
        );

        // Get swapchain dimensions for resolving extent modes
        let swapchain_width = self.width;
        let swapchain_height = self.height;

        for &resource_id in &compiled.resources_to_allocate {
            let resource = graph
                .get_resource(resource_id)
                .context("Resource not found in graph")?;

            log::debug!(
                "Allocating resource '{}' ({:?})",
                resource.name,
                resource.kind
            );

            match &resource.descriptor {
                ResourceDescriptor::Image {
                    format,
                    extent,
                    usage,
                    samples: _samples,
                    mip_levels,
                } => {
                    // Resolve extent based on mode
                    let resolved_extent = extent.resolve(swapchain_width, swapchain_height);

                    // Convert render graph types to backend types
                    let backend_format = Self::convert_format(*format);
                    let backend_usage = Self::convert_image_usage(*usage);

                    // Check for initial data
                    let initial_data = match &resource.init_data {
                        crate::render_graph::ResourceInitData::Buffer(data) => {
                            Some(data.as_slice())
                        }
                        crate::render_graph::ResourceInitData::None => None,
                    };

                    // Create texture descriptor
                    let desc = TextureDescriptor {
                        width: resolved_extent.width,
                        height: resolved_extent.height,
                        format: backend_format,
                        usage: backend_usage,
                        mip_levels: *mip_levels,
                        initial_data,
                        label: Some(resource.name.clone()),
                    };

                    // Create the texture
                    let texture = self.create_texture(&desc)?;

                    log::debug!(
                        "Created texture '{}': {}x{}, format {:?}, {} mip levels{}",
                        resource.name,
                        resolved_extent.width,
                        resolved_extent.height,
                        backend_format,
                        mip_levels,
                        if initial_data.is_some() {
                            " (with initial data)"
                        } else {
                            ""
                        }
                    );

                    // Store in resource map
                    self.resource_textures.insert(resource_id, texture);
                }
                ResourceDescriptor::Buffer { size, usage } => {
                    // Convert usage flags
                    let backend_usage = Self::convert_buffer_usage(*usage);

                    // Create buffer descriptor
                    let desc = BufferDescriptor {
                        size: *size as u64,
                        usage: backend_usage,
                        memory_location: MemoryLocation::GpuOnly, // Default to GPU memory
                        label: Some(resource.name.clone()),
                    };

                    // Create the buffer
                    let buffer = self.create_buffer(&desc)?;

                    // Upload initial data if present
                    use crate::render_graph::ResourceInitData;
                    match &resource.init_data {
                        ResourceInitData::Buffer(data) => {
                            log::debug!(
                                "Uploading {} bytes of initial data to buffer '{}'",
                                data.len(),
                                resource.name
                            );

                            self.upload_to_buffer(buffer.as_ref(), data, 0)?;
                        }
                        ResourceInitData::None => {
                            // No initial data
                        }
                    }

                    log::debug!(
                        "Created buffer '{}': {} bytes, usage {:?}",
                        resource.name,
                        size,
                        backend_usage
                    );

                    // Store in resource map
                    self.resource_buffers.insert(resource_id, buffer);
                }
                ResourceDescriptor::Sampler(_sampler_desc) => {
                    // Samplers are handled separately and don't need allocation
                    // They are created when bind groups are set up
                    log::debug!("Skipping sampler '{}' (created on demand)", resource.name);
                }
            }
        }

        log::info!(
            "Resource allocation complete: {} textures, {} buffers",
            self.resource_textures.len(),
            self.resource_buffers.len()
        );

        Ok(())
    }

    /// Convert render graph format to backend format
    fn convert_format(format: crate::render_graph::Format) -> TextureFormat {
        use crate::render_graph::Format;
        match format {
            Format::Rgba8Unorm => TextureFormat::Rgba8Unorm,
            Format::Rgba8Srgb => TextureFormat::Rgba8Srgb,
            Format::Bgra8Unorm => TextureFormat::Bgra8Unorm,
            Format::Bgra8Srgb => TextureFormat::Bgra8Srgb,
            Format::Rgba16Float => TextureFormat::Rgba8Unorm, // Fallback to 8-bit
            Format::Rgba32Float => TextureFormat::Rgba8Unorm, // Fallback to 8-bit
            Format::Depth24Stencil8 => TextureFormat::Depth24PlusStencil8,
            Format::Depth32Float => TextureFormat::Depth32Float,
        }
    }

    /// Convert render graph image usage to backend usage
    fn convert_image_usage(usage: crate::render_graph::ImageUsageFlags) -> TextureUsage {
        use crate::render_graph::ImageUsageFlags;

        TextureUsage {
            sampled: usage.contains(ImageUsageFlags::SAMPLED),
            render_target: usage.contains(ImageUsageFlags::COLOR_ATTACHMENT),
            depth_stencil: usage.contains(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
            transfer_src: usage.contains(ImageUsageFlags::TRANSFER_SRC),
            transfer_dst: usage.contains(ImageUsageFlags::TRANSFER_DST),
        }
    }

    /// Convert render graph buffer usage to backend usage
    fn convert_buffer_usage(usage: crate::render_graph::BufferUsageFlags) -> BufferUsage {
        use crate::render_graph::BufferUsageFlags;

        BufferUsage {
            vertex: usage.contains(BufferUsageFlags::VERTEX),
            index: usage.contains(BufferUsageFlags::INDEX),
            uniform: usage.contains(BufferUsageFlags::UNIFORM),
            staging: false, // Not directly mappable from render graph
            storage: usage.contains(BufferUsageFlags::STORAGE),
            transfer_src: usage.contains(BufferUsageFlags::TRANSFER_SRC),
            transfer_dst: usage.contains(BufferUsageFlags::TRANSFER_DST),
        }
    }

    /// Compile a pipeline from a render graph pipeline builder
    fn compile_pipeline_from_builder(
        &mut self,
        builder: &crate::render_graph::PipelineBuilder,
        shader_registry: &crate::render_graph::ShaderRegistry,
        pass_id: crate::render_graph::PassId,
    ) -> Result<(ID3D12PipelineState, ID3D12RootSignature)> {
        use crate::render_graph::{ShaderStage, VertexFormat};

        let device = self.device.as_ref().context("Device not initialized")?;

        log::debug!(
            "Compiling DirectX pipeline from builder for pass {:?}",
            pass_id
        );

        unsafe {
            // Load shader bytecode
            let mut vs_bytecode = None;
            let mut ps_bytecode = None;

            for shader_handle in builder.shaders() {
                let shader_desc = shader_registry
                    .get_by_handle(*shader_handle)
                    .context("Shader not found in registry")?;

                // Compile to DXIL (DirectX bytecode)
                let dxil = shader_desc
                    .compile_to_dxil()
                    .map_err(|e| anyhow::anyhow!("Shader compilation failed: {}", e))?;

                match shader_desc.stage {
                    ShaderStage::Vertex => vs_bytecode = Some(dxil),
                    ShaderStage::Fragment => ps_bytecode = Some(dxil),
                    ShaderStage::Compute => {
                        anyhow::bail!("Compute shaders not yet supported in DirectX")
                    }
                }
            }

            let vs_bytecode = vs_bytecode.context("No vertex shader provided")?;
            let ps_bytecode = ps_bytecode.context("No pixel shader provided")?;

            // Determine if this is a shadow map pass based on pass name
            let is_shadow_pass = {
                // Convert pass_id to string using Debug format
                let pass_str = format!("{:?}", pass_id);
                pass_str.contains("shadow")
            };

            log::debug!(
                "Creating root signature for pass {:?}, is_shadow_pass={}",
                pass_id,
                is_shadow_pass
            );

            // Create root signature based on pass type
            let root_signature: ID3D12RootSignature = if is_shadow_pass {
                // Shadow map pass: only needs light uniforms (b0)
                let root_parameters = [
                    // Light uniform buffer (CBV b0) - contains lightViewProj matrix
                    D3D12_ROOT_PARAMETER {
                        ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                        Anonymous: D3D12_ROOT_PARAMETER_0 {
                            Descriptor: D3D12_ROOT_DESCRIPTOR {
                                ShaderRegister: 0,
                                RegisterSpace: 0,
                            },
                        },
                        ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
                    },
                ];

                let root_sig_desc = D3D12_ROOT_SIGNATURE_DESC {
                    NumParameters: root_parameters.len() as u32,
                    pParameters: root_parameters.as_ptr(),
                    NumStaticSamplers: 0,
                    pStaticSamplers: std::ptr::null(),
                    Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
                };

                let mut signature_blob: Option<ID3DBlob> = None;
                let mut error_blob: Option<ID3DBlob> = None;

                D3D12SerializeRootSignature(
                    &root_sig_desc,
                    D3D_ROOT_SIGNATURE_VERSION_1,
                    &mut signature_blob,
                    Some(&mut error_blob),
                )?;

                let signature_blob =
                    signature_blob.context("Failed to serialize root signature")?;

                device.CreateRootSignature(
                    0,
                    std::slice::from_raw_parts(
                        signature_blob.GetBufferPointer() as *const u8,
                        signature_blob.GetBufferSize(),
                    ),
                )?
            } else {
                // Forward pass: needs full set of parameters
                // Shader layout (as declared in forward_simple.hlsl):
                // - b0: LightingUniforms (CBV, pixel shader only)
                // - b1: ShadowUniforms (CBV)
                // - b2: PushConstants (root constants, contains camera matrices)
                // - t0: baseColorTexture (SRV)
                // - t1: shadowMap (SRV)
                // - s1: baseColorSampler (static sampler)
                // - s2: shadowSampler (static comparison sampler)

                // Descriptor ranges for textures
                // Note: Only t0 for now since ENABLE_SHADOWS is not defined in shaders
                let texture_descriptor_ranges = vec![
                    // t0: baseColorTexture
                    D3D12_DESCRIPTOR_RANGE {
                        RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
                        NumDescriptors: 1,
                        BaseShaderRegister: 0, // t0
                        RegisterSpace: 0,
                        OffsetInDescriptorsFromTableStart: 0,
                    },
                    // TODO: Add t1 (shadowMap) when ENABLE_SHADOWS is defined
                ];

                let root_parameters = [
                    // Lighting uniform buffer (CBV b0)
                    D3D12_ROOT_PARAMETER {
                        ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                        Anonymous: D3D12_ROOT_PARAMETER_0 {
                            Descriptor: D3D12_ROOT_DESCRIPTOR {
                                ShaderRegister: 0,
                                RegisterSpace: 0,
                            },
                        },
                        ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
                    },
                    // Shadow uniform buffer (CBV b1)
                    D3D12_ROOT_PARAMETER {
                        ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                        Anonymous: D3D12_ROOT_PARAMETER_0 {
                            Descriptor: D3D12_ROOT_DESCRIPTOR {
                                ShaderRegister: 1,
                                RegisterSpace: 0,
                            },
                        },
                        ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
                    },
                    // Root constants (push constants b2)
                    D3D12_ROOT_PARAMETER {
                        ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
                        Anonymous: D3D12_ROOT_PARAMETER_0 {
                            Constants: D3D12_ROOT_CONSTANTS {
                                ShaderRegister: 2,
                                RegisterSpace: 0,
                                Num32BitValues: 48, // 192 bytes (3 x 4x4 matrices: viewProj + model + normal)
                            },
                        },
                        ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
                    },
                    // Texture descriptor table (t0 and t1)
                    D3D12_ROOT_PARAMETER {
                        ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
                        Anonymous: D3D12_ROOT_PARAMETER_0 {
                            DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                                NumDescriptorRanges: texture_descriptor_ranges.len() as u32,
                                pDescriptorRanges: texture_descriptor_ranges.as_ptr(),
                            },
                        },
                        ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
                    },
                ];

                // Static samplers
                // Note: Only s0 for now since ENABLE_SHADOWS is not defined in shaders
                let static_samplers = [
                    // s0: baseColorSampler (regular sampler)
                    D3D12_STATIC_SAMPLER_DESC {
                        Filter: D3D12_FILTER_MIN_MAG_MIP_LINEAR,
                        AddressU: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
                        AddressV: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
                        AddressW: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
                        MipLODBias: 0.0,
                        MaxAnisotropy: 16,
                        ComparisonFunc: D3D12_COMPARISON_FUNC_NEVER,
                        BorderColor: D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK,
                        MinLOD: 0.0,
                        MaxLOD: f32::MAX,
                        ShaderRegister: 0, // s0
                        RegisterSpace: 0,
                        ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
                    },
                    // TODO: Add s1 (shadowSampler) when ENABLE_SHADOWS is defined
                ];

                let root_sig_desc = D3D12_ROOT_SIGNATURE_DESC {
                    NumParameters: root_parameters.len() as u32,
                    pParameters: root_parameters.as_ptr(),
                    NumStaticSamplers: static_samplers.len() as u32,
                    pStaticSamplers: static_samplers.as_ptr(),
                    Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
                };

                let mut signature_blob: Option<ID3DBlob> = None;
                let mut error_blob: Option<ID3DBlob> = None;

                D3D12SerializeRootSignature(
                    &root_sig_desc,
                    D3D_ROOT_SIGNATURE_VERSION_1,
                    &mut signature_blob,
                    Some(&mut error_blob),
                )?;

                let signature_blob =
                    signature_blob.context("Failed to serialize root signature")?;

                device.CreateRootSignature(
                    0,
                    std::slice::from_raw_parts(
                        signature_blob.GetBufferPointer() as *const u8,
                        signature_blob.GetBufferSize(),
                    ),
                )?
            };

            // Create input layout from vertex layout
            let input_layout = if let Some(layout) = builder.get_vertex_layout() {
                layout
                    .attributes
                    .iter()
                    .map(|attr| {
                        let format = match attr.format {
                            VertexFormat::Float32 => DXGI_FORMAT_R32_FLOAT,
                            VertexFormat::Float32x2 => DXGI_FORMAT_R32G32_FLOAT,
                            VertexFormat::Float32x3 => DXGI_FORMAT_R32G32B32_FLOAT,
                            VertexFormat::Float32x4 => DXGI_FORMAT_R32G32B32A32_FLOAT,
                            VertexFormat::Sint32 => DXGI_FORMAT_R32_SINT,
                            VertexFormat::Sint32x2 => DXGI_FORMAT_R32G32_SINT,
                            VertexFormat::Sint32x3 => DXGI_FORMAT_R32G32B32_SINT,
                            VertexFormat::Sint32x4 => DXGI_FORMAT_R32G32B32A32_SINT,
                            VertexFormat::Uint32 => DXGI_FORMAT_R32_UINT,
                            VertexFormat::Uint32x2 => DXGI_FORMAT_R32G32_UINT,
                            VertexFormat::Uint32x3 => DXGI_FORMAT_R32G32B32_UINT,
                            VertexFormat::Uint32x4 => DXGI_FORMAT_R32G32B32A32_UINT,
                        };

                        D3D12_INPUT_ELEMENT_DESC {
                            SemanticName: match attr.location {
                                0 => PCSTR(b"POSITION\0".as_ptr()),
                                1 => PCSTR(b"NORMAL\0".as_ptr()),
                                2 => PCSTR(b"TEXCOORD\0".as_ptr()),
                                3 => PCSTR(b"COLOR\0".as_ptr()),
                                _ => PCSTR(b"TEXCOORD\0".as_ptr()),
                            },
                            SemanticIndex: if attr.location == 2 || attr.location == 3 {
                                0
                            } else {
                                0
                            },
                            Format: format,
                            InputSlot: 0,
                            AlignedByteOffset: attr.offset,
                            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                            InstanceDataStepRate: 0,
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            // Create pipeline state
            let mut pso_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                pRootSignature: std::mem::ManuallyDrop::new(Some(root_signature.clone())),
                VS: D3D12_SHADER_BYTECODE {
                    pShaderBytecode: vs_bytecode.as_ptr() as *const _,
                    BytecodeLength: vs_bytecode.len(),
                },
                PS: D3D12_SHADER_BYTECODE {
                    pShaderBytecode: ps_bytecode.as_ptr() as *const _,
                    BytecodeLength: ps_bytecode.len(),
                },
                DS: D3D12_SHADER_BYTECODE::default(),
                HS: D3D12_SHADER_BYTECODE::default(),
                GS: D3D12_SHADER_BYTECODE::default(),
                StreamOutput: D3D12_STREAM_OUTPUT_DESC::default(),
                BlendState: D3D12_BLEND_DESC {
                    AlphaToCoverageEnable: FALSE,
                    IndependentBlendEnable: FALSE,
                    RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC {
                        BlendEnable: FALSE,
                        LogicOpEnable: FALSE,
                        SrcBlend: D3D12_BLEND_ONE,
                        DestBlend: D3D12_BLEND_ZERO,
                        BlendOp: D3D12_BLEND_OP_ADD,
                        SrcBlendAlpha: D3D12_BLEND_ONE,
                        DestBlendAlpha: D3D12_BLEND_ZERO,
                        BlendOpAlpha: D3D12_BLEND_OP_ADD,
                        LogicOp: D3D12_LOGIC_OP_NOOP,
                        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
                    }; 8],
                },
                SampleMask: u32::MAX,
                RasterizerState: D3D12_RASTERIZER_DESC {
                    FillMode: D3D12_FILL_MODE_SOLID,
                    CullMode: D3D12_CULL_MODE_BACK,
                    FrontCounterClockwise: TRUE, // Counter-clockwise to match Vulkan
                    DepthBias: 0,
                    DepthBiasClamp: 0.0,
                    SlopeScaledDepthBias: 0.0,
                    DepthClipEnable: TRUE,
                    MultisampleEnable: FALSE,
                    AntialiasedLineEnable: FALSE,
                    ForcedSampleCount: 0,
                    ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
                },
                DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
                    DepthEnable: TRUE, // Enable depth testing
                    DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
                    DepthFunc: D3D12_COMPARISON_FUNC_LESS,
                    StencilEnable: FALSE,
                    StencilReadMask: 0,
                    StencilWriteMask: 0,
                    FrontFace: D3D12_DEPTH_STENCILOP_DESC::default(),
                    BackFace: D3D12_DEPTH_STENCILOP_DESC::default(),
                },
                InputLayout: D3D12_INPUT_LAYOUT_DESC {
                    pInputElementDescs: if input_layout.is_empty() {
                        std::ptr::null()
                    } else {
                        input_layout.as_ptr()
                    },
                    NumElements: input_layout.len() as u32,
                },
                IBStripCutValue: D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED,
                PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                NumRenderTargets: if is_shadow_pass { 0 } else { 1 },
                RTVFormats: {
                    let mut formats = [DXGI_FORMAT_UNKNOWN; 8];
                    if !is_shadow_pass {
                        formats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
                    }
                    formats
                },
                DSVFormat: DXGI_FORMAT_D32_FLOAT,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                NodeMask: 0,
                CachedPSO: D3D12_CACHED_PIPELINE_STATE::default(),
                Flags: D3D12_PIPELINE_STATE_FLAG_NONE,
            };

            // Debug logging
            // Verify shader container magic (accept DXBC and DXIL containers)
            fn has_valid_dxil_magic(bytecode: &[u8]) -> bool {
                bytecode.len() >= 4
                    && (bytecode.starts_with(b"DXBC") || bytecode.starts_with(b"DXIL"))
            }

            if !has_valid_dxil_magic(&vs_bytecode) {
                anyhow::bail!("Invalid vertex shader: unexpected container header");
            }
            if !has_valid_dxil_magic(&ps_bytecode) {
                anyhow::bail!("Invalid pixel shader: unexpected container header");
            }

            let pipeline_state: ID3D12PipelineState =
                device.CreateGraphicsPipelineState(&pso_desc)?;
            log::debug!(
                "DirectX pipeline compiled successfully for pass {:?}",
                pass_id
            );

            Ok((pipeline_state, root_signature))
        }
    }

    /// Get or create render target view for a pass's color output
    fn get_pass_rtv(
        &mut self,
        graph: &crate::render_graph::graph::RenderGraph,
        pass_id: crate::render_graph::PassId,
    ) -> Result<Option<D3D12_CPU_DESCRIPTOR_HANDLE>> {
        use crate::render_graph::ImageLayout;

        let pass = graph.get_pass(pass_id).context("Pass not found")?;

        // Find color attachment output
        for output in &pass.outputs {
            if let Some(ImageLayout::ColorAttachment) = output.layout {
                // Check if we have this texture allocated
                if let Some(texture) = self.resource_textures.get(&output.resource) {
                    // For now, we need to get RTV from the texture
                    // In a full implementation, we'd create RTVs on demand
                    // For simplicity, return the default RTV for now
                    // TODO: Properly implement per-resource RTV creation
                    log::debug!(
                        "Found color output for pass {:?}: {:?}",
                        pass_id,
                        output.resource
                    );
                }
            }
        }

        // Default: use swapchain render target (or offscreen in headless)
        let rtv_heap = self.rtv_heap.as_ref().context("RTV heap not initialized")?;
        let handle_base = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };
        
        // In headless mode, always use descriptor 0 (only one RTV exists)
        // In windowed mode, use frame_index to select the current swapchain buffer
        let descriptor_index = if self.headless { 0 } else { self.frame_index as usize };
        
        let handle = D3D12_CPU_DESCRIPTOR_HANDLE {
            ptr: handle_base.ptr + (descriptor_index * self.rtv_descriptor_size as usize),
        };
        Ok(Some(handle))
    }

    /// Get or create depth stencil view for a pass's depth output
    fn get_pass_dsv(
        &mut self,
        graph: &crate::render_graph::graph::RenderGraph,
        pass_id: crate::render_graph::PassId,
    ) -> Result<Option<D3D12_CPU_DESCRIPTOR_HANDLE>> {
        use crate::render_graph::ImageLayout;

        let pass = graph.get_pass(pass_id).context("Pass not found")?;

        // Find depth attachment output
        for output in &pass.outputs {
            if let Some(ImageLayout::DepthStencilAttachment) = output.layout {
                // Check if we have this texture allocated
                if let Some(_texture) = self.resource_textures.get(&output.resource) {
                    // For now, return the default DSV
                    // TODO: Properly implement per-resource DSV creation
                    log::debug!(
                        "Found depth output for pass {:?}: {:?}",
                        pass_id,
                        output.resource
                    );
                    let dsv_heap = self.dsv_heap.as_ref().context("DSV heap not created")?;
                    let dsv_handle = unsafe { dsv_heap.GetCPUDescriptorHandleForHeapStart() };
                    return Ok(Some(dsv_handle));
                }
            }
        }

        // No depth output for this pass
        Ok(None)
    }

    /// Execute a compiled render graph
    pub fn execute_graph(
        &mut self,
        graph: &crate::render_graph::graph::RenderGraph,
        compiled: &crate::render_graph::graph::CompiledGraph,
    ) -> Result<()> {
        use crate::render_graph::*;

        // Debug logging
        // Allocate resources if this is the first execution or resources changed
        // TODO: Add smarter resource lifecycle management (issue #87)
        if self.resource_buffers.is_empty() && self.resource_textures.is_empty() {
            self.allocate_graph_resources(graph, compiled)?;
        }

        // Compile/cache pipelines for all passes
        log::debug!(
            "Checking {} pipeline descriptions",
            compiled.pipeline_descriptions.len()
        );
        for (pass_id, builder) in &compiled.pipeline_descriptions {
            if !self.pipeline_cache.contains_key(pass_id) {
                log::debug!("Compiling pipeline for pass {:?}", pass_id);

                // Debug logging
                let (pipeline, root_sig) =
                    self.compile_pipeline_from_builder(builder, graph.shader_registry(), *pass_id)?;
                self.pipeline_cache.insert(*pass_id, pipeline);
                self.root_signature_cache.insert(*pass_id, root_sig);

                // Debug logging            } else {
                log::debug!("Using cached pipeline for pass {:?}", pass_id);
            }
        }
        log::info!("Pipelines compiled/cached successfully");

        // Debug logging
        log::debug!(
            "Executing render graph with {} passes, {} barriers",
            compiled.execution_order.len(),
            compiled.barriers.len()
        );

        unsafe {
            // Command list should already be reset by begin_frame()
            // We just record commands here
            let command_list = self
                .command_list
                .as_ref()
                .context("Command list not initialized")?;

            // Debug logging
            // Debug logging
            let _device = self.device.as_ref().context("Device not initialized")?;
            let command_list_ptr = self
                .command_list
                .as_ref()
                .context("Command list not initialized")?
                as *const ID3D12GraphicsCommandList;
            let command_list = &*command_list_ptr;
            let rtv_heap = self.rtv_heap.as_ref().context("RTV heap not initialized")?;

            // Debug logging
            // Extract values we need to avoid borrowing self
            let headless = self.headless;
            let width = self.width;
            let height = self.height;
            let frame_index = self.frame_index;
            let rtv_descriptor_size = self.rtv_descriptor_size;

            // Debug logging
            // Get render target for swapchain transitions (not for binding, just for transitions)
            let (render_target, _default_rtv_handle) = if headless {
                let resource = self
                    .offscreen_resource
                    .as_ref()
                    .context("Offscreen resource not initialized")?;
                let handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
                (resource.clone(), handle)
            } else {
                let resource = self.render_targets[frame_index as usize].clone();
                let handle_base = rtv_heap.GetCPUDescriptorHandleForHeapStart();
                let handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                    ptr: handle_base.ptr + (frame_index as usize * rtv_descriptor_size as usize),
                };
                (resource, handle)
            };

            // Transition to render target state (windowed only)
            if !headless {
                let transition_to_rt = D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: std::mem::ManuallyDrop::new(Some(render_target.clone())),
                    Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    StateBefore: D3D12_RESOURCE_STATE_PRESENT,
                    StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                };

                let barrier = D3D12_RESOURCE_BARRIER {
                    Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                    Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                    Anonymous: D3D12_RESOURCE_BARRIER_0 {
                        Transition: std::mem::ManuallyDrop::new(transition_to_rt),
                    },
                };

                command_list.ResourceBarrier(&[barrier]);
            }

            // Set viewport and scissor (common for all passes)
            let viewport = D3D12_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: width as f32,
                Height: height as f32,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };
            command_list.RSSetViewports(&[viewport]);

            let scissor = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            command_list.RSSetScissorRects(&[scissor]);

            // Set primitive topology (common for all passes)
            command_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

            // Set descriptor heaps (critical: must be done before any draw calls)
            if let Some(heap) = &self.cbv_srv_uav_heap {
                command_list.SetDescriptorHeaps(&[Some(heap.clone())]);
                log::debug!("Descriptor heap set for command list");
            } else {
                log::warn!("No CBV/SRV/UAV descriptor heap available");
            }
            // Execute passes in order
            for pass_id in &compiled.execution_order {
                log::debug!("Executing pass: {pass_id:?}");
                // Get render targets for this pass
                let rtv_handle = self.get_pass_rtv(graph, *pass_id)?;
                let dsv_handle = self.get_pass_dsv(graph, *pass_id)?;

                // Clear and set render targets for this pass
                if let Some(rtv) = rtv_handle {
                    let clear_color = [0.1f32, 0.1f32, 0.2f32, 1.0f32]; // Dark blue background (matches Vulkan)
                    log::debug!("Clearing RTV at ptr: 0x{:x} with color [{}, {}, {}, {}]", 
                        rtv.ptr, clear_color[0], clear_color[1], clear_color[2], clear_color[3]);
                    command_list.ClearRenderTargetView(rtv, &clear_color, None);

                    if let Some(dsv) = dsv_handle {
                        log::trace!("Clearing DSV and setting RTV+DSV");
                        command_list.ClearDepthStencilView(
                            dsv,
                            D3D12_CLEAR_FLAG_DEPTH,
                            1.0,
                            0,
                            &[],
                        );
                        command_list.OMSetRenderTargets(1, Some(&rtv), FALSE, Some(&dsv));
                    } else {
                        log::trace!("Setting RTV only (no DSV)");
                        command_list.OMSetRenderTargets(1, Some(&rtv), FALSE, None);
                    }
                } else if let Some(dsv) = dsv_handle {
                    // Depth-only pass (like shadow mapping)
                    log::trace!("Depth-only pass");
                    command_list.ClearDepthStencilView(dsv, D3D12_CLEAR_FLAG_DEPTH, 1.0, 0, &[]);
                    command_list.OMSetRenderTargets(0, None, FALSE, Some(&dsv));
                } else {
                    log::warn!("Pass {:?} has no RTV or DSV!", pass_id);
                }

                // Set per-pass pipeline state and root signature
                if let Some(pipeline_state) = self.pipeline_cache.get(pass_id) {
                    if let Some(root_signature) = self.root_signature_cache.get(pass_id) {
                        command_list.SetGraphicsRootSignature(root_signature);
                        command_list.SetPipelineState(pipeline_state);
                    } else {
                        log::warn!("No root signature for pass {:?}, skipping", pass_id);
                        continue;
                    }
                } else {
                    log::warn!("No pipeline for pass {:?}, skipping", pass_id);
                    continue;
                }

                // Note: Barriers are handled through resource state transitions
                // in the main graph execution. In a full implementation, we would
                // process barriers here.

                // Execute pass callback through context (M9)
                let pass = graph
                    .get_pass(*pass_id)
                    .context("Pass not found in graph")?;
                if let Some(callback) = &pass.callback {
                    // Phase 1: Prepare resources (no-op for DirectX)
                    let mut prep_context = DirectXPrepContext;
                    callback.prepare(&mut prep_context);
                    // Phase 2: Execute rendering
                    // Create pass context with command list and backend pointers
                    let backend_ptr = self as *mut DirectXBackendImpl;
                    let mut context = DirectXPassContext {
                        command_list: command_list as *const _ as *mut (),
                        backend: backend_ptr,
                        bound_texture_t0: None,
                        bound_texture_t1: None,
                        descriptor_table_set: false,
                    };

                    // Execute the pass callback
                    callback.execute(&mut context);
                } else {
                    log::warn!("Pass {:?} has no callback, skipping", pass_id);
                }
            }

            // Transition back to present (windowed mode only)
            if !headless {
                let transition_to_present = D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: std::mem::ManuallyDrop::new(Some(render_target.clone())),
                    Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                    StateBefore: D3D12_RESOURCE_STATE_RENDER_TARGET,
                    StateAfter: D3D12_RESOURCE_STATE_PRESENT,
                };

                let barrier = D3D12_RESOURCE_BARRIER {
                    Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                    Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                    Anonymous: D3D12_RESOURCE_BARRIER_0 {
                        Transition: std::mem::ManuallyDrop::new(transition_to_present),
                    },
                };

                command_list.ResourceBarrier(&[barrier]);
            }

            // Command list will be closed by end_frame()
            log::debug!("Render graph execution complete");
            Ok(())
        }
    }

    /// Insert DirectX 12 barriers from render graph barrier
    fn insert_dx12_barrier(
        &self,
        _command_list: &ID3D12GraphicsCommandList,
        _barrier: &crate::render_graph::Barrier,
        _resource: &ID3D12Resource,
    ) -> Result<()> {
        // DirectX 12 barrier translation
        // For now, barriers are handled by the main graph execution
        // In a full implementation, we would translate:
        // - ImageBarrier -> D3D12_RESOURCE_TRANSITION_BARRIER
        // - MemoryBarrier -> D3D12_RESOURCE_UAV_BARRIER
        // - Access flags -> D3D12_RESOURCE_STATES

        // Since we're doing simple render target transitions in execute_graph,
        // we don't need additional barriers for the triangle demo
        Ok(())
    }

    // Resource Management (M8.1)

    pub fn create_buffer(
        &mut self,
        desc: &crate::backends::BufferDescriptor,
    ) -> Result<Box<dyn crate::backends::Buffer>> {
        log::debug!(
            "Creating DirectX 12 buffer: {} bytes, usage: {:?}",
            desc.size,
            desc.usage
        );

        use windows::Win32::Graphics::Direct3D12::*;
        let device = self.device.as_ref().context("Device not initialized")?;
        // Map heap properties based on memory location
        let heap_props = match desc.memory_location {
            crate::backends::MemoryLocation::GpuOnly => D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                ..Default::default()
            },
            crate::backends::MemoryLocation::CpuToGpu => D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_UPLOAD,
                ..Default::default()
            },
            crate::backends::MemoryLocation::GpuToCpu => D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_READBACK,
                ..Default::default()
            },
        };

        // Map buffer usage to resource state
        // IMPORTANT: DirectX 12 requires specific initial states based on heap type:
        // - UPLOAD heap (CpuToGpu) must be GENERIC_READ
        // - READBACK heap (GpuToCpu) must be COPY_DEST
        // - DEFAULT heap (GpuOnly) can be any appropriate state
        let initial_state = match desc.memory_location {
            crate::backends::MemoryLocation::CpuToGpu => {
                // Upload heaps must start in GENERIC_READ state
                D3D12_RESOURCE_STATE_GENERIC_READ
            }
            crate::backends::MemoryLocation::GpuToCpu => {
                // Readback heaps must start in COPY_DEST state
                D3D12_RESOURCE_STATE_COPY_DEST
            }
            crate::backends::MemoryLocation::GpuOnly => {
                // For GPU-only buffers, start in COPY_DEST since we'll copy data to them
                // This avoids needing an explicit barrier before the copy
                D3D12_RESOURCE_STATE_COPY_DEST
            }
        };

        let buffer_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: 0,
            Width: desc.size,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN,
            SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: D3D12_RESOURCE_FLAG_NONE,
        };
        let mut resource: Option<ID3D12Resource> = None;
        let create_result = unsafe {
            device.CreateCommittedResource(
                &heap_props,
                D3D12_HEAP_FLAG_NONE,
                &buffer_desc,
                initial_state,
                None,
                &mut resource,
            )
        };
        if let Err(e) = create_result {
            log::error!(
                "Failed to create D3D12 buffer: size={}, usage={:?}, memory={:?}, error={}",
                desc.size,
                desc.usage,
                desc.memory_location,
                e
            );
            return Err(e.into());
        }
        let resource = resource.context("Failed to create D3D12 buffer resource")?;
        Ok(Box::new(DirectXBuffer {
            resource,
            size: desc.size,
            usage: desc.usage,
            memory_location: desc.memory_location,
        }))
    }

    pub fn upload_to_buffer(
        &mut self,
        buffer: &dyn crate::backends::Buffer,
        data: &[u8],
        offset: u64,
    ) -> Result<()> {
        use windows::Win32::Graphics::Direct3D12::*;

        let dx_buffer = buffer
            .as_any()
            .downcast_ref::<DirectXBuffer>()
            .context("Buffer is not a DirectXBuffer")?;

        // Check bounds
        if offset + data.len() as u64 > buffer.size() {
            anyhow::bail!(
                "Upload out of bounds: offset {} + size {} > buffer size {}",
                offset,
                data.len(),
                buffer.size()
            );
        }

        // GPU-only buffers need a staging buffer and GPU copy
        // CPU-accessible buffers can be mapped directly
        match buffer.memory_location() {
            crate::backends::MemoryLocation::GpuOnly => {
                // Create a temporary upload (staging) buffer
                let staging_buffer = self.create_buffer(&crate::backends::BufferDescriptor {
                    size: data.len() as u64,
                    usage: crate::backends::BufferUsage::staging(),
                    memory_location: crate::backends::MemoryLocation::CpuToGpu,
                    label: Some("Staging buffer".to_string()),
                })?;

                // Map staging buffer and copy data
                let staging_dx = staging_buffer
                    .as_any()
                    .downcast_ref::<DirectXBuffer>()
                    .context("Staging buffer is not a DirectXBuffer")?;

                unsafe {
                    let mut mapped_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                    log::debug!(
                        "DX: Mapping staging buffer for upload (size: {})",
                        data.len()
                    );
                    staging_dx
                        .resource
                        .Map(0, None, Some(&mut mapped_ptr))
                        .map_err(|e| {
                            log::error!("DX: Failed to map staging buffer: {:?}", e);
                            e
                        })?;
                    std::ptr::copy_nonoverlapping(data.as_ptr(), mapped_ptr as *mut u8, data.len());
                    staging_dx.resource.Unmap(0, None);
                }

                // Copy from staging to GPU buffer using upload command list
                let cmd_allocator = self
                    .upload_command_allocator
                    .as_ref()
                    .context("Upload command allocator not initialized")?;
                let cmd_list = self
                    .upload_command_list
                    .as_ref()
                    .context("Upload command list not initialized")?;

                unsafe {
                    // Wait for any pending operations on the upload command allocator
                    let fence = self.fence.as_ref().context("Fence not initialized")?;
                    let command_queue = self
                        .command_queue
                        .as_ref()
                        .context("Command queue not initialized")?;

                    // Wait for upload allocator's previous work to complete
                    if self.upload_allocator_fence_value > 0 {
                        let completed = fence.GetCompletedValue();
                        if completed < self.upload_allocator_fence_value {
                            fence.SetEventOnCompletion(
                                self.upload_allocator_fence_value,
                                self.fence_event,
                            )?;
                            WaitForSingleObject(self.fence_event, INFINITE);
                        }
                    }

                    cmd_allocator.Reset()?;
                    cmd_list.Reset(cmd_allocator, None)?;

                    // D3D12 supports implicit state promotion for buffers in DEFAULT heap
                    // COMMON state can be implicitly promoted to COPY_DEST for copy operations
                    // No explicit barriers needed for buffer copies from UPLOAD to DEFAULT heap
                    cmd_list.CopyBufferRegion(
                        &dx_buffer.resource,
                        offset,
                        &staging_dx.resource,
                        0,
                        data.len() as u64,
                    );

                    // Transition buffer from COPY_DEST to GENERIC_READ so it can be used for rendering
                    // This is CRITICAL - buffers start in COPY_DEST state but need to be readable
                    let transition = D3D12_RESOURCE_BARRIER {
                        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                        Anonymous: D3D12_RESOURCE_BARRIER_0 {
                            Transition: std::mem::ManuallyDrop::new(
                                D3D12_RESOURCE_TRANSITION_BARRIER {
                                    pResource: std::mem::ManuallyDrop::new(Some(
                                        dx_buffer.resource.clone(),
                                    )),
                                    Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                                    StateBefore: D3D12_RESOURCE_STATE_COPY_DEST,
                                    StateAfter: D3D12_RESOURCE_STATE_GENERIC_READ,
                                },
                            ),
                        },
                    };
                    cmd_list.ResourceBarrier(&[transition]);

                    log::debug!(
                        "DX: Added resource barrier to transition buffer to GENERIC_READ state"
                    );

                    cmd_list.Close()?;

                    // Execute command list
                    command_queue.ExecuteCommandLists(&[Some(cmd_list.cast()?)]);

                    // Wait for copy to complete and track upload allocator usage
                    let fence_value = self.fence_value;
                    command_queue.Signal(fence, fence_value)?;
                    self.upload_allocator_fence_value = fence_value;
                    self.fence_value += 1;

                    if fence.GetCompletedValue() < fence_value {
                        fence.SetEventOnCompletion(fence_value, self.fence_event)?;
                        WaitForSingleObject(self.fence_event, INFINITE);
                    }

                    log::debug!("DX: Buffer upload and state transition complete");
                }

                Ok(())
            }
            crate::backends::MemoryLocation::CpuToGpu
            | crate::backends::MemoryLocation::GpuToCpu => {
                // CPU-accessible buffers can be mapped directly

                // Log material buffers (32 bytes = material size)
                if data.len() == 32 {
                    let floats: &[f32] =
                        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const f32, 8) };
                    log::info!("DX: Uploading 32-byte buffer (material): base_color=[{:.2}, {:.2}, {:.2}, {:.2}], properties=[{:.2}, {:.2}, {:.2}, {:.2}]",
                        floats[0], floats[1], floats[2], floats[3],
                        floats[4], floats[5], floats[6], floats[7]);
                }

                unsafe {
                    let mut mapped_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                    log::debug!(
                        "DX: Mapping CPU-accessible buffer for upload (size: {}, offset: {})",
                        data.len(),
                        offset
                    );
                    dx_buffer
                        .resource
                        .Map(0, None, Some(&mut mapped_ptr))
                        .map_err(|e| {
                            log::error!("DX: Failed to map CPU-accessible buffer: {:?}", e);
                            e
                        })?;

                    let dst = (mapped_ptr as *mut u8).add(offset as usize);
                    std::ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());

                    dx_buffer.resource.Unmap(0, None);
                }
                Ok(())
            }
        }
    }

    pub fn create_texture(
        &mut self,
        desc: &crate::backends::TextureDescriptor,
    ) -> Result<Box<dyn crate::backends::Texture>> {
        #[cfg(windows)]
        {
            use windows::Win32::Graphics::Direct3D12::*;
            use windows::Win32::Graphics::Dxgi::Common::*;

            let device = self.device.as_ref().context("Device not initialized")?;

            let dxgi_format = dx12_helpers::texture_format_to_dxgi(desc.format);

            // For depth textures that will be sampled, we need to use typeless format
            // for the resource and specific formats for DSV and SRV
            let (resource_format, srv_format) = if desc.usage.depth_stencil && desc.usage.sampled {
                match desc.format {
                    TextureFormat::Depth32Float => {
                        (DXGI_FORMAT_R32_TYPELESS, DXGI_FORMAT_R32_FLOAT)
                    }
                    TextureFormat::Depth24PlusStencil8 => (
                        DXGI_FORMAT_R24G8_TYPELESS,
                        DXGI_FORMAT_R24_UNORM_X8_TYPELESS,
                    ),
                    _ => (dxgi_format, dxgi_format),
                }
            } else {
                (dxgi_format, dxgi_format)
            };

            // Create resource description
            let mut resource_desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Alignment: 0,
                Width: desc.width as u64,
                Height: desc.height,
                DepthOrArraySize: 1,
                MipLevels: desc.mip_levels as u16,
                Format: resource_format, // Use typeless format for depth+sampled textures
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                Flags: D3D12_RESOURCE_FLAG_NONE,
            };

            // Set resource flags based on usage
            if desc.usage.render_target {
                resource_desc.Flags |= D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET;
            }
            if desc.usage.depth_stencil {
                resource_desc.Flags |= D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL;
            }

            // Create heap properties (GPU-only for textures)
            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            // Create resource
            let mut resource: Option<ID3D12Resource> = None;
            unsafe {
                device.CreateCommittedResource(
                    &heap_props,
                    D3D12_HEAP_FLAG_NONE,
                    &resource_desc,
                    D3D12_RESOURCE_STATE_COMMON,
                    None,
                    &mut resource,
                )?;
            }

            let resource = resource.context("Failed to create DirectX texture resource")?;

            log::info!(
                "Created DirectX texture resource: {}x{}, format: {:?}, mip_levels: {}, usage: RT={} DS={} S={}",
                desc.width,
                desc.height,
                desc.format,
                desc.mip_levels,
                desc.usage.render_target,
                desc.usage.depth_stencil,
                desc.usage.sampled
            );

            // Create SRV for the texture
            let (srv_descriptor, srv_gpu_handle) = if desc.usage.sampled {
                let heap = self
                    .cbv_srv_uav_heap
                    .as_ref()
                    .context("SRV heap not created")?;
                let offset = self.descriptor_heap_offset;

                log::info!("Creating SRV at descriptor heap offset: {}", offset);

                // Get CPU and GPU handles
                let cpu_handle = unsafe {
                    let mut handle = heap.GetCPUDescriptorHandleForHeapStart();
                    handle.ptr += (offset * self.cbv_srv_uav_descriptor_size) as usize;
                    handle
                };

                let gpu_handle = unsafe {
                    let mut handle = heap.GetGPUDescriptorHandleForHeapStart();
                    handle.ptr += (offset * self.cbv_srv_uav_descriptor_size) as u64;
                    handle
                };

                // Create SRV
                let srv_desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
                    Format: srv_format, // Use appropriate read format (R32_FLOAT for depth)
                    ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                    Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                    Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                        Texture2D: D3D12_TEX2D_SRV {
                            MostDetailedMip: 0,
                            MipLevels: desc.mip_levels,
                            PlaneSlice: 0,
                            ResourceMinLODClamp: 0.0,
                        },
                    },
                };

                unsafe {
                    device.CreateShaderResourceView(&resource, Some(&srv_desc), cpu_handle);
                }

                // Increment descriptor offset for next texture
                self.descriptor_heap_offset += 1;

                log::info!(
                    "Created SRV at heap offset {}, GPU handle: 0x{:x}, next offset: {}",
                    offset,
                    gpu_handle.ptr,
                    self.descriptor_heap_offset
                );

                (Some(cpu_handle), Some(gpu_handle))
            } else {
                (None, None)
            };

            let mut texture = DirectXTexture {
                resource,
                width: desc.width,
                height: desc.height,
                format: desc.format,
                usage: desc.usage,
                mip_levels: desc.mip_levels,
                srv_descriptor,
                srv_gpu_handle,
            };

            // Upload initial data if provided
            if let Some(data) = desc.initial_data {
                log::info!("Uploading initial texture data: {} bytes", data.len());
                self.upload_to_texture(&mut texture, data, 0)?;
                log::info!("Texture upload completed");
            } else {
                log::info!("No initial data provided for texture");
            }

            Ok(Box::new(texture))
        }

        #[cfg(not(windows))]
        {
            let _ = desc;
            anyhow::bail!("DirectX 12 texture creation is only available on Windows")
        }
    }

    pub fn upload_to_texture(
        &mut self,
        texture: &dyn crate::backends::Texture,
        data: &[u8],
        mip_level: u32,
    ) -> Result<()> {
        #[cfg(windows)]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            // Downcast to DirectXTexture
            let dx_texture = texture
                .as_any()
                .downcast_ref::<DirectXTexture>()
                .context("Expected DirectXTexture")?;

            if mip_level >= texture.mip_levels() {
                anyhow::bail!(
                    "Mip level {} out of range (max {})",
                    mip_level,
                    texture.mip_levels() - 1
                );
            }

            // Calculate mip dimensions
            let mip_width = texture.width() >> mip_level;
            let mip_height = texture.height() >> mip_level;
            let bytes_per_pixel = texture.format().bytes_per_pixel();
            let row_pitch = mip_width * bytes_per_pixel;
            let expected_size = (row_pitch * mip_height) as usize;

            if data.len() != expected_size {
                anyhow::bail!(
                    "Data size mismatch: expected {} bytes for {}x{} texture, got {}",
                    expected_size,
                    mip_width,
                    mip_height,
                    data.len()
                );
            }

            let device = self.device.as_ref().context("Device not initialized")?;

            // Create upload buffer
            let upload_buffer_size = expected_size as u64;
            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_UPLOAD,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let buffer_desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Alignment: 0,
                Width: upload_buffer_size,
                Height: 1,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN,
                SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                Flags: D3D12_RESOURCE_FLAG_NONE,
            };

            let mut upload_buffer: Option<ID3D12Resource> = None;
            unsafe {
                device.CreateCommittedResource(
                    &heap_props,
                    D3D12_HEAP_FLAG_NONE,
                    &buffer_desc,
                    D3D12_RESOURCE_STATE_GENERIC_READ,
                    None,
                    &mut upload_buffer,
                )?;
            }

            let upload_buffer = upload_buffer.context("Failed to create upload buffer")?;

            // Map and copy data
            unsafe {
                let mut mapped_data = std::ptr::null_mut();
                upload_buffer.Map(0, None, Some(&mut mapped_data))?;
                std::ptr::copy_nonoverlapping(data.as_ptr(), mapped_data as *mut u8, data.len());
                upload_buffer.Unmap(0, None);
            }

            // Use upload command list for texture upload
            let command_list = self
                .upload_command_list
                .as_ref()
                .context("Upload command list not initialized")?;
            let command_allocator = self
                .upload_command_allocator
                .as_ref()
                .context("Upload command allocator not initialized")?;
            let command_queue = self
                .command_queue
                .as_ref()
                .context("Command queue not initialized")?;

            log::info!(
                "Starting texture upload: {}x{} mip{}, {} bytes",
                mip_width,
                mip_height,
                mip_level,
                data.len()
            );
            log::debug!(
                "Upload allocator fence value: {}",
                self.upload_allocator_fence_value
            );

            // Wait for upload allocator's previous work to complete before resetting
            unsafe {
                if let Some(fence) = &self.fence {
                    if self.upload_allocator_fence_value > 0 {
                        let completed = fence.GetCompletedValue();
                        log::debug!(
                            "Fence completed: {}, waiting for: {}",
                            completed,
                            self.upload_allocator_fence_value
                        );
                        if completed < self.upload_allocator_fence_value {
                            log::debug!("Waiting for upload allocator fence...");
                            fence.SetEventOnCompletion(
                                self.upload_allocator_fence_value,
                                self.fence_event,
                            )?;
                            WaitForSingleObject(self.fence_event, INFINITE);
                            log::debug!("Upload allocator fence wait complete");
                        }
                    }
                }

                command_allocator.Reset()?;
                command_list.Reset(command_allocator, None)?;
            }

            // Transition texture to COPY_DEST state
            let barrier_to_copy = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: unsafe { std::mem::transmute_copy(&dx_texture.resource) },
                        Subresource: mip_level,
                        StateBefore: D3D12_RESOURCE_STATE_COMMON,
                        StateAfter: D3D12_RESOURCE_STATE_COPY_DEST,
                    }),
                },
            };

            unsafe {
                command_list.ResourceBarrier(&[barrier_to_copy]);
            }

            // Set up texture copy region
            let texture_copy_location_dst = D3D12_TEXTURE_COPY_LOCATION {
                pResource: unsafe { std::mem::transmute_copy(&dx_texture.resource) },
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: mip_level,
                },
            };

            let texture_copy_location_src = D3D12_TEXTURE_COPY_LOCATION {
                pResource: unsafe { std::mem::transmute_copy(&upload_buffer) },
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                        Offset: 0,
                        Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                            Format: dx12_helpers::texture_format_to_dxgi(texture.format()),
                            Width: mip_width,
                            Height: mip_height,
                            Depth: 1,
                            RowPitch: row_pitch,
                        },
                    },
                },
            };

            // Copy from upload buffer to texture
            unsafe {
                command_list.CopyTextureRegion(
                    &texture_copy_location_dst,
                    0,
                    0,
                    0, // DstX, DstY, DstZ
                    &texture_copy_location_src,
                    None, // Copy entire source
                );
            }

            // Transition texture to PIXEL_SHADER_RESOURCE state
            let barrier_to_shader = D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: unsafe { std::mem::transmute_copy(&dx_texture.resource) },
                        Subresource: mip_level,
                        StateBefore: D3D12_RESOURCE_STATE_COPY_DEST,
                        StateAfter: D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                    }),
                },
            };

            unsafe {
                command_list.ResourceBarrier(&[barrier_to_shader]);
            }

            // Close and execute command list
            unsafe {
                command_list.Close()?;
                command_queue.ExecuteCommandLists(&[Some(command_list.clone().cast()?)]);
            }

            // Wait for upload to complete and track upload allocator usage
            let fence_value = self.fence_value;
            self.fence_value += 1;

            log::debug!("Signaling upload fence: {}", fence_value);

            unsafe {
                command_queue.Signal(
                    self.fence.as_ref().context("Fence not initialized")?,
                    fence_value,
                )?;
                self.upload_allocator_fence_value = fence_value;

                log::debug!(
                    "Waiting for upload to complete (fence value: {})...",
                    fence_value
                );
                let fence = self.fence.as_ref().unwrap();
                if fence.GetCompletedValue() < fence_value {
                    fence.SetEventOnCompletion(fence_value, self.fence_event)?;
                    WaitForSingleObject(self.fence_event, INFINITE);
                }
                log::debug!(
                    "Upload wait complete, fence completed: {}",
                    fence.GetCompletedValue()
                );
            }

            log::info!(
                "Texture upload complete: mip{} ({}x{}), fence: {}",
                mip_level,
                mip_width,
                mip_height,
                fence_value
            );

            Ok(())
        }

        #[cfg(not(windows))]
        {
            let _ = (texture, data, mip_level);
            anyhow::bail!("DirectX 12 texture upload is only available on Windows")
        }
    }

    pub fn create_sampler(
        &mut self,
        desc: &crate::backends::SamplerDescriptor,
    ) -> Result<Box<dyn crate::backends::Sampler>> {
        #[cfg(windows)]
        {
            // DirectX samplers are created as descriptors in the heap
            // For now, we just store the parameters
            log::debug!(
                "Created DirectX sampler: mag={:?}, min={:?}",
                desc.mag_filter,
                desc.min_filter
            );

            Ok(Box::new(DirectXSampler {
                mag_filter: desc.mag_filter,
                min_filter: desc.min_filter,
                mipmap_filter: desc.mipmap_filter,
                address_mode_u: desc.address_mode_u,
                address_mode_v: desc.address_mode_v,
                address_mode_w: desc.address_mode_w,
            }))
        }

        #[cfg(not(windows))]
        {
            let _ = desc;
            anyhow::bail!("DirectX 12 sampler creation is only available on Windows")
        }
    }

    // Shader Resource Binding (M8.3)

    pub fn create_bind_group_layout(
        &mut self,
        layout: &crate::backends::BindGroupLayout,
    ) -> Result<usize> {
        use crate::backends::ShaderBinding;
        use windows::Win32::Graphics::Direct3D12::*;

        log::debug!("Creating DirectX 12 root signature for bind group layout");

        let device = self.device.as_ref().context("Device not initialized")?;

        // Build root parameters for each binding
        let mut root_params: Vec<D3D12_ROOT_PARAMETER> = Vec::new();

        for binding in layout.bindings() {
            match binding {
                ShaderBinding::UniformBuffer {
                    binding: bind_idx, ..
                } => {
                    // Constant Buffer View (CBV)
                    let mut param = D3D12_ROOT_PARAMETER::default();
                    param.ParameterType = D3D12_ROOT_PARAMETER_TYPE_CBV;
                    param.ShaderVisibility = D3D12_SHADER_VISIBILITY_ALL;
                    unsafe {
                        param.Anonymous.Descriptor = D3D12_ROOT_DESCRIPTOR {
                            ShaderRegister: *bind_idx,
                            RegisterSpace: 0,
                        };
                    }
                    root_params.push(param);
                }
                ShaderBinding::StorageBuffer {
                    binding: bind_idx,
                    readonly,
                    ..
                } => {
                    // Unordered Access View (UAV) or Shader Resource View (SRV)
                    let mut param = D3D12_ROOT_PARAMETER::default();
                    if *readonly {
                        param.ParameterType = D3D12_ROOT_PARAMETER_TYPE_SRV;
                    } else {
                        param.ParameterType = D3D12_ROOT_PARAMETER_TYPE_UAV;
                    }
                    param.ShaderVisibility = D3D12_SHADER_VISIBILITY_ALL;
                    unsafe {
                        param.Anonymous.Descriptor = D3D12_ROOT_DESCRIPTOR {
                            ShaderRegister: *bind_idx,
                            RegisterSpace: 0,
                        };
                    }
                    root_params.push(param);
                }
                ShaderBinding::Texture { .. } | ShaderBinding::Sampler { .. } => {
                    // TODO: Implement descriptor table for textures/samplers
                    log::warn!("DirectX 12 texture/sampler binding via descriptor tables not yet fully implemented");
                }
            }
        }

        // Create root signature
        let root_sig_desc = D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: root_params.len() as u32,
            pParameters: if root_params.is_empty() {
                std::ptr::null()
            } else {
                root_params.as_ptr()
            },
            NumStaticSamplers: 0,
            pStaticSamplers: std::ptr::null(),
            Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };

        let mut blob: Option<ID3DBlob> = None;
        let mut error_blob: Option<ID3DBlob> = None;

        unsafe {
            D3D12SerializeRootSignature(
                &root_sig_desc,
                D3D_ROOT_SIGNATURE_VERSION_1,
                &mut blob,
                Some(&mut error_blob),
            )
            .context("Failed to serialize root signature")?;

            let blob = blob.context("Root signature blob is null")?;

            let root_signature: ID3D12RootSignature = device.CreateRootSignature(
                0,
                std::slice::from_raw_parts(
                    blob.GetBufferPointer() as *const u8,
                    blob.GetBufferSize(),
                ),
            )?;

            self.root_signatures.push(root_signature);
            let handle = self.root_signatures.len() - 1;

            log::debug!("Created DirectX 12 root signature with handle {}", handle);
            Ok(handle)
        }
    }

    pub fn create_bind_group(
        &mut self,
        layout_handle: usize,
        bind_group: &crate::backends::BindGroup,
    ) -> Result<usize> {
        use crate::backends::BoundResource;

        log::debug!("Creating DirectX 12 bind group (descriptor table)");

        // Verify layout exists
        let _layout = self
            .root_signatures
            .get(layout_handle)
            .context("Invalid bind group layout handle")?;

        // For DirectX 12, bind groups are descriptor tables
        // We allocate space in the descriptor heap
        let current_offset = self.descriptor_heap_offset;

        // Count how many descriptors we need
        let mut descriptor_count = 0;
        for (_binding, resource) in bind_group.resources() {
            match resource {
                BoundResource::UniformBuffer(_)
                | BoundResource::StorageBuffer(_)
                | BoundResource::Texture(_)
                | BoundResource::Sampler(_) => {
                    descriptor_count += 1;
                }
            }
        }

        // Reserve space in descriptor heap
        // TODO: Actually create descriptors in the heap
        self.descriptor_heap_offset += descriptor_count;

        self.descriptor_tables.push(current_offset);
        let handle = self.descriptor_tables.len() - 1;

        log::debug!(
            "Created DirectX 12 bind group with handle {} (descriptor offset: {})",
            handle,
            current_offset
        );
        Ok(handle)
    }

    pub fn bind_vertex_buffer(
        &mut self,
        binding: u32,
        buffer: &dyn crate::backends::Buffer,
        offset: u64,
    ) -> Result<()> {
        unsafe {
            let command_list = self
                .command_list
                .as_ref()
                .context("DirectX command list not initialized")?;

            // Downcast buffer implementation
            let dx_buffer = buffer
                .as_any()
                .downcast_ref::<DirectXBuffer>()
                .context("DirectX backend expected DirectXBuffer")?;

            // TODO: Make stride configurable per buffer; for now assume packed GPU vertex
            let stride = 48u32;
            let size_in_bytes: u32 = dx_buffer
                .size
                .saturating_sub(offset)
                .try_into()
                .context("Vertex buffer slice exceeds u32 range")?;

            let gpu_address = dx_buffer.resource.GetGPUVirtualAddress() + offset;

            // Validate GPU address is not null
            if gpu_address == 0 {
                log::error!(
                    "DirectX: Vertex buffer has null GPU address! Buffer size: {}, offset: {}",
                    dx_buffer.size,
                    offset
                );
                anyhow::bail!("Vertex buffer has null GPU virtual address");
            }

            let vbv = D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: gpu_address,
                SizeInBytes: size_in_bytes,
                StrideInBytes: stride,
            };

            command_list.IASetVertexBuffers(binding, Some(&[vbv]));
            log::trace!(
                "DirectX: bound vertex buffer (binding {}, stride {}, size {} bytes, GPU addr: 0x{:x})",
                binding,
                stride,
                size_in_bytes,
                gpu_address
            );
        }
        Ok(())
    }

    pub fn bind_index_buffer(
        &mut self,
        buffer: &dyn crate::backends::Buffer,
        offset: u64,
        index_type: crate::backends::IndexType,
    ) -> Result<()> {
        unsafe {
            let command_list = self
                .command_list
                .as_ref()
                .context("DirectX command list not initialized")?;

            let dx_buffer = buffer
                .as_any()
                .downcast_ref::<DirectXBuffer>()
                .context("DirectX backend expected DirectXBuffer")?;

            let format = match index_type {
                crate::backends::IndexType::U16 => DXGI_FORMAT_R16_UINT,
                crate::backends::IndexType::U32 => DXGI_FORMAT_R32_UINT,
            };

            let size_in_bytes: u32 = dx_buffer
                .size
                .saturating_sub(offset)
                .try_into()
                .context("Index buffer slice exceeds u32 range")?;

            let gpu_address = dx_buffer.resource.GetGPUVirtualAddress() + offset;

            // Validate GPU address is not null
            if gpu_address == 0 {
                log::error!(
                    "DirectX: Index buffer has null GPU address! Buffer size: {}, offset: {}",
                    dx_buffer.size,
                    offset
                );
                anyhow::bail!("Index buffer has null GPU virtual address");
            }

            let ibv = D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: gpu_address,
                SizeInBytes: size_in_bytes,
                Format: format,
            };

            command_list.IASetIndexBuffer(Some(&ibv));
            log::trace!(
                "DirectX: bound index buffer (format {:?}, size {} bytes, GPU addr: 0x{:x})",
                format,
                size_in_bytes,
                gpu_address
            );
        }
        Ok(())
    }

    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        unsafe {
            let command_list = self
                .command_list
                .as_ref()
                .context("DirectX command list not initialized")?;
            command_list.DrawInstanced(vertex_count, instance_count, first_vertex, first_instance);
            log::trace!(
                "DirectX: draw instanced (vertices {}, instances {})",
                vertex_count,
                instance_count
            );
        }
        Ok(())
    }

    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        unsafe {
            let command_list = self
                .command_list
                .as_ref()
                .context("DirectX command list not initialized")?;
            command_list.DrawIndexedInstanced(
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
            log::trace!(
                "DirectX: draw indexed (indices {}, instances {})",
                index_count,
                instance_count
            );
        }
        Ok(())
    }

    // Headless mode helper methods

    /// Create offscreen render target for headless mode
    fn create_offscreen_render_target(&mut self) -> Result<()> {
        unsafe {
            let device = self.device.as_ref().context("Device not initialized")?;

            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 1,
                VisibleNodeMask: 1,
            };

            let desc = D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Alignment: 0,
                Width: self.width as u64,
                Height: self.height,
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            };

            let clear_value = D3D12_CLEAR_VALUE {
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                Anonymous: D3D12_CLEAR_VALUE_0 {
                    Color: [0.0, 0.2, 0.4, 1.0],
                },
            };

            let mut resource: Option<ID3D12Resource> = None;
            device.CreateCommittedResource(
                &heap_props,
                D3D12_HEAP_FLAG_NONE,
                &desc,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                Some(&clear_value),
                &mut resource,
            )?;

            self.offscreen_resource = resource;

            log::info!(
                "Offscreen render target created: {}x{}",
                self.width,
                self.height
            );
            Ok(())
        }
    }

    /// Create RTV heap for headless mode
    fn create_rtv_heap_headless(&mut self) -> Result<()> {
        unsafe {
            let device = self.device.as_ref().context("Device not initialized")?;

            let desc = D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                NumDescriptors: 1, // Single offscreen target
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            };

            let rtv_heap: ID3D12DescriptorHeap = device.CreateDescriptorHeap(&desc)?;
            self.rtv_descriptor_size =
                device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

            // Create RTV for offscreen resource
            let rtv_handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
            device.CreateRenderTargetView(
                self.offscreen_resource.as_ref().unwrap(),
                None,
                rtv_handle,
            );

            self.rtv_heap = Some(rtv_heap);
            self.render_targets = vec![self.offscreen_resource.as_ref().unwrap().clone()];

            log::info!("RTV heap created for headless mode");
            Ok(())
        }
    }
}

// Stub device for trait implementation
struct DirectXDevice;

impl Device for DirectXDevice {
    fn name(&self) -> &str {
        "DirectX 12 Device"
    }

    fn supports_feature(&self, _feature: &str) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Stub swapchain for trait implementation
struct DirectXSwapchain {
    width: u32,
    height: u32,
    frame_index: usize,
}

impl Swapchain for DirectXSwapchain {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn current_frame(&self) -> usize {
        self.frame_index
    }

    fn acquire_next_image(&mut self) -> Result<()> {
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        Ok(())
    }

    fn recreate(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width;
        self.height = height;
        Ok(())
    }
}

// DirectXBuffer implementation
struct DirectXBuffer {
    resource: windows::Win32::Graphics::Direct3D12::ID3D12Resource,
    size: u64,
    usage: crate::backends::BufferUsage,
    memory_location: crate::backends::MemoryLocation,
}

impl crate::backends::Buffer for DirectXBuffer {
    fn size(&self) -> u64 {
        self.size
    }

    fn usage(&self) -> crate::backends::BufferUsage {
        self.usage
    }

    fn memory_location(&self) -> crate::backends::MemoryLocation {
        self.memory_location
    }

    fn map(&mut self) -> Result<&mut [u8]> {
        anyhow::bail!("DirectX buffer mapping not yet implemented")
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

// DirectXTexture implementation
#[allow(dead_code)] // Fields will be used when binding textures in Phase 4
struct DirectXTexture {
    resource: windows::Win32::Graphics::Direct3D12::ID3D12Resource,
    width: u32,
    height: u32,
    format: crate::backends::TextureFormat,
    usage: crate::backends::TextureUsage,
    mip_levels: u32,
    srv_descriptor: Option<D3D12_CPU_DESCRIPTOR_HANDLE>, // SRV descriptor handle
    srv_gpu_handle: Option<D3D12_GPU_DESCRIPTOR_HANDLE>, // GPU handle for binding
}

impl crate::backends::Texture for DirectXTexture {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn format(&self) -> crate::backends::TextureFormat {
        self.format
    }

    fn usage(&self) -> crate::backends::TextureUsage {
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

// DirectXSampler implementation
#[allow(dead_code)] // Will be used when binding samplers in Phase 4
struct DirectXSampler {
    // DirectX samplers are created inline in the descriptor heap
    // Store descriptor parameters for later binding
    mag_filter: crate::backends::FilterMode,
    min_filter: crate::backends::FilterMode,
    mipmap_filter: crate::backends::FilterMode,
    address_mode_u: crate::backends::AddressMode,
    address_mode_v: crate::backends::AddressMode,
    address_mode_w: crate::backends::AddressMode,
}

impl crate::backends::Sampler for DirectXSampler {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(windows)]
mod dx12_helpers {
    use super::*;
    use crate::backends::{AddressMode, FilterMode, TextureFormat};
    use windows::Win32::Graphics::Dxgi::Common::*;

    /// Convert TextureFormat to DXGI_FORMAT
    pub fn texture_format_to_dxgi(format: TextureFormat) -> DXGI_FORMAT {
        match format {
            TextureFormat::Rgba8Srgb => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
            TextureFormat::Rgba8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
            TextureFormat::Bgra8Srgb => DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
            TextureFormat::Bgra8Unorm => DXGI_FORMAT_B8G8R8A8_UNORM,
            TextureFormat::Depth32Float => DXGI_FORMAT_D32_FLOAT,
            TextureFormat::Depth24PlusStencil8 => DXGI_FORMAT_D24_UNORM_S8_UINT,
        }
    }
}

/// DirectX Pass execution context (M9)
///
/// Provides PassExecutionContext implementation for DirectX backend.
/// Uses raw pointer to avoid borrow checker issues (same pattern as Vulkan/wgpu).
// Preparation context for DirectX (no-op, DirectX doesn't need separate preparation)
struct DirectXPrepContext;

impl PassPreparationContext for DirectXPrepContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn prepare_uniform_buffer(
        &mut self,
        _set: u32,
        _binding: u32,
        _buffer_ptr: *const std::ffi::c_void,
        _offset: u64,
        _size: u64,
    ) -> Result<()> {
        // DirectX binds resources on-the-fly in execute(), no prep needed
        Ok(())
    }

    fn prepare_texture(
        &mut self,
        _set: u32,
        _binding: u32,
        _texture_ptr: *const std::ffi::c_void,
    ) -> Result<()> {
        // DirectX binds resources on-the-fly in execute(), no prep needed
        Ok(())
    }

    fn prepare_push_constants(
        &mut self,
        _stage_flags: u32,
        _offset: u32,
        _size: u32,
    ) -> Result<()> {
        // Push constants don't need preparation
        Ok(())
    }
}

struct DirectXPassContext {
    command_list: *mut (),
    backend: *mut DirectXBackendImpl,
    bound_texture_t0: Option<D3D12_GPU_DESCRIPTOR_HANDLE>,
    bound_texture_t1: Option<D3D12_GPU_DESCRIPTOR_HANDLE>,
    descriptor_table_set: bool,
}

impl DirectXPassContext {
    fn command_list(&self) -> &ID3D12GraphicsCommandList {
        unsafe { &*(self.command_list as *const ID3D12GraphicsCommandList) }
    }

    fn backend(&mut self) -> &mut DirectXBackendImpl {
        unsafe { &mut *self.backend }
    }
}

impl PassExecutionContext for DirectXPassContext {
    fn bind_vertex_buffer(
        &mut self,
        binding: u32,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
    ) -> Result<()> {
        log::info!(
            "bind_vertex_buffer called: binding={}, buffer_ptr={:?}, offset={}",
            binding,
            buffer_ptr,
            offset
        );
        unsafe {
            log::info!("Getting command list");
            let command_list = self.command_list();

            log::info!("Downcasting buffer ptr");
            // Downcast to DirectX buffer
            let buffer = &*(buffer_ptr as *const DirectXBuffer);
            let dx_buffer = &buffer.resource;

            // For now, use fixed stride for Vertex struct (48 bytes: 3 floats pos + 3 floats color + padding)
            // TODO: Pass stride through BufferDescriptor or separate parameter
            let stride = 48u32; // sizeof(Vertex) with padding

            log::info!("Getting GPU virtual address");
            let gpu_address = dx_buffer.GetGPUVirtualAddress() + offset;

            log::info!(
                "GPU address for vertex buffer: 0x{:x}, size: {}, stride: {}",
                gpu_address,
                buffer.size,
                stride
            );

            if gpu_address == 0 {
                log::error!(
                    "Invalid GPU address (0) for vertex buffer at binding {}",
                    binding
                );
                anyhow::bail!("Invalid GPU address for vertex buffer");
            }

            // Create vertex buffer view
            let vbv = D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: gpu_address,
                SizeInBytes: (buffer.size - offset) as u32,
                StrideInBytes: stride,
            };

            log::info!("Calling IASetVertexBuffers");
            command_list.IASetVertexBuffers(binding, Some(&[vbv]));

            log::info!(
                "Bound vertex buffer at binding {} (stride: {}, gpu_addr: 0x{:x})",
                binding,
                stride,
                gpu_address
            );
            Ok(())
        }
    }

    fn bind_index_buffer(
        &mut self,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
        index_type: crate::render_graph::IndexType,
    ) -> Result<()> {
        log::info!(
            "bind_index_buffer called: buffer_ptr={:?}, offset={}",
            buffer_ptr,
            offset
        );
        unsafe {
            log::info!("Getting command list");
            let command_list = self.command_list();

            log::info!("Downcasting buffer ptr");
            // Downcast to DirectX buffer
            let buffer = &*(buffer_ptr as *const DirectXBuffer);
            let dx_buffer = &buffer.resource;

            // Convert index type
            let format = match index_type {
                crate::render_graph::IndexType::U16 => DXGI_FORMAT_R16_UINT,
                crate::render_graph::IndexType::U32 => DXGI_FORMAT_R32_UINT,
            };

            log::info!("Getting GPU virtual address");
            let gpu_address = dx_buffer.GetGPUVirtualAddress() + offset;

            log::info!(
                "GPU address for index buffer: 0x{:x}, size: {}, format: {:?}",
                gpu_address,
                buffer.size,
                format
            );

            if gpu_address == 0 {
                log::error!("Invalid GPU address (0) for index buffer");
                anyhow::bail!("Invalid GPU address for index buffer");
            }

            // Create index buffer view
            let ibv = D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: gpu_address,
                SizeInBytes: (buffer.size - offset) as u32,
                Format: format,
            };

            log::info!("Calling IASetIndexBuffer");
            command_list.IASetIndexBuffer(Some(&ibv));

            log::info!(
                "Index buffer bound successfully (gpu_addr: 0x{:x})",
                gpu_address
            );
            Ok(())
        }
    }

    fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        unsafe {
            let command_list = self.command_list();
            command_list.DrawInstanced(vertex_count, instance_count, first_vertex, first_instance);
            log::trace!(
                "Draw: {} vertices, {} instances",
                vertex_count,
                instance_count
            );
            Ok(())
        }
    }

    fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        log::info!(
            "draw_indexed called: index_count={}, instance_count={}, first_index={}, vertex_offset={}, first_instance={}",
            index_count, instance_count, first_index, vertex_offset, first_instance
        );
        unsafe {
            log::info!("Getting command list for draw");
            let command_list = self.command_list();

            log::info!("Calling DrawIndexedInstanced");
            command_list.DrawIndexedInstanced(
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );

            log::info!(
                "DrawIndexed completed: {} indices, {} instances",
                index_count,
                instance_count
            );
            Ok(())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn bind_uniform_buffer(
        &mut self,
        set: u32,
        binding: u32,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
        _size: u64,
    ) -> Result<()> {
        log::debug!(
            "DirectXPassContext: Binding uniform buffer at set {set}, binding {binding}, offset {offset}"
        );

        // For MVP, we only support set 0
        if set != 0 {
            log::warn!("Only root parameter set 0 is currently supported, ignoring set {set}");
            return Ok(());
        }

        unsafe {
            let command_list = self.command_list();
            // Downcast to DirectX buffer
            let buffer = &*(buffer_ptr as *const DirectXBuffer);
            let dx_buffer = &buffer.resource;

            // Get GPU virtual address
            let gpu_address = dx_buffer.GetGPUVirtualAddress() + offset;

            // Validate GPU address
            if gpu_address == 0 {
                log::error!(
                    "Invalid GPU address (0) for buffer at set {}, binding {}",
                    set,
                    binding
                );
                anyhow::bail!("Invalid GPU address for uniform buffer");
            }

            // DirectX uses root parameter indices directly
            // binding 0 -> root parameter 0 (camera)
            // binding 1 -> root parameter 1 (lighting)
            // binding 3 -> root parameter 3 (material)
            // (binding 2 is root constants, not a buffer)
            let root_parameter_index = if binding >= 3 {
                binding // binding 3+ maps directly
            } else {
                binding // binding 0, 1 map directly
            };

            log::debug!(
                "DirectXPassContext: Binding uniform buffer at root parameter {}, GPU address: 0x{:X}, buffer size: {}",
                root_parameter_index, gpu_address, buffer.size
            );
            // Set the constant buffer view (CBV) for this root parameter
            command_list.SetGraphicsRootConstantBufferView(root_parameter_index, gpu_address);

            log::debug!(
                "DirectXPassContext: Uniform buffer bound to root parameter {}",
                root_parameter_index
            );
        }

        Ok(())
    }

    fn push_constants(&mut self, _stage_flags: u32, offset: u32, data: &[u8]) -> Result<()> {
        unsafe {
            let command_list = self.command_list();
            // Convert byte data to u32 array
            let num_values = data.len() / 4;
            let values = std::slice::from_raw_parts(data.as_ptr() as *const u32, num_values);
            // Root parameter 2 is for push constants (see root signature creation)
            const ROOT_PARAMETER_INDEX_PUSH_CONSTANTS: u32 = 2;

            // Set the 32-bit constants
            // offset is in bytes, but DirectX needs offset in DWORDs
            let offset_in_dwords = offset / 4;
            command_list.SetGraphicsRoot32BitConstants(
                ROOT_PARAMETER_INDEX_PUSH_CONSTANTS,
                num_values as u32,
                values.as_ptr() as *const _,
                offset_in_dwords,
            );
            log::debug!(
                "DirectXPassContext: Push constants set - {} DWORDs ({} bytes) at offset {}",
                num_values,
                data.len(),
                offset
            );
        }

        Ok(())
    }

    fn bind_texture(
        &mut self,
        set: u32,
        binding: u32,
        texture_ptr: *const std::ffi::c_void,
    ) -> Result<()> {
        use windows::Win32::Graphics::Direct3D12::*;

        log::info!(
            "DirectXPassContext: Binding texture at set {}, binding {}",
            set,
            binding
        );

        // Log to debug file for Wine/Proton debugging
        // For MVP, we support:
        // - set 0, binding 2 (baseColorTexture - t0)
        // - set 0, binding 4 (shadow map - t1)
        // Both are in the same descriptor table at root parameter 3
        if set != 0 {
            log::warn!(
                "Only set 0 is currently supported for textures, ignoring set {}",
                set
            );
            return Ok(());
        }

        unsafe {
            // Get texture
            let texture = &*(texture_ptr as *const DirectXTexture);

            log::info!(
                "Texture binding {}: {}x{}, format: {:?}, has SRV: {}, GPU handle: {:?}",
                binding,
                texture.width,
                texture.height,
                texture.format,
                texture.srv_gpu_handle.is_some(),
                texture.srv_gpu_handle.map(|h| format!("0x{:x}", h.ptr))
            );

            // Log to debug file
            // Store GPU handle based on binding
            match binding {
                2 => {
                    // baseColorTexture (t0)
                    if let Some(gpu_handle) = texture.srv_gpu_handle {
                        self.bound_texture_t0 = Some(gpu_handle);
                        log::info!("Stored t0 (albedo) GPU handle: 0x{:x}", gpu_handle.ptr);
                    } else {
                        log::warn!("Texture for t0 has no SRV!");
                    }
                }
                4 => {
                    // Shadow map (t1)
                    if let Some(gpu_handle) = texture.srv_gpu_handle {
                        self.bound_texture_t1 = Some(gpu_handle);
                        log::debug!("Stored t1 GPU handle: 0x{:x}", gpu_handle.ptr);
                    } else {
                        log::warn!("Texture for t1 has no SRV!");
                    }
                }
                _ => {
                    log::warn!("Unsupported texture binding {}, skipping", binding);
                    return Ok(());
                }
            };

            // CRITICAL: The descriptor table at root parameter 3 contains BOTH t0 and t1
            // in consecutive descriptor slots. We must:
            // 1. Wait until we have t0 bound (t1 is optional for some passes)
            // 2. Set the descriptor table ONCE, pointing to t0
            // 3. The GPU will automatically find t1 at the next descriptor slot
            //
            // Setting the table multiple times overwrites the previous binding!

            if !self.descriptor_table_set && self.bound_texture_t0.is_some() {
                let t0_handle = self.bound_texture_t0.unwrap();

                let cmd_list_ptr = self.command_list;
                let backend_ptr = self.backend;

                let command_list = &*(cmd_list_ptr as *const ID3D12GraphicsCommandList);
                let backend = &*(backend_ptr as *const DirectXBackendImpl);

                // Set SRV descriptor heap
                if let Some(heap) = &backend.cbv_srv_uav_heap {
                    command_list.SetDescriptorHeaps(&[Some(heap.clone())]);
                } else {
                    log::error!("No SRV descriptor heap available!");
                    anyhow::bail!("No SRV descriptor heap");
                }

                // Set the descriptor table pointing to t0
                // The shader expects t1 at the next consecutive descriptor
                command_list.SetGraphicsRootDescriptorTable(3, t0_handle);

                self.descriptor_table_set = true;

                log::info!(
                    "Set descriptor table at root parameter 3, t0 handle: 0x{:x} (t1 at next slot)",
                    t0_handle.ptr
                );
            }
        }

        Ok(())
    }

    fn get_buffer_ptr(&self, resource_id: ResourceId) -> anyhow::Result<*const std::ffi::c_void> {
        let backend = unsafe { &*self.backend };

        backend
            .resource_buffers
            .get(&resource_id)
            .map(|buffer| {
                buffer.as_ref() as *const dyn super::super::Buffer as *const std::ffi::c_void
            })
            .ok_or_else(|| anyhow::anyhow!("Buffer resource {:?} not found", resource_id))
    }

    fn get_texture_ptr(&self, resource_id: ResourceId) -> anyhow::Result<*const std::ffi::c_void> {
        let backend = unsafe { &*self.backend };

        backend
            .resource_textures
            .get(&resource_id)
            .map(|texture| {
                texture.as_ref() as *const dyn super::super::Texture as *const std::ffi::c_void
            })
            .ok_or_else(|| anyhow::anyhow!("Texture resource {:?} not found", resource_id))
    }
}
