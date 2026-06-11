"""Convert a fully linked PE image into a one-segment ET_EXEC ELF64.

The host toolchain (mingw gcc + ld) can only LINK PE images, and objcopy
mistranslates PE REL32 relocations into ELF (the implicit -4 addend is
lost, skewing every cross-reference). Linking entirely in PE sidesteps
relocation translation: this tool just re-wraps the resolved bytes -
sections are laid out at ImageBase+RVA, BSS becomes memsz beyond filesz,
and the entry point carries over.
"""

import argparse
import struct
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pe")
    parser.add_argument("elf")
    args = parser.parse_args()

    data = Path(args.pe).read_bytes()
    pe_off = struct.unpack_from("<I", data, 0x3C)[0]
    assert data[pe_off : pe_off + 4] == b"PE\x00\x00", "not a PE image"
    nsections, = struct.unpack_from("<H", data, pe_off + 6)
    opt_size, = struct.unpack_from("<H", data, pe_off + 20)
    opt = pe_off + 24
    magic, = struct.unpack_from("<H", data, opt)
    assert magic == 0x20B, "not PE32+"
    entry_rva, = struct.unpack_from("<I", data, opt + 16)
    image_base, = struct.unpack_from("<Q", data, opt + 24)
    size_of_image, = struct.unpack_from("<I", data, opt + 56)

    sec = opt + opt_size
    blob = bytearray()
    max_file_rva = 0
    for i in range(nsections):
        off = sec + i * 40
        vsize, rva, rawsize, rawptr = struct.unpack_from("<IIII", data, off + 8)
        copy = min(rawsize, vsize)
        if copy == 0:
            continue
        end = rva + copy
        if end > len(blob):
            blob.extend(b"\x00" * (end - len(blob)))
        blob[rva:end] = data[rawptr : rawptr + copy]
        max_file_rva = max(max_file_rva, end)

    payload = bytes(blob[:max_file_rva])
    memsz = max(size_of_image, max_file_rva)
    entry = image_base + entry_rva

    # ELF64: header + one program header, then the payload. The segment
    # maps ImageBase..ImageBase+memsz; vaddr-offset congruence holds
    # because the loader copies by p_offset/p_vaddr directly.
    ehsize, phsize = 64, 56
    payload_off = ehsize + phsize
    eh = struct.pack(
        "<4sBBBBB7xHHIQQQIHHHHHH",
        b"\x7fELF", 2, 1, 1, 0, 0,
        2,  # ET_EXEC
        0x3E,  # EM_X86_64
        1,
        entry,
        ehsize,  # phoff
        0,  # shoff
        0,
        ehsize,
        phsize,
        1,  # phnum
        0, 0, 0,
    )
    ph = struct.pack(
        "<IIQQQQQQ",
        1,  # PT_LOAD
        0x7,  # RWX
        payload_off,
        image_base,
        image_base,
        len(payload),
        memsz,
        0x1000,
    )
    Path(args.elf).write_bytes(eh + ph + payload)
    print(
        f"pe-to-elf: {args.elf} entry=0x{entry:x} base=0x{image_base:x} "
        f"filesz={len(payload)} memsz={memsz}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
