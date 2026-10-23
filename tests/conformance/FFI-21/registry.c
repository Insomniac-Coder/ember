#include "registry.h"
#include <stddef.h>

int each_pair(int count, pair_fn visit, intptr_t context) {
    int sum = 0;
    for (int index = 0; index < count; ++index) {
        sum += visit((intptr_t)index * 10, context, index);
    }
    return sum;
}

static int marker;

int both(swap_fn pick, void* right) {
    return pick(&marker, right);
}

static int last_given = 0;

int maybe_call(hook_fn hook, void* data, int value) {
    last_given = hook != NULL && data != NULL ? 1 : hook == NULL && data == NULL ? 0 : -2;
    return hook != NULL ? hook(data, value) : -1;
}

int given(void) {
    return last_given;
}

#define SLOTS 8
static hook_fn kept_hooks[SLOTS];
static void* kept_data[SLOTS];

int subscribe(hook_fn hook, void* data) {
    if (hook == NULL) {
        last_given = data == NULL ? 0 : -2;
        return 0;
    }
    for (int slot = 0; slot < SLOTS; ++slot) {
        if (kept_hooks[slot] == NULL) {
            kept_hooks[slot] = hook;
            kept_data[slot] = data;
            return slot + 100;
        }
    }
    return 0;
}

void unsubscribe(int id) {
    int slot = id - 100;
    if (slot >= 0 && slot < SLOTS) {
        kept_hooks[slot] = NULL;
        kept_data[slot] = NULL;
    }
}

int fire(int value) {
    int sum = 0;
    for (int slot = 0; slot < SLOTS; ++slot) {
        if (kept_hooks[slot] != NULL) {
            sum += kept_hooks[slot](kept_data[slot], value);
        }
    }
    return sum;
}

int subscribers(void) {
    int count = 0;
    for (int slot = 0; slot < SLOTS; ++slot) {
        count += kept_hooks[slot] != NULL;
    }
    return count;
}

static hook_fn deferred_hook;
static void* deferred_data;

void defer(hook_fn hook, void* data) {
    last_given = hook != NULL && data != NULL ? 1 : hook == NULL && data == NULL ? 0 : -2;
    if (hook != NULL) {
        deferred_hook = hook;
        deferred_data = data;
    }
}

int run_deferred(int value) {
    if (deferred_hook == NULL) {
        return -1;
    }
    hook_fn hook = deferred_hook;
    deferred_hook = NULL;
    return hook(deferred_data, value);
}
