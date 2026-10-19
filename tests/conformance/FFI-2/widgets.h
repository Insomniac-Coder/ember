/* The C side of the FFI-2 tests: a small library whose header states no contract. */
#ifndef WIDGETS_H
#define WIDGETS_H
#include <stddef.h>

typedef struct Widget Widget;
typedef struct Size { int w; int h; } Size;

Widget* widget_new(int id);
void widget_free(Widget* widget);
int widget_id(const Widget* widget);
int widget_sum(const int* items, size_t n);
void widget_fill(int* items, size_t n, int value);
const char* widget_label(int id);
size_t widget_length(const char* text);
int widget_add(int a, int b);
void widget_double(int* value);
int widget_freed(void);
#endif
