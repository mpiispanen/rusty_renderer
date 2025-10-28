#!/usr/bin/env python3
from PIL import Image
import numpy as np

def analyze_regions(img1_path, img2_path):
    img1 = Image.open(img1_path).convert('RGB')
    img2 = Image.open(img2_path).convert('RGB')
    
    arr1 = np.array(img1)
    arr2 = np.array(img2)
    
    diff = np.abs(arr1.astype(int) - arr2.astype(int))
    
    # Analyze different regions
    print("=== Region Analysis ===")
    
    # Background (top-left corner)
    print("\nBackground (0-100, 0-100):")
    region1 = arr1[0:100, 0:100]
    region2 = arr2[0:100, 0:100]
    print(f"  Vulkan mean:  {region1.mean(axis=(0,1))}")
    print(f"  DirectX mean: {region2.mean(axis=(0,1))}")
    print(f"  Max diff: {np.abs(region1.astype(int) - region2.astype(int)).max()}")
    
    # Cube area (likely center)
    print("\nCube center area (250-350, 350-450):")
    region1 = arr1[250:350, 350:450]
    region2 = arr2[250:350, 350:450]
    print(f"  Vulkan mean:  {region1.mean(axis=(0,1))}")
    print(f"  DirectX mean: {region2.mean(axis=(0,1))}")
    print(f"  Max diff: {np.abs(region1.astype(int) - region2.astype(int)).max()}")
    
    # Create a heatmap of differences
    diff_max = diff.max(axis=2)
    
    print("\n=== Difference Distribution ===")
    for threshold in [1, 5, 10, 25, 50, 100]:
        count = (diff_max > threshold).sum()
        print(f"Pixels with >{threshold:3d} diff: {count:6d} ({100*count/diff_max.size:5.2f}%)")
    
    # Find regions with largest differences
    print("\n=== Largest Differences (sample) ===")
    flat_idx = np.argsort(diff_max.flatten())[-10:][::-1]
    for idx in flat_idx[:5]:
        y, x = np.unravel_index(idx, diff_max.shape)
        print(f"  Pos [{y:3d},{x:3d}]: Vulkan={arr1[y,x]}, DirectX={arr2[y,x]}, Diff={diff[y,x]}")

if __name__ == "__main__":
    analyze_regions("screenshots/local/vulkan/gltf_textured.png", 
                    "screenshots/local/directx/gltf_textured.png")
