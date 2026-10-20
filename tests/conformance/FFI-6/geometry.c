/* Compiled by `import c "./geometry.c"` ([BLD-FFI-1]). */
#include "geometry.h"
#include <stdlib.h>

struct Polygon { int sides; };

int geometry_calls = 0;
const int geometry_version = 3;

int geometry_area(Rect r) { geometry_calls++; return (r.max.x - r.min.x) * (r.max.y - r.min.y); }
Point geometry_mid(const Point* a, const Point* b) { Point p = { (a->x + b->x) / 2, (a->y + b->y) / 2 }; return p; }
void geometry_grow(Rect* r, int by) { r->min.x -= by; r->min.y -= by; r->max.x += by; r->max.y += by; r->tag[0] = 1; }
size_t geometry_count_tagged(const Rect* rects, size_t n) {
    size_t count = 0;
    for (size_t i = 0; i < n; i++) if (rects[i].tag[0]) count++;
    return count;
}
Shape geometry_shape(int n) { return (Shape)n; }
scalar geometry_scale(scalar value) { return value * 2.5; }
struct Polygon* geometry_polygon(int sides) { struct Polygon* p = malloc(sizeof *p); p->sides = sides; return p; }
int geometry_sides(const struct Polygon* polygon) { return polygon->sides; }
void geometry_polygon_free(struct Polygon* polygon) { free(polygon); }
int type(int match) { return match + 1; }
