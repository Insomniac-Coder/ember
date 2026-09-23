"""GitHub Actions status for a branch (default `main`), without `gh`:
python tasks/impl-0.9.9/ci_status.py [branch]

Prints each recent run and its jobs; a failed job lists its failed steps. The repository is
public, so no token is needed (the API allows 60 unauthenticated requests an hour).
"""
import json
import sys
import urllib.request

REPO = 'Insomniac-Coder/ember'
BRANCH = sys.argv[1] if len(sys.argv) > 1 else 'main'


def get(url):
    with urllib.request.urlopen(url) as response:
        return json.load(response)


runs = get(f'https://api.github.com/repos/{REPO}/actions/runs?branch={BRANCH}&per_page=3')
for run in runs.get('workflow_runs', []):
    print(run['id'], run['head_sha'][:7], run['status'], run['conclusion'])
    for job in get(run['jobs_url']).get('jobs', []):
        print('   ', job['name'], '|', job['status'], job['conclusion'])
        for step in job.get('steps', []):
            if step['conclusion'] == 'failure':
                print('       failed step:', step['name'])
