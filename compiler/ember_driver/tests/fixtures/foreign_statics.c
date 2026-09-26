#include <stdint.h>

int32_t raw_counter = 41;
int32_t changing_counter = 7;
int32_t read_changing_counter(void) { return changing_counter; }
int32_t bump_changing_counter(void) { return ++changing_counter; }
const int32_t frozen_count = 1;
const int32_t aliased_count = 2;
