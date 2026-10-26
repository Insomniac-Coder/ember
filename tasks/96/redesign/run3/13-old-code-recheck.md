# 13 old-code re-check (run 3, agent 13)

Method: `target/redesign3/13-old-code-recheck/run.sh` ran `target/redesign3/bin/ember.exe check` afresh on every
.em in the five area folders (not `new/`): output in `fresh/<area>.txt`. `cmp.py` compared each fresh output
line by line with the saved check_output files (A, B, C, D); `rep.py` and `codes.py` compared with the report text;
E has only exit codes saved (`check_summary.txt`), compared too.

Files checked: A 45, B 51, C 63, D 44, E 42 = 245.

Results
- Fresh output equals the saved output for every file in A, B, C, D. Only difference: the thread number in the three
  stack-overflow lines of area C (an41_old_*), which changes every run (101276 vs 126072 etc.); the text and exit=127 match.
- B: `o22c_two_words_flat.em` has no line in the saved files; fresh output E3025, exit 1, as the report says.
- Every error code, warning (B o30c L3011) and exit code the reports state for a file equals the fresh one (B's summary
  table, E's check_summary, D and E partial pastes with "..." or "(two more errors follow)": m66 has 3 errors in total, an69_orphan 2, an76_module_assign 3, all as stated).
- Every "means exactly" file: exit=0, except `11-area-D/means/m66_shared_dyn.em`, which the report itself says is refused (E2020, E2072 twice).
- Not named in the report text (all exit=0, no claim to check): A m06_query_destroy, C an37_tr_refmut, D an68_match_cast, E a15_workaround4, a19_workaround.

Mismatches: none.
