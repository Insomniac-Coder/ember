# A19: a frame counter and a log every system can reach (Python: module globals).

static FRAME: int = 0
static LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push("tick".to_string())

fn main():
    tick()
