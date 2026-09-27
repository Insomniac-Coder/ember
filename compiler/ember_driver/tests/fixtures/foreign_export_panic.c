#include <stdint.h>

extern int32_t host_panic(void);

int32_t call_exported_panic(void) {
    return host_panic();
}
