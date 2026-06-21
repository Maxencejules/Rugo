/* rlibc implementation: POSIX-ish wrappers over the int 0x80 ABI,
 * strings, a bump heap, and a printf subset. Compiled -mabi=sysv,
 * freestanding; the only asm is in crt0.asm. */

#include <rugo/libc.h>

const char *rugo_args;
size_t rugo_args_len;
long rugo_stdin_fd;
long rugo_stdout_fd;

/* rlibc v2 (full-os guide Part V.11): errno, set by the syscall wrappers when a
 * call fails (the kernel ABI returns RUGO_ERR / -1 on error). */
int errno;

/* mingw gcc emits a call to __main at the top of main() as a
 * constructor hook; freestanding programs stub it out. */
void __main(void) {}

/* ---- unistd-ish ---- */

long open(const char *path, long flags, long mode) {
    long r = rugo_syscall3(SYS_OPEN, (long)path, flags, mode);
    if (r == RUGO_ERR)
        errno = EIO;
    return r;
}

ssize_t read(long fd, void *buf, size_t len) {
    ssize_t r = rugo_syscall3(SYS_READ, fd, (long)buf, (long)len);
    if (r == RUGO_ERR)
        errno = EIO;
    return r;
}

ssize_t write(long fd, const void *buf, size_t len) {
    ssize_t r = rugo_syscall3(SYS_WRITE, fd, (long)buf, (long)len);
    if (r == RUGO_ERR)
        errno = EIO;
    return r;
}

long close(long fd) { return rugo_syscall3(SYS_CLOSE, fd, 0, 0); }

long mkdir(const char *path) {
    return rugo_syscall3(SYS_FS_CTL, FS_CTL_MKDIR, (long)path, 0);
}

long unlink(const char *path) {
    return rugo_syscall3(SYS_FS_CTL, FS_CTL_UNLINK, (long)path, 0);
}

long stat_kind_size(const char *path, long *kind, long *size) {
    long r = rugo_syscall3(SYS_FS_CTL, FS_CTL_STAT, (long)path, 0);
    if (r == RUGO_ERR)
        return RUGO_ERR;
    if (kind)
        *kind = (r >> 32) & 0xFF;
    if (size)
        *size = r & 0xFFFFFFFF;
    return 0;
}

long pipe2fds(long *rfd, long *wfd) {
    long r = rugo_syscall3(SYS_FS_CTL, FS_CTL_PIPE, 0, 0);
    if (r == RUGO_ERR)
        return RUGO_ERR;
    *rfd = (r >> 8) & 0xFF;
    *wfd = r & 0xFF;
    return 0;
}

long spawn(const char *name, const char *args, long stdin_fd,
           long stdout_fd) {
    size_t alen = args ? strlen(args) : 0;
    return rugo_syscall6(SYS_SPAWN, (long)name, (long)strlen(name),
                         (long)args, (long)alen, stdin_fd, stdout_fd);
}

long waitpid(long tid) { return rugo_syscall3(SYS_WAIT, tid, 0, 0); }

void yield(void) { rugo_syscall3(SYS_YIELD, 0, 0, 0); }

void exit(int code) {
    rugo_syscall3(SYS_THREAD_EXIT, code, 0, 0);
    for (;;)
        ;
}

/* ---- heap: free-list allocator over a bump region in the demand-paged
 * exec window. malloc carves 16-byte-aligned blocks behind a size header;
 * free pushes them onto a LIFO free list so the next fitting malloc reuses
 * the freed block instead of growing the heap (rlibc v2: a real free(),
 * replacing the v1 no-op). ---- */

#define HEAP_BASE 0x01600000UL
#define HEAP_END 0x017F0000UL

static uintptr_t heap_brk = HEAP_BASE;

void *sbrk(intptr_t inc) {
    uintptr_t old = heap_brk;
    if (heap_brk + (uintptr_t)inc > HEAP_END)
        return (void *)-1;
    heap_brk += (uintptr_t)inc;
    return (void *)old;
}

/* 16-byte block header (keeps the payload 16-byte aligned); `size` is the
 * usable payload size. A freed block stores the next free pointer in its
 * own payload, which is always >= 16 bytes. */
typedef struct blk_hdr {
    size_t size;
    size_t _pad;
} blk_hdr;

static void *free_head; /* LIFO list of free payload pointers */

void *malloc(size_t n) {
    n = (n + 15) & ~(size_t)15;
    if (n < 16)
        n = 16; /* the payload must be able to hold the free-list link */
    /* first-fit reuse from the free list */
    void **prev = &free_head;
    void *cur = free_head;
    while (cur) {
        blk_hdr *h = (blk_hdr *)((uintptr_t)cur - sizeof(blk_hdr));
        if (h->size >= n) {
            *prev = *(void **)cur; /* unlink */
            return cur;
        }
        prev = (void **)cur;
        cur = *(void **)cur;
    }
    /* otherwise bump a fresh header+payload block */
    void *raw = sbrk((intptr_t)(sizeof(blk_hdr) + n));
    if (raw == (void *)-1)
        return NULL;
    ((blk_hdr *)raw)->size = n;
    return (void *)((uintptr_t)raw + sizeof(blk_hdr));
}

