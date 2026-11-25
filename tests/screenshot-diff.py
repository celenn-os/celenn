#!/usr/bin/env python3
"""Compares the boot test screenshots of a build with those of a reference
build (the latest build of main), so a pull request shows what it changes on
screen, even where nobody looks.

Each screenshot is compared pixel by pixel. Changed pixels are grouped into
areas; the clocks (top panel, login and lock screens) are ignored. A change
made only of short single text lines is reported apart, as it is most often
a date, a version, a free space size or a processor name, which change from
one build to the next.

Writes a Markdown report (for the run summary) and, when something changed,
one tall JPEG: for each changed screenshot, before and after side by side
with the changed areas framed, then each area at full size.

Usage: tests/screenshot-diff.py <reference dir> <new dir> <report.md> <sheet.jpg> [title]
Exit status 0 even when screenshots differ: the report is information only.
"""
import importlib.util
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

# Captions, colors and font of the screenshot sheet (screenshot-sheet.py)
_spec = importlib.util.spec_from_file_location("screenshot_sheet", Path(__file__).with_name("screenshot-sheet.py"))
_sheet = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_sheet)
ACCENT, BACKGROUND, TEXT, caption, font = _sheet.ACCENT, _sheet.BACKGROUND, _sheet.TEXT, _sheet.caption, _sheet.font

# A pixel changed when one of its channels moved by more than this (PNG
# screenshots of the same screen are identical; this only absorbs blending)
PIXEL_THRESHOLD = 16
# Changed pixels are counted in square cells; a cell with fewer changed
# pixels is noise (a blinking caret, one anti-aliased pixel)
CELL = 8
CELL_MIN_PIXELS = 4
# Cells this close (in cells) belong to the same area
JOIN = 2
# Areas smaller than this (in cells) are ignored: a progress bar a few pixels
# longer, a spinner frame
AREA_MIN_CELLS = 3
# An area this short and small is one short line of text
TEXT_LINE_HEIGHT = 24
TEXT_LINE_CELLS = 30
# The terminal shows fastfetch: memory, disk and uptime lines change on every
# boot, two lines at a time
TEXT_LINE_HEIGHT_BY_SHOT = {"terminal": 48}

# Clocks, as boxes of the 1920x1080 screen (left, top, right, bottom): the top
# panel's clock on every screen, the big clock of the login and lock screens
PANEL_CLOCK = (880, 0, 1040, 40)
GREETER_CLOCK = (560, 240, 940, 410)
GREETER_SHOTS = ("login-screen", "lock-screen", "login-users", "parental-refused")

SIDE = 636  # width of each screenshot in the side by side view
ZOOM_MAX = (1240, 360)  # largest full-size crop of an area
PAD = 12


def key(path):
    """Screenshot name without its number: 06-updates.png -> updates."""
    stem = path.stem
    return stem.split("-", 1)[1] if stem[:2].isdigit() else stem


def shots(directory):
    return {
        key(p): p
        for p in Path(directory).iterdir()
        if p.suffix in (".png", ".ppm") and not p.stem.endswith("-zoom")
    }


def ignored_boxes(name, size):
    scale_x, scale_y = size[0] / 1920, size[1] / 1080
    boxes = [PANEL_CLOCK]
    if name in GREETER_SHOTS:
        boxes.append(GREETER_CLOCK)
    return [
        (round(l * scale_x), round(t * scale_y), round(r * scale_x), round(b * scale_y))
        for l, t, r, b in boxes
    ]


def inside(area, box):
    l, t, r, b = area
    return l >= box[0] and t >= box[1] and r <= box[2] and b <= box[3]


