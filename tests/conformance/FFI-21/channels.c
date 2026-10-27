#include "channels.h"
#include <stddef.h>

#define CHANNELS 4
static tick_fn hooks[CHANNELS];
static void* datas[CHANNELS];
static int ids[CHANNELS];
static int last_removed = -1;
static int next_id = 7;

int subscribe_on(int channel, tick_fn hook, void* data) {
    if (hook == NULL) {
        last_removed = data == NULL ? -2 : -3;
        return 0;
    }
    hooks[channel] = hook;
    datas[channel] = data;
    ids[channel] = next_id;
    return next_id++;
}

void unsubscribe_on(int channel, int id) {
    if (ids[channel] == id) {
        hooks[channel] = NULL;
        datas[channel] = NULL;
    }
    last_removed = channel * 100 + id;
}

int tick(int channel, int value) {
    return hooks[channel] != NULL ? hooks[channel](datas[channel], value) : -1;
}

int removed(void) {
    return last_removed;
}
