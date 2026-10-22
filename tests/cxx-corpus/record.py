"""[CXX-1] — write this platform's records of the C++ corpus again, after a change to what the
importer gives that is meant: each entry's outcomes and thunk file, as `the_cpp_corpus_binds_as_recorded`
(compiler/ember_driver/tests/package.rs) compares them.

    python tests/cxx-corpus/record.py [path to ember]

Run it on Windows and on Linux (the records differ: `size_t` is `unsigned long long` on one and
`unsigned long` on the other), and read the diff before committing it.
"""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
PLATFORM = 'windows' if os.name == 'nt' else 'linux'
EMBER = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, 'target', 'ci', 'ember' + ('.exe' if os.name == 'nt' else ''))

# Each entry: its folder, its header, and the instantiations its import lists.
ENTRIES = [('nlohmann_json', 'nlohmann_json.hpp', ['nlohmann::basic_json<>'])]

for folder, header, instantiate in ENTRIES:
    entry = os.path.join(HERE, folder)
    stem = os.path.splitext(header)[0]
    bind = [EMBER, 'bind', header, '-I', os.path.join(HERE, 'third_party')]
    for template in instantiate:
        bind += ['--instantiate', template]
    outcomes = subprocess.run(bind + ['--outcomes'], cwd=entry, capture_output=True, text=True, check=True).stdout
    with open(os.path.join(entry, f'{stem}.{PLATFORM}.outcomes'), 'w', newline='\n') as out:
        out.write(outcomes)
    subprocess.run(bind + ['--emit-cpp', '--out', f'{stem}.{PLATFORM}.thunks.cpp'], cwd=entry, check=True, capture_output=True)
    print(f'{folder}: {len(outcomes.splitlines())} outcomes, thunks written for {PLATFORM}')
