#$ test: run-pass
#$ rules: LT-3, TYP-14
# A `str` literal has the `static` region, and a `@view struct` has a bounding
# region of its own, so storing one needs no exemption.
#
# The neighbouring case — a `str` in a `static` or a class field — is where
# `[TYP-15]` and `[LT-3]` contradict each other, and is ERR-044. Not tested
# either way here, because the document says both yes and no and a test would
# have to pick.

@view
struct Holder:
    pub name: str

fn main():
    h = Holder("hello")
    println(h.name)
#$ stdout: hello
