#!/usr/bin/env python3
"""cmdtyper Token 覆盖率审计脚本。

扫描命令题库和 lesson 数据，输出：
- token 覆盖率统计
- 空泛描述检测
- 高频 token 缺口
- 按命令家族的改造优先级
"""

import sys
import os
from pathlib import Path
from collections import Counter, defaultdict

try:
    import tomllib
except ImportError:
    import tomli as tomllib

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
    lesson_cmds = set()
    for lesson in lessons:
        for ex in lesson.get("examples", []):
            if ex.get("token_details"):
                lesson_cmds.add(ex.get("command", ""))

    covered = sum(1 for c in commands if c.get("command", "") in lesson_cmds)
    uncovered = total_cmds - covered

    print(f"\n📈 Lesson token_details 覆盖率:")
    pct = 100 * covered // max(total_cmds, 1)
    print(f"  有精细拆解的命令: {covered}/{total_cmds} ({pct}%)")
    print(f"  缺少精细拆解的命令: {uncovered}")

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
        with_details = sum(1 for c in cmds if c.get("command", "") in lesson_cmds)
        family_stats.append((family, total, with_details))

    for family, total, with_details in sorted(family_stats, key=lambda x: -(x[1] - x[2]))[:20]:
        status = "✅" if with_details == total else "⚠️" if with_details > 0 else "❌"
        print(f"  {status} {family}: {with_details}/{total} covered")

    print(f"\n{'='*60}")
    print("审计完成")
    return len(generic_tokens) == 0 and covered > total_cmds * 0.5


if __name__ == "__main__":
    ok = audit()
    sys.exit(0 if ok else 1)
