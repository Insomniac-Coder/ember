#$ test: run-pass
#$ rules: TYP-16, OBJ-2, DSP-3, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_itable_lookup)

# A generic interface application in handle position materializes its checked
# contract before the class is erased to the one-word interface handle.
interface Project[T]:
    fn project(self) -> T

class Pixel[T] implements Project[T]:
    value: T

    fn project(self) -> T:
        return self.value

fn project(value: Project[i32]) -> i32:
    return value.project()

fn main():
    pixel = Pixel[i32](42)
    println(project(pixel))
