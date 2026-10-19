/* One of two headers sharing common.h. */
#include "common.h"
static inline Vec2 left_make(float x, float y) { Vec2 v = { x, y }; return v; }
static inline Mode left_mode(int n) { return n == 2 ? MODE_B : MODE_A; }
static inline struct Handle* left_handle(void) { return (struct Handle*)0; }
