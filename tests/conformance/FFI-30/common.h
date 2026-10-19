/* The C side of the FFI-30 tests: declarations two headers share. */
#ifndef COMMON_H
#define COMMON_H
typedef struct Vec2 {
    float x;
    float y;
#ifdef COMMON_WIDE
    float z;
#endif
} Vec2;
typedef enum Mode { MODE_A = 1, MODE_B = 2 } Mode;
struct Handle;
#endif
