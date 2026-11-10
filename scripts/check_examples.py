"""Recheck the shipped snippets; --core runs the lightweight package only."""
from pathlib import Path
import argparse,subprocess,os,json,hashlib,shutil,tempfile
R=Path(__file__).resolve().parents[1]
a=argparse.ArgumentParser();a.add_argument('--core',action='store_true');args=a.parse_args()
projects=['core'] if args.core else ['core','migration','guides','integrations-check','integrations-local','integrations-lance','integrations-surreal']
env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(R/'examples-target'))
logs=R/'validation-logs';logs.mkdir(exist_ok=True)
work=tempfile.TemporaryDirectory(prefix='rig-docs-offline-')
def run(label,cmd,stdin=None,override=None):
 print(label,flush=True)
 result=subprocess.run(cmd,cwd=work.name if 'run' in cmd else R,env={**env,**(override or {})},input=stdin,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 (logs/(label+'.log')).write_text(result.stdout)
 if result.returncode:raise SystemExit(f'{label} failed; see validation-logs/{label}.log')
def cargo(name,*cmd):return ['cargo',*cmd,'--locked','--manifest-path',str(R/'examples'/name/'Cargo.toml')]
# Reject a code overlay edited independently of its compiler input.
v=json.loads((R/'validation.json').read_text())
for key,row in v['blocks'].items():
 page,index=key.split(':');code=json.loads((R/'content/codes'/f'{page}.codes.json').read_text())[int(index)]['text']
 if code.strip() not in (R/row['file']).read_text():raise SystemExit(f'{key}: displayed code differs from compiler input')
if not args.core and not (env.get('PROTOC') or shutil.which('protoc')):
 raise SystemExit('完整检查的 LanceDB 依赖需要 protoc。先安装 protobuf，或设置 PROTOC 为其可执行文件路径。可先运行 --core。')
for p in projects:run('check-'+p,cargo(p,'check','--all-targets','--all-features'))
run('core-offline',cargo('core','test','--test','offline'))
if not args.core:
 for name,opts,stdin in [('offline',[],None),('p16_2',[],None),('p05_8',[],None),('p05_6',[],None),('p27_6',[],None),('p27_7',[],None),('p22_2',[],None),('p22_3',[],None),('p10_0',['--offline'],'y\n'),('p10_0',['--offline'],'n\n')]:
  run('migration-'+name+('-deny' if stdin=='n\n' else ''),cargo('migration','run','--example',name)+(['--']+opts if opts else []),stdin,{'OPENAI_API_KEY':'offline-placeholder','DEEPSEEK_API_KEY':'offline-placeholder'})
 run('custom-provider',cargo('guides','run','--example','custom_provider'))
 run('flight-format',cargo('guides','test','--features','flight','--example','flight'))
 run('mcp-offline',cargo('integrations-check','run','--example','p47_005'))
 for name in ['p39_001','p39_002','p39_003','p39_004','p43_003','p45_003']:
  run('offline-'+name,cargo('integrations-check','run','--example',name))
for row in v['blocks'].values():
 if any(row['file'].startswith('examples/'+p+'/') for p in projects):
  row['status']='passed';row['sha256']=hashlib.sha256((R/row['file']).read_bytes()).hexdigest()
(R/'validation.json').write_text(json.dumps(v,ensure_ascii=False,indent=2))
work.cleanup()
print('PASS：编译与离线运行通过。未调用真实外部服务。')
