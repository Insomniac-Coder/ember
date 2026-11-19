#$ test: compile-pass
#$ rules: CLI-18

# Command-level root and target behavior is exercised by the driver integration
# test; this source anchors the shared ownership graph in the conformance map.
class Root:
    child: shares mut Child    #$ warning[L3001]: potential reference cycle

class Child:
    root: shares mut Root

fn main():
    return
