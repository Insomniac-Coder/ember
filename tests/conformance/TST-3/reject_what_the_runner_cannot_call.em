#$ test: compile-fail
#$ rules: TST-3, ATT-1
#$ profiles: debug
# The runner calls a test with nothing and expects nothing back: a test is a
# free function with no parameters and no type parameters that returns
# nothing. `@should_panic` qualifies a test, and neither takes arguments.

@test                       #$ error[E0104]: `@test` does not apply to this function: it takes 1 parameter
fn needs_a_value(x: int):
    assert(x > 0)

@test                       #$ error[E0104]: `@test` does not apply to this function: it returns `i64`
fn gives_a_value() -> int:
    return 1

@test                       #$ error[E0104]: `@test` does not apply to this function: it has type parameters
fn generic[T](value: T):
    pass

@should_panic               #$ error[E0104]: `@should_panic` applies to a `@test` function
fn not_a_test():
    panic("never run")

@test("named")              #$ error[E0104]: `@test` takes no arguments
fn with_arguments():
    pass

struct Counter:
    count: int

extend Counter:
    @test                   #$ error[E0104]: `@test` applies to a free function, not a method
    fn method(self):
        pass

fn main():
    pass
