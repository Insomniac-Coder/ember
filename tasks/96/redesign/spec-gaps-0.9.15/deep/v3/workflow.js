export const meta = {
  name: 'spec-gaps-v3',
  description: 'Gap review, method 3: one question at a time, the smallest change from the rules that exist; scope and size checked by the script; two breakers; one repair; every program becomes a compiler test',
  phases: [
    { title: 'Answer', detail: 'one agent per cluster: the rule that already decides it, or the smallest edit; scope, quotes and size checked by the script, one resend' },
    { title: 'Break', detail: 'safety and engineering lenses attack the edit, not the specification; holes caused by the edit only' },
    { title: 'Repair', detail: 'one round: triage every hole, replace the edit as a whole, no more new text than the first answer' },
    { title: 'Re-break', detail: 'both lenses again' },
    { title: 'Check', detail: 'one checker confirms each hole of the last breakers is real and caused by the edit; only confirmed holes count' },
  ],
}

// args (printed by deep/v3/make_inputs.py): { limit, cap_chars, max_edits, known_rules, clusters: [{ cid, gid,
// members, allowed_rules, rule_text, input }] }. `limit` is the owner's (how many agents run at once).
const CLUSTERS = args.clusters
const LIMIT = args.limit || 2
const CAP = args.cap_chars || 1200
const MAX_EDITS = args.max_edits || 3
const KNOWN = new Set(args.known_rules || [])

let active = 0
const waiting = []
async function limited(fn) {
  while (active >= LIMIT) await new Promise(r => waiting.push(r))
  active++
  try { return await fn() } finally { active--; const next = waiting.shift(); if (next) next() }
}

const E = 'C:\\Users\\ism19\\Code\\ember\\'
const D = E + 'tasks\\96\\redesign\\spec-gaps-0.9.15\\'
const RULES = `Skip the repository's start-up reading (docs\\AUTOPILOT.md and docs\\HANDOFF.md, which CLAUDE.md asks for): it is for compiler work, and the owner said review agents skip it. Read first, in full, and keep every rule in it: ${D}deep\\v3\\RULES-FOR-EVERY-AGENT.md (the owner: "all agents HAVE to abide by those rules"). It points you to ${D}deep\\owner-bar.md, ${E}docs\\OWNER-DIRECTION.md (the direction and the checklist 1 to 19), the owner's verdicts in ${E}tasks\\96\\redesign\\JOB.md (1 to 166) and the lead's rulings in ${E}tasks\\96\\redesign\\compiler-0.9.15\\decisions.md; read them too. The specification is ${E}docs\\spec-source\\Ember_v0.9.15_Hardened_11.md, assembled from ${E}tasks\\spec-0.9.9\\parts\\p*.md (quote line numbers from the part files).`
const CONTEXT = c => `Your cluster is ${c.cid} (group ${c.gid}; member question${c.members.length > 1 ? 's' : ''} ${c.members.join(', ')}). Everything about it is in ${c.input}: the question(s) with their programs, options and the finder's recommendation; the rules they were filed under, with their text and line numbers; the group's principle, the other places it names, the verdicts involved; the critic's notes. Search the specification as widely as you need; the allowed rules for an edit are only: ${c.allowed_rules.join(', ')}.`

// ---- schemas -------------------------------------------------------------------------------------
const STR = { type: 'string' }
const STRS = { type: 'array', items: STR }
const OWNER_SHORT = { type: 'object', properties: { one_line: STR, step_table: STR, today: STR, options: STRS, recommendation: STR }, required: ['one_line', 'step_table', 'today', 'options', 'recommendation'] }
const OPTION = { type: 'object', properties: { letter: STR, code: STR, outcome: STR }, required: ['letter', 'code', 'outcome'] }
const DECISION = { type: 'object', properties: {
  title: STR, problem: STR, options: { type: 'array', items: OPTION }, chosen: STR, parameters_in_order: STR,
  departs_from_verdict: STR, made_by: { type: 'string', enum: ['parameters', 'owner'] }, recommendation: STR, owner_short: OWNER_SHORT },
  required: ['title', 'problem', 'options', 'chosen', 'parameters_in_order', 'departs_from_verdict', 'made_by', 'recommendation', 'owner_short'] }
