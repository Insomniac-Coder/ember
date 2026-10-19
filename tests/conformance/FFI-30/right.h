/* The other header sharing common.h. */
#include "common.h"
static inline float right_length2(Vec2 v) { return v.x * v.x + v.y * v.y; }
static inline int right_mode_value(Mode m) { return (int)m; }
static inline int right_is_null(const struct Handle* h) { return h == 0; }
