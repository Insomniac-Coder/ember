#$ test: compile-pass
#$ rules: CLI-18

# Command-level root and target behavior is exercised by the driver integration
# test; this source anchors the shared ownership graph in the conformance map.
class Root:
    child: Child

class Child:
    root: Root

fn main():
    return
