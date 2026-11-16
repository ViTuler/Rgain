# -*- mode: python ; coding: utf-8 -*-
"""PyInstaller spec for Rgain Python sidecar (onefile, slim).

Keeps PDF/EPUB/MOBI text extraction working while excluding PyMuPDF layout
ONNX models, devel artifacts, and heavy optional deps (numpy / onnxruntime).

Run from python/ (or via build_sidecar.py):
    pyinstaller rgain_parser.spec
"""

from pathlib import Path

block_cipher = None

spec_dir = Path(SPECPATH).resolve()

# Only modules our extract path actually imports.
hiddenimports = [
    "rgain_parser",
    "rgain_parser.server",
    "rgain_parser.extract",
    "pymupdf",
    "fitz",
    "ebooklib",
    "ebooklib.epub",
    "mobi",
    "mobi.kindleunpack",
    "lxml",
    "lxml.html",
    "lxml.etree",
    "lxml._elementpath",
    "bs4",
    "soupsieve",
]

# Never pull these into the sidecar (PyMuPDF layout / ML extras).
excludes = [
    "numpy",
    "numpy.core",
    "onnxruntime",
    "onnx",
    "torch",
    "tensorflow",
    "scipy",
    "pandas",
    "cv2",
    "PIL",
    "Pillow",
    "matplotlib",
    "sklearn",
    "pymupdf.layout",
    "pymupdf.layout.onnx",
    "pymupdf.tgif",
    "pymupdf.show_features",
    "pymupdf.features",
    "tkinter",
    "unittest",
    "test",
    "tests",
]


def _path_str(entry) -> str:
    # TOC entry may be (name, path, typecode) or similar.
    if isinstance(entry, (tuple, list)) and len(entry) >= 2:
        return str(entry[1]).replace("\\", "/").lower()
    return str(entry).replace("\\", "/").lower()


def _dest_str(entry) -> str:
    if isinstance(entry, (tuple, list)) and len(entry) >= 1:
        return str(entry[0]).replace("\\", "/").lower()
    return str(entry).replace("\\", "/").lower()


_DROP_MARKERS = (
    "/layout/",
    "\\layout\\",
    "layout/resources/onnx",
    "mupdf-devel",
    "/numpy/",
    "\\numpy\\",
    "onnxruntime",
    ".onnx",
    ".pdb",
)


def _keep_toc(entry) -> bool:
    blob = _path_str(entry) + " " + _dest_str(entry)
    return not any(m in blob for m in _DROP_MARKERS)


a = Analysis(
    [str(spec_dir / "run_sidecar.py")],
    pathex=[str(spec_dir)],
    binaries=[],
    datas=[],
    hiddenimports=hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=excludes,
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

# Hooks / dependency graph may still stage layout ONNX or devel libs — drop them.
a.datas = [e for e in a.datas if _keep_toc(e)]
a.binaries = [e for e in a.binaries if _keep_toc(e)]
a.pure = [e for e in a.pure if _keep_toc(e)]

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name="rgain_parser",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    # console=False: no flash console window; Rust still uses piped stdin/stdout.
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
