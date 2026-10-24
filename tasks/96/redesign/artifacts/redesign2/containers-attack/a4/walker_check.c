/* Self-check (not timed) of the saved A4 rule for code the compiler cannot see: while a loop walks a
   list (r, w published in the loop's frame, the walking state in the field's access word), every
   operation other code makes through the field is mapped onto the logical list
       logical = data[0, w) ++ data[r, len)     (data[r] left out once the current element is removed)
   so that code sees Python's list at once: len, reading an element, removing any element (the current
   one included), appending. The loop itself keeps the one-pass filter (rules 1 to 4).
   Reference: a plain array holding the logical list and the index c of the current element, with
   Python's meaning minus its skip: removing the current element does not skip the next one.
   Random walks: each step the "callback" removes 0 to 2 random elements, sometimes the current one,
   appends, and reads; after every operation the mapped list must equal the reference, and at the end
   the visit order and the final list must be equal too. Prints "walker-check ok". */
#include "../harness.h"
#include <string.h>
#define FAIL(...) do { printf(__VA_ARGS__); printf("\n"); exit(1); } while (0)
#define MAXN 4096
static uint32_t data[MAXN]; static size_t len;                  /* the list */
static size_t r, w; static int cur_removed;                     /* the walker, published in its frame */
static uint32_t ref[MAXN]; static size_t ref_len, c; static int ref_cur_removed;   /* the reference */
static uint32_t visit_m[MAXN * 4], visit_r[MAXN * 4]; static size_t nv_m, nv_r;

static size_t m_len(void) { return len - (r - w) - (size_t)cur_removed; }
static size_t m_phys(size_t k) { return k < w ? k : k + (r - w) + (size_t)cur_removed; }
static uint32_t m_get(size_t k) { if (k >= m_len()) FAIL("index out of bounds"); return data[m_phys(k)]; }
static void m_remove(size_t k) {
    if (k >= m_len()) FAIL("index out of bounds");
    if (k < w) { memmove(&data[k], &data[k + 1], (w - k - 1) * 4); data[w - 1] = 0xDEAD; w--; return; }  /* the gap grows */
    size_t p = m_phys(k);
    if (p == r) { cur_removed = 1; return; }                    /* the loop will not keep it */
    memmove(&data[p], &data[p + 1], (len - p - 1) * 4); len--;
}
static void m_push(uint32_t x) { if (len == MAXN) FAIL("full"); data[len++] = x; }

static void ref_remove(size_t k) {
    memmove(&ref[k], &ref[k + 1], (ref_len - k - 1) * 4); ref_len--;
    if (k < c) c--; else if (k == c) ref_cur_removed = 1;
}
static void compare(uint64_t seed, const char *where) {
    if (m_len() != ref_len) FAIL("seed %llu %s: len %zu vs %zu", (unsigned long long)seed, where, m_len(), ref_len);
    for (size_t k = 0; k < ref_len; k++)
        if (m_get(k) != ref[k]) FAIL("seed %llu %s: element %zu differs", (unsigned long long)seed, where, k);
}

int main(void) {
    h_start();
    for (uint64_t seed = 1; seed <= 3000; seed++) {
        h_seed(seed);
        uint32_t next = 1; len = ref_len = 0; nv_m = nv_r = 0;
        size_t n0 = h_rand() % 60;
        for (size_t i = 0; i < n0; i++) { data[len++] = next; ref[ref_len++] = next; next++; }
        /* the reference walk and the mapped walk run side by side, driven by the same random choices */
        r = w = 0; c = 0;
        while (r < len) {
            cur_removed = 0; ref_cur_removed = 0;
            if (c >= ref_len) FAIL("seed %llu: reference ended early", (unsigned long long)seed);
            visit_m[nv_m++] = data[r]; visit_r[nv_r++] = ref[c];
            uint32_t act = h_rand() % 10;
            int removes = act < 4 ? (int)(h_rand() % 3) : 0;
            for (int q = 0; q < removes && ref_len > 0; q++) {
                size_t k = (h_rand() % 4 == 0 && !ref_cur_removed && c < ref_len) ? c : h_rand() % ref_len;
                if (k == c && ref_cur_removed) continue;
                m_remove(k); ref_remove(k); compare(seed, "after remove");
            }
            if (act >= 4 && act < 6 && len < MAXN - 8) { m_push(next); ref[ref_len++] = next; next++; compare(seed, "after push"); }
            if (act >= 6 && ref_len > 0) { size_t k = h_rand() % ref_len; if (m_get(k) != ref[k]) FAIL("seed %llu: read differs", (unsigned long long)seed); }
            /* the end of the step: the loop's own rule (keep e at w, or drop it) */
            if (!cur_removed) { data[w] = data[r]; w++; }
            r++;
            if (!ref_cur_removed) c++;
        }
        len = w;
        if (c != ref_len) FAIL("seed %llu: reference not finished", (unsigned long long)seed);
        if (nv_m != nv_r) FAIL("seed %llu: %zu visits vs %zu", (unsigned long long)seed, nv_m, nv_r);
        for (size_t k = 0; k < nv_m; k++) if (visit_m[k] != visit_r[k]) FAIL("seed %llu: visit %zu differs", (unsigned long long)seed, k);
        if (len != ref_len) FAIL("seed %llu: final len", (unsigned long long)seed);
        for (size_t k = 0; k < len; k++) if (data[k] != ref[k]) FAIL("seed %llu: final element %zu", (unsigned long long)seed, k);
    }
    printf("walker-check ok\n");
    return 0;
}
