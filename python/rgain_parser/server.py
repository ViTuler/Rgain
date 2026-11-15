"""stdio JSON line protocol for Rgain sidecar (Phase A: health + echo)."""

from __future__ import annotations

import json
import sys
from typing import Any

from . import __version__


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
            },
        }

    if cmd == "echo":
        return {
            "id": req_id,
            "ok": True,
            "data": msg.get("payload", {}),
        }

    return {
        "id": req_id,
        "ok": False,
        "error": f"unknown_cmd: {cmd}",
    }


def run() -> None:
    _write(
        {
            "ok": True,
            "event": "ready",
            "data": {"version": __version__},
        }
    )

    for line in sys.stdin:
        text = line.strip()
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
