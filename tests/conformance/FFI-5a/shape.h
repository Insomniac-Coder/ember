/* The C side of accept_a_struct_ember_builds_and_passes.em. */
#ifndef SHAPE_H
#define SHAPE_H
#include <stdint.h>

typedef struct Mixed { uint8_t a; double b; uint16_t c; } Mixed;

static inline double shape_sum(Mixed m) { return m.a + m.b + m.c; }
static inline Mixed shape_make(uint8_t a) { Mixed m = { a, 0.5, 3 }; return m; }
#endif
