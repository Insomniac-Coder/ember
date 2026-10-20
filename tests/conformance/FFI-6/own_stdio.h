/* The C side of accept_functions_the_runtime_header_also_declares.em: `puts` and `strlen`, which
   <stdio.h> and <string.h> declare too, and the generated C includes both. */
#ifndef OWN_STDIO_H
#define OWN_STDIO_H
#include <stddef.h>
int puts(const char* s);
size_t strlen(const char* s);
#endif
