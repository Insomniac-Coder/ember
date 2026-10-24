/* A3 rule 2 makes two parameters of one type "possibly the same place". For two list parameters that
   means: a loop over one list that pushes to the other must read the walked list's length and buffer
   again after every push (the push may have changed them, if the two are one list).
       fn copy_alive(src: Array[Enemy], mut dst: Array[Enemy]):    def copy_alive(src, dst):
           for e in src:                                                for e in src:
               if e.hp > 0: dst.push(e)                                     if e.hp > 0: dst.append(e)
   Called as copy_alive(a.units, b.units) where a and b are two elements of one list (rule 1 allows it).
   1,000,000 enemies (16 bytes), about 90% alive, 100 calls; dst is emptied before each call.
     RESTRICT : the two list headers cannot alias (Rust's &Vec and &mut Vec).
     ALIAS    : they may (the answer as written).
     CALLCHECK: saved: the call compares the two headers' addresses and calls the RESTRICT copy when
                they differ, the ALIAS copy when they are one list. */
#include "../harness.h"
#define N 1000000
typedef struct { uint32_t id; int32_t hp; float x, vx; } Enemy;
typedef struct { Enemy *data; size_t len, cap; } List;
static __attribute__((noinline)) void grow(List *l) { l->cap = l->cap ? l->cap * 2 : 4; l->data = realloc(l->data, l->cap * sizeof(Enemy)); }
#define PUSH(l, e) do { if ((l)->len == (l)->cap) grow(l); (l)->data[(l)->len++] = (e); } while (0)
static __attribute__((noinline)) void copy_alive_r(const List *restrict src, List *restrict dst) {
    for (size_t i = 0; i < src->len; i++) { Enemy e = src->data[i]; if (e.hp > 0) PUSH(dst, e); }
}
static __attribute__((noinline)) void copy_alive_a(const List *src, List *dst) {
    for (size_t i = 0; i < src->len; i++) { Enemy e = src->data[i]; if (e.hp > 0) PUSH(dst, e); }
}
#if defined(RESTRICT)
#define CALL(s, d) copy_alive_r(s, d)
#define MODE "restrict"
#elif defined(ALIAS)
#define CALL(s, d) copy_alive_a(s, d)
#define MODE "alias"
#else
#define CALL(s, d) ((const void *)(s) != (const void *)(d) ? copy_alive_r(s, d) : copy_alive_a(s, d))
#define MODE "callcheck"
#endif
int main(void) {
    h_start();
    static List a_units, b_units;
    List *volatile ap = &a_units, *volatile bp = &b_units;   /* the compiler does not know they differ */
    h_seed(5);
    for (uint32_t i = 0; i < N; i++) PUSH(&a_units, ((Enemy){i, (int32_t)(h_rand() % 10) - 1, (float)(i % 100), 1.0f}));
    uint64_t s = 0;
    for (int c = 0; c < 100; c++) {
        List *src = ap, *dst = bp;
        dst->len = 0;
        CALL(src, dst);
        s += dst->len + (uint64_t)dst->data[dst->len / 2].id;
    }
    char name[64]; snprintf(name, sizeof name, "a3-lists-%s", MODE);
    h_report(name, s);
    return 0;
}