void free(void *p) {
    if (!p)
        return;
    *(void **)p = free_head; /* push onto the free list */
    free_head = p;
}

/* ---- string.h subset ---- */

void *memset(void *dst, int c, size_t n) {
    unsigned char *d = dst;
    while (n--)
        *d++ = (unsigned char)c;
    return dst;
}

void *memcpy(void *dst, const void *src, size_t n) {
    unsigned char *d = dst;
    const unsigned char *s = src;
    while (n--)
        *d++ = *s++;
    return dst;
}

void *memmove(void *dst, const void *src, size_t n) {
    unsigned char *d = dst;
    const unsigned char *s = src;
    if (d < s) {
        while (n--)
            *d++ = *s++;
    } else {
        d += n;
        s += n;
        while (n--)
            *--d = *--s;
    }
    return dst;
}

size_t strlen(const char *s) {
    size_t n = 0;
    while (s[n])
        n++;
    return n;
}

int strcmp(const char *a, const char *b) {
    while (*a && *a == *b) {
        a++;
        b++;
    }
    return (unsigned char)*a - (unsigned char)*b;
}

int strncmp(const char *a, const char *b, size_t n) {
    while (n && *a && *a == *b) {
        a++;
        b++;
        n--;
    }
    return n ? (unsigned char)*a - (unsigned char)*b : 0;
}

int memcmp(const void *a, const void *b, size_t n) {
    const unsigned char *x = a, *y = b;
    while (n--) {
        if (*x != *y)
            return *x - *y;
        x++;
        y++;
    }
    return 0;
}

/* ---- rlibc v2 string helpers (full-os guide Part V.11) ---- */

char *strcpy(char *dst, const char *src) {
    char *d = dst;
    while ((*d++ = *src++)) {
    }
    return dst;
}

char *strncpy(char *dst, const char *src, size_t n) {
    size_t i = 0;
    for (; i < n && src[i]; i++)
        dst[i] = src[i];
    for (; i < n; i++)
        dst[i] = 0;
    return dst;
}

char *strcat(char *dst, const char *src) {
    char *d = dst + strlen(dst);
    while ((*d++ = *src++)) {
    }
    return dst;
}

char *strchr(const char *s, int c) {
    for (;; s++) {
        if (*s == (char)c)
            return (char *)s;
        if (!*s)
            return NULL;
    }
}

int atoi(const char *s) {
    int sign = 1, v = 0;
    while (*s == ' ' || *s == '\t')
        s++;
    if (*s == '-') {
        sign = -1;
        s++;
    } else if (*s == '+') {
        s++;
    }
    while (*s >= '0' && *s <= '9') {
        v = v * 10 + (*s - '0');
        s++;
    }
    return sign * v;
}

/* ---- stdio subset ---- */

static void out_flush(const char *buf, size_t n) {
    if (n == 0)
        return;
    if (rugo_stdout_fd != -1) {
        size_t off = 0;
        while (off < n) {
            ssize_t w = write(rugo_stdout_fd, buf + off, n - off);
            if (w == RUGO_ERR) {
                yield(); /* pipe full */
                continue;
            }
            off += (size_t)w;
        }
    } else {
        rugo_syscall3(SYS_DEBUG_WRITE, (long)buf, (long)n, 0);
    }
}

int putchar(int c) {
    char b = (char)c;
    out_flush(&b, 1);
    return c;
}

int puts(const char *s) {
    out_flush(s, strlen(s));
    out_flush("\n", 1);
    return 0;
}

/* ---- buffered FILE* stdio (rlibc v3): a read buffer over the fd layer, so
 * C ports get fopen/fgetc/fread/feof/fclose instead of raw open/read. ---- */

struct __FILE {
    long fd;
    int used;
    int eof;
    int wmode;  /* opened for writing: buf holds pending output, len = count */
    size_t pos; /* next unread byte in buf (read mode) */
    size_t len; /* read mode: valid bytes in buf; write mode: buffered count */
    unsigned char buf[256];
};

#define FOPEN_MAX 8
static struct __FILE file_pool[FOPEN_MAX];

