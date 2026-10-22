/* The C side of the FFI-8 tests of variadic functions and `va_list` (defined in variadic.c). */
#ifndef VARIADIC_H
#define VARIADIC_H
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
/* The sum of `count` further `int64_t` arguments. */
int64_t sum_wide(int count, ...);
/* `snprintf` and `vsnprintf` by other names. */
int format_into(char* buffer, size_t size, const char* format, ...);
int format_list(char* buffer, size_t size, const char* format, va_list arguments);
/* A sink a logger hands its format and the arguments it was given. */
typedef void (*log_sink)(void* context, const char* format, va_list arguments);
void log_through(log_sink sink, void* context, const char* format, ...);
#endif
