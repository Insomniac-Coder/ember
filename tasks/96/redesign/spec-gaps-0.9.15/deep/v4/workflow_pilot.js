export const meta = {
  name: 'spec-gaps-v4-team',
  description: 'Gap review, method 4: a team per question (designer, safety adversary, language engineer, spec integrator, arbiter); the fix goes to its cause with its impact; up to three rounds of review and revision; only a safety hole, an undecided question or a new contradiction blocks',
  phases: [
    { title: 'Design', detail: 'the designer finds the cause, wherever it is, and writes the change with its impact report' },
    { title: 'Review', detail: 'safety adversary, language engineer and spec integrator attack the change' },
    { title: 'Revise', detail: 'the designer answers every objection: fixed, rebutted, or a trade-off kept with its reasoning' },
    { title: 'Arbitrate', detail: 'an engineer from outside the debate judges every open objection and every trade-off' },
  ],
}

// Pilot resume (K07, K20, K10): round 1 replays from the cache; from round 2 the cheaper two-reviewer team.
const PILOT_ARGS = {"limit": 200, "cap_chars": 1200, "max_edits": 3, "clusters": [{"cid": "K07-raw-pointer-struct-safe-call", "gid": "P04", "members": ["J8-08"], "allowed_rules": ["FFI-10", "FFI-11", "FFI-30a", "FFI-56"], "input": "C:\\Users\\ism19\\Code\\ember\\tasks\\96\\redesign\\spec-gaps-0.9.15\\deep\\v3\\inputs\\K07-raw-pointer-struct-safe-call.md"}, {"cid": "K20-anyerror-send", "gid": "P01", "members": ["J6-16"], "allowed_rules": ["ERR-3", "ERR-8", "THR-2", "THR-8"], "input": "C:\\Users\\ism19\\Code\\ember\\tasks\\96\\redesign\\spec-gaps-0.9.15\\deep\\v3\\inputs\\K20-anyerror-send.md"}, {"cid": "K10-queued-drop-check-points", "gid": "P05", "members": ["J7-01", "J7-02"], "allowed_rules": ["BRW-18", "EFF-24", "FFI-22", "HR-3a", "JOB-2", "RC-3", "RC-4", "THR-16", "THR-19", "THR-20", "WK-3"], "input": "C:\\Users\\ism19\\Code\\ember\\tasks\\96\\redesign\\spec-gaps-0.9.15\\deep\\v3\\inputs\\K10-queued-drop-check-points.md"}], "first_round_roles": ["safety", "engineering", "integration"], "strict_members": true}

// args (from deep/v3/make_inputs.py, written into the run's script by code): { limit, clusters: [{ cid, gid,
// members, allowed_rules (where the problem was reported; not a limit here), rule_text, input }] }
const CLUSTERS = PILOT_ARGS.clusters
const LIMIT = PILOT_ARGS.limit || 200
const ROUNDS = 3
const FIRST_ROUND = PILOT_ARGS.first_round_roles || ['safety', 'engineering']   // the cheaper team: two reviewers
const STRICT_MEMBERS = !!PILOT_ARGS.strict_members

let active = 0
const waiting = []
async function limited(fn) {
  while (active >= LIMIT) await new Promise(r => waiting.push(r))
  active++
  try { return await fn() } finally { active--; const next = waiting.shift(); if (next) next() }
}

const E = 'C:\\Users\\ism19\\Code\\ember\\'
const D = E + 'tasks\\96\\redesign\\spec-gaps-0.9.15\\'
const RULES = `Read first, in full, and keep every rule in it: ${D}deep\\v4\\RULES-FOR-THE-TEAM.md. Read ${E}docs\\OWNER-DIRECTION.md (the goal, the principles, the checklist) and the owner's verdicts in ${E}tasks\\96\\redesign\\JOB.md (1 to 166) as the work needs them. Skip the repository's start-up reading (docs\\AUTOPILOT.md, docs\\HANDOFF.md): it is for compiler work.`
const CONTEXT = c => `The question is cluster ${c.cid} (member question${c.members.length > 1 ? 's' : ''} ${c.members.join(', ')}). Everything about it is in ${c.input}: the question(s) with their programs and options, the rules where the problem was reported (with their text and lines), the area's principle, the verdicts involved, the critic's notes. The problem was reported in: ${c.allowed_rules.join(', ') || '(no rule named)'}; the cause may be elsewhere. The specification is ${E}docs\\spec-source\\Ember_v0.9.15_Hardened_11.md, assembled from ${E}tasks\\spec-0.9.9\\parts\\p*.md (quote the part files' lines); search all of it.`
const TEAM = 'You are one of a team of seasoned engineers who have shipped C++, Rust and Swift language features and just came together to make Ember: as fast as C, as memory-safe as Rust or safer, as easy to read as Python. Hard on the work, fair to each other: every objection is a whole program and a walk through the rules; every answer is a fix or a rebuttal that quotes the rules; when the other side is right, you say so.'

