#$ test: compile-fail
#$ rules: SPN-2, UNS-1
## "`unsafe: s.get_unchecked(i)`" — the third form, and the only one that
## needs a boundary.

fn peek(xs: Span[i32]) -> i32:
    return xs.get_unchecked(0)     #$ error[E3100]: `get_unchecked` needs an `unsafe` block
fn main(): pass
