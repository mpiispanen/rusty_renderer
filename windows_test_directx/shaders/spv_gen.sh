#!/bin/bash
# This script generates SPIR-V from GLSL shaders
# Requires glslc (from Vulkan SDK) or glslangValidator

if command -v glslc &> /dev/null; then
    glslc triangle.vert -o triangle.vert.spv
    glslc triangle.frag -o triangle.frag.spv
    echo "Shaders compiled with glslc"
elif command -v glslangValidator &> /dev/null; then
    glslangValidator -V triangle.vert -o triangle.vert.spv
    glslangValidator -V triangle.frag -o triangle.frag.spv
    echo "Shaders compiled with glslangValidator"
else
    echo "No GLSL compiler found. Install Vulkan SDK or use online compiler."
    echo "Visit: https://shader-playground.timjones.io/"
    exit 1
fi
