#$ test: compile-fail
#$ rules: OWN-8, STR-5

# A derived `Clone` needs every payload to implement it. `Token` has its own
# `drop`, so it is `Clone` only if it says so (ODR-026), and it does not.

struct Token:
    value: i32

    fn drop(mut self):
        pass

@derive(Clone)
enum Entry:
    Value(Token) #$ error[E2040]: payload `_0` has type `Token`, which does not implement `Clone`

fn main():
    _entry = Entry.Value(Token(1))
