#include <stdlib.h>
#include "packets.h"
struct Packet* packet_new(uint8_t count) {
    struct Packet* packet = malloc(sizeof *packet + count * sizeof packet->values[0]);
    packet->kind = 7;
    packet->count = count;
    for (uint8_t i = 0; i < count; i++) packet->values[i] = (int64_t)(i + 1) * 10;
    return packet;
}
void packet_free(struct Packet* packet) { free(packet); }
int64_t packet_sum(const struct Packet* packet) {
    int64_t total = 0;
    for (uint8_t i = 0; i < packet->count; i++) total += packet->values[i];
    return total;
}