// ---- schemas -------------------------------------------------------------------------------------
const STR = { type: 'string' }
const STRS = { type: 'array', items: STR }
const obj = (props, req) => ({ type: 'object', properties: props, required: req || Object.keys(props) })
const OWNER_SHORT = obj({ one_line: STR, step_table: STR, today: STR, options: STRS, recommendation: STR })
const OPTION = obj({ letter: STR, code: STR, outcome: STR })
const DECISION = obj({ title: STR, problem: STR, options: { type: 'array', items: OPTION }, chosen: STR, parameters_in_order: STR,
  departs_from_verdict: STR, made_by: { type: 'string', enum: ['parameters', 'owner'] }, recommendation: STR, owner_short: OWNER_SHORT })
const TEST = obj({ name: STR, kind: { type: 'string', enum: ['must_refuse', 'must_compile', 'must_run'] }, program: STR, expected: STR, rule: STR })
const CHANGE = obj({
  cid: STR, finished: { type: 'boolean' },
  outcome: { type: 'string', enum: ['decided', 'decided_by_parameters', 'already_decided', 'owner_question', 'whole_feature'] },
  intent: STR,
  cause: obj({ where: STR, why_there: STR, reported_at: STR }),
  existing_rule: obj({ rule: STR, part_file: STR, lines: STR, words: STR }),
  edits: { type: 'array', items: obj({ rule: STR, kind: { type: 'string', enum: ['change', 'add', 'new_rule'] }, part_file: STR, lines: STR, before: STR, after: STR, why_general: STR, why_here: STR }) },
  impact: obj({ rules_changed: STR, programs_changed: { type: 'array', items: obj({ program: STR, before: STR, after: STR }) },
    readability: obj({ before: STR, after: STR, verdict: STR }), cost: STR,
    verdicts: { type: 'array', items: obj({ verdict: STR, kept_or_departs: { type: 'string', enum: ['kept', 'departs'] }, how: STR }) } }),
  answers: { type: 'array', items: obj({ question: STR, program: STR, today: STR, after: STR, decides: STR }) },
  sibling_cases: { type: 'array', items: obj({ case: STR, program: STR, outcome: STR }) },
  decisions: { type: 'array', items: DECISION },
  responses: { type: 'array', items: obj({ objection: STR, answer: { type: 'string', enum: ['fixed', 'rebutted', 'tradeoff_kept'] }, how: STR }) },
  tests: { type: 'array', items: TEST },
  to_verify: STRS, notes: STR,
})
const BLOCK = { type: 'string', enum: ['S', 'U', 'X', 'none'] }
const KIND = { type: 'string', enum: ['unsafe_allowed', 'crash_or_undefined', 'fault_not_tolerated', 'safety_guarantee_removed', 'question_undecided', 'contradiction', 'needless_refusal', 'readability_loss', 'cost_over_c', 'programmer_must_write_what_compiler_could_infer', 'special_case', 'not_general', 'verdict_clash', 'meaning_changed', 'impact_report_wrong', 'other'] }
const REVIEW = obj({
  cid: STR, role: { type: 'string', enum: ['safety', 'engineering', 'integration'] }, finished: { type: 'boolean' }, checked: STRS,
  previous: { type: 'array', items: obj({ objection: STR, status: { type: 'string', enum: ['resolved', 'withdrawn', 'open'] }, why: STR }) },
  objections: { type: 'array', items: obj({ block: BLOCK, kind: KIND, severity: { type: 'string', enum: ['high', 'medium', 'low'] }, caused_by_change: { type: 'boolean' },
    program: STR, walk: STR, impact: STR, suggestion: STR }) },
  tests: { type: 'array', items: TEST }, notes: STR,
})
const ARBITER = obj({
  cid: STR, finished: { type: 'boolean' },
  rulings: { type: 'array', items: obj({ objection: STR, verdict: { type: 'string', enum: ['real', 'refuted', 'pre_existing'] }, blocks: { type: 'boolean' }, evidence: STR }) },
  tradeoffs: { type: 'array', items: obj({ objection: STR, justified: { type: 'boolean' }, why: STR, owner_must_see: { type: 'boolean' } }) },
  reclassify: { type: 'string', enum: ['none', 'whole_feature', 'owner_question'] },
  summary: STR, owner_short: OWNER_SHORT, notes: STR,
})

