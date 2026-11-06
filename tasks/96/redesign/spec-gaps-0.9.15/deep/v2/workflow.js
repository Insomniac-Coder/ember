export const meta = {
  name: 'spec-gaps-deep-review-v2',
  description: 'Deep review method 2: check and group the holes by cause, one rule per cause (no longer than the first design), gate, two breakers; at most two rounds',
  phases: [
    { title: 'Check', detail: 'every hole: real? caused by this rule? grouped by cause' },
    { title: 'Redesign', detail: 'one rule per cause, existing tools first, choices decided by the owner\'s parameters and recorded' },
    { title: 'Gate', detail: 'length cap (script) + one checker, before any breaker' },
    { title: 'Break', detail: 'safety and engineering' },
  ],
}

// args: { groups: ['P02', ...], cap: { P02: chars, ... }, limit: N }
const GROUPS = args.groups
const CAP = args.cap
const LIMIT = args.limit || 3   // the owner sets it at launch

let active = 0
const waiting = []
async function limited(fn) {
  while (active >= LIMIT) await new Promise(r => waiting.push(r))
  active++
  try { return await fn() } finally { active--; const next = waiting.shift(); if (next) next() }
}

const D = 'C:\\Users\\ism19\\Code\\ember\\tasks\\96\\redesign\\spec-gaps-0.9.15\\'
const RULES = `Read first, in full, and keep every rule in it: ${D}deep\\RULES-FOR-EVERY-AGENT.md (the owner: "all agents HAVE to abide by those rules"). It points you to ${D}deep\\owner-bar.md, C:\\Users\\ism19\\Code\\ember\\docs\\OWNER-DIRECTION.md, the owner's verdicts in C:\\Users\\ism19\\Code\\ember\\tasks\\96\\redesign\\JOB.md (1 to 166) and ${D}brief.md; read them too.`
const CONTEXT = gid => `Your group is ${gid}: its principle, members, other_places and verdicts_involved are in ${D}deep\\phase1-groups.json (read the other groups' principles too, so your work does not tread on theirs); its members' drafts are in ${D}phase2.json and the critic's notes in ${D}phase3.json. The group's input is ${D}deep\\v2\\inputs\\${gid}.json: first_design (method 1's first design), method1_repair (method 1's repair, which patched one clause per hole and failed; for reference only) and holes (every hole the breakers found, each with its id, the design it was found against, its lens, kind, severity, program, what goes wrong and the breaker's fix hint). The specification is C:\\Users\\ism19\\Code\\ember\\docs\\spec-source\\Ember_v0.9.15_Hardened_11.md, assembled from C:\\Users\\ism19\\Code\\ember\\tasks\\spec-0.9.9\\parts\\p*.md (quote line numbers from the part files). The lead's rulings: C:\\Users\\ism19\\Code\\ember\\tasks\\96\\redesign\\compiler-0.9.15\\decisions.md.`

const RULES_CHECKED = { type: 'array', items: { type: 'object', properties: { rule: { type: 'string' }, how_kept: { type: 'string' } }, required: ['rule', 'how_kept'] } }
const OWNER_SHORT = { type: 'object', properties: { one_line: { type: 'string' }, step_table: { type: 'string' }, today: { type: 'string' }, options: { type: 'array', items: { type: 'string' } }, recommendation: { type: 'string' } }, required: ['one_line', 'step_table', 'today', 'options', 'recommendation'] }

const CHECK = {
  type: 'object',
  properties: {
    gid: { type: 'string' },
    finished: { type: 'boolean' },
    holes: { type: 'array', items: { type: 'object', properties: {
      id: { type: 'string' },
      verdict: { type: 'string', enum: ['real', 'refuted'] },
      caused_by: { type: 'string', enum: ['this_rule', 'elsewhere', 'patch_only'] },
      elsewhere: { type: 'string' },
      severity: { type: 'string', enum: ['high', 'medium', 'low'] },
      evidence: { type: 'string' } },
      required: ['id', 'verdict', 'caused_by', 'elsewhere', 'severity', 'evidence'] } },
    causes: { type: 'array', items: { type: 'object', properties: {
      id: { type: 'string' },
      cause: { type: 'string' },
      holes: { type: 'array', items: { type: 'string' } },
      existing_answer: { type: 'string' },
      owner_decision_needed: { type: 'boolean' },
      why: { type: 'string' } },
      required: ['id', 'cause', 'holes', 'existing_answer', 'owner_decision_needed', 'why'] } },
    notes: { type: 'string' },
    rules_checked: RULES_CHECKED,
  },
  required: ['gid', 'finished', 'holes', 'causes', 'notes', 'rules_checked'],
}

