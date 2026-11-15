"""PDF / EPUB text extraction for Rgain sidecar."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def extract_file(path: str) -> dict[str, Any]:
    p = Path(path)
    if not p.is_file():
        return {"ok": False, "error": f"not_a_file: {path}"}

    suffix = p.suffix.lower()
    if suffix == ".pdf":
        return _extract_pdf(p)
    if suffix == ".epub":
        return _extract_epub(p)
    return {"ok": False, "error": f"unsupported_kind: {suffix}"}


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


def _extract_epub(path: Path) -> dict[str, Any]:
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
        "kind": "epub",
        "text": "\n\n".join(parts),
        "chapters": len(parts),
        "warnings": warnings,
    }
