/* Twin of display_format.em: five million points, each formatted as "(x, y)"
   into a freshly allocated buffer, summing the lengths. */
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    long long total = 0;
    for (long long i = 0; i < 5000000; ++i) {
        char *s = malloc(64);
        if (!s) return 1;
        int n = snprintf(s, 64, "(%lld, %lld)", i, -i);
        total += n;
        free(s);
    }
    printf("%lld\n", total);
    return 0;
}
