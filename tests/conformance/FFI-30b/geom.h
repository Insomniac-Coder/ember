/* The C side of the FFI-30b tests. */
#ifndef GEOM_H
#define GEOM_H
typedef struct Point { int x; int y; } Point;
static inline int geom_sum(Point p) { return p.x + p.y; }
#endif
