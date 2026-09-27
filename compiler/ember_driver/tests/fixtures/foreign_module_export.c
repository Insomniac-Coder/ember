#include <stdint.h>

extern int32_t module_score(int32_t value);

int32_t call_module_score(int32_t value) {
    return module_score(value);
}
