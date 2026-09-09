#$ test: parse-pass
#$ rules: GRM-22
## `yield` occupies `return`'s position and precedence, but its type is the
## coroutine's resume type, so it may be bound. A bare `yield` is `yield ()`.

gen fn f() -> Coroutine[void]:
    yield
    v = yield 1
    print(v)
