/* The C side of the FFI-8 newtype tests (ODR-249): typedefs an overlay makes nominal, and one it
   leaves an alias. */
#ifndef NEWTYPES_H
#define NEWTYPES_H

typedef int fd_t;
typedef double seconds_t;
struct widget;
typedef struct widget* widget_ref;
typedef int count_t;

/* A slot's descriptor is its number plus 100. */
fd_t open_slot(int number);
int slot_number(fd_t fd);

/* `seconds_t` is passed as C passes a `double` (in a floating-point register on Win64 and
   System V), which a one-field struct is not. */
seconds_t later(seconds_t start, double by);

/* A widget held by its typedef'd pointer. */
widget_ref make_widget(int size);
int widget_size(widget_ref widget);
void free_widget(widget_ref widget);

count_t twice(count_t n);
#endif
