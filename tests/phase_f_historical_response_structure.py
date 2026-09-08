"""F1/F2 attacks through public historical validation and raw HTTP responses."""
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('phase_f_hrs', ROOT / 'docs/engineering_specification/phase_f/generate_phase_f_manifests.py')
m = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = m
spec.loader.exec_module(m)
P = 'e5deb18990972e6618ac4bc9731766febc0f4173'
A = '76561751dcdc40ae56943a0c09a2d6d0ae201e07'
C = 'd' * 40
API = 'https://api.github.com/repos/XingyuW/rust_electroanalysis_cli'


def obj(sha, family='commits'):
    return {'sha': sha, 'url': f'{API}/{family}/{sha}'}


def commit(sha):
    return {**obj(sha, 'git/commits'), 'tree': obj('b' * 40, 'git/trees'), 'parents': [obj('c' * 40, 'git/commits')]}


class Raw:
    def __init__(self, target=A, head=P):
        self.target, self.head, self.end = target, head, head
        self.calls = []
        self.commit = commit(target)
        self.head_commit = commit(head)
        self.comparison = {'url': f'{API}/compare/{target}...{head}',
            'base_commit': obj(target), 'merge_base_commit': obj(target),
            'status': 'identical' if target == head else 'ahead', 'behind_by': 0,
            'ahead_by': int(target != head), 'total_commits': int(target != head),
            'commits': [] if target == head else [obj(head)]}

    def __call__(self, request, timeout):
        url = request.full_url
        self.calls.append(url)
        assert request.get_method() == 'GET'
        if url == API:
            data = {'id': 1273879958, 'full_name': 'XingyuW/rust_electroanalysis_cli',
                    'name': 'rust_electroanalysis_cli', 'owner': {'login': 'XingyuW'}}
        elif url == API + '/git/ref/heads/main':
            head = self.head if self.calls.count(url) == 1 else self.end
            data = {'ref': 'refs/heads/main', 'object': {'type': 'commit', 'sha': head}}
        elif url == API + '/git/commits/' + self.target:
            data = self.commit
        elif url == API + '/git/commits/' + self.head:
            data = self.head_commit
        elif url == API + '/compare/' + self.target + '...' + self.head:
            data = self.comparison
        elif url.startswith(API + '/rulesets/') or url == API + '/git/ref/heads/phase-f-reviewer-bootstrap-head':
            raise m.HTTPError(url, 404, 'TEST_ONLY unprovisioned', {}, None)
        else:
            raise AssertionError('Unexpected request: ' + url)
        class Response:
            headers = {}
            def __enter__(self): return self
            def __exit__(self, *args): pass
            def read(self): return json.dumps(data).encode()
        return Response()


def final(raw, repository=ROOT):
    with patch.dict(os.environ, GH_TOKEN='TEST_ONLY_NEVER_SENT'), patch.object(m, 'urlopen', side_effect=raw):
        return m.validate_historical_normative_target(repository, raw.target)


