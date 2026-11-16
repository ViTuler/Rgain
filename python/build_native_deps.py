#!/usr/bin/env python3
"""Copy OpenSSL DLLs required by SQLCipher into src-tauri/native/ for Tauri bundle."""

from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path

# Must match the PE import name of release rgain.exe (bundled-sqlcipher + conda OpenSSL).
REQUIRED_DLLS = ("libcrypto-3-x64.dll",)


def resolve_openssl_bin() -> Path | None:
    openssl_dir = os.environ.get("OPENSSL_DIR", "").strip()
    candidates: list[Path] = []
    if openssl_dir:
        root = Path(openssl_dir)
        candidates.extend([root / "bin", root])
    # Fallback matching src-tauri/.cargo/config.toml default.
    candidates.append(Path(r"E:/Conda/envs/rgain/Library/bin"))
    for path in candidates:
        if path.is_dir():
            return path
    return None


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    native_dir = repo_root / "src-tauri" / "native"
    native_dir.mkdir(parents=True, exist_ok=True)

    openssl_bin = resolve_openssl_bin()
    if openssl_bin is None:
        print(
            "error: OPENSSL_DIR/bin not found; cannot stage libcrypto for MSI",
            file=sys.stderr,
        )
        return 1

    for name in REQUIRED_DLLS:
        src = openssl_bin / name
        if not src.is_file():
            print(f"error: missing {src}", file=sys.stderr)
            return 1
        dest = native_dir / name
        shutil.copy2(src, dest)
        print(f"copied {src} -> {dest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
