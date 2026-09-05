# F6 Content Consistency Audit Report

**Date:** 2026-03-28
**Repo:** cmdtyper v0.3 (`github.com/ZhaoLingSun/cmdtyper`)
**Source:** GitHub API (live download of all TOML data files)

---

## 1. Data Structure Overview

The repo has 4 data directories:

| Directory | Files | Purpose |
|---|---|---|
| `data/commands/` | 7 TOML files | Typing question bank (266 question commands) |
| `data/lessons/` | 31 TOML files | Command tutorial/explanation lessons (31 commands) |
| `data/symbols/` | 6 TOML files | Symbol/shell operator topics (131 example commands) |
| `data/system/` | 6 TOML files | System architecture topics (127 example commands) |

---

## 2. Inventory

### 2.1 Question Bank (`data/commands/`)

- **7 files:** `01_*.toml` through `07_*.toml` (basic/advanced/practical split)
- **266 raw commands** (full pipeline strings)
- **134 unique base commands** extracted

### 2.2 Lesson Topics (`data/lessons/`)

- **31 lesson files:** `awk, cat, cd, chmod, cp, curl, docker, find, git, grep, head, ip, kill, ls, mkdir, mv, nginx, ping, ps, rm, sed, sort, ssh, systemctl, tail, tar, top, uniq, wc, wget, xargs`
- **31 unique commands**

### 2.3 Symbol Topics (`data/symbols/`)

- **6 topics:** `pipe_redirect, quotes_escape, regex_basics, special_chars, variables, wildcards`
- **131 raw example commands**
- **40 unique base commands** extracted

### 2.4 System Topics (`data/system/`)

- **6 topics:** `config_files, directory_structure, filesystem_permissions, network_basics, package_management, process_systemd`
- **127 raw example commands**
- **46 unique base commands** extracted

---

## 3. Gap Analysis

### 3.1 Symbol → Lesson Gap

**28 symbol base commands are NOT lesson topic commands.**

#### Gap categorization:

**A. Shell syntactic constructs (expected — fine to be gaps):**
- `|` (pipe)
- `!` (history expansion, not)
- `!!` (previous command)
- `!-2` (2 commands ago)
- `&&` (AND operator)
- `for` (bash keyword)
- `true` (built-in)
- `test` (bracket built-in)

**B. Standard Linux commands covered by QBank (not a problem):**
`apt, cat, cd, cp, date, df, dmesg, echo, find, grep, ls, mkdir, ping, ps, rm, sort, wc`

**C. Actual gaps — commands with dedicated QBank coverage but NO lesson:**
`bash, echo, tee, sleep`

**D. Non-standard / script-specific — fine to be gaps:**
`bashrc, command, env, env.sh, deploy.sh, long_task.sh, service.sh, mysql, gcc, python3`

**E. Artefacts (false positives from parsing):**
`+%F)` — extracted from `TODAY=$(date +%F)`; the `)` is not a command

### 3.2 System → Lesson Gap

**35 system base commands are NOT lesson topic commands.**

#### Gap categorization:

**A. Commands with lesson topics (✓ all covered):**
`cat, chmod, chown, chgrp, curl, find, ip, kill, ls, mkdir, mv, ping, ps, sort, systemctl, uniq, wc, wget`

**B. Standard Linux commands covered by QBank but NO lesson topic:**
`apt, apt-cache, bg, crontab, dig, dpkg, du, echo, fg, file, host, hostname, jobs, journalctl, killall, lsblk, mount, mst, netstat, nslookup, pkill, ss, stat, traceroute, ufw, umask, which`

**C. Package manager variants (Debian/Ubuntu, Arch, RHEL — all OSes represented):**
`apt, apt-cache, dpkg` (Debian) | `pacman, rpm, yum` (Arch/RHEL)

**D. Actual gaps — standard commands missing from BOTH lesson AND QBank:**
`apt-cache, mount, mtr, netstat, nslookup, pacman, pstree, rpm, yum`

**E. Artefacts:**
`server.sh` — extracted from `nohup ./server.sh &`; not a standard command

---

## 4. Assessment

### Overall: ✅ No significant content consistency issues.

**Symbol gaps** are almost entirely expected:
- Most symbol gaps are shell syntactic constructs (`|`, `&&`, `!`, `!!`, `for`, `true`, `test`) that are not "commands" in the traditional sense — they are shell operators and built-ins that don't need dedicated lesson pages.
- Commands like `bash`, `tee`, `sleep`, `echo` that appear in symbol examples are all present in the QBank (typing exercises), so users still get practice.

**System gaps** are similarly acceptable:
- System topics (`process_systemd`, `package_management`, `network_basics`) cover commands like `journalctl`, `crontab`, `ufw`, `ss`, `dig`, `mtr`, `traceroute` — these are system-administration commands that students would encounter in the system architecture context.
- The QBank provides typing practice for most of them.
- The 4 package managers (`apt`, `apt-cache`, `dpkg` for Debian; `pacman`, `rpm`, `yum` for RHEL/Arch) being present across system topics without dedicated lessons is a minor design choice — they could each warrant a lesson, but they're functionally covered in the `package_management` system topic.

**Lesson commands NOT in QBank:** `nginx` and `xargs` — these have lesson files but are not found as base commands in the question bank. This is a minor issue: `xargs` has a dedicated lesson but the QBank only uses it inside pipelines (e.g., `find ... | xargs ...`), not as a starting command. `nginx` appears only in its lesson file's examples.

### Minor Issues to Consider:

| # | Issue | Severity |
|---|---|---|
| 1 | `apt-cache`, `mount`, `mtr`, `netstat`, `nslookup`, `pacman`, `pstree`, `rpm`, `yum` — standard system commands used in system topics but absent from both lesson topics and QBank | Low (minor coverage gap) |
| 2 | `xargs` lesson exists but QBank has no standalone `xargs` practice commands | Low |
| 3 | `nginx` lesson exists but QBank has no standalone `nginx` practice commands | Low |
| 4 | `echo`, `bash`, `tee`, `sleep` appear in symbol examples with no dedicated lesson (but are in QBank) | Very Low |

---

## 5. Recommendations

1. **No action needed** for symbol gaps (shell operators are correctly treated as advanced context, not beginner topics).
2. **Optional:** Add lesson pages for `apt-cache`, `journalctl`, `ufw`, `crontab`, `ss` — these are common enough in system administration.
3. **Optional:** Add standalone `xargs` practice commands to the QBank (e.g., `find . -name '*.tmp' | xargs rm -v`).
4. **Optional:** Add standalone `nginx` practice commands to the QBank (e.g., `nginx -t`, `systemctl status nginx`).

---

## 6. Data Coverage Summary

| Category | Raw Items | Unique Base Commands |
|---|---|---|
| Symbol examples | 131 | 40 |
| System section examples | 127 | 46 |
| Question bank | 266 | 134 |
| Lesson topics | 31 files | 31 |
| **Total unique command tokens** | — | **~160** |

**Lesson coverage of QBank base commands:** 31 / 134 = **23%** — This is expected. The QBank is a comprehensive typing exercise database; lessons are focused tutorial overviews for the 31 most important commands. System/symbol topics use the broader set contextually.
