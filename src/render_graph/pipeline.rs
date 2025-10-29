//! Pipeline configuration and builder
//!
//! Provides a declarative API for configuring graphics and compute pipelines.
//! Integrates with the shader registry for shader management.

use super::shader::ShaderHandle;
use std::fmt;

/// Polygon rasterization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

/// Face culling mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    None,
    Front,
    Back,
    FrontAndBack,
}

/// Front face winding order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

/// Vertex attribute description
#[derive(Debug, Clone, PartialEq)]
pub struct VertexAttribute {
    /// Attribute location in shader
    pub location: u32,
    /// Format of the attribute
    pub format: VertexFormat,
    /// Byte offset within the vertex
    pub offset: u32,
}

/// Vertex attribute formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
}

impl VertexFormat {
    /// Get the size in bytes of this format
    pub fn size(&self) -> u32 {
        match self {
            VertexFormat::Float32 | VertexFormat::Sint32 | VertexFormat::Uint32 => 4,
            VertexFormat::Float32x2 | VertexFormat::Sint32x2 | VertexFormat::Uint32x2 => 8,
            VertexFormat::Float32x3 | VertexFormat::Sint32x3 | VertexFormat::Uint32x3 => 12,
            VertexFormat::Float32x4 | VertexFormat::Sint32x4 | VertexFormat::Uint32x4 => 16,
        }
    }
}

/// Vertex input binding
#[derive(Debug, Clone, PartialEq)]
pub struct VertexBinding {
    /// Binding index
    pub binding: u32,
    /// Stride in bytes between consecutive elements
    pub stride: u32,
    /// Input rate (per-vertex or per-instance)
    pub input_rate: InputRate,
}

/// Vertex input rate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputRate {
    Vertex,
    Instance,
}

/// Complete vertex input layout
#[derive(Debug, Clone, PartialEq)]
pub struct VertexLayout {
    /// Vertex bindings
    pub bindings: Vec<VertexBinding>,
    /// Vertex attributes
    pub attributes: Vec<VertexAttribute>,
}

impl VertexLayout {
    /// Create a new empty vertex layout
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Add a binding
    pub fn add_binding(&mut self, binding: VertexBinding) -> &mut Self {
        self.bindings.push(binding);
        self
    }

    /// Add an attribute
    pub fn add_attribute(&mut self, attribute: VertexAttribute) -> &mut Self {
        self.attributes.push(attribute);
        self
    }
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Depth test and write configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepthState {
    /// Enable depth testing
    pub test_enable: bool,
    /// Enable depth writes
    pub write_enable: bool,
    /// Comparison function
    pub compare_op: CompareOp,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            test_enable: false,
            write_enable: false,
            compare_op: CompareOp::Less,
        }
    }
}

/// Comparison operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

/// Rasterizer state configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RasterizerState {
    /// Polygon rasterization mode
    pub polygon_mode: PolygonMode,
    /// Face culling mode
    pub cull_mode: CullMode,
    /// Front face winding order
    pub front_face: FrontFace,
    /// Line width (for wireframe mode)
    pub line_width: f32,
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            polygon_mode: PolygonMode::Fill,
            cull_mode: CullMode::None,
            front_face: FrontFace::CounterClockwise,
            line_width: 1.0,
        }
    }
}

/// Blend configuration for a color attachment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendState {
    /// Enable blending
    pub blend_enable: bool,
    /// Source color blend factor
    pub src_color_blend_factor: BlendFactor,
    /// Destination color blend factor
    pub dst_color_blend_factor: BlendFactor,
    /// Color blend operation
    pub color_blend_op: BlendOp,
    /// Source alpha blend factor
    pub src_alpha_blend_factor: BlendFactor,
    /// Destination alpha blend factor
    pub dst_alpha_blend_factor: BlendFactor,
    /// Alpha blend operation
    pub alpha_blend_op: BlendOp,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            blend_enable: false,
            src_color_blend_factor: BlendFactor::One,
            dst_color_blend_factor: BlendFactor::Zero,
            color_blend_op: BlendOp::Add,
            src_alpha_blend_factor: BlendFactor::One,
            dst_alpha_blend_factor: BlendFactor::Zero,
            alpha_blend_op: BlendOp::Add,
        }
    }
}

/// Blend factors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
}

/// Blend operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

/// Complete pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineDescriptor {
    /// Shader handles (vertex, fragment, compute, etc.)
    pub shaders: Vec<ShaderHandle>,
    /// Vertex input layout (for graphics pipelines)
    pub vertex_layout: Option<VertexLayout>,
    /// Depth/stencil state
    pub depth_state: DepthState,
    /// Rasterizer state
    pub rasterizer_state: RasterizerState,
    /// Blend states for color attachments
    pub blend_states: Vec<BlendState>,
}

