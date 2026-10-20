/* The C side of the FFI-29b tests: a single-header library, its implementation behind a macro. */
#ifndef SINGLE_H
#define SINGLE_H
int single_answer(void);
int single_calls(void);
#endif

#ifdef SINGLE_IMPLEMENTATION
static int single_count = 0;
int single_answer(void) { single_count++; return 42; }
int single_calls(void) { return single_count; }
#endif
