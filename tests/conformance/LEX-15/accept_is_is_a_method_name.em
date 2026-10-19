#$ test: run-pass
#$ rules: LEX-15, EXP-9, EXP-10
#$ profiles: debug, development
#$ stdout: true false true
#$ Some(true) None
#$ 7
# ODR-106 — `is` is a method name after `fn` in a type's body, after `.` and after `?.`, as `not`
# is (ODR-040), and the identity operator and the `Option` test everywhere else. A function standing
# alone writes the name raw.

class Node:
    n: int

struct Flag:
    on: bool
    fn is(self) -> bool:
        return self.on

fn r#is() -> int:
    return 7

fn main():
    a = Node(1)
    b = a
    flag = Flag(true)
    maybe: Option[Flag] = Some(Flag(false))
    nothing: Option[Flag] = None
    println(flag.is(), maybe?.is() == Some(true), a is b and maybe is not None)
    println(Some(flag)?.is(), nothing?.is())
    println(r#is())
