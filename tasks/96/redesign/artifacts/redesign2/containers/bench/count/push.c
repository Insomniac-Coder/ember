/* Cost of keeping a "walkers" count in the top byte of an Array's capacity word (the fallback that
   makes a list hold its order while a loop or a name needs it and an unseen call could shift it).
   The only change on the hot path is push: it must mask the count off before comparing len with cap.
   400,000,000 pushes of u32 into one list, emptied every 1,000,000 pushes (capacity kept), with and
   without the mask (MASKED). Shifting operations (remove, insert, sort, clear) also test the byte,
   but each of those moves memory anyway, so only push is measured. */
#include "../harness.h"
#define CAP_MASK 0x00FFFFFFFFFFFFFFull
typedef struct { uint32_t *data; uint64_t len, cap; } Arr;
#ifdef MASKED
#define CAPOF(a) ((a)->cap & CAP_MASK)
#define NAME "count-push-masked"
#else
#define CAPOF(a) ((a)->cap)
#define NAME "count-push-plain"
#endif
static __attribute__((noinline)) void grow(Arr *a) {
    uint64_t walkers = a->cap & ~CAP_MASK, c = CAPOF(a) ? CAPOF(a) * 2 : 4;
    a->data = realloc(a->data, c * 4); a->cap = c | walkers;
}
static inline void push(Arr *a, uint32_t v) { if (a->len == CAPOF(a)) grow(a); a->data[a->len++] = v; }
int main(void) {
    h_start();
    Arr a = {0}; uint64_t s = 0;
#ifdef MASKED
    a.cap = 1ull << 56;                 /* one walker registered: the mask must work */
#endif
    for (uint32_t round = 0; round < 400; round++) {
        for (uint32_t i = 0; i < 1000000; i++) push(&a, i ^ round);
        s += a.data[round * 997] + a.len; a.len = 0;
    }
    h_report(NAME, s);
    return 0;
}
