#include "events.h"

static struct { done_fn done; void* data; int value; } pending[8];
static int pending_count;

void run_later(int value, done_fn done, void* data) {
    pending[pending_count].done = done;
    pending[pending_count].data = data;
    pending[pending_count].value = value;
    pending_count += 1;
}

int run_pending(void) {
    int ran = pending_count;
    for (int i = 0; i < pending_count; i++) pending[i].done(pending[i].data, pending[i].value * 2);
    pending_count = 0;
    return ran;
}

static struct { int kind; listener_fn listener; void* data; } listeners[8];
static int listener_count;

void listen(int kind, listener_fn listener, void* data) {
    listeners[listener_count].kind = kind;
    listeners[listener_count].listener = listener;
    listeners[listener_count].data = data;
    listener_count += 1;
}

void unlisten(int kind, listener_fn listener, void* data) {
    for (int i = 0; i < listener_count; i++) {
        if (listeners[i].kind == kind && listeners[i].listener == listener && listeners[i].data == data) {
            listeners[i] = listeners[listener_count - 1];
            listener_count -= 1;
            return;
        }
    }
}

int emit(int kind, int event) {
    int total = 0;
    for (int i = 0; i < listener_count; i++) {
        if (listeners[i].kind == kind) total += listeners[i].listener(event, listeners[i].data);
    }
    return total;
}

int listening(void) { return listener_count; }
