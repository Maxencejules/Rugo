// Framebuffer text console (gap-analysis item 7): mirrors every serial
// line onto a Limine-provided linear framebuffer so the OS is usable
// outside a serial pipe. Compiled in every lane, like mm.rs.
//
// Font: the classic public-domain 8x8 bitmap font (LSB = leftmost pixel),
// ASCII 0x20..0x7E. Glyph fidelity is cosmetic; the console contract is
// that the boot transcript is drawn as pixels.

#![allow(dead_code)]

#[repr(C)]
struct LimineFramebuffer {
    address: u64,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
}

#[repr(C)]
struct LimineFramebufferResponse {
    revision: u64,
    framebuffer_count: u64,
    framebuffers: *const *const LimineFramebuffer,
}

#[repr(C)]
struct LimineFramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: *const LimineFramebufferResponse,
}

unsafe impl Sync for LimineFramebufferRequest {}

#[used]
#[link_section = ".limine_requests"]
static mut FB_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b,
         0x9d5827dcd881dd75, 0xa3148604f6fab11b],
    revision: 0,
    response: core::ptr::null(),
};

const GLYPH_W: u64 = 8;
const GLYPH_H: u64 = 8;
const FG: u32 = 0x00DCDCDC;
const BG: u32 = 0x00101018;

struct FbConsole {
    ready: bool,
    addr: u64,
    width: u64,
    height: u64,
    pitch: u64,
    cols: u64,
    rows: u64,
    col: u64,
    row: u64,
}

static mut FB: FbConsole = FbConsole {
    ready: false,
    addr: 0,
    width: 0,
    height: 0,
    pitch: 0,
    cols: 0,
    rows: 0,
    col: 0,
    row: 0,
};

pub fn fb_ready() -> bool {
    unsafe { FB.ready }
}

pub fn fb_size() -> (u64, u64) {
    unsafe { (FB.width, FB.height) }
}

/// Fill a rectangle in the framebuffer with a 32-bpp XRGB color, clamped to
/// the screen bounds (full-os guide Part III graphics). Returns false if no
/// framebuffer is present or the origin is off-screen.
pub fn fb_blit_rect(x: u64, y: u64, w: u64, h: u64, color: u32) -> bool {
    unsafe {
        if !FB.ready || x >= FB.width || y >= FB.height {
            return false;
        }
        let x_end = core::cmp::min(x + w, FB.width);
        let y_end = core::cmp::min(y + h, FB.height);
        let mut yy = y;
        while yy < y_end {
            let line = (FB.addr + yy * FB.pitch) as *mut u32;
            let mut xx = x;
            while xx < x_end {
                *line.add(xx as usize) = color;
                xx += 1;
            }
            yy += 1;
        }
        true
    }
}

/// Fill a rectangle with an ARGB color **alpha-blended** (src-over) onto the
/// existing pixels (full-os guide Part III graphics): `out = src*a + dst*(255-a)`
/// per channel, where `a` is the top byte of `argb`. Unlike `fb_blit_rect` (which
/// overwrites), this lets translucent surfaces show what is behind them. Clamped
/// to screen bounds; returns false if no framebuffer or the origin is off-screen.
pub fn fb_blit_rect_blend(x: u64, y: u64, w: u64, h: u64, argb: u32) -> bool {
    unsafe {
        if !FB.ready || x >= FB.width || y >= FB.height {
            return false;
        }
        let a = (argb >> 24) & 0xFF;
        let inv = 255 - a;
        let sr = (argb >> 16) & 0xFF;
        let sg = (argb >> 8) & 0xFF;
        let sb = argb & 0xFF;
        let x_end = core::cmp::min(x + w, FB.width);
        let y_end = core::cmp::min(y + h, FB.height);
        let mut yy = y;
        while yy < y_end {
            let line = (FB.addr + yy * FB.pitch) as *mut u32;
            let mut xx = x;
            while xx < x_end {
                let dst = *line.add(xx as usize);
                let dr = (dst >> 16) & 0xFF;
                let dg = (dst >> 8) & 0xFF;
                let db = dst & 0xFF;
                let or = (sr * a + dr * inv) / 255;
                let og = (sg * a + dg * inv) / 255;
                let ob = (sb * a + db * inv) / 255;
                *line.add(xx as usize) = (or << 16) | (og << 8) | ob;
                xx += 1;
            }
            yy += 1;
        }
        true
    }
}