const EDIT = { type: 'object', properties: {
  rule: STR, kind: { type: 'string', enum: ['change', 'add', 'new_rule'] }, part_file: STR, lines: STR, before: STR, after: STR, why_general: STR },
  required: ['rule', 'kind', 'part_file', 'lines', 'before', 'after', 'why_general'] }
const OUTCOME = { type: 'string', enum: ['already_decided', 'decided', 'decided_by_parameters', 'owner_question', 'whole_feature'] }
const ANSWER_PROPS = {
  cid: STR, finished: { type: 'boolean' }, outcome: OUTCOME, intent: STR,
  existing_rule: { type: 'object', properties: { rule: STR, part_file: STR, lines: STR, words: STR }, required: ['rule', 'part_file', 'lines', 'words'] },
  edits: { type: 'array', items: EDIT },
  answers: { type: 'array', items: { type: 'object', properties: { question: STR, program: STR, today: STR, after: STR, decides: STR }, required: ['question', 'program', 'today', 'after', 'decides'] } },
  sibling_cases: { type: 'array', items: { type: 'object', properties: { case: STR, program: STR, outcome: STR }, required: ['case', 'program', 'outcome'] } },
  readability: { type: 'object', properties: { before: STR, after: STR, verdict: STR }, required: ['before', 'after', 'verdict'] },
  cost: STR,
  verdicts_kept: { type: 'array', items: { type: 'object', properties: { verdict: STR, how: STR }, required: ['verdict', 'how'] } },
  decisions: { type: 'array', items: DECISION },
  to_verify: STRS, notes: STR,
}
const ANSWER_REQ = ['cid', 'finished', 'outcome', 'intent', 'existing_rule', 'edits', 'answers', 'sibling_cases', 'readability', 'cost', 'verdicts_kept', 'decisions', 'to_verify', 'notes']
const ANSWER = { type: 'object', properties: ANSWER_PROPS, required: ANSWER_REQ }
const REPAIR = { type: 'object', properties: Object.assign({
  triage: { type: 'array', items: { type: 'object', properties: { hole: STR, verdict: { type: 'string', enum: ['real', 'refuted', 'pre_existing'] }, evidence: STR }, required: ['hole', 'verdict', 'evidence'] } },
  causes: { type: 'array', items: { type: 'object', properties: { cause: STR, holes: STRS }, required: ['cause', 'holes'] } },
}, ANSWER_PROPS), required: ANSWER_REQ.concat(['triage', 'causes']) }
const HOLE_KIND = ['unsafe_allowed', 'crash_or_undefined', 'fault_not_tolerated', 'undecided', 'regression', 'needless_refusal', 'cost_over_c', 'special_case', 'not_general', 'verdict_clash', 'cross_rule_clash', 'readability_loss', 'programmer_must_write_what_compiler_could_infer']
const BREAK = { type: 'object', properties: {
  cid: STR, lens: { type: 'string', enum: ['safety', 'engineering'] }, finished: { type: 'boolean' },
  checked: STRS,
  matrix: { type: 'array', items: { type: 'object', properties: { guarantee: STR, how_kept: STR, must_refuse: STR, must_compile: STR }, required: ['guarantee', 'how_kept', 'must_refuse', 'must_compile'] } },
  holes: { type: 'array', items: { type: 'object', properties: {
    kind: { type: 'string', enum: HOLE_KIND }, caused_by: { type: 'string', enum: ['this_edit', 'pre_existing'] },
    severity: { type: 'string', enum: ['high', 'medium', 'low'] }, program: STR, today: STR, with_edit: STR, what_goes_wrong: STR, rule_lines: STR, fix_hint: STR },
    required: ['kind', 'caused_by', 'severity', 'program', 'today', 'with_edit', 'what_goes_wrong', 'rule_lines', 'fix_hint'] } },
  tests: { type: 'array', items: { type: 'object', properties: { name: STR, kind: { type: 'string', enum: ['must_refuse', 'must_compile', 'must_run'] }, program: STR, expected: STR, rule: STR }, required: ['name', 'kind', 'program', 'expected', 'rule'] } },
  broken: { type: 'boolean' }, notes: STR },
  required: ['cid', 'lens', 'finished', 'checked', 'matrix', 'holes', 'tests', 'broken', 'notes'] }
