#$ test: run-pass
#$ rules: TYP-16, STR-1
#$ stdout: 1 2 3 4
# Part IV §8 — `Self` in a type's methods is the type, and a constructor may
# call `Self(…)`. In a generic type, `Self` and the type's own name with its
# parameters (`W[T]`) are the instance.

struct W[T]:
    x: T

    fn make(v: T) -> W[T]:
        return W[T](v)

    fn again(self) -> Self:
        return Self(self.x)

extend[T] W[T]:
    fn make2(v: T) -> W[T]:
        return W[T](v)

    fn make3(v: T) -> Self:
        return Self(v)

fn main():
    println(W[int].make(1).x, W[int].make2(2).x, W[int].make3(3).x, W[int](4).again().x)
