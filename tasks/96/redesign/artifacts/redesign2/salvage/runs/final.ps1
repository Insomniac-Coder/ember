# final.ps1: the timing batch behind phase3b-salvage.md. 7 rounds per workload; in each round every
# program of the workload runs once, interleaved, pinned to the P-cores (ab.ps1).
$s = 'C:\Users\ism19\Code\ember\target\redesign2\salvage'; $b = "$s\bin"
function exes($names) { $names | ForEach-Object { "$b\$_.exe" } }
$w1 = 'plain_w1_group','rust_w1_group','d1_w1_group','d2_w1_group','d3_w1_group','d1_w1_group_proven','d1_w1_group_noinline','d3_w1_group_nobounds','plain_w1_arch','plain_w1_sparse','orig_D1_w1','orig_D2_w1','orig_D3_w1'
$w2 = 'plain_w2','plain_w2_emberlist','rust_w2','d1_w2','d2_w2','d3_w2','d1_w2_proven','d2_w2_checked','d3_w2_unproven','orig_D1_w2','orig_D2_w2','orig_D2_w2_listobj','orig_D3_w2'
$w3 = 'plain_w3','rust_w3','d1_w3','d2_w3','d3_w3','d1_w3_malloc','d1_w3_typeptr','orig_D1_w3','orig_D2_w3','orig_D2_w3_scan','orig_D3_w3'
$w4 = 'plain_w4','rust_w4','d1_w4','d2_w4','d3_w4','d1_w4_callee','d1_w4_oneclosure','d1_w4_array3','plain_w4_env32','orig_D1_w4','orig_D2_w4','orig_D2_w4_listobj','orig_D3_w4'
foreach ($w in @(@('final_w1', $w1), @('final_w4', $w4), @('final_w3', $w3), @('final_w2', $w2))) {
  $out = "$s\runs\$($w[0]).txt"
  if (Test-Path $out) { Remove-Item $out }
  & "$s\runs\ab.ps1" -Out $out -Rounds 7 -Exes (exes $w[1])
}
