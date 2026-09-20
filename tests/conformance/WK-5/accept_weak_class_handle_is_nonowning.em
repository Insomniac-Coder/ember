#$ test: run-pass
#$ rules: WK-5, WK-6, WK-7, WK-11, WK-12, WK-14, OBJ-3, TST-14, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 42

# A class weak handle is a non-owning edge, so it does not add a strong edge
# to the `[WK-5]` ownership graph. Its live upgrade still yields an owning
# handle while the original class handle remains alive.
class Token:
    value: i32

fn main():
    strong = Token(42)
    weak: Weak[Token] = Weak(strong)
    match weak.upgrade():
        Some(owner):
            println(owner.value)
        None:
            println(0)
