/* The C side of the FFI-20a tests: declarations Ember cannot represent yet, beside one it can. */
#ifndef UNREPRESENTABLE_H
#define UNREPRESENTABLE_H

union Packed { unsigned low : 4; int whole; };
static int unrepresentable_hidden = 4;

static inline int unrepresentable_log(const char* format, ...) { return format[0]; }
int unrepresentable_legacy();
int unrepresentable_number(union Packed n);
int unrepresentable_packed_low(const union Packed* packed);

static inline int unrepresentable_fine(int x) { return x + unrepresentable_hidden; }
#endif
