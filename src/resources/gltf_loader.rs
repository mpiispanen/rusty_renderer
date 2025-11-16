//! GLTF model loading
//!
//! Loads 3D models from GLTF/GLB files.

use crate::scene::{GeometryData, Material, SceneMetadata, SceneObject, Transform, VertexData};
use anyhow::{Context, Result};
use std::path::Path;

/// GLTF loader for loading 3D models
pub struct GltfLoader;

impl GltfLoader {
    /// Load a GLTF file and extract meshes
    ///
    /// Returns a list of objects (one per mesh/primitive) and materials
    pub fn load<P: AsRef<Path>>(
        path: P,
    ) -> Result<(Vec<SceneObject>, Vec<Material>, SceneMetadata)> {
        let path = path.as_ref();

        log::info!("Loading GLTF file: {}", path.display());

        let (gltf, buffers, images) = gltf::import(path)
            .with_context(|| format!("Failed to import GLTF file: {}", path.display()))?;

        let mut objects = Vec::new();
        let mut materials = Vec::new();

        // Load materials first
        for material in gltf.materials() {
            let mat = Self::load_material(&material, &images, path)?;
            materials.push(mat);
        }

        // If no materials, add a default one
        if materials.is_empty() {
            materials.push(Material {
                name: "default".to_string(),
                base_color: [0.8, 0.8, 0.8],
                metallic: 0.0,
                roughness: 0.5,
                diffuse_texture: None,
            });
        }

        // Load meshes from scene graph if available, otherwise load all meshes directly
        if let Some(scene) = gltf.default_scene().or_else(|| gltf.scenes().next()) {
            log::info!("Loading from scene: {}", scene.name().unwrap_or("default"));

            // Traverse scene nodes and load meshes with transforms
            for node in scene.nodes() {
                Self::load_node(&node, &buffers, &Transform::identity(), &mut objects)?;
            }
        } else {
            log::warn!("No scene found, loading meshes directly");

            // Fallback: load meshes without scene graph
            for mesh in gltf.meshes() {
                let mesh_name = mesh.name().unwrap_or("mesh").to_string();

                for (prim_idx, primitive) in mesh.primitives().enumerate() {
                    let obj = Self::load_primitive(
                        &primitive,
                        &buffers,
                        &mesh_name,
                        prim_idx,
                        Transform::default(),
                    )?;
                    objects.push(obj);
                }
            }
        }

        // Create metadata
        let metadata = SceneMetadata {
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("GLTF Scene")
                .to_string(),
            description: format!("Loaded from {}", path.display()),
            author: "GLTF Import".to_string(),
        };

        log::info!(
            "Loaded GLTF: {} objects, {} materials",
            objects.len(),
            materials.len()
        );

        Ok((objects, materials, metadata))
    }

    /// Load a GLTF node and its children recursively
    fn load_node(
        node: &gltf::Node,
        buffers: &[gltf::buffer::Data],
        parent_transform: &Transform,
        objects: &mut Vec<SceneObject>,
    ) -> Result<()> {
        // Get node transform
        let node_transform = Self::extract_transform(node);

        // Compose with parent transform
        let combined_transform = Self::compose_transforms(parent_transform, &node_transform);

        // Load mesh if present
        if let Some(mesh) = node.mesh() {
            let mesh_name = mesh
                .name()
                .unwrap_or_else(|| node.name().unwrap_or("mesh"))
                .to_string();

            for (prim_idx, primitive) in mesh.primitives().enumerate() {
                let obj = Self::load_primitive(
                    &primitive,
                    buffers,
                    &mesh_name,
                    prim_idx,
                    combined_transform,
                )?;
                objects.push(obj);
            }
        }

        // Recursively load children
        for child in node.children() {
            Self::load_node(&child, buffers, &combined_transform, objects)?;
        }

        Ok(())
    }

    /// Extract transform from a glTF node
    fn extract_transform(node: &gltf::Node) -> Transform {
        let (translation, rotation, scale) = node.transform().decomposed();

        // Convert quaternion to Euler angles (simplified - just extract Y rotation for now)
        // For full support, we'd need proper quaternion->euler conversion
        let rotation_y = 2.0
            * (rotation[3] * rotation[1] + rotation[0] * rotation[2])
                .atan2(1.0 - 2.0 * (rotation[1] * rotation[1] + rotation[2] * rotation[2]));

        Transform {
            position: [translation[0], translation[1], translation[2]],
            rotation: [0.0, rotation_y.to_degrees(), 0.0], // Simplified: only Y rotation
            scale: [scale[0], scale[1], scale[2]],
        }
    }

    /// Compose two transforms (parent * child)
    fn compose_transforms(parent: &Transform, child: &Transform) -> Transform {
        // For simplicity, we'll just add positions and rotations, multiply scales
        // A full implementation would use matrix multiplication
        Transform {
            position: [
                parent.position[0] + child.position[0],
                parent.position[1] + child.position[1],
                parent.position[2] + child.position[2],
            ],
            rotation: [
                parent.rotation[0] + child.rotation[0],
                parent.rotation[1] + child.rotation[1],
                parent.rotation[2] + child.rotation[2],
            ],
            scale: [
                parent.scale[0] * child.scale[0],
                parent.scale[1] * child.scale[1],
                parent.scale[2] * child.scale[2],
            ],
        }
    }

