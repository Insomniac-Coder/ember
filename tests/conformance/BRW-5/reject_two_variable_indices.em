#$ test: compile-fail
#$ rules: BRW-5
# And the default: two indices the compiler cannot tell apart conflict, because
# nothing says `i != j`. `split_at_mut`, `chunks_mut` and `iter_mut` are the
# sanctioned ways to get two mutable element borrows.

fn main():
    v: Array[i32] = Array[i32]()
    v.push(1)
    v.push(2)
    i: usize = 0
    j: usize = 1
    a: ref mut i32 = ref mut v[i]
    b: ref mut i32 = ref mut v[j]     #$ error[E3022]: `v[…]` is already mutably borrowed
    a = 10
    b = 20
    println(3)
