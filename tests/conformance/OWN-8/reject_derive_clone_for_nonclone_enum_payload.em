#$ test: compile-fail
#$ rules: OWN-8

struct Token:
    value: i32

@derive(Clone)
enum Entry:
    Value(Token) #$ error[E2040]: payload `_0` has type `Token`, which does not implement `Clone`

fn main():
    _entry = Entry.Value(Token(1))