/// Self-test the alpha blend on a single saved+restored pixel (so the on-screen
/// console is left untouched): paint an opaque blue background, blend 50%-alpha
/// red over it, read the result back, and confirm it equals the src-over mix.
/// Returns 1 = ok, 0 = mismatch, 2 = no framebuffer.
pub fn fb_alpha_selftest() -> u64 {
    unsafe {
        if !FB.ready {
            return 2;
        }
        let p = ((FB.addr + 10 * FB.pitch) as *mut u32).add(10);
        let saved = *p;
        let _ = fb_blit_rect(10, 10, 1, 1, 0x0000_00FF); // opaque blue
        let _ = fb_blit_rect_blend(10, 10, 1, 1, 0x80FF_0000); // 50% red (a=128)
        let got = *p & 0x00FF_FFFF;
        *p = saved; // restore the pixel
        let a = 128u32;
        let inv = 255 - a;
        let er = (255 * a) / 255; // red: src 255 over dst 0
        let eg = 0u32; // green: both 0
        let eb = (255 * inv) / 255; // blue: src 0 over dst 255
        let expect = (er << 16) | (eg << 8) | eb;
        if got == expect {
            1
        } else {
            0
        }
    }
}

// Mouse cursor compositing with save-under (full-os guide Part III): drawing the
// cursor over the screen must not destroy what is underneath, so the pixels the
// cursor covers are saved before it is drawn and restored when it moves -- the
// classic "save-under" technique a windowing system uses for a hardware-less
// cursor. v1 cursor is a solid 8x8 sprite.
const CURSOR_W: u64 = 8;
const CURSOR_H: u64 = 8;
const CURSOR_COLOR: u32 = 0x00FF_FFFF; // white
static mut CURSOR_SAVE: [u32; (CURSOR_W * CURSOR_H) as usize] = [0; (CURSOR_W * CURSOR_H) as usize];
static mut CURSOR_X: u64 = 0;
static mut CURSOR_Y: u64 = 0;
static mut CURSOR_ACTIVE: bool = false;

/// Restore the pixels saved under the cursor (undraw it), leaving the framebuffer
/// exactly as it was before fb_cursor_draw. A no-op if no cursor is drawn.
pub fn fb_cursor_restore() -> bool {
    unsafe {
        if !FB.ready || !CURSOR_ACTIVE {
            return false;
        }
        let mut j = 0u64;
        while j < CURSOR_H {
            let py = CURSOR_Y + j;
            if py < FB.height {
                let line = (FB.addr + py * FB.pitch) as *mut u32;
                let mut i = 0u64;
                while i < CURSOR_W {
                    let px = CURSOR_X + i;
                    if px < FB.width {
                        *line.add(px as usize) = CURSOR_SAVE[(j * CURSOR_W + i) as usize];
                    }
                    i += 1;
                }
            }
            j += 1;
        }
        CURSOR_ACTIVE = false;
        true
    }
}

/// Draw the mouse cursor at (x,y), first saving the pixels underneath so the next
/// fb_cursor_draw / fb_cursor_restore can undraw it cleanly (the cursor can move
/// without corrupting the screen it passes over). Any previously-drawn cursor is
/// restored first. Returns false if no framebuffer or the origin is off-screen.
pub fn fb_cursor_draw(x: u64, y: u64) -> bool {
    unsafe {
        if !FB.ready {
            return false;
        }
        if CURSOR_ACTIVE {
            fb_cursor_restore();
        }
        if x >= FB.width || y >= FB.height {
            return false;
        }
        // Save the pixels the cursor will cover (clamped; off-screen cells save 0).
        let mut j = 0u64;
        while j < CURSOR_H {
            let py = y + j;
            let mut i = 0u64;
            while i < CURSOR_W {
                let px = x + i;
                let idx = (j * CURSOR_W + i) as usize;
                CURSOR_SAVE[idx] = if px < FB.width && py < FB.height {
                    *((FB.addr + py * FB.pitch) as *const u32).add(px as usize)
                } else {
                    0
                };
                i += 1;
            }
            j += 1;
        }
        let _ = fb_blit_rect(x, y, CURSOR_W, CURSOR_H, CURSOR_COLOR);
        CURSOR_X = x;
        CURSOR_Y = y;
        CURSOR_ACTIVE = true;
        true
    }
}

