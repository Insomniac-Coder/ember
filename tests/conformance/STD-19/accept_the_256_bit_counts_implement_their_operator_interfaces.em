#$ test: run-pass
#$ rules: STD-19, TYP-1
#$ profiles: debug, release
#$ stdout:
#$ 8 18 -5 5
#$ -6 0 42 -42
#$ 680564733841876926926749214863536422910 680564733841876926926749214863536422911
# ODR-040 — every number type implements the interface of each operator it
# has: `i256` and `u256` add, subtract and negate, so they implement `Add`,
# `Sub`, `Neg`, `AddAssign` and `SubAssign`, and generic code bounded by them
# takes a 256-bit count (G8-4).

from std.core import Add, Sub, Neg, AddAssign

fn twice[T: Add[Output = T] + Copy](x: T) -> T:
    return x + x

fn less[T: Sub[Output = T] + Copy](x: T, y: T) -> T:
    return x - y

fn flip[T: Neg[Output = T]](x: T) -> T:
    return -x

fn grow[T: AddAssign + Copy](x: T) -> T:
    y = x
    y += x
    return y

fn main():
    println(twice(4 as i256), twice(9 as u256), less(4 as i256, 9 as i256), less(9 as u256, 4 as u256))
    println(flip(6 as i256), flip(0 as u256), grow(21 as u256), grow(-21 as i256))
    big = u128.MAX as u256
    println(twice(big), (big + big).add(1 as u256))
