"""PDF / EPUB / MOBI text extraction for Rgain sidecar."""

from __future__ import annotations

import contextlib
import io
import logging
import os
import shutil
from pathlib import Path
from typing import Any

_LOG = logging.getLogger("rgain.extract")


def _ensure_file_logging() -> None:
    if getattr(_ensure_file_logging, "_done", False):
        return
    log_dir = os.environ.get("RGAIN_LOG_DIR", "").strip()
    if not log_dir:
        setattr(_ensure_file_logging, "_done", True)
        return
    try:
        Path(log_dir).mkdir(parents=True, exist_ok=True)
        handler = logging.FileHandler(Path(log_dir) / "sidecar_extract.log", encoding="utf-8")
        handler.setFormatter(logging.Formatter("%(asctime)s [%(levelname)s] %(message)s"))
        root = logging.getLogger("rgain")
        root.setLevel(logging.INFO)
        if not any(isinstance(h, logging.FileHandler) for h in root.handlers):
            root.addHandler(handler)
    except Exception:  # noqa: BLE001
        pass
    setattr(_ensure_file_logging, "_done", True)


def extract_file(path: str) -> dict[str, Any]:
    _ensure_file_logging()
    p = Path(path)
    if not p.is_file():
        _LOG.error(
            "not_a_file path=%r exists=%s parent_exists=%s",
            path,
            p.exists(),
            p.parent.exists() if str(p.parent) else False,
        )
        return {"ok": False, "error": f"not_a_file: {path}"}

    suffix = p.suffix.lower()
    _LOG.info("extract_begin path=%s suffix=%s", path, suffix)
    if suffix == ".pdf":
        result = _extract_pdf(p)
    elif suffix == ".epub":
        result = _extract_epub(p)
    elif suffix == ".mobi":
        result = _extract_mobi(p)
    else:
        result = {"ok": False, "error": f"unsupported_kind: {suffix}"}

    if result.get("ok"):
        text = (result.get("text") or "").strip()
        _LOG.info(
            "extract_ok path=%s kind=%s chars=%s warnings=%s",
            path,
            result.get("kind"),
            len(text),
            len(result.get("warnings") or []),
        )
        if not text:
            result = {
                "ok": False,
                "error": "extract_empty_text",
                "warnings": list(result.get("warnings") or []),
            }
            _LOG.warning("extract_empty path=%s", path)
    else:
        _LOG.error("extract_fail path=%s error=%s", path, result.get("error"))
    return result


def _extract_pdf(path: Path) -> dict[str, Any]:
    try:
        import pymupdf as fitz
    except ImportError:
        try:
            import fitz  # legacy pymupdf alias
        except ImportError:
            return {"ok": False, "error": "pymupdf_not_installed"}

    warnings: list[str] = []
    parts: list[str] = []
    try:
        doc = fitz.open(path)
    except Exception as err:  # noqa: BLE001
        return {"ok": False, "error": f"pdf_open_failed: {err}"}

    try:
        for i, page in enumerate(doc):
            text = (page.get_text("text") or "").strip()
            if text:
                parts.append(f"## page {i + 1}\n{text}")
        if not parts:
            warnings.append("pdf_no_text_layer")
        return {
            "ok": True,
            "kind": "pdf",
            "text": "\n\n".join(parts),
            "pages": doc.page_count,
            "warnings": warnings,
        }
    finally:
        doc.close()


def _html_to_text(raw: str) -> str:
    try:
        from lxml import html as lxml_html

        tree = lxml_html.fromstring(raw)
        return " ".join(tree.text_content().split())
    except Exception:
        # very small fallback
        import re

        return re.sub(r"<[^>]+>", " ", raw)


