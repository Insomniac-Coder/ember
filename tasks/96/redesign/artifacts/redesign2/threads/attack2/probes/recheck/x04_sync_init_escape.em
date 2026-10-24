# A15: today's [THR-1] init clause: a field assigned in init after self escaped to a thread.
import std.thread

@sync
class Worker:
    name: String = "".to_string()

    fn init(self, name: String):
        h = thread.spawn(owned fn(): println(self.name))
        self.name = name
        h.join()

fn main():
    w = Worker("bob".to_string())
    println(w.name)
