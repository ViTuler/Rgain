"""stdio JSON line protocol for Rgain sidecar."""

from __future__ import annotations

import json
import sys
from typing import Any

from . import __version__
from .extract import extract_file


def _write(obj: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(obj, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def _handle(msg: dict[str, Any]) -> dict[str, Any]:
    req_id = msg.get("id")
    cmd = msg.get("cmd")

    if cmd == "health":
        return {
            "id": req_id,
            "ok": True,
            "data": {
                "status": "ok",
                "version": __version__,
                "env": "vi",
                "caps": ["extract_pdf", "extract_epub", "extract_mobi"],
            },
        }

    if cmd == "echo":
        return {
            "id": req_id,
            "ok": True,
            "data": msg.get("payload", {}),
        }

    if cmd == "extract":
        payload = msg.get("payload") or {}
        path = payload.get("path")
        if not isinstance(path, str) or not path:
            return {"id": req_id, "ok": False, "error": "missing_path"}
        result = extract_file(path)
        if not result.get("ok"):
            return {
                "id": req_id,
                "ok": False,
                "error": result.get("error", "extract_failed"),
            }
        return {"id": req_id, "ok": True, "data": result}

    return {
        "id": req_id,
        "ok": False,
        "error": f"unknown_cmd: {cmd}",
    }


def run() -> None:
    # Force UTF-8 on stdio so Chinese/Unicode book paths survive Windows pipes.
    for stream in (sys.stdin, sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8")  # type: ignore[attr-defined]
        except Exception:
            pass

    _write(
        {
            "ok": True,
            "event": "ready",
            "data": {"version": __version__},
        }
    )

    # Read bytes and decode as UTF-8; avoids cp936 stdin mangling paths from Rust.
    for raw in sys.stdin.buffer:
        text = raw.decode("utf-8", errors="replace").strip()
        if not text:
            continue
        try:
            msg = json.loads(text)
        except json.JSONDecodeError as err:
            _write({"ok": False, "error": f"invalid_json: {err}"})
            continue
        if not isinstance(msg, dict):
            _write({"ok": False, "error": "message_must_be_object"})
            continue
        _write(_handle(msg))
