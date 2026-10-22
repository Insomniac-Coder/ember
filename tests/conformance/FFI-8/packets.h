/* The C side of the FFI-8 tests of a flexible array member: a struct its elements follow, made
   and freed by C. */
#ifndef PACKETS_H
#define PACKETS_H
#include <stdint.h>
struct Packet { uint16_t kind; uint8_t count; int64_t values[]; };
struct Packet* packet_new(uint8_t count);
void packet_free(struct Packet* packet);
int64_t packet_sum(const struct Packet* packet);
#endif
