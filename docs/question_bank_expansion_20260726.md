# cmdtyper v0.3 题库扩充与质量审计报告

- 日期：2026-07-26
- 最终范围：`data/commands/20_*.toml` 至 `data/commands/35_*.toml`
- 关联实现总结：[`topic_training_expansion_20260726.md`](topic_training_expansion_20260726.md)
- 关联设计评审：[`review/design_review_20260726.md`](../review/design_review_20260726.md)
- 外部学习方案调研：[`linux_learning_landscape_20260726.md`](linux_learning_landscape_20260726.md)

## 1. 目标与安全边界

本轮工作不再把“扩充题量”视为唯一目标，而是同时处理四件事：

1. 补齐中文 Linux 新手最常遇到的任务与报错场景；
2. 把新增命令组织为可发现、可排序的专题；
3. 用 lesson、符号专题和系统专题建立解释层；
4. 为跟打、单词元填空和独立默写提供同一套规范命令数据。

项目安全边界保持不变：

> cmdtyper 不执行任何真实 shell 命令。用户输入只用于打字反馈或字符串匹配，所有“终端输出”都来自 TOML 预设内容。

题库可以讲解 `rm`、`sudo`、`tmux kill-session`、`reptyr` 等有副作用的命令，但应用不会执行它们；内容必须同时说明前置核对、影响范围、权限边界和停止条件。

## 2. 最终内容清单

### 2.1 全库规模

最终工作树中的内容规模为：

| 指标 | 最终值 |
|---|---:|
| 命令文件 | **35** |
| 规范命令 | **554** |
| 数据驱动命令专题 | **16** |
| 专题映射命令 | **283** |
| Lesson 文件 | **75** |
| v0.3 新增 Lesson | **44** |
| 符号专题 / 练习 | **8 / 120** |
| 系统专题 / 章节 | **11 / 52** |

原有 01–19 号文件在基线中包含 273 条命令。最终专题文件 20–35 共承载 283 条命令，同时把原先位于 `08_practical_devops.toml` 的两条 tmux 命令迁入第 35 专题，避免同一规范命令在旧文件和新专题重复保留。因此全库从 273 条变为 554 条，**净增加 281 条**，而新专题实际组织了 **283 条**。

### 2.2 16 个命令专题

| 顺序 | 文件 | 专题 | 题数 |
|---:|---|---|---:|
| 1 | `20_beginner_help_rescue.toml` | 求助自救与终端急救 | 19 |
| 2 | `21_basic_apt_workflow.toml` | APT 软件管理工作流 | 18 |
| 3 | `22_basic_tar_zip.toml` | 打包压缩与解压 | 19 |
| 4 | `23_beginner_fileops_safety.toml` | 文件操作与安全边界 | 18 |
| 5 | `24_basic_redirect_pipe.toml` | 重定向与管道基础 | 18 |
| 6 | `25_basic_env_shell.toml` | 环境变量与 Shell 配置 | 17 |
| 7 | `26_basic_find_grep.toml` | find 与 grep 搜索 | 18 |
| 8 | `27_basic_disk_space.toml` | 磁盘空间排查 | 18 |
| 9 | `28_basic_process_port.toml` | 进程与端口排查 | 16 |
| 10 | `29_basic_ssh_remote.toml` | SSH 远程与文件传输 | 17 |
| 11 | `30_basic_systemd_cron.toml` | systemd 服务与定时任务 | 20 |
| 12 | `31_beginner_vim_survival.toml` | Vim 生存与编辑器 | 18 |
| 13 | `32_basic_users_permission.toml` | 用户、组与最小权限 | 17 |
| 14 | `33_basic_text_toolkit.toml` | 文本处理工具箱 | 18 |
| 15 | `34_beginner_zh_locale.toml` | 中文环境与 Locale | 18 |
| 16 | `35_practical_terminal_session_recovery.toml` | 终端会话与任务接管 | 14 |
|  | **合计** |  | **283** |

第 35 专题补齐了 tmux 会话观察、窗口/窗格定位、TTY 与会话字段、普通 reptyr 接管前核对，以及 Yama `ptrace_scope` 的只读诊断。`reptyr -T` 只作为风险边界说明，**没有可执行练习条目**。

### 2.3 难度与类别分布

| 难度 | 题数 |
|---|---:|
| Beginner | 121 |
| Basic | 287 |
| Advanced | 67 |
| Practical | 79 |
| **合计** | **554** |

| 类别 | 题数 | 类别 | 题数 |
|---|---:|---|---:|
| Archive | 32 | FileOps | 66 |
| Network | 43 | Permission | 29 |
| Pipeline | 60 | Process | 47 |
| Scripting | 37 | Search | 36 |
| System | 159 | TextProcess | 45 |

