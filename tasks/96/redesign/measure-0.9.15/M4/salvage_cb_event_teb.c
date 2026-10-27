/* salvage_cb_event_teb.c: the model with the creator compared as the OS thread id that the thread's
   own environment block holds (Windows x64: gs:[0x48], what GetCurrentThreadId reads), standing for
   both the attach check and the thread-local number: one load from gs and one compare, no TLS index
   lookup (a `__declspec(thread)` variable in an executable costs four loads: `_tls_index`, gs:[0x58],
   the module's TLS block, the variable).

   Safety needs one more runtime duty: an OS thread id is unique only among live threads, so when a
   thread ends or detaches, the runtime ends every `threads(creator)` slot that thread created (its TLS
   callback / DLL_THREAD_DETACH; detaching does it too). Those slots could never run legally anyway (their
   creator is gone), and ending them means a later thread that reuses the id finds no slot that names
   it. With that, an equal id proves this is the creator, and the creator, which registered from Ember
   code, is attached. Not covered: a thread killed by TerminateThread, which runs no callback, so its
   slots stay; the program must not use TerminateThread on a thread that registered callbacks (Windows
   documents it as leaving process state inconsistent). On Linux the same idea uses the thread
   pointer (fs:0), unique among live threads, with the same end-of-thread duty. */
#define TEB_THREAD 1
#define NAME "salvage_cb_event_teb"
#include "model_cb_event.c"
