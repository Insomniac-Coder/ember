#$ test: compile-fail
#$ rules: IFC-2
# `[IFC-2]` — methods of a type's own belong to the package that declares it;
# the language's types and the standard library's are another package's.

extend i64:   #$ error[E2120]: `i64` is another package's type, so `extend` cannot give it methods of its own
    fn twice(self) -> i64:
        return self * 2

extend[T] Array[T]:   #$ error[E2120]: `Array` is another package's type, so `extend` cannot give it methods of its own
    fn second(self) -> T:
        return self[1]