// ---- what a script can check (no agent re-checks it) --------------------------------------------
const CODE_LINE = /^\s*(import|from|fn|gen|class|struct|enum|interface|static|type|extern|unsafe|pub|const|@)\b/m
const DESCRIPTION = /verbatim|\bas in (the answer|round|r\d|the first)|\bappears?\b|\bno freeze\b|\bonly after\b|\bevery line\b|\bthe last line\b|[a-z]\. [A-Z]/i
const testProblem = t => (!CODE_LINE.test(t.program || '') || /verbatim|\bas in (the answer|round|r\d|the first)/i.test(t.program || '')) ? `test ${t.name}: the program is a description, not code`
  : (t.kind === 'must_run' && DESCRIPTION.test(t.expected || '')) ? `test ${t.name}: the expected output is a description, not the exact text` : null

function formProblems(c, d, open) {
  const p = []
  const edits = d.edits || []
  const needsEdit = d.outcome === 'decided' || d.outcome === 'decided_by_parameters'
  if (needsEdit && !edits.length) p.push(`outcome ${d.outcome} needs at least one edit`)
  if (!needsEdit && edits.length) p.push(`outcome ${d.outcome} takes no edit`)
  for (const e of edits) {
    if (!(e.rule || '').trim() || !(e.after || '').trim()) p.push(`an edit without its rule or its new words`)
    if (e.kind !== 'new_rule' && !(e.before || '').trim()) p.push(`edit of ${e.rule}: 'before' must quote the words it replaces or follows`)
    if (!(e.why_here || '').trim()) p.push(`edit of ${e.rule}: say why the cause is in this rule (why_here)`)
  }
  const dec = d.decisions || []
  if (d.outcome === 'decided_by_parameters' && !dec.some(x => x.made_by === 'parameters' && (x.chosen || '').trim())) p.push('decided_by_parameters needs a decision made by the parameters, with the option chosen')
  if ((d.outcome === 'owner_question' || d.outcome === 'whole_feature') && !dec.some(x => x.made_by === 'owner' && (x.recommendation || '').trim())) p.push(`${d.outcome} needs the question for the owner as a decision with made_by 'owner' and a recommendation`)
  if ((needsEdit || d.outcome === 'whole_feature') && (d.sibling_cases || []).length < 3) p.push('at least three sibling cases (other types, constructs and kinds of software)')
  // an answer names its member question, often followed by its text ("J8-08: ..."); the pilot's resume keeps
  // the exact match its first run used, so that run's finished agents replay from the cache
  const names = (a, m) => STRICT_MEMBERS ? a.question === m : new RegExp('^\\s*' + m + '(?![0-9])').test(a.question || '')
  for (const m of c.members) if (!(d.answers || []).some(a => names(a, m))) p.push(`no answer for member ${m}`)
  for (const t of d.tests || []) { const q = testProblem(t); if (q) p.push(q) }
  // a response names its objection by id, often followed by a description ("S1-7: ..."); "S1-1" must not match "S1-10"
  for (const o of open || []) if (!(d.responses || []).some(r => new RegExp('^\\s*' + o.id + '(?![0-9])').test(r.objection || ''))) p.push(`no response to objection ${o.id}`)
  return p
}

// ---- prompts -------------------------------------------------------------------------------------
const earlierNote = earlier => earlier.length ? `\n\nEarlier questions of this area were answered by the team before you (their changes below). Stay consistent with them; if one is wrong, say so in notes.\n${JSON.stringify(earlier)}` : ''

