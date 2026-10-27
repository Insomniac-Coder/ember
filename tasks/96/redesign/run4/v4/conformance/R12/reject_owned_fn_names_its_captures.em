#$ test: compile-fail
#$ rules: R12, R3
#$ help: f = fn() with owns name => println(name)
## R12 (rules.md lines 741-748, 756-757) and section 3's removed words (line 1062: "the capture list for
## `owned fn`"): `owned fn` handed some captures over and silently copied others; a function says what it
## takes or copies on each captured name (E1010). The vocabulary's `owned fn` rows, in this program's
## names (completeness G8).

fn main():
    name = String("ann")
    f = owned fn() => println(name)    #$ error[E1010]: `owned fn` is not Ember: say what the function takes or copies on each name it uses: `f = fn() with owns name => println(name)` (it takes `name` over), or `f = fn() with copy of name => println(name)` (a copy of its own)
    f()
