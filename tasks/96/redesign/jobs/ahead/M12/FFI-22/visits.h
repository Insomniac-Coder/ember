/* The C side of the FFI-22 test of a borrowed callback with no thread word: declarations only. */
#ifndef VISITS_H
#define VISITS_H

typedef void (*visit_fn)(void* data, int index);

/* Calls `visit(data, i)` for each `i` below `count`, perhaps from several threads at once. */
void visit_each(int count, visit_fn visit, void* data);
#endif