const designPrompt = (c, earlier, previous, open, problems) => `${TEAM} You are the question's lead designer. ${RULES} ${CONTEXT(c)}${earlierNote(earlier)}` +
  (previous ? `\n\nYour change of the last round:\n${JSON.stringify(previous)}\n\nThe objections still open against it (answer every one in responses, by its id: fixed, rebutted with the rule lines, or tradeoff_kept with the reasoning and its impact; a blocking one, block S, U or X, can only be fixed or rebutted):\n${JSON.stringify(open)}\n\nRewrite the change as a whole: no clause per objection. If the debate shows the cause is somewhere else, move the change there and say why.` : '') +
  (problems ? `\n\nThe script rejected your last output for these reasons; fix every one (this is the one resend):\n${JSON.stringify(problems)}` : '') +
  `\n\nYour job: find the cause of the problem, wherever it is in the specification (the question names where it showed), and settle it with the smallest change at the cause, from the rules and verdicts that exist (rules D of the team file). Fill: outcome; intent (what the question's program is meant to do, one line); cause (where it is, why there, where it was reported); existing_rule (the rule whose property decides it, with lines and words); edits (rule, kind, part file, lines, before = the exact words, after, why_general, why_here); impact (every rule changed, every program whose outcome changes before and after, readability before and after for an ordinary program, cost per operation against C, every verdict kept or departed from: a departure is listed for the owner, never applied silently); answers (one per member question: the whole program, today, after, the line that decides it); sibling_cases (at least three: other types, constructs and kinds of software, decided by the same words with nothing added); decisions (made by the owner's parameters in their order, recorded with options as code); tests (whole programs, exact expected results); to_verify (what the lead must run on the compiler). Set finished true only when all of it is done.`

const ROLE = {
  safety: 'You are the team\'s safety engineer: you have found use-after-free, data races and undefined behaviour in C++ engines and in unsafe Rust. Your only job is to break the change\'s safety: find every program the changed specification lets through that reads freed memory, races, ends a value twice, has undefined behaviour or faults without a clear stop at its line, and every safety guarantee of today\'s specification the change takes away. Such a finding is block S. Walk each program line by line through the rules as changed, quoting the lines; a program that goes wrong the same way without the change is caused_by_change false (a gap for the list).',
  engineering: 'You are the team\'s compiler and language-experience engineer: you have built a borrow checker and kept a large codebase readable. Your job is everything that is not safety: correct programs the change refuses, readability of an ordinary program, cost over C, words the programmer must write that the compiler could work out, special cases and lists where a property would do, clashes with the owner\'s verdicts, programs whose meaning changes. These are weighed, not blocking: block none, unless you find a safety hole (then block S). Give each its impact and the fix you suggest.',
  integration: 'You are the team\'s specification editor: you have kept a standard of a thousand rules consistent. Your job is the change\'s effect on the rest of the specification: grep every rule that cites, or is cited by, each changed rule and check it still holds and that no two rules now decide one program differently (block X); check that each edit\'s before words are really in that part file at those lines; check that the question\'s own program is decided by the changed words (block U if not); check that the impact report is complete and true (impact_report_wrong, block none). Name each rule with its part file and lines.',
}

// The cheaper team (owner: "MAKE IT CHEAPER"): after the first round, two reviewers: the safety engineer, and
// one engineer who covers both the engineering and the specification duties (returning role 'engineering').
const ROLE_COMBINED = `You are the team's compiler, language-experience and specification engineer: you have built a borrow checker, kept a large codebase readable and kept a standard of a thousand rules consistent. Your job is everything that is not safety. (1) The specification: grep every rule that cites, or is cited by, each changed rule and check it still holds and that no two rules now decide one program differently (block X); check that each edit's before words are really in that part file at those lines; check that the question's own program is decided by the changed words (block U if not); check that the impact report is complete and true. (2) The engineering: correct programs the change refuses, readability of an ordinary program, cost over C, words the programmer must write that the compiler could work out, special cases and lists where a property would do, clashes with the owner's verdicts, programs whose meaning changes: these are weighed, block none (unless you find a safety hole: then block S). Name each rule with its part file and lines; give each objection its impact and the fix you suggest.`
const COMBINED_ROLES = ['engineering', 'integration']
const ownsObjection = (role, combined, o) => combined ? COMBINED_ROLES.includes(o.role) : o.role === role

