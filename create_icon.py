#!/usr/bin/env python3
"""
High-resolution icon generator for Disk Usage Analyzer
Requires: pip install pillow
"""

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("Installing Pillow...")
    import subprocess
    subprocess.check_call(['pip', 'install', 'pillow'])
    from PIL import Image, ImageDraw

import os
import struct
import io

def create_disk_icon(size):
    """Create a single disk usage icon at specified size"""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Calculate margins based on size
    margin = max(1, size // 10)
    center = size // 2

    # Colors
    bg_dark = (30, 35, 45, 255)
    cyan_main = (0, 200, 220, 255)
    used_color = (0, 180, 200, 255)

    # Draw outer ring (background)
    bbox = [margin, margin, size - margin, size - margin]
    draw.ellipse(bbox, fill=bg_dark)

    # Draw used space arc (75% usage indicator)
    draw.pieslice(bbox, start=-90, end=180, fill=used_color)

    # Draw inner circle (donut hole)
    inner_margin = size // 4
    inner_bbox = [inner_margin, inner_margin, size - inner_margin, size - inner_margin]
    draw.ellipse(inner_bbox, fill=bg_dark)

    # Draw center highlight dot
    center_size = max(2, size // 6)
    center_offset = center - center_size // 2
    center_bbox = [center_offset, center_offset, center_offset + center_size, center_offset + center_size]
    draw.ellipse(center_bbox, fill=cyan_main)

    # Add subtle highlight on larger sizes
    if size >= 48:
        highlight = Image.new('RGBA', (size, size), (0, 0, 0, 0))
        highlight_draw = ImageDraw.Draw(highlight)
        highlight_draw.pieslice(bbox, start=-135, end=-45, fill=(255, 255, 255, 30))
        img = Image.alpha_composite(img, highlight)

    return img

def write_ico_file(images, filepath):
    """Write multiple images as a proper ICO file"""
    num_images = len(images)

    # ICO header: 2 bytes reserved, 2 bytes type (1=icon), 2 bytes count
    header = struct.pack('<HHH', 0, 1, num_images)

    # Prepare image data as PNG for each size
    image_data_list = []
    for img in images:
        buf = io.BytesIO()
        img.save(buf, format='PNG')
        image_data_list.append(buf.getvalue())

    # Calculate directory entries and offsets
    # Directory entry: 16 bytes each
    # Offset starts after header (6 bytes) + directory (16 * num_images)
    offset = 6 + 16 * num_images

    directory = b''
    for i, (img, data) in enumerate(zip(images, image_data_list)):
        width = img.width if img.width < 256 else 0
        height = img.height if img.height < 256 else 0
        # Entry: width, height, colors, reserved, planes, bpp, size, offset
        entry = struct.pack('<BBBBHHII',
            width, height, 0, 0, 1, 32, len(data), offset)
        directory += entry
        offset += len(data)

    # Write the file
    with open(filepath, 'wb') as f:
        f.write(header)
        f.write(directory)
        for data in image_data_list:
            f.write(data)

def create_icon():
    # Standard Windows icon sizes for crisp display at all DPIs
    sizes = [16, 24, 32, 48, 64, 128, 256]

    # Generate images for each size
    images = [create_disk_icon(s) for s in sizes]

    # Save directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    ico_path = os.path.join(script_dir, 'assets', 'icon.ico')
    os.makedirs(os.path.dirname(ico_path), exist_ok=True)

    # Save as multi-resolution ICO using custom writer
    write_ico_file(images, ico_path)

    # Verify the ICO file
    ico_size = os.path.getsize(ico_path)
    print(f"High-resolution icon created: {ico_path}")
    print(f"File size: {ico_size:,} bytes")
    print(f"Included sizes: {sizes}")

    # Also save a large PNG for other uses
    png_path = os.path.join(script_dir, 'assets', 'icon_512.png')
    large_icon = create_disk_icon(512)
    large_icon.save(png_path, format='PNG')
    print(f"Large PNG created: {png_path}")

if __name__ == '__main__':
    create_icon()
