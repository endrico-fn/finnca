"""Generate flat-square README badges in the finnca industrial palette.

Full control over text fill (shields.io auto-picks font color and offers
no override): background #0c6338, label text gray, value text cyan-green.

Usage: python3 assets/badges/badges.py [--version V]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

OUT_DIR = Path(__file__).resolve().parent
FONT_SIZE = 11
CHAR_W = 6.6
PAD_X = 7
HEIGHT = 20

BG = "#0c6338"
LABEL_TEXT = "#a1a1aa"
VALUE_TEXT = "#2dd4bf"
DIVIDER = "#232527"


def text_width(text: str) -> float:
    return len(text) * CHAR_W


def badge(label: str, value: str) -> str:
    label = label.upper()
    value = value.upper()
    lw = text_width(label) + PAD_X * 2
    vw = text_width(value) + PAD_X * 2
    total = lw + vw
    lx = lw / 2
    vx = lw + vw / 2
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{total:.0f}" height="{HEIGHT}"'
        f' font-family="ui-monospace,SFMono-Regular,Menlo,Consolas,monospace"'
        f' font-size="{FONT_SIZE}" font-weight="bold" letter-spacing="1">'
        f'<rect width="{total:.0f}" height="{HEIGHT}" fill="{BG}"/>'
        f'<rect x="{lw:.0f}" width="1" height="{HEIGHT}" fill="{DIVIDER}"/>'
        f'<text x="{lx:.1f}" y="14" text-anchor="middle" fill="{LABEL_TEXT}">{label}</text>'
        f'<text x="{vx:.1f}" y="14" text-anchor="middle" fill="{VALUE_TEXT}">{value}</text>'
        "</svg>\n"
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", default="0.1.0")
    args = parser.parse_args()

    badges = {
        "license.svg": ("license", "MIT"),
        "release.svg": ("release", f"v{args.version}"),
        "platform.svg": ("platform", "linux | windows"),
        "status.svg": ("status", "active development"),
    }
    for filename, (label, value) in badges.items():
        (OUT_DIR / filename).write_text(badge(label, value))
        print(f"wrote {filename}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
