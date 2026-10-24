# Attack on A23 7.3 (an emptied place must be refilled before anything can see it): a `defer` block
# reads the place at the block's exit. Checked here on a local, where [OWN-3]'s tracking already
# exists: does the checker see the defer body as running at the exit, after the move?
fn main():
    s: String = "kept"
    if true:
        defer:
            println(s)
        t = s
        println(t)
    println("done")
