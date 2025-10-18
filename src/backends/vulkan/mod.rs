//! Vulkan backend implementation
//!
//! This module implements the Vulkan graphics backend using vulkanalia.
//! Supports validation layers in debug mode and provides a complete
//! Vulkan rendering pipeline.

mod shaders;

use super::*;
use anyhow::{Context, Result};
use std::ffi::CStr;
use std::os::raw::c_void;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ExtDebugUtilsExtensionInstanceCommands;
use vulkanalia::vk::KhrSurfaceExtensionInstanceCommands;
use vulkanalia::vk::KhrSwapchainExtensionDeviceCommands;
use vulkanalia::window as vk_window;

// Alias to avoid name collision with our Device trait
type VkDevice = vulkanalia::Device;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

/// Vulkan backend implementation
pub struct VulkanBackend {
    // Core Vulkan objects (wrapped in Option for initialization order)
    entry: Option<Entry>,
    instance: Option<Instance>,
    messenger: Option<vk::DebugUtilsMessengerEXT>,
    validation_enabled: bool, // Track if validation is actually enabled

    // Device and queues
    physical_device: vk::PhysicalDevice,
    device: Option<VkDevice>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    // Surface and swapchain
    surface: vk::SurfaceKHR,
    swapchain_khr: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_outdated: bool,

    // Render pass and pipeline
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    framebuffers: Vec<vk::Framebuffer>,

    // Command buffers and synchronization
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    // Per-swapchain-image semaphores (one set per swapchain image)
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    // Per-frame-in-flight fences
    in_flight_fences: Vec<vk::Fence>,
    // Track which fence is associated with each swapchain image
    images_in_flight: Vec<Option<vk::Fence>>,
    current_frame: usize,
    image_index: u32,

    // Stub components (will be replaced in future issues)
    device_wrapper: VulkanDevice,
    swapchain: VulkanSwapchain,
}

impl VulkanBackend {
    /// Create a new Vulkan backend
    pub fn new() -> Result<Self> {
        log::info!("Creating Vulkan backend");

        // Load Vulkan library
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry = unsafe { Entry::new(loader).map_err(|e| anyhow::anyhow!("{e}"))? };

        log::info!("Vulkan library loaded successfully");

        Ok(Self {
            entry: Some(entry),
            instance: None,
            messenger: None,
            validation_enabled: false, // Will be set during instance creation
            physical_device: vk::PhysicalDevice::null(),
            device: None,
            graphics_queue: vk::Queue::null(),
            present_queue: vk::Queue::null(),
            surface: vk::SurfaceKHR::null(),
            swapchain_khr: vk::SwapchainKHR::null(),
            swapchain_format: vk::Format::default(),
            swapchain_extent: vk::Extent2D::default(),
            swapchain_images: vec![],
            swapchain_image_views: vec![],
            swapchain_outdated: false,
            render_pass: vk::RenderPass::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            pipeline: vk::Pipeline::null(),
            framebuffers: vec![],
            command_pool: vk::CommandPool::null(),
            command_buffers: vec![],
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![],
            images_in_flight: vec![],
            current_frame: 0,
            image_index: 0,
            device_wrapper: VulkanDevice::new(),
            swapchain: VulkanSwapchain::new(),
        })
    }

    /// Create Vulkan instance with validation layers
    fn create_instance(&mut self, window: &winit::window::Window) -> Result<()> {
        let entry = self.entry.as_ref().context("Entry not initialized")?;

        // Application info
        let app_info = vk::ApplicationInfo::builder()
            .application_name(b"Rusty Renderer\0")
            .application_version(vk::make_version(0, 1, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(0, 1, 0))
            .api_version(vk::make_version(1, 0, 0));

        // Get required extensions from window
        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        // Add debug utils extension in debug mode
        if VALIDATION_ENABLED {
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }

        // Validation layers
        let available_layers = unsafe { entry.enumerate_instance_layer_properties()? }
            .iter()
            .map(|l| l.layer_name)
            .collect::<Vec<_>>();

        let mut layers = Vec::new();

        if VALIDATION_ENABLED {
            if available_layers.contains(&VALIDATION_LAYER) {
                layers.push(VALIDATION_LAYER.as_ptr());
                self.validation_enabled = true;
                log::info!("Validation layers enabled");
            } else {
                self.validation_enabled = false;
                log::warn!("Validation layers requested but not available");
            }
        } else {
            self.validation_enabled = false;
        }

        // Create instance
        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);

        // Debug messenger for instance creation/destruction
        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .user_callback(Some(debug_callback));

        if self.validation_enabled {
            info = info.push_next(&mut debug_info);
        }

        let instance = unsafe { entry.create_instance(&info, None)? };

        log::info!("Vulkan instance created successfully");

        self.instance = Some(instance);

        Ok(())
    }

