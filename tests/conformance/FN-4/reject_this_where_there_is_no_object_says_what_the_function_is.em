#$ test: compile-fail
#$ rules: FN-4, LEX-15
# `[FN-4]` — `this` in a `static fn` or a free function is `E1010`: "`this` can only be used in a
# method, and `starting_at` is a `static fn`, not a method"; "`this` can only be used in a method,
# and `double` is not a method", with the help "to make `double` a method, move it into the struct or
# class it belongs to".

struct Counter:
    n: int

    static fn starting_at(k: int) -> Counter:
        this.n = k                  #$ error[E1010]: `this` can only be used in a method, and `starting_at` is a `static fn`, not a method
        return Counter(n=k)

fn double(x: int) -> int:
    return this.n * x               #$ error[E1010]: `this` can only be used in a method, and `double` is not a method
#$ help: to make `double` a method, move it into the struct or class it belongs to

fn main():
    println(double(2), Counter.starting_at(5).n)
