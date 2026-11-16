"""Quick health + PDF extract check against the frozen sidecar exe."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def _request(proc: subprocess.Popen, msg: dict) -> dict:
    assert proc.stdin is not None and proc.stdout is not None
    proc.stdin.write(json.dumps(msg) + "\n")
    proc.stdin.flush()
    while True:
        line = proc.stdout.readline()
        if not line:
            raise RuntimeError("sidecar closed stdout")
        line = line.strip()
        if not line:
            continue
        try:
            data = json.loads(line)
        except json.JSONDecodeError:
            continue
        if data.get("id") == msg.get("id") or data.get("event") == "ready":
            return data


def main() -> int:
    exe = Path(__file__).resolve().parents[1] / "src-tauri" / "sidecars" / "rgain_parser.exe"
    if not exe.is_file():
        print(f"missing: {exe}", file=sys.stderr)
        return 1

    proc = subprocess.Popen(
        [str(exe)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    )
    assert proc.stdin is not None and proc.stdout is not None
    ready = proc.stdout.readline().strip()
    print("READY", ready)
    health = _request(proc, {"id": 1, "cmd": "health"})
    print("HEALTH", json.dumps(health, ensure_ascii=False))
    if not health.get("ok"):
        proc.kill()
        return 1

    # Minimal PDF via pymupdf in the build env is not required; write bytes by hand.
    # %PDF with one empty page is enough for open; prefer real tiny file from tempfile.
    try:
        import pymupdf
    except ImportError:
        print("skip extract smoke (pymupdf not in runner env)")
        proc.kill()
        return 0

    with tempfile.TemporaryDirectory() as tmp:
        pdf_path = Path(tmp) / "tiny.pdf"
        doc = pymupdf.open()
        page = doc.new_page()
        page.insert_text((72, 72), "rgain sidecar smoke")
        doc.save(pdf_path)
        doc.close()
        extract = _request(
            proc,
            {"id": 2, "cmd": "extract", "payload": {"path": str(pdf_path)}},
        )
        print("EXTRACT", json.dumps(extract, ensure_ascii=False)[:500])
        ok = bool(extract.get("ok"))
    proc.kill()
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
