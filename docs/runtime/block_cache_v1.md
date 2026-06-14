# Block buffer cache — contract v1

Status: boot-verified via `make test-blockcache-v1`
Source: `kernel_rs/src/cache.rs` (`cache_read`, `cache_write`, `cache_flush_all`,
`evict`, `cache_selftest`); wired in `kmain` (go lane).
Proof: `tests/runtime/test_blockcache_v1.py`.

Full-OS guide Part II.5 (filesystem maturity), the page/buffer cache: a layer
between the VFS and raw sector I/O that serves repeated reads from RAM and
batches writes.

## Behaviour

A 4-entry, fully-associative, **write-back** LRU over 512-byte sectors:

- **`cache_read(lba)`**: a hit copies the cached sector from RAM and bumps its LRU
  age; a miss evicts the least-recently-used line (flushing it first if dirty),
  loads the sector from disk, and caches it.
- **`cache_write(lba, buf)`**: updates/installs the line and marks it **dirty**;
  the disk write is **deferred** until the line is evicted (flush-on-evict) — so a
  hot sector written repeatedly hits the disk once, not every write.
- **`evict()`**: prefers a free line, else the lowest-age valid line; a dirty
  victim is written to disk before reuse.
- **`cache_flush_all()`**: writes every dirty line back (e.g. before unmount).

Hit/miss counters back the self-test's read-caching assertion.

## Acceptance

`make test-blockcache-v1`: the boot self-test (with the go-lane disk attached)
shows `CACHE: selftest ok`, proving — against scratch sectors — write-back
deferral (after `cache_write` the disk still holds the old bytes), flush-on-evict
(after the LRU line is pushed out the new bytes are on disk), LRU eviction order,
and read-hit caching (a fresh read is one miss; the immediate repeat is one hit) —
with no `CACHE: selftest fail` and no `CACHE: selftest skip`.

## v1 boundary / carry-forward

- **Not yet spliced into the live VFS path.** `vfs_read`/`vfs_write` still call
  `block_io_dispatch` directly; routing them through `cache_read`/`cache_write`
  (and `cache_flush_all` on unmount/sync) is carry-forward — the same staging the
  DMA pool ([`dma_v1.md`](dma_v1.md)) uses (mechanism proven, wiring deferred) to
  keep the change low-risk.
- 4 entries, no associativity index (linear scan), no read-ahead, no
  write-coalescing across adjacent sectors, no per-mount isolation tag.
- All I/O still funnels through the shared `BLK_DATA_PAGE`, so the cache keeps its
  own per-line buffers and copies through it for each disk access.
