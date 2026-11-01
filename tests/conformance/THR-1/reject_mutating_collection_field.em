#$ test: compile-fail
#$ rules: THR-1, CLS-8
#$ profiles: debug
#$ error[E7003]: cannot write field `items` of an `@sync` class after `init` or after using `this` as a whole

@sync
class Token:
    items: Array[int]

    fn init():
        this.items = [1]

fn main():
    token = Token()
    token.items.push(owns 2)
