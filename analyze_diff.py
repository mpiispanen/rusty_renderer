#!/usr/bin/env python3
from PIL import Image
import numpy as np
import sys

def analyze_diff(img1_path, img2_path):
    img1 = Image.open(img1_path).convert('RGB')
    img2 = Image.open(img2_path).convert('RGB')
    
    arr1 = np.array(img1)
    arr2 = np.array(img2)
    
    diff = np.abs(arr1.astype(int) - arr2.astype(int))
    
    print(f"Image 1: {img1.size}")
    print(f"Image 2: {img2.size}")
    print(f"Max difference per channel: R={diff[:,:,0].max()}, G={diff[:,:,1].max()}, B={diff[:,:,2].max()}")
    print(f"Mean difference per channel: R={diff[:,:,0].mean():.2f}, G={diff[:,:,1].mean():.2f}, B={diff[:,:,2].mean():.2f}")
    print(f"Pixels with >10 difference: {(diff.max(axis=2) > 10).sum()}/{diff.shape[0]*diff.shape[1]} ({100*(diff.max(axis=2) > 10).sum()/(diff.shape[0]*diff.shape[1]):.1f}%)")
    
    # Sample some pixels
    print("\nSample pixel values (center, middle):")
    y, x = img1.size[1]//2, img1.size[0]//2
    print(f"  Vulkan  [{y},{x}]: {arr1[y, x]}")
    print(f"  DirectX [{y},{x}]: {arr2[y, x]}")
    print(f"  Diff: {diff[y, x]}")
    
    # Top-left corner of cube (likely textured area)
    y, x = 200, 300
    print(f"\nTextured area [{y},{x}]:")
    print(f"  Vulkan  : {arr1[y, x]}")
    print(f"  DirectX : {arr2[y, x]}")
    print(f"  Diff: {diff[y, x]}")

if __name__ == "__main__":
    analyze_diff("screenshots/local/vulkan/gltf_textured.png", 
                 "screenshots/local/directx/gltf_textured.png")
