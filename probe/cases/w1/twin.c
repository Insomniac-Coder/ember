#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct { int64_t* data; size_t len, cap; } Vec;

static void push(Vec* v, int64_t x) {
    if (v->len == v->cap) {
        v->cap = v->cap ? v->cap * 2 : 4;
        v->data = (int64_t*)realloc(v->data, v->cap * sizeof *v->data);
    }
    v->data[v->len++] = x;
}

typedef struct { Vec items; } Bag;

int main(void) {
    Bag* bags[2];
    for (int k = 0; k < 2; k++) {
        Bag* b = (Bag*)calloc(1, sizeof *b);
        for (int64_t i = 0; i < 1000; i++) push(&b->items, i % 7);
        bags[k] = b;
    }
    int64_t total = 0;
    for (int64_t round = 0; round < 300000; round++) {
        Bag* b = bags[round % 2];
        for (int64_t i = 0; i < 1000; i++) total += b->items.data[i];
    }
    printf("%lld\n", (long long)total);
    return 0;
}
