//! DirectX 12 implementation details (Windows-only)
//!
//! This module contains the actual DirectX 12 implementation using the `windows` crate.
//! It's only compiled on Windows platforms.

use super::*;
use anyhow::{Context, Result};
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*,
    Win32::System::Threading::*,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

// HLSL shader source code - matches triangle.hlsl
const HLSL_SHADER_SOURCE: &str = r#"
struct VSOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR0;
};

VSOutput VSMain(uint vertexID : SV_VertexID) {
    VSOutput output;
    
    // Hardcoded triangle vertices (NDC coordinates)
    // DirectX uses Y-down, so we flip Y coordinates from Vulkan's Y-up
    float2 positions[3] = {
        float2(0.0, 0.5),    // Bottom center (flipped from -0.5)
        float2(0.5, -0.5),   // Top right (flipped from 0.5)
        float2(-0.5, -0.5)   // Top left (flipped from 0.5)
    };
    
    // Hardcoded vertex colors
    float3 colors[3] = {
        float3(1.0, 0.0, 0.0),  // Red - bottom center (vertex 0)
        float3(0.0, 1.0, 0.0),  // Green - top right (vertex 1)
        float3(0.0, 0.0, 1.0)   // Blue - top left (vertex 2)
    };
    
    output.position = float4(positions[vertexID], 0.0, 1.0);
    output.color = colors[vertexID];
    
    return output;
}

float4 PSMain(VSOutput input) : SV_TARGET {
    return float4(input.color, 1.0);
}
"#;

/// DirectX 12 backend implementation
pub struct DirectXBackendImpl {
    // Core D3D12 objects
    device: Option<ID3D12Device>,
    command_queue: Option<ID3D12CommandQueue>,
    command_allocator: Option<ID3D12CommandAllocator>,
    command_list: Option<ID3D12GraphicsCommandList>,

    // Swap chain
    dxgi_factory: Option<IDXGIFactory4>,
    swap_chain: Option<IDXGISwapChain3>,
    frame_index: u32,
    render_targets: Vec<ID3D12Resource>,
    rtv_heap: Option<ID3D12DescriptorHeap>,
    rtv_descriptor_size: u32,

    // Pipeline (will be created with embedded HLSL bytecode)
    pipeline_state: Option<ID3D12PipelineState>,
    root_signature: Option<ID3D12RootSignature>,

