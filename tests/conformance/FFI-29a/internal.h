/* The C side of the FFI-29a tests: names with internal linkage. */
#ifndef INTERNAL_H
#define INTERNAL_H

/* A constant is imported by its value, not by its symbol. */
static const int INTERNAL_LIMIT = 9;

/* A mutable `static`: each C file has its own, which no binding may name. */
static int internal_counter = 5;

/* A `static` function is reached through the shim's wrapper. */
static int internal_bump(void) { return ++internal_counter; }
#endif
