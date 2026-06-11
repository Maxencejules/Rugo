/* hello: a real C program against rlibc (gap item 9). Exercises printf,
 * malloc, open/read on the /data tree, and clean exit - the POSIX-ish
 * surface a port needs. Argument: a file path to read back. */

#include <rugo/libc.h>

int main(void) {
    printf("HELLOC: printf d=%d x=0x%x s=%s\n", 42, 0xFF, "works");

    char *box = malloc(64);
    if (!box) {
        puts("HELLOC: malloc err");
        return 1;
    }
    memset(box, 0, 64);
    memcpy(box, rugo_args, rugo_args_len < 63 ? rugo_args_len : 63);
    printf("HELLOC: args=%s\n", box);

    if (rugo_args_len > 0) {
        long fd = open(rugo_args, O_RDONLY, 0);
        if (fd == RUGO_ERR) {
            puts("HELLOC: open err");
            return 1;
        }
        char buf[128];
        ssize_t n = read(fd, buf, sizeof(buf) - 1);
        close(fd);
        if (n == RUGO_ERR) {
            puts("HELLOC: read err");
            return 1;
        }
        buf[n] = 0;
        printf("HELLOC: file[%d]=%s\n", (long)n, buf);
    }

    puts("HELLOC: done");
    return 0;
}
