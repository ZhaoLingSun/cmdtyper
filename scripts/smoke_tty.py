#!/usr/bin/env python3
"""Exercise the actual TUI through a PTY using isolated practice data."""
import argparse
import codecs
import re
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time
import tomllib

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--docker", action="store_true")
parser.add_argument("--workflow", action="store_true")
parser.add_argument("--output-dir", type=Path, help="Directory for recordings and JSON results")
args = parser.parse_args()
commands = {command["id"]: command["command"]
            for source in (ROOT / "data/commands").glob("*.toml")
            for command in tomllib.loads(source.read_text())["commands"]}
scenario = tomllib.loads((ROOT / "data/scenarios/01_recovered_host.toml").read_text())
output = args.output_dir or (ROOT / "docs/validation")
output.mkdir(parents=True, exist_ok=True)
name = "docker" if args.docker else "release"
kind = "workflow" if args.workflow else "smoke"
recording = output / f"{name}-{kind}.cast"
with tempfile.TemporaryDirectory(prefix="cmdtyper-tty-smoke-") as user_dir:
    workflow = next(sequence for sequence in tomllib.loads((ROOT / "data/sequences/workflows.toml").read_text())["sequences"]
                    if sequence["id"] == "tar-xzf-website")
    if args.workflow:
        # Resume a published lesson through the application's normal saved-screen loader.
        (Path(user_dir) / "resume_state.json").write_text(json.dumps({
            "screen": "command_lesson_practice", "lesson_command": "tar-extract-safety", "example_index": 2,
            "category_index": 0, "command_index": 0, "topic_index": 0, "symbol_index": 0, "section_index": 0}))
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 36, 120, 0, 0))
    environment = dict(os.environ, TERM="xterm-256color", TZ="Asia/Shanghai",
                       CMDTYPER_DATA_DIR=str(ROOT / "data"), CMDTYPER_USER_DIR=user_dir)
    command = (["docker", "run", "--rm", "-it", "--network=none", "-e", "TZ=Asia/Shanghai",
                "-v", f"{user_dir}:/userdata", "cmdtyper:v0.4"] if args.docker
               else [str(ROOT / "target/release/cmdtyper")])
    started = time.monotonic()
    events = [{"version": 2, "width": 120, "height": 36, "timestamp": int(time.time()),
               "title": f"cmdtyper v0.4 {name}: " + ("sequential workflow with per-step output" if args.workflow else "calendar, guided typing, evidence and resume"),
               "env": {"TERM": "xterm-256color", "SHELL": "/bin/bash"}}]
    process = subprocess.Popen(command, stdin=slave, stdout=slave, stderr=slave,
                               cwd=ROOT, env=environment, start_new_session=True)
    os.close(slave)
    captured = bytearray()
    decoder = codecs.getincrementaldecoder("utf-8")("replace")

    def pump(seconds=0.2):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
            if not ready:
                break
            try:
                chunk = os.read(master, 65536)
            except OSError:
                break
            if not chunk:
                break
            captured.extend(chunk)
            events.append([round(time.monotonic() - started, 5), "o", decoder.decode(chunk)])

    def send(value, pause=0.2):
        encoded = value.encode()
        events.append([round(time.monotonic() - started, 5), "i", value])
        os.write(master, encoded)
        pump(pause)

    def state():
        return json.loads((Path(user_dir) / "scenario_progress.json").read_text())

    def phase(expected, step):
        current = state()
        assert (current["phase"], current["step_index"]) == (expected, step), current

    try:
        pump(2)
        if args.workflow:
            for step in workflow["steps"]:
                for character in commands[step["command_id"]]:
                    send(character, 0.025)
                send("\r", 0.3)
                if step["output"]:
                    terminal_text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", captured.decode("utf-8", "replace"))
                    assert step["output"].splitlines()[0][:35] in terminal_text
            send("\r")
        else:
            for _ in range(5):
                send("j")
            send("\r")  # calendar
            send("\x1b", 0.35)
            for _ in range(4):
                send("k")
            send("\r")  # learning center
            for _ in range(5):
                send("j")
            send("\r")  # scenarios
            send("\r")  # introduction
            send("\r")  # first target
            phase("typing", 0)
            send("~~")  # repeated error at the same first target position
            for index, step in enumerate(scenario["steps"][:3]):
                for character in commands[step["command_id"]]:
                    send("\r" if character == "\n" else character, 0.04)
                send("\r")
                phase("evidence", index)
                send("\r")
            phase("decision", 2)
            send("j")
            send("\r")  # incorrect answer returns to evidence
            phase("evidence", 2)
            send("\r")
            send("\r")  # correct answer advances
            phase("typing", 3)
            for character in commands[scenario["steps"][3]["command_id"]][:2]:
                send(character, 0.08)
            send("\x1b", 0.35)  # save incomplete target
            send("\r")  # resume same scenario step
            phase("typing", 3)
            send("\x1b", 0.35)
            send("\x1b", 0.35)
            send("\x1b", 0.35)
            for _ in range(4):
                send("j")
            send("\r")  # populated calendar
            send("\x1b[D")
            send("\x1b[C")
        send("\x03", 0.4)
        assert process.wait(timeout=5) == 0
        history = json.loads((Path(user_dir) / "history.json").read_text())
        if args.workflow:
            assert len(history) == 1, history
            assert history[0]["command_id"] == workflow["id"]
            assert history[0]["mode"] == "lesson_practice"
            assert history[0]["typing"]["completed"]
            assert history[0]["error_count"] == 0
            assert len(history[0]["typing"]["positions"]) == len(commands[workflow["id"]].replace("\n", ""))
            summary = {"environment": name, "exit_code": 0, "workflow": workflow["id"],
                       "steps": len(workflow["steps"]), "records": 1, "completed": 1,
                       "step_output_checked": True, "first_character_after_output_preserved": True}
        else:
            assert len(history) == 4, history
            assert all(record["mode"] == "scenario_typing" for record in history)
            assert history[0]["error_count"] == 1, history[0]
            assert not history[-1]["typing"]["completed"], history[-1]
            assert len({record["id"] for record in history}) == 4
            summary = {"environment": name, "exit_code": 0, "records": len(history),
                       "completed": 3, "partial": 1, "same_position_errors": history[0]["error_count"],
                       "wrong_decision_returns_to_evidence": True, "resume_step": 4}
        assert b"panic" not in captured.lower()
        summary["elapsed_seconds"] = round(time.monotonic() - started, 2)
        (output / f"{name}-{kind}.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary))
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=5)
        os.close(master)
        recording.write_text("\n".join(json.dumps(event, ensure_ascii=False) for event in events) + "\n")
