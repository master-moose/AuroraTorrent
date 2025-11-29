#!/usr/bin/env python3
"""
Fix icon transparency by removing white/grey checkerboard backgrounds
and ensuring proper alpha channel handling.
"""

import sys
from PIL import Image
import os

def remove_checkerboard_background(image):
    """
    Remove checkerboard/white background and ensure proper transparency.
    Uses flood fill + edge shaving to remove halos.
    """
    # Convert to RGBA if not already
    if image.mode != 'RGBA':
        image = image.convert('RGBA')
    
    width, height = image.size
    pixels = image.load()
    
    # 1. Aggressive Background Detection
    def is_background_color(r, g, b):
        # White / Near White
        if r > 240 and g > 240 and b > 240:
            return True
            
        # Greys (Checkerboard)
        # Check for low saturation (grey)
        diff_rg = abs(r - g)
        diff_gb = abs(g - b)
        diff_rb = abs(r - b)
        
        is_low_saturation = (diff_rg < 20 and diff_gb < 20 and diff_rb < 20)
        
        # Background is usually light grey
        if is_low_saturation and r > 180: 
            return True
            
        return False

    # 2. Flood Fill from corners
    print("  - Running flood fill...")
    stack = []
    visited = set()
    
    # Start from corners/edges
    for x in range(width):
        stack.append((x, 0))
        stack.append((x, height - 1))
    for y in range(height):
        stack.append((0, y))
        stack.append((width - 1, y))
        
    # Track transparent pixels for the next step
    transparent_pixels = set()
    
    while stack:
        x, y = stack.pop()
        
        if (x, y) in visited:
            continue
        if x < 0 or x >= width or y < 0 or y >= height:
            continue
            
        visited.add((x, y))
        
        r, g, b, a = pixels[x, y]
        
        if is_background_color(r, g, b) or a == 0:
            pixels[x, y] = (0, 0, 0, 0)
            transparent_pixels.add((x, y))
            
            # Add neighbors
            stack.append((x + 1, y))
            stack.append((x - 1, y))
            stack.append((x, y + 1))
            stack.append((x, y - 1))

    # 3. Shave Light Edges (Halo Removal)
    # Iteratively remove non-transparent pixels that border transparent ones AND are light-colored
    print("  - Shaving light edges (halo removal)...")
    
    # How many pixels deep to check/remove?
    # The halo looks to be about 2-4 pixels wide in some spots
    shave_iterations = 4
    
    for i in range(shave_iterations):
        print(f"    Pass {i+1}/{shave_iterations}")
        pixels_to_remove = []
        
        # Find boundary pixels
        # We only need to check pixels adjacent to currently transparent ones
        # But optimizing that set maintenance is complex, simpler to iterate or check boundary
        # Let's do a scan of the image to find boundary pixels
        
        for y in range(1, height - 1):
            for x in range(1, width - 1):
                r, g, b, a = pixels[x, y]
                
                if a == 0: continue # Already transparent
                
                # Check neighbors
                has_transparent_neighbor = (
                    pixels[x+1, y][3] == 0 or
                    pixels[x-1, y][3] == 0 or
                    pixels[x, y+1][3] == 0 or
                    pixels[x, y-1][3] == 0
                )
                
                if has_transparent_neighbor:
                    # This is an edge pixel.
                    # Is it light colored? (Part of the white halo)
                    # We use a stricter threshold here - we want to keep the dark icon, remove light edge
                    is_light = (r > 100 and g > 100 and b > 100)
                    
                    # Also check if it's "greyish" (remnants of checkerboard)
                    diff = max(abs(r-g), abs(g-b), abs(r-b))
                    is_greyish = diff < 30
                    
                    if is_light:
                        pixels_to_remove.append((x, y))
        
        # Apply removals
        if not pixels_to_remove:
            break
            
        for x, y in pixels_to_remove:
            pixels[x, y] = (0, 0, 0, 0)
            
    return image

def fix_icon(input_path, output_path):
    """
    Fix transparency in an icon file.
    """
    print(f"Processing: {input_path}")
    
    try:
        # Open image
        img = Image.open(input_path)
        print(f"  Original mode: {img.mode}, size: {img.size}")
        
        # Fix transparency
        fixed_img = remove_checkerboard_background(img)
        
        # Save with proper transparency settings
        fixed_img.save(output_path, 'PNG', optimize=True)
        print(f"  Saved fixed image to: {output_path}")
        
        return True
    except Exception as e:
        print(f"  Error processing {input_path}: {e}", file=sys.stderr)
        return False

def main():
    icon_dir = os.path.join(os.path.dirname(__file__), 'apps/ui/src-tauri/icons')
    input_icon = os.path.join(icon_dir, 'newIcon.png')
    output_icon = os.path.join(icon_dir, 'newIcon_fixed.png')
    
    if not os.path.exists(input_icon):
        print(f"Error: {input_icon} not found", file=sys.stderr)
        sys.exit(1)
    
    print("Fixing icon transparency...")
    success = fix_icon(input_icon, output_icon)
    
    if success:
        # Replace original with fixed version
        backup_path = input_icon + '.backup'
        os.rename(input_icon, backup_path)
        os.rename(output_icon, input_icon)
        print(f"\n✓ Icon fixed! Original backed up to: {backup_path}")
        print(f"✓ Fixed icon saved to: {input_icon}")
        print("\nNow run: cd apps/ui && npm run tauri icon ./src-tauri/icons/newIcon.png")
    else:
        print("\n✗ Failed to fix icon", file=sys.stderr)
        sys.exit(1)

if __name__ == '__main__':
    main()