def response_mutations():
    wrong = 'https://api.github.com/repos/other/repo'
    return [
        ('wrong commit repository', lambda t: t.commit.update(url=f'{wrong}/git/commits/{A}')),
        ('wrong commit SHA URL', lambda t: t.commit.update(url=f'{API}/git/commits/{C}')),
        ('wrong tree repository', lambda t: t.commit['tree'].update(url=f'{wrong}/git/trees/' + 'b'*40)),
        ('wrong tree SHA URL', lambda t: t.commit['tree'].update(url=f'{API}/git/trees/{C}')),
        ('wrong parent repository', lambda t: t.commit['parents'][0].update(url=f'{wrong}/git/commits/' + 'c'*40)),
        ('missing commit URL', lambda t: t.commit.pop('url')),
        ('missing tree URL', lambda t: t.commit['tree'].pop('url')),
        ('missing parent URL', lambda t: t.commit['parents'][0].pop('url')),
        ('wrong canonical head object', lambda t: t.head_commit.update(sha=C)),
        ('wrong compare repository', lambda t: t.comparison.update(url=f'{wrong}/compare/{A}...{P}')),
        ('wrong compare BASE', lambda t: t.comparison.update(url=f'{API}/compare/{C}...{P}')),
        ('wrong compare HEAD', lambda t: t.comparison.update(url=f'{API}/compare/{A}...{C}')),
        ('missing compare URL', lambda t: t.comparison.pop('url')),
        ('wrong base SHA', lambda t: t.comparison.update(base_commit=obj(C))),
        ('wrong base repository', lambda t: t.comparison['base_commit'].update(url=f'{wrong}/commits/{A}')),
        ('wrong merge base SHA', lambda t: t.comparison.update(merge_base_commit=obj(C))),
        ('wrong merge base repository', lambda t: t.comparison['merge_base_commit'].update(url=f'{wrong}/commits/{A}')),
        ('positive total empty commits', lambda t: t.comparison.update(commits=[])),
        ('wrong final head', lambda t: t.comparison.update(commits=[obj(C)])),
        ('wrong final head repository', lambda t: t.comparison['commits'][-1].update(url=f'{wrong}/commits/{P}')),
        ('status count contradiction', lambda t: t.comparison.update(status='identical')),
        ('ahead total contradiction', lambda t: t.comparison.update(ahead_by=2)),
        ('missing commits', lambda t: t.comparison.pop('commits')),
        ('boolean count', lambda t: t.comparison.update(ahead_by=True)),
        ('duplicate list identities', lambda t: t.comparison.update(ahead_by=2,total_commits=2,commits=[obj(P),obj(P)])),
    ] + [(f'compare URL alias {name}', lambda t, transform=transform: t.comparison.update(url=transform(t.comparison['url']))) for name, transform in [
        ('query', lambda u: u+'?page=1'), ('fragment', lambda u: u+'#x'),
        ('port', lambda u: u.replace('api.github.com','api.github.com:443')),
        ('userinfo', lambda u: u.replace('api.github.com','user@api.github.com')),
        ('percent encoding', lambda u: u.replace('/compare/','/%63ompare/')),
        ('normalization', lambda u: u.replace('/compare/','/x/../compare/')),
        ('trailing segment', lambda u: u+'/'), ('wrong endpoint', lambda u: u.replace('/compare/','/commits/')),
        ('wrong origin', lambda u: u.replace('https://','http://')),
    ]]


class ResponseBinding(unittest.TestCase):
    def test_positive_and_response_matrix(self):
        for target, head in [(P,P),(A,P),(P,C)]:
            t=Raw(target,head)
            self.assertEqual(final(t), {'historical_publication_valid': True,
                'historical_normative_structure_valid': True, 'current_operational_authority': False})
            self.assertIn(API+'/git/commits/'+head,t.calls)
        for name, mutate in response_mutations():
            with self.subTest(name=name):
                t=Raw();mutate(t)
                with self.assertRaises(m.G3ValidationError): final(t)
        t=Raw(P,P);t.comparison.update(commits=[obj(P)])
        with self.assertRaises(m.G3ValidationError): final(t)
        t=Raw();t.end=C
        with self.assertRaisesRegex(m.G3ValidationError,'publication_head_changed'): final(t)

    def test_wrong_head_replay(self):
        response=Raw(A,P).comparison
        t=Raw(A,C);t.comparison=response
        with self.assertRaisesRegex(m.G3ValidationError,'historical_compare_identity_mismatch'): final(t)

    def test_unpaginated_boundaries(self):
        for total in [1,249,250,251,1000]:
            t=Raw()
            t.comparison.update(ahead_by=total,total_commits=total,
                commits=[obj(f'{i:040x}') for i in range(1,min(total,250))]+[obj(P)])
            self.assertTrue(final(t)['historical_publication_valid'])
            self.assertTrue(all('?' not in u for u in t.calls))
            if total > 1:
                t.comparison['commits'].pop(0)
                self.assertTrue(final(t)['historical_publication_valid'])

        for commits in [[], [obj(f'{i:040x}') for i in range(1, 251)] + [obj(P)]]:
            t=Raw()
            t.comparison.update(ahead_by=251,total_commits=251,commits=commits)
            with self.assertRaises(m.G3ValidationError): final(t)
        t=Raw()
        t.comparison.update(ahead_by=1,total_commits=1,commits=[obj('1'*40),obj(P)])
        with self.assertRaises(m.G3ValidationError): final(t)

    def test_shorter_bounded_compare_list_is_compatible(self):
        t=Raw()
        t.comparison.update(ahead_by=251,total_commits=251,
            commits=[obj(f'{i:040x}') for i in range(1,249)]+[obj(P)])
        self.assertEqual(final(t), {'historical_publication_valid': True,
            'historical_normative_structure_valid': True, 'current_operational_authority': False})

    def test_removed_semantic_validators_fail_closed(self):
        for name in ['_validate_canonical_commit','_validate_canonical_comparison']:
            with patch.object(m,name,None):
                with self.assertRaises(m.G3ValidationError): final(Raw())

    def test_raw_transport_cannot_bypass_resolver_validation(self):
        # Bypass each transport semantic validator while preserving raw parsing.
        # Registered publication dispatch independently validates its evidence.
        for method, attribute in [('get_commit','commit'),('compare_commits','comparison')]:
            t=Raw()
            getattr(t,attribute)['url']='https://wrong.invalid'
            with patch.object(m.GitHubApiTransport, method, lambda self,*args: getattr(t,attribute)):
                with self.assertRaises(m.G3ValidationError): final(t)