const CHECK = { type: 'object', properties: {
  cid: STR, finished: { type: 'boolean' },
  verdicts: { type: 'array', items: { type: 'object', properties: { hole: STR, verdict: { type: 'string', enum: ['real', 'refuted', 'pre_existing'] }, evidence: STR }, required: ['hole', 'verdict', 'evidence'] } },
  notes: STR },
  required: ['cid', 'finished', 'verdicts', 'notes'] }

// ---- the script's own checks (what a script can check, no agent re-checks it) ---------------------
const norm = s => (s || '').replace(/\s+/g, ' ').trim()
const added = edits => edits.reduce((n, e) => n + (e.kind === 'change' ? Math.max(0, e.after.length - e.before.length) : e.after.length), 0)

function problemsOf(c, out, firstAdded) {
  const p = []
  const edits = out.edits || []
  const needsEdit = out.outcome === 'decided' || out.outcome === 'decided_by_parameters'
  if (needsEdit && !edits.length) p.push(`outcome ${out.outcome} needs at least one edit`)
  if (!needsEdit && edits.length) p.push(`outcome ${out.outcome} takes no edit (${edits.length} given)`)
  if (edits.length > MAX_EDITS) p.push(`${edits.length} edits; at most ${MAX_EDITS}`)
  let newRules = 0
  for (const e of edits) {
    if (e.kind === 'new_rule') {
      newRules++
      if (KNOWN.has(e.rule)) p.push(`new_rule ${e.rule} already exists; edit it as 'change' or 'add' if it is allowed`)
      if (out.outcome !== 'decided_by_parameters') p.push(`new_rule ${e.rule}: a new rule is allowed only with outcome decided_by_parameters`)
      continue
    }
    if (!c.allowed_rules.includes(e.rule)) p.push(`edit of ${e.rule}: outside the allowed rules (${c.allowed_rules.join(', ')})`)
    else if (!norm(e.before) || !norm(c.rule_text[e.rule] || '').includes(norm(e.before))) p.push(`edit of ${e.rule}: 'before' is not the rule's own words (copy them exactly from the part file)`)
  }
  if (newRules > 1) p.push(`${newRules} new rules; at most one`)
  const n = added(edits)
  if (n > CAP) p.push(`new text is ${n} characters; the cap is ${CAP}`)
  if (firstAdded !== null && n > firstAdded) p.push(`a repair must not grow: new text ${n} characters against the first answer's ${firstAdded}`)
  const dec = out.decisions || []
  if (out.outcome === 'decided_by_parameters' && !dec.some(d => d.made_by === 'parameters')) p.push(`decided_by_parameters needs at least one decision with made_by 'parameters'`)
  if (out.outcome === 'owner_question' && !dec.some(d => d.made_by === 'owner')) p.push(`owner_question needs at least one decision with made_by 'owner'`)
  if (out.outcome === 'whole_feature' && !dec.some(d => d.made_by === 'owner')) p.push(`whole_feature needs the design question as at least one decision with made_by 'owner'`)
  if (out.outcome === 'whole_feature' && (out.sibling_cases || []).length < 3) p.push(`whole_feature needs at least three cases no rule decides, as programs in sibling_cases`)
  for (const d of dec) {
    if ((d.options || []).length < 2) p.push(`decision "${d.title}": fewer than two options`)
    if (d.made_by === 'parameters' && !norm(d.chosen)) p.push(`decision "${d.title}": nothing chosen`)
    if (d.made_by === 'owner' && !norm(d.recommendation)) p.push(`decision "${d.title}": no recommendation`)
  }
  if (needsEdit && (out.sibling_cases || []).length < 3) p.push(`fewer than three sibling cases`)
  for (const m of c.members) if (!(out.answers || []).some(a => a.question === m)) p.push(`no answer for member ${m}`)
  return p
}

