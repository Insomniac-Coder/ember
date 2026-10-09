# Map lookup performance — 2026-10-09

The baseline is `b9847eacd0f007da4b26c425299059314aa7ba7d`. ADR-144 replaces
`Map.find`'s private tagged result with one integer: a nonnegative table position
or -1 for absence. All twelve callers check that result; public Option/reference
results retain their contracts. Only the immutable indexed-read adapter is forced
inline. The implementation applies to every map key, value and hasher.

## Matched results

Each ratio below is new Ember time / baseline Ember time. Lower is faster.

| Workload | MSVC | clang | GCC |
|---|---:|---:|---:|
| Dense integer keys | 0.713 | 0.999 | 0.848 |
| Scrambled integer keys | 0.916 | 0.998 | 0.992 |
| Integer keys spaced by 1024 | 0.873 | 1.003 | 0.863 |
| Integer keys spaced by 1048576 | 0.729 | 0.978 | 0.893 |
| Text keys | 0.923 | 1.009 | 0.904 |

MSVC's text map is about 8% faster and now 1.03 times its C++ twin; GCC's is
about 10% faster. Clang's 51-sample text-map comparison has a paired-resampling
95% interval of 0.989–1.022, including parity. No speedup is claimed for it.

All 150 compiler/program comparisons produce matching output. No selected
new/baseline median exceeds the existing regression threshold of 1.05. All 135
non-map Ember executable `.text` sections are byte-identical to the baseline.
Suspicious rows were repeated; raw samples retain the small timing variations.
The charts now contain 46 programs within 5% of C or faster, one within 10%, and
three over 10% with at least one compiler.

## Measurement and controls

The final samples use the owner's plugged-in laptop, release native profiles,
and the same C compiler/optimization flags for Ember and its twin. Old Ember,
new Ember and the twin run in shuffled order, after warmups, with output checked
on every run. Selected rows have 11–51 samples per executable. Windows affinity
is `0xC03C03`; CPUID confirmed that all eight selected logical CPUs report the
performance-core type. GCC is pinned to WSL guest CPU 0; host performance-core
placement remains unverified. The owner suspended temperature limits for this work.
Windows power-source event logs contain no changes in the retained timing windows.

The published GCC matrix executes byte-verified copies from one temporary
directory on Linux's home filesystem, with executable permissions and device
identity checked. That directory was removed afterward. Sources and logs stay
in the repository. Earlier matched runs on the Windows-mounted filesystem are
diagnostics; they are not the published GCC matrix, because filesystem startup
cost dominates the empty-program comparison.

The b16 C twins exposed an independent reference problem: MSVC's same scalar
loop ranged from about 0.5 to 7 seconds with ordinary allocation placement.
Equal page-offset controls reproduced about 6.8 seconds; separated offsets
gave about 0.46 seconds with the same inner-loop instructions and checksum.
The final references use offsets 0, 1024 and 2048 with sufficient allocation
padding. The initially tried 64-byte spacing slowed clang's streaming loop and
was rejected. Final timings are about 0.46 seconds with MSVC and 0.39 seconds
with clang/GCC. See the [reference-buffer rationale](../../../bench/README.md#stable-reference-buffers).
This corrects the comparison; it is not an Ember speedup.

Forcing both map methods inline was rejected after the first full matrix found
9–17% regressions in three GCC integer-map workloads. A two-digit integer
formatter also failed to improve the target and was discarded. Diagnostic
removal of bounds/overflow panic branches did not consistently explain the
original gap; no check removal was adopted.

## Verification and evidence

The compact-result conformance assertion fails with the old representation and
passes with the new one under MSVC, clang and GCC in debug, release and shipping.
The fixture covers every lookup caller, empty maps, position zero, mutation,
entry operations, removal, reinsertion and owned/borrowed text keys. Existing
map, collision, close-up and missing-key checks pass too. All twelve local
repository gate commands pass, including the unchanged Appendix byte check.
Full-suite CI remains the promotion gate.

- [Raw final samples and code-section comparisons](map-performance.json)
- [Windows published results](../../../bench/results/2026-10-09-windows.log)
- [GCC published results](../../../bench/results/2026-10-09-gcc.log)
- [Associated-type review, a separate open task](associated-types.md)
