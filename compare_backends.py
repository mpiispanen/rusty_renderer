#!/usr/bin/env python3
"""Compare rendering outputs from Vulkan and DirectX backends."""

import sys
from PIL import Image, ImageChops, ImageDraw, ImageFont
import numpy as np

def compare_images(vk_path, dx_path, output_path):
    """Compare two images and create a side-by-side comparison."""
    
    # Load images
    vk_img = Image.open(vk_path).convert('RGB')
    dx_img = Image.open(dx_path).convert('RGB')
    
    # Ensure same size
    if vk_img.size != dx_img.size:
        print(f"Warning: Image sizes differ: VK={vk_img.size}, DX={dx_img.size}")
        dx_img = dx_img.resize(vk_img.size)
    
    # Calculate difference
    diff = ImageChops.difference(vk_img, dx_img)
    
    # Convert to numpy for statistics
    vk_arr = np.array(vk_img)
    dx_arr = np.array(dx_img)
    diff_arr = np.array(diff)
    
    # Calculate metrics
    max_diff = np.max(diff_arr)
    mean_diff = np.mean(diff_arr)
    nonzero_pixels = np.count_nonzero(diff_arr)
    total_pixels = diff_arr.shape[0] * diff_arr.shape[1] * diff_arr.shape[2]
    percent_diff = (nonzero_pixels / total_pixels) * 100
    
    # Check if images are upside down relative to each other
    vk_flipped = vk_img.transpose(Image.FLIP_TOP_BOTTOM)
    diff_flipped = ImageChops.difference(vk_flipped, dx_img)
    diff_flipped_arr = np.array(diff_flipped)
    mean_diff_flipped = np.mean(diff_flipped_arr)
    
    # Create comparison image
    width, height = vk_img.size
    comparison = Image.new('RGB', (width * 3 + 40, height + 100), 'white')
    
    # Paste images
    comparison.paste(vk_img, (10, 50))
    comparison.paste(dx_img, (width + 20, 50))
    comparison.paste(diff, (width * 2 + 30, 50))
    
    # Add labels
    draw = ImageDraw.Draw(comparison)
    try:
        font = ImageFont.truetype("/usr/share/fonts/liberation/LiberationSans-Regular.ttf", 20)
        font_small = ImageFont.truetype("/usr/share/fonts/liberation/LiberationSans-Regular.ttf", 14)
    except:
        font = ImageFont.load_default()
        font_small = font
    
    draw.text((10, 10), "Vulkan", fill='black', font=font)
    draw.text((width + 20, 10), "DirectX 12", fill='black', font=font)
    draw.text((width * 2 + 30, 10), "Difference", fill='black', font=font)
    
    # Add statistics
    stats_y = height + 60
    stats = [
        f"Max diff: {max_diff}",
        f"Mean diff: {mean_diff:.2f}",
        f"Pixels differ: {percent_diff:.2f}%",
    ]
    
    if mean_diff_flipped < mean_diff * 0.5:
        stats.append(f"⚠ Y-axis appears flipped!")
        stats.append(f"Mean diff flipped: {mean_diff_flipped:.2f}")
    
    for i, stat in enumerate(stats):
        draw.text((10, stats_y + i * 20), stat, fill='black', font=font_small)
    
    # Save
    comparison.save(output_path)
    
    print(f"Comparison saved: {output_path}")
    print(f"  Max difference: {max_diff}")
    print(f"  Mean difference: {mean_diff:.2f}")
    print(f"  Pixels different: {percent_diff:.2f}%")
    if mean_diff_flipped < mean_diff * 0.5:
        print(f"  ⚠ WARNING: Images appear to be flipped on Y-axis!")
        print(f"  Mean diff when flipped: {mean_diff_flipped:.2f}")
    
    return max_diff, mean_diff, percent_diff

if __name__ == '__main__':
    if len(sys.argv) != 4:
        print("Usage: compare_backends.py <vk_image> <dx_image> <output>")
        sys.exit(1)
    
    compare_images(sys.argv[1], sys.argv[2], sys.argv[3])
