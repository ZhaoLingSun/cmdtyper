#!/usr/bin/env python3
"""Merge learning commands into a canonical catalog; preserve teaching context and progress aliases.
Run only after authoring, then commit generated content with its inventory report.
"""
from pathlib import Path
import hashlib,json,tomllib
ROOT=Path(__file__).resolve().parents[1]
DATA=ROOT/'data'

def value(v):
    return json.dumps(v,ensure_ascii=False)
def dump(document):
    out=[]
    def table(d,path=(),header=None):
        if header:out.append(header)
        for k,v in d.items():
            if not isinstance(v,dict) and not (isinstance(v,list) and v and isinstance(v[0],dict)):
                out.append(f'{k} = {value(v)}')
        out.append('')
        for k,v in d.items():
            joined='.'.join((*path,k))
            if isinstance(v,dict):table(v,(*path,k),f'[{joined}]')
            elif isinstance(v,list) and v and isinstance(v[0],dict):
                for item in v:table(item,(*path,k),f'[[{joined}]]')
    table(document)
    return '\n'.join(out).rstrip()+'\n'

def main():
    registry={};by_id={};meta={};promoted=[];promotion_meta={};aliases={};rewrites=0;legacy_candidates={};current_meta={}
    for p in sorted((DATA/'commands').glob('*.toml')):
        d=tomllib.loads(p.read_text())
        for c in d['commands']:
            key=c['command'].replace('\r\n','\n').strip()
            if key in registry and registry[key]['id']!=c['id']:
                aliases[c['id']]=registry[key]['id']
            else:registry[key]=c;by_id[c['id']]=c;meta[c['id']]=d['meta']
    migration_path=ROOT/'docs/workflow_migration_20260906.json'
    old_strings={x['old']:by_id.get(id) for id,x in json.loads(migration_path.read_text()).items()} if migration_path.exists() else {}
    def register(command,summary,output=None):
        canonical=old_strings.get(command) or registry.get(command.replace('\r\n','\n').strip())
        if canonical:return canonical['id']
        identifier='shared-'+hashlib.sha256(command.encode()).hexdigest()[:16]
        c={'id':identifier,'command':command,'summary':summary or '按题目给出的上下文练习该命令','tokens':[{'text':command,'desc':summary or '命令输入'}],'dictation':{'prompt':summary or '输入本题命令','answers':[command]}}
        if output is not None:c['simulated_output']=output
        registry[command.strip()]=c;by_id[identifier]=c;promoted.append(c)
        category=current_meta.get('category','system')
        if category not in {'file_ops','permission','text_process','search','process','network','archive','system','pipeline','scripting'}:category='system'
        promotion_meta[identifier]=(category,current_meta.get('difficulty','basic'))
        return identifier
    alias_path=DATA/'command_aliases.toml'
    if alias_path.exists():aliases.update(tomllib.loads(alias_path.read_text()).get('aliases',{}))
    def reference(item,legacy=None,field='command',summary='summary'):
        nonlocal rewrites
        old_id=item.get('command_id')
        canonical=by_id.get(aliases.get(old_id,old_id)) if old_id else None
        text=item.get(field) or (canonical['command'] if canonical else None)
        if not text:return
        id=canonical['id'] if canonical else register(text,item.get(summary,''),item.get('simulated_output'))
        if legacy and legacy!=id and legacy not in by_id:legacy_candidates.setdefault(legacy,set()).add(id)
        item['command_id']=id
        item.pop(field,None)
        rewrites+=1
    for p in sorted((DATA/'lessons').glob('*.toml')):
        d=tomllib.loads(p.read_text());name=d['meta']['command'];current_meta=d['meta']
        for i,e in enumerate(d['examples']):
            old=e.get('command_id'); eid=e.get('id')
            legacy=old or (f'lesson:{name}:{eid}' if eid else name)
            e.setdefault('id',f'example-{i+1:03}')
            reference(e,legacy)
        p.write_text(dump(d))
    for p in sorted((DATA/'symbols').glob('*.toml')):
        d=tomllib.loads(p.read_text());name=d['meta']['id'];current_meta=d['meta']
        for symbol in d['symbols']:
            for i,e in enumerate(symbol['examples']):
                e.setdefault('id',f'{symbol["id"]}-example-{i+1:03}');reference(e,summary='explanation')
        for i,e in enumerate(d.get('exercises',[])):
            legacy=e.get('command_id') or (f'symbol:{name}:{e["id"]}' if e.get('id') else f'symbol:{name}:typing:{i}')
            e.setdefault('id',f'exercise-{i+1:03}')
            if not e.get('command') and not e.get('command_id') and e.get('answers'):e['command']=e['answers'][0]
            reference(e,legacy,summary='prompt')
        p.write_text(dump(d))
    for p in sorted((DATA/'system').glob('*.toml')):
        d=tomllib.loads(p.read_text());name=d['meta']['id'];current_meta=d['meta']
        for si,section in enumerate(d['sections']):
            for ci,c in enumerate(section.get('commands',[])):
                legacy=c.get('command_id') or (f'system:{name}:{section["id"]}:{c["id"]}' if c.get('id') else f'system:{name}:{si}:{ci}')
                c.setdefault('id',f'command-{ci+1:03}');reference(c,legacy)
        p.write_text(dump(d))
    for p in sorted((DATA/'scenarios').glob('*.toml')):
        d=tomllib.loads(p.read_text());current_meta=d
        for step in d['steps']:
            command=step.get('command') or by_id[step['command_id']]['command']
            id=register(command,step['instruction'],None)
            aliases[f'scenario:{d["id"]}:{step["id"]}']=id
            step['command_id']=id
            step.pop('command',None)
        p.write_text(dump(d))
    for p in sorted((DATA/'sequences').glob('*.toml')):
        d=tomllib.loads(p.read_text());current_meta={'category':'system','difficulty':'basic'}
        for seq in d['sequences']:
            if seq.get('commands'):
                seq['command_ids']=[register(command,'分步操作：确认本条成功后再继续下一条') for command in seq.pop('commands')]
        p.write_text(dump(d))
    if promoted:
        buckets={}
        for command in promoted:buckets.setdefault(promotion_meta[command['id']],[]).append(command)
        for (category,difficulty),commands in buckets.items():
            p=DATA/'commands'/f'zz_shared_{category}_{difficulty}.toml'
            d=tomllib.loads(p.read_text()) if p.exists() else {'meta':{'category':category,'difficulty':difficulty,'description':'跨模块共用的规范命令（由内容归一化维护）'},'commands':[]}
            d['commands'].extend(commands);p.write_text(dump(d))
    # Remove duplicate definitions while retaining each topic's membership by reference.
    for p in sorted((DATA/'commands').glob('*.toml')):
        d=tomllib.loads(p.read_text());removed=[c['id'] for c in d['commands'] if c['id'] in aliases]
        if removed:
            d['commands']=[c for c in d['commands'] if c['id'] not in aliases]
            if d['meta'].get('topic'):
                refs=d['meta']['topic'].setdefault('command_ids',[])
                refs.extend(aliases[id] for id in removed if aliases[id] not in refs)
            p.write_text(dump(d))
    for old,targets in legacy_candidates.items():
        if len(targets)==1:aliases[old]=next(iter(targets))
        else:aliases.pop(old,None)
    # Flatten chains for deterministic one-hop runtime migration.
    for old in list(aliases):
        seen={old};new=aliases[old]
        while new in aliases:
            if new in seen:raise ValueError(f'alias cycle {old}')
            seen.add(new);new=aliases[new]
        if old==new:del aliases[old]
        else:aliases[old]=new
    # Alias keys can contain colons or spaces and therefore must be TOML-quoted.
    alias_path.write_text('[aliases]\n'+''.join(f'{value(k)} = {value(v)}\n' for k,v in sorted(aliases.items())))
    report={'canonical_commands':len(by_id),'promoted_reference_commands':len(promoted),'references_rewritten':rewrites,'aliases':len(aliases)}
    (ROOT/'docs/canonicalization_audit.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
