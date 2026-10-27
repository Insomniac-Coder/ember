#include "wide_sum.h"
#include <stdarg.h>

int64_t sum_wide(int count, ...) {
    va_list arguments;
    va_start(arguments, count);
    int64_t sum = 0;
    for (int i = 0; i < count; i++) sum += va_arg(arguments, int64_t);
    va_end(arguments);
    return sum;
}
