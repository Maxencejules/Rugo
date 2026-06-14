# DMA allocator — contract v1

Status: boot-verified via `make test-dma-v1`
Source: `kernel_rs/src/dma.rs` (`dma_init`, `dma_alloc`, `dma_free`,
`dma_selftest`); wired in `kmain` (go lane).
Proof: `tests/runtime/test_dma_v1.py`.

Full-OS guide Part II.7 (driver model), DMA allocator: a kernel-side pool that
hands drivers physically-contiguous memory for descriptor rings and buffers.

## Behaviour

At boot `dma_init` reserves one physically-contiguous run of `DMA_POOL_PAGES`
(256 = 1 MiB) frames from the PMM (`alloc_frames_contig`) and records its base.
The pool is bitmap-managed (`DMA_USED`, one bool per page):

- **`dma_alloc(pages) -> phys`**: first-fit scan for `pages` consecutive free
  slots; marks them used and returns the **physical** base address (devices DMA
  to physical memory, not the kernel's HHDM virtual addresses). `None` if no
  contiguous run fits.
- **`dma_free(phys, pages)`**: clears the slots for a region previously returned
  by `dma_alloc`; out-of-range / misaligned frees are ignored.

Returned addresses are page-aligned and lie within the reserved pool, so a driver
can program a device's ring base register directly.

## Acceptance

`make test-dma-v1`: the boot transcript shows `DMA: pool base=0x<phys>
pages=0x100` then `DMA: selftest ok` — the self-test allocates two regions and
asserts they are page-aligned, non-overlapping, contiguous, and pool-contained,
then frees the first and confirms a same-size re-alloc reuses its exact base
(first-fit) — with no `DMA: pool none` and no `DMA: selftest fail`.

## v1 boundary / carry-forward

- One fixed-size (1 MiB) pool, first-fit, no fragmentation compaction.
- No per-device IOMMU domains / address-translation isolation (the pool hands out
  raw physical addresses).
- Not yet consumed by the existing virtio/NVMe probes (they use their own static
  buffers); migrating them onto `dma_alloc` is carry-forward.
- Go lane only — it reserves 1 MiB at boot, which would shift the free-frame
  accounting other lanes assert.