const reviewPrompt = (c, role, design, open, round, combined) => `${TEAM} ${combined ? ROLE_COMBINED : ROLE[role]} ${RULES} ${CONTEXT(c)}` +
  `\n\nThe change under review (round ${round}):\n${JSON.stringify(design)}` +
  (open.length ? `\n\nThe objections still open from earlier rounds, with the designer's responses in the change above. In previous, give each of the ones ${combined ? 'listed here' : `your role raised (ids starting ${role[0].toUpperCase()})`} a status: resolved (the change fixes it), withdrawn (the rebuttal is right), or open (say why, with the program).\n${JSON.stringify(open.filter(o => ownsObjection(role, combined, o)))}` : '') +
  `\n\nThen list your new objections: each with block (S, U, X or none, by rule C of the team file), kind, severity, caused_by_change, the whole program, the walk through the rules (part file, lines, words), its impact, and the fix you suggest (a property, never a clause). Add tests: every must-refuse, must-compile and must-run program you used, whole, with its exact expected result. List in checked what you covered; set finished true only when you checked the whole change against your job.`

const arbiterPrompt = (c, design, open) => `${TEAM} You are the arbiter: a senior engineer from outside this debate, trusted by both sides. ${RULES} ${CONTEXT(c)}` +
  `\n\nThe final change (with the designer's responses):\n${JSON.stringify(design)}\n\nThe objections still open:\n${JSON.stringify(open)}` +
  `\n\nJudge every open objection in rulings: real (walk its program through the rules as changed and quote the lines), refuted (quote what settles it; when it is not shown, refuted), or pre_existing (the same program goes wrong the same way without the change); blocks is true only for a real objection of block S, U or X. For every objection that does not block, judge the trade-off in tradeoffs: is the designer's reasoning justified by the common goal (rule B), and must the owner see it (a verdict departure, an ordinary program refused or made harder, a cost over C). Reclassify only if the question is really a whole missing feature or needs the owner. Write summary and owner_short (rule F) for the owner. You do not repair the change and you do not hunt for new problems.`

// ---- one question ------------------------------------------------------------------------------
async function makeDesign(c, earlier, previous, open, steps) {
  let problems = null, last = null
  for (let attempt = 1; attempt <= 2; attempt++) {
    const d = await limited(() => agent(designPrompt(c, earlier, previous, open, problems), { label: `${previous ? 'revise' : 'design'}:${c.cid}`, phase: previous ? 'Revise' : 'Design', schema: CHANGE, effort: 'xhigh' }))
    steps.push({ step: previous ? 'revise' : 'design', attempt, out: d })
    if (!d) return { why: 'the designer returned nothing' }
    last = d
    if (!d.finished) return { d, why: 'the designer was stopped before finishing (not retried)' }
    problems = formProblems(c, d, open)
    if (!problems.length) return { d, ok: true }
    log(`${c.cid}: ${previous ? 'revision' : 'design'} attempt ${attempt} rejected by the script: ${problems.join('; ')}`)
  }
  return { d: last, why: `rejected twice by the script: ${problems.join('; ')}` }
}

