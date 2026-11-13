#$ test: compile-fail
#$ rules: GRM-47
## `[GRM-47]` — an `owns` expression that is an operand of an operator of rows 4 to 17 (`or` to `**`: a
## comparison, a range's end, `in`, `is`, a prefix `-`, `~`, `not` included) is `E0119`, with or
## without brackets of its own.

fn main():
    a = 1
    b = 2
    c = 3
    d = 4
    e = 5
    f = 6
    g = 7
    h = true
    k = 9
    r1 = owns a + b            #$ error[E0119]: an operand of `+`
    r2 = c < owns d            #$ error[E0119]: an operand of `<`
    r3 = owns (e + 1) * 2      #$ error[E0119]: an operand of `*`
    r4 = 1 + owns f ** 2       #$ error[E0119]: an operand of `+`
    r5 = -owns g               #$ error[E0119]: an operand of `-`
    r6 = not owns h            #$ error[E0119]: an operand of `not`
    r7 = 0..owns k             #$ error[E0119]: an operand of `..`
    println(r1, r2, r3, r4, r5, r6)