    /// Load a GLTF primitive as a mesh
    fn load_primitive(
        primitive: &gltf::Primitive,
        buffers: &[gltf::buffer::Data],
        mesh_name: &str,
        prim_idx: usize,
        transform: Transform,
    ) -> Result<SceneObject> {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        // Read positions
        let positions = reader
            .read_positions()
            .context("GLTF primitive missing positions")?
            .collect::<Vec<_>>();

        // Read normals (or generate default)
        let normals: Vec<[f32; 3]> = if let Some(normals) = reader.read_normals() {
            normals.collect()
        } else {
            // Default normals pointing up
            vec![[0.0, 1.0, 0.0]; positions.len()]
        };

        // Read UVs (or generate default)
        let uvs: Vec<[f32; 2]> = if let Some(uvs) = reader.read_tex_coords(0) {
            uvs.into_f32().collect()
        } else {
            // Default UVs
            vec![[0.0, 0.0]; positions.len()]
        };

        // Read colors (or use white)
        let colors: Vec<[f32; 3]> = if let Some(colors) = reader.read_colors(0) {
            colors.into_rgb_f32().collect()
        } else {
            vec![[1.0, 1.0, 1.0]; positions.len()]
        };

        // Build vertices
        let mut vertices = Vec::new();
        for (i, &position) in positions.iter().enumerate() {
            vertices.push(VertexData {
                position,
                normal: normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]),
                uv: uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                color: colors.get(i).copied().unwrap_or([1.0, 1.0, 1.0]),
            });
        }

        // Handle indices
        if let Some(indices) = reader.read_indices() {
            let indices: Vec<u32> = indices.into_u32().collect();

            // Convert indexed vertices to triangle list
            let mut indexed_vertices = Vec::new();
            for idx in indices {
                indexed_vertices.push(vertices[idx as usize]);
            }
            vertices = indexed_vertices;
        }

        let name = if prim_idx == 0 {
            mesh_name.to_string()
        } else {
            format!("{mesh_name}_{prim_idx}")
        };

        let material_index = primitive.material().index();

        Ok(SceneObject::Mesh {
            name,
            geometry: GeometryData::Inline {
                vertices,
                indices: None, // Already converted to triangle list
            },
            transform,
            material: material_index,
        })
    }

    /// Load a GLTF material
    fn load_material(
        material: &gltf::Material,
        images: &[gltf::image::Data],
        gltf_path: &Path,
    ) -> Result<Material> {
        let pbr = material.pbr_metallic_roughness();

        let base_color = pbr.base_color_factor();
        let base_color = [base_color[0], base_color[1], base_color[2]];

        let metallic = pbr.metallic_factor();
        let roughness = pbr.roughness_factor();

        // Handle embedded textures by extracting them to the cache
        let diffuse_texture = if let Some(texture_info) = pbr.base_color_texture() {
            let texture = texture_info.texture();
            let image = &images[texture.source().index()];

            log::info!(
                "Material has embedded texture: {}x{} (format: {:?})",
                image.width,
                image.height,
                image.format
            );

            // Extract embedded texture to cache directory
            Some(Self::extract_embedded_texture(
                gltf_path,
                material.index(),
                image,
            )?)
        } else {
            None
        };

        Ok(Material {
            name: material.name().unwrap_or("material").to_string(),
            base_color,
            metallic,
            roughness,
            diffuse_texture,
        })
    }

    /// Extract an embedded texture to the cache directory
    fn extract_embedded_texture(
        gltf_path: &Path,
        material_idx: Option<usize>,
        image: &gltf::image::Data,
    ) -> Result<String> {
        use std::fs;

        // Create cache directory next to the GLTF file
        let gltf_dir = gltf_path.parent().unwrap_or(Path::new("."));
        let cache_dir = gltf_dir.join(".gltf_cache");
        fs::create_dir_all(&cache_dir)?;

        // Generate filename based on GLTF name and material index
        let gltf_stem = gltf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("texture");
        let mat_suffix = material_idx.map(|i| format!("_mat{i}")).unwrap_or_default();

        // Determine format extension
        let ext = match image.format {
            gltf::image::Format::R8G8B8A8 | gltf::image::Format::R8G8B8 => "png",
            gltf::image::Format::R16G16B16A16 | gltf::image::Format::R16G16B16 => "png",
            _ => "png",
        };

        let texture_filename = format!("{}{}_{}.{}", gltf_stem, mat_suffix, "basecolor", ext);
        let texture_path = cache_dir.join(&texture_filename);

        // Convert image data to PNG and save
        let img = match image.format {
            gltf::image::Format::R8G8B8A8 => {
                image::RgbaImage::from_raw(image.width, image.height, image.pixels.clone())
                    .context("Failed to create RGBA image")?
            }
            gltf::image::Format::R8G8B8 => {
                // Convert RGB to RGBA
                let mut rgba_data = Vec::with_capacity(image.pixels.len() * 4 / 3);
                for chunk in image.pixels.chunks(3) {
                    rgba_data.extend_from_slice(chunk);
                    rgba_data.push(255); // Alpha = 1.0
                }
                image::RgbaImage::from_raw(image.width, image.height, rgba_data)
                    .context("Failed to create RGB->RGBA image")?
            }
            _ => anyhow::bail!("Unsupported image format: {:?}", image.format),
        };

        img.save(&texture_path)
            .with_context(|| format!("Failed to save texture to {}", texture_path.display()))?;

        log::info!("Extracted embedded texture to: {}", texture_path.display());

        // Return relative path
        Ok(texture_path
            .to_str()
            .context("Invalid texture path")?
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    // We can't easily test without actual GLTF files
    // This would require test fixtures
}