async function runCluster(c, earlier) {
  const steps = []
  let design = null
  let open = []
  const closed = []
  const gaps = []              // problems the reviewers found that exist without the change: listed for later
  const done = (status, extra) => {
    const tests = steps.flatMap(s => (s.out ? [s.out] : s.outs || []).filter(Boolean).flatMap(x => x.tests || []))
    const r = Object.assign({ cid: c.cid, gid: c.gid, members: c.members, status, final: design, open, closed, new_gaps: gaps, tests,
      agents: steps.reduce((n, s) => n + (s.outs ? s.outs.length : 1), 0), steps }, extra || {})
    log(`${c.cid}: ${status} (${r.agents} agents)`)
    return r
  }

  const first = await makeDesign(c, earlier, null, null, steps)
  design = first.d || null
  if (!first.ok) return done('not reviewed: ' + first.why)
  if (design.outcome === 'whole_feature' || design.outcome === 'owner_question') return done(design.outcome)

  let prevBlocking = Infinity
  const idIs = (id, text) => new RegExp('^\\s*' + id + '(?![0-9])').test(text || '')
  for (let round = 1; round <= ROUNDS; round++) {
    const roles = round === 1 ? FIRST_ROUND : ['safety', 'engineering']
    // the engineering reviewer covers the specification too, unless a separate integrator ran this round
    const combinedFor = role => role === 'engineering' && !roles.includes('integration')
    const reviews = await parallel(roles.map(role => () => limited(() => agent(reviewPrompt(c, role, design, open, round, combinedFor(role)), { label: `review${round}-${role}:${c.cid}`, phase: 'Review', schema: REVIEW, effort: 'xhigh' }))))
    steps.push({ step: 'review', round, outs: reviews })
    for (const rv of reviews.filter(Boolean)) {
      for (const p of rv.previous || []) {
        const i = open.findIndex(o => idIs(o.id, p.objection) && ownsObjection(rv.role, combinedFor(rv.role), o))
        if (i >= 0 && p.status !== 'open') closed.push(Object.assign({ closed_as: p.status, closed_why: p.why }, open.splice(i, 1)[0]))
      }
      ;(rv.objections || []).forEach((o, i) => {
        const item = Object.assign({ id: `${rv.role[0].toUpperCase()}${round}-${i + 1}`, role: rv.role, round }, o)
        if (o.caused_by_change === false) gaps.push(item); else open.push(item)
      })
    }
    if (reviews.some(rv => !rv || !rv.finished)) log(`${c.cid}: a reviewer did not finish in round ${round}`)
    const blocking = open.filter(o => o.block !== 'none')
    log(`${c.cid}: round ${round}: ${blocking.length} blocking, ${open.length - blocking.length} weighed`)
    if (!blocking.length) break
    if (blocking.length >= prevBlocking) { log(`${c.cid}: no progress on blocking objections; to the arbiter`); break }
    prevBlocking = blocking.length
    if (round === ROUNDS) break
    const rev = await makeDesign(c, earlier, design, open, steps)
    if (!rev.ok) return done('revision failed: ' + rev.why)
    design = rev.d
    if (design.outcome === 'whole_feature' || design.outcome === 'owner_question') return done(design.outcome)
  }

  const unfinishedReview = steps.some(s => s.step === 'review' && s.outs.some(x => !x || !x.finished))
  const departs = ((design.impact || {}).verdicts || []).filter(v => v.kept_or_departs === 'departs')
  // the cheaper team: the arbiter runs only when a blocking objection is still disputed; weighed ones go to the owner
  if (!open.some(o => o.block !== 'none')) {
    if (unfinishedReview) return done('unverified: a reviewer did not finish')
    return done(open.length || departs.length ? 'accepted, the owner must see trade-offs' : 'accepted', { owner_must_see: open, departs })
  }
  const arb = await limited(() => agent(arbiterPrompt(c, design, open), { label: `arbiter:${c.cid}`, phase: 'Arbitrate', schema: ARBITER, effort: 'xhigh' }))
  steps.push({ step: 'arbiter', out: arb })
  // the arbiter names each objection by id, often followed by a description ("S2-1 (safety): ..."): match the id
  if (arb) for (const x of (arb.rulings || []).concat(arb.tradeoffs || [])) x.id = (open.find(o => idIs(o.id, x.objection)) || {}).id
  if (!arb || !arb.finished || open.some(o => !arb.rulings.some(x => x.id === o.id))) return done('unverified: the arbiter did not judge every open objection')
  if (arb.reclassify !== 'none') return done(arb.reclassify, { arbiter: arb })
  const blockers = arb.rulings.filter(x => x.verdict === 'real' && x.blocks)
  if (blockers.length) return done('rejected', { arbiter: arb, blockers })
  if (unfinishedReview) return done('unverified: a reviewer did not finish', { arbiter: arb })
  const ownerMustSee = arb.tradeoffs.filter(t => t.owner_must_see || !t.justified)
  return done(ownerMustSee.length || departs.length ? 'accepted, the owner must see trade-offs' : 'accepted', { arbiter: arb, owner_must_see: ownerMustSee, departs })
}

// ---- areas run in parallel; the questions of one area one after another, each seeing the earlier ------
const groups = []
for (const c of CLUSTERS) {
  let g = groups.find(x => x.gid === c.gid)
  if (!g) { g = { gid: c.gid, clusters: [] }; groups.push(g) }
  g.clusters.push(c)
}
log(`${CLUSTERS.length} question(s) in ${groups.length} area(s); ${LIMIT} agents at once; up to ${ROUNDS} review rounds`)
const perGroup = await pipeline(groups, async (first, item) => {
  const g = item && item.gid ? item : first
  const earlier = []
  const out = []
  for (const c of g.clusters) {
    const r = await runCluster(c, earlier)
    out.push(r)
    if (r.final && r.final.outcome) earlier.push({ cid: c.cid, outcome: r.final.outcome, status: r.status, edits: (r.final.edits || []).map(e => ({ rule: e.rule, after: e.after })) })
  }
  return out
})
const results = perGroup.filter(Boolean).flat()
log(results.map(r => `${r.cid}: ${r.status}`).join('; '))
return { clusters: results }
