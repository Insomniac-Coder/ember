/* The C side of the FFI-6a tests: object-like macros Clang can evaluate, and some it cannot. */
#ifndef CONSTANTS_H
#define CONSTANTS_H
#define CONSTANTS_COUNT 12
#define CONSTANTS_DOUBLED (CONSTANTS_COUNT * 2 + 1)
#define CONSTANTS_MASK 0xFFu
#define CONSTANTS_BIG (1ULL << 40)
#define CONSTANTS_NEGATIVE (-7)
#define CONSTANTS_LETTER 'A'
#define CONSTANTS_RATIO 0.25
#define CONSTANTS_HALF 0.5f
#define CONSTANTS_GREETING "hi\tthere"
#define CONSTANTS_JOINED "con" "cat"
#define CONSTANTS_SIZE sizeof(long long)
#define CONSTANTS_NIL ((void*)0)
#define CONSTANTS_INVALID ((void*)-1)
#define CONSTANTS_PAGE ((const int*)4096)
#endif
