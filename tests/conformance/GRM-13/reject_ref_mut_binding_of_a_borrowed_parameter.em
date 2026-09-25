#$ test: compile-fail
#$ rules: GRM-13, FN-1
# D-295 — `ref mut s` in a pattern borrows the matched place mutably, the
# same write as `ref mut o`: a borrowed parameter cannot be written through
# it. The binding used to skip every write check.

fn shout(o: Option[String]):
    match o:    #$ error[E3023]: cannot mutate borrowed parameter `o`
        Some(ref mut s):
            s += "!"
        None:
            pass

fn main():
    a: Option[String] = Some("hi")
    shout(a)
    println(a)
