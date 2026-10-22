/* Compiled by `import c "./labels.c"` in DIA-5's accept case, which links and runs. */
#include "labels.h"
#include <string.h>

int label_length(const char *text) {
    return (int)strlen(text);
}

int label_sum(const int *items, size_t n) {
    int total = 0;
    for (size_t i = 0; i < n; i++) {
        total += items[i];
    }
    return total;
}
