/* `[FFI-36a]`, `[FFI-36b]`, `[FFI-33b]` — foreign objects a test program adopts: each knows its
 * id, and every free adds what freed it to a total the program reads back. */
#include <stdint.h>
#include <stdlib.h>

typedef struct Widget { int32_t id; } Widget;
typedef struct Gadget { int32_t id; } Gadget;

static int32_t g_live;
static int32_t g_freed;

Widget* widget_new(int32_t id) {
    Widget* widget = (Widget*)malloc(sizeof *widget);
    widget->id = id;
    g_live += 1;
    return widget;
}

void widget_free(Widget* widget) {
    g_freed += widget->id;
    g_live -= 1;
    free(widget);
}

int32_t widget_id(Widget* widget) { return widget->id; }

Gadget* gadget_new(int32_t id) {
    Gadget* gadget = (Gadget*)malloc(sizeof *gadget);
    gadget->id = id;
    g_live += 1;
    return gadget;
}

/* A destructor with a status, as C's often have: the box drops it. */
int32_t gadget_close(Gadget* gadget) {
    g_freed += gadget->id * 100;
    g_live -= 1;
    free(gadget);
    return 0;
}

/* `returns_owned`: a new widget, or none for a negative id. */
Widget* widget_maybe(int32_t id) { return id < 0 ? NULL : widget_new(id); }

/* `owned`: C takes the widget and frees it itself, adding ten times its id. */
void widget_take(Widget* widget) {
    if (widget == NULL) {
        g_freed += 1000;
        return;
    }
    g_freed += widget->id * 10;
    g_live -= 1;
    free(widget);
}

int32_t objects_live(void) { return g_live; }
int32_t objects_freed(void) { return g_freed; }
