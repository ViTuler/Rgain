#!/usr/bin/env python3
"""Build rgain_parser onefile sidecar and copy into src-tauri/sidecars/."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path


def main() -> int:
    python_dir = Path(__file__).resolve().parent
    repo_root = python_dir.parent
    spec_path = python_dir / "rgain_parser.spec"
    dist_dir = python_dir / "dist"
    sidecars_dir = repo_root / "src-tauri" / "sidecars"

    if "CONDA_DEFAULT_ENV" in os.environ and os.environ["CONDA_DEFAULT_ENV"] == "base":
        print(
            "error: refusing to build in conda base; activate env `rgain` first",
            file=sys.stderr,
        )
        return 1

    try:
        import PyInstaller  # noqa: F401
    except ImportError:
        print(
            "error: PyInstaller not installed in this interpreter\n"
            f"  python={sys.executable}\n"
            "  activate conda env `rgain` and: pip install pyinstaller",
            file=sys.stderr,
        )
        return 1

    if not spec_path.is_file():
        print(f"error: missing spec: {spec_path}", file=sys.stderr)
        return 1

    cmd = [
        sys.executable,
        "-m",
        "PyInstaller",
        "--noconfirm",
        "--clean",
        str(spec_path),
    ]
    print(f"building sidecar with: {' '.join(cmd)}")
    print(f"cwd={python_dir}")
    subprocess.run(cmd, cwd=python_dir, check=True)

    exe_name = "rgain_parser.exe" if os.name == "nt" else "rgain_parser"
    built = dist_dir / exe_name
    if not built.is_file():
        print(f"error: expected build output missing: {built}", file=sys.stderr)
        return 1

    sidecars_dir.mkdir(parents=True, exist_ok=True)
    dest = sidecars_dir / exe_name
    shutil.copy2(built, dest)
    print(f"copied {built} -> {dest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
