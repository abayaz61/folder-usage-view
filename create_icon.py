#!/usr/bin/env python3
"""
Simple script to create an ICO file for Disk Usage Analyzer
Requires: pip install pillow
"""

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    print("Installing Pillow...")
    import subprocess
    subprocess.check_call(['pip', 'install', 'pillow'])
    from PIL import Image, ImageDraw, ImageFont

import os

def create_icon():
    sizes = [16, 32, 48, 64, 128, 256]
    images = []

    for size in sizes:
        # Create image with transparent background
        img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)

        # Draw a pie chart style disk usage icon
        margin = size // 8
        bbox = [margin, margin, size - margin, size - margin]

        # Background circle (dark)
        draw.ellipse(bbox, fill=(40, 40, 50, 255))

        # Used space (cyan/teal gradient effect)
        draw.pieslice(bbox, start=0, end=270, fill=(0, 180, 200, 255))

        # Inner circle (darker center)
        inner_margin = size // 4
        inner_bbox = [inner_margin, inner_margin, size - inner_margin, size - inner_margin]
        draw.ellipse(inner_bbox, fill=(30, 30, 40, 255))

        # Center dot
        center_margin = size // 3
        center_bbox = [center_margin, center_margin, size - center_margin, size - center_margin]
        draw.ellipse(center_bbox, fill=(0, 200, 220, 255))

        images.append(img)

    # Save as ICO
    script_dir = os.path.dirname(os.path.abspath(__file__))
    ico_path = os.path.join(script_dir, 'assets', 'icon.ico')

    # Ensure assets directory exists
    os.makedirs(os.path.dirname(ico_path), exist_ok=True)

    # Save with all sizes
    images[0].save(
        ico_path,
        format='ICO',
        sizes=[(s, s) for s in sizes],
        append_images=images[1:]
    )

    print(f"Icon created: {ico_path}")

if __name__ == '__main__':
    create_icon()
