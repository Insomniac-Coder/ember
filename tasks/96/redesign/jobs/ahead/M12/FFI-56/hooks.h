/* The C side of the FFI-56 tests: function pointers C gives back, with no contract. */
#ifndef HOOKS_H
#define HOOKS_H

typedef void (*release_fn)(int handle);
typedef void (*log_fn)(const char* text);

release_fn get_release(void);
log_fn get_log(void);
#endif
