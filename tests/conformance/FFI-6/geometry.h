/* The C side of accept_a_header_and_its_c_source.em ([FFI-6], [BLD-FFI-1]). */
#ifndef GEOMETRY_H
#define GEOMETRY_H
#include <stddef.h>
#include <stdint.h>

#define GEOMETRY_MAX 16
#define GEOMETRY_UNITS "mm"

typedef struct Point {
    int32_t x;
    int32_t y;
} Point;

typedef struct {
    Point min;
    Point max;
    uint8_t tag[4];
} Rect;

typedef enum Shape { SHAPE_DOT = 1, SHAPE_LINE = 2, SHAPE_AREA = 4, SHAPE_FIRST = SHAPE_DOT } Shape;
enum { GEOMETRY_FLAG_A = 1, GEOMETRY_FLAG_B = 2 };

struct Polygon;
typedef double scalar;

extern int geometry_calls;
extern const int geometry_version;

int geometry_area(Rect r);
Point geometry_mid(const Point* a, const Point* b);
void geometry_grow(Rect* r, int by);
size_t geometry_count_tagged(const Rect* rects, size_t n);
Shape geometry_shape(int n);
scalar geometry_scale(scalar value);
struct Polygon* geometry_polygon(int sides);
int geometry_sides(const struct Polygon* polygon);
void geometry_polygon_free(struct Polygon* polygon);
int type(int match);
#endif
