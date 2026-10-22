# Rulings made under the delegation, waiting for the next spec cut (Hardened_36)

Cut 35 (2026-10-23, ADR-274) wrote ODR-239..ODR-259, and ODR-223 again (the LLVM backend is part of
1.0, phase 9), into the spec; their record is `docs/OWNER-QUEUE.md`. Open, for the owner: ODR-238
(`[CLO-15]`, the user registry).

- ODR-261 [HR-1] lead, under the owner's ODR-242 (cut 35 left HR-1's fixed time behind): an edit to one function body of a 50k-line reloadable package is running in the process with all live state preserved, measured end to end (B2's build, then load, plan, prepare and commit, over a live set of 10,000 instances, `[BEN-8]`) as a published figure that `[BUD-3]` gates by before-and-after comparison on one machine; no limit in seconds. NOT BUILT (helper AE's gate).
- ODR-262 [BEN-4] lead: retired instructions are read from the hardware where the host gives them (Linux `perf_event_open`); where it does not (a VM, WSL2, a Windows user process, GitHub's runners), from Cachegrind's exact instruction count (`valgrind --tool=cachegrind`, its `Ir`) on Linux, deterministic and the same on every run; a host with neither reports the figure as unavailable, never as passing. The ± 0.5 % gate applies to whichever count the run recorded, against a baseline of the same kind. NOT BUILT (WSL needs `sudo apt install valgrind`).
- OPEN (lead): AB Q9, `Iterable` as a declared interface (`[FFI-42]`, `[CTL-1]`): std declares none, and `[FFI-42a]`'s `iter` takes `mut self`.
