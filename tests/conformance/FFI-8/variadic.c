/* The definitions of variadic.h. */
#include "variadic.h"
#include <stdio.h>

int64_t sum_wide(int count, ...) {
    va_list arguments;
    va_start(arguments, count);
    int64_t sum = 0;
    for (int i = 0; i < count; i++) sum += va_arg(arguments, int64_t);
    va_end(arguments);
    return sum;
}

int format_into(char* buffer, size_t size, const char* format, ...) {
    va_list arguments;
    va_start(arguments, format);
    int written = vsnprintf(buffer, size, format, arguments);
    va_end(arguments);
    return written;
}

int format_list(char* buffer, size_t size, const char* format, va_list arguments) {
    return vsnprintf(buffer, size, format, arguments);
}

void log_through(log_sink sink, void* context, const char* format, ...) {
    va_list arguments;
    va_start(arguments, format);
    sink(context, format, arguments);
    va_end(arguments);
}
