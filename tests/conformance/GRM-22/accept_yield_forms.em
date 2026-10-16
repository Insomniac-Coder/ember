#$ test: parse-pass
#$ rules: GRM-22
## `yield` occupies `return`'s position and precedence: `yield e`, and a bare
## `yield`, parse wherever a whole expression may stand. Whether it may stand
## there is the checker's (`[GRM-16]`: `v = yield 1` is `E0107`, a `yield`
## being of type `void`).

gen fn f() -> Generator[int]:
    yield
    v = yield 1
    print(v)