    // Synchronization
    fence: Option<ID3D12Fence>,
    fence_value: u64,
    fence_event: HANDLE,

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
            dxgi_factory: None,
            swap_chain: None,
            frame_index: 0,
            render_targets: Vec::new(),
            rtv_heap: None,
            rtv_descriptor_size: 0,
            pipeline_state: None,
            root_signature: None,
            fence: None,
            fence_value: 0,
            fence_event: HANDLE::default(),
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
        })
    }

    pub fn initialize(&mut self, window: &winit::window::Window) -> Result<()> {
        log::info!("Initializing DirectX 12 backend");

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

        // Create command objects
        self.create_command_objects()?;

        // Create fence
        self.create_fence()?;

        // Create pipeline with shaders
        self.create_pipeline()?;

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

    fn create_command_objects(&mut self) -> Result<()> {
        log::info!("Creating command allocator and list");

        unsafe {
            let device = self.device.as_ref().context("Device not created")?;

            let allocator: ID3D12CommandAllocator =
                device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;

            let command_list: ID3D12GraphicsCommandList =
                device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)?;

            // Close the command list initially
            command_list.Close()?;

            self.command_allocator = Some(allocator);
            self.command_list = Some(command_list);
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

            // Create empty root signature (no parameters needed for this simple triangle)
            let root_signature_desc = D3D12_ROOT_SIGNATURE_DESC {
                NumParameters: 0,
                pParameters: std::ptr::null(),
                NumStaticSamplers: 0,
                pStaticSamplers: std::ptr::null(),
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
                    CullMode: D3D12_CULL_MODE_NONE,
                    FrontCounterClockwise: FALSE,
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
                    DepthEnable: FALSE,
                    DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
                    DepthFunc: D3D12_COMPARISON_FUNC_LESS,
                    StencilEnable: FALSE,
                    StencilReadMask: D3D12_DEFAULT_STENCIL_READ_MASK as u8,
                    StencilWriteMask: D3D12_DEFAULT_STENCIL_WRITE_MASK as u8,
                    FrontFace: D3D12_DEPTH_STENCILOP_DESC::default(),
                    BackFace: D3D12_DEPTH_STENCILOP_DESC::default(),
                },
                InputLayout: D3D12_INPUT_LAYOUT_DESC {
                    pInputElementDescs: std::ptr::null(),
                    NumElements: 0,
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
                DSVFormat: DXGI_FORMAT_UNKNOWN,
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

    fn compile_shader(&self, entry_point: &str, target: &str) -> Result<ID3DBlob> {
        unsafe {
            let entry_cstr = format!("{}\0", entry_point);
            let target_cstr = format!("{}\0", target);

            let shader_source = PCSTR::from_raw(HLSL_SHADER_SOURCE.as_ptr());
            let entry = PCSTR::from_raw(entry_cstr.as_ptr());
            let target_pcstr = PCSTR::from_raw(target_cstr.as_ptr());

            let mut shader_blob: Option<ID3DBlob> = None;
            let mut error_blob: Option<ID3DBlob> = None;

            let result = D3DCompile(
                shader_source.as_ptr() as *const _,
                HLSL_SHADER_SOURCE.len(),
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
        // Reset command allocator and list
        unsafe {
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
            // Record rendering commands
            if let (
                Some(command_list),
                Some(rtv_heap),
                Some(root_signature),
                Some(pipeline_state),
            ) = (
                &self.command_list,
                &self.rtv_heap,
                &self.root_signature,
                &self.pipeline_state,
            ) {
                // Get current render target (offscreen for headless, swapchain for windowed)
                let rtv_handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
                let rtv_handle = if self.headless {
                    // Headless: use single offscreen target
                    rtv_handle
                } else {
                    // Windowed: use current swapchain target
                    D3D12_CPU_DESCRIPTOR_HANDLE {
                        ptr: rtv_handle.ptr
                            + (self.frame_index * self.rtv_descriptor_size) as usize,
                    }
                };

                // Get render target resource
                let frame_idx = if self.headless {
                    0
                } else {
                    self.frame_index as usize
                };
                if let Some(render_target) = self.render_targets.get(frame_idx) {
                    // Transition to render target (only for windowed mode from PRESENT state)
                    if !self.headless {
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

                    // Clear to black
                    let clear_color = [0.0f32, 0.0f32, 0.0f32, 1.0f32];
                    command_list.ClearRenderTargetView(rtv_handle, &clear_color, None);

                    // Set pipeline and draw triangle
                    command_list.SetGraphicsRootSignature(root_signature);
                    command_list.SetPipelineState(pipeline_state);
                    command_list.OMSetRenderTargets(1, Some(&rtv_handle), FALSE, None);

                    // Set viewport and scissor
                    let viewport = D3D12_VIEWPORT {
                        TopLeftX: 0.0,
                        TopLeftY: 0.0,
                        Width: self.width as f32,
                        Height: self.height as f32,
                        MinDepth: 0.0,
                        MaxDepth: 1.0,
                    };
                    command_list.RSSetViewports(&[viewport]);

                    let scissor = RECT {
                        left: 0,
                        top: 0,
                        right: self.width as i32,
                        bottom: self.height as i32,
                    };
                    command_list.RSSetScissorRects(&[scissor]);

                    // Set primitive topology
                    command_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

                    // Draw the triangle (3 vertices)
                    command_list.DrawInstanced(3, 1, 0, 0);

                    // Transition back to present (only for windowed mode)
                    if !self.headless {
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
                }

                // Close command list
                command_list.Close()?;
            }

            // Execute commands
            if let (Some(command_queue), Some(command_list)) =
                (&self.command_queue, &self.command_list)
            {
                let command_lists = [Some(command_list.cast()?)];
                command_queue.ExecuteCommandLists(&command_lists);
            }

            // Present (only for windowed mode)
            if !self.headless {
                if let Some(swap_chain) = &self.swap_chain {
                    swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
                }
            }

            // Wait for frame
            self.wait_for_previous_frame()?;
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

            // Create command objects
            self.create_command_objects()?;

            // Create fence
            self.create_fence()?;

            // Create pipeline
            self.create_pipeline()?;

            log::info!("DirectX 12 backend initialized successfully in headless mode");
            Ok(())
        }
    }

    pub fn capture_frame(&mut self) -> Result<(u32, u32, Vec<u8>)> {
        if !self.headless {
            anyhow::bail!("Frame capture is only available in headless mode");
        }

        let device = self.device.as_ref().context("Device not initialized")?;
        let command_list = self
            .command_list
            .as_ref()
            .context("Command list not initialized")?;
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
                    CreationNodeMask: 0,
                    VisibleNodeMask: 0,
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

            // Reset command allocator and list
            let allocator = self.command_allocator.as_ref().unwrap();
            allocator.Reset()?;
            command_list.Reset(allocator, None)?;

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

            command_list.ResourceBarrier(&[barrier_to_copy]);

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

            command_list.CopyTextureRegion(&dst_location, 0, 0, 0, &src_location, None);

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

            command_list.ResourceBarrier(&[barrier_to_rt]);

            // Close and execute command list
            command_list.Close()?;
            let command_queue = self.command_queue.as_ref().unwrap();
            command_queue.ExecuteCommandLists(&[Some(command_list.cast()?)]);

            // Wait for completion
            self.wait_for_previous_frame()?;

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

    // Headless mode helper methods

    /// Create offscreen render target for headless mode
    fn create_offscreen_render_target(&mut self) -> Result<()> {
        unsafe {
            let device = self.device.as_ref().context("Device not initialized")?;

            let heap_props = D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                CreationNodeMask: 0,
                VisibleNodeMask: 0,
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
