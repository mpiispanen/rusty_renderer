#!/usr/bin/env python3
"""
Generate a simple GLTF cube for testing.
This creates a minimal but valid GLTF 2.0 file.
"""

import json
import struct
import base64

def create_cube_gltf():
    """Create a simple textured cube GLTF file"""
    
    # Cube vertices (8 corners)
    # We need 24 vertices (4 per face) for proper normals and UVs
    vertices = []
    normals = []
    uvs = []
    indices = []
    
    # Front face (Z+)
    vertices.extend([
        [-0.5, -0.5,  0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5],
    ])
    normals.extend([[0.0, 0.0, 1.0]] * 4)
    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    indices.extend([0, 1, 2, 0, 2, 3])
    
    # Back face (Z-)
    vertices.extend([
        [ 0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [-0.5,  0.5, -0.5], [ 0.5,  0.5, -0.5],
    ])
    normals.extend([[0.0, 0.0, -1.0]] * 4)
    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    indices.extend([4, 5, 6, 4, 6, 7])
    
    # Right face (X+)
    vertices.extend([
        [ 0.5, -0.5,  0.5], [ 0.5, -0.5, -0.5], [ 0.5,  0.5, -0.5], [ 0.5,  0.5,  0.5],
    ])
    normals.extend([[1.0, 0.0, 0.0]] * 4)
    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    indices.extend([8, 9, 10, 8, 10, 11])
    
    # Left face (X-)
    vertices.extend([
        [-0.5, -0.5, -0.5], [-0.5, -0.5,  0.5], [-0.5,  0.5,  0.5], [-0.5,  0.5, -0.5],
    ])
    normals.extend([[-1.0, 0.0, 0.0]] * 4)
    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    indices.extend([12, 13, 14, 12, 14, 15])
    
    # Top face (Y+)
    vertices.extend([
        [-0.5,  0.5,  0.5], [ 0.5,  0.5,  0.5], [ 0.5,  0.5, -0.5], [-0.5,  0.5, -0.5],
    ])
    normals.extend([[0.0, 1.0, 0.0]] * 4)
    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    indices.extend([16, 17, 18, 16, 18, 19])
    
    # Bottom face (Y-)
    vertices.extend([
        [-0.5, -0.5, -0.5], [ 0.5, -0.5, -0.5], [ 0.5, -0.5,  0.5], [-0.5, -0.5,  0.5],
    ])
    normals.extend([[0.0, -1.0, 0.0]] * 4)
    uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    indices.extend([20, 21, 22, 20, 22, 23])
    
    # Pack data into binary buffers
    vertex_data = b''.join(struct.pack('fff', *v) for v in vertices)
    normal_data = b''.join(struct.pack('fff', *n) for n in normals)
    uv_data = b''.join(struct.pack('ff', *uv) for uv in uvs)
    index_data = b''.join(struct.pack('H', i) for i in indices)
    
    # Combine all buffers
    buffer_data = vertex_data + normal_data + uv_data + index_data
    
    # Calculate offsets
    vertex_offset = 0
    vertex_length = len(vertex_data)
    normal_offset = vertex_length
    normal_length = len(normal_data)
    uv_offset = normal_offset + normal_length
    uv_length = len(uv_data)
    index_offset = uv_offset + uv_length
    index_length = len(index_data)
    
    # Create GLTF structure
    gltf = {
        "asset": {
            "version": "2.0",
            "generator": "Rusty Renderer GLTF Generator"
        },
        "scene": 0,
        "scenes": [
            {
                "nodes": [0]
            }
        ],
        "nodes": [
            {
                "mesh": 0
            }
        ],
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 0,
                            "NORMAL": 1,
                            "TEXCOORD_0": 2
                        },
                        "indices": 3,
                        "material": 0
                    }
                ]
            }
        ],
        "materials": [
            {
                "pbrMetallicRoughness": {
                    "baseColorFactor": [0.8, 0.8, 0.8, 1.0],
                    "metallicFactor": 0.0,
                    "roughnessFactor": 0.5
                },
                "name": "CubeMaterial"
            }
        ],
        "buffers": [
            {
                "byteLength": len(buffer_data),
                "uri": "data:application/octet-stream;base64," + base64.b64encode(buffer_data).decode('ascii')
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": vertex_offset,
                "byteLength": vertex_length,
                "target": 34962  # ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": normal_offset,
                "byteLength": normal_length,
                "target": 34962  # ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": uv_offset,
                "byteLength": uv_length,
                "target": 34962  # ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": index_offset,
                "byteLength": index_length,
                "target": 34963  # ELEMENT_ARRAY_BUFFER
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "byteOffset": 0,
                "componentType": 5126,  # FLOAT
                "count": len(vertices),
                "type": "VEC3",
                "max": [0.5, 0.5, 0.5],
                "min": [-0.5, -0.5, -0.5]
            },
            {
                "bufferView": 1,
                "byteOffset": 0,
                "componentType": 5126,  # FLOAT
                "count": len(normals),
                "type": "VEC3"
            },
            {
                "bufferView": 2,
                "byteOffset": 0,
                "componentType": 5126,  # FLOAT
                "count": len(uvs),
                "type": "VEC2"
            },
            {
                "bufferView": 3,
                "byteOffset": 0,
                "componentType": 5123,  # UNSIGNED_SHORT
                "count": len(indices),
                "type": "SCALAR"
            }
        ]
    }
    
    return gltf

if __name__ == "__main__":
    import sys
    import os
    
    # Determine output path
    if len(sys.argv) > 1:
        output_path = sys.argv[1]
    else:
        # Default to assets/models/cube.gltf
        script_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(script_dir)
        output_path = os.path.join(project_root, "assets", "models", "cube.gltf")
    
    # Create directory if needed
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    # Generate and save
    gltf = create_cube_gltf()
    with open(output_path, 'w') as f:
        json.dump(gltf, f, indent=2)
    
    print(f"✓ Generated GLTF cube: {output_path}")
    print(f"  24 vertices, 36 indices (12 triangles)")