const DESIGN = {
  type: 'object',
  properties: {
    gid: { type: 'string' },
    finished: { type: 'boolean' },
    rule: { type: 'string' },
    spec_text: { type: 'string' },
    cause_answers: { type: 'array', items: { type: 'object', properties: { cause: { type: 'string' }, how_answered: { type: 'string' }, existing_tool: { type: 'string' } }, required: ['cause', 'how_answered', 'existing_tool'] } },
    answers: { type: 'array', items: { type: 'object', properties: { member: { type: 'string' }, decides: { type: 'string' }, program: { type: 'string' }, outcome: { type: 'string' } }, required: ['member', 'decides', 'program', 'outcome'] } },
    other_cases: { type: 'array', items: { type: 'object', properties: { case: { type: 'string' }, program: { type: 'string' }, outcome: { type: 'string' } }, required: ['case', 'program', 'outcome'] } },
    prior_art: { type: 'string' },
    cost: { type: 'string' },
    readability: { type: 'object', properties: { before: { type: 'string' }, after: { type: 'string' }, verdict: { type: 'string' } }, required: ['before', 'after', 'verdict'] },
    compiler_does: { type: 'array', items: { type: 'string' } },
    misuse: { type: 'string' },
    verdicts: { type: 'array', items: { type: 'object', properties: { verdict: { type: 'string' }, kept_or_revisited: { type: 'string' }, evidence: { type: 'string' } }, required: ['verdict', 'kept_or_revisited', 'evidence'] } },
    decisions_taken: { type: 'array', items: { type: 'object', properties: {
      title: { type: 'string' }, problem: { type: 'string' },
      options: { type: 'array', items: { type: 'object', properties: { letter: { type: 'string' }, code: { type: 'string' }, outcome: { type: 'string' } }, required: ['letter', 'code', 'outcome'] } },
      chosen: { type: 'string' }, why: { type: 'string' }, departs_from_verdict: { type: 'string' }, owner_short: OWNER_SHORT },
      required: ['title', 'problem', 'options', 'chosen', 'why', 'departs_from_verdict', 'owner_short'] } },
    open_risks: { type: 'array', items: { type: 'string' } },
    rules_checked: RULES_CHECKED,
  },
  required: ['gid', 'finished', 'rule', 'spec_text', 'cause_answers', 'answers', 'other_cases', 'prior_art', 'cost', 'readability', 'compiler_does', 'misuse', 'verdicts', 'decisions_taken', 'open_risks', 'rules_checked'],
}

const GATE = {
  type: 'object',
  properties: {
    gid: { type: 'string' },
    finished: { type: 'boolean' },
    pass: { type: 'boolean' },
    problems: { type: 'array', items: { type: 'object', properties: { rule: { type: 'string' }, where: { type: 'string' }, why: { type: 'string' } }, required: ['rule', 'where', 'why'] } },
    rules_checked: RULES_CHECKED,
  },
  required: ['gid', 'finished', 'pass', 'problems', 'rules_checked'],
}

const BREAK = {
  type: 'object',
  properties: {
    gid: { type: 'string' },
    lens: { type: 'string', enum: ['safety', 'engineering'] },
    finished: { type: 'boolean' },
    covered: { type: 'array', items: { type: 'string' } },
    matrix: { type: 'array', items: { type: 'object', properties: { guarantee: { type: 'string' }, how_kept: { type: 'string' }, edge_cases: { type: 'string' }, must_refuse: { type: 'string' }, must_compile: { type: 'string' } }, required: ['guarantee', 'how_kept', 'edge_cases', 'must_refuse', 'must_compile'] } },
    holes: { type: 'array', items: { type: 'object', properties: {
      kind: { type: 'string', enum: ['unsafe_allowed', 'crash_or_undefined', 'fault_not_tolerated', 'needless_refusal', 'cost_over_c', 'special_case', 'not_general', 'verdict_clash', 'cross_rule_clash', 'readability_loss', 'programmer_must_write_what_compiler_could_infer'] },
      caused_by: { type: 'string', enum: ['this_rule', 'elsewhere'] },
      program: { type: 'string' }, what_goes_wrong: { type: 'string' }, severity: { type: 'string', enum: ['high', 'medium', 'low'] }, fix_hint: { type: 'string' } },
      required: ['kind', 'caused_by', 'program', 'what_goes_wrong', 'severity', 'fix_hint'] } },
    broken: { type: 'boolean' },
    notes: { type: 'string' },
    rules_checked: RULES_CHECKED,
  },
  required: ['gid', 'lens', 'finished', 'covered', 'matrix', 'holes', 'broken', 'notes', 'rules_checked'],
}

