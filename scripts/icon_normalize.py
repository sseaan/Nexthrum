#!/usr/bin/env python3
"""Normalize an app icon: trim transparent padding, then re-pad to a target ratio.

Standard library only (zlib), so it runs on any platform without Pillow.

Usage:
    icon_normalize.py SOURCE.png DEST.png --canvas 1024 --content 0.92

- `--canvas`  output edge length in pixels (square)
- `--content` fraction of the canvas the artwork should occupy (0 < r <= 1)

The artwork is cropped to its alpha bounding box and resampled with a
premultiplied-alpha box filter, which keeps edges clean when the source has
partially transparent borders.
"""

import argparse
import struct
import sys
import zlib


def read_png(path):
    with open(path, "rb") as handle:
        data = handle.read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"{path}: not a PNG file")

    pos = 8
    header = None
    idat = bytearray()
    while pos < len(data):
        (length,) = struct.unpack(">I", data[pos : pos + 4])
        kind = data[pos + 4 : pos + 8]
        payload = data[pos + 8 : pos + 8 + length]
        pos += 12 + length
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", payload)
        elif kind == b"IDAT":
            idat += payload
        elif kind == b"IEND":
            break

    if header is None:
        raise ValueError(f"{path}: missing IHDR")
    width, height, depth, color_type, _, _, interlace = header
    if depth != 8 or interlace != 0:
        raise ValueError(f"{path}: only 8-bit non-interlaced PNGs are supported")
    if color_type not in (2, 6):
        raise ValueError(f"{path}: only RGB/RGBA PNGs are supported (got color type {color_type})")

    channels = 4 if color_type == 6 else 3
    raw = zlib.decompress(bytes(idat))
    stride = width * channels
    pixels = [[0] * (width * 4) for _ in range(height)]
    previous = bytearray(stride)
    offset = 0
    for y in range(height):
        filter_type = raw[offset]
        offset += 1
        line = bytearray(raw[offset : offset + stride])
        offset += stride
        for i in range(stride):
            left = line[i - channels] if i >= channels else 0
            up = previous[i]
            up_left = previous[i - channels] if i >= channels else 0
            value = line[i]
            if filter_type == 1:
                value += left
            elif filter_type == 2:
                value += up
            elif filter_type == 3:
                value += (left + up) // 2
            elif filter_type == 4:
                p = left + up - up_left
                pa, pb, pc = abs(p - left), abs(p - up), abs(p - up_left)
                predictor = left if pa <= pb and pa <= pc else (up if pb <= pc else up_left)
                value += predictor
            elif filter_type != 0:
                raise ValueError(f"{path}: unsupported PNG filter {filter_type}")
            line[i] = value & 0xFF
        previous = line
        row = pixels[y]
        for x in range(width):
            base = x * channels
            out = x * 4
            row[out] = line[base]
            row[out + 1] = line[base + 1]
            row[out + 2] = line[base + 2]
            row[out + 3] = line[base + 3] if channels == 4 else 255
    return width, height, pixels


def content_bounds(width, height, pixels, threshold=8):
    min_x, min_y, max_x, max_y = width, height, -1, -1
    for y in range(height):
        row = pixels[y]
        for x in range(width):
            if row[x * 4 + 3] > threshold:
                min_x = min(min_x, x)
                max_x = max(max_x, x)
                min_y = min(min_y, y)
                max_y = max(max_y, y)
    if max_x < 0:
        raise ValueError("image is fully transparent")
    return min_x, min_y, max_x - min_x + 1, max_y - min_y + 1


def resample(width, height, pixels, box_x, box_y, box_w, box_h, target):
    """Premultiplied-alpha box filter from the crop box into a target x target grid."""
    scale_x = box_w / target
    scale_y = box_h / target
    out = [[0] * (target * 4) for _ in range(target)]
    for ty in range(target):
        y0 = box_y + int(ty * scale_y)
        y1 = max(y0 + 1, box_y + int((ty + 1) * scale_y))
        y1 = min(y1, box_y + box_h, height)
        for tx in range(target):
            x0 = box_x + int(tx * scale_x)
            x1 = max(x0 + 1, box_x + int((tx + 1) * scale_x))
            x1 = min(x1, box_x + box_w, width)
            r = g = b = a = 0
            count = 0
            for y in range(y0, y1):
                row = pixels[y]
                for x in range(x0, x1):
                    alpha = row[x * 4 + 3]
                    r += row[x * 4] * alpha
                    g += row[x * 4 + 1] * alpha
                    b += row[x * 4 + 2] * alpha
                    a += alpha
                    count += 1
            if count == 0 or a == 0:
                continue
            pixel = tx * 4
            out[ty][pixel] = r // a
            out[ty][pixel + 1] = g // a
            out[ty][pixel + 2] = b // a
            out[ty][pixel + 3] = a // count
    return out


def write_png(path, size, pixels):
    raw = bytearray()
    for row in pixels:
        raw.append(0)  # filter: none
        raw.extend(row)

    def chunk(kind, payload):
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    with open(path, "wb") as handle:
        handle.write(b"\x89PNG\r\n\x1a\n")
        handle.write(chunk(b"IHDR", header))
        handle.write(chunk(b"IDAT", zlib.compress(bytes(raw), 9)))
        handle.write(chunk(b"IEND", b""))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source")
    parser.add_argument("dest")
    parser.add_argument("--canvas", type=int, default=1024)
    parser.add_argument("--content", type=float, default=0.92)
    args = parser.parse_args()

    if args.canvas <= 0 or not 0 < args.content <= 1:
        parser.error("--canvas must be positive and --content must be in (0, 1]")

    width, height, pixels = read_png(args.source)
    box_x, box_y, box_w, box_h = content_bounds(width, height, pixels)

    content_size = max(1, round(args.canvas * args.content))
    artwork = resample(width, height, pixels, box_x, box_y, box_w, box_h, content_size)

    padding = (args.canvas - content_size) // 2
    canvas = [[0] * (args.canvas * 4) for _ in range(args.canvas)]
    for y, row in enumerate(artwork):
        target = canvas[y + padding]
        offset = padding * 4
        target[offset : offset + content_size * 4] = row

    write_png(args.dest, args.canvas, canvas)
    print(
        f"{args.dest}: canvas {args.canvas}, content {content_size} "
        f"({args.content * 100:.1f}%), cropped {box_w}x{box_h} from {width}x{height}"
    )


if __name__ == "__main__":
    sys.exit(main())
