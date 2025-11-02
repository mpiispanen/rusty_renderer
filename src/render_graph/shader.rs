//! Shader registry and descriptor types
//!
//! This module provides a centralized shader management system for the render graph.
//! Shaders can be registered by name and referenced declaratively in passes.

use std::collections::HashMap;
use thiserror::Error;

/// Shader compilation and loading errors
#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Shader not found: {0}")]
    NotFound(String),

    #[error("Failed to load shader file: {0}")]
    LoadError(String),

    #[error("Shader compilation failed: {0}")]
    CompilationError(String),

    #[error("Invalid shader stage")]
    InvalidStage,

    #[error("Backend-specific error: {0}")]
    BackendError(String),
}

/// Result type for shader operations
pub type Result<T> = std::result::Result<T, ShaderError>;

/// Shader pipeline stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    /// Vertex shader
    Vertex,
    /// Fragment/Pixel shader
    Fragment,
    /// Compute shader
    Compute,
}

/// Source of shader code
///
/// Shaders can come from various sources:
/// - Source files (GLSL, HLSL, etc.)
/// - Pre-compiled SPIR-V or DXIL
/// - Embedded bytecode in the binary
#[derive(Debug, Clone)]
pub enum ShaderSource {
    /// Path to a shader source file
    ///
    /// The file will be compiled at runtime or build time depending on configuration.
    /// Supports GLSL and HLSL based on file extension.
    File(&'static str),

    /// Path to a pre-compiled shader binary
    ///
    /// SPIR-V (.spv) for Vulkan, DXIL (.dxil) for DirectX
    Compiled(&'static str),

    /// Shader bytecode embedded in the binary
    ///
    /// Useful for shipping without external files
    Embedded(&'static [u8]),
}

/// Shader descriptor
///
/// Describes how to load and compile a shader. Used when registering shaders
/// with the shader registry.
///
/// # Example
/// ```ignore
/// let descriptor = ShaderDescriptor {
///     source: ShaderSource::File("shaders/vertex.glsl"),
///     entry_point: "main",
///     stage: ShaderStage::Vertex,
///     backend_compile: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    /// Where to get the shader code
    pub source: ShaderSource,

    /// Entry point function name
    ///
    /// Typically "main" for GLSL, but can be different for HLSL
    pub entry_point: &'static str,

    /// Pipeline stage this shader runs in
    pub stage: ShaderStage,

    /// Whether to compile at runtime using backend
    ///
    /// If true, the shader will be compiled by the graphics backend.
    /// If false, the shader must be pre-compiled.
    pub backend_compile: bool,
}

impl ShaderDescriptor {
    /// Create a new shader descriptor for a source file
    pub fn from_file(path: &'static str, stage: ShaderStage) -> Self {
        Self {
            source: ShaderSource::File(path),
            entry_point: "main",
            stage,
            backend_compile: true,
        }
    }

    /// Create a new shader descriptor for pre-compiled shader
    pub fn from_compiled(path: &'static str, stage: ShaderStage) -> Self {
        Self {
            source: ShaderSource::Compiled(path),
            entry_point: "main",
            stage,
            backend_compile: false,
        }
    }

    /// Create a new shader descriptor for embedded bytecode
    pub fn from_embedded(bytecode: &'static [u8], stage: ShaderStage) -> Self {
        Self {
            source: ShaderSource::Embedded(bytecode),
            entry_point: "main",
            stage,
            backend_compile: false,
        }
    }

    /// Set a custom entry point
    pub fn with_entry_point(mut self, entry_point: &'static str) -> Self {
        self.entry_point = entry_point;
        self
    }

    /// Compile shader to SPIR-V
    ///
    /// This method compiles the shader source to SPIR-V bytecode suitable for Vulkan.
    /// For HLSL shaders, it uses DXC to compile to SPIR-V.
    ///
    /// # Returns
    /// SPIR-V bytecode as a vector of u32 words
    pub fn compile_to_spirv(&self) -> Result<Vec<u32>> {
        match &self.source {
            ShaderSource::Embedded(bytecode) => {
                // Assume embedded bytecode is already SPIR-V
                // Convert from u8 slice to u32 vec
                if bytecode.len() % 4 != 0 {
                    return Err(ShaderError::CompilationError(
                        "Embedded bytecode length not aligned to 4 bytes".to_string(),
                    ));
                }
                let spirv: Vec<u32> = bytecode
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                Ok(spirv)
            }
            ShaderSource::Compiled(path) => {
                // Load pre-compiled SPIR-V
                let bytecode = ShaderRegistry::load_compiled(path)?;
                if bytecode.len() % 4 != 0 {
                    return Err(ShaderError::CompilationError(format!(
                        "Compiled shader {} length not aligned to 4 bytes",
                        path
                    )));
                }
                let spirv: Vec<u32> = bytecode
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                Ok(spirv)
            }
            ShaderSource::File(path) => {
                // Compile from source file
                if !self.backend_compile {
                    return Err(ShaderError::CompilationError(
                        "Shader marked as not backend-compilable but source is a file".to_string(),
                    ));
                }

                // For now, assume all source files are HLSL and use DXC to compile to SPIR-V
                self.compile_hlsl_to_spirv(path)
            }
        }
    }

    /// Compile HLSL source to SPIR-V using DXC
    #[cfg(unix)]
    fn compile_hlsl_to_spirv(&self, path: &str) -> Result<Vec<u32>> {
        use std::process::Command;

        // Determine shader profile based on stage
        let profile = match self.stage {
            ShaderStage::Vertex => "vs_6_0",
            ShaderStage::Fragment => "ps_6_0",
            ShaderStage::Compute => "cs_6_0",
        };

        // Create output path for SPIR-V
        let output_path = format!("{}.spv", path);

        // Run DXC to compile HLSL to SPIR-V
        let output = Command::new("dxc")
            .arg("-spirv")
            .arg("-fspv-target-env=vulkan1.2")
            .arg("-fvk-use-gl-layout")
            .arg("-T")
            .arg(profile)
            .arg("-E")
            .arg(self.entry_point)
            .arg(path)
            .arg("-Fo")
            .arg(&output_path)
            .output()
            .map_err(|e| ShaderError::CompilationError(format!("Failed to run DXC: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ShaderError::CompilationError(format!(
                "DXC compilation failed for {}: {}",
                path, stderr
            )));
        }

        // Load the compiled SPIR-V
        let bytecode = std::fs::read(&output_path).map_err(|e| {
            ShaderError::LoadError(format!("Failed to load compiled SPIR-V: {}", e))
        })?;

        if bytecode.len() % 4 != 0 {
            return Err(ShaderError::CompilationError(
                "Compiled SPIR-V length not aligned to 4 bytes".to_string(),
            ));
        }

        let spirv: Vec<u32> = bytecode
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        // Clean up temporary file
        let _ = std::fs::remove_file(&output_path);

        Ok(spirv)
    }

    #[cfg(windows)]
    fn compile_hlsl_to_spirv(&self, path: &str) -> Result<Vec<u32>> {
        // On Windows, use DXC to compile HLSL to SPIR-V
        use std::process::Command;

        let output_path = format!("{}.spv", path);

        let output = Command::new("dxc")
            .args(&[
                "-spirv",
                "-T",
                "vs_6_0", // Will need to determine this from shader type
                "-E",
                "main",
                path,
                "-Fo",
                &output_path,
            ])
            .output()
            .map_err(|e| ShaderError::CompilationError(format!("Failed to execute dxc: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ShaderError::CompilationError(format!(
                "DXC compilation failed: {}",
                error
            )));
        }

        // Read the compiled SPIR-V
        let spirv_bytes = std::fs::read(&output_path).map_err(|e| {
            ShaderError::LoadError(format!("Failed to read compiled SPIR-V: {}", e))
        })?;

        // Convert bytes to u32 words
        let spirv = spirv_bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok(spirv)
    }
}

