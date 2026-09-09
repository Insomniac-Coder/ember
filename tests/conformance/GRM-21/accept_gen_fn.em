#$ test: parse-pass
#$ rules: GRM-21, CORO-1
## `gen_fn := "gen" fn_decl`, admitted wherever `fn_decl` is.

gen fn ticks() -> Coroutine[void]:
    yield 1
    yield 2
