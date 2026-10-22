/* The C side of the FFI-8 tests of unions: a union by value and behind a pointer, and a struct with
   an anonymous union member. */
#ifndef UNIONS_H
#define UNIONS_H
#include <stdint.h>
union Number { int32_t i; float f; uint8_t bytes[4]; };
struct Value {
    int32_t kind;
    union { int64_t whole; double real; };
};
static inline union Number number_of_float(float f) { union Number n; n.f = f; return n; }
static inline int32_t number_bits(union Number n) { return n.i; }
static inline void number_twice(union Number* n) { n->i *= 2; }
static inline struct Value value_real(double d) { struct Value v; v.kind = 2; v.real = d; return v; }
static inline double value_half(struct Value v) { return v.kind == 2 ? v.real / 2 : (double)v.whole / 2; }
#endif