/// Shader handle for referencing registered shaders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderHandle(pub usize);

/// Compiled shader data
///
/// Internal representation of a compiled shader. The actual format
/// depends on the backend (SPIR-V for Vulkan, DXIL for DirectX).
#[derive(Debug, Clone)]
pub struct CompiledShader {
    /// Shader descriptor
    pub descriptor: ShaderDescriptor,

    /// Compiled bytecode (SPIR-V or DXIL)
    pub bytecode: Vec<u8>,

    /// Backend-specific metadata
    #[allow(dead_code)]
    metadata: HashMap<String, String>,
}

/// Shader registry
///
/// Central registry for all shaders in the application. Shaders are registered
/// by name and can be looked up for use in pipelines.
///
/// # Example
/// ```ignore
/// let mut registry = ShaderRegistry::new();
///
/// registry.register(
///     "forward_vertex",
///     ShaderDescriptor::from_file("shaders/forward.vert", ShaderStage::Vertex)
/// );
///
/// let shader = registry.get("forward_vertex")?;
/// ```
pub struct ShaderRegistry {
    /// Registered shader descriptors by name
    shaders: HashMap<String, ShaderDescriptor>,

    /// Compiled shader cache
    compiled_cache: HashMap<String, CompiledShader>,

    /// Name to handle mapping
    name_to_handle: HashMap<String, ShaderHandle>,

    /// Next handle ID
    next_handle: usize,
}

