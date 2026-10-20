/* Compiled by `import c "./widgets.c"` in the FFI-2 tests. */
#include "widgets.h"
#include <stdlib.h>
#include <string.h>

struct Widget { int id; };
static int freed = 0;

Widget* widget_new(int id) { Widget* w = malloc(sizeof *w); w->id = id; return w; }
void widget_free(Widget* widget) { freed++; free(widget); }
void widget_take(Widget* widget) { widget_free(widget); }
int widget_id(const Widget* widget) { return widget->id; }
int widget_sum(const int* items, size_t n) { int total = 0; for (size_t i = 0; i < n; i++) total += items[i]; return total; }
void widget_fill(int* items, size_t n, int value) { for (size_t i = 0; i < n; i++) items[i] = value; }
const char* widget_label(int id) { return id > 0 ? "positive" : "other"; }
size_t widget_length(const char* text) { return strlen(text); }
int widget_add(int a, int b) { return a + b; }
void widget_double(int* value) { *value *= 2; }
int widget_freed(void) { return freed; }
