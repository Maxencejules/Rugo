//! These tests compile the product parser itself, not a Python ELF model.
#[path = "../../kernel_rs/src/elf_static.rs"]
mod elf_static;

use std::cell::RefCell;
use std::collections::BTreeMap;

const BASE: u64 = 0x0140_0000;
const ARGS: u64 = 0x017f_f000;

fn put16(image: &mut [u8], offset: usize, value: u16) {
    image[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(image: &mut [u8], offset: usize, value: u32) {
    image[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn put64(image: &mut [u8], offset: usize, value: u64) {
    image[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn fixture(count: u16) -> Vec<u8> {
    let mut image = vec![0; 512];
    image[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    put16(&mut image, 16, 2);
    put16(&mut image, 18, 62);
    put32(&mut image, 20, 1);
    put64(&mut image, 24, BASE);
    put64(&mut image, 32, 64);
    put16(&mut image, 52, 64);
    put16(&mut image, 54, 56);
    put16(&mut image, 56, count);
    for i in 0..usize::from(count) {
        let ph = 64 + i * 56;
        put32(&mut image, ph, 1);
        put32(&mut image, ph + 4, 5);
        put64(&mut image, ph + 8, 256 + i as u64 * 8);
        put64(&mut image, ph + 16, BASE + i as u64 * 4096);
        put64(&mut image, ph + 32, 4);
        put64(&mut image, ph + 40, 8);
        put64(&mut image, ph + 48, 1);
        image[256 + i * 8..260 + i * 8].copy_from_slice(&[11 + i as u8, 22, 33, 44]);
    }
    image
}

fn rejected(image: &[u8]) {
    let writes = RefCell::new(0);
    assert_eq!(
        elf_static::load_static(
            image,
            BASE,
            ARGS,
            |_, _| {
                *writes.borrow_mut() += 1;
                true
            },
            |_, _| {
                *writes.borrow_mut() += 1;
                true
            },
        ),
        None
    );
    assert_eq!(*writes.borrow(), 0, "malformed input touched child memory");
}

/// Sparse fresh pages, like the kernel callback contract. Mapping an existing
/// page does not erase bytes: this catches BSS/load ordering at shared pages.
#[derive(Default)]
struct Memory(BTreeMap<u64, [u8; 4096]>);
impl Memory {
    fn map(&mut self, va: u64, len: usize) -> bool {
        for offset in 0..len {
            self.0
                .entry((va + offset as u64) & !4095)
                .or_insert([0; 4096]);
        }
        true
    }
    fn copy(&mut self, va: u64, bytes: &[u8]) -> bool {
        self.map(va, bytes.len());
        for (offset, byte) in bytes.iter().enumerate() {
            let address = va + offset as u64;
            self.0.get_mut(&(address & !4095)).unwrap()[(address & 4095) as usize] = *byte;
        }
        true
    }
    fn read(&self, va: u64, len: usize) -> Vec<u8> {
        (0..len)
            .map(|offset| {
                let address = va + offset as u64;
                self.0[&(address & !4095)][(address & 4095) as usize]
            })
            .collect()
    }
}

fn loaded(image: &[u8]) -> Memory {
    let memory = RefCell::new(Memory::default());
    assert!(elf_static::load_static(
        image,
        BASE,
        ARGS,
        |va, bytes| memory.borrow_mut().copy(va, bytes),
        |va, len| memory.borrow_mut().map(va, len),
    )
    .is_some());
    memory.into_inner()
}

#[test]
fn file_bytes_and_bss_match_independent_memory_expectation() {
    let image = fixture(2);
    let memory = loaded(&image);
    assert_eq!(memory.read(BASE, 10), [11, 22, 33, 44, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        memory.read(BASE + 4096, 10),
        [12, 22, 33, 44, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(memory.0.len(), 2);
}

#[test]
fn multipage_file_and_bss_use_all_bytes() {
    let mut image = fixture(1);
    image.resize(9000, 0);
    for (i, byte) in image[256..8756].iter_mut().enumerate() {
        *byte = (i % 251) as u8;
    }
    put64(&mut image, 96, 8500);
    put64(&mut image, 104, 12300);
    let memory = loaded(&image);
    assert_eq!(memory.read(BASE, 8500), image[256..8756]);
    assert!(memory.read(BASE + 8500, 3800).iter().all(|byte| *byte == 0));
    assert_eq!(memory.0.len(), 4);
}

#[test]
fn disjoint_loads_can_share_a_page_in_either_header_order() {
    for reverse in [false, true] {
        let mut image = fixture(2);
        put64(&mut image, 136, BASE + 16);
        if reverse {
            let first: Vec<_> = image[64..120].to_vec();
            image.copy_within(120..176, 64);
            image[120..176].copy_from_slice(&first);
        }
        let memory = loaded(&image);
        assert_eq!(memory.read(BASE, 8), [11, 22, 33, 44, 0, 0, 0, 0]);
        assert_eq!(memory.read(BASE + 16, 8), [12, 22, 33, 44, 0, 0, 0, 0]);
        assert_eq!(memory.0.len(), 1);
    }
}

#[test]
fn padded_unaligned_program_headers_and_ignored_notes_are_supported() {
    let mut image = fixture(2);
    image.copy_within(120..176, 145);
    image.copy_within(64..120, 65);
    put64(&mut image, 32, 65);
    put16(&mut image, 54, 80);
    put32(&mut image, 145, 4); // PT_NOTE, its irrelevant offsets may be arbitrary
    put64(&mut image, 153, u64::MAX);
    assert_eq!(loaded(&image).read(BASE, 4), [11, 22, 33, 44]);
}

#[test]
fn upper_boundary_is_exclusive_and_args_page_is_reserved() {
    let mut image = fixture(1);
    put64(&mut image, 80, ARGS - 8);
    put64(&mut image, 24, ARGS - 1);
    assert_eq!(
        loaded(&image).read(ARGS - 8, 8),
        [11, 22, 33, 44, 0, 0, 0, 0]
    );
    put64(&mut image, 104, 9);
    rejected(&image);
    put64(&mut image, 104, 8);
    put64(&mut image, 24, ARGS);
    rejected(&image);
}

#[test]
fn malformed_ident_machine_version_and_type_reject_without_writes() {
    for (offset, value) in [
        (0, 0),
        (4, 1),
        (5, 2),
        (6, 0),
        (16, 3),
        (18, 183),
        (20, 0),
        (52, 63),
    ] {
        let mut image = fixture(1);
        image[offset] = value;
        rejected(&image);
    }
    for length in 0..64 {
        rejected(&fixture(1)[..length]);
    }
}

#[test]
fn table_offsets_counts_and_truncation_are_checked() {
    for offset in [0, 63, 512, u64::MAX - 55, u64::MAX] {
        let mut image = fixture(1);
        put64(&mut image, 32, offset);
        rejected(&image);
    }
    for count in [0, 9, u16::MAX] {
        let mut image = fixture(1);
        put16(&mut image, 56, count);
        rejected(&image);
    }
    for stride in [0, 55, u16::MAX] {
        let mut image = fixture(1);
        put16(&mut image, 54, stride);
        rejected(&image);
    }
    rejected(&fixture(2)[..175]);
}

#[test]
fn file_range_overflow_and_filesz_above_memsz_reject() {
    for (offset, filesz, memsz) in [
        (u64::MAX, 1, 1),
        (510, 4, 4),
        (256, u64::MAX, u64::MAX),
        (256, 9, 8),
    ] {
        let mut image = fixture(1);
        put64(&mut image, 72, offset);
        put64(&mut image, 96, filesz);
        put64(&mut image, 104, memsz);
        rejected(&image);
    }
}

#[test]
fn virtual_ranges_are_checked_without_wrapping() {
    for (va, memsz) in [
        (BASE - 1, 8),
        (ARGS, 8),
        (BASE, u64::MAX),
        (u64::MAX - 3, 8),
    ] {
        let mut image = fixture(1);
        put64(&mut image, 80, va);
        put64(&mut image, 104, memsz);
        rejected(&image);
    }
}

#[test]
fn no_load_or_only_empty_load_cannot_create_a_child() {
    let mut image = fixture(1);
    put32(&mut image, 64, 0);
    rejected(&image);
    put32(&mut image, 64, 1);
    put64(&mut image, 96, 0);
    put64(&mut image, 104, 0);
    rejected(&image);
}

#[test]
fn entry_must_belong_to_an_executable_load() {
    for entry in [BASE - 1, BASE + 8, BASE + 4096, ARGS] {
        let mut image = fixture(1);
        put64(&mut image, 24, entry);
        rejected(&image);
    }
    let mut image = fixture(1);
    put32(&mut image, 68, 6); // PF_R | PF_W, no PF_X
    rejected(&image);
}

#[test]
fn malformed_late_segment_is_rejected_before_any_child_write() {
    let mut image = fixture(2);
    put64(&mut image, 136, ARGS);
    rejected(&image);
}

#[test]
fn overlapping_load_or_bss_ranges_reject() {
    for va in [BASE, BASE + 3, BASE + 4, BASE + 7] {
        let mut image = fixture(2);
        put64(&mut image, 136, va);
        rejected(&image);
    }
    let mut adjacent = fixture(2);
    put64(&mut adjacent, 136, BASE + 8);
    assert_eq!(loaded(&adjacent).read(BASE + 8, 4), [12, 22, 33, 44]);
}

#[test]
fn alignment_is_power_of_two_and_congruent() {
    for align in [0, 1, 2, 4, 8, 256] {
        let mut image = fixture(1);
        put64(&mut image, 112, align);
        loaded(&image);
    }
    for align in [3, 4096, u64::MAX] {
        let mut image = fixture(1);
        put64(&mut image, 112, align);
        rejected(&image);
    }
}

#[test]
fn static_dynamic_linker_requirements_are_not_silently_ignored() {
    for kind in [2, 3] {
        let mut image = fixture(2);
        put32(&mut image, 120, kind);
        rejected(&image);
    }
}

#[test]
fn invalid_windows_do_not_invoke_callbacks() {
    for (lower, upper) in [(BASE, BASE), (ARGS, BASE)] {
        assert_eq!(
            elf_static::load_static(
                &fixture(1),
                lower,
                upper,
                |_, _| panic!("copy on invalid window"),
                |_, _| panic!("map on invalid window")
            ),
            None
        );
    }
}

#[test]
fn maximum_eight_headers_and_bss_only_data_load_are_supported() {
    let mut image = fixture(1);
    image.resize(1024, 0);
    put16(&mut image, 56, 8);
    for i in 0..8 {
        let ph = 64 + 56 * i;
        put32(&mut image, ph, 1);
        put32(&mut image, ph + 4, if i == 0 { 5 } else { 6 });
        put64(&mut image, ph + 8, 768 + i as u64 * 8);
        put64(&mut image, ph + 16, BASE + i as u64 * 4096);
        put64(&mut image, ph + 32, if i == 7 { 0 } else { 4 });
        put64(&mut image, ph + 40, 8);
        put64(&mut image, ph + 48, 1);
        image[768 + i * 8..772 + i * 8].copy_from_slice(&[i as u8, 22, 33, 44]);
    }
    let memory = loaded(&image);
    assert_eq!(memory.0.len(), 8);
    assert_eq!(memory.read(BASE + 6 * 4096, 8), [6, 22, 33, 44, 0, 0, 0, 0]);
    assert_eq!(memory.read(BASE + 7 * 4096, 8), [0; 8]);
}

#[test]
fn copy_failure_stops_before_mapping_or_next_segment() {
    let mut copies = 0;
    assert_eq!(
        elf_static::load_static(
            &fixture(2),
            BASE,
            ARGS,
            |_, _| {
                copies += 1;
                false
            },
            |_, _| panic!("map after failed copy")
        ),
        None
    );
    assert_eq!(copies, 1);
}

#[test]
fn map_failure_stops_before_next_segment() {
    let mut copies = 0;
    let mut maps = 0;
    assert_eq!(
        elf_static::load_static(
            &fixture(2),
            BASE,
            ARGS,
            |_, _| {
                copies += 1;
                true
            },
            |_, _| {
                maps += 1;
                false
            }
        ),
        None
    );
    assert_eq!((copies, maps), (1, 1));
}

#[test]
fn bounded_seeded_mutations_never_panic_or_write_outside_the_window() {
    let mut seed = 0x4d59_5df4_d0f3_3173u64;
    for iteration in 0..5000 {
        let mut image = fixture(2);
        for _ in 0..1 + iteration % 8 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let offset = (seed >> 32) as usize % image.len();
            image[offset] ^= (seed >> 16) as u8;
        }
        if iteration % 3 == 0 {
            image.truncate(iteration % 512);
        }
        let writes = RefCell::new(0);
        let entry = elf_static::load_static(
            &image,
            BASE,
            ARGS,
            |va, bytes| {
                assert!(va >= BASE && va.checked_add(bytes.len() as u64).unwrap() <= ARGS);
                *writes.borrow_mut() += 1;
                true
            },
            |va, len| {
                assert!(va >= BASE && va.checked_add(len as u64).unwrap() <= ARGS);
                *writes.borrow_mut() += 1;
                true
            },
        );
        if let Some(entry) = entry {
            assert!((BASE..ARGS).contains(&entry));
        } else {
            assert_eq!(*writes.borrow(), 0);
        }
    }
}
