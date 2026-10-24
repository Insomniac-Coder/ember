/* vcall.c: the cost of finding a virtual method through the object header, today's way and the lean
 * class's way (phase3b-ownership.md 2.3: "a 32-bit type index (for virtual calls, casts and dropping
 * through an interface)").
 *   -DMODE=0  today: the header holds a pointer to the class's type record; the record holds the
 *             method pointers (load the pointer, load the method, call)
 *   -DMODE=1  lean, table of pointers: the header holds a 32-bit index; a global table maps the index to
 *             the type record (load the index, load the record pointer, load the method, call)
 *   -DMODE=2  lean, records in the table: the global table holds the records themselves at a fixed
 *             size (load the index, load the method, call)
 * 8 classes, 65,536 objects of random classes (one heap block each), 2^27 calls in a random object
 * order, each method adding a class-specific amount to a field of the object.
 * Every object is 32 bytes in every mode (the lean ones padded), so only the way to the method differs.
 * -DONECLASS: every object is of class 3, so the call always goes to the same method (well predicted).
 * Build: clang -O2 -DMODE=n [-DONECLASS] vcall.c -o vcall_n.exe */
#include "../../../redesign/bench/harness.h"
#ifndef MODE
#define MODE 0
#endif
#define NOBJ (1u << 16)
#define CALLS (1u << 27)
typedef struct Obj Obj;
typedef struct { void (*update)(Obj *); void (*other)(Obj *); uint32_t size; uint32_t flags; } TypeRec;
#if MODE == 0
struct Obj { uint32_t strong, weak, access, flags; const TypeRec *ti; int64_t v; };
#else
struct Obj { uint32_t strong, type_index; int64_t v; char pad[16]; };   /* padded to today's 32 bytes, so only the lookup differs */
#endif
#define M(k) static __declspec(noinline) void upd##k(Obj *o) { o->v += k + 1; } \
             static void oth##k(Obj *o) { o->v -= k; }
M(0) M(1) M(2) M(3) M(4) M(5) M(6) M(7)
static const TypeRec recs[8] = {
    {upd0, oth0, 24, 0}, {upd1, oth1, 24, 0}, {upd2, oth2, 24, 0}, {upd3, oth3, 24, 0},
    {upd4, oth4, 24, 0}, {upd5, oth5, 24, 0}, {upd6, oth6, 24, 0}, {upd7, oth7, 24, 0}};
static const TypeRec *rec_ptrs[8] = {&recs[0], &recs[1], &recs[2], &recs[3], &recs[4], &recs[5], &recs[6], &recs[7]};
int main(void) {
    h_start();
    h_seed(11);
    Obj **objs = (Obj **)malloc(NOBJ * sizeof(Obj *));
    uint32_t *order = (uint32_t *)malloc(NOBJ * sizeof(uint32_t));
    if (!objs || !order) abort();
    for (uint32_t i = 0; i < NOBJ; i++) {
        uint32_t k = h_rand() & 7;
#ifdef ONECLASS
        k = 3;
#endif
        Obj *o = (Obj *)calloc(1, sizeof(Obj));
        if (!o) abort();
        o->strong = 1;
#if MODE == 0
        o->ti = &recs[k];
#else
        o->type_index = k;
#endif
        objs[i] = o;
        order[i] = h_rand() & (NOBJ - 1);
    }
    for (uint32_t c = 0; c < CALLS; c++) {
        Obj *o = objs[order[c & (NOBJ - 1)] ^ (c >> 16 & (NOBJ - 1))];
#if MODE == 0
        o->ti->update(o);
#elif MODE == 1
        rec_ptrs[o->type_index]->update(o);
#else
        recs[o->type_index].update(o);
#endif
    }
    uint64_t sum = 0;
    for (uint32_t i = 0; i < NOBJ; i++) sum += (uint64_t)objs[i]->v;
#ifdef ONECLASS
#define SUFFIX "-oneclass"
#else
#define SUFFIX "-mixed"
#endif
#if MODE == 0
    h_report("vcall-type-pointer" SUFFIX, sum);
#elif MODE == 1
    h_report("vcall-index-table-of-pointers" SUFFIX, sum);
#else
    h_report("vcall-index-records-in-table" SUFFIX, sum);
#endif
    return 0;
}