## 3. Lesson 扩充与链接覆盖

最终共有 **75** 个 lesson 文件，其中 31 个为既有课程，44 个为本轮新增课程。44 个新增 lesson 按专题分批：

| 专题文件编号 | Lesson 数 | 专题文件编号 | Lesson 数 |
|---|---:|---|---:|
| 20 | 3 | 28 | 2 |
| 21 | 3 | 29 | 3 |
| 22 | 3 | 30 | 3 |
| 23 | 3 | 31 | 3 |
| 24 | 3 | 32 | 3 |
| 25 | 3 | 33 | 3 |
| 26 | 3 | 34 | 2 |
| 27 | 3 | 35 | 1 |
| **合计** | **44** |  |  |

新增 lesson 不复制完整规范命令记录，而是在示例中使用稳定 `command_id`。加载时由内容目录中的规范命令补全命令串、摘要、token 解释和模拟输出。当前覆盖结果为：

```text
内联命令链接：19
command_id 链接：283
唯一覆盖命令：301/554（54%）
未链接命令：253
悬空 command_id：0
重复 command_id 引用：0
```

283 条专题命令都被 44 个 v0.3 lesson **恰好引用一次**。另有 19 条既有 lesson 内联命令链接；按规范命令字符串去重后，全库共有 301 条命令获得 lesson 链接。

## 4. 内容设计原则

### 4.1 场景先于命令名

默写提示描述用户要完成的任务、观察对象和边界条件，而不是写成“执行命令：X”或“键入：X”。当多个命令可能完成相似目标时，题面会限定工具、字段、路径或安全前置；只有明确等价的写法才加入 `answers`。

### 4.2 观察、验证、最小动作

涉及权限、进程、远程主机、服务和覆盖写入的内容，优先采用以下顺序：

1. 只读观察当前状态；
2. 交叉核对对象、所有者、范围和上下文；
3. 选择影响最小、可回退或可停止的动作；
4. 保留诊断信息，不用静默重定向伪装成功。

例如：信号发送前重新确认 PID 与命令行；SSH 首次连接核验主机指纹；`rsync --delete` 先 dry-run；APT 变更前检查计划；tmux 清理前区分 detach 与 kill；reptyr 失败时尊重内核与权限策略。

### 4.3 为新手控制认知负担

本轮避免把复杂布尔 `find`、持续 root shell、`chmod 777`、真实账号变更、破坏性批量处理或高级 Vim 启动技巧当作新手记忆目标。危险操作可以出现在讲解中，但必须使用虚拟路径、明确前置条件和静态输出。

## 5. 规范重复问题已解决

早期报告记录的 7 组旧题 canonical command 重复现已全部处理，不再是待办项。最终审计结果为：

```text
canonical command 重复组：0
重复 command ID：0
```

处理方式不是机械删除所有内容，而是把重复题改成不同且明确的学习目标：

| 原重复命令 | 最终处理 |
|---|---|
| `top` | 改为 `top -o %CPU`，学习按 CPU 字段排序 |
| `df -h` | 改为 `df -h /home`，限定到包含指定路径的文件系统 |
| `du -sh /var/log` | 改为 `du -sh /var/log/journal`，聚焦持久日志目录 |
| `free -h` | 改为 `free -h -w`，区分 buffers 与 cache |
| `wc -l access.log` | 改为统计 `notes.txt`，形成独立练习场景 |
| `cut -d: -f1 /etc/passwd` | 改为 `-f1,7`，同时观察用户名与登录 shell |
| `curl -I https://example.com` | 改为健康检查端点 `/health` |

此外，旧文件中的 `tmux new -s dev` 与 `tmux attach -t dev` 被迁移到第 35 专题，不在旧文件中保留第二份。这既维持了稳定命令 ID，也避免专题接入时产生新重复。

## 6. 最终审计结果

### 6.1 Token 与 Lesson 审计

`python3 scripts/audit_tokens.py` 当前返回成功，关键结果如下：

```text
命令总数：554
题库 token：2003
唯一 token 文本：855
lesson 示例：469
lesson token_details：607
lesson 唯一覆盖：301/554（54%）
悬空 command_id：0
重复 command_id 引用：0
空泛模板描述：0
```

审计门槛已经从旧报告中的失败状态转为通过：Lesson 链接覆盖超过 50%，没有悬空引用，也没有命中空泛模板描述。

仍可继续改进、但不阻塞本轮验收的指标包括：

- 显式 `kind` 标注仍为 `0/2003`；运行时会用稳定启发式推断 token 类型；
- 少于 8 个字符的短描述为 191 条，需要后续按学习价值逐步扩写。

### 6.2 结构与引用门禁

当前门禁覆盖：

