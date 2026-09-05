#!/usr/bin/env python3
"""Convert `cargo run --example render_validation` cell snapshots to PNGs."""
import json
import re
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs/validation"
OUTPUT.mkdir(parents=True, exist_ok=True)
FONT = ImageFont.truetype("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 18)
COLORS = {"Reset": (220, 225, 235), "Black": (18, 21, 28), "White": (238, 242, 248),
          "Gray": (166, 176, 192), "DarkGray": (111, 125, 145), "Red": (244, 104, 110),
          "Green": (104, 210, 155), "Yellow": (241, 205, 117), "Blue": (110, 162, 240),
          "Magenta": (193, 140, 230), "Cyan": (106, 211, 222), "LightRed": (255, 135, 141),
          "LightGreen": (141, 235, 181), "LightYellow": (255, 221, 138),
          "LightBlue": (141, 189, 255), "LightMagenta": (218, 169, 255), "LightCyan": (145, 236, 246)}

def color(value, fallback):
    if value in COLORS:
        return COLORS[value]
    if value.startswith("Rgb("):
        return tuple(map(int, re.findall(r"\d+", value)))
    return fallback

for source in sorted((ROOT / "target/ui-validation").glob("*.json")):
    data = json.loads(source.read_text())
    image = Image.new("RGB", (data["width"] * 10 + 32, data["height"] * 25 + 32), COLORS["Black"])
    draw = ImageDraw.Draw(image)
    for cell in data["cells"]:
        if cell["bg"] != "Reset":
            x, y = 16 + cell["x"] * 10, 16 + cell["y"] * 25
            draw.rectangle((x, y, x + 9, y + 24), fill=color(cell["bg"], COLORS["Black"]))
    for cell in data["cells"]:
        if cell["s"].strip():
            draw.text((16 + cell["x"] * 10, 12 + cell["y"] * 25), cell["s"],
                      font=FONT, fill=color(cell["fg"], COLORS["Reset"]))
    image.save(OUTPUT / (source.stem + ".png"))
print(f"Rendered {len(list(OUTPUT.glob('*.png')))} screenshots to {OUTPUT}")