/// Self-test the cursor save-under (full-os guide Part III): paint a known
/// background patch, draw the cursor over it (the cursor color must appear),
/// restore (the background must come back pixel-for-pixel), then leave the screen
/// exactly as found. Proves the cursor can be drawn/moved without corrupting what
/// is underneath. Returns 1 = ok, 0 = mismatch, 2 = no framebuffer.
pub fn fb_cursor_selftest() -> u64 {
    unsafe {
        if !FB.ready {
            return 2;
        }
        const N: usize = (CURSOR_W * CURSOR_H) as usize;
        let (cx, cy) = (40u64, 40u64);
        if cx + CURSOR_W >= FB.width || cy + CURSOR_H >= FB.height {
            return 2;
        }
        // Snapshot the original patch so the screen is left untouched afterwards.
        let mut orig = [0u32; N];
        let mut j = 0u64;
        while j < CURSOR_H {
            let line = (FB.addr + (cy + j) * FB.pitch) as *const u32;
            let mut i = 0u64;
            while i < CURSOR_W {
                orig[(j * CURSOR_W + i) as usize] = *line.add((cx + i) as usize);
                i += 1;
            }
            j += 1;
        }
        // Paint a known, non-cursor-coloured background, then exercise the cursor.
        let bg = 0x0000_2040u32;
        let _ = fb_blit_rect(cx, cy, CURSOR_W, CURSOR_H, bg);
        let p = ((FB.addr + cy * FB.pitch) as *const u32).add(cx as usize);
        let bg_seen = *p & 0x00FF_FFFF;
        fb_cursor_draw(cx, cy);
        let drawn = *p & 0x00FF_FFFF; // cursor colour over the bg
        fb_cursor_restore();
        let restored = *p & 0x00FF_FFFF; // save-under brings the bg back
        // Leave the framebuffer exactly as we found it.
        j = 0;
        while j < CURSOR_H {
            let line = (FB.addr + (cy + j) * FB.pitch) as *mut u32;
            let mut i = 0u64;
            while i < CURSOR_W {
                *line.add((cx + i) as usize) = orig[(j * CURSOR_W + i) as usize];
                i += 1;
            }
            j += 1;
        }
        if bg_seen == (bg & 0x00FF_FFFF)
            && drawn == (CURSOR_COLOR & 0x00FF_FFFF)
            && restored == (bg & 0x00FF_FFFF)
        {
            1
        } else {
            0
        }
    }
}

/// Blit a client-provided `w`x`h` ARGB pixel buffer (`src`, row-major, 4 bytes
/// per pixel, little-endian, top byte ignored) to the framebuffer at (x,y),
/// clamped to screen bounds (full-os guide Part III, per-client pixel surfaces).
/// Unlike `fb_blit_rect` (one solid color) this paints REAL per-pixel bitmaps --
/// the basis for a compositor rendering app surfaces. Returns false if no
/// framebuffer, the origin is off-screen, or `src` is too small for `w*h`.
pub fn fb_blit_pixels(x: u64, y: u64, w: u64, h: u64, src: &[u8]) -> bool {
    unsafe {
        if !FB.ready || x >= FB.width || y >= FB.height {
            return false;
        }
        if src.len() < (w * h * 4) as usize {
            return false;
        }
        let x_end = core::cmp::min(x + w, FB.width);
        let y_end = core::cmp::min(y + h, FB.height);
        let mut yy = y;
        while yy < y_end {
            let line = (FB.addr + yy * FB.pitch) as *mut u32;
            let srow = ((yy - y) * w) as usize;
            let mut xx = x;
            while xx < x_end {
                let si = (srow + (xx - x) as usize) * 4;
                let px = u32::from_le_bytes([src[si], src[si + 1], src[si + 2], src[si + 3]]);
                *line.add(xx as usize) = px & 0x00FF_FFFF;
                xx += 1;
            }
            yy += 1;
        }
        true
    }
}

