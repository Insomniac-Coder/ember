/* The C side of the FFI-8 tests of `volatile T*`: a register C reads and writes, and a struct
   holding one. */
#ifndef REGISTERS_H
#define REGISTERS_H
#include <stdint.h>
struct Device { volatile uint32_t* status; uint32_t mask; };
static inline uint32_t register_read(const volatile uint32_t* reg) { return *reg; }
static inline void register_write(volatile uint32_t* reg, uint32_t value) { *reg = value; }
static inline uint32_t device_masked(struct Device device) { return *device.status & device.mask; }
#endif
