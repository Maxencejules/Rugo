# Huge pages (2 MiB) — contract v1

Status: boot-verified via `make test-hugepage-v1`
Source: `kernel_rs/src/mm.rs` (`vm_map_huge_current`, `alloc_frames_contig_aligned`,
`pd_entry_current`, `huge_page_selftest`, `PTE_PS`).
Proof: `tests/runtime/test_hugepage_v1.py`.

Full-OS guide Part I.4 (memory), huge pages: map memory with 2 MiB pages so a
single TLB entry and a single PD entry cover 2 MiB instead of 512 4 KiB PTEs.

## Behaviour

- **`alloc_frames_contig_aligned(count, align)`**: scans the PMM for `count`
  physically-contiguous frames whose start frame is a multiple of `align` — for a
  huge page, `count = align = 512`, giving a 2 MiB-aligned 2 MiB physical region
  (a 2 MiB page's physical base must be 2 MiB-aligned).
- **`vm_map_huge_current(va, prot)`**: requires `va` 2 MiB-aligned; walks/creates
  the PML4 and PDPT (4 KiB tables) for the current CR3, then installs **one PD
  entry** with the page-size bit (`PTE_PS`) pointing at the aligned 2 MiB region
  (`prot` bit1 = W, bit2 = X else NX; kernel page, no user bit).

## Acceptance

`make test-hugepage-v1`: the boot self-test maps a 2 MiB huge page, confirms the
PD entry carries the page-size bit (it is a single 2 MiB mapping, not 512 4 KiB
PTEs), and reads/writes at offset 0 and at the last 8 bytes of the 2 MiB through
that one mapping — `HUGEPAGE: 2M ok`, with no `HUGEPAGE: 2M fail`.

## v1 boundary / carry-forward

- A kernel-mapped 2 MiB page exercised by the self-test; exposing huge pages to
  userspace (an `mmap(MAP_HUGETLB)`-style flag through `sys_vm_ctl`), 1 GiB pages,
  transparent huge pages, and huge-page-aware unmap/CoW are carry-forward.
- `alloc_frames_contig_aligned` leaks nothing here (one boot allocation), but a
  general huge-page allocator with a free path is carry-forward.