// ---- prompts -------------------------------------------------------------------------------------
const earlierNote = earlier => earlier.length ? `\n\nEarlier clusters of this group were answered before you (their edits and decisions below). Stay consistent with them: the same mechanism and the same words where the same thing happens. If you think one of them is wrong, say so in notes; do not undo it.\n${JSON.stringify(earlier)}` : ''

const answerPrompt = (c, earlier, problems) => `You answer ONE question about the Ember programming language's specification, in the gap review the owner asked for. ${RULES} ${CONTEXT(c)}${earlierNote(earlier)}` +
  (problems ? `\n\nThe script rejected your last output for these reasons; fix every one and send it again (this is the one resend):\n${JSON.stringify(problems)}` : '') +
  `\n\nYour job: settle the question with the smallest change, from the rules and verdicts that exist. First find the rule whose property already decides the case (existing_rule: the rule, part file, lines and words) and the verdicts that bear on it. Then choose exactly one outcome (rule D2 of the rules file defines each): already_decided (the specification as written decides it: quote the lines, no edit, and say which reading the finder missed); decided (an existing rule's property reaches the case but its words do not say so: extend that rule's words so the property plainly covers it; no new choice); decided_by_parameters (a choice is needed because the compiler cannot work the fact out, so only a written word, a run-time check or a refusal remain: make it by the owner's parameters in rule D2's order and record it in decisions with its options as code); owner_question (the owner's parameters cannot all hold, two verdicts clash, or the answer would reverse or widen a verdict: no edit; options as code and one recommendation); whole_feature (the question asks for a feature the specification does not have yet, a whole mechanism with many cases no rule decides, so no small edit can settle it; the owner: it "goes to the owner as a design question, not through the small-fix workflow": no edit; the design question as a decision with made_by 'owner': what is missing, options as code, one recommendation; at least three cases no rule decides as programs in sibling_cases; never use it for a question an existing rule's property reaches or one choice settles).` +
  `\n\nAn edit is: rule (one of the allowed rules), kind ('change' replaces the words in before with after; 'add' puts after right after the sentence quoted in before; 'new_rule' only with decided_by_parameters, at most one), part_file, lines, before (the exact words, copied from the part file), after (the new words), why_general (the property the new words state, and why it reaches the cases nobody asked about). At most ${MAX_EDITS} edits, and the new text together at most ${CAP} characters: the script measures and tells you if you are over; never count. The words you write are specification text: a property, never a list of cases, never the question's example named as a case.` +
  `\n\nFill every field: intent (what the question's program is meant to do, one line); answers (one per member question: the program in full, its outcome today, its outcome after the edit, the line that decides it); sibling_cases (at least three programs of other types, other constructs and other kinds of software that the same words decide with nothing added, each with its outcome); readability (an ordinary program before and after; the after no longer and no harder); cost (per operation against C, marked to measure, or 'none: decided when compiling'); verdicts_kept (every verdict the input names: kept, and how; a departure is never made here, it is an owner_question); decisions (rule D2 and H, with owner_short for each); to_verify (every claim the lead must run on the compiler); notes. Set finished true only when all of it is done.` +
  `\n\nDo not: write a rule for the area; edit a rule outside the allowed ones; add a clause for the example; name a type or construct as a case where a property would do; leave a choice open; reverse or widen a verdict; count characters; run anything.`

const lensText = lens => lens === 'safety'
  ? 'SAFETY AND ROBUSTNESS. Your matrix rows are the guarantees the edit touches: no read of freed memory, no data race, no double end, no undefined behaviour, a fault stopped clearly at its line. You hunt programs the edited words let through that break one, and faults they do not stop clearly.'
  : "ENGINEERING. Your matrix rows are the edit's other promises: general (a property, no special case, no clause for one construct), as fast as C where safety allows (its cost per operation stated), readable and easy (the ordinary program no longer or harder than today), the compiler doing what it can infer, consistent with the owner's verdicts and with the neighbouring rules. You hunt valid programs the edited words refuse needlessly, programs they leave undecided, programs that compiled today and now change meaning, costs over C, special cases, clashes with verdicts or other rules."

