#!/usr/bin/env python3
"""Create a traceable first-pass scene manifest from an immutable screenshot."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path

from PIL import Image

from visual_check import DEFAULT_LIMITS, SCHEMA, sha256_file


def create_scene(image: Path, output: Path, scale: float, state: str, crop: list[int] | None = None) -> dict:
    image = image.resolve()
    output = output.resolve()
    with Image.open(image) as img:
        if img.format != "PNG":
            raise ValueError("reference image must be PNG")
        width, height = img.size
    if scale <= 0 or scale != scale or scale == float("inf"):
        raise ValueError("pixels-per-point must be finite and > 0")
    if not state.strip():
        raise ValueError("state must not be empty")
    content = crop if crop is not None else [0, 0, width, height]
    from visual_check import ensure_rect
    ensure_rect(content, width, height, "content_rect_px")
    return {
        "schema": SCHEMA,
        "reference": {
            "path": os.path.relpath(image, output.parent),
            "sha256": sha256_file(image),
            "pixel_size": [width, height],
            "pixels_per_point": scale,
            "content_rect_px": content,
            "state": state,
            "platform": "unverified",
            "renderer": "unverified",
            "theme": "unverified",
            "font": "unverified",
        },
        "matte_rgb": [255, 255, 255],
        "regions": [{"id": "full", "role": "whole reference", "rect_px": [0, 0, width, height], "weight": 1, "confidence": "high"}],
        "ignore_rects_px": [],
        "tolerances": dict(DEFAULT_LIMITS),
        "observed_text": [],
        "assumed_interactions": [],
        "uncertainties": ["pixels_per_point is an input assumption until verified with capture metadata"],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--pixels-per-point", type=float, required=True)
    parser.add_argument("--state", default="default")
    parser.add_argument("--content-rect", type=int, nargs=4, metavar=("X", "Y", "W", "H"))
    args = parser.parse_args()
    try:
        scene = create_scene(args.image, args.out, args.pixels_per_point, args.state, args.content_rect)
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(scene, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    except (OSError, ValueError) as exc:
        parser.error(str(exc))
    print(f"Created {args.out} ({scene['reference']['pixel_size']}); edit regions/tokens before implementation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