    /// Create debug messenger
    fn create_debug_messenger(&mut self) -> Result<()> {
        if !self.validation_enabled {
            return Ok(());
        }

        let instance = self.instance.as_ref().context("Instance not initialized")?;

        let info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .user_callback(Some(debug_callback));

        let messenger = unsafe { instance.create_debug_utils_messenger_ext(&info, None)? };

        self.messenger = Some(messenger);
        log::info!("Debug messenger created");

        Ok(())
    }

    /// Pick physical device (GPU)
    fn pick_physical_device(&mut self, window: &winit::window::Window) -> Result<()> {
        let instance = self.instance.as_ref().context("Instance not initialized")?;

        // Create temporary surface for device selection
        let surface = unsafe { vk_window::create_surface(instance, window, window)? };

        let devices = unsafe { instance.enumerate_physical_devices()? };

        log::info!("Available devices: {}", devices.len());

        // Find suitable device
        let physical_device = devices
            .iter()
            .find(|d| self.is_device_suitable(instance, **d, surface))
            .copied()
            .context("No suitable physical device found")?;

        // Log device info
        let props = unsafe { instance.get_physical_device_properties(physical_device) };
        let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) }.to_string_lossy();

        log::info!("Selected device: {device_name}");
        log::info!("Device type: {:?}", props.device_type);

        self.physical_device = physical_device;

        // Clean up temporary surface
        unsafe {
            instance.destroy_surface_khr(surface, None);
        }

        Ok(())
    }

    /// Check if device is suitable
    fn is_device_suitable(
        &self,
        instance: &Instance,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> bool {
        // Check queue families
        let queue_families = QueueFamilyIndices::get(instance, device, surface);
        if !queue_families.is_complete() {
            return false;
        }

        // Check device extensions
        if !self.check_device_extension_support(instance, device) {
            return false;
        }

        // Check swapchain support
        let swapchain_support = SwapchainSupport::get(instance, device, surface);
        if swapchain_support.formats.is_empty() || swapchain_support.present_modes.is_empty() {
            return false;
        }

        true
    }

    /// Check device extension support
    fn check_device_extension_support(
        &self,
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> bool {
        let available = unsafe {
            instance
                .enumerate_device_extension_properties(device, None)
                .unwrap_or_default()
        };

        let available_names: std::collections::HashSet<_> =
            available.iter().map(|e| e.extension_name).collect();

        DEVICE_EXTENSIONS
            .iter()
            .all(|ext| available_names.contains(ext))
    }

    /// Create logical device
    fn create_logical_device(&mut self, window: &winit::window::Window) -> Result<()> {
        let instance = self.instance.as_ref().context("Instance not initialized")?;

        // Create surface for queue family detection
        let surface = unsafe { vk_window::create_surface(instance, window, window)? };

        let indices = QueueFamilyIndices::get(instance, self.physical_device, surface);

        // Queue create infos
        let mut unique_indices = vec![indices.graphics, indices.present];
        unique_indices.dedup();

        let queue_priorities = [1.0];
        let queue_infos: Vec<_> = unique_indices
            .iter()
            .map(|&i| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(i)
                    .queue_priorities(&queue_priorities)
                    .build()
            })
            .collect();

        // Device features (none required for now)
        let features = vk::PhysicalDeviceFeatures::builder();

        // Extensions
        let extensions = DEVICE_EXTENSIONS
            .iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<_>>();

        // Validation layers
        let layers = if self.validation_enabled {
            vec![VALIDATION_LAYER.as_ptr()]
        } else {
            vec![]
        };

        let info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_features(&features)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);

        let device = unsafe { instance.create_device(self.physical_device, &info, None)? };

        // Get queue handles
        let graphics_queue = unsafe { device.get_device_queue(indices.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(indices.present, 0) };

        log::info!("Logical device created");
        log::info!("Graphics queue family: {}", indices.graphics);
        log::info!("Present queue family: {}", indices.present);

        self.device = Some(device);
        self.graphics_queue = graphics_queue;
        self.present_queue = present_queue;

        // Clean up temporary surface
        unsafe {
            instance.destroy_surface_khr(surface, None);
        }

        Ok(())
    }

    /// Create surface
    fn create_surface(&mut self, window: &winit::window::Window) -> Result<()> {
        let instance = self.instance.as_ref().context("Instance not initialized")?;

        let surface = unsafe { vk_window::create_surface(instance, window, window)? };

        self.surface = surface;
        log::info!("Vulkan surface created");

        Ok(())
    }

    /// Choose swapchain surface format
    fn choose_swapchain_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .unwrap_or(formats[0])
    }

    /// Choose swapchain present mode
    fn choose_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        present_modes
            .iter()
            .find(|&&m| m == vk::PresentModeKHR::MAILBOX)
            .copied()
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    /// Choose swapchain extent
    fn choose_swapchain_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window: &winit::window::Window,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let size = window.inner_size();
            vk::Extent2D {
                width: size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    /// Create swapchain
    fn create_swapchain(&mut self, window: &winit::window::Window) -> Result<()> {
        let instance = self.instance.as_ref().context("Instance not initialized")?;
        let device = self.device.as_ref().context("Device not initialized")?;

        // Get swapchain support
        let support = SwapchainSupport::get(instance, self.physical_device, self.surface);

        let format = Self::choose_swapchain_format(&support.formats);
        let present_mode = Self::choose_swapchain_present_mode(&support.present_modes);
        let extent = Self::choose_swapchain_extent(&support.capabilities, window);

        // Image count
        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count > 0
            && image_count > support.capabilities.max_image_count
        {
            image_count = support.capabilities.max_image_count;
        }

        // Queue family indices
        let indices = QueueFamilyIndices::get(instance, self.physical_device, self.surface);
        let queue_family_indices = vec![indices.graphics, indices.present];

        let (image_sharing_mode, queue_family_indices_slice) =
            if indices.graphics != indices.present {
                (vk::SharingMode::CONCURRENT, queue_family_indices.as_slice())
            } else {
                (vk::SharingMode::EXCLUSIVE, &[] as &[u32])
            };

        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(queue_family_indices_slice)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe { device.create_swapchain_khr(&info, None)? };

        self.swapchain_khr = swapchain;
        self.swapchain_format = format.format;
        self.swapchain_extent = extent;

        // Get swapchain images
        self.swapchain_images = unsafe { device.get_swapchain_images_khr(swapchain)? };

        log::info!("Swapchain created: {}x{}", extent.width, extent.height);
        log::info!("Swapchain images: {}", self.swapchain_images.len());

        Ok(())
    }

    /// Create swapchain image views
    fn create_swapchain_image_views(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;

        self.swapchain_image_views = self
            .swapchain_images
            .iter()
            .map(|&image| {
                let info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::_2D)
                    .format(self.swapchain_format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });

                unsafe { device.create_image_view(&info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        log::info!(
            "Created {} swapchain image views",
            self.swapchain_image_views.len()
        );

        Ok(())
    }

    /// Destroy swapchain resources
    fn destroy_swapchain(&mut self) {
        if let Some(device) = &self.device {
            unsafe {
                for &image_view in &self.swapchain_image_views {
                    device.destroy_image_view(image_view, None);
                }
                device.destroy_swapchain_khr(self.swapchain_khr, None);
            }
        }

        self.swapchain_image_views.clear();
        self.swapchain_images.clear();
    }

    /// Recreate swapchain (for resize)
    #[allow(dead_code)]
    fn recreate_swapchain(&mut self, window: &winit::window::Window) -> Result<()> {
        log::info!("Recreating swapchain");

        let device = self.device.as_ref().context("Device not initialized")?;

        // Wait for device to be idle
        unsafe {
            device.device_wait_idle()?;
        }

        // Destroy old swapchain
        self.destroy_swapchain();

        // Create new swapchain
        self.create_swapchain(window)?;
        self.create_swapchain_image_views()?;

        // Reset images_in_flight tracking for new swapchain
        let swapchain_image_count = self.swapchain_images.len();
        self.images_in_flight = vec![None; swapchain_image_count];

        Ok(())
    }

    /// Create render pass
    fn create_render_pass(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;

        let color_attachment = vk::AttachmentDescription::builder()
            .format(self.swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = &[color_attachment_ref];
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments);

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = &[color_attachment];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies);

        self.render_pass = unsafe { device.create_render_pass(&info, None)? };

        log::info!("Render pass created successfully");
        Ok(())
    }

    /// Create graphics pipeline
    fn create_pipeline(&mut self) -> Result<()> {
        log::info!("Creating graphics pipeline");
        let device = self.device.as_ref().context("Device not initialized")?;

        log::info!("Creating shader modules");
        // Create shader modules
        let vert_shader_module = self.create_shader_module(shaders::VERTEX_SHADER)?;
        log::info!("Vertex shader module created");
        let frag_shader_module = self.create_shader_module(shaders::FRAGMENT_SHADER)?;
        log::info!("Fragment shader module created");

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let stages = &[vert_stage, frag_stage];

        // Vertex input (none - hardcoded in shader)
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();

        // Input assembly
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // Viewport and scissor (dynamic)
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(self.swapchain_extent.width as f32)
            .height(self.swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.swapchain_extent);

        let viewports = &[viewport];
        let scissors = &[scissor];
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors);

        // Rasterization
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        // Multisampling (disabled)
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::_1);

        // Color blending
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false);

        let color_blend_attachments = &[color_blend_attachment];
        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(color_blend_attachments);

        // Pipeline layout (no descriptors)
        log::info!("Creating pipeline layout");
        let layout_info = vk::PipelineLayoutCreateInfo::builder();
        self.pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };
        log::info!("Pipeline layout created");

        // Create pipeline
        log::info!("Building graphics pipeline create info");
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .color_blend_state(&color_blend_info)
            .layout(self.pipeline_layout)
            .render_pass(self.render_pass)
            .subpass(0);

        log::info!("Creating graphics pipeline");
        let (pipelines, _) = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)?
        };
        log::info!("Graphics pipeline created successfully");

        self.pipeline = pipelines[0];
        log::info!("Graphics pipeline created successfully");

        // Cleanup shader modules
        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        log::info!("Graphics pipeline created");
        Ok(())
    }

    /// Create shader module from SPIR-V
    fn create_shader_module(&self, code: &[u32]) -> Result<vk::ShaderModule> {
        let device = self.device.as_ref().context("Device not initialized")?;

        if code.is_empty() {
            anyhow::bail!("Shader code is empty!");
        }

        let info = vk::ShaderModuleCreateInfo::builder()
            .code_size(std::mem::size_of_val(code))
            .code(code);

        Ok(unsafe { device.create_shader_module(&info, None)? })
    }

    /// Create framebuffers
    fn create_framebuffers(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;

        self.framebuffers = self
            .swapchain_image_views
            .iter()
            .map(|&image_view| {
                let attachments = &[image_view];
                let info = vk::FramebufferCreateInfo::builder()
                    .render_pass(self.render_pass)
                    .attachments(attachments)
                    .width(self.swapchain_extent.width)
                    .height(self.swapchain_extent.height)
                    .layers(1);

                unsafe { device.create_framebuffer(&info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        log::info!("Created {} framebuffers", self.framebuffers.len());
        Ok(())
    }

    /// Destroy pipeline resources
    fn destroy_pipeline(&mut self) {
        if let Some(device) = &self.device {
            unsafe {
                device.destroy_pipeline(self.pipeline, None);
                device.destroy_pipeline_layout(self.pipeline_layout, None);
                device.destroy_render_pass(self.render_pass, None);

                for &framebuffer in &self.framebuffers {
                    device.destroy_framebuffer(framebuffer, None);
                }
            }
        }
        self.framebuffers.clear();
    }

    /// Create command pool
    fn create_command_pool(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;
        let instance = self.instance.as_ref().context("Instance not initialized")?;

        let indices = QueueFamilyIndices::get(instance, self.physical_device, self.surface);

        let info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(indices.graphics)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        self.command_pool = unsafe { device.create_command_pool(&info, None)? };

        log::info!("Command pool created");
        Ok(())
    }

    /// Create command buffers
    fn create_command_buffers(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(self.framebuffers.len() as u32);

        self.command_buffers = unsafe { device.allocate_command_buffers(&allocate_info)? };

        log::info!("Created {} command buffers", self.command_buffers.len());
        Ok(())
    }

    /// Create synchronization objects
    fn create_sync_objects(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;

        const MAX_FRAMES_IN_FLIGHT: usize = 2;
        let swapchain_image_count = self.swapchain_images.len();

        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        // Create per-swapchain-image semaphores
        for _ in 0..swapchain_image_count {
            self.image_available_semaphores
                .push(unsafe { device.create_semaphore(&semaphore_info, None)? });
            self.render_finished_semaphores
                .push(unsafe { device.create_semaphore(&semaphore_info, None)? });
        }

        // Create per-frame-in-flight fences
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            self.in_flight_fences
                .push(unsafe { device.create_fence(&fence_info, None)? });
        }

        // Initialize images_in_flight tracking
        self.images_in_flight = vec![None; swapchain_image_count];

        log::info!(
            "Created synchronization objects: {} semaphore pairs (per swapchain image), {} fences (frames in flight)",
            swapchain_image_count,
            MAX_FRAMES_IN_FLIGHT
        );
        Ok(())
    }

    /// Record command buffer for a specific image index
    fn record_command_buffer(&self, image_index: usize) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;
        let command_buffer = self.command_buffers[image_index];

        let begin_info = vk::CommandBufferBeginInfo::builder();

        unsafe {
            device.begin_command_buffer(command_buffer, &begin_info)?;
        }

        let clear_values = &[vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(clear_values);

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            // Draw triangle (3 vertices, no vertex buffer - hardcoded in shader)
            device.cmd_draw(command_buffer, 3, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer)?;
        }

        Ok(())
    }

    /// Destroy synchronization objects
    fn destroy_sync_objects(&mut self) {
        if let Some(device) = &self.device {
            unsafe {
                for &semaphore in &self.image_available_semaphores {
                    device.destroy_semaphore(semaphore, None);
                }
                for &semaphore in &self.render_finished_semaphores {
                    device.destroy_semaphore(semaphore, None);
                }
                for &fence in &self.in_flight_fences {
                    device.destroy_fence(fence, None);
                }
            }
        }
        self.image_available_semaphores.clear();
        self.render_finished_semaphores.clear();
        self.in_flight_fences.clear();
        self.images_in_flight.clear();
    }
}

impl GraphicsBackend for VulkanBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Vulkan
    }

    fn initialize(&mut self, window: &winit::window::Window) -> Result<()> {
        log::info!("Initializing Vulkan backend");

        // Create instance
        self.create_instance(window)
            .context("Failed to create Vulkan instance")?;

        // Create debug messenger
        self.create_debug_messenger()
            .context("Failed to create debug messenger")?;

        // Pick physical device
        self.pick_physical_device(window)
            .context("Failed to pick physical device")?;

        // Create logical device
        self.create_logical_device(window)
            .context("Failed to create logical device")?;

        // Create surface
        self.create_surface(window)
            .context("Failed to create surface")?;

        // Create swapchain
        self.create_swapchain(window)
            .context("Failed to create swapchain")?;

        // Create swapchain image views
        self.create_swapchain_image_views()
            .context("Failed to create swapchain image views")?;

        // Create render pass
        self.create_render_pass()
            .context("Failed to create render pass")?;
        log::info!("Render pass completed successfully");

        log::info!("Creating graphics pipeline");
        // Create graphics pipeline
        self.create_pipeline()
            .context("Failed to create graphics pipeline")?;
        log::info!("Graphics pipeline creation completed");

        // Create framebuffers
        self.create_framebuffers()
            .context("Failed to create framebuffers")?;

        // Create command pool
        self.create_command_pool()
            .context("Failed to create command pool")?;

        // Create command buffers
        self.create_command_buffers()
            .context("Failed to create command buffers")?;

        // Create synchronization objects
        self.create_sync_objects()
            .context("Failed to create synchronization objects")?;

        log::info!("Vulkan backend initialized");
        Ok(())
    }

    fn begin_frame(&mut self) -> Result<()> {
        let device = match self.device.as_ref() {
            Some(d) => d,
            None => return Ok(()), // Not initialized yet
        };

        // Wait for the current frame's fence
        let in_flight_fence = self.in_flight_fences[self.current_frame];
        unsafe {
            device.wait_for_fences(&[in_flight_fence], true, u64::MAX)?;
        }

        // Acquire next image - we don't know which image yet, so we use current_frame semaphore temporarily
        // This is a limitation - ideally we'd use a fence here but that's more complex
        let image_available = self.image_available_semaphores[self.current_frame % self.image_available_semaphores.len()];
        let result = unsafe {
            device.acquire_next_image_khr(
                self.swapchain_khr,
                u64::MAX,
                image_available,
                vk::Fence::null(),
            )
        };

        let image_index = match result {
            Ok((index, _)) => index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                self.swapchain_outdated = true;
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        // Wait for the image-specific fence if this image is still in use
        if let Some(image_fence) = self.images_in_flight[image_index] {
            if image_fence != in_flight_fence {
                unsafe {
                    device.wait_for_fences(&[image_fence], true, u64::MAX)?;
                }
            }
        }

        // Mark this image as now being used by this frame's fence
        self.images_in_flight[image_index] = Some(in_flight_fence);

        self.image_index = image_index as u32;

        // Reset fence only after we're sure we can use it
        unsafe {
            device.reset_fences(&[in_flight_fence])?;
        }

        // Record command buffer
        self.record_command_buffer(image_index)?;

        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        let device = match self.device.as_ref() {
            Some(d) => d,
            None => return Ok(()), // Not initialized yet
        };

        if self.swapchain_outdated {
            return Ok(());
        }

        let image_index = self.image_index as usize;

        // Use the image-specific semaphores for this swapchain image
        let wait_semaphores = &[self.image_available_semaphores[self.current_frame % self.image_available_semaphores.len()]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.command_buffers[image_index]];
        let signal_semaphores = &[self.render_finished_semaphores[image_index]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        let in_flight_fence = self.in_flight_fences[self.current_frame];

        unsafe {
            device.queue_submit(self.graphics_queue, &[submit_info], in_flight_fence)?;
        }

        let swapchains = &[self.swapchain_khr];
        let image_indices = &[self.image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = unsafe { device.queue_present_khr(self.present_queue, &present_info) };

        let changed = match result {
            Ok(vk::SuccessCode::SUCCESS) => false,
            Ok(vk::SuccessCode::SUBOPTIMAL_KHR) | Err(vk::ErrorCode::OUT_OF_DATE_KHR) => true,
            Err(e) => return Err(e.into()),
            Ok(_) => false,
        };

        if changed {
            self.swapchain_outdated = true;
        }

        // Advance to next frame
        self.current_frame = (self.current_frame + 1) % self.in_flight_fences.len();

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        // Check if size actually changed
        if self.swapchain_extent.width == width && self.swapchain_extent.height == height {
            return Ok(());
        }

        log::info!(
            "Resizing swapchain: {}x{} -> {width}x{height}",
            self.swapchain_extent.width,
            self.swapchain_extent.height
        );
        self.swapchain_outdated = true;
        Ok(())
    }

    fn cleanup(&mut self) {
        log::info!("Cleaning up Vulkan backend");

        unsafe {
            // Wait for device to finish
            if let Some(device) = &self.device {
                let _ = device.device_wait_idle();
            }

            // Destroy synchronization objects
            self.destroy_sync_objects();

            // Destroy command pool (also frees command buffers)
            if let Some(device) = &self.device {
                device.destroy_command_pool(self.command_pool, None);
            }

            // Destroy pipeline
            self.destroy_pipeline();

            // Destroy swapchain
            self.destroy_swapchain();

            // Destroy surface
            if let Some(instance) = &self.instance {
                instance.destroy_surface_khr(self.surface, None);
            }

            // Destroy logical device
            if let Some(device) = &self.device {
                device.destroy_device(None);
            }

            // Destroy debug messenger and instance
            if let Some(instance) = &self.instance {
                if let Some(messenger) = self.messenger {
                    instance.destroy_debug_utils_messenger_ext(messenger, None);
                }
                instance.destroy_instance(None);
            }
        }

        log::info!("Vulkan backend cleaned up");
    }

    fn device(&self) -> &dyn super::Device {
        &self.device_wrapper
    }

    fn swapchain(&self) -> &dyn Swapchain {
        &self.swapchain
    }
}

/// Debug callback for Vulkan validation layers
unsafe extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = *data;
    let message = if data.message.is_null() {
        "<no message>".into()
    } else {
        CStr::from_ptr(data.message).to_string_lossy()
    };

    match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("Vulkan [{type_:?}]: {message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("Vulkan [{type_:?}]: {message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("Vulkan [{type_:?}]: {message}");
        }
        _ => {
            log::debug!("Vulkan [{type_:?}]: {message}");
        }
    }

    vk::FALSE
}

/// Vulkan device stub
#[derive(Default)]
pub struct VulkanDevice;

impl VulkanDevice {
    /// Create a new Vulkan device stub
    pub fn new() -> Self {
        Self
    }
}

impl super::Device for VulkanDevice {
    fn name(&self) -> &str {
        "Vulkan Stub Device"
    }

    fn supports_feature(&self, _feature: &str) -> bool {
        // Stub: return false for all features
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Vulkan command buffer stub
#[derive(Default)]
pub struct VulkanCommandBuffer;

impl VulkanCommandBuffer {
    /// Create a new Vulkan command buffer stub
    pub fn new() -> Self {
        Self
    }
}

impl CommandBuffer for VulkanCommandBuffer {
    fn begin(&mut self) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn end(&mut self) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn clear_color(&mut self, _r: f32, _g: f32, _b: f32, _a: f32) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn bind_pipeline(&mut self, _pipeline: &dyn Pipeline) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn draw(&mut self, _vertex_count: u32, _instance_count: u32) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }
}

/// Vulkan pipeline stub
#[derive(Default)]
pub struct VulkanPipeline;

impl VulkanPipeline {
    /// Create a new Vulkan pipeline stub
    pub fn new() -> Self {
        Self
    }
}

impl Pipeline for VulkanPipeline {
    fn name(&self) -> &str {
        "Vulkan Stub Pipeline"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Vulkan resource stub
#[derive(Default)]
pub struct VulkanResource;

impl VulkanResource {
    /// Create a new Vulkan resource stub
    pub fn new() -> Self {
        Self
    }
}

impl Resource for VulkanResource {
    fn size(&self) -> usize {
        // Stub: return 0
        0
    }

    fn resource_type(&self) -> &str {
        "Vulkan Stub Resource"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Vulkan swapchain stub
pub struct VulkanSwapchain {
    width: u32,
    height: u32,
    current_frame: usize,
}

impl Default for VulkanSwapchain {
    fn default() -> Self {
        Self::new()
    }
}

impl VulkanSwapchain {
    /// Create a new Vulkan swapchain stub
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            current_frame: 0,
        }
    }
}

impl Swapchain for VulkanSwapchain {
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
        // Stub: will be implemented in M3
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn recreate(&mut self, width: u32, height: u32) -> Result<()> {
        // Stub: update dimensions
        self.width = width;
        self.height = height;
        Ok(())
    }
}

/// Queue family indices
#[derive(Copy, Clone, Debug)]
struct QueueFamilyIndices {
    graphics: u32,
    present: u32,
}

impl QueueFamilyIndices {
    fn get(instance: &Instance, device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Self {
        let properties = unsafe { instance.get_physical_device_queue_family_properties(device) };

        let mut graphics = None;
        let mut present = None;

        for (index, properties) in properties.iter().enumerate() {
            if properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics = Some(index as u32);
            }

            let present_support = unsafe {
                instance
                    .get_physical_device_surface_support_khr(device, index as u32, surface)
                    .unwrap_or(false)
            };

            if present_support {
                present = Some(index as u32);
            }

            if graphics.is_some() && present.is_some() {
                break;
            }
        }

        Self {
            graphics: graphics.unwrap_or(0),
            present: present.unwrap_or(0),
        }
    }

    fn is_complete(&self) -> bool {
        self.graphics != u32::MAX && self.present != u32::MAX
    }
}

/// Swapchain support details
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
struct SwapchainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    fn get(instance: &Instance, device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Self {
        unsafe {
            Self {
                capabilities: instance
                    .get_physical_device_surface_capabilities_khr(device, surface)
                    .unwrap_or_default(),
                formats: instance
                    .get_physical_device_surface_formats_khr(device, surface)
                    .unwrap_or_default(),
                present_modes: instance
                    .get_physical_device_surface_present_modes_khr(device, surface)
                    .unwrap_or_default(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulkan_backend_creation() {
        let backend = VulkanBackend::new();
        assert!(backend.is_ok(), "Failed to create Vulkan backend");
    }

    #[test]
    fn test_vulkan_backend_type() {
        let backend = VulkanBackend::new().unwrap();
        assert_eq!(backend.backend_type(), BackendType::Vulkan);
    }

    #[test]
    fn test_vulkan_device_access() {
        let backend = VulkanBackend::new().unwrap();
        let device = backend.device();
        assert_eq!(device.name(), "Vulkan Stub Device");
    }

    #[test]
    fn test_vulkan_device_features() {
        let backend = VulkanBackend::new().unwrap();
        let device = backend.device();
        assert!(!device.supports_feature("any_feature"));
    }

    #[test]
    fn test_vulkan_swapchain_access() {
        let backend = VulkanBackend::new().unwrap();
        let swapchain = backend.swapchain();
        assert_eq!(swapchain.width(), 800);
        assert_eq!(swapchain.height(), 600);
        assert_eq!(swapchain.current_frame(), 0);
    }

    #[test]
    fn test_vulkan_swapchain_recreate() {
        let mut backend = VulkanBackend::new().unwrap();
        let swapchain = backend.swapchain();
        assert_eq!(swapchain.width(), 800);
        assert_eq!(swapchain.height(), 600);

        // Access the mutable swapchain field directly for testing
        let result = backend.swapchain.recreate(1920, 1080);
        assert!(result.is_ok());

        let swapchain = backend.swapchain();
        assert_eq!(swapchain.width(), 1920);
        assert_eq!(swapchain.height(), 1080);
    }

    #[test]
    fn test_vulkan_command_buffer_creation() {
        let _cmd_buffer = VulkanCommandBuffer::new();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_vulkan_command_buffer_operations() {
        let mut cmd_buffer = VulkanCommandBuffer::new();
        assert!(cmd_buffer.begin().is_ok());
        assert!(cmd_buffer.clear_color(0.0, 0.0, 0.0, 1.0).is_ok());
        assert!(cmd_buffer.draw(3, 1).is_ok());
        assert!(cmd_buffer.end().is_ok());
    }

    #[test]
    fn test_vulkan_pipeline_creation() {
        let pipeline = VulkanPipeline::new();
        assert_eq!(pipeline.name(), "Vulkan Stub Pipeline");
    }

    #[test]
    fn test_vulkan_pipeline_bind() {
        let mut cmd_buffer = VulkanCommandBuffer::new();
        let pipeline = VulkanPipeline::new();
        assert!(cmd_buffer.bind_pipeline(&pipeline).is_ok());
    }

    #[test]
    fn test_vulkan_resource_creation() {
        let resource = VulkanResource::new();
        assert_eq!(resource.size(), 0);
        assert_eq!(resource.resource_type(), "Vulkan Stub Resource");
    }

    #[test]
    fn test_vulkan_backend_frame_operations() {
        let mut backend = VulkanBackend::new().unwrap();
        assert!(backend.begin_frame().is_ok());
        assert!(backend.end_frame().is_ok());
    }

    #[test]
    fn test_vulkan_backend_resize() {
        let mut backend = VulkanBackend::new().unwrap();
        assert!(backend.resize(1024, 768).is_ok());
    }

    #[test]
    fn test_vulkan_backend_cleanup() {
        let mut backend = VulkanBackend::new().unwrap();
        // Should not panic
        backend.cleanup();
    }

    #[test]
    fn test_vulkan_swapchain_operations() {
        let mut backend = VulkanBackend::new().unwrap();
        assert!(backend.swapchain.acquire_next_image().is_ok());
        assert!(backend.swapchain.present().is_ok());
    }
}
