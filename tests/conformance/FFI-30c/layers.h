/* The C side of the FFI-30c tests: one header, several overlays. */
#ifndef LAYERS_H
#define LAYERS_H
typedef struct { int w; int h; } Size;
static inline int size_area(Size s) { return s.w * s.h; }
static inline int layers_scale(int value, int by) { return value * by; }
static inline int layers_add(int a, int b) { return a + b; }
#define LAYERS_TWICE(x) ((x) * 2)
#endif
