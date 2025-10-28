#!/usr/bin/env python3
from PIL import Image
import numpy as np

def analyze_brightness(img_path, name):
    img = Image.open(img_path).convert('RGB')
    arr = np.array(img)
    
    print(f"\n=== {name} ===")
    print(f"Shape: {arr.shape}")
    print(f"Min values (R,G,B): ({arr[:,:,0].min()}, {arr[:,:,1].min()}, {arr[:,:,2].min()})")
    print(f"Max values (R,G,B): ({arr[:,:,0].max()}, {arr[:,:,1].max()}, {arr[:,:,2].max()})")
    print(f"Mean values (R,G,B): ({arr[:,:,0].mean():.1f}, {arr[:,:,1].mean():.1f}, {arr[:,:,2].mean():.1f})")
    print(f"Median values (R,G,B): ({np.median(arr[:,:,0]):.1f}, {np.median(arr[:,:,1]):.1f}, {np.median(arr[:,:,2]):.1f})")
    
    # Count black pixels
    black_pixels = ((arr[:,:,0] == 0) & (arr[:,:,1] == 0) & (arr[:,:,2] == 0)).sum()
    print(f"Black pixels: {black_pixels}/{arr.shape[0]*arr.shape[1]} ({100*black_pixels/(arr.shape[0]*arr.shape[1]):.1f}%)")
    
    # Histogram of brightness
    brightness = arr.mean(axis=2)
    print(f"Brightness histogram:")
    print(f"  <25:  {(brightness < 25).sum()}")
    print(f"  25-75:  {((brightness >= 25) & (brightness < 75)).sum()}")
    print(f"  75-150: {((brightness >= 75) & (brightness < 150)).sum()}")
    print(f"  150+:   {(brightness >= 150).sum()}")

analyze_brightness("screenshots/local/vulkan/gltf_textured.png", "Vulkan")
analyze_brightness("screenshots/local/directx/gltf_textured.png", "DirectX")
