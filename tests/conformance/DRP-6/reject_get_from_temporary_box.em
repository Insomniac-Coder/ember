#$ test: compile-fail
#$ rules: DRP-6, LT-3, BRW-1
# A reference returned by get is rooted at its Box owner. A statement-end
# temporary cannot provide a place whose lifetime contains the reference.

fn main():
    payload = Box(9).get() #$ error[E2140]: `Box.get` needs an owner place so its returned reference cannot outlive a temporary
    println(payload)