class CommittedStructure(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory=Path(tempfile.mkdtemp(prefix='phase-f-hrs-structures-'))
        cls.repo=cls.directory/'repository'
        subprocess.run(['git','clone','--no-hardlinks','--no-checkout',str(ROOT),str(cls.repo)],check=True,capture_output=True)
        cls.git('checkout','--detach',P)
        cls.git('config','user.name','TEST_ONLY HRS')
        cls.git('config','user.email','hrs@example.invalid')
        cls.paths=[m.BUNDLE_PATH,m.TRACE_PATH,m.AUTHORITY_GRAPH_PATH,m.ARCH,
            m.R11_SOURCE,m.MIGRATION_LEDGER,*m.SPECS.values(),m.NORMATIVE_MATRIX_PATH]
        cls.original={p: (cls.repo/p.relative_to(m.ROOT)).read_bytes() for p in cls.paths}
        cls.records=[]
        print('Retained committed malformed fixtures:',cls.directory,flush=True)

    @classmethod
    def git(cls,*args):
        return subprocess.check_output(['git','--no-replace-objects',*args],cwd=cls.repo,stderr=subprocess.PIPE).decode().strip()

    def save(self,path,value):
        (self.repo/path.relative_to(m.ROOT)).write_bytes(json.dumps(value,sort_keys=True,indent=2).encode()+b'\n')

    def refresh_hashes(self,b,t,g):
        def fixture_path(p):
            if p.is_absolute():
                return p if p == self.repo or self.repo in p.parents else self.repo/p.relative_to(m.ROOT)
            return self.repo/p
        def raw(p): return fixture_path(p).read_bytes()
        def digest(p): return m.sha256_bytes(raw(p))
        t['authority_graph']['sha256']=digest(m.AUTHORITY_GRAPH_PATH)
        for node in t['generated_source_sha256s']:
            t['generated_source_sha256s'][node]=digest(Path(g['node_identity_rules'][node]['path']))
        self.save(m.TRACE_PATH,t)
        paths={'architecture_plan':m.ARCH,'wire_specification':m.SPECS['F-WIRE'],
            'scientific_specification':m.SPECS['F-SCI'],'operations_specification':m.SPECS['F-OPS'],
            'conformance_specification':m.SPECS['F-CNF'],'implementation_readiness_specification':m.SPECS['F-IMPL'],
            'migration_ledger':m.MIGRATION_LEDGER,'normative_traceability_matrix':m.NORMATIVE_MATRIX_PATH,
            'authority_graph':m.AUTHORITY_GRAPH_PATH,'generated_traceability_manifest':m.TRACE_PATH}
        inputs=b['bundle_inputs']
        inputs['source_sha256s']={k:digest(v) for k,v in paths.items()}
        inputs['authority_graph_sha256']=digest(m.AUTHORITY_GRAPH_PATH)
        for node,v in inputs['authority_bindings'].items():
            rule=g['node_identity_rules'][node]
            if rule['type']=='repository_file_sha256':v['sha256']=digest(Path(rule['path']))
        for field in ['architecture_plan','traceability_manifest','migration_ledger','normative_traceability_matrix','authority_graph']:
            p=self.repo/Path(b[field]['path']);b[field]['sha256']=digest(p)
            if 'git_blob' in b[field]:b[field]['git_blob']=m._git_blob_bytes(raw(p))
        inputs['sha256']=m.sha256_bytes(m.canonical_json_bytes({k:v for k,v in inputs.items() if k!='sha256'}))
        b['target_revision']['sha256']=inputs['sha256']
        self.save(m.BUNDLE_PATH,b)
        self.assertEqual(inputs['sha256'],m.sha256_bytes(m.canonical_json_bytes({k:v for k,v in inputs.items() if k!='sha256'})))

    def replace_text(self, path, old, new):
        fixture = self.repo/path.relative_to(m.ROOT)
        text = fixture.read_text()
        self.assertIn(old, text)
        fixture.write_text(text.replace(old, new, 1))

    def commit_source_attack(self, name, mutate):
        for path, raw in self.original.items():
            (self.repo/path.relative_to(m.ROOT)).write_bytes(raw)
        b=json.loads(self.original[m.BUNDLE_PATH]);t=json.loads(self.original[m.TRACE_PATH]);g=json.loads(self.original[m.AUTHORITY_GRAPH_PATH])
        mutate()
        self.save(m.AUTHORITY_GRAPH_PATH,g)
        self.refresh_hashes(b,t,g)
        self.git('add','.')
        self.git('commit','-qm','TEST_ONLY historical source attack: '+name)
        sha=self.git('rev-parse','HEAD')
        with self.assertRaises((m.G3ValidationError,ValueError)) as error:
            final(Raw(sha,sha),self.repo)
        self.assertNotIn('publication',str(error.exception))
        self.records.append({'case':name,'sha':sha,'rejected':str(error.exception)})

    def test_architecture_F0_inventory(self):
        for name in ['missing F0 owner decision','duplicate F0 owner decision','duplicate F0 JSON member']:
            with self.subTest(name=name):
                for p,raw in self.original.items():(self.repo/p.relative_to(m.ROOT)).write_bytes(raw)
                b=json.loads(self.original[m.BUNDLE_PATH]);t=json.loads(self.original[m.TRACE_PATH]);g=json.loads(self.original[m.AUTHORITY_GRAPH_PATH])
                arch=self.original[m.ARCH].decode()
                line=next(line for line in arch.splitlines(keepends=True) if line.startswith('| `F-OD-01` |'))
                if name=='missing F0 owner decision':arch=arch.replace(line,'')
                if name=='duplicate F0 owner decision':arch=arch.replace(line,line+line)
                (self.repo/m.ARCH.relative_to(m.ROOT)).write_text(arch)
                self.refresh_hashes(b,t,g)
                if name=='duplicate F0 JSON member':
                    path=self.repo/m.BUNDLE_PATH.relative_to(m.ROOT)
                    raw=path.read_text().replace('"f0_decisions": {','"f0_decisions": {"approved_tag": null, "decision_bundle_sha256": null}, "f0_decisions": {',1)
                    path.write_text(raw)
                self.git('add','.');self.git('commit','-qm','TEST_ONLY '+name)
                sha=self.git('rev-parse','HEAD')
                with self.assertRaises((m.G3ValidationError,ValueError)) as error:final(Raw(sha,sha),self.repo)
                self.records.append({'case':name,'sha':sha,'rejected':str(error.exception)})

    def test_committed_mutations_with_recomputed_hashes(self):
        cases=[
            ('nested schema 999',lambda b,t,g:b['bundle_inputs'].update(schema_version=999)),
            ('nested wrong artifact',lambda b,t,g:b['bundle_inputs'].update(artifact_kind='wrong')),
            ('bundle wrong artifact',lambda b,t,g:b.update(artifact_kind='wrong')),
            ('bundle schema boolean',lambda b,t,g:b.update(schema_version=True)),
            ('missing F0',lambda b,t,g:b.pop('f0_decisions')),
            ('F0 wrong type',lambda b,t,g:b.update(f0_decisions=[])),
            ('missing F0 entry',lambda b,t,g:b['f0_decisions'].pop('approved_tag')),
            ('inconsistent F0 entry',lambda b,t,g:b['f0_decisions'].update(approved_tag='unapproved',decision_bundle_sha256='a'*64)),
            ('missing component scope',lambda b,t,g:b['component_specifications'].pop()),
            ('duplicate component scope',lambda b,t,g:b['component_specifications'].append(deepcopy(b['component_specifications'][0]))),
            ('review decision counts',lambda b,t,g:b['component_specifications'][0].update(review_status='GO',p0_count=1,p1_count=0)),
            ('contradictory reviewed draft',lambda b,t,g:b['component_specifications'][0].update(review_status='GO',p0_count=0,p1_count=0,independent_review_bundle_sha256='a'*64)),
            ('review target mismatch',lambda b,t,g:b['component_specifications'][0].update(target_git_commit=C)),
            ('review missing roles',lambda b,t,g:b['component_specifications'][0].update(reviews=[])),
            ('review duplicate roles',lambda b,t,g:b['component_specifications'][0].update(reviews=[{'role':'scientific'},{'role':'scientific'}])),
            ('lifecycle eligibility',lambda b,t,g:b.update(eligible_for_g3=True)),
            ('lifecycle decision',lambda b,t,g:b.update(approval_decision='GO')),
            ('trace artifact',lambda b,t,g:t.update(artifact_kind='wrong')),
            ('trace schema',lambda b,t,g:t.update(schema_version=999)),
            ('trace target mismatch',lambda b,t,g:t.update(target_revision={'sha256':'a'*64})),
            ('bundle target mismatch',lambda b,t,g:b['target_revision'].update(type='git_commit')),
            ('graph schema',lambda b,t,g:g.update(schema_version=999)),
            ('graph artifact',lambda b,t,g:g.update(artifact_kind='wrong')),
            ('legacy catalog omission',lambda b,t,g:g['external_trust_dependency_contract']['nodes'].pop()),
            ('trace missing requirement',lambda b,t,g:t['requirements'].pop()),
            ('trace duplicate requirement',lambda b,t,g:t['requirements'].append(deepcopy(t['requirements'][0]))),
        ]
        for name,mutate in cases:
            with self.subTest(name=name):
                for p,raw in self.original.items():(self.repo/p.relative_to(m.ROOT)).write_bytes(raw)
                b=json.loads(self.original[m.BUNDLE_PATH]);t=json.loads(self.original[m.TRACE_PATH]);g=json.loads(self.original[m.AUTHORITY_GRAPH_PATH])
                mutate(b,t,g);self.save(m.AUTHORITY_GRAPH_PATH,g);self.refresh_hashes(b,t,g)
                self.git('add','.');self.git('commit','-qm','TEST_ONLY malformed historical structure: '+name)
                sha=self.git('rev-parse','HEAD')
                with self.assertRaises((m.G3ValidationError,ValueError)) as error:final(Raw(sha,sha),self.repo)
                self.assertNotIn('publication',str(error.exception))
                self.records.append({'case':name,'sha':sha,'rejected':str(error.exception)})
        (self.directory/'results.json').write_text(json.dumps(self.records,indent=2)+'\n')
        self.assertEqual(self.git('status','--porcelain=v1'),'')

    def test_historical_source_attack_matrix(self):
        r11_line = '| R11-01 |'
        migration_line = next(line for line in self.original[m.MIGRATION_LEDGER].decode().splitlines(keepends=True) if line.startswith(r11_line))
        f0_line = next(line for line in self.original[m.ARCH].decode().splitlines(keepends=True) if line.startswith('| `F-OD-01` |'))
        actor_row = next(line for line in self.original[m.SPECS['F-WIRE']].decode().splitlines(keepends=True) if line.startswith('| PhaseFReviewerActorAttestationV1 |'))
        actor_anchor = '<a id="schema-def-PhaseFReviewerActorAttestationV1"></a>'
        kat_row = next(line for line in self.original[m.SPECS['F-CNF']].decode().splitlines(keepends=True) if line.startswith('| R12-NEG-G3-WRONG-FIELD-NAME |'))

        def mutate_r11(mutator):
            path = self.repo/m.R11_SOURCE.relative_to(m.ROOT)
            path.write_bytes(mutator(path.read_bytes()))

        def duplicate_normative_row():
            path = self.repo/m.NORMATIVE_MATRIX_PATH.relative_to(m.ROOT)
            matrix = json.loads(path.read_text())
            matrix['requirements'].append(deepcopy(matrix['requirements'][0]))
            path.write_text(json.dumps(matrix, sort_keys=True, indent=2)+'\n')

        attacks = [
            ('R1 altered exact-pinned R11 bytes', lambda: mutate_r11(lambda raw: raw[:100] + bytes([raw[100] ^ 1]) + raw[101:])),
            ('R11 truncated', lambda: mutate_r11(lambda raw: raw[:-1])),
            ('R11 extra content', lambda: mutate_r11(lambda raw: raw+b'\nextra')),
            ('R1 missing migration inventory', lambda: self.replace_text(m.MIGRATION_LEDGER, migration_line, '')),
            ('migration inventory duplicate', lambda: self.replace_text(m.MIGRATION_LEDGER, migration_line, migration_line+ migration_line)),
            ('migration unknown ID', lambda: self.replace_text(m.MIGRATION_LEDGER, '| R11-01 |', '| R11-99 |')),
            ('migration wrong R11 reference', lambda: self.replace_text(m.MIGRATION_LEDGER, '| R11-01 |', '| R11-02 |')),
            ('R1 missing required actor-attestation schema anchor', lambda: self.replace_text(m.SPECS['F-WIRE'], actor_anchor+'\n', '')),
            ('schema anchor renamed', lambda: self.replace_text(m.SPECS['F-WIRE'], actor_anchor, actor_anchor.replace('ActorAttestation', 'RenamedActorAttestation'))),
            ('R1 missing required actor-attestation catalog row', lambda: self.replace_text(m.SPECS['F-WIRE'], actor_row, '')),
            ('schema catalog duplicate', lambda: self.replace_text(m.SPECS['F-WIRE'], actor_row, actor_row+actor_row)),
            ('schema catalog wrong authority kind', lambda: self.replace_text(m.SPECS['F-WIRE'], 'PhaseFReviewerActorAttestationV1 | SIGNED_EXTERNAL_AUTHORITY', 'PhaseFReviewerActorAttestationV1 | WRONG_AUTHORITY')),
            ('KAT row removed', lambda: self.replace_text(m.SPECS['F-CNF'], kat_row, '')),
            ('KAT row duplicated', lambda: self.replace_text(m.SPECS['F-CNF'], kat_row, kat_row+kat_row)),
            ('KAT wrong requirement mapping', lambda: self.replace_text(m.SPECS['F-CNF'], kat_row, kat_row.replace('Replace the first key', 'Replace an unrelated field', 1))),
            ('KAT malformed mapping', lambda: self.replace_text(m.SPECS['F-CNF'], kat_row, kat_row.replace('unknown_field', 'malformed_category', 1))),
            ('KAT unsupported schema reference', lambda: self.replace_text(m.NORMATIVE_MATRIX_PATH, 'PhaseFReviewerActorAttestationV1', 'PhaseFUnsupportedSchemaV1')),
            ('wire source section removed', lambda: self.replace_text(m.SPECS['F-WIRE'], '## 4. Current R12 schema catalog closure', '## 4. Removed schema catalog closure')),
            ('wire required heading renamed', lambda: self.replace_text(m.SPECS['F-WIRE'], '## 6. Review gate', '## 6. Renamed review gate')),
            ('architecture F-OD inventory removed', lambda: self.replace_text(m.ARCH, f0_line, '')),
            ('architecture F-OD duplicate', lambda: self.replace_text(m.ARCH, f0_line, f0_line+f0_line)),
            ('normative matrix required row removed', lambda: self.replace_text(m.NORMATIVE_MATRIX_PATH, '"requirement_id": "F-ARCH-001"', '"requirement_id": "F-ARCH-001_REMOVED"')),
            ('normative matrix duplicate row', duplicate_normative_row),
        ]
        for name, mutate in attacks:
            with self.subTest(name=name):
                self.commit_source_attack(name, mutate)
        (self.directory/'results.json').write_text(json.dumps(self.records,indent=2)+'\n')
        self.assertEqual(self.git('status','--porcelain=v1'),'')


if __name__=='__main__': unittest.main(verbosity=2)
