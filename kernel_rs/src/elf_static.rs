//! Bounded ET_EXEC preflight shared by the product loader and host tests.
//! No allocation, unsafe code, memory writes or platform dependencies in parsing.

const MAX_HEADERS: usize = 8;

#[derive(Clone, Copy)]
struct Segment<'a> {
    va: u64,
    end: u64,
    bytes: &'a [u8],
    zero_len: usize,
}

const EMPTY: Segment<'static> = Segment {
    va: 0,
    end: 0,
    bytes: &[],
    zero_len: 0,
};

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

fn u64_at(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?,
    ))
}

/// Validate the entire image before invoking either memory callback.
///
/// `lower..upper` is the allowed app-byte window (excluding the args page).
/// Callbacks must target a fresh, zero-initialized child address space. The map
/// callback only ensures BSS pages exist; it need not clear already mapped pages.
/// Malformed images produce no callbacks. A callback failure returns None; the
/// caller must release the child address space, which may then be partly loaded.
pub fn load_static(
    image: &[u8],
    lower: u64,
    upper: u64,
    mut copy: impl FnMut(u64, &[u8]) -> bool,
    mut map_zeroed: impl FnMut(u64, usize) -> bool,
) -> Option<u64> {
    if lower >= upper
        || image.len() < 64
        || image.get(..7)? != b"\x7fELF\x02\x01\x01"
        || u16_at(image, 16)? != 2 // ET_EXEC, not the separately handled ET_DYN lane
        || u16_at(image, 18)? != 62 // EM_X86_64
        || u32_at(image, 20)? != 1
        || u16_at(image, 52)? != 64
    {
        return None;
    }
    let entry = u64_at(image, 24)?;
    let phoff = usize::try_from(u64_at(image, 32)?).ok()?;
    let stride = usize::from(u16_at(image, 54)?);
    let count = usize::from(u16_at(image, 56)?);
    if phoff < 64 || stride < 56 || count == 0 || count > MAX_HEADERS {
        return None;
    }
    let table_end = phoff.checked_add(stride.checked_mul(count)?)?;
    image.get(phoff..table_end)?;

    let mut segments = [EMPTY; MAX_HEADERS];
    let mut loaded = 0;
    let mut executable_entry = false;
    for index in 0..count {
        // The complete table/stride was checked above, including padding.
        let ph = &image[phoff + index * stride..phoff + index * stride + 56];
        match u32_at(ph, 0)? {
            2 | 3 => return None, // PT_DYNAMIC / PT_INTERP need unsupported ET_EXEC runtime linking
            1 => {}
            _ => continue,
        }
        let offset = u64_at(ph, 8)?;
        let va = u64_at(ph, 16)?;
        let filesz = u64_at(ph, 32)?;
        let memsz = u64_at(ph, 40)?;
        let align = u64_at(ph, 48)?;
        if filesz > memsz
            || (align > 1 && (!align.is_power_of_two() || va % align != offset % align))
        {
            return None;
        }
        let file_end = offset.checked_add(filesz)?;
        let bytes = image.get(usize::try_from(offset).ok()?..usize::try_from(file_end).ok()?)?;
        let end = va.checked_add(memsz)?;
        if va < lower || end > upper {
            return None;
        }
        if memsz == 0 {
            continue;
        }
        for previous in &segments[..loaded] {
            if va < previous.end && previous.va < end {
                return None; // prevent load/BSS results depending on header order
            }
        }
        executable_entry |= u32_at(ph, 4)? & 1 != 0 && va <= entry && entry < end;
        segments[loaded] = Segment {
            va,
            end,
            bytes,
            zero_len: usize::try_from(memsz - filesz).ok()?,
        };
        loaded += 1;
    }
    if loaded == 0 || !executable_entry {
        return None;
    }

    // Every range and the entry are valid before the first child write.
    for segment in &segments[..loaded] {
        if !segment.bytes.is_empty() && !copy(segment.va, segment.bytes) {
            return None;
        }
        if segment.zero_len != 0
            && !map_zeroed(segment.va + segment.bytes.len() as u64, segment.zero_len)
        {
            return None;
        }
    }
    Some(entry)
}
