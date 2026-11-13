#$ test: compile-fail
#$ rules: UNS-2
# `[UNS-2]` — `unsafe` does not turn off the checks of Part VIII: inside an `unsafe:` block in
# `drop`, storing `this` where it outlives the call is still `E3016` (`[OBJ-5]`).

class Token:
    name: String

    fn init(owns name: String):
        this.name = name

    fn drop():
        n = this.name.len()
        # SAFETY: one slot, written once and freed here
        unsafe:    #$ warning[L3010]: larger than necessary
            p = alloc[usize](1)
            write(p, 0, n)
            free(p, 1)
            retained: Array[Token] = Array[Token]()
            retained.push(owns this)    #$ error[E3016]: `drop` ends this `Token`; an owner made from it would point at a freed object

fn main():
    _token = Token("a")
