#$ test: compile-fail
#$ rules: MOD-7
## "Outside the declaring module, the following are errors `E1050`:
## assignment (`h.value = x`, augmented assignment), taking `ref mut h.value`,
## passing `h.value` to a `mut` parameter or `mut self` method, and
## destructuring it with a mutable binding."

from support.health import make

fn bump(mut n: i32):
    n = n + 1

fn main():
    h = make(10)
    h.value = 99          #$ error[E1050]: `support.health.Health.value` is read-only outside its module
    bump(h.value)         #$ error[E1050]: `support.health.Health.value` is read-only outside its module
