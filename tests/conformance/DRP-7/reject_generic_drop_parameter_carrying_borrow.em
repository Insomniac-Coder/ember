#$ test: compile-fail
#$ rules: DRP-7
# Holder's own drop reads a separate, long-lived view. Its generic payload
# carries the short-lived view and must remain valid through the generic drop.

struct Borrowed:
    text: str

struct Holder[T]:
    payload: T
    witness: str

    fn drop(mut self):
        println(self.witness)

fn main():
    stable: String = "stable"
    _outer: Option[Holder[Borrowed]] = None
    if true:
        transient: String = "short"
        payload = Borrowed(transient.as_str()) #$ error[E3060]:
        _outer = Some(Holder(payload, stable.as_str()))
