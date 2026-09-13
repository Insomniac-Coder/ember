#$ test: compile-fail
#$ rules: UNS-10, MOD-5

fn main():
    cell: UnsafeCell[i32] #$ error[E1010]: cannot find type `UnsafeCell` in this scope
