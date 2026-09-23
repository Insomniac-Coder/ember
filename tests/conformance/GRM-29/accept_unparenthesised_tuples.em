#$ test: run-pass
#$ rules: GRM-29, GRM-5
#$ profiles: debug, release, shipping
#$ stdout: 2
#$ 1
#$ -4
#$ 1
#$ 7
#$ 5
#$ 2.5
# An `expr_list` of two or more expressions, or of one followed by a comma, is
# a tuple: after `return`, on the right of `=`, and as a declaration's value.

fn pair() -> (int, int):
    return 1, 2

fn divmod(a: int, b: int) -> (int, int):
    return a // b, a % b

fn main():
    a, b = pair()
    a, b = b, a
    println(a)
    println(b)
    q, r = divmod(-7, 2)
    println(q)
    println(r)
    t = 3, 4
    println(t.0 + t.1)
    one = 5,
    println(one.0)
    p: (int, float) = 1, 2.5
    println(p.1)
