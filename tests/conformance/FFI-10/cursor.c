/* The C side of accept_globals_holding_pointers.em: globals holding pointers. */
#include "cursor.h"

static int slot = 7;
int* cursor = &slot;
const char* banner = "hi";

/* Declared by hand in the test, not in the header. */
int* foreign_cursor = 0;
const int* foreign_base = &slot;
