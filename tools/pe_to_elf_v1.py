"""Convert a fully linked, relocatable PE image into an ET_DYN (PIE) ELF64.

The host toolchain (mingw gcc + ld) can only LINK PE images, and objcopy
mistranslates PE REL32 relocations into ELF (the implicit -4 addend is lost,
skewing every cross-reference). Linking entirely in PE sidesteps relocation
translation: this tool re-wraps the resolved bytes.

For code-base ASLR (full-os guide Part IV.10) the app is now ET_DYN: the PE is
linked --dynamicbase --image-base 0x0, so its sections sit at clean RVAs and a
.reloc directory lists the absolute (DIR64) pointers gcc emits for globals
(.refptr.<name>). This tool lays the sections out at vaddr == RVA, synthesizes a
.rela.dyn of R_X86_64_RELATIVE entries from the .reloc DIR64 list, and appends a
PT_DYNAMIC with DT_RELA/RELASZ/RELAENT. The kernel's exec_load_pie loads the image
at a RANDOM base and applies those RELATIVE relocations (val = base + addend), so
the .refptr pointers land at base + global_rva -- correct at any base. base_shell
(pure RIP-relative) needs no relocs; C apps need exactly these DIR64 fixups.
"""

import argparse
import struct
from pathlib import Path

# ELF dynamic-tag + reloc constants.
DT_NULL = 0
DT_RELA = 7
DT_RELASZ = 8
DT_RELAENT = 9
R_X86_64_RELATIVE = 8
# PE base-relocation entry types.
IMAGE_REL_BASED_ABSOLUTE = 0  # padding, skip
IMAGE_REL_BASED_DIR64 = 10    # 64-bit absolute field to fix up


def _parse_reloc_dir(data, blob, reloc_rva, reloc_size):
    """Parse the PE .reloc directory (already copied into `blob` at its RVA) into a
    list of (target_rva, addend) for each DIR64 entry. addend is the value currently
    stored at target_rva -- i.e. the global's RVA (image_base is 0) -- so the kernel's
    RELATIVE reloc writes base+addend = base+global_rva there."""
    relocs = []
    if reloc_size == 0:
        return relocs
    pos = reloc_rva
    end = reloc_rva + reloc_size
    while pos + 8 <= end:
        page_rva, block_size = struct.unpack_from("<II", blob, pos)
        if block_size < 8 or pos + block_size > end + 8:
            break
        nentries = (block_size - 8) // 2
        for i in range(nentries):
            entry, = struct.unpack_from("<H", blob, pos + 8 + i * 2)
            typ = entry >> 12
            off = entry & 0xFFF
            if typ == IMAGE_REL_BASED_DIR64:
                target = page_rva + off
                if target + 8 > len(blob):
                    raise SystemExit(
                        f"pe_to_elf: DIR64 reloc target 0x{target:x} past mapped image "
                        f"(len 0x{len(blob):x}) -- unexpected for a .refptr fixup"
                    )
                (value,) = struct.unpack_from("<Q", blob, target)
                relocs.append((target, value))
            # ABSOLUTE (0) and any other type: skip (padding / unsupported).
        pos += block_size
    return relocs


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
    # PE32+ data directories begin at opt+112; index 5 = base relocation table.
    reloc_dir_rva, reloc_dir_size = struct.unpack_from("<II", data, opt + 112 + 5 * 8)
    assert image_base == 0, (
        "PE must be linked --image-base 0x0 for PIE (got 0x%x)" % image_base
    )

    # Lay sections out at their RVA (one vaddr-indexed blob), as the old tool did.
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

    # Synthesize R_X86_64_RELATIVE relocs from the PE DIR64 base relocations.
    dir64 = _parse_reloc_dir(data, blob, reloc_dir_rva, reloc_dir_size)
    rela = bytearray()
    for target_rva, value in dir64:
        # Elf64_Rela: r_offset, r_info (sym<<32 | type), r_addend.
        rela += struct.pack("<QQQ", target_rva, R_X86_64_RELATIVE, value)
    rela = bytes(rela)

    # Place .rela.dyn + .dynamic AFTER the loaded image (past any BSS), page-aligned, in
    # one extra PT_LOAD. .dynamic immediately follows .rela.dyn -- both stay 8-aligned
    # (rela_vaddr is page-aligned, len(rela) is a multiple of 24), so no padding needed
    # and the file offset == vaddr-delta mapping is exact.
    img_end = max(size_of_image, max_file_rva)
    rela_vaddr = (img_end + 0xFFF) & ~0xFFF
    dyn_vaddr = rela_vaddr + len(rela)
    dyn = struct.pack(
        "<QQQQQQQQ",
        DT_RELA, rela_vaddr,
        DT_RELASZ, len(rela),
        DT_RELAENT, 24,
        DT_NULL, 0,
    )

    # ELF: header + 3 program headers (2x PT_LOAD + PT_DYNAMIC). The payload maps at
    # vaddr 0 (p_filesz=max_file_rva, p_memsz=size_of_image covers BSS); the second
    # segment carries .rela.dyn + .dynamic at rela_vaddr. NB p_offset is not congruent
    # to p_vaddr mod p_align -- ld.so would reject, but exec_load_pie copies by absolute
    # p_offset/p_vaddr (no mmap, no alignment check).
    PT_LOAD, PT_DYNAMIC = 1, 2
    ehsize, phsize = 64, 56
    phnum = 3
    payload_off = ehsize + phnum * phsize
    rela_off = payload_off + len(payload)
    dyn_off = rela_off + len(rela)
    seg2_filesz = len(rela) + len(dyn)
    img_memsz = max(size_of_image, max_file_rva)

    eh = struct.pack(
        "<4sBBBBB7xHHIQQQIHHHHHH",
        b"\x7fELF", 2, 1, 1, 0, 0,
        3,        # ET_DYN
        0x3E,     # EM_X86_64
        1,
        entry_rva,  # entry is an offset (loaded at base + entry_rva)
        ehsize,   # phoff
        0,        # shoff
        0,
        ehsize,
        phsize,
        phnum,
        0, 0, 0,
    )
    ph_load1 = struct.pack(
        "<IIQQQQQQ",
        PT_LOAD, 0x7, payload_off, 0, 0, len(payload), img_memsz, 0x1000,
    )
    ph_load2 = struct.pack(
        "<IIQQQQQQ",
        PT_LOAD, 0x6, rela_off, rela_vaddr, rela_vaddr, seg2_filesz, seg2_filesz, 0x1000,
    )
    ph_dyn = struct.pack(
        "<IIQQQQQQ",
        PT_DYNAMIC, 0x6, dyn_off, dyn_vaddr, dyn_vaddr, len(dyn), len(dyn), 8,
    )

    out = bytearray()
    out += eh + ph_load1 + ph_load2 + ph_dyn
    out += payload
    out += rela
    out += dyn

    Path(args.elf).write_bytes(out)
    print(
        f"pe-to-elf(PIE): {args.elf} entry_rva=0x{entry_rva:x} relocs={len(dir64)} "
        f"filesz={max_file_rva} memsz={max(size_of_image, max_file_rva)} rela@0x{rela_vaddr:x}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
