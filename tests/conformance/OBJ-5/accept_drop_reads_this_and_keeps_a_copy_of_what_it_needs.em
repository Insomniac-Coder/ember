#$ test: run-pass
#$ rules: OBJ-5, CLS-7a
#$ stdout: ended a
# `[OBJ-5]`, `[CLS-7a]` — a `drop` reads its object and hands on a copy of what it needs, never the
# object itself.

class Token:
    name: String

    fn init(owns name: String):
        this.name = name

    fn drop():
        kept = copy of this.name
        println("ended", kept)

fn main():
    _token = Token(owns "a".to_string())
