#$ test: run-pass
#$ rules: TXT-8, LT-1
#$ stdout: fixed hello hello
# `[TXT-8]` — a `str`'s region is what it views: a literal is static and may be returned from
# anywhere; a `str` of a parameter may be returned to the caller that owns the text, alone or
# inside a struct holding it.

struct Holder:
    view: str

fn label() -> str:
    return "fixed"

fn same(text: str) -> str:
    return text

fn wrap(text: str) -> Holder:
    return Holder(text)

fn main():
    greeting: String = "hello"
    println(label(), same(greeting.as_str()), wrap(greeting.as_str()).view)
