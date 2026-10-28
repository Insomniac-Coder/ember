#$ test: compile-fail
#$ rules: BRW-18, CTL-2
## design rules: R15, R8
## R15: "a scoped task started inside a walk of what it changes is refused as any call that changes the
## walked list is (E3020: start it after the loop)"; R18 (verdict 73): E3020, not E7007, whose fix (a
## lock or an `Atomic`) would leave the walk refused (section 3's E3020 message, in this program's
## lines). The nearest allowed program is `accept_a_task_started_after_the_walk_changes_the_list.em`.

import std.thread

fn grow(ref mut names: Array[String]):
    with scope = thread.scope():
        for n in names:
            println(n)
            scope.spawn(fn(): names.push(String("x")))    #$ error[E3020]: `scope.spawn(...)` (line 14) starts a task that pushes to `names` (`fn(): names.push(String("x"))`), which this loop walks (line 12); start it after the loop: move line 14 below the loop

fn main():
    names = [String("a")]
    grow(ref mut names)
    println(names.len())
