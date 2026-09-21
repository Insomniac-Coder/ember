#$ test: compile-fail
#$ rules: OWN-8, TYP-16, ENM-4

struct Token:
    value: i32

@derive(Copy)
enum Flag[T]:
    Value(value: T) #$ error[E2080]: payload `value` has type `Token`, which is not Copy

fn main():
    _flag = Flag[Token].Value(Token(3))
