#$ test: run-pass
#$ rules: HR-38
#$ stdout: SchemaRefused Particle: field `mass` added with no default
#$ Migration Particle 7: overflow
#$ Allocation
#$ Foreign errno 5
#$ Timeout
# `std.hot.ReloadError` is `SchemaRefused(type, reason)`, `Migration(type, instance, message)`,
# `Allocation`, `Foreign(error)` or `Timeout`: a match naming the five is exhaustive.

from std.hot import ReloadError

fn describe(e: ReloadError) -> String:
    match e:
        ReloadError.SchemaRefused(type_name, reason):
            return f"SchemaRefused {type_name}: {reason}"
        ReloadError.Migration(type_name, instance, message):
            return f"Migration {type_name} {instance}: {message}"
        ReloadError.Allocation:
            return "Allocation"
        ReloadError.Foreign(error):
            return f"Foreign {error}"
        ReloadError.Timeout:
            return "Timeout"

fn main():
    println(describe(ReloadError.SchemaRefused("Particle".to_string(), "field `mass` added with no default".to_string())))
    println(describe(ReloadError.Migration("Particle".to_string(), 7, "overflow".to_string())))
    println(describe(ReloadError.Allocation))
    println(describe(ReloadError.Foreign("errno 5".to_string())))
    println(describe(ReloadError.Timeout))
