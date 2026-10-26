# 26 Timer (run 3, phase 4)

Power: PowerLineStatus printed `Online` before the builds; `run.ps1` checks mains before every round (no ABORT line in any run file).
Builds: fresh, clang -O2 and `rustc -C opt-level=2 --edition 2021`, into `target/redesign3/26-timer/bin/`. Model C = default builds of `target/redesign3/25-c-writer-fix/wN.c`; `plain_same` = same files `-DPLAIN`; extras `diag_w2_hop_proved` (`-DHOP_PROVED`) and `model_w4_state_block` (`-DSTATE_BLOCK`). Baselines from `target/redesign2/bench/`.
Timing: `run.ps1 -Rounds 5`, one set per workload, nothing else running. Raw lines: `runs/wN.txt` (first), `wNb.txt`, `wNc.txt` (reruns). Medians: `med.py` -> `summary_a/b/c.txt`. Ratio = median ms / plain C median ms of the same run set (plain C = `plain_w1_group`, `plain_w2`, `plain_w3`, `plain_w4`). Spread = (slowest-fastest)/fastest. Checksums: every run of every program equals plain C's (W1 2111831520, W2 647034814, W3 9288233970, W4 34946993000), exit 0.

Reruns: W1 and W2 sets had a program over 10% spread, so each was rerun twice (the maximum); W3 and W4 stayed under 10% (W4 rust_w4 9.1%). The spread stayed above 10% for a few programs even in the third set (listed below), so those numbers are noisy; the medians agree across the three sets (W2 model 959.6 / 941.2 / 909.6 ms; W1 model 89.4 / 85.2 / 81.4 ms). Reported = the last set for W1, W2 (summary_c), the only set for W3, W4 (summary_a).

## Results (median ms, peak KB, ratio to plain C, spread)

| wl | program | ms | KB | x C | spread |
|---|---|---|---|---|---|
| w1 | plain_w1_group | 93.306 | 39176 | 1.0 | 9.0% |
| w1 | plain_w1_group_local | 83.485 | 39108 | 0.895 | 9.6% |
| w1 | plain_w1_arch | 44.078 | 17116 | 0.472 | 13.9% |
| w1 | rust_w1 | 100.401 | 39548 | 1.076 | 12.0% |
| w1 | model_w1 | 81.419 | 35800 | 0.873 | 9.1% |
| w1 | plain_same_w1 | 78.484 | 35800 | 0.841 | 27.2% |
| w2 | plain_w2 | 575.953 | 54940 | 1.0 | 20.6% |
| w2 | rust_w2 | 923.408 | 52296 | 1.603 | 6.0% |
| w2 | model_w2 | 909.582 | 84368 | 1.579 | 3.7% |
| w2 | diag_w2_hop_proved | 677.598 | 84376 | 1.176 | 14.8% |
| w2 | plain_same_w2 | 742.039 | 84328 | 1.288 | 3.8% |
| w3 | plain_w3 | 443.483 | 9752 | 1.0 | 5.1% |
| w3 | rust_w3 | 635.435 | 11532 | 1.433 | 4.8% |
| w3 | model_w3 | 439.525 | 8632 | 0.991 | 3.1% |
| w3 | plain_same_w3 | 420.887 | 8640 | 0.949 | 1.6% |
| w4 | plain_w4 | 23.673 | 4072 | 1.0 | 3.1% |
| w4 | rust_w4 | 32.142 | 4476 | 1.358 | 9.1% |
| w4 | rust_w4_return | 17.028 | 4472 | 0.719 | 6.8% |
| w4 | model_w4 | 29.717 | 4532 | 1.255 | 2.2% |
| w4 | model_w4_state_block | 25.821 | 4264 | 1.091 | 4.6% |
| w4 | plain_same_w4 | 21.539 | 4264 | 0.91 | 3.8% |

Reading: the model (default) is 0.87x C on W1, 1.58x on W2, 0.99x on W3, 1.26x on W4 (W4 with the state-block placement 1.09x). Safe Rust: 1.08x, 1.60x, 1.43x, 1.36x. The W2 cost is the per-hop DEAD test: without it (`diag_w2_hop_proved`) 1.18x. `plain_same` is not a floor on W1 and W2 (it keeps the model's layout; W1 `plain_w1_arch` is a different layout, 0.47x). W2 memory is 84 MB for the model layout against 55 MB for plain C.
