/* The C side of the FFI-20a tests: declarations Ember cannot represent yet, beside one it can. */
#ifndef UNREPRESENTABLE_H
#define UNREPRESENTABLE_H

union Number { int i; float f; };
enum Mode { MODE_OFF, MODE_ON };
struct Flags { enum Mode mode : 2; unsigned count : 6; };
static int unrepresentable_hidden = 4;

int unrepresentable_log(const char* format, ...);
long double unrepresentable_precise(void);
int unrepresentable_number(union Number n);
int unrepresentable_flags_ready(const struct Flags* flags);

static inline int unrepresentable_fine(int x) { return x + unrepresentable_hidden; }
#endif
