#$ test: compile-fail
#$ rules: ATT-1, PHIL-12
#$ help: write `@fp(allow_fma)` in place of line 9
## design rules: R3, R16
## R3, R16 and the removed words of `V5/rules.md` section 3: `@fp(contract)` is written `@fp(allow_fma)`,
## an attribute that is gone (E0104, `V5/rules.md` section 4; the `@fp` row of `V5/vocabulary.md`;
## step 3's re-check: a refusal with no test). The nearest program that is allowed is
## `accept_fp_allow_fma_lets_the_compiler_fuse`.

@fp(contract)                  #$ error[E0104]: `@fp(contract)` is now `@fp(allow_fma)`
fn mix(a: f64, b: f64, c: f64) -> f64:
    return a * b + c

fn main():
    println(mix(2.0, 3.0, 1.0))
