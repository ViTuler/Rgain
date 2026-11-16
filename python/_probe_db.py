"""Probe Rgain sqlite file header (dev diagnostics)."""
from pathlib import Path

p = Path(r"F:/VAppData/com.ViTuler.Rgain/Rgain.sqlite3")
print("exists", p.exists())
if p.exists():
    b = p.read_bytes()[:32]
    print("size", p.stat().st_size)
    print("head_hex", b[:16].hex())
    print("plain_sqlite_header", b.startswith(b"SQLite format 3"))
    for suf in ("-wal", "-shm", ".pre_cipher.bak", ".encrypting.tmp"):
        s = Path(str(p) + suf) if not suf.startswith(".") else p.with_name(p.name + suf)
        # wal/shm append; bak/tmp use name suffix
        if suf.startswith("-"):
            s = Path(str(p) + suf)
        else:
            s = p.with_name(p.name + suf)
        print(suf, s.exists(), s.stat().st_size if s.exists() else 0)