/// Adopt the Limine framebuffer if one was provided (32 bpp only).
pub fn fb_init() {
    unsafe {
        let resp = core::ptr::read_volatile(core::ptr::addr_of!(FB_REQUEST.response));
        if resp.is_null() || (*resp).framebuffer_count == 0 {
            return;
        }
        let fb = *(*resp).framebuffers;
        if fb.is_null() || (*fb).bpp != 32 {
            return;
        }
        FB.addr = (*fb).address;
        FB.width = (*fb).width;
        FB.height = (*fb).height;
        FB.pitch = (*fb).pitch;
        FB.cols = FB.width / GLYPH_W;
        FB.rows = FB.height / GLYPH_H;
        FB.col = 0;
        FB.row = 0;
        // Clear to the background colour.
        let mut y = 0;
        while y < FB.height {
            let line = (FB.addr + y * FB.pitch) as *mut u32;
            let mut x = 0;
            while x < FB.width {
                *line.add(x as usize) = BG;
                x += 1;
            }
            y += 1;
        }
        FB.ready = true;
    }
}

unsafe fn scroll() {
    let row_bytes = (GLYPH_H * FB.pitch) as usize;
    let total = (FB.rows * GLYPH_H * FB.pitch) as usize;
    core::ptr::copy(
        (FB.addr as *const u8).add(row_bytes),
        FB.addr as *mut u8,
        total - row_bytes,
    );
    // Blank the last text row.
    let base = FB.addr + (FB.rows - 1) * GLYPH_H * FB.pitch;
    let mut y = 0;
    while y < GLYPH_H {
        let line = (base + y * FB.pitch) as *mut u32;
        let mut x = 0;
        while x < FB.width {
            *line.add(x as usize) = BG;
            x += 1;
        }
        y += 1;
    }
}

unsafe fn draw_glyph(ch: u8, col: u64, row: u64) {
    let glyph = &FONT8X8[glyph_index(ch)];
    let base = FB.addr + row * GLYPH_H * FB.pitch + col * GLYPH_W * 4;
    let mut y = 0u64;
    while y < GLYPH_H {
        let bits = glyph[y as usize];
        let line = (base + y * FB.pitch) as *mut u32;
        let mut x = 0u64;
        while x < GLYPH_W {
            *line.add(x as usize) = if bits & (1 << x) != 0 { FG } else { BG };
            x += 1;
        }
        y += 1;
    }
}

unsafe fn put_char(ch: u8) {
    match ch {
        b'\n' => {
            FB.col = 0;
            FB.row += 1;
        }
        b'\r' => {
            FB.col = 0;
        }
        8 | 127 => {
            if FB.col > 0 {
                FB.col -= 1;
                draw_glyph(b' ', FB.col, FB.row);
            }
        }
        _ => {
            draw_glyph(ch, FB.col, FB.row);
            FB.col += 1;
            if FB.col >= FB.cols {
                FB.col = 0;
                FB.row += 1;
            }
        }
    }
    if FB.row >= FB.rows {
        scroll();
        FB.row = FB.rows - 1;
    }
}

/// Mirror a serial buffer onto the framebuffer console.
pub fn fb_write(buf: &[u8]) {
    unsafe {
        if !FB.ready {
            return;
        }
        for &b in buf {
            put_char(b);
        }
    }
}

fn glyph_index(ch: u8) -> usize {
    if (0x20..0x7F).contains(&ch) {
        (ch - 0x20) as usize
    } else {
        0
    }
}

