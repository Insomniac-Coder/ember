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
    Bag* bags[18];
    for (int64_t n = 0; n < 18; n++) {
        Bag* b = (Bag*)calloc(1, sizeof *b);
        push(&b->items, n);
        bags[n] = b;
    }
    const int64_t* s0 = bags[2]->items.data;
    const int64_t* s1 = bags[3]->items.data;
    const int64_t* s2 = bags[4]->items.data;
    const int64_t* s3 = bags[5]->items.data;
    const int64_t* s4 = bags[6]->items.data;
    const int64_t* s5 = bags[7]->items.data;
    const int64_t* s6 = bags[8]->items.data;
    const int64_t* s7 = bags[9]->items.data;
    const int64_t* s8 = bags[10]->items.data;
    const int64_t* s9 = bags[11]->items.data;
    const int64_t* s10 = bags[12]->items.data;
    const int64_t* s11 = bags[13]->items.data;
    const int64_t* s12 = bags[14]->items.data;
    const int64_t* s13 = bags[15]->items.data;
    const int64_t* s14 = bags[16]->items.data;
    const int64_t* s15 = bags[17]->items.data;
    int64_t total = 0;
    for (int64_t round = 0; round < 100000; round++) {
        Bag* sink = bags[round % 2];
        sink->items.len = 0;
        for (int64_t i = 0; i < 1000; i++) push(&sink->items, i);
        total += (int64_t)sink->items.len;
    }
    printf("%lld %lld\n", (long long)total, (long long)(s0[0] + s1[0] + s2[0] + s3[0] + s4[0] + s5[0] + s6[0] + s7[0] + s8[0] + s9[0] + s10[0] + s11[0] + s12[0] + s13[0] + s14[0] + s15[0]));
    return 0;
}
