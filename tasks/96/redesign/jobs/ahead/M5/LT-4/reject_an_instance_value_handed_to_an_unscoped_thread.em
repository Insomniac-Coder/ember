# Clause LT-4.2: "An unscoped thread, a job outside a scope and a callback foreign code keeps may
# outlive every instance but a `static` (...), so a value made in an instance, a `thread_shared` one
# included (`[TYP-46a]`), goes to one of them only when its instance is a `static` (...); from a
# local, a parameter, a field or a `static mut` (...) it is `E3063` at the hand-over ("`st` was made
# in `level` (line 7), ...")."
#$ test: compile-fail
#$ rules: LT-4, TYP-46a, THR-10
# The spec's message, with the file's line numbers. `t.join()` changes nothing: an unscoped thread
# may outlive `start` by its kind.
import std.thread

class Stats:
    hits: int = 0

fn start():
    level = Arena.with_capacity(1024)
    st = thread_shared Stats() using level
    t = thread.spawn(owns fn() owns st: println(st.hits))    #$ error[E3063]: `st` was made in `level` (line 17), which ends when `start` returns, and the thread started on line 18 may outlive `start`; make it in the built-in manager, `st = thread_shared Stats()`, in a `static` manager whose `allocate` reads it (a pool that locks inside), or run the thread as a task of a scope that ends before `level`: `with scope = thread.scope():` and `scope.spawn(owns fn() owns st: ...)` in place of line 18
    t.join()

fn main():
    start()
