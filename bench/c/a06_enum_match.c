#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
typedef enum { CIRCLE, RECT, TRI } Kind;
typedef struct { Kind kind; double a, b; } Shape;
static double area(Shape s) {
    switch (s.kind) {
    case CIRCLE: return 3.0 * s.a * s.a;
    case RECT: return s.a * s.b;
    default: return 0.5 * s.a * s.b;
    }
}
int main(void) {
    size_t n = 1000000;
    Shape* shapes = (Shape*)malloc(n * sizeof *shapes);
    for (size_t i = 0; i < n; i++) {
        size_t k = i % 3;
        shapes[i] = k == 0 ? (Shape){ CIRCLE, 1.0, 0.0 } : k == 1 ? (Shape){ RECT, 2.0, 3.0 } : (Shape){ TRI, 4.0, 5.0 };
    }
    double total = 0.0;
    for (int round = 0; round < 100; round++)
        for (size_t i = 0; i < n; i++) total = total + area(shapes[i]);
    printf("%lld\n", (long long)total);
    return 0;
}
