Compiler speed, Intel64 Family 6 Model 198 Stepping 2, GenuineIntel (Windows, 24 logical processors); bigpkg 400 modules, 50000 lines; 15 rounds after 3 warm-up rounds.

| Figure | Operation | Before | After | Change | Verdict |
|---|---|---|---|---|---|
| B1 | `ember check` after a one-function-body edit | 1074.6 ms (spread 2.9%) | 1079.2 ms (spread 7.4%) | no measurable change, 1.004x | pass |
| B3 | `ember build` after a one-function-body edit, `debug` | 2141.6 ms (spread 4.4%) | 2133.7 ms (spread 5.0%) | no measurable change, 0.996x | pass |
| B6 | clean `debug` build, cold cache | 5029.9 ms (spread 5.6%) | 4973.0 ms (spread 1.6%) | no measurable change, 0.989x | pass |
| B9 | compiler peak memory, clean build | 309.9 MB (spread 0.9%) | 309.7 MB (spread 0.5%) | no measurable change, 0.999x | pass |