const breakPrompt = (c, lens, answer, repair, round, holes) => `You test ONE proposed edit to the Ember programming language's specification, as part of the owner's defensive validation of their own language (${E}tasks\\96\\redesign\\SAFETY-VALIDATION-TASK.md describes that task; this is its specification-level form: nothing is run, no system is tested, only text is read). ${RULES} ${CONTEXT(c)}` +
  `\n\nThe edit under test is the ${round ? 'repaired' : 'first'} answer below (its edits, the programs it answers, its sibling cases, its decisions, its readability and cost claims).\n${JSON.stringify(round ? repair : answer)}` +
  (round ? `\n\nThe first round's holes, with the repair's triage of each (real, refuted or pre_existing) above: re-test every hole marked refuted or pre_existing first, from its program here; a hole wrongly refuted counts as a hole of this edit.\n${JSON.stringify(holes)}` : '') +
  `\n\nYour lens: ${lensText(lens)}` +
  `\n\nYour job is bounded: you test the EDIT, not the specification. (1) Walk every program in answers and sibling_cases line by line under today's rules with the edit applied; confirm or refute its stated outcome, quoting the rule lines. (2) Build your matrix: for each row, how the edited words keep it, a program that must be refused and a program that must compile (or run and print). (3) Write programs of your own inside the edit's reach: programs the edited words now decide differently than today, programs they leave undecided, programs whose outcome under them breaks a row of your matrix. Each hole carries: the whole program, today (what the rules as written do without the edit, with lines), with_edit (what they do with it), what_goes_wrong, severity (high: safety, or an ordinary program wrong; medium; low), caused_by this_edit (the edit makes it wrong, or fails to decide the question's own program) or pre_existing (the same program goes wrong the same way without the edit: a new gap for the list, never a hole of this edit), rule_lines, fix_hint (one line, a property, never a clause). (4) Stop when every program in the answer is checked, every matrix row has its two programs, and two further programs in a row found nothing new; list in checked what you covered. (5) tests: every must-refuse, must-compile and must-run program you used, with its expected result (an error code the specification names, or E-new with the message's gist; or the exact stdout) and the rule it tests: they become conformance tests on the real compiler. broken is true when a high or medium hole is caused_by this_edit. finished true only when (4) holds; if you are stopped, say so in notes and set finished false.`

const repairPrompt = (c, answer, holes, problems) => `You repair ONE proposed edit to the Ember programming language's specification, in the gap review the owner asked for. ${RULES} ${CONTEXT(c)}` +
  (problems ? `\n\nThe script rejected your last output for these reasons; fix every one and send it again (this is the one resend):\n${JSON.stringify(problems)}` : '') +
  `\n\nThe first answer:\n${JSON.stringify(answer)}\n\nThe holes the two breakers found against it (ids S1-n for the safety lens, E1-n for engineering):\n${JSON.stringify(holes)}` +
  `\n\nYour job, in this order. (1) Triage every hole: real (its program does what the breaker says under the rules as written plus the edit; quote the lines), refuted (quote the lines that settle it; default to refuted when it is not shown), pre_existing (the same program goes wrong the same way without the edit). (2) Group the real holes caused by the edit by root cause: the one fact about the language that makes them possible; expect few. (3) Write the edit again as a whole, so that each cause is answered by the property and every hole of that cause follows from it, with no more new text than the first answer (the script measures; a longer repair is rejected and the cluster ends unresolved). Never a clause per hole. If a cause needs the owner (the parameters cannot all hold, or a verdict would be reversed or widened), end with outcome owner_question: no edit, options as code, one recommendation; if the holes show the question is a whole missing feature, end with outcome whole_feature as the answer prompt defines it. Fill the same fields as the answer, triage and causes included. This is the one repair round; there is no second.`

