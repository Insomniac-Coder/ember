#$ test: run-pass
#$ rules: DRP-6, TYP-14
#$ profiles: debug, development, release
#$ stdout: 7

# Method lookup first reads through `ref Box[Token]`, then applies Box's
# specified payload auto-dereference for an ordinary inherent method.
struct Token:
    value: i32

    fn read() -> i32:
        return this.value

fn read_boxed(value: ref Box[Token]) -> i32:
    return value.read()

fn main():
    boxed = Box(Token(7))
    println(read_boxed(ref boxed))
