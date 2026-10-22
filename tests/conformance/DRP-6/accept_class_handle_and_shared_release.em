#$ test: run-pass
#$ rules: DRP-6, CLS-1, HEAP-3, HEAP-4, OBJ-3
#$ profiles: debug, development, release
#$ stdout: shared
#$ stdout: handle
#$ assert-c: contains(&em_ti_Handle))
#$ assert-c: contains(ember_obj_new_copy)
#$ assert-c: contains(ember_release((ember_obj_header*)

# `[DRP-6]` distinguishes the two counted owners from the existing Box probe.
# The copied Shared value releases its payload only at the final strong release;
# the class handle then releases its own object at scope exit. (The object is the runtime's to
# make, `ember_obj_new`, or in `main`'s frame, `ember_obj_new_frame`, as `[OPT-1]` permits: either
# way its last release, at scope exit, runs its `drop`.)

class Handle:
    fn drop(mut self):
        println("handle")

struct Token:
    fn drop(mut self):
        println("shared")

fn main():
    _handle = Handle()
    owner = Shared(Token())
    _alias = owner
