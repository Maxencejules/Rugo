pub const GO_IMAGE_PAGE_SIZE: usize = 4096;
pub const GO_IMAGE_MAX_PAGES: usize = 4;
pub const GO_IMAGE_MAX_BYTES: usize = GO_IMAGE_PAGE_SIZE * GO_IMAGE_MAX_PAGES;

pub fn go_image_chunk(blob: &[u8], page_idx: usize) -> &[u8] {
    let start = page_idx.saturating_mul(GO_IMAGE_PAGE_SIZE);
    if start >= blob.len() {
        return &[];
    }
    let end = core::cmp::min(start + GO_IMAGE_PAGE_SIZE, blob.len());
    &blob[start..end]
}
