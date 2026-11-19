#$ test: compile-pass
#$ rules: WK-9

# The integration suite asks `ember explain --cycle` to render this graph.
class Root:
    child: shares mut Child    #$ warning[L3001]: potential reference cycle

class Child:
    root: shares mut Root

fn main():
    return
