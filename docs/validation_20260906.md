# cmdtyper v0.4 验收记录（2026-09-06）

实现位置：`intranet:/home/ace/workspaces/cmdtyper`；本地检出位置：`/home/yueling/workspaces/cmdtyper-r`。原有未提交工作已从完整备份保留为独立提交 `8a22401`，备份文件仍在 `/home/ace/workspaces/cmdtyper-before-expansion-20260906.tar.gz`。本次升级不覆盖日常安装程序或真实用户的练习历史。

## 交付范围

- 40 条旧长流程改为 216 个有序步骤，每条 Enter 后显示自己的模拟输出、提示符和解释；176 个静默步骤明确显示无终端输出。变量与目录上下文保留；条件执行教学按原语义保留。
- 2467 条规范命令、110 个命令文件、53 个训练专题；课程、符号、系统、基础练习、工作流和场景通过 ID 引用。
- 独立新增 489 条专题命令，覆盖 24 个网络诊断专题和 13 个其他运维专题；保守排除旧 554 条命令的任意字面子串后仍有 488 条，超过 400 条要求。
- 281 个基础用法组覆盖 174 个命令 L1 示例、55 个符号条目和 52 个系统章节，每组 1 个教学示例加 3 道简单练习；843 个练习位置对应 690 道独立题。全部组有核心能力契约与逐题对应理由，见[语义复核](foundation_semantic_review_20260906.md)。
- 学习中心删除四个重复难度入口，增加基础用法练习与场景实训；专题详情按 P 进入基础组，规范命令共享进度。
- 20 个实训案例、245 步、41 个判断点。真实 App 按键测试完整走通每个案例及全部错误/正确判断分支。
- 日历覆盖全部完整跟打入口，按来源区分命令、符号、系统基础练习；记录有效时长、WPM、CPM、位置准确率和未完成片段。逐字符窗口为最近 50 个位置、至少 10 个样本起算、两端各 10% 缩尾；跨日重打不会改写旧日窗口。
- 历史先落盘，缓存可重建；迁移前原子发布备份，保留旧口径基线。损坏历史、备份失败与保存失败均有明确处理，不能无声覆盖历史或丢弃当前输入。

完整对应见[实施计划](cmdtyper-v0.4-plan.md)和[需求验收矩阵](v0.4-acceptance-matrix.md)。`canonicalization_audit.json` 是初次规范迁移阶段的 2258 条命令与 507 个别名快照；后续教学复核新增 209 条针对性短题，最终为 2467。最终数量以 `content_expansion_audit.json` 及内容审计脚本为准。

## 自动化验证

| 命令 | 结果 |
| --- | --- |
| `cargo test --no-fail-fast --quiet` | **265 项通过，0 失败、0 忽略**；121 单元测试、144 集成测试，见[结果 JSON](validation/cargo-tests.json) |
| `cargo fmt --all -- --check`、`git diff --cached --check` | 通过 |
| `cargo build --release --quiet` | 通过，产物 `target/release/cmdtyper` |
| `python3 scripts/audit_practice_content.py` | 去重、引用、基础覆盖和新增清单通过；1098 条 Bash 语法、45 项精确回归、281 个语义契约及 843 个逐题对应检查通过 |
| `docker compose config --quiet` | 通过 |
| `docker build --network=host -t cmdtyper:v0.4 .` | 通过，包含所有新增数据目录与迁移元数据 |
| `python3 scripts/smoke_tty.py`、追加 `--workflow` | 原生 release 两条真实 PTY 路径通过，退出码均为 0 |
| `python3 scripts/smoke_tty.py --docker`、追加 `--workflow` | Docker 两条真实 PTY 路径通过，退出码均为 0 |

原生 release SHA-256：`c95fc247e9515ad91cee46c8b03e380c309456bc14692f7187e3ecff96f4a71f`。验收镜像 ID：`sha256:fb29f060686d9f076fb62553f163bca77512118999b5ab9a79a57e1a84ed1b54`。

Docker 代理配置指向主机 `127.0.0.1:7890`。直接 `docker compose build` 的首次尝试因桥接构建容器无法访问该地址而失败；改用 `--network=host` 完成构建，未修改主机代理配置。运行验收使用 `--network=none`，绑定临时用户目录。

场景 PTY 路径从菜单进入日历和场景，完成前三步，错答后返回证据，再作出正确判断；第四步输入部分字符后退出并续练。原生与 Docker 都生成 3 条完成记录和 1 条未完成记录，同一位置连续误按只记 1 错，记录 ID 不重复。工作流 PTY 路径通过正常课程恢复入口，完成四步解压流程，验证每步输出及输出后首字符；最终只产生 1 条完成记录。

所有测试练习数据均隔离在临时目录。Bash 语法审计只运行 `bash -n`；应用和验收不会执行教学内容中的运维命令。

## 界面与录像

20 张图片来自 ratatui TestBackend 的真实渲染单元格，使用演示练习数据；覆盖 40×10、80×24、120×36。以下录像分别记录实际 release 和 Docker 进程：

- [完整日历](validation/calendar-120x36.png) / [40×10 日历](validation/calendar-40x10.png)
- [逐条输入](validation/workflow-80x24.png) / [步骤输出](validation/workflow-output-80x24.png) / [40×10 工作流](validation/workflow-40x10.png)
- [基础组列表](validation/foundations-120x36.png) / [教学示例](validation/foundation-teaching-80x24.png) / [三题中的第 1 题](validation/three-exercises-80x24.png)
- [场景跟打](validation/scenario-typing-80x24.png) / [证据解释](validation/scenario-evidence-80x24.png)
- Release：[场景录像](validation/release-smoke.cast)、[结果](validation/release-smoke.json)；[工作流录像](validation/release-workflow.cast)、[结果](validation/release-workflow.json)
- Docker：[场景录像](validation/docker-smoke.cast)、[结果](validation/docker-smoke.json)；[工作流录像](validation/docker-workflow.cast)、[结果](validation/docker-workflow.json)

安装 asciinema 的环境可使用 `asciinema play docs/validation/release-workflow.cast` 回放。重新生成图片：

```bash
TZ=Asia/Shanghai cargo run --example render_validation --quiet
python3 scripts/render_validation.py
```

PNG 转换脚本需要 Pillow 和系统 Noto Sans CJK 字体。低于 20 行时，日历优先显示当天指标；长工作流自动保持输入光标可见，PgUp/PgDn 可阅读说明。

## 版本管理

升级分支为 `feat/v0.4-training-upgrade`，发布标签为 `v0.4.0`。原始 `v0.2` 与 GitHub `main` 保持原有历史；升级基于实际实施目录及其已有工作，以普通提交和新增分支发布，不强制改写远程分支。代码审查、测试、内容审计和终端验收通过后再推送。

复核提交及版本位置：

```bash
git status --short --branch
git rev-parse HEAD v0.4.0^{commit} origin/feat/v0.4-training-upgrade
git log --oneline -3
```

## 启动

在本地或 intranet 的仓库根目录：

```bash
TZ=Asia/Shanghai CMDTYPER_DATA_DIR=./data ./target/release/cmdtyper
```

显式指定题库目录可避免读到旧安装数据。TZ 仅为当前进程选择上海日期，不修改服务器时区；已记录的输入日期不被重算。常规构建、安装和 Docker 运行方式见 README。
