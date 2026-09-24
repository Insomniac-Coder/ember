#$ test: parse-fail
#$ rules: LT-6, LEX-22
# `[LT-6]` — named lifetimes are not part of Ember and never will be;
# `[LEX-22]` — a `'` begins a character literal and nothing else, so
# `'a` with no closing quote is an unterminated literal (`E0008`), whose note
# names what to write instead.

fn longest['a](x: str) -> str:     #$ error[E0008]: unterminated character literal
    return x

fn main():
    println(1)
