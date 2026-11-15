"""Manual smoke test for stdio protocol. Run:
E:\\Conda\\envs\\vi\\python.exe python/smoke_test.py
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PYTHON = sys.executable


def main() -> None:
    proc = subprocess.Popen(
        [PYTHON, "-u", "-m", "rgain_parser"],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )
    assert proc.stdin and proc.stdout
    ready = proc.stdout.readline()
    print("ready:", ready.strip())
    proc.stdin.write(json.dumps({"id": 1, "cmd": "health"}) + "\n")
    proc.stdin.flush()
    print("health:", proc.stdout.readline().strip())
    proc.stdin.write(
        json.dumps({"id": 2, "cmd": "echo", "payload": {"ping": "rgain"}}) + "\n"
    )
    proc.stdin.flush()
    print("echo:", proc.stdout.readline().strip())

    # optional extract self-check if deps present
    try:
        import ebooklib  # noqa: F401
        import pymupdf  # noqa: F401
    except ImportError as err:
        print("extract_deps_skip:", err)
    else:
        print("extract_deps: ok")

    proc.kill()
    proc.wait(timeout=3)
    print("ok")


if __name__ == "__main__":
    main()
