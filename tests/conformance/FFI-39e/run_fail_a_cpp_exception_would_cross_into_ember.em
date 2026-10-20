#$ test: run-fail
#$ rules: FFI-39e
#$ panics: a C++ exception would cross into Ember from Task::act: std::runtime_error: the task refused
# `[FFI-39e]` — a C++ exception never crosses into Ember code: the base implementation of a
# virtual, called from Ember, throws, and the program aborts at the boundary, naming the function
# and the exception.

import cpp "tasks.hpp" with (overlay = "overlays/tasks.em")

class Quiet(cpp.Task):
    pass

fn main():
    quiet = Quiet()
    quiet.act()
    println("not reached")
