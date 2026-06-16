/* rlibc: Rugo's libc-equivalent (gap item 9). A freestanding POSIX-ish
 * layer over the int 0x80 ABI v3 syscall surface, for C programs that
 * run from the package store. v1 covers what the kernel provides:
 * file/dir I/O on /data, pipes, spawn/wait, console output, a bump
 * heap, strings, and a printf subset. */

#ifndef RUGO_LIBC_H
#define RUGO_LIBC_H

typedef unsigned long size_t;
typedef long ssize_t;
typedef unsigned long uintptr_t;
typedef long intptr_t;

#define NULL ((void *)0)

/* ---- syscall numbers (docs/abi/syscall_v3.md) ---- */
#define SYS_DEBUG_WRITE 0
#define SYS_THREAD_EXIT 2
#define SYS_YIELD 3
#define SYS_OPEN 18
#define SYS_READ 19
#define SYS_WRITE 20
#define SYS_CLOSE 21
#define SYS_WAIT 22
#define SYS_PROC_INFO 28
#define SYS_SPAWN 46
#define SYS_FS_CTL 47

/* fs_ctl ops */
#define FS_CTL_MKDIR 1
#define FS_CTL_UNLINK 2
#define FS_CTL_STAT 3
#define FS_CTL_PIPE 4

#define O_RDONLY 0x0
#define O_WRONLY 0x1
#define O_RDWR 0x2
#define O_CREAT 0x4

#define RUGO_ERR ((long)-1)

long rugo_syscall3(long nr, long a1, long a2, long a3);
long rugo_syscall6(long nr, long a1, long a2, long a3, long a4, long a5,
                   long a6);

/* ---- process entry state (set by crt0) ---- */
extern const char *rugo_args;   /* NUL-terminated spawn argument string */
extern size_t rugo_args_len;
extern long rugo_stdin_fd;      /* -1 when not piped */
extern long rugo_stdout_fd;     /* -1 = console */

/* ---- unistd-ish ---- */
long open(const char *path, long flags, long mode);
ssize_t read(long fd, void *buf, size_t len);
ssize_t write(long fd, const void *buf, size_t len);
long close(long fd);
long mkdir(const char *path);
long unlink(const char *path);
long stat_kind_size(const char *path, long *kind, long *size);
long pipe2fds(long *rfd, long *wfd);
long spawn(const char *name, const char *args, long stdin_fd,
           long stdout_fd);
long waitpid(long tid);
void yield(void);
void exit(int code) __attribute__((noreturn));

/* ---- heap (bump allocator in the demand-paged exec window) ---- */
void *malloc(size_t n);
void free(void *p);
void *sbrk(intptr_t inc);

/* ---- string.h subset ---- */
void *memset(void *dst, int c, size_t n);
void *memcpy(void *dst, const void *src, size_t n);
void *memmove(void *dst, const void *src, size_t n);
size_t strlen(const char *s);
int strcmp(const char *a, const char *b);
int strncmp(const char *a, const char *b, size_t n);
int memcmp(const void *a, const void *b, size_t n);
/* rlibc v2 string helpers */
char *strcpy(char *dst, const char *src);
char *strncpy(char *dst, const char *src, size_t n);
char *strcat(char *dst, const char *src);
char *strchr(const char *s, int c);
int atoi(const char *s);

/* ---- errno (rlibc v2) ---- */
extern int errno;
#define EIO 5

/* ---- stdio subset (console or rugo_stdout_fd) ---- */
int putchar(int c);
int puts(const char *s);
int printf(const char *fmt, ...);

#endif /* RUGO_LIBC_H */