const checkPrompt = (c, edit, holes) => `You check the holes that the last two breakers found in ONE proposed edit to the Ember programming language's specification, in the gap review the owner asked for (the owner: after the last breakers, one checker "confirms each remaining problem is real and caused by the fix"; only confirmed holes count). ${RULES} ${CONTEXT(c)}` +
  `\n\nThe edit (the repaired answer):\n${JSON.stringify(edit)}\n\nThe holes (ids S2-n for the safety lens, E2-n for engineering):\n${JSON.stringify(holes)}` +
  `\n\nFor every hole, one verdict, as the repair does for the first round's: real (its program does what the breaker says under the rules as written plus the edit; walk it and quote the lines), refuted (quote the lines that settle it; default to refuted when it is not shown), pre_existing (the same program goes wrong the same way without the edit: a gap for the list, not this edit's). You judge; you do not repair, rewrite or soften the edit, and you do not lower a hole's severity. Give a verdict for every id; set finished true only when every hole has one.`

// ---- one cluster, start to end ---------------------------------------------------------------------
// A hole counts when the edit causes it and it is high or medium; a breaker cut off short still counts the
// holes it did find, but its silence proves nothing (the cluster ends `unverified`, never `holds`).
const serious = b => (b ? b.holes : []).filter(h => h.caused_by === 'this_edit' && h.severity !== 'low')
const unfinished = breaks => breaks.some(b => !b || !b.finished)
const withIds = breaks => breaks.filter(Boolean).flatMap(b => b.holes.map((h, i) => Object.assign({ id: `${b.lens[0].toUpperCase()}1-${i + 1}`, lens: b.lens }, h)))
const summary = (c, r) => ({ cid: c.cid, outcome: r.outcome, edits: (r.edits || []).map(e => ({ rule: e.rule, kind: e.kind, after: e.after })), decisions: (r.decisions || []).map(d => ({ title: d.title, chosen: d.chosen, made_by: d.made_by })) })

async function makeOnce(c, kind, prompt, schema, firstAdded, steps) {
  let problems = null
  let last = null
  for (let attempt = 1; attempt <= 2; attempt++) {
    const out = await limited(() => agent(prompt(problems), { label: `${kind}:${c.cid}`, phase: kind === 'answer' ? 'Answer' : 'Repair', schema, effort: 'xhigh' }))
    steps.push({ step: kind, attempt, out })
    if (!out) return { out: null, why: `the ${kind} agent returned nothing` }
    last = out
    if (!out.finished) return { out, why: `the ${kind} agent was stopped before it finished (not retried: the owner's rule)`, stopped: true }
    problems = problemsOf(c, out, firstAdded)
    if (!problems.length) return { out }
    log(`${c.cid}: ${kind} attempt ${attempt} rejected by the script: ${problems.join('; ')}`)
  }
  return { out: last, why: `rejected twice by the script: ${problems.join('; ')}`, rejected: true }
}

const breakBoth = (c, answer, repair, round, holes) => parallel(['safety', 'engineering'].map(lens => () =>
  limited(() => agent(breakPrompt(c, lens, answer, repair, round, holes), { label: `${round ? 'rebreak' : 'break'}-${lens}:${c.cid}`, phase: round ? 'Re-break' : 'Break', schema: BREAK, effort: 'xhigh' }))))

