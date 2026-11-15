#!/usr/bin/env python3
"""Auditable reference-vs-candidate PNG comparison for egui screenshot tasks.

No scaling, image warping, crops, or registration is applied when scoring.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import numpy as np
from PIL import Image, ImageDraw


SCHEMA = "egui-visual-forge.scene.v1"
DEFAULT_LIMITS = {
    "max_mae": 12.0,
    "max_changed_fraction": 0.12,
    "min_edge_f1": 0.70,
    "pixel_delta_threshold": 18,
    "edge_threshold": 24,
    "max_mask_fraction": 0.10,
}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def ensure_rect(rect: Any, width: int, height: int, what: str) -> tuple[int, int, int, int]:
    if (not isinstance(rect, list) or len(rect) != 4 or
            not all(type(v) is int for v in rect)):
        raise ValueError(f"{what} must contain four integer values [x,y,w,h]")
    x, y, w, h = rect
    if x < 0 or y < 0 or w <= 0 or h <= 0 or x + w > width or y + h > height:
        raise ValueError(f"{what} {rect} outside {width}x{height}")
    return x, y, w, h


def validate_scene(scene: dict[str, Any], scene_path: Path) -> tuple[Path, int, int]:
    if scene.get("schema") != SCHEMA:
        raise ValueError(f"expected schema {SCHEMA}")
    reference = scene.get("reference")
    if not isinstance(reference, dict):
        raise ValueError("scene.reference must be an object")
    ref_path = reference.get("path")
    if not isinstance(ref_path, str) or not ref_path.strip():
        raise ValueError("scene.reference.path missing")
    reference_path = (scene_path.parent / ref_path).resolve()
    if not reference_path.exists():
        raise ValueError(f"reference image does not exist: {reference_path}")
    expected_sha = reference.get("sha256")
    if not isinstance(expected_sha, str) or len(expected_sha) != 64:
        raise ValueError("reference.sha256 must contain the original 64-character SHA-256")
    actual_sha = sha256_file(reference_path)
    if actual_sha != expected_sha:
        raise ValueError(f"reference checksum changed: expected {expected_sha}, got {actual_sha}")
    size = reference.get("pixel_size")
    if (not isinstance(size, list) or len(size) != 2 or
            not all(type(v) is int and v > 0 for v in size)):
        raise ValueError("reference.pixel_size must be [positive_int,positive_int]")
    width, height = size
    scale = reference.get("pixels_per_point")
    if not isinstance(scale, (int, float)) or isinstance(scale, bool) or not math.isfinite(scale) or scale <= 0:
        raise ValueError("reference.pixels_per_point must be positive and finite")
    ensure_rect(reference.get("content_rect_px"), width, height, "content_rect_px")
    if not isinstance(reference.get("state"), str) or not reference["state"].strip():
        raise ValueError("reference.state must be a nonempty string")
    regions = scene.get("regions")
    if not isinstance(regions, list) or not regions:
        raise ValueError("scene.regions must contain at least one region")
    seen: set[str] = set()
    for idx, region in enumerate(regions):
        if not isinstance(region, dict):
            raise ValueError(f"region {idx} must be object")
        name = region.get("id")
        if not isinstance(name, str) or not name or name in seen:
            raise ValueError(f"region {idx} id missing or duplicated: {name}")
        seen.add(name)
        ensure_rect(region.get("rect_px"), width, height, f"region {name}")
        weight = region.get("weight", 1.0)
        if not isinstance(weight, (int, float)) or isinstance(weight, bool) or not 0.1 <= weight <= 10:
            raise ValueError(f"region {name} weight must be between 0.1 and 10")
        regional_quality = region.get("quality", {})
        if not isinstance(regional_quality, dict):
            raise ValueError(f"region {name} quality must be an object")
        for key, value in regional_quality.items():
            if key not in ("max_mae", "max_changed_fraction", "min_edge_f1", "pixel_delta_threshold", "edge_threshold"):
                raise ValueError(f"region {name} unknown quality field {key}")
            if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value):
                raise ValueError(f"region {name} invalid quality value {key}")
    masks = scene.get("ignore_rects_px", [])
    if not isinstance(masks, list):
        raise ValueError("ignore_rects_px must be a list")
    for idx, item in enumerate(masks):
        if not isinstance(item, dict) or not isinstance(item.get("reason"), str) or not item["reason"].strip():
            raise ValueError(f"mask {idx} requires a nonempty reason")
        ensure_rect(item.get("rect_px"), width, height, f"mask {idx}")
    limits = scene.get("tolerances", {})
    if not isinstance(limits, dict):
        raise ValueError("tolerances must be an object")
    for key, value in limits.items():
        if key not in DEFAULT_LIMITS or not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value):
            raise ValueError(f"invalid tolerance field {key}")
    return reference_path, width, height


def load_rgb(path: Path, matte: list[int]) -> np.ndarray:
    with Image.open(path) as src:
        if src.format != "PNG":
            raise ValueError(f"file must be PNG: {path}")
        rgba = src.convert("RGBA")
        background = Image.new("RGBA", rgba.size, tuple(matte) + (255,))
        merged = Image.alpha_composite(background, rgba).convert("RGB")
        return np.asarray(merged, dtype=np.uint8)


def dilated(binary: np.ndarray) -> np.ndarray:
    h, w = binary.shape
    padded = np.pad(binary, 1, constant_values=False)
    expanded = np.zeros((h, w), dtype=bool)
    for dy in range(3):
        for dx in range(3):
            expanded |= padded[dy:dy + h, dx:dx + w]
    return expanded


def edges(img: np.ndarray, threshold: int) -> np.ndarray:
    lum = np.sum(img.astype(np.float32) * np.array([0.299, 0.587, 0.114], np.float32), axis=2)
    grad = np.zeros(lum.shape, dtype=np.float32)
    grad[:, 1:] = np.maximum(grad[:, 1:], np.abs(lum[:, 1:] - lum[:, :-1]))
    grad[:, :-1] = np.maximum(grad[:, :-1], np.abs(lum[:, 1:] - lum[:, :-1]))
    grad[1:, :] = np.maximum(grad[1:, :], np.abs(lum[1:, :] - lum[:-1, :]))
    grad[:-1, :] = np.maximum(grad[:-1, :], np.abs(lum[1:, :] - lum[:-1, :]))
    return grad > threshold


def measure(ref: np.ndarray, cand: np.ndarray, valid: np.ndarray, thresholds: dict[str, float]) -> dict[str, float | int]:
    if not valid.any():
        raise ValueError("no valid pixels in region after masks")
    delta = np.abs(ref.astype(np.int16) - cand.astype(np.int16))
    selected = delta[valid]
    maxima = delta.max(axis=2)
    changed_fraction = float(np.count_nonzero(maxima[valid] > thresholds["pixel_delta_threshold"]) / np.count_nonzero(valid))
    edge_a = edges(ref, int(thresholds["edge_threshold"])) & valid
    edge_b = edges(cand, int(thresholds["edge_threshold"])) & valid
    count_a = int(edge_a.sum())
    count_b = int(edge_b.sum())
    if count_a == 0 and count_b == 0:
        ef1 = 1.0
    elif count_a == 0 or count_b == 0:
        ef1 = 0.0
    else:
        precision = float((edge_b & dilated(edge_a)).sum() / count_b)
        recall = float((edge_a & dilated(edge_b)).sum() / count_a)
        ef1 = (2 * precision * recall / (precision + recall)) if precision + recall else 0.0
    square = selected.astype(np.float64) ** 2
    return {
        "pixels_evaluated": int(valid.sum()),
        "mae_rgb": round(float(selected.mean()), 6),
        "rmse_rgb": round(float(np.sqrt(square.mean())), 6),
        "changed_fraction": round(changed_fraction, 6),
        "edge_f1": round(ef1, 6),
        "reference_edge_pixels": count_a,
        "candidate_edge_pixels": count_b,
    }


def check_gates(metrics: dict[str, Any], limits: dict[str, float]) -> list[str]:
    errors = []
    if metrics["mae_rgb"] > limits["max_mae"]:
        errors.append(f"mae_rgb {metrics['mae_rgb']:.3f} > {limits['max_mae']}")
    if metrics["changed_fraction"] > limits["max_changed_fraction"]:
        errors.append(f"changed_fraction {metrics['changed_fraction']:.4f} > {limits['max_changed_fraction']}")
    if metrics["edge_f1"] < limits["min_edge_f1"]:
        errors.append(f"edge_f1 {metrics['edge_f1']:.4f} < {limits['min_edge_f1']}")
    return errors


def diagnose_shift(ref: np.ndarray, cand: np.ndarray, valid: np.ndarray, radius: int) -> dict[str, float | int] | None:
    if not radius:
        return None
    h, w = valid.shape
    best = (float("inf"), 0, 0)
    for dy in range(-radius, radius + 1):
        for dx in range(-radius, radius + 1):
            xa, xb = max(0, -dx), min(w, w - dx)
            ya, yb = max(0, -dy), min(h, h - dy)
            if xa >= xb or ya >= yb:
                continue
            mask = valid[ya:yb, xa:xb]
            if not mask.any():
                continue
            a = ref[ya:yb, xa:xb]
            b = cand[ya + dy:yb + dy, xa + dx:xb + dx]
            score = float(np.abs(a.astype(np.int16) - b.astype(np.int16))[mask].mean())
            if score < best[0]:
                best = (score, dx, dy)
    return {"candidate_offset_x": best[1], "candidate_offset_y": best[2], "mae_diagnostic_only": round(best[0], 6)}


def annotated_comparison(ref: np.ndarray, cand: np.ndarray, heat: np.ndarray) -> Image.Image:
    h, w = ref.shape[:2]
    panel = Image.new("RGB", (w * 3, h + 28), "#F4F5F7")
    for idx, (title, array) in enumerate([
        ("Reference (unchanged)", ref),
        ("Candidate (unchanged)", cand),
        ("Absolute difference heatmap", heat),
    ]):
        panel.paste(Image.fromarray(array, "RGB"), (idx * w, 28))
        ImageDraw.Draw(panel).text((idx * w + 6, 7), title, fill="black")
    return panel


def make_report_md(report: dict[str, Any]) -> str:
    lines = ["# egui visual comparison", "", f"**Result:** {'PASS' if report['passed'] else 'FAIL'}", "",
             f"**Reference:** `{report['reference']}`", f"**Candidate:** `{report['candidate']}`", "",
             f"**Reference state:** `{report['reference_state']}`", "",
             f"**Valid pixels:** {report['global']['pixels_evaluated']}", "",
             "| Region | MAE | Changed % | Edge F1 | Gate |", "|---|---:|---:|---:|---|"]
    all_entries = [("whole image", report["global"], report["global_failures"]) ]
    all_entries += [(r["id"], r["metrics"], r["failures"]) for r in report["regions"]]
    for name, metrics, failures in all_entries:
        lines.append(f"| {name} | {metrics['mae_rgb']:.2f} | {metrics['changed_fraction']*100:.2f}% | {metrics['edge_f1']:.3f} | {'PASS' if not failures else 'FAIL'} |")
    lines += ["", "## Failing checks", ""]
    for name, _, failures in all_entries:
        for f in failures:
            lines.append(f"- **{name}:** {f}")
    if report.get("shift_diagnostic"):
        lines += ["", "## Diagnostic only", "", f"Candidate translation estimate: `{report['shift_diagnostic']}`. It was **not** applied to scores."]
    if not any(x[2] for x in all_entries):
        lines.append("- None")
    lines += ["", "Generated image files: `overlay.png`, `heatmap.png`, `comparison.png`. Do not use the heatmap as a replacement for manual inspection.", ""]
    return "\n".join(lines)


def run(scene_path: Path, candidate_path: Path, output_dir: Path, tag: str | None,
        case_id: str | None, run_id: str | None, shift_radius: int) -> dict[str, Any]:
    scene_path = scene_path.resolve()
    candidate_path = candidate_path.resolve()
    scene = json.loads(scene_path.read_text(encoding="utf-8"))
    reference_path, width, height = validate_scene(scene, scene_path)
    matte = scene.get("matte_rgb", [255, 255, 255])
    if not isinstance(matte, list) or len(matte) != 3 or not all(type(c) is int and 0 <= c <= 255 for c in matte):
        raise ValueError("matte_rgb must be [R,G,B] with integers in 0..255")
    ref, cand = load_rgb(reference_path, matte), load_rgb(candidate_path, matte)
    if ref.shape[:2] != (height, width):
        raise ValueError(f"recorded reference.pixel_size {width}x{height} differs from actual {ref.shape[1]}x{ref.shape[0]}")
    if cand.shape != ref.shape:
        raise ValueError(f"mismatched images: reference {width}x{height}, candidate {cand.shape[1]}x{cand.shape[0]}. Fix capture, not the comparator")
    limits: dict[str, float] = {**DEFAULT_LIMITS, **scene.get("tolerances", {})}
    for field in ("max_changed_fraction", "min_edge_f1", "max_mask_fraction"):
        if not 0 <= limits[field] <= 1:
            raise ValueError(f"{field} must be in [0,1]")
    if limits["max_mae"] < 0 or not 0 <= limits["pixel_delta_threshold"] <= 255 or not 0 <= limits["edge_threshold"] <= 255:
        raise ValueError("invalid pixel threshold or MAE tolerance")
    valid = np.ones((height, width), dtype=bool)
    for item in scene.get("ignore_rects_px", []):
        x, y, w, h = item["rect_px"]
        valid[y:y+h, x:x+w] = False
    mask_fraction = 1 - float(valid.mean())
    if mask_fraction > limits["max_mask_fraction"]:
        raise ValueError(f"ignored area {mask_fraction:.2%} exceeds max_mask_fraction {limits['max_mask_fraction']:.2%}")
    global_metrics = measure(ref, cand, valid, limits)
    global_failures = check_gates(global_metrics, limits)
    region_data = []
    for region in scene["regions"]:
        x, y, w, h = region["rect_px"]
        region_limits = {**limits, **region.get("quality", {})}
        for field in ("max_changed_fraction", "min_edge_f1"):
            if not 0 <= region_limits[field] <= 1:
                raise ValueError(f"invalid region threshold for {region['id']}: {field}")
        if (region_limits["max_mae"] < 0 or
            not 0 <= region_limits["pixel_delta_threshold"] <= 255 or
            not 0 <= region_limits["edge_threshold"] <= 255):
            raise ValueError(f"invalid region threshold for {region['id']}: pixel/MAE")
        metrics = measure(ref[y:y+h, x:x+w], cand[y:y+h, x:x+w], valid[y:y+h, x:x+w], region_limits)
        failures = check_gates(metrics, region_limits)
        region_data.append({"id": region["id"], "rect_px": region["rect_px"],
                            "weight": region.get("weight", 1), "metrics": metrics,
                            "quality": region_limits, "failures": failures, "passed": not failures})
    region_data.sort(key=lambda r: (r["passed"], -r["weight"] * (
        r["metrics"]["mae_rgb"] / max(r["quality"]["max_mae"], 1e-6) +
        r["metrics"]["changed_fraction"] / max(r["quality"]["max_changed_fraction"], 1e-6))))
    quality_contract = {
        "reference_sha256": scene["reference"]["sha256"],
        "reference_state": scene["reference"]["state"],
        "reference_pixel_size": scene["reference"]["pixel_size"],
        "pixels_per_point": scene["reference"]["pixels_per_point"],
        "content_rect_px": scene["reference"]["content_rect_px"],
        "regions": scene["regions"],
        "ignore_rects_px": scene.get("ignore_rects_px", []),
        "tolerances": limits,
        "matte_rgb": matte,
    }
    quality_hash = hashlib.sha256(json.dumps(quality_contract, sort_keys=True, ensure_ascii=False).encode("utf-8")).hexdigest()
    report = {
        "schema": "egui-visual-forge.report.v1",
        "scene_quality_sha256": quality_hash,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "tag": tag, "case_id": case_id, "run_id": run_id,
        "scene_file": str(scene_path), "reference": str(reference_path),
        "reference_sha256": scene["reference"]["sha256"], "reference_state": scene["reference"]["state"],
        "candidate": str(candidate_path), "candidate_sha256": sha256_file(candidate_path),
        "pixel_size": [width, height], "pixels_per_point": scene["reference"]["pixels_per_point"],
        "ignored_fraction": round(mask_fraction, 6),
        "global": global_metrics, "global_limits": limits, "global_failures": global_failures,
        "regions": region_data,
        "passed": not global_failures and all(r["passed"] for r in region_data),
        "shift_diagnostic": diagnose_shift(ref, cand, valid, shift_radius),
    }
    output_dir.mkdir(parents=True, exist_ok=True)
    overlay = np.rint(ref.astype(np.float32)*0.5 + cand.astype(np.float32)*0.5).astype(np.uint8)
    delta = np.max(np.abs(ref.astype(np.int16) - cand.astype(np.int16)), axis=2).astype(np.float32)
    intensity = np.clip(delta * 2.5, 0, 255).astype(np.uint8)
    heat = np.stack([intensity, np.minimum(intensity, 165), np.zeros_like(intensity)], axis=2)
    heat[~valid] = [72, 72, 72]
    Image.fromarray(overlay, "RGB").save(output_dir / "overlay.png")
    Image.fromarray(heat, "RGB").save(output_dir / "heatmap.png")
    annotated_comparison(ref, cand, heat).save(output_dir / "comparison.png")
    (output_dir / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    (output_dir / "report.md").write_text(make_report_md(report), encoding="utf-8")
    return report


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scene", required=True, type=Path, help="JSON manifest created by init_scene.py")
    parser.add_argument("--candidate", required=True, type=Path, help="PNG captured from actual egui UI")
    parser.add_argument("--out", required=True, type=Path, help="output folder for PNG and JSON QA artifacts")
    parser.add_argument("--tag", help="experiment condition, e.g. baseline/with-skills")
    parser.add_argument("--case-id", help="controlled benchmark case id")
    parser.add_argument("--run-id", help="replicate identifier")
    parser.add_argument("--diagnose-shift", type=int, default=0, metavar="PX", help="diagnose +/- N px offsets; never changes official metrics")
    args = parser.parse_args(argv)
    if not 0 <= args.diagnose_shift <= 8:
        parser.error("--diagnose-shift must be 0..8 pixels")
    try:
        report = run(args.scene, args.candidate, args.out, args.tag, args.case_id, args.run_id, args.diagnose_shift)
    except (ValueError, OSError, KeyError, TypeError, json.JSONDecodeError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    print(f"{'PASS' if report['passed'] else 'FAIL'} | MAE={report['global']['mae_rgb']:.2f} | changed={report['global']['changed_fraction']:.2%} | edge F1={report['global']['edge_f1']:.3f}")
    print(f"Reports: {args.out / 'report.json'} and {args.out / 'comparison.png'}")
    return 0 if report["passed"] else 3


if __name__ == "__main__":
    sys.exit(main())
