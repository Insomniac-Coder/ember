#$ test: compile-fail
#$ rules: THR-1
#$ profiles: debug
#$ error[E7003]: cannot write field `value` of an `@sync` class after `init` or after using `self` as a whole

@sync
class Token:
    value: int = 1

    fn init(mut self):
        self
        self.value = 2
