"""List DLL imports of a PE executable."""

from __future__ import annotations

import struct
import sys
from pathlib import Path


def rva_to_off(sections: list[tuple], rva: int) -> int | None:
    for _name, vaddr, vsize, rawptr, rawsize in sections:
        if vaddr <= rva < vaddr + max(vsize, rawsize):
            return rawptr + (rva - vaddr)
    return None


def main() -> int:
    exe = Path(sys.argv[1] if len(sys.argv) > 1 else "src-tauri/target/release/rgain.exe")
    data = exe.read_bytes()
    if data[:2] != b"MZ":
        print("not pe", file=sys.stderr)
        return 1
    e_lfanew = struct.unpack_from("<I", data, 0x3C)[0]
    if data[e_lfanew : e_lfanew + 4] != b"PE\0\0":
        print("bad pe", file=sys.stderr)
        return 1
    _machine, num_sections, _a, _b, _c, size_opt, _d = struct.unpack_from(
        "<HHIIIHH", data, e_lfanew + 4
    )
    opt_off = e_lfanew + 24
    magic = struct.unpack_from("<H", data, opt_off)[0]
    if magic == 0x20B:
        import_rva = struct.unpack_from("<I", data, opt_off + 112)[0]
    else:
        import_rva = struct.unpack_from("<I", data, opt_off + 96)[0]
    sec_off = opt_off + size_opt
    sections = []
    for i in range(num_sections):
        off = sec_off + i * 40
        name = data[off : off + 8].rstrip(b"\0").decode("ascii", "ignore")
        vsize, vaddr, rawsize, rawptr = struct.unpack_from("<IIII", data, off + 8)
        sections.append((name, vaddr, vsize, rawptr, rawsize))
    off = rva_to_off(sections, import_rva)
    if off is None:
        print("import table not found", file=sys.stderr)
        return 1
    dlls = []
    while True:
        fields = struct.unpack_from("<IIIII", data, off)
        if fields[0] == 0 and fields[3] == 0:
            break
        name_off = rva_to_off(sections, fields[3])
        if name_off is None:
            break
        end = data.index(b"\0", name_off)
        dlls.append(data[name_off:end].decode("ascii", "ignore"))
        off += 20
    for name in dlls:
        print(name)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
