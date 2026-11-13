#$ test: run-pass
#$ rules: TYP-12
#$ stdout: 1 2 4 4 8
# `[TYP-12]` (D-720) - with no `@repr` the tag is the smallest *signed* type that holds every
# discriminant: 127 fits `i8`, 200 needs `i16`, 40000 needs `i32`, 3000000000 needs `i64`.

enum Small:
    A = 127

enum Byte:
    A = 200

enum Half:
    A = 32768

enum Word:
    A = 40000

enum Big:
    A = 3000000000

fn main():
    println(mem.size_of[Small](), mem.size_of[Byte](), mem.size_of[Half](), mem.size_of[Word](), mem.size_of[Big]())
