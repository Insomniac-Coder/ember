/* plain_cb_once.c: workload cb_once, plain C. Completion callbacks, each called once: a C library
   queues requests (`lib_request(cb, ud)`) and later runs every queued callback once (`lib_pump()`),
   as an asynchronous I/O or job library does. The program makes 48,828 batches of 1,024 requests
   (49,999,872 callbacks); each request carries its own number, and its callback adds it to a total.

   What a careful C programmer writes: each request's state ({id}) comes from a free list of small
   blocks and goes back to it in the callback (no malloc per request). */
#include "cb_once_lib.h"

typedef struct Ctx { int64_t id; struct Ctx *next; } Ctx;
static Ctx *free_ctx;
static int64_t total;

static void on_done(void *ud) {
    Ctx *c = (Ctx *)ud;
    total += c->id;
    c->next = free_ctx; free_ctx = c;                           /* back to the free list */
}
static cb_t volatile the_cb = on_done;

int main(void) {
    h_start();
    Ctx *block = (Ctx *)calloc(BATCH, sizeof(Ctx));
    for (int i = 0; i < BATCH; i++) { block[i].next = free_ctx; free_ctx = &block[i]; }
    int64_t next_id = 0;
    for (int b = 0; b < BATCHES; b++) {
        cb_t cb = the_cb;
        for (int k = 0; k < BATCH; k++) {
            Ctx *c = free_ctx; free_ctx = c->next;
            c->id = next_id++;
            lib_request(cb, c);
        }
        lib_pump();
    }
    h_report("plain_cb_once", (uint64_t)total + (uint64_t)next_id);
    return 0;
}
