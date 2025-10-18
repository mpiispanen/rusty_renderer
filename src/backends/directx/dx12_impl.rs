//! DirectX 12 implementation details (Windows-only)
//!
//! This module contains the actual DirectX 12 implementation using the `windows` crate.
//! It's only compiled on Windows platforms.

use super::*;
use anyhow::{Context, Result};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::Common::*,
    Win32::Graphics::Dxgi::*,
    Win32::System::Threading::*,
};

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
    
    // Trait implementations
    device_wrapper: DirectXDevice,
    swapchain_wrapper: DirectXSwapchain,
}

// SAFETY: DirectX 12 objects are thread-safe once created
// HANDLE is just a pointer that we manage carefully
unsafe impl Send for DirectXBackendImpl {}
unsafe impl Sync for DirectXBackendImpl {}

impl DirectXBackendImpl {
    pub fn new() -> Result<Self> {
        let use_warp = std::env::var("RUSTY_RENDERER_USE_WARP").is_ok();
        
        log::info!("Creating DirectX 12 backend (WARP: {})", use_warp);
        
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
        
        // Enable debug layer in debug builds
        #[cfg(debug_assertions)]
        unsafe {
            let mut debug: Option<ID3D12Debug> = None;
            if D3D12GetDebugInterface(&mut debug).is_ok() {
                if let Some(debug) = debug {
                    debug.EnableDebugLayer();
                    log::info!("DirectX 12 debug layer enabled");
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
        
        // TODO: Create pipeline with shaders
        // This requires compiling HLSL or embedding compiled bytecode
        
        log::info!("DirectX 12 backend initialized successfully");
        Ok(())
    }

    fn create_factory(&mut self) -> Result<()> {
        log::info!("Creating DXGI factory");
        
        unsafe {
            #[cfg(debug_assertions)]
            let flags = DXGI_CREATE_FACTORY_DEBUG;
            
            #[cfg(not(debug_assertions))]
            let flags = DXGI_CREATE_FACTORY_FLAGS(0);
            
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
            let command_queue = self.command_queue.as_ref().context("Command queue not created")?;
            
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
            self.rtv_descriptor_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
            
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
            
            let allocator: ID3D12CommandAllocator = device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
            
            let command_list: ID3D12GraphicsCommandList = device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &allocator,
                None,
            )?;
            
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

    pub fn begin_frame(&mut self) -> Result<()> {
        log::debug!("DirectX: begin_frame");
        
        // Reset command allocator
        unsafe {
            if let Some(allocator) = &self.command_allocator {
                allocator.Reset()?;
                log::debug!("Command allocator reset");
            }
            
            if let (Some(command_list), Some(allocator)) = (&self.command_list, &self.command_allocator) {
                command_list.Reset(allocator, None)?;
                log::debug!("Command list reset");
            }
        }
        
        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<()> {
        log::info!("DirectX: end_frame (frame_index: {})", self.frame_index);
        
        unsafe {
            // Record simple clear commands
            if let (Some(command_list), Some(rtv_heap)) = (&self.command_list, &self.rtv_heap) {
                log::info!("Recording clear commands");
                
                // Get current back buffer
                let rtv_handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
                let rtv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                    ptr: rtv_handle.ptr + (self.frame_index * self.rtv_descriptor_size) as usize,
                };
                
                // Transition to render target
                if let Some(render_target) = self.render_targets.get(self.frame_index as usize) {
                    log::info!("Transitioning to render target");
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
                    log::info!("Transitioned to render target");
                    
                    // Clear to a dark blue color so we can see something
                    let clear_color = [0.0f32, 0.2f32, 0.4f32, 1.0f32];
                    command_list.ClearRenderTargetView(rtv_handle, &clear_color, None);
                    log::info!("Cleared to blue ({:?})", clear_color);
                    
                    // Transition back to present
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
                    log::info!("Transitioned to present");
                    
                    log::info!("Clear commands recorded");
                }
                
                // Close command list
                command_list.Close()?;
                log::info!("Command list closed");
            }
            
            // Execute commands
            if let (Some(command_queue), Some(command_list)) = (&self.command_queue, &self.command_list) {
                let command_lists = [Some(command_list.cast()?)];
                command_queue.ExecuteCommandLists(&command_lists);
                log::info!("Commands executed");
            }
            
            // Present
            if let Some(swap_chain) = &self.swap_chain {
                swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
                log::info!("Frame presented");
            }
            
            // Wait for frame
            self.wait_for_previous_frame()?;
            log::info!("Frame complete");
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
        
        log::info!("Resizing DirectX swap chain: {}x{} -> {}x{}", 
            self.width, self.height, width, height);
        
        self.width = width;
        self.height = height;
        
        // TODO: Recreate swap chain
        
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
        self.render_targets.clear();
        self.rtv_heap = None;
        self.swap_chain = None;
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
