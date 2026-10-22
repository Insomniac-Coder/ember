#include "newtypes.h"
#include <stdlib.h>

struct widget {
    int size;
};

fd_t open_slot(int number) {
    return number + 100;
}

int slot_number(fd_t fd) {
    return fd - 100;
}

seconds_t later(seconds_t start, double by) {
    return start + by;
}

widget_ref make_widget(int size) {
    widget_ref widget = malloc(sizeof *widget);
    if (widget != NULL) {
        widget->size = size;
    }
    return widget;
}

int widget_size(widget_ref widget) {
    return widget->size;
}

void free_widget(widget_ref widget) {
    free(widget);
}

count_t twice(count_t n) {
    return n * 2;
}
