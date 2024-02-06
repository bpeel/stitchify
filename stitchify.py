#!/usr/bin/python3

import cairo
from PIL import Image
import sys
import os

GAUGE_STITCHES = 22
GAUGE_ROWS = 30

BOX_WIDTH = 20
BOX_HEIGHT = BOX_WIDTH * GAUGE_STITCHES // GAUGE_ROWS
LINE_WIDTH = BOX_WIDTH / 6.0

N_STITCHES = GAUGE_STITCHES

def most_popular_color(image, start_x, end_x, start_y, end_y):
    colors = {}

    for y in range(start_y, end_y):
        for x in range(start_x, end_x):
            color = image.getpixel((x, y))
            try:
                colors[color] += 1
            except KeyError:
                colors[color] = 1

    color = max(colors.keys(), key=lambda color: colors[color])

    return (color[0] / 255.0, color[1] / 255.0, color[2] / 255.0)

png_file = sys.argv[1]
svg_file = os.path.splitext(png_file)[0] + ".svg"

image = Image.open(sys.argv[1])

(image_width, image_height) = image.size

sample_width = image_width / N_STITCHES
sample_height = sample_width * GAUGE_STITCHES / GAUGE_ROWS

n_rows = int(image_height / sample_height)

surface = cairo.SVGSurface(svg_file,
                           (N_STITCHES + 1) * BOX_WIDTH + LINE_WIDTH / 2.0,
                           (n_rows + 1) * BOX_HEIGHT + LINE_WIDTH / 2.0)

cr = cairo.Context(surface)

cr.translate(LINE_WIDTH / 2.0, LINE_WIDTH / 2.0)

for stitch_y in range(n_rows):
    sample_start_y = round(sample_height * stitch_y)
    sample_end_y = min(round(sample_height * (stitch_y + 1)), image_height)

    for stitch_x in range(N_STITCHES):
        sample_start_x = round(sample_width * stitch_x)
        sample_end_x = min(round(sample_width * (stitch_x + 1)), image_width)

        best_color = most_popular_color(image,
                                        sample_start_x, sample_end_x,
                                        sample_start_y, sample_end_y)

        cr.set_source_rgb(*best_color)
        cr.rectangle(stitch_x * BOX_WIDTH,
                     stitch_y * BOX_HEIGHT,
                     BOX_WIDTH,
                     BOX_HEIGHT)
        cr.fill()

cr.save()
cr.set_line_width(LINE_WIDTH)
cr.set_source_rgb(0.71, 0.71, 0.71)
cr.set_line_cap(cairo.LINE_CAP_SQUARE)

for x in range(N_STITCHES + 1):
    cr.move_to(x * BOX_WIDTH, 0)
    cr.rel_line_to(0, BOX_HEIGHT * n_rows)

for y in range(n_rows + 1):
    cr.move_to(0, y * BOX_HEIGHT)
    cr.rel_line_to(BOX_WIDTH * N_STITCHES, 0)

cr.stroke()

cr.restore()

cr.set_source_rgb(0.0, 0.0, 0.0)
cr.set_font_size(BOX_HEIGHT * 0.6)

for x in range(N_STITCHES):
    text = f"{x + 1}"
    extents = cr.text_extents(text)
    cr.move_to((N_STITCHES - 1 - x + 0.5) * BOX_WIDTH
               - extents.x_advance / 2.0,
               (n_rows + 0.7) * BOX_HEIGHT)
    cr.show_text(text)

for y in range(n_rows):
    text = f"{y + 1}"
    extents = cr.text_extents(text)
    cr.move_to((N_STITCHES + 0.5) * BOX_WIDTH - extents.x_advance / 2.0,
               (n_rows - 1 - y + 0.7) * BOX_HEIGHT)
    cr.show_text(text)
