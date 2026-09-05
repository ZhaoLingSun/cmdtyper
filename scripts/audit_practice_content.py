#!/usr/bin/env python3
"""Read-only checks for authored content, references, and three-exercise coverage.

Python 3.11+ is required for tomllib. `bash -n` checks syntax only: no training
command, shell expansion, network probe, or file operation is executed.
"""
from collections import Counter
import json
from pathlib import Path
import re
import shlex
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "data"


def read(path):
    return tomllib.loads(path.read_text())


def main():
    commands, strings, topics = {}, {}, {}
    for path in sorted((DATA / "commands").glob("*.toml")):
        document = read(path)
        for command in document["commands"]:
            identifier = command["id"]
            normalized = command["command"].replace("\r\n", "\n").strip()
            assert identifier not in commands, f"Duplicate ID: {identifier}"
            assert normalized not in strings, f"Duplicate command: {identifier} and {strings.get(normalized)}"
            assert "".join(t["text"] for t in command["tokens"]) == command["command"], identifier
            assert command["dictation"]["answers"][0] == command["command"], identifier
            commands[identifier], strings[normalized] = command, identifier
        if "topic" in document["meta"]:
            topic = document["meta"]["topic"]
            assert topic["id"] not in topics, f"Duplicate topic: {topic['id']}"
            topics[topic["id"]] = [c["id"] for c in document["commands"]] + topic.get("command_ids", [])

    alias_path = DATA / "command_aliases.toml"
    aliases = read(alias_path).get("aliases", {}) if alias_path.exists() else {}

    def resolve(identifier):
        seen = set()
        while identifier in aliases:
            assert identifier not in seen, f"Alias cycle: {identifier}"
            seen.add(identifier)
            identifier = aliases[identifier]
        assert identifier in commands, f"Missing reference: {identifier}"
        return identifier

    for members in topics.values():
        for identifier in members:
            resolve(identifier)

    groups = [g for path in sorted((DATA / "practice").glob("*.toml")) for g in read(path)["groups"]]
    assert len({g["id"] for g in groups}) == len(groups), "Duplicate practice group IDs"
    actual, exercises = Counter(), set()
    for group in groups:
        teaching = resolve(group["teaching_command_id"])
        practice = [resolve(cid) for cid in group["exercise_command_ids"]]
        assert len(practice) == len(set(practice)) == 3, f"Three distinct exercises required: {group['id']}"
        assert teaching not in practice, f"Teaching example reused as exercise: {group['id']}"
        unit = teaching if group["source_kind"] == "lesson" else group["unit_id"]
        actual[(group["source_kind"], group["source_id"], unit)] += 1
        exercises.update(practice)

    def example_id(example):
        if example.get("command_id"):
            return resolve(example["command_id"])
        return strings[example["command"].replace("\r\n", "\n").strip()]

    expected = Counter()
    for path in (DATA / "lessons").glob("*.toml"):
        lesson = read(path)
        for example in lesson["examples"]:
            if example.get("level") == 1:
                expected[("lesson", lesson["meta"]["command"], example_id(example))] += 1
    for kind, directory, units in (("symbol", "symbols", "symbols"), ("system", "system", "sections")):
        for path in (DATA / directory).glob("*.toml"):
            document = read(path)
            for unit in document[units]:
                expected[(kind, document["meta"]["id"], unit["id"])] += 1
    assert actual == expected, f"Coverage mismatch: missing={expected - actual}, extra={actual - expected}"

    # Coverage alone cannot tell whether a drill teaches the promised operation.
    # Keep the direction/option contracts that exposed real curriculum defects.
    group_by_id = {g["id"]: g for g in groups}
    operation_contracts = {
        "v03_33_fields_sort_unique-02": lambda c: bool(re.search(r"sort -[a-z]*n", c)),
        "v03_33_fields_sort_unique-03": lambda c: bool(re.search(r"sort -[a-z]*r", c)),
        "v03_26_grep_context_regex-01": lambda c: bool(re.search(r"grep -[A-Za-z]*[rR]", c)),
        "v03_26_grep_context_regex-02": lambda c: bool(re.search(r"grep -[A-Za-z]*n", c)),
        "v03_26_grep_context_regex-03": lambda c: bool(re.search(r"grep -[A-Za-z]*i", c)),
        "v03_24_redirection_streams-01": lambda c: " > " in c and " >> " not in c,
        "v03_24_redirection_streams-02": lambda c: " >> " in c,
        "v03_24_redirection_streams-04": lambda c: " < " in c,
        "v03_24_pipeline_control-04": lambda c: c.endswith(" | less"),
        "v03_24_pipes_tee-01": lambda c: " | head " in c,
        "v03_24_pipes_tee-02": lambda c: c.startswith("grep ") and c.endswith(" | wc -l"),
        "v03_28_process_inspection-02": lambda c: "--sort=-%cpu" in c,
        "v03_28_ports_signals_jobs-05": lambda c: bool(re.search(r"jobs -[a-z]*p", c)),
        "v03_29_network_downloads-04": lambda c: c.startswith("curl -I "),
        "curl-01": lambda c: c.startswith("curl ") and " -I " not in c,
        "v03_29_network_downloads-01": lambda c: c.startswith("wget ") and "--spider" not in c,
        "wget-01": lambda c: c.startswith("wget ") and "--spider" not in c,
        "v03_31_vim_recovery_readonly-01": lambda c: c.startswith(("view ", "vim -R ")),
        "v03_31_vim_recovery_readonly-02": lambda c: "vim --version" in c,
        "v03_31_vim_recovery_readonly-04": lambda c: "vim --help" in c,
        "v03_31_nano_survival-01": lambda c: "nano --version" in c,
        "v03_31_nano_survival-05": lambda c: c.startswith("nano -l "),
        "v03_31_vim_navigation_editing-07": lambda c: c.startswith("cat -n "),
        "v03_25_alias_source_shell-03": lambda c: "BASH_VERSION" in c or "bash --version" in c,
        "v03_34_locale_variables_encoding-02": lambda c: "LANG" in c,
        "v03_34_locale_variables_encoding-03": lambda c: "locale -a" in c,
        "v03_34_locale_variables_encoding-04": lambda c: "charmap" in c,
        "v03_33_counts_head_tail-05": lambda c: c.startswith("wc -l "),
        "wc-02": lambda c: c.startswith("wc -l "),
        "rm-01": lambda c: c.startswith("rm "),
        "rm-02": lambda c: bool(re.search(r"rm -[a-z]*i", c)),
        "v03_26_find_names_types-02": lambda c: " -iname " in c,
        "v03_26_find_names_types-03": lambda c: " -type f" in c and " -type d" not in c,
        "sed-01": lambda c: bool(re.search(r"sed 's/", c)),
        "v03_21_apt_indexes_search-01": lambda c: "apt-get " in c and "update" in c,
    }
    for suffix, accepts in operation_contracts.items():
        group = group_by_id["practice-lesson-" + suffix]
        for identifier in group["exercise_command_ids"]:
            command = commands[resolve(identifier)]["command"]
            assert accepts(command), f"Off-topic foundation drill: {group['id']}: {command}"

    for suffix, is_download in (("v03_29_scp_rsync-01", False), ("v03_29_scp_rsync-02", True)):
        for identifier in group_by_id["practice-lesson-" + suffix]["exercise_command_ids"]:
            args = shlex.split(commands[resolve(identifier)]["command"])[1:]
            if args[0] == "-P":
                args = args[2:]
            assert len(args) == 2, identifier
            assert (":" in args[0]) == is_download, f"Wrong SCP source direction: {identifier}"
            assert (":" in args[1]) != is_download, f"Wrong SCP target direction: {identifier}"

    for suffix, accepts in {
        "ping-02": lambda c: c.startswith("ping ") and "example.test" in c,
        "v03_25_path_command_diagnostics-04": lambda c: "PATH" in c,
        "v03_31_vim_recovery_readonly-03": lambda c: c.startswith("readlink -f "),
    }.items():
        for identifier in group_by_id["practice-lesson-" + suffix]["exercise_command_ids"]:
            assert accepts(commands[resolve(identifier)]["command"]), f"Missing taught operation: {suffix}"
    history_drills = [commands[resolve(i)]["command"] for i in group_by_id["practice-symbol-special_chars-exclamation"]["exercise_command_ids"]]
    assert sum(c.startswith(("!!", "!-")) and c.endswith(":p") for c in history_drills) >= 2, "History expansion must be practiced"
    assert any(c.startswith("! ") for c in history_drills), "Logical negation must also remain covered"

    release_steps = read(DATA / "scenarios" / "12_git_release.toml")["steps"]
    release_commands = [commands[resolve(s["command_id"])]["command"] for s in release_steps]
    for index, command in enumerate(release_commands):
        if "compose up -d --no-deps" in command:
            following = release_commands[index + 1:index + 3]
            assert following == ["docker compose exec nginx nginx -t", "docker compose exec nginx nginx -s reload"], "Recreated API must refresh the static Nginx upstream"

    stderr = commands[resolve("v04-ref-8aee894088")]["command"]
    assert "\ncat errors.txt" in stderr and "&&" not in stderr, "A failed ls must not prevent reading its error file"
    permission = commands[resolve("v04-ref-eb870b3db3")]["command"]
    assert "-type f" in permission and "-exec chmod 644" in permission, "Do not remove traversal permission from directories"
    port = commands[resolve("v04-ref-9e60a9053e")]["command"]
    assert "sport = :80" in port and "grep :80" not in port, "Port 80 must exclude 8000 and 8080"
    deletion = commands[resolve("v04-ref-0cc3054757")]["command"]
    assert "-print0" in deletion and "xargs -0 -r rm --" in deletion, "Filename boundaries and empty matches must remain safe"

    manifest = json.loads((ROOT / "docs" / "content_expansion_audit.json").read_text())
    baseline = json.loads((ROOT / "docs" / "command_baseline_20260906.json").read_text())
    baseline_strings = {s.replace("\r\n", "\n").strip() for s in baseline.values()}
    authored = {resolve(cid) for cid in manifest["authored_new_ids"]}
    authored_strings = {commands[cid]["command"].replace("\r\n", "\n").strip() for cid in authored}
    assert not authored_strings & baseline_strings, "Authored-new inventory includes a baseline command"
    assert len(authored_strings) >= 400, "At least 400 distinct newly authored commands required"
    assert len(manifest["network_scenarios"]) == 24, "Expected 24 new network diagnosis topics"
    for topic in manifest["network_scenarios"]:
        assert len({resolve(cid) for cid in topics["v04_" + topic]}) >= 10, topic

    # A literal baseline fragment is conservatively excluded from this stronger
    # count too. This does not prove teaching quality, but prevents old compound
    # commands being counted merely because their pieces get standalone IDs.
    nonfragment_authored = {s for s in authored_strings if not any(s in old for old in baseline_strings)}
    assert len(nonfragment_authored) >= 400, "New-content count relies on baseline command fragments"

    sequences = read(DATA / "sequences" / "workflows.toml")["sequences"]
    assert len({s["id"] for s in sequences}) == len(sequences), "Duplicate sequence IDs"
    sequence_by_id = {s["id"]: s for s in sequences}
    sequence_steps = []
    for sequence in sequences:
        steps = sequence["steps"]
        assert [resolve(s["command_id"]) for s in steps] == [resolve(i) for i in sequence["command_ids"]], f"Context order differs from commands: {sequence['id']}"
        assert steps and "独立还原" in steps[0]["explanation"], f"Missing sequence starting context: {sequence['id']}"
        for step in steps:
            assert isinstance(step["output"], str), "Silence is an explicit empty string, not absent output"
            assert len(step["explanation"].strip()) >= 20, f"Missing specific explanation: {step['command_id']}"
            command = commands[resolve(step["command_id"])]
            assert "分步操作：确认本条成功后再继续下一条" not in str(command), step["command_id"]
            # Successful assignments and these non-verbose mutations print no
            # success banner. Their state changes belong in the explanation.
            if re.match(r"^(?:[A-Za-z_][A-Za-z0-9_]*=|umask |chmod |cd |mkdir |touch |ln |truncate |rmdir )", command["command"]):
                assert step["output"] == "", f"Invented output for silent operation: {step['command_id']}"
        sequence_steps.extend(steps)

    def last_output(identifier):
        return sequence_by_id[identifier]["steps"][-1]["output"]

    assert last_output("tar-backup-etc") == "./etc-backup-d4E5f6.tar.gz\n"
    assert last_output("tar-exclude-vcs") == "./release-j1K2l3.tar.gz\n", "Shared printf must use the current sequence's archive"
    assert "/cmdtyper-deploy-key.A1b2C3/id_ed25519_deploy.pub" in last_output("ssh-keygen-ed25519")
    assert "/cmdtyper-work-key.G7h8I9/work_ed25519.pub" in last_output("ssh-keygen-ed25519-basic"), "Shared printf must use this sequence's key paths"
    assert last_output("redirect-both-after-stdout") == ""
    assert "No such file or directory" in last_output("redirect-order-stderr-terminal"), "Redirection order must retain the terminal diagnostic"
    assert "退出状态为 2" in sequence_by_id["redirect-silence-stderr"]["steps"][-1]["explanation"], "A hidden error is not successful grep"
    assert last_output("tee-display-and-save") == "build complete\n"
    assert last_output("tee-append-monitor-log") == "health check passed\n", "tee -a emits the new line, not previous file contents"
    assert last_output("fileops-stat-settings-metadata") == f"regular file 600 {len('mode=safe'.encode()) + 1} settings.ini\n"
    assert last_output("fileops-stat-report-size") == f"report.csv {24 * 1024} bytes\n"
    assert last_output("fileops-rm-force-recursive-warning") == ".keep\nthumbnails\n", "The preview must include hidden names without deleting"

    # Every foundation group has a separately reviewed core capability and
    # three command-specific rationales. Validate semantic operators/options,
    # not just a shared executable name or the number of references.
    semantic_review = json.loads((ROOT / "docs" / "foundation_semantic_review_20260906.json").read_text())
    semantic_groups = semantic_review["groups"]
    assert len(semantic_groups) == len(groups) == semantic_review["reviewed_groups"]
    assert {r["group_id"] for r in semantic_groups} == set(group_by_id), "Semantic review must cover every foundation group"
    for reviewed in semantic_groups:
        group = group_by_id[reviewed["group_id"]]
        assert reviewed["teaching_command_id"] == resolve(group["teaching_command_id"])
        assert reviewed["teaching_command"] == commands[resolve(group["teaching_command_id"])]["command"]
        assert [r["command_id"] for r in reviewed["exercises"]] == [resolve(i) for i in group["exercise_command_ids"]], f"Stale semantic review: {group['id']}"
        assert len(reviewed["core_ability"]) >= 10 and reviewed["all_regex"], group["id"]
        assert [r["role"] for r in reviewed["exercises"]] == ["原用法", "参数变化", "小任务迁移"]
        for exercise in reviewed["exercises"]:
            command = commands[exercise["command_id"]]["command"]
            assert exercise["command"] == command, f"Stale rationale: {exercise['command_id']}"
            assert len(exercise["rationale"]) >= 10, f"Missing rationale: {exercise['command_id']}"
            assert all(re.search(pattern, command) for pattern in reviewed["all_regex"]), f"Core operation missing: {group['id']}: {command}"
            assert not any(re.search(pattern, command) for pattern in reviewed["forbidden_regex"]), f"Adjacent operation substituted: {group['id']}: {command}"
            if "grep_pattern_regex" in reviewed:
                args = shlex.split(command)
                position = args.index("grep") + 1
                flags = []
                while args[position].startswith("-"):
                    flags.append(args[position])
                    position += 1
                assert re.search(reviewed["grep_pattern_regex"], args[position]), f"Regex operator absent from actual grep pattern: {group['id']}"
                if reviewed.get("grep_requires_ere"):
                    assert any(re.fullmatch(r"-[A-Za-z]*E[A-Za-z]*", flag) for flag in flags), f"Extended regex requires grep -E: {group['id']}"

    # Include the newly authored short exercises in syntax validation, while
    # keeping legacy reference promotions separate from the new-content count.
    new_syntax_ids = authored | {cid for cid in commands if cid.startswith("v04-ex-")}
    for identifier in sorted(new_syntax_ids):
        result = subprocess.run(["bash", "--noprofile", "--norc", "-n", "-c", commands[identifier]["command"]], capture_output=True, text=True)
        assert result.returncode == 0, f"Shell syntax error in {identifier}: {result.stderr}"

    print(json.dumps({
        "canonical_commands": len(commands),
        "authored_distinct_commands": len(authored_strings),
        "authored_not_baseline_substrings": len(nonfragment_authored),
        "network_scenarios": len(manifest["network_scenarios"]),
        "practice_groups": len(groups),
        "coverage_by_source": dict(Counter(g["source_kind"] for g in groups)),
        "exercise_references": len(groups) * 3,
        "unique_exercise_commands": len(exercises),
        "bash_syntax_checked": len(new_syntax_ids),
        "specific_operation_contracts": len(operation_contracts) + 10,
        "foundation_semantic_contracts": len(semantic_groups),
        "foundation_semantic_exercise_checks": sum(len(r["exercises"]) for r in semantic_groups),
        "contextual_sequences": len(sequences),
        "contextual_sequence_steps": len(sequence_steps),
        "silent_sequence_steps": sum(s["output"] == "" for s in sequence_steps),
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
