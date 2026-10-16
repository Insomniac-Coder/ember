#$ test: run-pass
#$ rules: LEX-15
#$ profiles: debug
#$ stdout: 15
# `[LEX-15]` (changed in 0.9.9) — only `async`, `await` and `macro` are reserved for a future
# version. `actor`, `move`, `trait`, `loop` and `unless`, which earlier versions reserved, are
# ordinary names (D-642).

fn main():
    actor = 1
    move = 2
    trait = 3
    loop = 4
    unless = 5
    println(actor + move + trait + loop + unless)
