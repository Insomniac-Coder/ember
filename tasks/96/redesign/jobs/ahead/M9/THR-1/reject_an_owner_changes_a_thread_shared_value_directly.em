# Clause THR-1.2: "**no owner changes a `thread_shared` value directly** (`E7014`"
#$ test: compile-fail
#$ rules: THR-1, THR-7
#$ not-help: thread-safe
# The spec's message, with this file's line numbers. `[THR-7]`: no diagnostic calls `thread_shared`
# "thread-safe".

class Names:
    items: Array[String] = []

fn fill(s: String):
    names = thread_shared Names()
    names.items.push(owns copy of s)    #$ error[E7014]: `names` is shared between threads (line 12), so no owner changes it directly, and line 13 pushes to its list; fill the list where the object is made: write `names = thread_shared Names(items=[copy of s])` instead of lines 12 and 13
    println(names.items.len())

fn main():
    fill(String("a"))