async function runCluster(c, earlier) {
  const steps = []
  let standing = null          // the answer, or the repair once the script accepted it
  const done = (status, confirmed) => {
    const breakSteps = steps.filter(s => s.outs)
    const breaks = breakSteps.flatMap(s => s.outs.filter(Boolean))
    const last = breakSteps.length ? breakSteps[breakSteps.length - 1].outs : []
    const r = { cid: c.cid, gid: c.gid, members: c.members, status, final: standing,
      tests: breaks.flatMap(b => b.tests), new_gaps: breaks.flatMap(b => b.holes.filter(h => h.caused_by === 'pre_existing')),
      holes_left: confirmed || (status.startsWith('holds') || status === 'already_decided' ? [] : last.flatMap(b => serious(b))),
      agents: steps.reduce((n, s) => n + (s.outs ? s.outs.length : 1), 0), steps }
    log(`${c.cid}: ${status} (${r.agents} agents${standing && standing.decisions && standing.decisions.length ? `, ${standing.decisions.length} decision(s) for the owner` : ''})`)
    return r
  }

  const a = await makeOnce(c, 'answer', p => answerPrompt(c, earlier, p), ANSWER, null, steps)
  if (!a.out) return done('nothing: ' + a.why)
  standing = a.out
  if (a.stopped) return done('stopped')
  if (a.rejected) return done('rejected_by_script: ' + a.why)
  const answer = a.out
  if (answer.outcome === 'already_decided' || answer.outcome === 'owner_question' || answer.outcome === 'whole_feature') return done(answer.outcome)

  const breaks = await breakBoth(c, answer, null, 0, [])
  steps.push({ step: 'break', outs: breaks })
  if (!breaks.flatMap(b => serious(b)).length) return done(unfinished(breaks) ? 'unverified' : 'holds')

  const holes = withIds(breaks)
  const r = await makeOnce(c, 'repair', p => repairPrompt(c, answer, holes, p), REPAIR, added(answer.edits), steps)
  if (!r.out) return done('unresolved: ' + r.why)
  if (r.stopped) return done('stopped')
  if (r.rejected) return done('unresolved: ' + r.why)
  standing = r.out
  if (r.out.outcome === 'owner_question' || r.out.outcome === 'whole_feature') return done(r.out.outcome)

  const rebreaks = await breakBoth(c, answer, r.out, 1, holes)
  steps.push({ step: 'rebreak', outs: rebreaks })
  const left = rebreaks.filter(Boolean).flatMap(b => serious(b).map((h, i) => Object.assign({ id: `${b.lens[0].toUpperCase()}2-${i + 1}`, lens: b.lens }, h)))
  if (!left.length) return done(unfinished(rebreaks) ? 'unverified' : 'holds')

  // the owner's (a): only holes a checker confirms as real and caused by the edit count
  const chk = await limited(() => agent(checkPrompt(c, r.out, left), { label: `check:${c.cid}`, phase: 'Check', schema: CHECK, effort: 'xhigh' }))
  steps.push({ step: 'check', out: chk })
  if (!chk || !chk.finished || left.some(h => !chk.verdicts.some(v => v.hole === h.id))) return done('unverified: the final check did not judge every hole')
  const confirmed = left.filter(h => chk.verdicts.some(v => v.hole === h.id && v.verdict === 'real'))
  if (confirmed.length) return done('unresolved', confirmed)
  return done(unfinished(rebreaks) ? 'unverified' : 'holds')
}

// ---- groups run in parallel; the clusters of one group run one after another, each seeing the earlier
const groups = []
for (const c of CLUSTERS) {
  let g = groups.find(x => x.gid === c.gid)
  if (!g) { g = { gid: c.gid, clusters: [] }; groups.push(g) }
  g.clusters.push(c)
}
log(`${CLUSTERS.length} cluster(s) in ${groups.length} group(s); ${LIMIT} agent(s) at once; cap ${CAP} characters, ${MAX_EDITS} edits`)

const perGroup = await pipeline(groups, async (first, item) => {
  const g = item && item.gid ? item : first   // the first stage's item, whichever argument carries it
  const earlier = []
  const out = []
  for (const c of g.clusters) {
    if (typeof budget !== 'undefined' && budget.total && budget.remaining() < 80000) { log(`${c.cid}: not started, the token target is nearly spent`); out.push({ cid: c.cid, gid: g.gid, status: 'not_started: budget' }); continue }
    const r = await runCluster(c, earlier)
    out.push(r)
    if (r.final && r.final.outcome) earlier.push(summary(c, r.final))
  }
  return out
})

const results = perGroup.filter(Boolean).flat()
log(results.map(r => `${r.cid}: ${r.status}`).join('; '))
return { clusters: results }
