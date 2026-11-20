# Clause CLS-6.2: "The kept results of the object's `@cached` methods (`[CLS-12]`) end with its
# fields."
#$ test: run-pass
#$ rules: CLS-6, CLS-12
#$ stdout: tag t1
#$ stdout: report ends
#$ stdout: tag t1 ends
# The kept `Tag` ends after the object's own `drop`, with its fields; it is not made again by the
# second call.

class Tag:
    name: String

    fn init(owns name: String):
        this.name = name

    fn drop():
        println(f"tag {this.name} ends")

class Report:
    title: String

    fn init(owns title: String):
        this.title = title

    @cached
    fn tag() -> Tag:
        return Tag(f"{this.title}")

    fn drop():
        println("report ends")

fn show():
    r = Report(String("t1"))
    t = r.tag()
    println(f"tag {t.name}")
    _u = r.tag()

fn main():
    show()
