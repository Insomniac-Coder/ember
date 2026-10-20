/* The C side of accept_bit_fields_are_read_and_written_by_accessors.em: bit-fields of several types,
   which MSVC and the System V ABI lay out differently, set and read by C and by Ember. */
#ifndef BITFIELDS_H
#define BITFIELDS_H
#include <stdbool.h>
#include <stddef.h>
struct Flags {
    unsigned char tag;
    unsigned int ready : 1;
    unsigned int level : 4;
    int delta : 5;
    bool on : 1;
    unsigned short count;
    unsigned int : 0;
    unsigned long long wide : 40;
};
static inline struct Flags flags_make(void) {
    struct Flags f = {0};
    f.tag = 7; f.ready = 1; f.level = 9; f.delta = -6; f.on = true; f.count = 300; f.wide = 0x123456789AULL;
    return f;
}
static inline unsigned flags_level(struct Flags f) { return f.level; }
static inline int flags_delta(struct Flags f) { return f.delta; }
static inline unsigned long long flags_wide(struct Flags f) { return f.wide; }
static inline size_t flags_size(void) { return sizeof(struct Flags); }
#endif
