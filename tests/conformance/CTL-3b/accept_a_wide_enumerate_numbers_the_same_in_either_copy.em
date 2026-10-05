#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release
#$ stdout:
#$ -2:5 -1:6 0:7 1:8
#$ 9223372036854775806:5 9223372036854775807:6 9223372036854775808:7 9223372036854775809:8
#$ 6:8 5:7 4:6 3:5
#$ 9223372036854775808:8 9223372036854775807:7 9223372036854775806:6 9223372036854775805:5
#$ 1:6 3:8 5:10
#$ 9223372036854775806:6 9223372036854775808:8 9223372036854775810:10
#$ 7:0 8:1 7:2 8:3
#$ 9223372036854775807:0 9223372036854775808:1 9223372036854775807:2 9223372036854775808:3
#$ -1:3 0:4 1:5
#$ 9223372036854775807:3 9223372036854775808:4 9223372036854775809:5
# `[CTL-3b]` (G8-4 decision C) — an `enumerate` whose numbers are not visible
# numbers a range of 64-bit numbers in `i128` (128-bit numbers in `i256`), and
# the loop has a copy computing them as `int`s, taken when every number fits
# one. Each line pair is one loop: its numbers fit an `int`, then they pass
# its top, forwards, backwards, under `skip` and `step_by`, and in a `chain`.

fn show_for(m: int, n: int, s: int) -> String:
    out: Array[String] = []
    for (i, x) in (m..n).iter().enumerate(start=s):
        out.push(f"{i}:{x}")
    return out.join(" ")

fn show_rev(m: int, n: int, s: int) -> String:
    out: Array[String] = []
    for (i, x) in (m..n).iter().enumerate(start=s).rev():
        out.push(f"{i}:{x}")
    return out.join(" ")

fn show_skip(m: int, n: int, s: int) -> String:
    out: Array[String] = []
    for (i, x) in (m..n).iter().skip(1).enumerate(start=s).step_by(2):
        out.push(f"{i}:{x}")
    return out.join(" ")

fn show_chain(m: int, n: int, s: int) -> String:
    out: Array[String] = []
    for (i, x) in (m..n).iter().enumerate(start=s).chain((n..n + 2).iter().enumerate(start=s)):
        out.push(f"{i}:{x}")
    return out.join(" ")

fn show_wide(m: u128, n: u128, s: int) -> String:
    out: Array[String] = []
    for (i, x) in (m..n).iter().enumerate(start=s):
        out.push(f"{i}:{x}")
    return out.join(" ")

fn main():
    seed: Array[int] = [1]
    z = seed.len() - 1
    println(show_for(z + 5, z + 9, z - 2))
    println(show_for(z + 5, z + 9, int.MAX - 1))
    println(show_rev(z + 5, z + 9, z + 3))
    println(show_rev(z + 5, z + 9, int.MAX - 2))
    println(show_skip(z + 5, z + 12, z + 1))
    println(show_skip(z + 5, z + 12, int.MAX - 1))
    println(show_chain(z, z + 2, z + 7))
    println(show_chain(z, z + 2, int.MAX))
    println(show_wide((z + 3) as u128, (z + 6) as u128, z - 1))
    println(show_wide((z + 3) as u128, (z + 6) as u128, int.MAX))