def changed_areas(before, after, ignored):
    """Groups the changed pixels into areas: list of ((l, t, r, b), cells)."""
    diff = ImageChops.difference(before, after).convert("L")
    mask = diff.point(lambda v: 255 if v > PIXEL_THRESHOLD else 0)
    if mask.getbbox() is None:
        return []
    width, height = mask.size
    # Changed pixels per cell: a box filter by downscaling (mean of 0/255)
    small = mask.resize(((width + CELL - 1) // CELL, (height + CELL - 1) // CELL), Image.Resampling.BOX)
    cutoff = 255 * CELL_MIN_PIXELS / (CELL * CELL)
    cols = small.width
    data = small.tobytes()
    cells = {(i % cols, i // cols) for i, v in enumerate(data) if v >= cutoff}
    areas, seen = [], set()
    for start in cells:
        if start in seen:
            continue
        seen.add(start)
        stack, members = [start], []
        while stack:
            x, y = stack.pop()
            members.append((x, y))
            for dx in range(-JOIN, JOIN + 1):
                for dy in range(-JOIN, JOIN + 1):
                    near = (x + dx, y + dy)
                    if near in cells and near not in seen:
                        seen.add(near)
                        stack.append(near)
        xs = [x for x, _ in members]
        ys = [y for _, y in members]
        box = (min(xs) * CELL, min(ys) * CELL, min((max(xs) + 1) * CELL, width), min((max(ys) + 1) * CELL, height))
        if len(members) < AREA_MIN_CELLS or any(inside(box, i) for i in ignored):
            continue
        areas.append((box, len(members)))
    return sorted(areas, key=lambda a: (a[0][1], a[0][0]))


def is_text_line(name, area):
    (l, t, r, b), cells = area
    return b - t <= TEXT_LINE_HEIGHT_BY_SHOT.get(name, TEXT_LINE_HEIGHT) and cells <= TEXT_LINE_CELLS


def compare(reference_dir, new_dir):
    """Returns {name: (status, areas, before, after)}; status is changed,
    text, same, new, gone or size."""
    old, new = shots(reference_dir), shots(new_dir)
    results = {}
    for name in sorted(old.keys() | new.keys(), key=lambda n: (new.get(n) or old.get(n)).name):
        if name not in old:
            results[name] = ("new", [], None, new[name])
            continue
        if name not in new:
            results[name] = ("gone", [], old[name], None)
            continue
        before = Image.open(old[name]).convert("RGB")
        after = Image.open(new[name]).convert("RGB")
        if before.size != after.size:
            results[name] = ("size", [], old[name], new[name])
            continue
        areas = changed_areas(before, after, ignored_boxes(name, after.size))
        if not areas:
            status = "same"
        elif all(is_text_line(name, a) for a in areas):
            status = "text"
        else:
            status = "changed"
        results[name] = (status, areas, old[name], new[name])
    return results


LABELS = {
    "changed": "Changed",
    "text": "Text only (likely a date, version or size)",
    "new": "New screenshot",
    "gone": "Screenshot no longer taken",
    "size": "Different screen size",
}
ORDER = ("changed", "size", "new", "gone", "text")


def framed(path, areas, color=(255, 92, 92)):
    image = Image.open(path).convert("RGB")
    draw = ImageDraw.Draw(image)
    for (l, t, r, b), _ in areas:
        draw.rectangle((l - 4, t - 4, r + 3, b + 3), outline=color, width=4)
    return image


def zooms(before_path, after_path, areas):
    """Each changed area at full size, before above after."""
    before = Image.open(before_path).convert("RGB")
    after = Image.open(after_path).convert("RGB")
    parts = []
    for (l, t, r, b), _ in areas[:4]:
        cx, cy = (l + r) // 2, (t + b) // 2
        w = min(max(r - l + 120, 640), ZOOM_MAX[0])
        h = min(max(b - t + 48, 72), ZOOM_MAX[1])
        left = min(max(cx - w // 2, 0), max(after.width - w, 0))
        top = min(max(cy - h // 2, 0), max(after.height - h, 0))
        box = (left, top, min(left + w, after.width), min(top + h, after.height))
        parts.append(("", before.crop(box)))
        parts.append(("", after.crop(box)))
    return parts


def thumb(image):
    return image.resize((SIDE, round(image.height * SIDE / image.width)), Image.Resampling.LANCZOS)


def sheet(results, out, title):
    shown = [(n, r) for s in ORDER for n, r in results.items() if r[0] == s]
    if not shown:
        return False
    title_font, caption_font, small_font = font(40), font(28), font(22)
    blocks = []
    for name, (status, areas, before_path, after_path) in shown:
        parts = [(f"{caption(name)}: {LABELS[status]}", None)]
        if status == "text":
            # Most often a date or a version: the changed lines only
            parts += zooms(before_path, after_path, areas)
            blocks.append(parts)
            continue
        before = framed(before_path, areas) if before_path else None
        after = framed(after_path, areas) if after_path else None
        thumbs = [thumb(i) if i else None for i in (before, after)]
        height = max(t.height for t in thumbs if t)
        row = Image.new("RGB", (2 * SIDE + PAD, height + 30), BACKGROUND)
        draw = ImageDraw.Draw(row)
        for i, (label, t) in enumerate(zip(("Before (main)", "After (this build)"), thumbs)):
            x = i * (SIDE + PAD)
            draw.text((x, 0), label, font=small_font, fill=TEXT)
            if t:
                row.paste(t, (x, 30))
        parts.append(("", row))
        if before and after:
            parts += zooms(before_path, after_path, areas)
        blocks.append(parts)

    width = 2 * SIDE + PAD + 2 * 24
    height = 90 + sum(
        sum((56 if text else 0) + (img.height + 10 if img else 0) for text, img in parts) + 30
        for parts in blocks
    )
    out_image = Image.new("RGB", (width, height), BACKGROUND)
    draw = ImageDraw.Draw(out_image)
    draw.text((24, 24), title, font=title_font, fill=ACCENT)
    y = 90
    for parts in blocks:
        for text, img in parts:
            if text:
                draw.text((24, y + 10), text, font=caption_font, fill=TEXT)
                y += 56
            if img:
                out_image.paste(img, (24, y))
                y += img.height + 10
        y += 30
    out_image.save(out, "JPEG", quality=85)
    return True


def report(results):
    counts = {s: sum(1 for r in results.values() if r[0] == s) for s in (*ORDER, "same")}
    lines = []
    visible = counts["changed"] + counts["size"] + counts["new"] + counts["gone"]
    if visible == 0 and counts["text"] == 0:
        lines.append(f"No visible change on the {counts['same']} screenshots.")
        return "\n".join(lines) + "\n"
    if visible:
        lines.append(f"**{visible} screenshot(s) changed** compared with the latest build of main.")
    else:
        lines.append("No visible change, apart from small text changes (dates, versions, sizes).")
    lines.append("")
    lines.append("| Screenshot | Change |")
    lines.append("| --- | --- |")
    for status in ORDER:
        for name, r in results.items():
            if r[0] == status:
                lines.append(f"| {caption(name)} | {LABELS[status]} |")
    lines.append("")
    lines.append(f"Unchanged: {counts['same']}.")
    return "\n".join(lines) + "\n"


def main():
    if len(sys.argv) < 5:
        sys.exit(__doc__)
    reference, new, report_path, sheet_path = sys.argv[1:5]
    title = sys.argv[5] if len(sys.argv) > 5 else "Screenshot changes"
    results = compare(reference, new)
    made = sheet(results, sheet_path, title)
    Path(report_path).write_text(report(results))
    for name, (status, areas, _, _) in results.items():
        if status != "same":
            print(f"{name}: {status} {[a[0] for a in areas]}")


if __name__ == "__main__":
    main()
