#$ test: compile-fail
#$ rules: THR-1
#$ profiles: debug
#$ error[E7003]: cannot write field `items` of an `@sync` class after `init` or after using `self` as a whole

@sync
class Token:
    items: Array[int]

    fn init(mut self):
        self.items = [1]

fn main():
    token = Token()
    token.items.push(2)
