# Clause CLO-6a.1: "A call of one held in a class object's field, in an `owns` block (`[EXP-6]`), or
# reached through a `shares`, `shares mut`, `thread_shared`, `weakref` or `sharedref mut` link or a
# view, is `E3013`"
#$ test: compile-fail
#$ rules: CLO-6a, CLO-6, EXP-6
# The spec's message.

fn consume(owns s: String):
    println(s)

class Job:
    task: once fn()

    fn init(owns task: once fn()):
        this.task = task

fn main():
    s = String("a")
    j = Job(fn() owns s: consume(owns s))
    j.task()    #$ error[E3013]: `j.task` is a field of a `Job` object, which other names may reach and would find empty; take the function out first: `t = mem.replace(ref mut j.task, fn(): pass)`, then `t()`