impl ShaderRegistry {
    /// Create a new empty shader registry
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
            compiled_cache: HashMap::new(),
            name_to_handle: HashMap::new(),
            next_handle: 0,
        }
    }

    /// Register a shader by name
    ///
    /// # Arguments
    /// * `name` - Unique identifier for the shader
    /// * `descriptor` - Shader descriptor defining source and compilation
    ///
    /// # Returns
    /// A handle to the registered shader
    ///
    /// # Example
    /// ```ignore
    /// let handle = registry.register(
    ///     "my_shader",
    ///     ShaderDescriptor::from_file("shaders/my_shader.glsl", ShaderStage::Vertex)
    /// );
    /// ```
    pub fn register(
        &mut self,
        name: impl Into<String>,
        descriptor: ShaderDescriptor,
    ) -> ShaderHandle {
        let name = name.into();
        let handle = ShaderHandle(self.next_handle);
        self.next_handle += 1;

        self.shaders.insert(name.clone(), descriptor);
        self.name_to_handle.insert(name, handle);

        handle
    }

    /// Get a shader descriptor by name
    ///
    /// # Arguments
    /// * `name` - Name of the registered shader
    ///
    /// # Returns
    /// The shader descriptor if found
    pub fn get(&self, name: &str) -> Result<&ShaderDescriptor> {
        self.shaders
            .get(name)
            .ok_or_else(|| ShaderError::NotFound(name.to_string()))
    }

    /// Get a shader handle by name
    ///
    /// # Arguments
    /// * `name` - Name of the registered shader
    ///
    /// # Returns
    /// The shader handle if found
    pub fn get_handle(&self, name: &str) -> Result<ShaderHandle> {
        self.name_to_handle
            .get(name)
            .copied()
            .ok_or_else(|| ShaderError::NotFound(name.to_string()))
    }

    /// Get a shader descriptor by handle
    pub fn get_by_handle(&self, handle: ShaderHandle) -> Result<&ShaderDescriptor> {
        // Find the name associated with this handle
        let name = self
            .name_to_handle
            .iter()
            .find(|(_, &h)| h == handle)
            .map(|(n, _)| n)
            .ok_or(ShaderError::NotFound(format!("handle {:?}", handle)))?;

        self.get(name)
    }

    /// Load shader source from file
    ///
    /// Reads the shader source code from the filesystem.
    ///
    /// # Arguments
    /// * `path` - Path to the shader file
    ///
    /// # Returns
    /// The shader source code as a string
    pub fn load_source(path: &str) -> Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| ShaderError::LoadError(format!("{}: {}", path, e)))
    }

    /// Load pre-compiled shader bytecode
    ///
    /// Reads compiled shader bytecode from the filesystem.
    ///
    /// # Arguments
    /// * `path` - Path to the compiled shader file (.spv, .dxil)
    ///
    /// # Returns
    /// The shader bytecode
    pub fn load_compiled(path: &str) -> Result<Vec<u8>> {
        std::fs::read(path).map_err(|e| ShaderError::LoadError(format!("{}: {}", path, e)))
    }

    /// Cache compiled shader
    ///
    /// Store a compiled shader in the cache for faster subsequent access.
    ///
    /// # Arguments
    /// * `name` - Name of the shader
    /// * `compiled` - Compiled shader data
    pub fn cache_compiled(&mut self, name: impl Into<String>, compiled: CompiledShader) {
        self.compiled_cache.insert(name.into(), compiled);
    }

    /// Get cached compiled shader
    ///
    /// Retrieve a previously compiled shader from the cache.
    ///
    /// # Arguments
    /// * `name` - Name of the shader
    ///
    /// # Returns
    /// The compiled shader if cached
    pub fn get_cached(&self, name: &str) -> Option<&CompiledShader> {
        self.compiled_cache.get(name)
    }

    /// Check if a shader is registered
    pub fn contains(&self, name: &str) -> bool {
        self.shaders.contains_key(name)
    }

    /// Get all registered shader names
    pub fn shader_names(&self) -> impl Iterator<Item = &String> {
        self.shaders.keys()
    }

    /// Clear the compiled shader cache
    ///
    /// Useful for forcing recompilation or freeing memory
    pub fn clear_cache(&mut self) {
        self.compiled_cache.clear();
    }

    /// Get number of registered shaders
    pub fn len(&self) -> usize {
        self.shaders.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.shaders.is_empty()
    }
}

impl Default for ShaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CompiledShader {
    /// Create a new compiled shader
    pub fn new(descriptor: ShaderDescriptor, bytecode: Vec<u8>) -> Self {
        Self {
            descriptor,
            bytecode,
            metadata: HashMap::new(),
        }
    }