const checkPrompt = (gid, holesNote, design) => `You check the holes found in one proposed rule of the Ember programming language's specification, in a deep review the owner asked for. ${RULES} ${CONTEXT(gid)}\n\n` +
  `The holes to check: ${holesNote}\n\n` +
  `For every high or medium hole: (1) is it real? It is real only when its program, under the rules as written (the design and the specification), does what the breaker claims; otherwise refuted, with the rule lines that settle it. Default to refuted when it is not shown. (2) What causes it: this rule (this_rule), a gap the rule does not own (elsewhere: name the group from phase1-groups.json or the spec rule), or only text that method 1's repair added (patch_only: the program does not break the first design and shows no case the rule must decide). Keep each hole's severity, or correct it with the reason. ` +
  `Then group the real holes caused by this rule by their ROOT CAUSE: the one fact about the language that makes them all possible. Expect few causes; two holes with one cause are one cause. For each cause, search the specification and the owner's verdicts for a rule that already answers it (verdict 53 is one example: a fact worked out from a body, kept with the function and checked at each call) and name it in existing_answer, or say none. Set owner_decision_needed only when you can show that the owner's requirements cannot all hold for some program of this cause (the compiler cannot work the fact out, so only a written word, a run-time check or a refusal remain). List low holes too, with verdict and cause. Set finished true only when every hole has a verdict.` +
  (design ? `\n\nThe design these holes were found against:\n${JSON.stringify(design)}` : '')

const designPrompt = (gid, causes, previous, gateProblems) => `You are the designer for principle group ${gid} in a deep review of the Ember programming language's specification. ${RULES} ${CONTEXT(gid)}\n\n` +
  `The first design is first_design in the input file. Its holes were checked and grouped by cause:\n${JSON.stringify(causes)}\n\n` +
  (previous ? `Your design of the last round, whose holes these are:\n${JSON.stringify(previous)}\n\n` : '') +
  (gateProblems ? `The gate rejected your last attempt for these reasons; answer every one:\n${JSON.stringify(gateProblems)}\n\n` : '') +
  `Write the group's rule again so that each cause is answered at its root, and every hole of that cause follows from the rule. Rules D1 to D5 bind you: the language's own tools first (an existing rule or verdict used as it is), one rule per cause and never a clause per hole, and the rule plus spec_text no longer than ${CAP[gid]} characters (the first design's; a longer text is rejected before anyone reads it). A cause whose owner_decision_needed is true, or any choice the rule needs, you DECIDE yourself by the owner's parameters in the order rule D4 gives (the owner: "I want the workflow to make the decision based on the parameters that I set, then maybe later once stuff is done then you can discuss the changes"): never leave it open, never patch around it, and record each in decisions_taken (the options as code and outcome, the one chosen, which parameters chose it, any verdict it departs from and why that is really better, and owner_short for the owner's later review). Fill every field as the schema asks: answers (each member), other_cases (at least five beyond the members), prior_art (C++, Rust, Swift, and others where they bear on it), cost (per operation against C, marked to measure), readability (an ordinary program before and after: the after no longer and no harder), compiler_does, misuse, verdicts, rules_checked. Set finished true only when every cause, member and other case is answered.`

const gatePrompt = (gid, design) => `You are the gate for one proposed rule of the Ember programming language's specification: you decide whether it may go to the breakers. ${RULES} The group's principle is ${gid} in ${D}deep\\phase1-groups.json; its first design is first_design in ${D}deep\\v2\\inputs\\${gid}.json.\n\n` +
  `Reject the design (pass false) for any of: a clause or exception for one construct, one type, one example or one program (rule C1, D2; principle 6 allows an exception only where the general wording would be untrue, saying why); a cause answered by something new where an existing rule or verdict already answers it (D1); a choice left open, made against the owner's parameters or their order in D4, or missing from decisions_taken (D4); a verdict departed from without being really better by the owner's parameters, or widened silently (D5); the ordinary program longer or harder to read (C2); a word the programmer must write that the compiler could work out (C3); an entry of rules_checked that is untrue. Quote the words in where. Pass only when none applies. You read the design; you do not search for safety holes (the breakers do). The design:\n${JSON.stringify(design)}`

