/* The C side of the FFI-6b tests: function-like macros, which an overlay entry declares. */
#ifndef MACROS_H
#define MACROS_H
typedef struct { int x; int y; } Point;
struct Pair { long long first; long long second; };
typedef struct Hidden Hidden;
static inline int twice(int x) { return 2 * x; }
#define twice(x) (2 * (x))
#define SQUARE(x) ((x) * (x))
#define SCALE(value, by) ((value) * (by))
#define POINT_SUM(p) ((p).x + (p).y)
#define PAIR_FIRST(p) ((p)->first)
#define IS_EVEN(n) (((n) % 2) == 0)
#define HIDDEN_ID(h) 0
#define LOG_ALL(...) 0
#endif
