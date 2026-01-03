#!/usr/bin/env python3
"""
Count code lines in Astra GUI categories using `tokei` JSON output.

Categories:
- library: crates/astra-gui*/src (astra-gui, astra-gui-fonts, astra-gui-text, astra-gui-interactive, astra-gui-wgpu)
- examples: crates/astra-gui-wgpu/examples (shared/ + each *.rs example)

Defaults:
- counts Rust *code* lines only (tokei: Rust.code)

Flags:
- --full / -f:
    show per-subpart breakdown
    - library: per crate
    - examples: shared + each example file
- --detailed / -d:
    show Rust code + Markdown lines + Total (= Rust code + Markdown lines)
    (Markdown lines come from actual .md files in the scanned path)
- --color:
    colorize output (Catppuccin Blue for library, Mauve for examples) using ANSI escape codes
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

# Optional (nice table formatting). We'll fall back if unavailable.
try:
    from tabulate import tabulate  # type: ignore
except Exception:  # pragma: no cover
    tabulate = None  # type: ignore


REPO_ROOT_MARKER = "Cargo.toml"

# Catppuccin (approx) ANSI 24-bit colors:
# Blue  = #89B4FA (Mocha Blue)
# Mauve = #CBA6F7 (Mocha Mauve)
ANSI_RESET = "\x1b[0m"
ANSI_BLUE = "\x1b[38;2;137;180;250m"
ANSI_MAUVE = "\x1b[38;2;203;166;247m"


@dataclass(frozen=True)
class Stats:
    rust_code: int = 0
    md_lines: int = 0

    @property
    def total(self) -> int:
        return self.rust_code + self.md_lines

    def __add__(self, other: "Stats") -> "Stats":
        return Stats(self.rust_code + other.rust_code, self.md_lines + other.md_lines)


@dataclass(frozen=True)
class Entry:
    category: str  # "library" | "examples"
    name: str  # "library" root or subpart name like "astra-gui"
    path: str
    stats: Stats


def _should_colorize(color_flag: bool) -> bool:
    # Only colorize when explicitly requested and stdout is a TTY.
    return bool(color_flag and sys.stdout.isatty())


def _color_for_category(category: str) -> str:
    if category == "library":
        return ANSI_BLUE
    if category == "examples":
        return ANSI_MAUVE
    return ""


def _colorize(s: str, category: str, enabled: bool) -> str:
    if not enabled:
        return s
    c = _color_for_category(category)
    if not c:
        return s
    return f"{c}{s}{ANSI_RESET}"


def _repo_root(cwd: Path) -> Path:
    """
    Find the repo root by walking up to the first directory that contains Cargo.toml.
    """
    cur = cwd.resolve()
    for _ in range(20):
        if (cur / REPO_ROOT_MARKER).is_file():
            return cur
        if cur.parent == cur:
            break
        cur = cur.parent
    raise SystemExit(
        f"count.py: run from repo root (expected {REPO_ROOT_MARKER} in cwd or parent dirs)"
    )


def _run_tokei_json(paths: Sequence[Path]) -> Dict[str, Any]:
    """
    Run tokei with JSON output over one or more paths.
    Returns parsed JSON dict.
    """
    cmd = ["tokei", "--output", "json", "--", *[str(p) for p in paths]]
    try:
        proc = subprocess.run(
            cmd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )
    except FileNotFoundError:
        raise SystemExit("count.py: `tokei` not found in PATH")
    except subprocess.CalledProcessError as e:
        # Include stderr for debugging.
        stderr = (e.stderr or "").strip()
        raise SystemExit(f"count.py: tokei failed: {stderr or e}")

    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as e:
        raise SystemExit(f"count.py: failed to parse tokei JSON: {e}")


def _int_or_zero(v: Any) -> int:
    try:
        if v is None:
            return 0
        return int(v)
    except Exception:
        return 0


def _extract_stats(tokei_json: Dict[str, Any]) -> Stats:
    """
    Convert tokei JSON to our Stats:
    - Rust: code lines only (.Rust.code)
    - Markdown: total lines (.Markdown.lines)
    """
    rust = tokei_json.get("Rust") or {}
    md = tokei_json.get("Markdown") or {}

    rust_code = _int_or_zero(rust.get("code"))
    md_lines = _int_or_zero(md.get("lines"))

    return Stats(rust_code=rust_code, md_lines=md_lines)


def _stats_for_path(path: Path) -> Stats:
    return _extract_stats(_run_tokei_json([path]))


def _library_rows(repo_root: Path) -> List[Tuple[str, Path]]:
    return [
        ("astra-gui", repo_root / "crates/astra-gui/src"),
        ("astra-gui-fonts", repo_root / "crates/astra-gui-fonts/src"),
        ("astra-gui-text", repo_root / "crates/astra-gui-text/src"),
        ("astra-gui-interactive", repo_root / "crates/astra-gui-interactive/src"),
        ("astra-gui-wgpu", repo_root / "crates/astra-gui-wgpu/src"),
    ]


def _examples_rows(repo_root: Path) -> List[Tuple[str, Path]]:
    base = repo_root / "crates/astra-gui-wgpu/examples"
    rows: List[Tuple[str, Path]] = []

    shared = base / "shared"
    if shared.is_dir():
        rows.append(("shared", shared))

    # Each example file, label without .rs
    if base.is_dir():
        for p in sorted(base.glob("*.rs")):
            rows.append((p.stem, p))

    return rows


def _sum_stats(rows: Iterable[Tuple[str, Path]]) -> Stats:
    total = Stats()
    for _, p in rows:
        total += _stats_for_path(p)
    return total


def _make_entries(repo_root: Path, full: bool) -> List[Entry]:
    """
    Build entries in output order:
    - library root + optional children
    - examples root + optional children
    """
    entries: List[Entry] = []

    lib_rows = _library_rows(repo_root)
    ex_rows = _examples_rows(repo_root)

    lib_total = _sum_stats(lib_rows)
    ex_total = _sum_stats(ex_rows)

    entries.append(
        Entry(category="library", name="library", path=str(repo_root), stats=lib_total)
    )
    if full:
        for name, p in lib_rows:
            entries.append(
                Entry(
                    category="library", name=name, path=str(p), stats=_stats_for_path(p)
                )
            )

    entries.append(
        Entry(category="examples", name="examples", path=str(repo_root), stats=ex_total)
    )
    if full:
        for name, p in ex_rows:
            entries.append(
                Entry(
                    category="examples",
                    name=name,
                    path=str(p),
                    stats=_stats_for_path(p),
                )
            )

    return entries


def _render_table(entries: List[Entry], detailed: bool, full: bool, color: bool) -> str:
    """
    Render a single combined table for library + examples.

    We insert a visual separator row between categories:
    - With tabulate (github), this is rendered as a row whose cells are the right
      number of dashes so it appears like a horizontal divider.
    - In the minimal fallback renderer, it's also rendered as a dashed row.

    Optional coloring:
    - everything in the library portion is colored Catppuccin Blue
    - everything in the examples portion is colored Catppuccin Mauve

    Note: we only colorize if stdout is a TTY and --color is provided.
    """
    color_enabled = _should_colorize(color)

    def rows_for(cat: str) -> List[Entry]:
        return [e for e in entries if e.category == cat]

    def as_rows(cat_entries: List[Entry], category: str) -> List[List[str]]:
        out: List[List[str]] = []
        for e in cat_entries:
            is_root = e.name == e.category
            label = e.name
            if full and not is_root:
                label = "├─ " + label

            if detailed:
                row = [
                    label,
                    str(e.stats.rust_code),
                    str(e.stats.md_lines),
                    str(e.stats.total),
                ]
            else:
                row = [label, str(e.stats.rust_code)]

            out.append([_colorize(cell, category, color_enabled) for cell in row])
        return out

    lib_entries = rows_for("library")
    ex_entries = rows_for("examples")

    if detailed:
        headers = ["Name", "Rust", "Markdown", "Total"]
    else:
        headers = ["Name", "Rust"]

    data: List[List[str]] = []
    if lib_entries:
        data += as_rows(lib_entries, "library")
    if lib_entries and ex_entries:
        # Separator row: uncolored
        data.append(["-" * len(h) for h in headers])
    if ex_entries:
        data += as_rows(ex_entries, "examples")

    if tabulate is not None:
        return tabulate(data, headers=headers, tablefmt="github")
    else:
        # Minimal fallback: compute column widths and format.
        # Since ANSI codes impact string length but not visible width, avoid color in fallback.
        # (We already gated coloring to TTY; this fallback is mainly for non-tabulate setups.)
        if color_enabled:
            # Strip color in fallback to avoid misalignment.
            data_plain: List[List[str]] = []
            for row in data:
                data_plain.append(
                    [
                        cell.replace(ANSI_BLUE, "")
                        .replace(ANSI_MAUVE, "")
                        .replace(ANSI_RESET, "")
                        for cell in row
                    ]
                )
            data = data_plain

        widths = [len(h) for h in headers]
        for row in data:
            for j, cell in enumerate(row):
                widths[j] = max(widths[j], len(cell))

        def fmt_row(row: Sequence[str]) -> str:
            return "  ".join(cell.ljust(widths[j]) for j, cell in enumerate(row))

        lines = [fmt_row(headers), fmt_row(["-" * w for w in widths])]
        lines += [fmt_row(r) for r in data]
        return "\n".join(lines)


def _parse_args(argv: Sequence[str]) -> argparse.Namespace:
    # argparse already supports combined short flags like -dfc as long as each flag
    # is a simple store_true option (no parameters).
    p = argparse.ArgumentParser(prog="count.py", add_help=True)
    p.add_argument(
        "-f", "--full", action="store_true", help="Show per-subpart breakdown"
    )
    p.add_argument(
        "-d", "--detailed", action="store_true", help="Show Rust + Markdown + Total"
    )
    p.add_argument(
        "-c",
        "--color",
        action="store_true",
        help="Colorize output (Catppuccin Blue for library, Mauve for examples). Only applies on TTY.",
    )
    return p.parse_args(argv)


def main(argv: Sequence[str]) -> int:
    args = _parse_args(argv)
    repo_root = _repo_root(Path.cwd())

    # Ensure we’re reasonably likely to be in the right place (helpful error early)
    if not (repo_root / "crates").is_dir():
        print("count.py: expected `crates/` directory at repo root", file=sys.stderr)
        return 1

    entries = _make_entries(repo_root, full=args.full)
    print(
        _render_table(entries, detailed=args.detailed, full=args.full, color=args.color)
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
