"""stdio JSON line protocol for Rgain sidecar."""

from __future__ import annotations

import io
import json
import os
import sys
from typing import Any, Optional, TextIO

from . import __version__
from .extract import extract_file


def _reopen_fd(fd: int, mode: str) -> Optional[TextIO]:
    """Rebind a stdio stream from an inherited OS handle (windowed PyInstaller)."""
    try:
        # Validate the fd is usable before wrapping.
        os.fstat(fd)
    except OSError:
        return None
    try:
        raw = open(fd, mode + "b", buffering=0, closefd=False)  # noqa: SIM115
        return io.TextIOWrapper(
            raw,
            encoding="utf-8",
            errors="replace",
            line_buffering=True,
            write_through=True,
        )
    except OSError:
        return None


def _ensure_stdio() -> None:
    """
    PyInstaller console=False leaves sys.stdin/stdout/stderr as None when not
    attached to a console. When Rgain spawns us with pipes, the fds are still
    valid — reopen them so the JSON line protocol works.
    """
    if sys.stdin is None:
        stream = _reopen_fd(0, "r")
        if stream is not None:
            sys.stdin = stream
    if sys.stdout is None:
        stream = _reopen_fd(1, "w")
        if stream is not None:
            sys.stdout = stream
    if sys.stderr is None:
        stream = _reopen_fd(2, "w")
        if stream is not None:
            sys.stderr = stream

    for stream in (sys.stdin, sys.stdout, sys.stderr):
        if stream is None:
            continue
        try:
            stream.reconfigure(encoding="utf-8")  # type: ignore[attr-defined]
        except Exception:
            pass


def _write(obj: dict[str, Any]) -> None:
    out = sys.stdout
    if out is None:
        return
    out.write(json.dumps(obj, ensure_ascii=False) + "\n")
    out.flush()


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
                "env": "rgain",
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
    _ensure_stdio()

    if sys.stdin is None or not hasattr(sys.stdin, "buffer") or sys.stdout is None:
        # Double-clicked windowed exe with no pipes: exit quietly.
        log_dir = os.environ.get("RGAIN_LOG_DIR", "").strip()
        if log_dir:
            try:
                os.makedirs(log_dir, exist_ok=True)
                with open(
                    os.path.join(log_dir, "sidecar_boot.log"),
                    "a",
                    encoding="utf-8",
                ) as fh:
                    fh.write("sidecar started without stdio pipes; exiting\n")
            except OSError:
                pass
        return

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
