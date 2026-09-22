#$ test: run-pass
#$ rules: RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1
#$ stdout: 7

# A normal closure captures the loop's class-handle yield as a reference. An
# ordinary inherent method call must auto-dereference that receiver without
# retaining the borrowed handle.
class Token:
    value: i32

    fn read(self) -> i32:
        return self.value

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        task = fn() -> i32:
            return token.read()
        println(task())
