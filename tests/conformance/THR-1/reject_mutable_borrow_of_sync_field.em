#$ test: compile-fail
#$ rules: THR-1
#$ profiles: debug
#$ error[E7003]: cannot write field `value` of an `@sync` class after `init` or after using `this` as a whole

@sync
class Token:
    value: int

    fn init():
        this.value = 1

fn main():
    token = Token()
    borrow: ref mut int = ref mut token.value
