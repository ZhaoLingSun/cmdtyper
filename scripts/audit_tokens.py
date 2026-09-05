#!/usr/bin/env python3
"""cmdtyper Token 覆盖率审计脚本。

扫描命令题库和 lesson 数据，输出：
- token 覆盖率统计
- 空泛描述检测
- 高频 token 缺口
- 按命令家族的改造优先级
"""

import sys
from collections import Counter, defaultdict
from pathlib import Path

import tomllib

DATA_DIR = Path(__file__).parent.parent / "data"

GENERIC_PATTERNS = [
    "命令参数，用于指定",
    "指定要处理的对象或行为细节",
    "目标对象",
    "某种选项",
    "用于控制输出",
]


def load_commands():
    commands = []
    for path in sorted((DATA_DIR / "commands").glob("*.toml")):
        data = tomllib.loads(path.read_text())
        for cmd in data.get("commands", []):
            cmd["_file"] = path.name
            commands.append(cmd)
    return commands


def load_lessons():
    lessons = []
    for path in sorted((DATA_DIR / "lessons").glob("*.toml")):
        data = tomllib.loads(path.read_text())
        data["_file"] = path.name
        lessons.append(data)
    return lessons


def audit():
    commands = load_commands()
    lessons = load_lessons()

    print("=" * 60)
    print("cmdtyper Token 覆盖率审计报告")
    print("=" * 60)

    # 1. Basic stats
    total_cmds = len(commands)
    total_tokens = sum(len(c.get("tokens", [])) for c in commands)
    unique_tokens = set()
    for c in commands:
        for t in c.get("tokens", []):
            unique_tokens.add(t.get("text", ""))

    lesson_examples = []
    lesson_token_details_count = 0
    for lesson in lessons:
        for ex in lesson.get("examples", []):
            lesson_examples.append(ex)
            lesson_token_details_count += len(ex.get("token_details", []))

    print(f"\n📊 基本统计:")
    print(f"  命令总数: {total_cmds}")
    print(f"  题库 token 总数: {total_tokens}")
    print(f"  唯一 token 文本: {len(unique_tokens)}")
    print(f"  lesson 示例总数: {len(lesson_examples)}")
    print(f"  lesson token_details 总数: {lesson_token_details_count}")

    # 2. Coverage
    command_ids_by_text = defaultdict(set)
    canonical_ids = set()
    for cmd in commands:
        command_id = cmd.get("id")
        if command_id:
            canonical_ids.add(command_id)
            command_ids_by_text[cmd.get("command", "")].add(command_id)

    inline_covered_ids = set()
    command_id_covered_ids = set()
    command_id_ref_locations = defaultdict(list)
    dangling_refs = []

    for lesson in lessons:
        for example_index, ex in enumerate(lesson.get("examples", []), start=1):
            inline_command = ex.get("command")
            if inline_command is not None:
                inline_covered_ids.update(command_ids_by_text.get(inline_command, ()))

            if "command_id" not in ex:
                continue

            command_id = ex.get("command_id")
            location = f"{lesson['_file']} example #{example_index}"
            command_id_ref_locations[command_id].append(location)
            if command_id in canonical_ids:
                command_id_covered_ids.add(command_id)
            else:
                dangling_refs.append((command_id, location))

    duplicate_refs = {
        command_id: locations
        for command_id, locations in command_id_ref_locations.items()
        if len(locations) > 1
    }
    duplicate_ref_count = sum(len(locations) - 1 for locations in duplicate_refs.values())
    covered_ids = inline_covered_ids | command_id_covered_ids
    covered = len(covered_ids)
    uncovered = total_cmds - covered

    print(f"\n📈 Lesson 命令覆盖率:")
    pct = 100 * covered // max(total_cmds, 1)
    print(f"  内联命令链接: {len(inline_covered_ids)}")
    print(f"  command_id 链接: {len(command_id_covered_ids)}")
    print(f"  唯一覆盖命令: {covered}/{total_cmds} ({pct}%)")
    print(f"  缺少 Lesson 链接的命令: {uncovered}")
    print(f"  悬空 command_id 引用: {len(dangling_refs)}")
    print(f"  重复 command_id 引用: {duplicate_ref_count}")

    for command_id, location in sorted(
        dangling_refs, key=lambda item: (str(item[0]), item[1])
    ):
        print(f"    dangling {command_id!r}: {location}")

    for command_id, locations in sorted(
        duplicate_refs.items(), key=lambda item: str(item[0])
    ):
        print(f"    duplicate {command_id!r}: {', '.join(locations)}")

    # 3. Generic descriptions
    generic_tokens = []
    for cmd in commands:
        for tok in cmd.get("tokens", []):
            desc = tok.get("desc", "")
            if any(p in desc for p in GENERIC_PATTERNS):
                generic_tokens.append({
                    "file": cmd["_file"],
                    "command": cmd.get("command", ""),
                    "token": tok.get("text", ""),
                    "desc": desc,
                })

    print(f"\n⚠️  空泛描述检测:")
    print(f"  含空泛模板的 token 数: {len(generic_tokens)}")
    if generic_tokens:
        for g in generic_tokens[:20]:
            print(f"    [{g['file']}] {g['command']} → {g['token']}: {g['desc'][:50]}")

    # 4. High frequency tokens without kind
    token_freq = Counter()
    tokens_without_kind = []
    for cmd in commands:
        for tok in cmd.get("tokens", []):
            text = tok.get("text", "").strip()
            if text:
                token_freq[text] += 1
                if not tok.get("kind"):
                    tokens_without_kind.append(text)

    no_kind_freq = Counter(tokens_without_kind)
    print(f"\n🏷️  Token kind 标注:")
    total_with_kind = total_tokens - len(tokens_without_kind)
    pct2 = 100 * total_with_kind // max(total_tokens, 1)
    print(f"  已标注 kind: {total_with_kind}/{total_tokens} ({pct2}%)")
    high_freq_no_kind = [(t, n) for t, n in no_kind_freq.most_common(20) if n >= 3]
    if high_freq_no_kind:
        print(f"  高频未标注 token (>=3次):")
        for t, n in high_freq_no_kind:
            print(f"    {n}x  {t}")

    # 5. Short descriptions
    short_descs = []
    for cmd in commands:
        for tok in cmd.get("tokens", []):
            desc = tok.get("desc", "")
            if 0 < len(desc) < 8:
                short_descs.append({
                    "command": cmd.get("command", ""),
                    "token": tok.get("text", ""),
                    "desc": desc,
                })

    print(f"\n📝 过短描述 (<8字符):")
    print(f"  数量: {len(short_descs)}")
    for s in short_descs[:15]:
        print(f"    {s['command']} → {s['token']}: \"{s['desc']}\"")

    # 6. Command family priority
    families = defaultdict(list)
    for cmd in commands:
        first_word = cmd.get("command", "").split()[0] if cmd.get("command") else "unknown"
        families[first_word].append(cmd)

    print(f"\n🎯 命令家族覆盖优先级:")
    family_stats = []
    for family, cmds in sorted(families.items(), key=lambda x: -len(x[1])):
        total = len(cmds)
        with_details = sum(1 for c in cmds if c.get("id") in covered_ids)
        family_stats.append((family, total, with_details))

    for family, total, with_details in sorted(family_stats, key=lambda x: -(x[1] - x[2]))[:20]:
        status = "✅" if with_details == total else "⚠️" if with_details > 0 else "❌"
        print(f"  {status} {family}: {with_details}/{total} covered")

    print(f"\n{'='*60}")
    print("审计完成")
    return (
        len(generic_tokens) == 0
        and covered > total_cmds * 0.5
        and len(dangling_refs) == 0
    )


if __name__ == "__main__":
    ok = audit()
    sys.exit(0 if ok else 1)
