# A grade book: record scores per student, compute averages and letter grades, rank the class and draw a histogram.

struct Student:
    name: String
    scores: Array[int] = []

    fn average(self) -> float:
        if self.scores.len() == 0:
            return 0.0
        return sum(self.scores) as float / self.scores.len() as float

    fn best(self) -> int:
        return self.scores.iter().max().unwrap()

fn letter(avg: float) -> str:
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

    fn init(self, owned course: String):
        self.course = course

    fn record(self, name: str, score: int):
        if not 0 <= score <= 100:
            println(f"ignoring impossible score {score} for {name}")
            return
        if name not in self.students:
            self.students[name] = Student(name=name.to_string())
        self.students[name].scores.push(score)

    fn class_average(self) -> float:
        averages = [s.average() for s in self.students.values()]
        return sum(averages) / averages.len() as float

    fn ranking(self) -> Array[Student]:
        result = [s.clone() for s in self.students.values()]
        result.sort_by_key(fn(s) => -s.average())
        return result

    fn histogram(self) -> Map[str, int]:
        counts: Map[str, int] = {"A": 0, "B": 0, "C": 0, "D": 0, "F": 0}
        for s in self.students.values():
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
