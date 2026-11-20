#$ test: compile-fail
#$ rules: ALC-2, UNS-1
## `[ALC-2]` — giving memory back, or moving it, is unsafe, since Ember may still use it and a wrong
## `Layout` is undefined in C: a program's own call of `deallocate` or `reallocate`, through a bound
## or on a manager it names, is made inside `unsafe:` (`E3100`). `allocate` is safe.

from std.mem import Allocator, Layout, Global

[A: needs Allocator]
fn give_back(a: A, p: *mut u8):
    a.deallocate(p, Layout(16, 8))    #$ error[E3100]: `deallocate` gives memory back that Ember may still use, and a wrong `Layout` is undefined in C; write it inside `unsafe:` with a `# SAFETY:` note

fn main():
    p = Global().allocate(Layout(16, 8))
    q = Global().reallocate(p, Layout(16, 8), 32)    #$ error[E3100]: `reallocate` moves memory that Ember may still use, and a wrong `Layout` is undefined in C; write it inside `unsafe:` with a `# SAFETY:` note
    give_back(Global(), q)
