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

/* ---- heap: bump allocator in the demand-paged exec window ---- */

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

void *malloc(size_t n) {
    n = (n + 15) & ~(size_t)15;
    void *p = sbrk((intptr_t)n);
    return p == (void *)-1 ? NULL : p;
}

void free(void *p) { (void)p; /* bump heap: no-op in v1 */ }

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
