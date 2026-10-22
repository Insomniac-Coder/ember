# The overlay of ../accept_variadic_calls_and_va_list.em: the formats are C strings, which a
# variadic function's call passes as they are.

unsafe overlay c "variadic.h":
    fn format_into(buffer, size, format: borrowed nul_terminated)
    fn log_through(sink, context, format: borrowed nul_terminated)
