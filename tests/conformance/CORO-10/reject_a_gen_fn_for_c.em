#$ test: compile-fail
#$ rules: CORO-10
#$ profiles: debug
# `[CORO-10]` — a `gen fn` may not be `extern`, `@export`ed or passed to C (`E2222`): calling
# a generator builds a frame only Ember code resumes.

extern "C" gen fn foreign() -> Generator[int]:    #$ error[E2222]: cannot be `extern` or `@export`ed
    yield 1

@export
gen fn exported() -> Generator[int]:    #$ error[E2222]: cannot be `extern` or `@export`ed
    yield 2

fn main():
    pass