// Public-domain 8x8 font (font8x8_basic), LSB = leftmost pixel.
static FONT8X8: [[u8; 8]; 95] = [
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // space
    [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00], // !
    [0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // "
    [0x36, 0x36, 0x7F, 0x36, 0x7F, 0x36, 0x36, 0x00], // #
    [0x0C, 0x3E, 0x03, 0x1E, 0x30, 0x1F, 0x0C, 0x00], // $
    [0x00, 0x63, 0x33, 0x18, 0x0C, 0x66, 0x63, 0x00], // %
    [0x1C, 0x36, 0x1C, 0x6E, 0x3B, 0x33, 0x6E, 0x00], // &
    [0x06, 0x06, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00], // '
    [0x18, 0x0C, 0x06, 0x06, 0x06, 0x0C, 0x18, 0x00], // (
    [0x06, 0x0C, 0x18, 0x18, 0x18, 0x0C, 0x06, 0x00], // )
    [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00], // *
    [0x00, 0x0C, 0x0C, 0x3F, 0x0C, 0x0C, 0x00, 0x00], // +
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x06], // ,
    [0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00], // -
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x00], // .
    [0x60, 0x30, 0x18, 0x0C, 0x06, 0x03, 0x01, 0x00], // /
    [0x3E, 0x63, 0x73, 0x7B, 0x6F, 0x67, 0x3E, 0x00], // 0
    [0x0C, 0x0E, 0x0C, 0x0C, 0x0C, 0x0C, 0x3F, 0x00], // 1
    [0x1E, 0x33, 0x30, 0x1C, 0x06, 0x33, 0x3F, 0x00], // 2
    [0x1E, 0x33, 0x30, 0x1C, 0x30, 0x33, 0x1E, 0x00], // 3
    [0x38, 0x3C, 0x36, 0x33, 0x7F, 0x30, 0x78, 0x00], // 4
    [0x3F, 0x03, 0x1F, 0x30, 0x30, 0x33, 0x1E, 0x00], // 5
    [0x1C, 0x06, 0x03, 0x1F, 0x33, 0x33, 0x1E, 0x00], // 6
    [0x3F, 0x33, 0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x00], // 7
    [0x1E, 0x33, 0x33, 0x1E, 0x33, 0x33, 0x1E, 0x00], // 8
    [0x1E, 0x33, 0x33, 0x3E, 0x30, 0x18, 0x0E, 0x00], // 9
    [0x00, 0x0C, 0x0C, 0x00, 0x00, 0x0C, 0x0C, 0x00], // :
    [0x00, 0x0C, 0x0C, 0x00, 0x00, 0x0C, 0x0C, 0x06], // ;
    [0x18, 0x0C, 0x06, 0x03, 0x06, 0x0C, 0x18, 0x00], // <
    [0x00, 0x00, 0x3F, 0x00, 0x00, 0x3F, 0x00, 0x00], // =
    [0x06, 0x0C, 0x18, 0x30, 0x18, 0x0C, 0x06, 0x00], // >
    [0x1E, 0x33, 0x30, 0x18, 0x0C, 0x00, 0x0C, 0x00], // ?
    [0x3E, 0x63, 0x7B, 0x7B, 0x7B, 0x03, 0x1E, 0x00], // @
    [0x0C, 0x1E, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x00], // A
    [0x3F, 0x66, 0x66, 0x3E, 0x66, 0x66, 0x3F, 0x00], // B
    [0x3C, 0x66, 0x03, 0x03, 0x03, 0x66, 0x3C, 0x00], // C
    [0x1F, 0x36, 0x66, 0x66, 0x66, 0x36, 0x1F, 0x00], // D
    [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x46, 0x7F, 0x00], // E
    [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x06, 0x0F, 0x00], // F
    [0x3C, 0x66, 0x03, 0x03, 0x73, 0x66, 0x7C, 0x00], // G
    [0x33, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x33, 0x00], // H
    [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00], // I
    [0x78, 0x30, 0x30, 0x30, 0x33, 0x33, 0x1E, 0x00], // J
    [0x67, 0x66, 0x36, 0x1E, 0x36, 0x66, 0x67, 0x00], // K
    [0x0F, 0x06, 0x06, 0x06, 0x46, 0x66, 0x7F, 0x00], // L
    [0x63, 0x77, 0x7F, 0x7F, 0x6B, 0x63, 0x63, 0x00], // M
    [0x63, 0x67, 0x6F, 0x7B, 0x73, 0x63, 0x63, 0x00], // N
    [0x1C, 0x36, 0x63, 0x63, 0x63, 0x36, 0x1C, 0x00], // O
    [0x3F, 0x66, 0x66, 0x3E, 0x06, 0x06, 0x0F, 0x00], // P
    [0x1E, 0x33, 0x33, 0x33, 0x3B, 0x1E, 0x38, 0x00], // Q
    [0x3F, 0x66, 0x66, 0x3E, 0x36, 0x66, 0x67, 0x00], // R
    [0x1E, 0x33, 0x07, 0x0E, 0x38, 0x33, 0x1E, 0x00], // S
    [0x3F, 0x2D, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00], // T
    [0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x3F, 0x00], // U
    [0x33, 0x33, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00], // V
    [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00], // W
    [0x63, 0x63, 0x36, 0x1C, 0x1C, 0x36, 0x63, 0x00], // X
    [0x33, 0x33, 0x33, 0x1E, 0x0C, 0x0C, 0x1E, 0x00], // Y
    [0x7F, 0x63, 0x31, 0x18, 0x4C, 0x66, 0x7F, 0x00], // Z
    [0x1E, 0x06, 0x06, 0x06, 0x06, 0x06, 0x1E, 0x00], // [
    [0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x40, 0x00], // backslash
    [0x1E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1E, 0x00], // ]
    [0x08, 0x1C, 0x36, 0x63, 0x00, 0x00, 0x00, 0x00], // ^
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF], // _
    [0x0C, 0x0C, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00], // `
    [0x00, 0x00, 0x1E, 0x30, 0x3E, 0x33, 0x6E, 0x00], // a
    [0x07, 0x06, 0x06, 0x3E, 0x66, 0x66, 0x3B, 0x00], // b
    [0x00, 0x00, 0x1E, 0x33, 0x03, 0x33, 0x1E, 0x00], // c
    [0x38, 0x30, 0x30, 0x3E, 0x33, 0x33, 0x6E, 0x00], // d
    [0x00, 0x00, 0x1E, 0x33, 0x3F, 0x03, 0x1E, 0x00], // e
    [0x1C, 0x36, 0x06, 0x0F, 0x06, 0x06, 0x0F, 0x00], // f
    [0x00, 0x00, 0x6E, 0x33, 0x33, 0x3E, 0x30, 0x1F], // g
    [0x07, 0x06, 0x36, 0x6E, 0x66, 0x66, 0x67, 0x00], // h
    [0x0C, 0x00, 0x0E, 0x0C, 0x0C, 0x0C, 0x1E, 0x00], // i
    [0x30, 0x00, 0x30, 0x30, 0x30, 0x33, 0x33, 0x1E], // j
    [0x07, 0x06, 0x66, 0x36, 0x1E, 0x36, 0x67, 0x00], // k
    [0x0E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00], // l
    [0x00, 0x00, 0x33, 0x7F, 0x7F, 0x6B, 0x63, 0x00], // m
    [0x00, 0x00, 0x1F, 0x33, 0x33, 0x33, 0x33, 0x00], // n
    [0x00, 0x00, 0x1E, 0x33, 0x33, 0x33, 0x1E, 0x00], // o
    [0x00, 0x00, 0x3B, 0x66, 0x66, 0x3E, 0x06, 0x0F], // p
    [0x00, 0x00, 0x6E, 0x33, 0x33, 0x3E, 0x30, 0x78], // q
    [0x00, 0x00, 0x3B, 0x6E, 0x66, 0x06, 0x0F, 0x00], // r
    [0x00, 0x00, 0x3E, 0x03, 0x1E, 0x30, 0x1F, 0x00], // s
    [0x08, 0x0C, 0x3E, 0x0C, 0x0C, 0x2C, 0x18, 0x00], // t
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x33, 0x6E, 0x00], // u
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00], // v
    [0x00, 0x00, 0x63, 0x6B, 0x7F, 0x7F, 0x36, 0x00], // w
    [0x00, 0x00, 0x63, 0x36, 0x1C, 0x36, 0x63, 0x00], // x
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x3E, 0x30, 0x1F], // y
    [0x00, 0x00, 0x3F, 0x19, 0x0C, 0x26, 0x3F, 0x00], // z
    [0x38, 0x0C, 0x0C, 0x07, 0x0C, 0x0C, 0x38, 0x00], // {
    [0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00], // |
    [0x07, 0x0C, 0x0C, 0x38, 0x0C, 0x0C, 0x07, 0x00], // }
    [0x6E, 0x3B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // ~
];
