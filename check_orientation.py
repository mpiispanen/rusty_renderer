#!/usr/bin/env python3
"""Check orientation differences between Vulkan and DirectX rendering."""

from PIL import Image
import numpy as np
import sys

def analyze_image(path):
    """Analyze image orientation by checking key points."""
    img = Image.open(path)
    arr = np.array(img)
    
    print(f"\n{path}:")
    print(f"  Size: {img.size}")
    print(f"  Mode: {img.mode}")
    
    # Sample key positions to check orientation
    h, w = arr.shape[:2]
    
    # Top-left corner (approximate)
    tl = arr[h//4, w//4, :3].tolist()
    # Top-right corner
    tr = arr[h//4, 3*w//4, :3].tolist()
    # Bottom-left corner
    bl = arr[3*h//4, w//4, :3].tolist()
    # Bottom-right corner
    br = arr[3*h//4, 3*w//4, :3].tolist()
    # Center
    center = arr[h//2, w//2, :3].tolist()
    
    print(f"  Top-left: {tl}")
    print(f"  Top-right: {tr}")
    print(f"  Bottom-left: {bl}")
    print(f"  Bottom-right: {br}")
    print(f"  Center: {center}")
    
    return arr

def compare_images(path1, path2):
    """Compare two images to detect flipping."""
    arr1 = analyze_image(path1)
    arr2 = analyze_image(path2)
    
    print("\nComparison:")
    
    # Check if one is vertically flipped relative to the other
    arr2_vflip = np.flipud(arr2)
    diff_normal = np.mean(np.abs(arr1.astype(float) - arr2.astype(float)))
    diff_vflip = np.mean(np.abs(arr1.astype(float) - arr2_vflip.astype(float)))
    
    print(f"  Difference (normal): {diff_normal:.2f}")
    print(f"  Difference (vertical flip): {diff_vflip:.2f}")
    
    if diff_vflip < diff_normal * 0.9:
        print(f"  ⚠️  Image 2 appears to be VERTICALLY FLIPPED relative to image 1")
        # Save comparison
        comp = Image.new('RGB', (arr1.shape[1] * 3, arr1.shape[0]))
        comp.paste(Image.fromarray(arr1), (0, 0))
        comp.paste(Image.fromarray(arr2), (arr1.shape[1], 0))
        comp.paste(Image.fromarray(arr2_vflip), (arr1.shape[1] * 2, 0))
        comp.save('/var/home/matpii01/orientation_comparison.png')
        print(f"  Saved comparison to orientation_comparison.png")
        print(f"    (Left: {path1}, Middle: {path2}, Right: {path2} flipped)")
        return "vflipped"
    else:
        print(f"  ✓ Images have the same orientation")
        return "same"

if __name__ == "__main__":
    vk_path = "/var/home/matpii01/vk_helmet_test_now.png"
    dx_path = "/var/home/matpii01/dx_helmet_test_now.png"
    
    result = compare_images(vk_path, dx_path)
    sys.exit(0 if result == "same" else 1)
