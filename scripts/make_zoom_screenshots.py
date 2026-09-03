#!/usr/bin/env python3
"""Crop YapCap panels from full-screen screenshots."""
import os
import subprocess
import tempfile
from pathlib import Path

SRC_DIR = Path.home() / "Pictures/Screenshots"
DST_DIR = Path(__file__).parent.parent / "resources/screenshots"

PROVIDERS = [
    "codex",
    "claude",
    "cursor",
    "antigravity",
    "gemini",
    "copilot",
    "minimax",
    "kimi",
]

CROP_LEFT = 2816
CROP_TOP = 250
CROP_WIDTH = 417
CROP_HEIGHT = 620

SETTINGS = [
    ("settings-general.png", "screenshot-settings.png"),
    ("settings-accounts.png", "screenshot-accounts.png"),
]
SETTINGS_CROP = "417x1004+2816+37"

THEMES = [
    ("theme-light-red.png", "screenshot-theme-light-red.png"),
    ("theme-light-blue.png", "screenshot-theme-light-blue.png"),
    ("theme-dark-orange.png", "screenshot-theme-dark-orange.png"),
    ("theme-dark-blue.png", "screenshot-theme-dark-blue.png"),
]
THEME_CROP = "417x915+2816+37"


def crop_image(src: Path, dst: Path, crop: str) -> None:
    if not src.exists():
        print(f"skip: missing {src}")
        return
    with tempfile.NamedTemporaryFile(suffix=dst.suffix, dir=dst.parent, delete=False) as file:
        candidate = Path(file.name)
    try:
        subprocess.run(
            ["convert", str(src), "-crop", crop, "+repage", str(candidate)],
            check=True,
        )
        if dst.exists():
            comparison = subprocess.run(
                ["compare", "-metric", "AE", str(dst), str(candidate), "null:"],
                capture_output=True,
            )
            if comparison.returncode == 0:
                print(f"Unchanged {dst}")
                return
            if comparison.returncode != 1:
                comparison.check_returncode()
        os.replace(candidate, dst)
        print(f"Saved {dst}")
    finally:
        candidate.unlink(missing_ok=True)


def main() -> None:
    DST_DIR.mkdir(parents=True, exist_ok=True)
    provider_crop = f"{CROP_WIDTH}x{CROP_HEIGHT}+{CROP_LEFT}+{CROP_TOP}"
    for provider in PROVIDERS:
        src = SRC_DIR / f"{provider}.png"
        dst = DST_DIR / f"screenshot-zoom-{provider}.png"
        crop_image(src, dst, provider_crop)
    for src_name, dst_name in SETTINGS:
        crop_image(SRC_DIR / src_name, DST_DIR / dst_name, SETTINGS_CROP)
    for src_name, dst_name in THEMES:
        crop_image(SRC_DIR / src_name, DST_DIR / dst_name, THEME_CROP)


if __name__ == "__main__":
    main()