- 35 个 command TOML、75 个 lesson TOML、8 个 symbol TOML、11 个 system TOML 可由真实 Rust 类型解析；
- 每条命令的 token 拼接严格等于规范命令串；
- command ID、专题 ID/顺序和各内容命名空间保持唯一；
- 16 个专题恰好映射 283 个互不重复的规范命令 ID；
- lesson、symbol、system 的 `command_id` 必须回链主命令库，并在加载时验证或补全；
- 规范命令、提示、答案、模拟输出和 token 解释不会被引用内容悄悄覆盖为冲突值；
- 运行时代码存在静态门禁，禁止 shell/process 执行路径。

### 6.3 最终验收基线

2026-07-26 的最终工作树验收结果：

```text
cargo test --no-fail-fast：178 passed / 0 failed（11 suites）
cargo clippy --all-targets --locked -- -D warnings：0 warnings
规范命令与 accepted answers 的 bash -n：1288 checks / 0 failures
符号、系统与 kill lesson 补充 bash -n：574 checks / 0 failures
git diff --check：通过
最终对抗性内容复核：no P0/P1 content findings remain
```

这里的 `bash -n` 只把教材字符串交给 Bash 做语法解析，不执行任何题库命令。

包装与运行验收也已完成：

- `./scripts/install.sh` 成功发布最终二进制与四类数据目录；安装后二进制和数据与工作树逐字节一致，`data/reviews/` 未安装；
- 临时 HOME 下验证 staging copy、data publish、binary publish 和 SIGHUP 四类失败，退出码分别为 90、91、92、129，均完整恢复旧二进制、旧数据和 sibling JSON；
- fresh Bash/Zsh 都解析到 `/home/ace/.local/bin/cmdtyper`，并使用 `/home/ace/.local/share/cmdtyper/data`；
- `docker build --network=host -t cmdtyper:v0.3 .` 成功，镜像内清单为 35/75/8/11，且不含 `data/reviews/`；
- 已安装 TUI 和 Docker TUI 均在真实 tmux 终端中显示 `v0.3.0` 并 clean exit；已安装版本实测进入学习中心、16 个专题、终端会话专题及 L1/L3/L5 级别选择。

## 7. 来源与许可证策略

中文 summary、prompt、token 解释、模拟场景和安全说明均以独立中文重新编写。外部资料用于核对事实、课程顺序和常见新手问题，不直接复制不兼容许可证的正文。

主要事实来源：

- [Debian Manpages](https://manpages.debian.org/)：Debian/Ubuntu 命令与选项行为；
- [GNU Bash Reference Manual](https://www.gnu.org/software/bash/manual/bash.html)：Shell、变量、作业、展开与重定向；
- [OpenSSH manuals](https://man.openbsd.org/ssh)：主机密钥、认证与远程操作边界；
- [systemd manuals](https://www.freedesktop.org/software/systemd/man/latest/)：服务、日志与 systemd 语义；
- [APT sources.list(5)](https://manpages.debian.org/bookworm/apt/sources.list.5.en.html)：APT 源格式与启用规则；
- tmux、procfs/Yama 和 reptyr 的公开文档用于核对会话、TTY 与 ptrace 语义。

可兼容借鉴的开放内容生态包括 MIT 的 `jaywcjlove/linux-command`、`cmdchallenge`、`srsudar/eg`，CC0 的 `cheat/cheatsheets`，以及需要署名的 CC-BY 4.0 `tldr-pages` 和 Linux Upskill Challenge。GFDL、GPL、CC-BY-SA、NC/ND 或无明确许可的来源只用于事实核对和结构研究，正文不并入 MIT 题库。完整调研见 [`linux_learning_landscape_20260726.md`](linux_learning_landscape_20260726.md)。

## 8. 结论

本轮最终交付已经离开“只扩题量、讲解覆盖不足”的中间状态，形成以下完整清单：

- 35 个命令文件、554 条规范命令；
- 16 个数据驱动专题、283 条专题命令；
- 75 个 lesson，其中 44 个新增 lesson 覆盖全部专题命令；
- 301/554（54%）的全库 lesson 链接覆盖；
- 8 个符号专题、120 题；
- 11 个系统专题、52 节；
- canonical command 重复为 0；
- token/lesson 审计通过；
- 178 项 Rust 测试、严格 Clippy、1862 项 shell 语法解析、事务安装与 Docker/TUI 验收全部通过。

题库规模、专题入口、解释层和 L1/L3/L5 训练已经形成一致链路。后续工作应继续提升剩余 253 条命令的 lesson 链接深度、补充显式 token kind，并在独立版本中评估 L2、L4 与间隔调度，而不是把这些尚未实现的能力写入当前产品承诺。