const breakPrompt = (gid, lens, design) => `You check one proposed rule of the Ember programming language's specification, as part of the owner's defensive validation of their own language (C:\\Users\\ism19\\Code\\ember\\tasks\\96\\redesign\\SAFETY-VALIDATION-TASK.md describes that task; this is its specification-level form: no system is tested, nothing is run). ${RULES} ${CONTEXT(gid)}\n\n` +
  `Your lens: ${lens === 'safety'
    ? 'SAFETY AND ROBUSTNESS. Build the safety-invariant matrix for the rule: for each guarantee it must keep (no read of freed memory, no data race, no double end, no undefined behaviour, a fault stopped clearly at its line), how the rule keeps it, the edge cases, a program that must be refused and one that must compile. Then find every program the rule lets through that breaks a guarantee, and every fault it does not stop clearly.'
    : "ENGINEERING. Build the same matrix for the rule's other promises: general (no special case, no construct-specific patch), as fast as C where safety allows, readable and easy (the ordinary program no longer or harder than today), the compiler doing what it can infer, consistent with the owner's verdicts and with the other groups' principles. Then find every valid program it refuses needlessly, every special case, every cost over C, every clash."}\n\n` +
  `Every hole comes with the concrete program, in the specification's 0.9.15 spelling, and what goes wrong under the rules as written; say whether this rule causes it (this_rule) or a gap it does not own does (elsewhere). Severity: high (safety, or a common program wrong), medium, low. broken is true if a high or medium hole is caused by this rule. List in covered every member, other case and guarantee you checked; set finished true only when you checked them all. The design:\n${JSON.stringify(design)}`

const ok = r => r && r.finished
const lenOf = d => (d.rule || '').length + (d.spec_text || '').length
const realHoles = c => c.holes.filter(h => h.verdict === 'real' && h.caused_by === 'this_rule' && h.severity !== 'low')

// One redesign that passes the length cap and the gate, with one redo; null when it cannot.
async function redesign(gid, causes, previous, log_) {
  let problems = null
  for (let attempt = 1; attempt <= 2; attempt++) {
    const design = await limited(() => agent(designPrompt(gid, causes, previous, problems), { label: `redesign:${gid}`, phase: 'Redesign', schema: DESIGN, effort: 'xhigh' }))
    if (!design) return { design: null, why: 'designer returned nothing' }
    if (lenOf(design) > CAP[gid]) {
      problems = [{ rule: 'D3', where: `rule + spec_text = ${lenOf(design)} characters`, why: `longer than the first design's ${CAP[gid]}` }]
      log_.push({ step: 'gate', attempt, pass: false, problems })
      continue
    }
    const gate = await limited(() => agent(gatePrompt(gid, design), { label: `gate:${gid}`, phase: 'Gate', schema: GATE, effort: 'high' }))
    log_.push({ step: 'gate', attempt, gate })
    if (gate && gate.finished && gate.pass) return { design }
    problems = gate ? gate.problems : [{ rule: 'gate', where: '', why: 'gate returned nothing' }]
  }
  return { design: null, why: 'rejected twice by the gate' }
}

const breakBoth = (gid, design, round) => parallel(['safety', 'engineering'].map(lens => () =>
  limited(() => agent(breakPrompt(gid, lens, design), { label: `break${round}-${lens}:${gid}`, phase: 'Break', schema: BREAK, effort: 'xhigh' }))))

const results = await pipeline(GROUPS, async (gid) => {
  const steps = []
  let check = await limited(() => agent(checkPrompt(gid, 'the holes array in the input file (found by method 1 against first_design and against method1_repair)', null), { label: `check0:${gid}`, phase: 'Check', schema: CHECK, effort: 'xhigh' }))
  steps.push({ step: 'check', round: 0, check })
  if (!ok(check)) return { gid, status: 'unverified', why: 'the first check did not finish', steps }
  let current = null
  for (let round = 1; round <= 2; round++) {
    const made = await redesign(gid, check.causes, current, steps)
    if (!made.design) return { gid, status: 'unresolved: method', why: made.why, steps }
    current = made.design
    steps.push({ step: 'design', round, design: current })
    const breaks = await breakBoth(gid, current, round)
    steps.push({ step: 'break', round, breaks })
    const holes = breaks.filter(Boolean).flatMap(b => b.holes.map((h, i) => ({ id: `${b.lens[0].toUpperCase()}${round}-${i + 1}`, lens: b.lens, ...h })))
    check = await limited(() => agent(checkPrompt(gid, `these holes, found by the two breakers against the design below:\n${JSON.stringify(holes)}`, current), { label: `check${round}:${gid}`, phase: 'Check', schema: CHECK, effort: 'xhigh' }))
    steps.push({ step: 'check', round, check })
    const unverified = breaks.some(b => !ok(b)) || !ok(check)
    const real = check ? realHoles(check) : []
    if (!real.length) {
      const status = unverified ? 'unverified' : 'holds'
      log(`${gid}: ${status} after ${round} round(s), ${current.decisions_taken.length} decision(s) for the owner's review`)
      return { gid, status, final: current, decisions: current.decisions_taken, steps }
    }
  }
  log(`${gid}: unresolved after 2 rounds`)
  return { gid, status: 'unresolved', final: current, decisions: current.decisions_taken, open: realHoles(check), steps }
})

const done = results.filter(Boolean)
log(done.map(r => `${r.gid}: ${r.status}`).join('; '))
return { groups: done }
