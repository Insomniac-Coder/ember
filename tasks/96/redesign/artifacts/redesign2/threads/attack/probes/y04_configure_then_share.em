# PROPOSED syntax. A write outside init, before the share: refused by "every write to C.f outside init must
# be guarded" (4.6), though only this thread can see the object when the write happens.
import std.thread

class Config:
    scale: int = 2

fn main():
    cfg = Config()
    cfg.scale = 3
    worker = thread.spawn(fn(): println(cfg.scale))
    worker.join()