/// Builder for pipeline configuration
pub struct PipelineBuilder {
    shaders: Vec<ShaderHandle>,
    vertex_layout: Option<VertexLayout>,
    depth_state: DepthState,
    rasterizer_state: RasterizerState,
    blend_states: Vec<BlendState>,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self {
            shaders: Vec::new(),
            vertex_layout: None,
            depth_state: DepthState::default(),
            rasterizer_state: RasterizerState::default(),
            blend_states: Vec::new(),
        }
    }

    /// Add a vertex shader
    pub fn vertex_shader(&mut self, handle: ShaderHandle) -> &mut Self {
        self.shaders.push(handle);
        self
    }

    /// Add a fragment shader
    pub fn fragment_shader(&mut self, handle: ShaderHandle) -> &mut Self {
        self.shaders.push(handle);
        self
    }

    /// Add a compute shader
    pub fn compute_shader(&mut self, handle: ShaderHandle) -> &mut Self {
        self.shaders.push(handle);
        self
    }

    /// Set vertex layout
    pub fn vertex_layout(&mut self, layout: VertexLayout) -> &mut Self {
        self.vertex_layout = Some(layout);
        self
    }

    /// Enable/disable depth testing
    pub fn depth_test(&mut self, enable: bool) -> &mut Self {
        self.depth_state.test_enable = enable;
        self
    }

    /// Enable/disable depth writes
    pub fn depth_write(&mut self, enable: bool) -> &mut Self {
        self.depth_state.write_enable = enable;
        self
    }

    /// Set depth comparison operation
    pub fn depth_compare(&mut self, op: CompareOp) -> &mut Self {
        self.depth_state.compare_op = op;
        self
    }

    /// Set cull mode
    pub fn cull_mode(&mut self, mode: CullMode) -> &mut Self {
        self.rasterizer_state.cull_mode = mode;
        self
    }

    /// Set polygon mode
    pub fn polygon_mode(&mut self, mode: PolygonMode) -> &mut Self {
        self.rasterizer_state.polygon_mode = mode;
        self
    }

    /// Set front face winding
    pub fn front_face(&mut self, face: FrontFace) -> &mut Self {
        self.rasterizer_state.front_face = face;
        self
    }

    /// Set line width (for wireframe mode)
    pub fn line_width(&mut self, width: f32) -> &mut Self {
        self.rasterizer_state.line_width = width;
        self
    }

    /// Add a blend state for a color attachment
    pub fn blend_state(&mut self, state: BlendState) -> &mut Self {
        self.blend_states.push(state);
        self
    }

    /// Get the list of shader handles
    pub fn shaders(&self) -> &[ShaderHandle] {
        &self.shaders
    }

    /// Get the vertex layout
    pub fn get_vertex_layout(&self) -> Option<&VertexLayout> {
        self.vertex_layout.as_ref()
    }

    /// Get the depth state
    pub fn get_depth_state(&self) -> &DepthState {
        &self.depth_state
    }

    /// Get the rasterizer state
    pub fn get_rasterizer_state(&self) -> &RasterizerState {
        &self.rasterizer_state
    }

    /// Get the blend states
    pub fn get_blend_states(&self) -> &[BlendState] {
        &self.blend_states
    }

    /// Build the pipeline descriptor
    pub fn build(self) -> PipelineDescriptor {
        PipelineDescriptor {
            shaders: self.shaders,
            vertex_layout: self.vertex_layout,
            depth_state: self.depth_state,
            rasterizer_state: self.rasterizer_state,
            blend_states: self.blend_states,
        }
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PipelineBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PipelineBuilder")
            .field("shaders", &self.shaders)
            .field("vertex_layout", &self.vertex_layout)
            .field("depth_state", &self.depth_state)
            .field("rasterizer_state", &self.rasterizer_state)
            .field("blend_states", &self.blend_states)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_format_sizes() {
        assert_eq!(VertexFormat::Float32.size(), 4);
        assert_eq!(VertexFormat::Float32x2.size(), 8);
        assert_eq!(VertexFormat::Float32x3.size(), 12);
        assert_eq!(VertexFormat::Float32x4.size(), 16);
    }

    #[test]
    fn test_vertex_layout() {
        let mut layout = VertexLayout::new();
        layout.add_binding(VertexBinding {
            binding: 0,
            stride: 24,
            input_rate: InputRate::Vertex,
        });
        layout.add_attribute(VertexAttribute {
            location: 0,
            format: VertexFormat::Float32x3,
            offset: 0,
        });
        layout.add_attribute(VertexAttribute {
            location: 1,
            format: VertexFormat::Float32x3,
            offset: 12,
        });

        assert_eq!(layout.bindings.len(), 1);
        assert_eq!(layout.attributes.len(), 2);
    }

    #[test]
    fn test_pipeline_builder_fluent() {
        let shader = ShaderHandle(0);
        let mut builder = PipelineBuilder::new();

        builder
            .vertex_shader(shader)
            .fragment_shader(shader)
            .depth_test(true)
            .depth_write(true)
            .cull_mode(CullMode::Back);

        let desc = builder.build();
        assert_eq!(desc.shaders.len(), 2);
        assert!(desc.depth_state.test_enable);
        assert!(desc.depth_state.write_enable);
        assert_eq!(desc.rasterizer_state.cull_mode, CullMode::Back);
    }

    #[test]
    fn test_default_states() {
        let depth = DepthState::default();
        assert!(!depth.test_enable);
        assert!(!depth.write_enable);
        assert_eq!(depth.compare_op, CompareOp::Less);

        let rasterizer = RasterizerState::default();
        assert_eq!(rasterizer.polygon_mode, PolygonMode::Fill);
        assert_eq!(rasterizer.cull_mode, CullMode::None);
        assert_eq!(rasterizer.front_face, FrontFace::CounterClockwise);
        assert_eq!(rasterizer.line_width, 1.0);

        let blend = BlendState::default();
        assert!(!blend.blend_enable);
    }

    #[test]
    fn test_pipeline_descriptor_clone() {
        let shader = ShaderHandle(0);
        let desc = {
            let mut builder = PipelineBuilder::new();
            builder.vertex_shader(shader).fragment_shader(shader);
            builder.build()
        };

        let cloned = desc.clone();
        assert_eq!(desc.shaders.len(), cloned.shaders.len());
    }
}
