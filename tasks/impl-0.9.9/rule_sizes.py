"""The phase table's weights: each phase's rules' length in words, in the development target.

Usage: python tasks/impl-0.9.9/rule_sizes.py P1 P2 P3 P4 P5 P6 P7a
(the phases' current percentages, in that order) prints each phase's size and the overall
percentage weighted by size. With no arguments it prints the sizes only.

Phases are the 0.9.8 plan's (Part XXI of 0.9.8_Hardened_3), by rule family: Phase 1 is every rule
in Parts I-VI except `CLS` (Phase 3) and `CORO` (Phase 7a); the others are the families below.
"""
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
PHASES = {
    '2': {'OWN', 'BRW', 'LT', 'DRP', 'SPN', 'CELL'},
    '3': {'OBJ', 'RC', 'EXC', 'DSP', 'WK', 'CLS'},
    '4': {'EFF', 'CT', 'RFL', 'DRV'},
    '5': {'FFI'},
    '6': {'THR', 'JOB', 'PAR', 'SOA', 'SIMD', 'ECS'},
    '7a': {'CORO', 'DET', 'HR', 'BUD', 'MONO'},
}
EARLY_PARTS = {'front', 'I', 'II', 'III', 'IV', 'V', 'VI'}
ORDER = ['1', '2', '3', '4', '5', '6', '7a']


def rule_sizes(doc):
    """Every `* `[FAM-N]`` rule with its part, family and length in words."""
    rules, part, current = {}, 'front', None
    for line in doc.split('\n'):
        heading = re.match(r'^# Part ([IVXL]+[a-zA-Z-]*)', line)
        if heading:
            part, current = heading.group(1), None
            continue
        rule = re.match(r'^\* `\[([A-Z]+(?:-[A-Z]+)?)-([0-9]+[a-z0-9]*)\]`(.*)', line)
        if rule:
            current = f'{rule.group(1)}-{rule.group(2)}'
            entry = rules.setdefault(current, {'part': part, 'family': rule.group(1), 'words': 0})
            entry['words'] += len(rule.group(3).split())
        elif current and line.startswith('  ') and line.strip():
            rules[current]['words'] += len(line.split())
        else:
            current = None
    return rules


def phase_of(rule):
    for phase, families in PHASES.items():
        if rule['family'] in families:
            return phase
    return '1' if rule['part'] in EARLY_PARTS else None


def main():
    target = json.loads((ROOT / 'docs/spec-source/development-target.json').read_text(encoding='utf-8'))['path']
    rules = rule_sizes((ROOT / target).read_text(encoding='utf-8'))
    sizes = {phase: [0, 0] for phase in ORDER}
    for rule in rules.values():
        phase = phase_of(rule)
        if phase:
            sizes[phase][0] += 1
            sizes[phase][1] += rule['words']
    print(f'target {target}')
    for phase in ORDER:
        print(f'phase {phase:>2}: {sizes[phase][0]:3} rules, {sizes[phase][1]:5} words')
    if len(sys.argv) == len(ORDER) + 1:
        done = dict(zip(ORDER, map(float, sys.argv[1:])))
        total = sum(sizes[phase][1] for phase in ORDER)
        overall = sum(done[phase] * sizes[phase][1] for phase in ORDER) / total
        print(f'overall, weighted by words: {overall:.1f}%')


if __name__ == '__main__':
    main()
