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

class Stitch:
    def __init__(self, color):
        self.color = color
        self.thread = None

class Thread:
    def __init__(self, thread_index, x, y, color):
        if thread_index == 0:
            self.text = "A"
        else:
            text = []
            while thread_index > 0:
                text.append(chr(ord("A") + thread_index % 26))
                thread_index //= 26
            self.text = "".join(reversed(text))

        self.x = x
        self.y = y
        self.color = color
        self.stitch_count = 0

    def __repr__(self):
        return f"{{{self.text} {self.x},{self.y} {self.color}}}"

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

def find_thread(threads, color, stitch_x, stitch_y):
    for i in range(len(threads) - 1, -1, -1):
        thread = threads[i]

        if thread.y - stitch_y > 2:
            break

        if thread.color != color:
            continue

        if abs(thread.x - stitch_x) < 2:
            del threads[i]
            threads.append(thread)
            thread.x = stitch_x
            thread.y = stitch_y
            return thread

    threads.append(Thread(len(threads), stitch_x, stitch_y, color))
    return threads[-1]

png_file = sys.argv[1]
svg_file = os.path.splitext(png_file)[0] + ".svg"

image = Image.open(sys.argv[1])

(image_width, image_height) = image.size

sample_width = image_width / N_STITCHES
sample_height = sample_width * GAUGE_STITCHES / GAUGE_ROWS

n_rows = int(image_height / sample_height)

stitches = []

for stitch_y in range(n_rows):
    sample_start_y = round(sample_height * stitch_y)
    sample_end_y = min(round(sample_height * (stitch_y + 1)), image_height)

    for stitch_x in range(N_STITCHES):
        sample_start_x = round(sample_width * stitch_x)
        sample_end_x = min(round(sample_width * (stitch_x + 1)), image_width)

        best_color = most_popular_color(image,
                                        sample_start_x, sample_end_x,
                                        sample_start_y, sample_end_y)

        stitches.append(Stitch(best_color))

threads = []

for stitch_y in range(n_rows - 1, -1, -1):
    for stitch_x in range(N_STITCHES):
        if (n_rows - 1 - stitch_y) & 1 == 0:
            stitch_x = N_STITCHES - 1 - stitch_x

        stitch = stitches[stitch_x + stitch_y * N_STITCHES]

        thread = find_thread(threads, stitch.color, stitch_x, stitch_y)
        thread.stitch_count += 1

        stitch.thread = thread

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

        stitch = stitches[stitch_x + stitch_y * N_STITCHES]

        cr.set_source_rgb(*stitch.color)
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

for y in range(n_rows):
    for x in range(N_STITCHES):
        thread = stitches[x + y * N_STITCHES].thread

        if sum(thread.color) < 1.5:
            cr.set_source_rgb(1.0, 1.0, 1.0)
        else:
            cr.set_source_rgb(0.0, 0.0, 0.0)

        extents = cr.text_extents(thread.text)
        cr.move_to((x + 0.5) * BOX_WIDTH - extents.x_advance / 2.0,
                   (y + 0.7) * BOX_HEIGHT)
        cr.show_text(thread.text)
