/* The C side of the FFI-1 tests: functions whose declarations state their effects. */
#ifndef MATHS_H
#define MATHS_H

#if defined(_MSC_VER) && !defined(__clang__)
#define __attribute__(x)
#endif

/* No effects, and the same result for the same argument. */
int square(int x) __attribute__((const));
/* No effects: it only reads memory (here, the table of offsets). */
int offset(int index) __attribute__((pure));
/* Its declaration states nothing. */
int counter(void);
#endif
