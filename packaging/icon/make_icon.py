"""Render the MaxCap app icon (pixel art, OVERWORLD palette) into assets/icon/.

Run from the repo root:  python packaging/icon/make_icon.py   (needs Pillow)
Writes maxcap.png (1024), maxcap.ico (16-256) and maxcap.icns.
"""
from pathlib import Path
from PIL import Image

INK = (0x16, 0x13, 0x10, 255)
BONE = (0xFF, 0xFA, 0xEF, 255)
COBALT = (0x2E, 0x5D, 0xD6, 255)
VERMILLION = (0xE2, 0x52, 0x2E, 255)

# (colour, x, y, w, h) in grid units, painted in order.
GRID32 = (32, [
    (COBALT, 5, 5, 24, 24),       # hard shadow
    (INK, 3, 3, 24, 24),          # tile outline
    (BONE, 5, 5, 20, 20),         # tile face
    (INK, 7, 8, 16, 10),          # screen bezel
    (COBALT, 8, 9, 14, 8),        # screen
    (VERMILLION, 13, 11, 4, 4),   # the ■ from the wordmark
    (INK, 14, 18, 2, 2),          # stand neck
    (INK, 11, 20, 8, 2),          # stand base
])
# Hand-fitted for 16 px so every line stays a whole pixel.
GRID16 = (16, [
    (COBALT, 2, 2, 13, 13),
    (INK, 1, 1, 13, 13),
    (BONE, 2, 2, 11, 11),
    (INK, 3, 3, 9, 7),
    (COBALT, 4, 4, 7, 5),
    (VERMILLION, 6, 5, 3, 3),
    (INK, 7, 10, 1, 1),
    (INK, 5, 11, 5, 1),
])


def render(grid, size):
    n, rects = grid
    img = Image.new("RGBA", (n, n), (0, 0, 0, 0))
    for colour, x, y, w, h in rects:
        img.paste(colour, (x, y, x + w, y + h))
    if size % n == 0:
        return img.resize((size, size), Image.NEAREST)
    big = img.resize((n * 32, n * 32), Image.NEAREST)
    return big.resize((size, size), Image.LANCZOS)


def main():
    out = Path("assets/icon")
    out.mkdir(parents=True, exist_ok=True)
    master = render(GRID32, 1024)
    master.save(out / "maxcap.png")

    sizes = [16, 24, 32, 48, 64, 128, 256]
    frames = {s: render(GRID16 if s < 32 else GRID32, s) for s in sizes}
    # Pillow's ICO writer takes the largest image and the size list; pass our own
    # per-size frames so 16/24 use the hand-fitted grid instead of a downscale.
    frames[256].save(out / "maxcap.ico", sizes=[(s, s) for s in sizes],
                     append_images=[frames[s] for s in sizes if s != 256])
    master.save(out / "maxcap.icns")
    print("wrote", ", ".join(str(p) for p in sorted(out.iterdir())))


if __name__ == "__main__":
    main()
