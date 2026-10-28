/* salvage_cb_parse_fold.c: the model with the attach check folded into the thread compare.

   Idea: a `threads(creator)` trampoline compares the calling thread's number (a thread-local the
   runtime sets when the thread attaches) with the slot's creator. If they are equal, this thread is
   the creator, and it is attached, provided (1) detaching clears the thread's number, and (2) the
   runtime ending (which bumps the global generation [FFI-22] compares) ends every slot first. Then the
   separate attach compare (a thread-local load, a global load, a compare) goes; a mismatch takes the
   cold path, which attaches and compares again before stopping the program.

   Safety: a call that passes the fold passes both checks it replaces; a call that fails it is
   attached and checked again exactly as the model checks it. Run 3 measured this form as `ffi-2`
   (artifacts/redesign2/threads/bench/ffi.c model 2, 1.03x) on cb_event. */
#define FOLD_ATTACH 1
#define NAME "salvage_cb_parse_fold"
#include "model_cb_parse.c"
