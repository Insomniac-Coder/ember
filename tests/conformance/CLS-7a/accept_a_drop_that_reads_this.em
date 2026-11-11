#$ test: run-pass
#$ rules: CLS-7a
#$ stdout: 5
# `[CLS-7a]` — inside `drop`, `this` is read; it is only never stored anywhere that outlives the call.

class Token:
    id: int

    fn init(id: int):
        this.id = id

    fn drop():
        println(this.id)

fn main():
    _token = Token(5)
