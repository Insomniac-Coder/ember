#$ test: compile-fail
#$ rules: R5
#$ help: fn reset() changes this:
## R5, rules section 3's removed words: `mut self` is gone; a method that changes its object says
## `changes this` after the brackets and names it `this` (E1010; completeness G8).

struct Counter:
    count: int = 0

    fn reset(mut self):    #$ error[E1010]: `mut self` is not Ember: inside a method its object is `this`, and a method that changes it says so after the brackets; write `fn reset() changes this:`, and `this.count = 0` in its body
        self.count = 0

fn main():
    c = Counter(count=3)
    c.reset()
    println(c.count)
