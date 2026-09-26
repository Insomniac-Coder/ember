#$ test: compile-fail
#$ rules: THR-1
#$ profiles: debug
#$ error[E7003]: `@sync` class `Token` cannot declare a `mut self` method

@sync
class Token:
    value: int = 1

    fn bump(mut self):
        pass
