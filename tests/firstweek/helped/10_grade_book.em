# A grade book: record scores per student, compute averages and letter grades, rank the class and draw a histogram.

struct Student:
    name: String
    scores: Array[int] = []

    fn average() -> float:
        if this.scores.len() == 0:
            return 0.0
        return sum(this.scores) as float / this.scores.len() as float

    fn best() -> int:
        return this.scores.iter().max().unwrap()

fn letter(avg: float) -> StrView:
    if avg >= 90.0:
        return "A"
    elif avg >= 80.0:
        return "B"
    elif avg >= 70.0:
        return "C"
    elif avg >= 60.0:
        return "D"
    return "F"

class GradeBook:
    course: String
    students: Map[String, Student] = {}

    fn init(owns course: String):
        this.course = course

    fn record(name: StrView, score: int):
        if not 0 <= score <= 100:
            println(f"ignoring impossible score {score} for {name}")
            return
        if name not in this.students:
            this.students[name] = Student(name=name.to_string())
        this.students[name].scores.push(score)

    fn class_average() -> float:
        averages = [s.average() for s in this.students.values()]
        return sum(averages) / averages.len() as float

    fn ranking() -> Array[Student]:
        result = [s.clone() for s in this.students.values()]
        result.sort_by_key(fn(s) => -s.average())
        return result

    fn histogram() -> Map[StrView, int]:
        counts: Map[StrView, int] = {"A": 0, "B": 0, "C": 0, "D": 0, "F": 0}
        for s in this.students.values():
            counts[letter(s.average())] += 1
        return counts

fn main():
    book = GradeBook("Intro to Ember")
    raw = [("ann", 92), ("bob", 71), ("cid", 58), ("ann", 88), ("bob", 79),
           ("dee", 100), ("cid", 64), ("dee", 97), ("eve", 83), ("eve", 105), ("bob", 66)]
    for name, score in raw:
        book.record(name, score)

    println(f"== {book.course} ==")
    for rank, s in book.ranking().iter().enumerate():
        avg = s.average()
        println(f"{rank + 1}. {s.name:<5} avg {avg:6.2f}  best {s.best():>3}  grade {letter(avg)}")
    println(f"class average: {book.class_average():.1f}")

    println("grades:")
    for grade, n in book.histogram().items():
        println(f"  {grade}: {"*".repeat(n)}")
