/* The C side of accept_packed_and_aligned_layouts_match_clangs.em. */
#ifndef LAYOUTS_H
#define LAYOUTS_H
#include <stddef.h>
#include <stdint.h>

#pragma pack(push, 1)
typedef struct Wire { uint8_t kind; uint32_t length; uint16_t port; } Wire;
#pragma pack(pop)

typedef struct Lane { _Alignas(16) float values[4]; } Lane;

typedef struct Mixed { uint8_t a; double b; uint16_t c; } Mixed;

static inline int layouts_wire_size(void) { return (int)sizeof(Wire); }
static inline int layouts_lane_align(void) { return (int)_Alignof(Lane); }
static inline int layouts_mixed_c(void) { return (int)offsetof(Mixed, c); }
#endif
