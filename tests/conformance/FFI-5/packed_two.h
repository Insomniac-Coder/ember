/* The C side of reject_a_layout_ember_cannot_write.em: packed to 2, which Ember cannot write. */
#ifndef PACKED_TWO_H
#define PACKED_TWO_H
#include <stdint.h>
#pragma pack(push, 2)
typedef struct Header { uint8_t tag; uint32_t size; } Header;
#pragma pack(pop)
#endif
