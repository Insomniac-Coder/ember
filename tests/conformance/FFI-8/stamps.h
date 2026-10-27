/* The C side of FFI-8's newtype-in-a-struct test: structs whose members are typedefs an overlay
   makes nominal, passed and returned by value. */
#ifndef STAMPS_H
#define STAMPS_H

typedef double when_t;
typedef int tag_t;

/* Two doubles: in two SSE registers on System V, in memory on Win64 (16 bytes). */
typedef struct {
    when_t start;
    when_t end;
} span_t;

/* A double and an int: SSE and integer registers on System V, in memory on Win64. */
typedef struct {
    when_t at;
    tag_t tag;
} stamp_t;

/* One double: an SSE register on System V, an integer register on Win64 (8 bytes). */
typedef struct {
    when_t only;
} lone_t;

double span_length(span_t s);
span_t span_shift(span_t s, when_t by);
int stamp_tag(stamp_t s);
stamp_t stamp_make(when_t at, tag_t tag);
when_t lone_value(lone_t l);
lone_t lone_twice(lone_t l);
#endif