FILE *fopen(const char *path, const char *mode) {
    long flags = O_RDONLY;
    if (mode && mode[0] == 'w')
        flags = O_WRONLY | O_CREAT;
    else if (mode && mode[0] == 'r' && mode[1] == '+')
        flags = O_RDWR;
    long fd = open(path, flags, 0);
    if (fd == RUGO_ERR)
        return NULL;
    for (int i = 0; i < FOPEN_MAX; i++) {
        if (!file_pool[i].used) {
            FILE *f = &file_pool[i];
            f->fd = fd;
            f->used = 1;
            f->eof = 0;
            f->wmode = (mode && mode[0] == 'w') ? 1 : 0;
            f->pos = 0;
            f->len = 0;
            return f;
        }
    }
    close(fd); /* pool exhausted */
    return NULL;
}

/* Ensure at least one buffered byte; returns 0 at EOF/error. */
static int frefill(FILE *f) {
    if (f->pos < f->len)
        return 1;
    ssize_t n = read(f->fd, f->buf, sizeof(f->buf));
    if (n <= 0) {
        f->eof = 1;
        return 0;
    }
    f->pos = 0;
    f->len = (size_t)n;
    return 1;
}

int fgetc(FILE *f) {
    if (!f || !frefill(f))
        return EOF;
    return f->buf[f->pos++];
}

size_t fread(void *ptr, size_t size, size_t nmemb, FILE *f) {
    if (!f || size == 0)
        return 0;
    size_t total = size * nmemb;
    size_t got = 0;
    unsigned char *out = ptr;
    while (got < total && frefill(f))
        out[got++] = f->buf[f->pos++];
    return got / size;
}

int feof(FILE *f) { return f ? f->eof : 1; }

/* Drain the write buffer to the fd (write mode only). */
int fflush(FILE *f) {
    if (!f || !f->wmode)
        return 0;
    size_t off = 0;
    while (off < f->len) {
        ssize_t w = write(f->fd, f->buf + off, f->len - off);
        if (w == RUGO_ERR) {
            f->eof = 1;
            return EOF;
        }
        off += (size_t)w;
    }
    f->len = 0;
    return 0;
}

int fputc(int c, FILE *f) {
    if (!f || !f->wmode)
        return EOF;
    f->buf[f->len++] = (unsigned char)c;
    if (f->len == sizeof(f->buf) && fflush(f) == EOF)
        return EOF;
    return c;
}

size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *f) {
    if (!f || !f->wmode || size == 0)
        return 0;
    const unsigned char *in = ptr;
    size_t total = size * nmemb, i;
    for (i = 0; i < total; i++)
        if (fputc(in[i], f) == EOF)
            break;
    return i / size;
}

int fclose(FILE *f) {
    if (!f || !f->used)
        return RUGO_ERR;
    if (f->wmode)
        fflush(f);
    long r = close(f->fd);
    f->used = 0;
    return r == RUGO_ERR ? RUGO_ERR : 0;
}

/* printf subset: %s %c %d %u %x %% with single-write line buffering so
 * concurrent tasks cannot splice the output. */

typedef __builtin_va_list va_list;
#define va_start(v, l) __builtin_va_start(v, l)
#define va_arg(v, t) __builtin_va_arg(v, t)
#define va_end(v) __builtin_va_end(v)

static size_t fmt_u(char *dst, unsigned long v, unsigned base, int upper) {
    const char *digits = upper ? "0123456789ABCDEF" : "0123456789abcdef";
    char tmp[24];
    size_t n = 0;
    do {
        tmp[n++] = digits[v % base];
        v /= base;
    } while (v);
    for (size_t i = 0; i < n; i++)
        dst[i] = tmp[n - 1 - i];
    return n;
}

int printf(const char *fmt, ...) {
    char buf[256];
    size_t n = 0;
    va_list ap;
    va_start(ap, fmt);
    for (const char *p = fmt; *p && n < sizeof(buf) - 24; p++) {
        if (*p != '%') {
            buf[n++] = *p;
            continue;
        }
        p++;
        switch (*p) {
        case 's': {
            const char *s = va_arg(ap, const char *);
            if (!s)
                s = "(null)";
            while (*s && n < sizeof(buf) - 1)
                buf[n++] = *s++;
            break;
        }
        case 'c':
            buf[n++] = (char)va_arg(ap, int);
            break;
        case 'd': {
            long v = va_arg(ap, long);
            if (v < 0) {
                buf[n++] = '-';
                v = -v;
            }
            n += fmt_u(buf + n, (unsigned long)v, 10, 0);
            break;
        }
        case 'u':
            n += fmt_u(buf + n, va_arg(ap, unsigned long), 10, 0);
            break;
        case 'x':
            n += fmt_u(buf + n, va_arg(ap, unsigned long), 16, 0);
            break;
        case 'X':
            n += fmt_u(buf + n, va_arg(ap, unsigned long), 16, 1);
            break;
        case '%':
            buf[n++] = '%';
            break;
        default:
            buf[n++] = '%';
            if (n < sizeof(buf) - 1)
                buf[n++] = *p;
            break;
        }
    }
    va_end(ap);
    out_flush(buf, n);
    return (int)n;
}