    /// Get the shader bytecode
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// Get the shader stage
    pub fn stage(&self) -> ShaderStage {
        self.descriptor.stage
    }

    /// Get the entry point
    pub fn entry_point(&self) -> &str {
        self.descriptor.entry_point
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_registry_basic() {
        let mut registry = ShaderRegistry::new();

        let handle = registry.register(
            "test_shader",
            ShaderDescriptor::from_file("test.glsl", ShaderStage::Vertex),
        );

        assert!(registry.contains("test_shader"));
        assert_eq!(registry.len(), 1);

        let shader = registry.get("test_shader").unwrap();
        assert_eq!(shader.stage, ShaderStage::Vertex);
        assert_eq!(shader.entry_point, "main");

        let handle2 = registry.get_handle("test_shader").unwrap();
        assert_eq!(handle, handle2);
    }

    #[test]
    fn test_shader_not_found() {
        let registry = ShaderRegistry::new();
        assert!(registry.get("nonexistent").is_err());
    }

    #[test]
    fn test_shader_descriptor_builders() {
        let from_file = ShaderDescriptor::from_file("test.vert", ShaderStage::Vertex);
        assert_eq!(from_file.entry_point, "main");
        assert!(from_file.backend_compile);

        let from_compiled = ShaderDescriptor::from_compiled("test.spv", ShaderStage::Fragment);
        assert!(!from_compiled.backend_compile);

        let with_entry = ShaderDescriptor::from_file("test.comp", ShaderStage::Compute)
            .with_entry_point("compute_main");
        assert_eq!(with_entry.entry_point, "compute_main");
    }

    #[test]
    fn test_compiled_shader_cache() {
        let mut registry = ShaderRegistry::new();

        registry.register(
            "cached",
            ShaderDescriptor::from_file("test.glsl", ShaderStage::Vertex),
        );

        let bytecode = vec![0x03, 0x02, 0x23, 0x07]; // Fake SPIR-V magic number
        let descriptor = registry.get("cached").unwrap().clone();
        let compiled = CompiledShader::new(descriptor, bytecode.clone());

        registry.cache_compiled("cached", compiled);

        let cached = registry.get_cached("cached").unwrap();
        assert_eq!(cached.bytecode(), &bytecode);
        assert_eq!(cached.stage(), ShaderStage::Vertex);
    }

    #[test]
    fn test_multiple_shaders() {
        let mut registry = ShaderRegistry::new();

        registry.register(
            "vert",
            ShaderDescriptor::from_file("test.vert", ShaderStage::Vertex),
        );
        registry.register(
            "frag",
            ShaderDescriptor::from_file("test.frag", ShaderStage::Fragment),
        );
        registry.register(
            "comp",
            ShaderDescriptor::from_file("test.comp", ShaderStage::Compute),
        );

        assert_eq!(registry.len(), 3);

        let names: Vec<_> = registry.shader_names().collect();
        assert!(names.contains(&&"vert".to_string()));
        assert!(names.contains(&&"frag".to_string()));
        assert!(names.contains(&&"comp".to_string()));
    }

    #[test]
    fn test_shader_source_types() {
        let file_source = ShaderSource::File("test.glsl");
        let compiled_source = ShaderSource::Compiled("test.spv");
        let embedded_source = ShaderSource::Embedded(&[1, 2, 3, 4]);

        match file_source {
            ShaderSource::File(path) => assert_eq!(path, "test.glsl"),
            _ => panic!("Wrong variant"),
        }

        match compiled_source {
            ShaderSource::Compiled(path) => assert_eq!(path, "test.spv"),
            _ => panic!("Wrong variant"),
        }

        match embedded_source {
            ShaderSource::Embedded(data) => assert_eq!(data, &[1, 2, 3, 4]),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_clear_cache() {
        let mut registry = ShaderRegistry::new();

        registry.register(
            "test",
            ShaderDescriptor::from_file("test.glsl", ShaderStage::Vertex),
        );

        let descriptor = registry.get("test").unwrap().clone();
        let compiled = CompiledShader::new(descriptor, vec![1, 2, 3]);
        registry.cache_compiled("test", compiled);

        assert!(registry.get_cached("test").is_some());

        registry.clear_cache();

        assert!(registry.get_cached("test").is_none());
        assert!(registry.contains("test")); // Registry still has the descriptor
    }

    #[test]
    fn test_get_by_handle() {
        let mut registry = ShaderRegistry::new();

        let handle = registry.register(
            "test",
            ShaderDescriptor::from_file("test.glsl", ShaderStage::Vertex),
        );

        let descriptor = registry.get_by_handle(handle).unwrap();
        assert_eq!(descriptor.stage, ShaderStage::Vertex);
    }
}
