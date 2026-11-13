# Clause LT-4.2 (its ways out, `[DIA-25]`): "make it in the built-in manager, `st = thread_shared
# Stats()`, ... or run the thread as a task of a scope that ends before `level`: `with scope =
# thread.scope():` and `scope.spawn(owns fn() owns st: ...)`"
#$ test: run-pass
#$ rules: LT-4, TYP-46a, THR-10, THR-5
#$ stdout: 0
#$ stdout: 0
import std.thread

class Stats:
    hits: int = 0

fn built_in():
    st = thread_shared Stats()
    t = thread.spawn(owns fn() owns st: println(st.hits))
    t.join()

fn scoped():
    level = Arena.with_capacity(1024)
    st = thread_shared Stats() using level
    with scope = thread.scope():
        scope.spawn(owns fn() owns st: println(st.hits))

fn main():
    built_in()
    scoped()
