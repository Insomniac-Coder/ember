#$ test: run-pass
#$ rules: TYP-15, LT-3
# Owner decision, ERR-044. `[TYP-15]`'s condition is on the **view's region**,
# not on the kind of place: a view may be stored where its region outlives the
# destination. A `static` has no bounding region, and `[LT-3]` gives a string
# literal the `static` region, which outlives everything — so the condition is
# met rather than waived.
#
# This was rejected, because `[TYP-15]` used to enumerate `static`s as "always
# forbidden" while `[LT-3]` said a literal may be stored in a class field. Both
# were normative and `[TYP-15]`'s own principle sided with `[LT-3]`.

static GREETING: str = "hi"

fn pick() -> str:
    return GREETING

fn main():
    println(pick())
#$ stdout: hi
