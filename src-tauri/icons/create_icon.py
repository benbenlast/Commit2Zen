import struct

# Create a valid ICO file
width = 16
height = 16

# ICO Header
ico_header = struct.pack('<HHH', 0, 1, 1)

# Image directory entry
bmp_size = 40 + width * height * 4
image_offset = 6 + 16
dir_entry = struct.pack('<BBBBHHII', width, height, 0, 0, 1, 32, bmp_size, image_offset)

# BMP header
bmp_header = struct.pack('<IiiHHIIIIII', 40, width, height * 2, 1, 32, 0, 0, 0, 0, 0, 0)

# Pixel data (green #18A058 in BGRA format)
pixel = struct.pack('BBBB', 0x58, 0xA0, 0x18, 0xFF) * (width * height)

with open('D:/aicode/Commit2Zen/src-tauri/icons/icon.ico', 'wb') as f:
    f.write(ico_header + dir_entry + bmp_header + pixel)

print('Created icon.ico')
