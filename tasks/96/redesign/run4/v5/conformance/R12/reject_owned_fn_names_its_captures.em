#$ test: compile-fail
#$ rules: MOD-3
#$ help: f = fn() owns name => println(name)
## design rules: R12, R3
## R12 (rules.md lines 752-753, 766-767) and section 3's removed words (line 1113: "the capture words after
## the brackets for `owned fn`"): `owned fn` handed some captures over and silently copied others; a
## function says what it takes or copies on each captured name, after its brackets with no `with`
## (verdict 83) (E1010). The vocabulary's `owned fn` row, in this program's names (completeness G8). The
## nearest allowed program is accept_a_lambda_takes_the_name_it_uses_with_owns.

fn main():
    name = String("ann")
    f = owned fn() => println(name)    #$ error[E1010]: `owned fn` is not Ember: say what the function takes or copies on each name it uses: `f = fn() owns name => println(name)` (it takes `name` over), or `f = fn() copy of name => println(name)` (a copy of its own)
    f()