def _extract_epub(path: Path, kind: str = "epub") -> dict[str, Any]:
    try:
        from ebooklib import epub, ITEM_DOCUMENT
    except ImportError:
        return {"ok": False, "error": "ebooklib_not_installed"}

    warnings: list[str] = []
    parts: list[str] = []
    try:
        book = epub.read_epub(str(path))
    except Exception as err:  # noqa: BLE001
        return {"ok": False, "error": f"epub_open_failed: {err}"}

    for item in book.get_items_of_type(ITEM_DOCUMENT):
        try:
            raw = item.get_content().decode("utf-8", errors="ignore")
        except Exception:
            warnings.append(f"epub_decode_failed:{item.get_name()}")
            continue
        text = _html_to_text(raw)
        if text:
            parts.append(f"## {item.get_name()}\n{text}")

    if not parts:
        warnings.append("epub_no_text")

    return {
        "ok": True,
        "kind": kind,
        "text": "\n\n".join(parts),
        "chapters": len(parts),
        "warnings": warnings,
    }


def _extract_html_file(path: Path, kind: str = "mobi") -> dict[str, Any]:
    warnings: list[str] = []
    try:
        raw = path.read_text(encoding="utf-8", errors="ignore")
    except Exception as err:  # noqa: BLE001
        return {"ok": False, "error": f"mobi_html_read_failed: {err}"}
    text = _html_to_text(raw)
    if not text:
        warnings.append("mobi_no_text")
    return {
        "ok": True,
        "kind": kind,
        "text": text,
        "chapters": 1 if text else 0,
        "warnings": warnings,
    }


def _find_unpacked_candidate(tempdir: Path, preferred: Path | None) -> Path | None:
    if preferred is not None and preferred.is_file():
        return preferred
    for pattern in ("*.epub", "*.html", "*.htm", "*.xhtml", "*.pdf"):
        hits = sorted(tempdir.rglob(pattern))
        if hits:
            return hits[0]
    return preferred if preferred and preferred.exists() else None


def _extract_mobi(path: Path) -> dict[str, Any]:
    try:
        import mobi
    except ImportError:
        return {"ok": False, "error": "mobi_not_installed"}

    tempdir = None
    sink_out = io.StringIO()
    sink_err = io.StringIO()
    try:
        # mobi/loguru may write to stdout and break the sidecar JSON line protocol.
        with contextlib.redirect_stdout(sink_out), contextlib.redirect_stderr(sink_err):
            tempdir, unpacked = mobi.extract(str(path))
    except Exception as err:  # noqa: BLE001
        noise = (sink_out.getvalue() + sink_err.getvalue()).strip()
        _LOG.error("mobi_open_failed path=%s err=%s noise=%s", path, err, noise[:500])
        return {"ok": False, "error": f"mobi_open_failed: {err}"}

    noise = (sink_out.getvalue() + sink_err.getvalue()).strip()
    if noise:
        _LOG.info("mobi_extract_noise path=%s bytes=%s", path, len(noise))

    try:
        temp_path = Path(tempdir) if tempdir else None
        unpacked_path = _find_unpacked_candidate(
            temp_path if temp_path else Path("."),
            Path(unpacked) if unpacked else None,
        )
        if unpacked_path is None:
            return {"ok": False, "error": "mobi_unpacked_missing"}

        suffix = unpacked_path.suffix.lower()
        _LOG.info(
            "mobi_unpacked path=%s unpacked=%s suffix=%s",
            path,
            unpacked_path,
            suffix,
        )
        if suffix == ".epub":
            result = _extract_epub(unpacked_path, kind="mobi")
        elif suffix in {".html", ".htm", ".xhtml"}:
            result = _extract_html_file(unpacked_path, kind="mobi")
        elif suffix == ".pdf":
            result = _extract_pdf(unpacked_path)
            if result.get("ok"):
                warnings = list(result.get("warnings") or [])
                warnings.append("mobi_print_replica_pdf")
                result["warnings"] = warnings
        else:
            return {
                "ok": False,
                "error": f"mobi_unsupported_unpacked:{suffix or 'none'}",
            }
        if result.get("ok"):
            result["kind"] = "mobi"
        return result
    finally:
        if tempdir:
            shutil.rmtree(tempdir, ignore_errors=True)
