#$ test: parse-pass
#$ rules: GRM-21, CORO-1
## `fn_header := ["extern" STRING] ["unsafe"] ["virtual" | "override"] ["gen"] "fn" …`:
## `gen` comes right before `fn`, wherever `fn` may (D-660).

gen fn ticks() -> Generator[int]:
    yield 1
    yield 2

unsafe gen fn raw() -> Generator[int]:  # SAFETY: parse only
    yield 3

extern "C" gen fn foreign() -> Generator[int]:
    yield 4

class C:
    virtual gen fn walk(self) -> Generator[int]:
        yield 5

enum E:
    A

    gen fn each(self) -> Generator[int]:
        yield 6
