#$ test: run-pass
#$ rules: OWN-12, FFI-54, UNS-1
#$ profiles: debug, development, release
#$ stdout: 42 false
#$ stdout: end
#$ stdout: dropped 1
# `[OWN-12]`, `[FFI-54]` — `x.into_raw()`, inside `unsafe:`, gives the address of what an `owns T`
# holds (a `*mut T`) and stops Ember freeing it: the name ends, and nothing of it is dropped, so
# only `_kept` prints at the end. The compiler makes that step itself (there is no `mem.forget`).

struct Noisy:
    id: i64

    fn drop():
        println("dropped", this.id)

fn main():
    _kept: owns Noisy = owns Noisy(id=1)
    given: owns Noisy = owns Noisy(id=2)
    number: owns i64 = owns 40
    # SAFETY: neither address goes anywhere; both blocks are left to the end of the process
    unsafe:
        p = given.into_raw()
        q = number.into_raw()
        println(q.read() + 2, p.is_null())
    println("end")
