# Linux 入门学习方案对标调研

- **日期**：2026-07-26
- **目的**：为 cmdtyper v0.3 设计改进与题库/讲解扩充提供外部参照。核心问题：面向基础薄弱的中文 Linux 新手，如何弥合"跟打练习 → 独立默写命令"的断层，以及题库/讲解可以从哪些现有资源借鉴（含许可证可行性）。
- **方法**：多 agent 并行网络调研，五个方向（权威文档体系 / 体系化课程书籍 / 交互式练习平台 / 速查解释工具 / 社区新手痛点）+ 一轮综合映射。结论均附来源 URL；"可否借用内容"均以许可证核查为准（cmdtyper 为 MIT）。
- **配套文档**：本仓库内部设计评审见 `review/design_review_20260726.md`（另行产出）。

---

## 一、综合结论：映射到 cmdtyper 的决策清单

### 1. 可直接借内容的源（按许可证可行性排序）

#### 第一梯队：可直接翻译/改写入 MIT 仓库

| 源 | 许可 | 覆盖 | 引入方式与改写成本 | 优先级 |
|---|---|---|---|---|
| **jaywcjlove/linux-command** | MIT（GitHub API 确认 SPDX=MIT） | 580+ 命令，**原生中文**，结构=概述/补充说明/选项逐条注释/实例 | 结构几乎一对一映射 lesson TOML 与 token_details，改写成本最低（无需翻译）。https://github.com/jaywcjlove/linux-command | **P0** |
| **cmdchallenge** | MIT（LICENSE 已 clone 核实） | 60 道"目标→写命令"题，含 oops 排障剧场、12days 先讲后练、expected_failures/多解字段 | 翻译题面+改造 schema 入 reviews/*.toml；judge 思想移植 matcher。本地已有克隆 /tmp/cmdchallenge-mirror/challenges.yaml。https://github.com/jarv/cmdchallenge | **P0** |
| **tldr pages** | **CC-BY 4.0**（不是 MIT——LICENSE.md 核实；兼容 MIT，须署名） | 英文 7391 页、**简中 1532 页**；覆盖 cmdtyper 现有 134 命令中的 104（78%） | 写解析脚本转 TOML 草稿；人工工作=实例化 {{占位符}}（tokens_consistency 要求）+ 手写预置输出。合规：README+关于页署名 tldr-pages team、注明修改。https://github.com/tldr-pages/tldr/blob/main/LICENSE.md | **P0** |
| **Linux Upskill Challenge** | CC BY 4.0 | 21 天服务器向课程（grep/cut/awk、find、tar、inode/链接、history/Tab） | 唯一可整体翻译改写的**成体系教材**；署名即可。挖 Day5/8/11/16/19。https://linuxupskillchallenge.org/ | **P1** |
| **srsudar/eg** | MIT | 75 命令深度示例，"具体在前、抽象在后" | 翻译引入，保留版权声明。https://github.com/srsudar/eg | P1 |
| **chubin/cheat.sheets** | MIT（LICENSE 核实） | 命令速查片段 | 补充性素材。https://github.com/chubin/cheat.sheets/blob/master/LICENSE | P2 |
| **TLDP Garrels 两本书** | BSD-3 风格（版权页核实，非 GFDL） | 《Intro to Linux》《Bash Guide for Beginners》：Quickstart 表、chmod 权限码表、各章 Exercises | 可汉化改写，须在 NOTICE 保留版权声明；**内容 2008 年止，需逐条甄别过时项**，改写成本高。https://tldp.org/LDP/intro-linux/html/intro_07.html | P2 |

#### 存疑（调研互相矛盾，用前须复核）

- **cheat/cheatsheets**（约 275 命令）：速查工具组 curl 核实 `.github/LICENSE.txt` 为 **CC0 1.0**（https://raw.githubusercontent.com/cheat/cheatsheets/master/.github/LICENSE.txt ）；社区痛点组用 GitHub API 查得 license=null 判为不可用。**矛盾原因**：LICENSE 藏在 .github/ 下，API 检测不到。倾向采信 curl 直查结果（CC0，可用且零署名义务），但引入前应再抓一次原文存档留证。

#### 只能借结构/思路，不得抄一句文字

- **GFDL**：ArchWiki、coreutils 手册（分类框架可用）
- **CC-BY-SA**：Ubuntu 教程/文档、Linux Journey（其 lessons/zh 只作术语对照）、Stack Exchange 全部内容、the-art-of-command-line
- **CC BY-NC-ND**：TLCL（playground 结构可借）；**CC BY-NC-SA**：Missing Semester
- **GPL 系**：Debian Reference（GPL-2+，命令清单属事实可用）、explainshell（GPL-3，且数据沿用 man 上游许可）、bashcrawl（GPLv3）；**AGPL**：keybr 代码
- **专有**：鸟哥（非营利可引用需注明，改写入库不行）、runoob、SadServers、ShortcutFoo、Linux Survival、AnkiWeb 牌组
- 不受版权保护、可自由用：man 页章节结构惯例、单条命令字符串（事实）、SRS/置信度/提示分层等**机制与算法**

---

### 2. 教学编排共识（8 套教材共识 × 社区痛点校正）

**8 套教材无一例外的骨架**（鸟哥/TLCL/Linux Journey/LUC/Missing Semester/runoob/Linux Survival/《大全》）：
shell 概念 → 导航(pwd/cd/ls) → 文件操作(touch/cp/mv/mkdir/rm) → 求助(man/--help/history/Tab，普遍前置为生存技能) → 通配符 → 重定向/管道（公认第一个 aha 时刻）→ 权限 → 进程 → 软件包；vim 居中段，脚本压轴。
（分歧点：鸟哥把权限放在文件管理**之前**（第5章），TLCL 放在第10章——对新手建议从 TLCL 序，权限在管道之后，因为权限报错需要先有文件操作经验作场景。）

**痛点校正**（Stack Exchange API 实测高票分布 vs 教材章节）——**错位本质：教材按知识体系自底向上，新手按任务和报错原文提问**（https://askubuntu.com/questions?tab=Votes ）：

- **提权**：apt 软件包管理（AU 全站第 1 名 2748 票，鸟哥却排第 21-22 章且讲 YUM）→ 前移到导航之后；tar/unzip（2317 票级）→ 独立早期专题；PATH/.bashrc/export（1412+2273 票）→ 教材普遍散落，应独立成课；vim 生存四键（"How do I exit Vim?" 5550 票 331 万浏览）→ 单独做题
- **教材完全缺失、必须新增**：报错排查/误操作急救（unable to lock dpkg、Argument list too long、Ctrl-S 假死、rm 事故预防）；概念辨析卡（terminal/shell/tty、sh vs bash、2>&1、`--` 双横线）
- **降权/不进题库**：磁盘分割、Quota/RAID/LVM、开机流程、内核编译（鸟哥重点章节，高票区几乎为零）

**推荐大纲（cmdtyper v0.3 主线）**：

1. shell 是什么 + 导航（pwd/cd/ls）
2. 文件操作（touch/cp/mv/mkdir -p/rm/ln -s）+ 危险命令警告框
3. 求助与自救（man/--help/history/Tab/type/which）——所有教材共识的早期生存课
4. 软件安装 apt 全工作流（↑痛点提权）+ 中文换源
5. 打包解压 tar/gzip/unzip（↑痛点提权）
6. 通配符 → 重定向 > >> 2>&1 | tee
7. 权限 chmod/chown/sudo（数字+符号双写法，"为什么别 777"）
8. 环境：PATH/.bashrc/export/source（↑提权）
9. find/grep 高频姿势
10. 磁盘排查 df/du；进程/端口 ps/kill/lsof
11. vim 生存 → ssh/scp/nohup → systemctl/crontab
12. 贯穿专题：报错急救卡、概念辨析卡、中文用户生存专题（换源/乱码/输入法/WSL——英文产品完全不覆盖的差异化卖点）

**每课显式声明前置课程并可跳转**（ArchWiki 反面教训：假设前置知识+RTFM 即劝退；Diátaxis 教训：教学层与速查参考层必须在 UI 上分开。https://wiki.archlinux.org/title/Frequently_asked_questions 、https://documentation.ubuntu.com/server/ ）

---

### 3. 练习梯度设计模式

**核心规律（跨平台归纳）**：示范→独立的过渡都靠**控制"给出信息的层级"而非改题目**。层级从高到低：完整答案照打 → 教学框+题目 → 提示可选 → 只给目标+工具名 → 只给目标 → 限时纯回忆。衰减触发=显式分段（ShortcutFoo）或掌握度驱动（keybr）。

现成模式清单：

| 模式 | 出处 | 说明 |
|---|---|---|
| ★① token 填空（cloze） | Linux Journey quiz / navi 变量槽（https://github.com/denisidoro/navi ） | 命令骨架固定、挖 1→2-3 个 token 让用户补全；与 cmdtyper token 数据模型天然对齐，**实现成本最低** |
| ★② ghost text 渐隐提示 | fish autosuggestion（https://fishshell.com/docs/current/design.html ） | 目标命令灰字显示供跟打，随正确次数先隐变量→再隐选项→全隐=默写；TypingEngine 已具备逐字符着色 |
| ★③ 分层提示按键 | Bandit 三层信息 + ShortcutFoo 四段式（https://overthewire.org/wargames/bandit/ ） | 默写卡 h 键逐层揭示：命令名清单→关键 flag→完整答案；按提示次数折算掌握度 |
| ★④ 失败兜底+错题队列 | Katacoda solutionText / Execute Program 无惩罚重试（https://interactive-docs.oreilly.com/configuration/index-json.html ） | 默写失败 3 次显示完整答案并入错题队列；复习答错不扣分，进度奖励偏向推进新课 |
| ★⑤ 间隔重复调度 | srsh（https://github.com/ryanbloom/srsh ）/ FSRS repeater | 卡片="中文任务描述→命令"，与默写题同构；review_loader 已存在，加 SM-2 简化版即可 |
| ⑥ 课末 check 关卡 | Katacoda verify + ShortcutFoo Test 70% 门槛 | 本课命令混合默写，达标才标记完成 |
| ⑦ 情景串 playground | TLCL 第 5 章（结构可借，文字 BY-NC-ND 不可抄） | 6-8 条命令连贯叙事，前一条预置输出是后一条操作对象 |
| ⑧ 例题反白揭晓 | 鸟哥（形态可借） | 讲解页每 1-2 知识点插例题，按空格揭晓答案 |
| ⑨ 多解展示 | cmdchallenge /c/s 端点（MIT 可抄） | 判对后展示等价写法（tail -n5 / tail -5）各附一句差异 |
| ⑩ 报错卡 | 社区高票"报错原文即标题"现象 | 预置报错输出→默写修复命令；教材没有、新手最需要 |

★=最适合移植 TUI 的 5 个。①+②+③串成"跟打→半掩填空→分层提示默写→限时复测"四阶管线，直接填补跟打→默写断层。

**避雷清单（keybr 社区实证，https://github.com/aradzie/keybr.com ）**：稀有 token 出现频率过低会导致掌握度卡死→调度器保底出现频率；瞬时速度可被"停顿+爆发"刷分→滚动窗口多次采样；勿一次性解锁一大批新符号；**必须留手动越级逃生门**（keybr 最大怨点）。1986 年 HCI 实验佐证"先引导后独立+时间上分散练习"效果最好（https://www.cs.cmu.edu/~07131/f18/topics/readings/week-2 ）。

---

### 4. 一条命令的新手讲解模板

结构惯例（man 式四栏）无版权问题；篇幅比例来自鸟哥 cp 章实测（范例 55%）与 runoob 版式，为被验证比例：

```
① 场景引入（5-10%）——为什么需要它，一句话任务场景（学 TLCL 的动机式开场）
② 语法框 + token 拆解（15-20%）——骨架一行；固定 token 与可替换占位符用不同颜色
   （映射 man 的粗体=照打/斜体=占位符惯例）；选项逐条一行中文，优先长选项括注短选项
③ 带预置输出的示例（≥50%）——2-4 个，具体示例在前、抽象通式在后（eg 序）；
   按真实使用频率排序（tldr 规则）；行内 # 注解（鸟哥式）
④ 易错点/纠偏（10-15%）——1-2 条"常见错误直觉"（如 2>1 为什么错、rm * .html 多个空格）；
   危险命令加警告框（Ubuntu tutorial 式强警告）
⑤ 小结 + 相关命令（5-10%）——SEE ALSO 链到题库其他命令形成学习图；课末 SUMMARY 清单（vimtutor 式）
```

写作规则（tldr style guide + 三条 1500-3800 票高票答案共性提炼，https://github.com/tldr-pages/tldr/blob/main/contributing-guides/style-guide.md 、https://stackoverflow.com/q/818255 ）：
- **命令先行**：第一行就是完整可打的命令，再解释
- summary 一句祈使句（"列出…""创建…"），dictation.prompt 对齐此句式
- 每个 flag 单独一行拆解，标注顺序约束（"f 必须在最后"式记忆钩子）
- 只给 1-2 个高频变体，不穷举；注明边界条件；讲解必须自足，**禁止 RTFM 式"请自行查手册"**
- 语气用对话式白话（简体版鸟哥删口语被读者视为损失）；每个选项绑定一个使用场景（runoob 纯参数表的反面教训）
- 术语基准：大陆译法+括注英文（目录/进程/重定向/管道/索引节点），命令与选项保留英文原样；以 debian-reference 简中版术语为对照表

---

### 5. 题库主题优先级 Top 15（均有高票数据支撑）

| # | 主题 | 最佳参考源 |
|---|---|---|
| 1 | tar/unzip/gzip 参数组合 | tldr zh tar.md（CC-BY）+ linux-command tar.md（MIT）+ 2039 票答案的逐 flag 讲法结构（https://askubuntu.com/q/25347 ，结构可借） |
| 2 | apt 全工作流 + dpkg -i + 报错(lock/unmet deps) | AU 高票题面做题干蓝本（分布事实）；讲解底料 linux-command；换源部分自写（https://mirrors.tuna.tsinghua.edu.cn/help/ubuntu/ 参考事实） |
| 3 | chmod/chown/sudo（数字+符号、-R、为什么别 777） | TLDP《Intro to Linux》chmod 权限码表（BSD-3 可改写）+ linux-command |
| 4 | cp/mv/rm/mkdir -p/ln -s 及其坑（symlink 2254 票） | cmdchallenge 主线题（MIT 直接翻译）+ TLCL playground 结构 |
| 5 | 重定向 > >> 2>&1 \| tee | 3338 票 2>&1 答案的"先驳直觉写法"结构（https://stackoverflow.com/q/818255 ，只借结构）；概念卡自写 |
| 6 | PATH/.bashrc/export/source | 1584 票答案三段结构（https://unix.stackexchange.com/q/26047 ）+ LUC Day5（CC BY 可改写） |
| 7 | find/grep 高频姿势（含排除目录；SO linux 第 1 名 7705 票） | cmdchallenge find/grep 题 + Bandit 式属性谜题写法（思路）+ LUC Day11 |
| 8 | 磁盘排查 df -h/du -sh 链 | tldr zh + cheat/cheatsheets（CC0，复核后用） |
| 9 | ps aux\|grep / kill / lsof -i:端口 | cmdchallenge oops 剧场（MIT，含 /proc 找进程）+ tldr |
| 10 | ssh/scp/nohup/& | LUC Day1/Day12（CC BY）+ tldr 英文页（ssh-keygen 等简中缺失需自译） |
| 11 | systemctl/journalctl/crontab | LUC Day10/Day18 + tldr 英文页（journalctl 简中缺失） |
| 12 | vim 生存四键 | vimtutor 结构（文本不可抄）+ 5550 票"exit Vim"作动机叙事 |
| 13 | useradd/usermod -aG/groups/passwd | LUC Day13（CC BY 可改写；tldr 简中缺 useradd/usermod 需自译） |
| 14 | 文本处理 sed s///g、cut/sort/uniq/wc、head/tail -f | cmdchallenge 中后段管道题（MIT）+ LUC Day8 + coreutils 分类框架（仅框架） |
| 15 | 中文特供：换源三步、locale/LANG 乱码、unzip -O GBK | 无现成开放语料，全部自写（CSDN 只作痛点证据：https://blog.csdn.net/qq_45988641/article/details/123631126 ）——差异化卖点 |

**辅助层**（少打字内容，放符号/系统专题）：概念辨析卡（terminal/shell/tty、sh vs bash、man 页数字、`--`）+ 急救卡（Ctrl-S→Ctrl-Q、Ctrl-Z/fg、rm 事故预防）。覆盖度对照表用 Debian Reference 表 1.17 约 50 命令清单（命令清单属事实，https://www.debian.org/doc/manuals/debian-reference/ch01.zh-cn.html ）；扩到 500+ 题时保持金字塔：核心 12-20 命令做全四阶段+lesson+复习卡，长尾只做跟打+讲解（Linux Survival "一打命令覆盖大多数任务"原则）。

**明确标注的不确定/矛盾点汇总**：
1. cheat/cheatsheets 许可两路结论相反（CC0 vs null），采信前须复核 `.github/LICENSE.txt`。
2. 权限章节位置：鸟哥（第5章，前置）vs TLCL/多数英文教材（管道之后）——本清单采后者，理由见 §2。
3. tldr 简中 30 个缺失命令（pwd/ssh-keygen/journalctl/useradd 等）需从英文页自译，质量不能直接搬。
4. Reddit 新手痛点（选发行版/驱动/系统坏了）与 Q&A 站痛点（命令怎么写）性质不同；前者不适合打字题，最多一张导读卡。
5. LUC 内容为服务器管理向（Apache/ufw 等对纯 CLI 新手偏深），改写时需裁剪；TLDP 两书 2008 年止，逐条甄别。

---

## 二、分方向调研详情

### 2.1 权威文档体系（man / ArchWiki / Debian / Ubuntu / TLDP / coreutils）

**概要**：调研六大权威文档体系：man 页的 NAME/SYNOPSIS/DESCRIPTION/EXAMPLES 是命令讲解的最小模板（行业惯例，不受版权保护）；Ubuntu 新手教程的"沙箱+危险警告+自检"与 TLDP 的"正文+Summary+Exercises"是最适合新手的教学结构；Debian 参考手册第 1 章有现成中文新手命令大纲（约 50 命令表）；coreutils 手册提供按任务分类法；ArchWiki 因假设前置知识+RTFM 文化劝退新手，是反面教材。许可证上仅 TLDP Garrels 两本书（BSD-3 风格）与 MIT 兼容可直接改写引入；GFDL/CC-BY-SA/GPL 内容只能借结构大纲。各中文翻译普遍滞后或有机翻痕迹，cmdtyper 自写中文讲解本身即稀缺价值。

**发现**：

- 【man 页标准结构】man-pages(7) 规定 24 个标准 section 及固定顺序，命令类核心为 NAME→SYNOPSIS→DESCRIPTION→OPTIONS→EXIT STATUS→EXAMPLES→SEE ALSO；EXAMPLES 要求短小完整、shell 会话中用户输入加粗；字体约定：粗体=原样照打的部分，斜体=需替换的参数/占位符。此结构是行业惯例不受版权保护，可自由采用；但 man 页正文多为 Linux-man-pages-copyleft（verbatim 类 copyleft），coreutils 自带 man 页为 GPLv3+，均不能直接抄文字。来源：https://man7.org/linux/man-pages/man7/man-pages.7.html 、https://spdx.org/licenses/Linux-man-pages-copyleft.html
- 【ArchWiki 文章模板】Help:Style 规定标准章节：前言/介绍→安装→（用法/配置）→已知问题→提示与技巧→问题解决→参见；风格要求正式精确、禁第一人称、中文翻译避免'你/您'、普通用户命令用 $ 提示符 root 用 #、推荐写 '# command' 而非 '$ sudo command'、客观列各选项优缺点不做主观推荐。明确假设读者已会 pacman 等基础操作。许可证：GFDL 1.3+（英文与中文站相同），与 MIT 不兼容，只能借结构。来源：https://wiki.archlinuxcn.org/wiki/Help:Style （英文原版 https://wiki.archlinux.org/title/Help:Style 被反爬拦截，经中文镜像确认）
- 【ArchWiki 劝退新手的机制（反面教材）】官方 FAQ 明确目标用户是'愿意 DIY 的 competent users'，新手须自行投入学习并接受 RTFM 文化；2016 年 7 月官方删除 Beginners' Guide 并入精简的 Installation Guide，论坛管理员立场是'Arch 没有消费者，扩大用户群是完全错误的目标'；General recommendations 开篇即声明假设读者已完成安装并读懂系统管理/包管理章节。失败原因可归纳为：假设前置知识、参考式而非教学式结构、无渐进路径、出错自负。来源：https://wiki.archlinux.org/title/Frequently_asked_questions 、https://bbs.archlinux.org/viewtopic.php?id=216665 、https://wiki.archlinux.org/title/General_recommendations
- 【Debian 参考手册第 1 章＝现成中文新手大纲】结构：1.1 控制台基础（提示符/root/关机/sudo）→1.2 类 Unix 文件系统（权限/umask/链接/设备文件）→1.3 MC→1.4 工作环境（bash 定制/分页器/vim）→1.5 简单 shell 命令（环境变量/通配符/返回值/重定向/别名）→1.6 文本处理（正则/sed/awk 片段）。'表 1.17 基本 Unix 命令列表'两列式列出约 50 个命令；教授命令含 pwd/whoami/id/type/which/man/apropos/ls/mkdir/cd/touch/cp/rm/mv/ln/chmod/chown/umask/find/locate/grep/sed/cut/sort/uniq/tr/xargs/diff/tar/gzip/xz/ps/top/kill/su/sudo 等。讲解格式=命令-说明两列表格+'尝试下列例子'可跟打示例块+注意/提示/警告标注框。官方简体中文版由 26 人团队经 Weblate 翻译（2016 起，688 次提交），整体通顺、术语准确（索引节点/硬链接/正则表达式），偶有英文残留和翻译腔。许可证 GPL-2+（copyleft，MIT 项目不能直接抄正文；大纲与命令清单属事实可借）。来源：https://www.debian.org/doc/manuals/debian-reference/ch01.zh-cn.html 、https://www.debian.org/doc/manuals/debian-reference/apa.zh-cn.html 、https://www.debian.org/doc/manuals/debian-reference/debian-reference.en.txt
- 【The Debian Administrator's Handbook】双许可 GPL-2+ 与 CC-BY-SA 3.0（2012 年'liberation'众筹解放版权）；16 章面向系统管理员（APT/网络服务/安全/打包），非命令行新手向，对 cmdtyper 主要参考点是附录 B'简短辅导课程'和第 7 章'问题解决与信息检索'（教人查文档的方法论）。简体中文版在线可读但完成度不高：多个章节标题仍是英文（如 Dynamic Routing、DNS 等节），无翻译进度标注，术语偶有生硬（'品质服务'=QoS）。两个许可证均为 copyleft，不能直接抄入 MIT 项目。来源：https://debian-handbook.info/liberation/ 、https://debian-handbook.info/browse/zh-CN/stable/ 、https://lwn.net/Articles/496797/
- 【Ubuntu 'The Linux command line for beginners' 教程＝最佳新手脚手架范例】章节：历史简介→打开终端→创建文件夹与文件→移动操作文件→管道→超级用户→隐藏文件；命令顺序：pwd→cd（含 cd /、cd ..、cd ~、cd -）→whoami→mkdir(-p/-v)→ls(-a)→echo+重定向 >/>>→cat（含通配符）→less→mv→cp→rm(-r/-i)→rmdir→wc -l→管道|→uniq→sort→man→su/sudo→apt install tree→logout。教学手法：开场声明'假设零基础'（We'll assume no prior knowledge）；全程在 /tmp/tutorial 沙箱操作；每步用 pwd 自检位置；专设术语辨析（root 的目录/用户双义）；危险命令强警告（rm 'deletes them totally, utterly and irrevocably'、'Don't use su'、'Be careful with sudo'）；命名规范建议（小写+下划线避转义）。许可证：教程页面本身未标注，但 Ubuntu 官方文档法律页声明除非另注均为 CC-BY-SA（现行 4.0），server 文档仓库亦为 CC-BY-SA 4.0——只能借编排不能抄文字。来源：https://ubuntu.com/tutorials/command-line-for-beginners 、https://help.ubuntu.com/legal.html 、https://github.com/canonical/ubuntu-server-documentation
- 【Ubuntu Server 文档＝Diátaxis 分层样板】明确采用 Diátaxis 四分法：Tutorial（零基础、step-by-step）/ How-to（'assume basic familiarity'）/ Reference（术语表、命令行速查表）/ Explanation（概念背景），并给新手三条 Getting started 路线（安装→系统基础→命令行）。成功要素＝把'教学'和'查阅'显式分层、每层声明前置假设——正是 ArchWiki 缺的东西。许可 CC-BY-SA 4.0。来源：https://ubuntu.com/server/docs/ 、https://documentation.ubuntu.com/server/
- 【TLDP 两本 Garrels 书＝唯一可直接改写引入的文本源】《Introduction to Linux – A Hands on Guide》（2008，v1.27）与《Bash Guide for Beginners》（2008，v1.11）版权页均为 BSD 三条款风格许可（'Redistribution and use in source and binary forms, with or without modification, are permitted...'），非 GFDL——与 MIT 兼容，可翻译改写引入，只需保留版权声明与条件文本。两书每章固定'正文+Summary+Exercises'教学结构；《Intro to Linux》含 Quickstart commands 表、Bash 快捷键表、chmod 权限码表、DOS vs Linux 命令对照表（附录 B）。缺点：2008 年后未更新，打印/声音/网络章节过时，需甄别改写。来源：https://tldp.org/LDP/intro-linux/html/intro_07.html 、https://tldp.org/LDP/Bash-Beginners-Guide/html/intro_07.html 、https://tldp.org/LDP/intro-linux/html/index.html 、https://tldp.org/LDP/Bash-Beginners-Guide/html/
- 【TLDP 中文（CLDP）已死】CLDP 1997 年由台湾志愿者发起翻译 LDP HOWTO，2000 年代初停滞，内容极旧；tldp.cn 镜像存在但陈旧且 TLDP 官网在大陆访问受限；当前活跃的中文文档翻译力量已转向 Linux 内核官方文档（docs.kernel.org zh_CN）。结论：TLDP 中文不可作为内容来源。来源：https://zh.wikipedia.org/zh-hans/Linux%E4%B8%AD%E6%96%87%E6%96%87%E4%BB%B6%E8%A8%88%E5%8A%83 、https://docs.kernel.org/translations/zh_CN/index.html
- 【GNU coreutils 手册＝按任务分类法】9.11 版手册按功能分约 31 章：Output of entire files（cat/tac/nl）、Output of parts of files（head/tail）、Summarizing files（wc）、Operating on sorted files（sort/uniq）、Operating on fields（cut/paste/join）、Directory listing（ls）、Basic operations（cp/mv/rm/dd）、Special file types（ln/mkdir）、Changing file attributes（chmod/chown）、File space usage（du/df）、Printing text（echo/printf）、File permissions、Date input formats 等；单命令页结构='命令名: 一句话功能'→概述→选项说明→示例；末章 'Opening the Software Toolbox' 是面向新手的管道组合教学（who|cut|sort|uniq）。许可 GFDL 1.3+（无 Invariant Sections/封面文本）——与 MIT 不兼容不能抄文字，但分类框架与命令事实可用。来源：https://www.gnu.org/software/coreutils/manual/html_node/index.html
- 【结构对比结论】五种'讲一个命令/主题'的模板：man=参考式最小卡片（新手可用但不足以教学）；ArchWiki=任务参考式（假设前置知识，新手不适）；coreutils=功能分类+选项枚举（适合做题库分类骨架）；Debian Reference=表格+可跟打示例+警告框（半教学式，中文可用）；TLDP/Ubuntu tutorial=完整教学式（渐进+沙箱+小结+练习+警告）。对'基础薄弱新手'最适合的是教学式三件套：正文渐进讲解＋章末小结＋练习题（TLDP），加沙箱与自检（Ubuntu），以卡片式 NAME/SYNOPSIS/EXAMPLES 做速查层（man）；Diátaxis 的教训是教学层与参考层必须分开、各自声明前置假设。

**对 cmdtyper 的启示**：

- 讲解卡片模板可直接定型为 man 式四栏（结构惯例无版权问题）：名称+一句话功能（NAME）/ 语法骨架（SYNOPSIS，粗体=照打部分、斜体=占位符的惯例可映射为 TUI 中 token 高亮：固定 token 与可替换参数用不同颜色）/ 1-3 个带预置输出的示例（EXAMPLES）/ 相关命令（SEE ALSO，链接到题库其他命令形成学习图）。这与 cmdtyper 现有 token 化数据模型天然契合。
- 解决'跟打→默写'断层的中间脚手架，抄 TLDP 章结构+Ubuntu 手法组合：每课三段式=跟打正文→要点小结卡→练习题；练习题引入'填空默写'台阶——给出命令骨架挖掉 option 或 argument 让用户补全（比全默写低一级），再到提示语默写。另抄 Ubuntu 的：预置输出模拟 /tmp/tutorial 沙箱会话（符合'绝不执行真实命令'不变量）、每课末用 pwd/ls 预置输出做状态自检、rm/sudo/chmod 等危险命令设专门警告框和独立'危险命令'专题。
- 题库扩充大纲直接采用 Debian 参考手册第 1 章顺序（控制台基础→文件系统与权限→shell 环境：变量/通配符/重定向/别名→文本处理管道：grep/sed/cut/sort/uniq/tr/xargs），其'表 1.17'约 50 命令清单作为 v0.3 覆盖度对照表——命令清单属事实不受版权限制；对照现有 273 题查缺补漏。
- 题库主题标签体系照抄 coreutils 手册功能分章法（整文件输出/部分输出/统计/排序操作/字段操作/目录列表/基本操作/属性修改/磁盘用量/权限/时间戳），比按字母或按难度分类更利于'按任务找命令'的新手心智模型；分类框架本身可自由采用。
- 唯一可直接改写引入正文的来源是 Garrels 的两本 TLDP 书（BSD-3 风格，MIT 兼容）：其 Quickstart commands 表、Bash 快捷键表、chmod 权限码表、DOS vs Linux 对照表、各章 Exercises 题目都可汉化改写进 lessons/reviews 数据文件，条件是在项目 About/NOTICE 中保留其版权声明与许可条件；注意内容是 2008 年的，需人工甄别过时项。GFDL（ArchWiki/coreutils 手册）、CC-BY-SA（Ubuntu 文档/教程）、GPL-2+（Debian Reference/Handbook）的正文一律只借结构大纲和教学编排，不抄句子。
- 反面教材教训（ArchWiki）：cmdtyper 面向的正是被 Arch 劝退的人群——所以每课要显式声明并链接前置依赖（把 General recommendations 式'假设你已读过 X'变成'先学 X 课'的课程内跳转），讲解必须自足、不得出现 RTFM 式'请自行查手册'；同时 Diátaxis 教训要求把'课程教学层'（lessons）与'速查参考层'（卡片/字典）在 UI 上分开，各自明示读者假设。
- 中文内容策略：没有可整段搬运的高质量中文语料（Debian 手册中文完成度低、debian-reference 中文可读但有机翻痕迹、TLDP 中文已死、ArchWiki 中文同步滞后且为 GFDL）——cmdtyper 自写的中文讲解本身就是差异化价值；建议以 debian-reference 简体中文版的术语译法（索引节点、硬链接、正则表达式、管道等）作为项目术语表基准，保证与用户日后查阅官方中文文档时术语一致。

### 2.2 体系化课程与书籍（鸟哥 / TLCL / Linux Journey / Missing Semester / LUC 等）

**概要**：调研 8 套体系化 Linux 教材/课程。编排共识：shell 概念→导航(pwd/cd/ls)→文件操作→man 求助→通配符/重定向管道→权限→进程→软件包，vim 居中段，脚本压轴。讲解共识：范例占篇幅 50% 以上，"语法框+选项表+分层示例+易错点"四段式。练习形态四种：单题 quiz（Linux Journey）、反白答案例题（鸟哥）、连贯 playground 情景串（TLCL）、每日任务+extension（LUC）。许可证结论：仅 Linux Upskill Challenge（CC BY 4.0）可改写翻译引入 MIT 项目；Linux Journey（CC BY-SA）、TLCL（BY-NC-ND）、Missing Semester（BY-NC-SA）、鸟哥/runoob（专有）只能借结构与思路。

**发现**：

- 【鸟哥私房菜·编排】基础学习篇 CentOS7 版共 25 章、六大部分：第0章计算机概论（专为非科班读者）→ 1-4 章 Linux 是什么/规划/安装/首次登入与线上求助 → 5-8 章文件权限、文件目录管理、磁盘文件系统、压缩打包 → 9-12 章 vim、BASH、正则、Shell Scripts → 13-16 章账号/Quota/crontab/进程与SELinux → 17-24 章 daemon、日志、开机流程、软件安装(源码/RPM/YUM)、内核。注意：鸟哥把'权限'(第5章)放在'文件管理'(第6章)之前，比英文教材(TLCL第10章)早得多。来源：https://linux.vbird.org/linux_basic/centos7/
- 【鸟哥·讲解写法】每个命令四段式：语法框([root@study ~]# mkdir [-mp] 目录名称)→选项逐条说明→编号范例(简单命令2-3个、cp/find达7-9个，每例含指令+输出+行内#注解)→使用情境/误区补充。以 cp 为例篇幅：引入5%/语法10%/选项注意15%/范例55%/总结检查清单15%。正文穿插'例題：…答：…'即时巩固；章末'本章習題'含情境操作题+简答题，答案需鼠标反白才可见。来源：https://linux.vbird.org/linux_basic/centos7/0220filemanager.php
- 【鸟哥·为何成中文圈事实标准】(a)白话絮叨、案例驱动，针对非科班读者补计算机概论；(b)强调实作、内容体系完整；(c)网站20余年免费开放。批评：参数罗列像学习笔记、部分过时；简体版删口语化损失生动性——说明'语气'本身是价值。来源：https://book.douban.com/subject/30359954/ 、https://linux.vbird.org/
- 【鸟哥·版权】非开放许可。官方声明：非营利/非有偿用途（如教学讲义）可用，须注明出处并保留作者姓名与联系方式；其他用途须事先联系授权；下载内容未经同意不得再发布到互联网。结论：MIT 项目不能抄其文字，只能借章节编排、四段式讲法、例题/反白答案等结构。来源：https://linux.vbird.org/vbird/declare.php
- 【TLCL·编排】W. Shotts《The Linux Command Line》共37章：1引言→2什么是shell→3文件系统中跳转→4探究操作系统(ls/file/less)→5操作文件和目录(cp/mv/mkdir/rm/ln)→6使用命令(type/which/help/man)→7重定向→8从shell眼中看世界(展开/引用)→9键盘技巧→10权限→11进程→12-14 shell环境/vi/提示符→15-24软件包、存储、网络、find、归档、正则、文本处理→25-37 shell脚本。许可证：CC BY-NC-ND 3.0（NC+ND，不能改写引入 MIT 项目，仅可借结构）。来源：https://linuxcommand.org/tlcl.php 、中文版目录 http://billie66.github.io/TLCL/book/
- 【TLCL·讲解写法（第5章样本）】开场先场景辩护（'按更新时间筛选HTML文件复制'证明CLI优于GUI），再把通配符作为公共概念先讲(15%)；每命令'语法格式+选项表+实例表'三段(合计35%)；最大亮点是章末'创建游戏场(playground)'实战占40%：建沙盒目录后按 mkdir→cp→mv→ln→ln -s→rm 顺序连续操作，前一命令输出是后一命令操作对象，形成连贯叙事而非孤立片段，用 ls -li 验证 inode；无正式习题，以'反复练习并自行拓展 playground'收尾。来源：http://billie66.github.io/TLCL/book/chap05.html
- 【LinuxCommand.org·Learning the Shell】网站版10课顺序：What is the Shell→Navigation→Looking Around→A Guided Tour→Manipulating Files→Working with Commands→I/O Redirection→Expansion→Permissions→Job Control。同站与 TLCL 同作者，按 CC BY-NC-ND 对待。来源：https://linuxcommand.org/lc3_learning_the_shell.php
- 【Linux Journey·编排】三大阶段：Grasshopper(入门史/命令行/Text-Fu文本处理/进阶文本/用户管理/权限/进程/软件包)→Journeyman(设备/文件系统/启动/内核/init/进程监控/日志)→Networking Nomad(7个网络模块)。Command Line 模块19课，一课一命令：the-shell→pwd→cd→ls→touch→file→cat→less→history→cp→mv→mkdir→rm→find→help→man→whatis→alias→exit。来源：https://github.com/labex-labs/linuxjourney
- 【Linux Journey·每课形态与许可】课程文件为带 frontmatter 的 markdown：index 排序、quiz_question/quiz_answer（单题 quiz，答案是一个 flag 或单词，如'复制目录需要什么选项？'答'-r'）；正文小节化+约12个代码示例+'Common Questions'问答+文末 Exercise 链接。许可：CC BY-SA 4.0 附加条款（禁止做混淆性相似网站、转载须声明独立并链回）——SA 与 MIT 不兼容，不能直接抄文字；仓库自带 lessons/zh/ 中文翻译，可作术语对照参考。来源：https://github.com/labex-labs/linuxjourney 、https://raw.githubusercontent.com/labex-labs/linuxjourney/master/lessons/en/command-line/copy-cp-command.md
- 【MIT Missing Semester】经典2020版顺序：课程概览与shell→Shell工具和脚本→编辑器(Vim)→数据整理→命令行环境→Git→调试与性能分析→元编程→安全密码学→大杂烩；2026版改为9讲（shell入门→命令行环境→开发环境→调试→Git→打包→Agentic Coding→…）。每讲带 exercises，中文社区另有习题解答站。特点：假设读者会编程，第一课就讲管道重定向，节奏对纯新手过快。许可证：CC BY-NC-SA 4.0（NC，不能引入 MIT 项目，翻译需守同许可）。来源：https://missing.csail.mit.edu/ 、https://missing.csail.mit.edu/license/ 、https://missing-semester-cn.github.io/
- 【Linux Upskill Challenge·编排与许可】21天服务器管理向：Day1 ssh+ls/uptime/free/df/uname→Day2 导航/man/文件层级→Day3 sudo/时区→Day4 apt→Day5 less/dotfiles/history/Tab补全/nano→Day6 vim(vimtutor)→Day7 Apache服务→Day8 grep/cut/awk/管道→Day9 网络ss/nmap/ufw→Day10 cron→Day11 locate/find/which→Day12 SFTP→Day13 用户组→Day14 权限→Day15 软件源→Day16 tar/gzip→Day17 源码编译→Day18 logrotate→Day19 inode/硬链接/软链接/stat→Day20 脚本。形式：每日1-2小时任务式+extension扩展任务+精选外链。许可证：CC BY 4.0（2020年起开源）——唯一可合法改写翻译引入 MIT 项目的成体系教材，署名即可。来源：https://linuxupskillchallenge.org/
- 【菜鸟教程 runoob·编排与单页结构】目录：简介→安装→系统启动过程→目录结构→远程登录→文件基本属性→文件与目录管理→用户组→磁盘→vi/vim→yum/apt→Shell教程12篇→命令大全；无嵌入练习，仅独立'Linux 测验'页+用户笔记区。单命令页(ls)结构：一句话简介→一行语法→近20参数大表(信息密度最高)→分层实例(6个基础单行+约10个场景化小标题实例如'按大小反向排序')→注意事项(脚本中勿解析ls输出等)→权限知识延伸。定位是'中文man page替代/速查'，专有版权不可抄，但'参数表+场景化实例+注意事项'的版式可借。来源：https://www.runoob.com/linux/linux-tutorial.html 、https://www.runoob.com/linux/linux-comm-ls.html
- 【Linux命令行与shell脚本编程大全(第4版)】人民邮电出版社2022，636页，四部分25章：1-10章命令行(初识shell→走进shell→bash基础命令→更多命令→理解shell→环境变量→文件权限→文件系统→安装软件→文本编辑器)→11-16章脚本基础→17-23章高级脚本(sed/gawk/正则)→24-25章实用脚本。商业版权书，仅可借目录逻辑：它把'理解shell'(父子shell、内建命令)单列一章放在基础命令之后，是别家没有的编排。来源：https://book.douban.com/subject/35933905/
- 【Linux Survival】4模块+模拟终端('zoo>'提示符真输入真反馈)：模块1列目录/看文件/建目录/移动改名/切换目录；模块2路径/复制删除/权限/通配符；模块3主目录/man/找文件/合并/重定向；模块4目录树/磁盘空间/ps/管道/kill。每模块末一个 Quiz。免费但 Copyright 2000-2026 Guy Hummel，无开放许可——'模拟终端+阶段quiz'的形态与 cmdtyper 天然同构，只能借形态。来源：https://linuxsurvival.com/
- 【编排共识提炼】(1)全部8套教材都以'shell是什么+导航(pwd/cd/ls)'开局，紧接文件操作(cp/mv/rm/mkdir)，无一例外——文件系统心智模型先于一切；(2)'求助能力'(man/help/history/Tab补全)被普遍前置为生存技能（TLCL第6章、LJ第15-19课、LUC Day2/Day5、Linux Survival模块3、鸟哥第4章'首次登入与线上求助'）；(3)管道/重定向是公认的第一个'aha时刻'，位于文件操作之后、权限之前（TLCL第7章、LJ Text-Fu、LUC Day8）；权限→进程→软件包的后续顺序也高度一致；vim 一律居中段（鸟哥ch9/TLCL ch13/LUC Day6），脚本一律压轴。
- 【中文教材独特处理】(1)补前置知识：鸟哥加第0章计算机概论，runoob加'系统启动过程/目录结构'专篇——中文新手常缺英文教材默认的背景；(2)术语两岸差异是坑：台版'檔案/目錄/程序(=process)'vs 大陆'文件/目录/进程'，大陆'程序'=program，cmdtyper 用大陆术语并括注英文原词最稳妥；(3)中文教材普遍保留命令与选项英文原样、只译概念词（重定向 redirection、管道 pipe）；(4)简体版鸟哥删口语被读者批评，说明对话感、絮叨式白话本身是留存率的来源；(5)runoob 型'速查页'在中文圈实际充当 man page 替代，新手依赖参数表而非原理。

**对 cmdtyper 的启示**：

- 【填补跟打→默写断层的中间题型：挖空补 token】直接抄 Linux Journey 的单题 quiz 形态（quiz_question→单 flag/单词答案）：显示完整命令模板但挖掉 1 个 token（如 'cp __ src/ dst/' 问'递归复制需要哪个选项'），用户只输入该 token。这是现成的中间脚手架，与 cmdtyper 已有的 token 结构（tokens_consistency）完全对齐——可在 data/reviews/*.toml 里加 cloze 题型，难度阶梯：跟打全句→补1个token→补2-3个token→默写全句。
- 【情景串默写模式（抄 TLCL playground 结构，非文字）】一课一个连贯虚拟目录场景，6-8 条命令构成叙事（mkdir→cp→mv→ln→ls -li 验证 inode→rm），前一条的预置输出是后一条的操作对象。cmdtyper 的'预置输出+绝不执行真命令'机制天然支持；比孤立命令题记忆留存好，且 TLCL 证明该结构可占一章 40% 篇幅仍受欢迎。注意只借结构，TLCL 文字是 CC BY-NC-ND 3.0 不可抄。
- 【唯一可直接改写引入的内容源：Linux Upskill Challenge（CC BY 4.0）】其 Day1-21 命令清单与讲解可合法翻译改写进 MIT 仓库（在 data/ 相应文件和 README 中署名并注明 CC BY 4.0 来源即可）。优先挖：Day8 文本处理(grep/cut/awk/管道)、Day11 查找、Day16 tar/gzip、Day19 inode/链接、Day5 history/Tab补全——正是 273 题题库最需要扩充的实用方向。其 'extension 扩展任务' 形态也可抄为每课的选做加餐题。
- 【课程顺序按共识重排】cmdtyper 专题课的推荐主线：shell概念→导航(pwd/cd/ls)→文件操作(touch/cp/mv/mkdir/rm)→求助(man/help/history/Tab)→通配符→重定向/管道→权限→进程→软件包(apt/yum)→find/tar→脚本入门。特别注意把 man/--help/history 提为独立早期课程（所有教材共识，且对'基础薄弱新手'是自救能力）；vim 类内容居中段。
- 【每课讲解模板固定四段（融合鸟哥+runoob 版式）】①一句话场景引入（为什么需要这个命令，学 TLCL 的'CLI 比 GUI 强在哪'式动机）②语法框+选项拆解（对应已有 token_details/lexicon，学鸟哥语法框格式）③2-4 个带行内注释的范例（预置输出；篇幅占 50% 以上——鸟哥 cp 章范例占 55%、runoob 实例占比最大，这是被验证的比例）④易错点 1-2 条（学 runoob '注意事项' + Linux Journey 'Common Questions' 问答形态，如 'rm * .html 多打一个空格会删掉所有文件'）。
- 【例题穿插+反白答案机制（抄鸟哥形态）】讲解页每 1-2 个知识点后插一道'例題'，先显示题面，按键后才显示答案——TUI 里等价于'按空格揭晓'。比集中章末习题更适合 cmdtyper 的逐屏阅读节奏，也是中文读者熟悉的形态。
- 【题库扩充的合规路径】单条命令字符串本身（'ls -lh /var/log'）是无独创性的事实性内容，可自由收集自任何教材的命令清单（鸟哥/runoob/LJ 的命令覆盖面可作 checklist）；但讲解文字除 LUC(CC BY) 外一律须自写。Linux Journey 的 lessons/zh/ 中文翻译（CC BY-SA）可作术语翻译对照表参考（不抄句子），确保 cmdtyper 术语与主流中文教材一致：目录/进程/重定向/管道/权限，命令与选项保留英文。
- 【讲解语气】简体版鸟哥删口语被读者视为损失——cmdtyper 的中文讲解应保持对话式白话（'你可能会想…其实…'），而非 man page 式条目文。runoob 的教训反面：纯参数表无场景导致新手只会查不会用，cmdtyper 每个选项讲解都应绑定一个使用场景。

### 2.3 交互式练习平台与命令行游戏（Bandit / cmdchallenge / vimtutor / Anki 等）

**概要**：调研了 Bandit、cmdchallenge、KillerCoda、SadServers、vimtutor、bashcrawl、Terminus、keybr、ShortcutFoo、Execute Program、Linux Survival 及 Anki/SRS 生态。核心结论：解决"跟打→默写断层"的成熟配方是"提示分层给出+按掌握度衰减"（ShortcutFoo 的 Learn/Practice/Fight/Test 四段、Bandit 的三层信息、keybr 的置信度解锁）；cmdchallenge 为 MIT 许可，60 道"目标→写命令"题（含 oops 排障剧场）可直接翻译改写入库；间隔重复用于命令记忆有 srsh、Execute Program 等直接先例。

**发现**：

- 【cmdchallenge｜MIT 可直接借用】主仓库 gitlab.com/jarv/cmdchallenge（GitHub 镜像 github.com/jarv/cmdchallenge），LICENSE 为 MIT（已 clone 验证 /tmp/cmdchallenge-mirror/LICENSE）。challenges.yaml 共 60 题（grep slug 计数），字段：slug/description/example(参考答案)/expected_output.lines(期望输出行)/order(默认 true，可 false 表示行序无关)/regex(行为正则)/re_sub(判分前正则替换)/ignore_non_matching(忽略多余行)/expected_failures(必须判错的作弊样例，如 list_files 题里直接 echo 文件名)/completions(tab 补全候选)/learn(教学文本)/disp_learn(默认展开教学框)/tags/version(改题必须升版本刷缓存)。判分=在 Docker 沙箱跑用户命令后对输出做行匹配，另有 /c/s?slug= 端点在解题后展示他人通过的解法（多解展示）。来源：https://github.com/jarv/cmdchallenge
- 【cmdchallenge 题目难度谱系】主线从 hello_world(echo)→pwd→ls→cat→tail→touch→mkdir -p→cp/mv/ln→rm→grep→find→grep -r→提取 IP(正则)→wc→sort→cut→seq→sed 替换→求和(awk)→去重(uniq)→comm→awk 按键排序→IPv4 监听端口，完整覆盖新手到中级管道组合；另有两个叙事支线：12days_1~12（圣诞十二天，每题带 learn 教学框，disp_learn: true，先讲后练）和 oops_1~5（'/bin 被删光只剩 shell 内建'的排障剧场：pwd→echo *(代替 ls)→while read 读文件(代替 cat)→查 /proc 找进程名→kill）。来源：/tmp/cmdchallenge-mirror/challenges.yaml，https://github.com/jarv/cmdchallenge
- 【OverTheWire Bandit｜脚手架三件套】每关页面固定结构：Level Goal（目标+文件特征线索，如 bandit6：'inhere 目录下、human-readable、1033 字节、不可执行'）+ 'Commands you may need'（6 个命令名，各链到 man 页，但不给组合方法）+ 'Helpful Reading Material'。给判据和工具、扣住方法，是'提示给到命令名为止'的经典梯度。卡住协议明确写在首页：man → help(内建) → 搜索引擎 → 最后才进社区聊天；口号 'Don't panic! Don't give up!'。34 关难度弧线：0-13 基础文件操作与文本工具(grep/sort/uniq/strings/base64/rot13/hexdump)→14-19 网络(ssh key/nc/openssl/nmap)→20-27 权限与自动化(SUID/cron/脚本/受限 shell 逃逸)→28-33 Git。站点未见明确内容许可证，只能借关卡思路不能抄文本。来源：https://overthewire.org/wargames/bandit/ 、https://overthewire.org/wargames/bandit/bandit6.html 、https://mayadevbe.me/posts/overthewire/bandit/overview/
- 【vimtutor｜就地练习范式】结构：7 课约 30 小节；每小节=目标声明（** Press x to delete... **）+3-6 步编号指令+NOTE 补充框+'--->'标记的待修改练习行（学员直接在教材文本上操作，有的给 before/after 对照行）；每课末 SUMMARY 命令清单。刻意安排：undo 在学员已用过 x/dw/d$/dd 多个破坏性命令之后才教（1.2.7），让人先体会需求再给工具；原文强调 'do not try to memorize, learn by usage'。难度弧线：移动→单字删改→operator+motion 语法→搜索替换→文件级操作→帮助系统（元技能收尾）。归属 Vim 发行版（Vim license，charityware），结构思路可自由借鉴、文本不宜照搬进 MIT 项目。来源：https://raw.githubusercontent.com/vim/vim/master/runtime/tutor/tutor1 、https://www.cs.cmu.edu/~07131/f21/topics/readings/week-3
- 【KillerCoda/Katacoda 血统｜引导场景格式】场景=目录：index.json(元数据/步骤表)+intro.md+stepN/text.md+stepN/verify.sh+finish.md+setup.sh(自动预置环境)。verify 机制：步骤配 verify.sh 时界面出现 [CHECK] 按钮，exit 0 即过关进入下一步。同血统的 O'Reilly 步骤 schema 还有 solutionText/solutionScript 字段——学员卡住时可看官方解并由脚本代做，是'验证失败→展示答案'的兜底设计。官方样例仓库 github.com/killercoda/scenario-examples。来源：https://blog.1mg.org/posts/killercoda/create_scenario/ 、https://interactive-docs.oreilly.com/configuration/index-json.html 、https://github.com/Piotr1215/killercoda-cli
- 【SadServers｜排障题形态】'Like LeetCode for Linux'：每题一台预配置好故障的真实 VM + 问题描述 + 自动判定脚本（提交即自动 check），easy/medium/hard 三档难度+限时窗口，Pro 才有更长时限、无限重试、命令历史。代码不开源（README 明言为保护题解和基础设施暂不公开），fduran/sadservers 仓库无明确内容许可证——只能借'场景描述+终态判定'的题型思路。来源：https://sadservers.com/ 、https://github.com/fduran/sadservers 、https://itsfoss.com/news/sadservers/
- 【keybr｜渐进解锁与置信度模型】开源（github.com/aradzie/keybr.com，AGPL-3.0——思路可借、代码不能抄进 MIT）。机制：从语言最高频少量字母起步，逐键记录每次击键并计算逐键统计，课程自动偏向最弱键；当前所有已解锁字母置信度全部到 1（约等于达到目标速度 ~35WPM 且低错误率）才解锁下一个字母；设置里有'Extend alphabet size'滑杆可手动越级（防卡死逃生门）。社区暴露的坑对 cmdtyper 是现成避雷清单：稀有字母出现频率太低导致置信度涨不回来而长期卡关；靠停顿+爆发式输入可刷置信度作弊；一次性倒入 10 个标点键造成体验断崖。来源：https://github.com/aradzie/keybr.com 、https://groups.google.com/g/keybr/c/nWWdhRpFn1c 、https://groups.google.com/g/keybr/c/9SDXQDvoapE
- 【ShortcutFoo｜提示衰减四段式（最接近'跟打→默写'解法）】每个 dojo 分课，每课三段：Learn（先显示'指令名+按键答案'照打，复习轮不再给提示、纯回忆；全对才算完成且当天不能重刷）→Practice（自定节奏，提示可开可关的中间档）→Fight（限速对抗、洪水式出题）→Test（≥70% 才升段，Noob 到 Shark Norris 六级腰带）。调度用间隔重复，按功能域分簇（编辑/导航/调试），先母手势再加修饰键再进多步流程的渐进组块。商业闭源，只借机制。来源：https://www.pcworld.com/article/436020/review-shortcutfoo-turns-you-into-a-keyboard-hotkey-ninja.html 、https://www.shortcutfoo.com/app/dojos/command-line/learn
- 【Execute Program｜SRS 用于编程概念的成熟先例】课程=散文与可运行微例交错，常要求'预测这段代码的输出'再揭晓；单课 5-10 分钟，数百个小例子复杂度缓升；完成的课进入间隔重复队列，间隔逐次拉大，一个月后仍会提醒复习；复习答错不扣分（除非主动'give up'），进度机制刻意奖励推进新课而非刷记忆时长——低挫败设计。商业闭源，机制可借。来源：https://www.executeprogram.com/spaced-repetition 、https://mike.place/2020/executeprogram/ 、https://code.brettchalupa.com/execute-program-review
- 【Terminus（MIT 课程项目）与 bashcrawl｜游戏化先例】Terminus：文字冒险，所有动词即真实 UNIX 命令（cd LOCATION 移动），旁侧图形窗口把敲的命令和世界变化可视化关联；用户研究证实玩家能把游戏技能迁移到真实终端任务。bashcrawl：用目录当房间、文件当物品的地牢，cd/ls -F/cat/变量收集宝物，刻意保持脚本极简让新手能读懂游戏本体源码来二次学习——'游戏本身是教材'；许可证 GPLv3，内容不能进 MIT 项目，思路可借。来源：https://web.mit.edu/mprat/Public/web/Terminus/Java/CMS.590Game2FinalReport.pdf 、https://www.csteachingtips.org/tip/gamify-command-line-learning-through-terminus-text-based-adventure-game 、https://gitlab.com/slackermedia/bashcrawl 、https://opensource.com/article/19/10/learn-bash-command-line-games
- 【Linux Survival｜与 cmdtyper 同路线的模拟终端】明确使用'simulated Linux terminal'（预置响应、非真实 shell），4 模块渐进（基本导航→路径/权限/通配符→man/查找/重定向→磁盘/进程/管道），每模块末有小结页+独立 Quiz；定位哲学'几百个命令里只需约一打就能完成大多数任务'——范围控制的范本。内容版权 Guy Hummel 所有、无开放许可证，只借结构。来源：https://linuxsurvival.com/
- 【Anki 牌组与命令记忆 SRS 生态】AnkiWeb 现成牌组：'Linux Commands'（https://ankiweb.net/shared/info/1171788920）与 '106 Linux Commands'（https://ankiweb.net/shared/info/142660396）——AnkiWeb 共享牌组通常不标许可证，默认不可直接抄内容，只可参考卡片形态。更相关的先例：srsh（github.com/ryanbloom/srsh）专为 shell 命令做 SRS，卡片=自然语言任务（'列出含隐藏文件的所有文件'）→答案命令（ls -a），SQLite 存进度、按递增间隔重现——形态与 cmdtyper 默写题完全同构；repeater（FSRS 算法、目标 90% 保留率）、TerminalFlashcards（SM-2）、clsr、ariadne（代码卡片附带练习用样例文件）提供算法与数据结构参考。来源：https://github.com/ryanbloom/srsh 、https://github.com/shaankhosla/repeater 、https://github.com/gottenheim/ariadne
- 【梯度设计的共性规律（跨平台归纳）】所有平台的'示范→独立'过渡都靠同一杠杆：控制'给出信息的层级'而非改变题目本身。层级从高到低：完整答案照打（ShortcutFoo Learn 首轮/vimtutor 步骤指令）→讲解框+题目（cmdchallenge learn 框/12days）→提示可选开关（ShortcutFoo Practice）→只给目标+工具名（Bandit 的命令清单）→只给目标（cmdchallenge 主线后段/SadServers）→限时纯回忆（Fight/Test）。衰减触发条件有两种：显式分段（ShortcutFoo 按课内阶段）与掌握度驱动（keybr 置信度、Test 70% 门槛）。答错处理：立即重现直到正确（ShortcutFoo）、无惩罚重试（Execute Program）、verify 失败给 solution（Katacoda 系）。来源：见上述各条
- 【1986 年 HCI 经典实验佐证】学习电子表格命令的对照实验：纯解题练习比引导练习更难但测验成绩最好，且练习题在时间上分散安排能进一步提升成绩；只读不练的对照组最差——支持'先引导后独立+间隔分布'的编排。来源：https://www.academia.edu/1324923/Design_and_implementation_of_interactive_tutorials_for_data_structures（转引自 CMU 07-131 阅读材料 https://www.cs.cmu.edu/~07131/f18/topics/readings/week-2）

**对 cmdtyper 的启示**：

- 直接抄题库：cmdchallenge 60 题为 MIT 许可，可整体翻译改写成中文默写/情景题入库（建议保留原 slug 便于溯源、在 data/ 中注明 'adapted from cmdchallenge (MIT, jarv)'）。最有价值的三块：主线 hello_world→IPv4_listening_ports 的难度谱系可直接映射为 cmdtyper 的默写题分级；oops_1~5 排障小剧场可改编为中文叙事关卡（cmdtyper 是预置输出，正好把'进程删了 /bin'的输出预演出来）；12days 的 learn 框+disp_learn 模式=先讲后练的题内教学框，可加进 reviews/*.toml schema。本地已有克隆：/tmp/cmdchallenge-mirror/challenges.yaml
- 抄判分 schema：把 cmdchallenge 的 expected_output.order=false / regex / ignore_non_matching 思想移植进 core/matcher——默写判分支持'等价命令列表'（ls -la 与 ls -al 同判对）；把 expected_failures 反转为'常见错误答案→针对性中文讲解'字段（学员答 rm dir 时提示'目录需要 -r'），这是 cmdtyper 无真实执行环境下最接近'输出判等价'的替代品
- 解决跟打→默写断层的具体管线（ShortcutFoo 四段式+Bandit 提示分层的合成）：每条命令走 4 阶段——①跟打（现状，全显）→②半掩填空（隐藏 flag 或关键 token 的 cloze，TypingEngine 已按 token 建模，实现成本低）→③目标默写（只给中文任务描述，h 键分层揭示：第 1 次按=相关命令名清单，第 2 次=关键 flag，第 3 次=完整答案；用提示次数折算掌握度）→④限时复测。阶段晋升由掌握度驱动（参考 keybr 置信度=速度+准确率双条件），但必须提供手动越级入口（keybr 社区最大怨点是卡关无路可走）
- 避雷清单（来自 keybr 社区实证）：a) 稀有 token（如 chmod 的数字权限）出现频率低会导致掌握度只降不升→调度器需保证每个未掌握 token 的最低出现频率；b) 掌握度公式若纯看瞬时速度可被'停顿+爆发'刷分→用滚动窗口多次采样；c) 不要一次性解锁一大批新符号（keybr 标点断崖教训）——cmdtyper 符号训练接入主线时应逐个引入
- 间隔重复落地：review_loader 已存在，建议加 SM-2 或 FSRS 简化版调度（srsh/TerminalFlashcards 是单文件级参考实现，repeater 的 FSRS+Markdown 结构可参考）；卡片形态照 srsh：'中文任务描述→命令'；采纳 Execute Program 两条低挫败原则——复习答错不惩罚（只重排队列）、进度奖励偏向推进新内容而非刷复习
- 抄 vimtutor 的'就地练习'：lesson 讲解页内直接嵌入 1-2 行 ---> 迷你练习（讲完 -r 立刻在讲解页打一次 rm -r dir），而非讲解/练习分屏切换；每课末尾加 SUMMARY 命令清单页；模仿其'先让学员用破坏性命令、再教补救'的顺序编排（如先教 rm 再教 -i/回收站习惯）
- 抄 Katacoda/KillerCoda 的兜底结构：默写题失败 N 次（建议 3 次）后显示完整 solution 并自动转入错题队列（对应 solutionText 设计）；lesson 末尾加'check 关卡'（本课命令混合默写，全对才标记课程完成，对应 verify.sh+[CHECK] 与 ShortcutFoo Test 的 70% 门槛）
- 抄 Bandit 的题面写法（用于扩充讲解与新题）：题面=目标+可验证的特征线索（'找出 inhere 下 1033 字节、可读、不可执行的文件'式属性谜题），配'你可能需要的命令'清单——这类 find/grep 属性谜题目前 cmdtyper 题库缺失，且天然适合预置输出形态；34 关的四段难度弧线（文件→文本工具→权限/进程→网络/git）可作为 v0.3 题库扩容的目录骨架
- 抄 cmdchallenge 的多解展示：默写判对后展示'其他等价写法'（tail -n5 / tail -5），并各附一句差异说明——数据来源就是 matcher 里维护的等价命令列表，一份数据两用
- 范围控制与定位自检（Linux Survival）：'约一打命令覆盖大多数任务'——扩题库到 500+ 时保持金字塔结构：核心 12-20 命令做深（每个配完整 lesson+全部四阶段+复习卡），长尾命令只做跟打+讲解；每模块（当前的专题）末尾加 Quiz 而非只有练习
- 不能抄的内容红线：bashcrawl（GPLv3）、vimtutor 文本（Vim license）、Bandit 关卡原文（无许可证）、SadServers 场景（明确不开源）、AnkiWeb 牌组（无许可证）、keybr 代码（AGPL-3.0）——以上只借结构与机制；可放心改写引入的只有 cmdchallenge（MIT）；机制/算法本身（SRS、置信度解锁、提示分层）不受版权保护可自由实现

### 2.4 命令速查与解释工具（tldr / cheat.sh / explainshell 等）

**概要**：tldr pages 内容许可为 CC-BY 4.0（非 MIT），署名后可改写入 MIT 项目；简中有 1532 页，覆盖 cmdtyper 134 个基础命令中的 104 个，格式极易转 TOML，是题库/讲解首选来源。cheat/cheatsheets 为 CC0（约275命令）、eg 为 MIT（75命令），可直接用。explainshell 代码 GPL-3、数据沿用 man 页上游许可，只能借 token 匹配思路。navi 的变量槽填空+fish 的灰字实时提示，正是"跟打→默写"中间脚手架的现成方案。共识颗粒度：一句祈使句人话+至多8个典型示例。

**发现**：

- 【tldr 许可证——推翻任务假设】tldr pages 的页面内容许可是 CC-BY 4.0（不是 MIT）：LICENSE.md 原文 "This work is licensed under the Creative Commons Attribution 4.0 International License (CC-BY)"；仅 scripts/ 目录代码是 MIT。CC-BY 4.0 无 ShareAlike 条款，与 MIT 项目兼容：改写引入 cmdtyper 需在 README/关于页署名 tldr-pages team and contributors、注明出处与修改。来源：https://github.com/tldr-pages/tldr/blob/main/LICENSE.md
- 【tldr 规模与中文覆盖】（2026-07-26 本地浅克隆实测）：英文 pages/ 共 7391 页，简中 pages.zh 共 1532 页（约 21%，其中 common 956、linux 225），繁中 pages.zh_TW 577 页。仓库 63.2k star、23k+ commits，社区活跃。翻译进度看板：https://lukwebsforge.github.io/tldri18n/ 。来源：https://github.com/tldr-pages/tldr
- 【tldr 与 cmdtyper 题库重合度】（本地比对实测）cmdtyper 273 条命令题去重后 134 个基础命令，tldr 简中已覆盖 104 个（78%）；英文覆盖 133/134（唯缺 shell 历史符 !!）。简中缺失的 30 个（chgrp dig dirname dpkg env for free groups host id journalctl last nc ncdu passwd pwd readlink reset ssh-keygen strace traceroute ufw useradd userdel usermod visudo watch whereis while）英文页全有，可从英文翻译。
- 【tldr 条目写作规范】每页最多 8 个示例（建议约 5 个）；描述强制祈使语气（List 而非 Listing）、每例一句话；占位符 {{placeholder}}，路径写 {{path/to/file}}，多值 {{item1 item2 ...}}，互斥 {{item1|item2}}，数值区间 {{1..5}}；破坏性操作（如磁盘设备）必须用占位符防止误粘贴执行；示例排序：位置参数→选项→带参选项，--help/--version 固定放最后两条；优先 GNU 长选项；技术词用反引号；有 tldr-lint 自动校验。来源：https://github.com/tldr-pages/tldr/blob/main/contributing-guides/style-guide.md
- 【tldr 中文质量】抽查 pages.zh/common/tar.md：8 个示例全部翻译，占位符也本地化为 {{路径/到/文件1}}、{{目标文件.tar}}，术语准确、整体通顺；瑕疵是个别直译（"详细地提取"对应 verbosely）和术语不统一（提取/解压缩混用），轻度润色即可用。来源：https://raw.githubusercontent.com/tldr-pages/tldr/main/pages.zh/common/tar.md
- 【explainshell 实现思路】代码 GPL-3.0。流程：mandoc -T markdown 转换 man 页→选项提取器（解析 roff 宏，或 LLM 模式——LLM 只返回原文行号范围而非生成文字，"hallucinations are structurally impossible"）→SQLite 三表（manpages 原文 / parsed_manpages 选项 JSON / mappings 命令名→man 页映射，支持 git commit→git-commit 子命令与别名）→bashlex 解析用户命令为 AST，逐 token 与已知选项列表匹配后高亮对应帮助文本。来源：https://github.com/idank/explainshell
- 【explainshell 数据不可借用】README 明确：数据库含数万个从 Ubuntu/Arch 包提取的 man 页，"each of those pages keeps its own upstream license"（各自沿用上游许可，含 GPL/GFDL 等），作者只拥有薄薄一层 schema/选取工作；代码 GPL-3 与 MIT 不兼容。结论：只能借架构思路，任何文本数据都不能进 cmdtyper。来源：https://github.com/idank/explainshell
- 【cheat.sh 聚合源及许可】cheat.sh 自身代码 MIT。聚合源：chubin/cheat.sheets（MIT，LICENSE 原文核实 "MIT License / Copyright (c) 2017 Igor Chubin"）、tldr-pages（CC-BY 4.0）、cheat/cheat（代码 MIT）、learnxinyminutes-docs（CC-BY-SA 系，不可直接抄）、StackOverflow（CC BY-SA，不可直接抄）。来源：https://github.com/chubin/cheat.sh 、https://github.com/chubin/cheat.sheets/blob/master/LICENSE
- 【cheat/cheatsheets 是 CC0】社区速查表仓库 cheat/cheatsheets 许可为 CC0 1.0 Universal（LICENSE 藏在 .github/LICENSE.txt，GitHub API 检测不到故显示 null，已 curl 原文核实）；根目录约 275 个命令速查文件；格式：`# 注释说明` + 命令行，空行分块，行长≤80，可选 YAML front matter（syntax/tags）。CC0 等同公有领域，无署名义务，可直接翻译改写入 cmdtyper。来源：https://github.com/cheat/cheatsheets 、https://raw.githubusercontent.com/cheat/cheatsheets/master/.github/LICENSE.txt
- 【eg：man EXAMPLES 的替位工具】srsudar/eg，MIT 许可（全仓库统一），75 个命令的示例文件，markdown 格式；写法特点：先给贴近真实场景的具体用法（真实文件名），再给带 <placeholder> 的 Basic Usage 抽象语法，再按常见用途分节——"具体在前、抽象在后"与 man 相反，教学价值高；行长≤80。英文内容，翻译后可直接用。来源：https://github.com/srsudar/eg
- 【navi 的交互格式】Apache-2.0。.cheat 格式：% 标签行、# 一句话描述、命令模板中变量用 <branch> 尖括号、`$ branch: git branch | awk ...` 定义变量候选值来源；配合 fzf：选模板→逐个填变量→生成完整命令（写入 shell 历史的是真实命令）。README 自述价值 "it will teach you new one-liners"。这种"命令骨架固定+变量槽填空"结构是现成的中间脚手架设计。来源：https://github.com/denisidoro/navi
- 【bropages 已死】客户端 hubsmoke/bro 为 BSD 许可，标注 DEPRECATED，2022-03-04 归档只读，gem 在现代 Ruby 上运行报错；官方也推荐改用 tldr/cheat。其"社区提交示例+投票排序"模式是唯一遗产，内容源不可依赖。来源：https://github.com/hubsmoke/bro 、http://bropages.org/
- 【fish 自动建议/语法高亮的学习机制】fish 设计文档：autosuggestions、语法高亮、补全"just work"零配置，须全程异步保持响应（"bad performance...trains users to route around slow use cases"）；PR #8376 的维护者论述点明二者本质是两类反馈——语法反馈（对错着色）与状态反馈（建议灰字从哪开始）。对学习者：输入时即时看到命令合法性（高亮红/正常）+ 灰色 ghost text 提示剩余部分（可接受可忽略），是天然的"渐隐提示"记忆脚手架。来源：https://fishshell.com/docs/current/design.html 、https://github.com/fish-shell/fish-shell/pull/8376
- 【man EXAMPLES 节不可作管道】EXAMPLES 节是惯例但填写极不一致、位置不标准：HN 讨论举证 gcc man 页 17549 行且无 EXAMPLES 节、find 超 1100 行 vs tldr 37 行；tldr 项目 2013 年因此而生（README 名场面：man tar 第一个选项是磁带机的 -b blocksize）。且 man 页各有上游许可（见 explainshell 条）。结论：不要写 man 页抓取管道，直接用 tldr/cheatsheets。来源：https://news.ycombinator.com/item?id=7166257 、https://tldr.sh/
- 【颗粒度共识】所有工具收敛到同一结论：新手要的是"一句祈使句人话 + 若干典型示例（约5个、不超过8个）+ 占位符高亮"，完整选项列表交给 man（tldr style guide 原话 "It's OK if the page doesn't cover everything; that's what man is for"）；示例按真实使用频率排序而非字母序；eg 证明"具体示例先于抽象语法"；navi 证明示例要可执行、可填空。来源：https://github.com/tldr-pages/tldr/blob/main/contributing-guides/style-guide.md

**对 cmdtyper 的启示**：

- 题库/讲解扩充主管道 = tldr pages.zh：写一个转换脚本解析其 markdown（格式极规整：页首 > 描述、`- 描述：` + 反引号命令），生成 TOML 草稿——summary 取页首描述、dictation.prompt 取示例描述、command 取实例化占位符后的具体命令。人工只需两件事：把 {{占位符}} 填成具体值（tokens_consistency 测试要求 token 拼接==命令串，占位符必须实例化）、手写 simulated_output（tldr 无输出，这是无法自动化的部分）。合规动作：在 README 和应用"关于"页加 CC-BY 4.0 署名（tldr-pages team and contributors + 仓库链接 + 已修改声明）。
- 扩充优先级现成清单：先补 tldr 简中已有但措辞可直接搬的 104 个已收录命令的讲解；再翻译 30 个简中缺失命令的英文页（列表见 findings）；新命令候选从 pages.zh/common(956页)+linux(225页) 里按新手相关性筛，等于免费获得一份按社区共识排好的命令池。
- "跟打→默写"中间脚手架直接抄 navi+fish 的组合：新增一个"填空模式"（cloze）——命令骨架固定显示（tar czf ▁▁▁ ▁▁▁），用户只默写变量槽或只默写选项部分，数据层用 tldr 的 {{占位符}} 约定天然标记哪些 token 是槽位；再叠加 fish 式渐隐提示——初期目标命令以灰色 ghost text 显示、用户跟打，随正确次数增加灰字逐段消失（先隐变量、再隐选项、最后全隐=默写），配合已有的逐字符对错着色（TypingEngine 已具备）。这条脚手架链完全落在现有 AppState/flow 架构内，只需新增状态与渲染。
- lexicon.rs 三个可抄的 explainshell 机制（思路不涉数据，无许可问题）：(1) 选项簇拆解——查 -la 时拆成 -l、-a 分别命中；(2) --opt=value 与 -oVALUE 的分割匹配；(3) 子命令上下文已支持（context 字段的 starts_with 即 explainshell mappings 表的简化版），可再补"git commit → 独立词表"式的二级映射。lexicon 条目文本来源：用 cheat/cheatsheets（CC0，零义务）和 tldr（CC-BY 署名）的描述改写为中文，绝不从 man 页抓。
- 可直接引入内容的源（许可结论）：cheat/cheatsheets = CC0 1.0，约275命令，随便改写无署名义务，最省心；srsudar/eg = MIT，75命令深度示例（含典型用途分节），翻译引入需保留版权声明；tldr pages = CC-BY 4.0，覆盖最广+唯一有现成中文，署名即可；chubin/cheat.sheets = MIT。不可引入：explainshell 数据（GPL-3+man 上游许可）、StackOverflow/learnxinyminutes（CC-BY-SA）、bropages（已死）。navi 是 Apache-2.0 且只借格式思路，无引入问题。
- 讲解写作规范直接采纳 tldr style guide 并写进 CLAUDE.md/内容规范：每条命令 summary 用一句祈使句；每题的 dictation.prompt 措辞对齐 tldr 中文描述句式（动词开头："创建…""列出…"）；lesson 示例≤8个、常用在前；讲解优先展示 GNU 长选项并括注短选项（对新手长选项自解释）；破坏性命令（rm、dd、mkfs）的示例一律用占位符路径并加警示——这条 tldr 规则对"绝不执行真实命令"的 cmdtyper 是天然契合的安全叙事。
- eg 的"具体示例在前、抽象语法在后"排序值得用于 lesson 结构：先给带真实文件名的完整命令（可跟打），再给带槽位的通式（用于填空模式），最后才是选项明细（lexicon/token_details）——正好对应练习难度递增序列。

### 2.5 社区论坛新手痛点（linux4noobs / AskUbuntu / 中文社区等）

**概要**：用 Stack Exchange API 实测了 Ask Ubuntu / Unix&Linux / SO(bash+linux) 各前 50-60 高票题，结合 r/linux4noobs FAQ 与 flair 存档、V2EX Linux 节点、知乎/CSDN 中文资料。结论：命令行新手真实痛点高度集中在软件安装(apt)、解压(tar)、权限(chmod/sudo)、重定向(2>&1)、PATH/.bashrc、find/grep、磁盘排查、进程/端口、ssh 十余簇，且提问几乎全是"任务式/报错式"；教材(鸟哥/TLCL)按知识体系编排与此严重错位。中文特有痛点：换源、中文乱码、输入法、command not found。可直接抄的内容源：jaywcjlove/linux-command(MIT，中文，580+命令)与 tldr-pages(CC-BY 4.0，有中文)。

**发现**：

- 【方法与数据】Reddit/LinuxQuestions 网页均反爬（reddit 403、LQ 403、redlib 镜像 bot-check），改用 Stack Exchange 官方 API 抓到 Ask Ubuntu、Unix&Linux、SO(bash/linux tag) 各前 50-60 高票题原始分数与标题（https://api.stackexchange.com/2.3/questions?order=desc&sort=votes&site=askubuntu 等），Reddit 用 Wayback Machine CDX/快照取证。以下痛点聚类均有具体高票题目佐证。
- 【痛点簇1·软件安装/包管理=英文区最大簇】Ask Ubuntu 全站第1名即 'How to list all installed packages'(2748票)；前60里还有 'kept back packages 怎么办'(1779)、'如何删 PPA'(1695)、'命令行装 .deb'(1581)、'unable to lock /var/lib/dpkg'(1175)、'apt vs apt-get 区别'(860)、'sudo apt-get update 到底干了什么'(716)、'unmet dependencies'(671)。注意其中三条是拿报错原文当标题问的。来源: https://askubuntu.com/questions?tab=Votes (API 实测)
- 【痛点簇2·tar/解压参数记不住】'How to unzip a zip from the Terminal'(2317)、'extract .tar.gz 用什么命令'(1272)、'.xz tarball'(1037)、'tar 解压到指定目录'(1000)、'如何安装 .tar.gz'(706)；Unix SE 'Zip all files in directory'(815)、'zip/unzip 命令行'(692)。该主题在中英文社区都是 top 级。来源: askubuntu/unix API 实测
- 【痛点簇3·权限与 sudo】SO 'chmod 递归改文件夹权限'(2177)、'sudo 重定向到无权限文件为什么失败'(1153)；AU '把用户加进 sudoer'(1032)、'改文件夹权限和属主'(793)；论坛大量 chmod -R 777 事故贴（如 HPE 社区 root 下 chmod -R 777 / 求救 https://community.hpe.com/t5/operating-system-hp-ux/chmod-r-777-from-as-root/m-p/2511991 ）。来源: API 实测 + WebSearch
- 【痛点簇4·文件基本操作的坑】AU 'cp 复制文件夹内容'(1379)、'重命名目录'(839)、'删除非空目录'(809)、'按扩展名递归删除'(947)；SO 'symlink 怎么建'(2254)、'删 symlink'(1219)、'cp 强制覆盖不确认'(924)、'Argument list too long'(912)。来源: API 实测
- 【痛点簇5·重定向/管道是概念性最大坑】SO '2>&1 是什么意思'(3338)、'stdout+stderr 一起追加到文件'(2119)、'输出同时到文件和屏幕(tee)'(1686)、'只 pipe stderr'(1323)；AU '终端输出存到文件'(1447)。来源: API 实测
- 【痛点簇6·PATH/环境变量/.bashrc】Unix SE 'How to correctly add a path to PATH'(1412)、SO '重载 .bashrc'(2273)、'永久设置 $PATH'(1116)、'export 与不 export 的区别'(1278)、'删除已 export 的变量'(2589)、AU '目录加进 PATH'(1001)。与中文区高发的 command not found 同根。来源: API 实测
- 【痛点簇7·find/grep】SO linux tag 全站第1名 'Find all files containing a specific text'(7705票)，另有 'find 通配符递归'(3155)、'find 排除目录'(2341)、'grep -R 排除目录'(1205)、'find -exec missing argument'(788, 又是报错式提问)。来源: API 实测
- 【痛点簇8·磁盘空间排查】Unix SE '目录大小怎么看'(1677)、'磁盘空间去哪了'(837)、'ls 按 MB 显示'(936)；AU '目录总大小'(1254)、'剩余空间'(845)、'清理 /boot'(660)、'删旧内核'(772)。来源: API 实测
- 【痛点簇9·进程/端口/杀不死】SO '杀掉占用某端口的进程'(1535)、'按名字杀进程'(892)；Unix SE '找占端口的 PID'(832)、'kill -9 都杀不掉怎么办'(656)、'top 按内存排序'(623)。来源: API 实测
- 【痛点簇10·ssh/远程/后台】AU 'ssh 断开后进程继续跑'(1044)；Unix SE 'ssh 两机间拷文件'(1339)、'scp 远程拷回本地'(572)、'nohup/disown/& 区别'(812)、'有公钥还是要密码'(746)。来源: API 实测
- 【痛点簇11·服务/定时任务】AU 'cron 日志在哪'(1216)、'启停服务'(957)、'开机跑脚本'(789)、'设 cron job'(660)；Unix SE 'systemctl status 看完整日志'(1084)。来源: API 实测
- 【痛点簇12·shell 脚本入门带】SO bash tag 前50几乎全是脚本初级问题：'脚本所在目录'(6422)、'判断目录存在'(4555)、'字符串包含'(3664)、'字符串拼接'(3617)、'解析命令行参数'(2632)、'for 循环范围'(2281)——这是'会打命令'到'会写脚本'之间的真实台阶。来源: API 实测
- 【痛点簇13·vim 生存问题】SO 'How do I exit Vim?' 5550 票、331 万浏览（API 实测 view_count=3316419）——单个问题的量级证明'vim 退出'值得单独做题。
- 【痛点簇14·概念混淆区】Unix SE 'terminal/shell/tty/console 区别'(1637)、SO 'sh 与 bash 区别'(1927)、'login shell 与 non-login'(593)、'/usr/bin vs /usr/local/bin'(720)、'man 页括号数字含义'(775)、'-- 双横线含义'(852)、'~ 为什么代表家目录'(865)。来源: API 实测
- 【痛点簇15·误操作急救】Unix SE '误按 Ctrl-S 终端假死'(974)、AU '忘了管理员密码'(834)、'Ubuntu 死机怎么办'(774)；rm -rf 事故是跨论坛文化现象（AnandTech 'Whoops' 事故贴 https://forums.anandtech.com/threads/whoops.2237973 、DEV 'rm -rf overdose' https://dev.to/voluntadpear/rm-rf-overdose-mfk ）。
- 【Reddit 侧证据·痛点性质不同】r/linux4noobs 官方 FAQ 存档全部是元问题：'该装哪个发行版''迁移前该知道什么''去哪找安装教程'（Wayback 快照 https://web.archive.org/web/20240827160421/https://www.reddit.com/r/linux4noobs/wiki/faq/ ）；官方 flair 分类为 distro selection / hardware-drivers / learning-research / migrating to Linux / Meganoob（CDX 记录）；2022 top 快照头部帖为 '登录界面变成终端了''刚试 Linux 就把电脑搞坏了' 等系统损坏/引导/驱动帖（ https://web.archive.org/web/20220815082424/https://old.reddit.com/r/linux4noobs/top/ ）。即：Reddit 新手区问'系统坏了/选哪个'，Q&A 站问'这条命令怎么写'——后者才是 cmdtyper 能教的。
- 【与教材的错位】鸟哥《Linux私房菜》CentOS7 版目录（ https://linux.vbird.org/linux_basic/centos7/ ）：前4章计概/磁盘分割/安装，第7章文件系统、14章 Quota/RAID/LVM、19章开机流程、24章内核编译——这些主题在三大 Q&A 站高票区几乎为零；而高票最大簇'软件安装'被排到第21-22章且讲 RPM/YUM（多数新手用 apt）。TLCL（ https://linuxcommand.org/tlcl.php ）编排较贴近 CLI 但也无'报错排查/误操作急救'类内容。错位本质：教材按知识体系自底向上，新手按任务('How do I X')和报错原文('unable to lock dpkg')提问；高票问题里没有一个是'请系统讲讲文件系统'。
- 【高票答案的共性写法】实测抓取三条典范答案正文：tar.gz 答案(2039票, https://askubuntu.com/q/25347 )结构=第一行给可复制命令→逐 flag 单行解释(f 必须在最后且紧跟文件名/z=gzip/x=extract/v=verbose)→给一个 -C 变体；2>&1 答案(3801票, https://stackoverflow.com/q/818255 )结构=最小概念(fd1/fd2)→先驳'直觉写法 2>1 为什么错(会被当成文件名1)'→再给正解；PATH 答案(1584票, https://unix.stackexchange.com/q/26047 )结构='最短可用两行'→'该写进哪个文件(.profile vs .bashrc, 点名常见错误 ~/.bash_rc)'→'不要写在哪里及原因'。共性五条：命令先行、flag 逐字拆解、明确纠正典型错误直觉、只给1-2个高频变体、注明边界条件。
- 【中文特有痛点1·换源】'Ubuntu 换清华源'是中文新手第一课级刚需，保姆级教程遍布 CSDN/博客园/腾讯云/阿里云（官方帮助 https://mirrors.tuna.tsinghua.edu.cn/help/ubuntu/ ；CSDN 例 https://blog.csdn.net/fanyun_01/article/details/124519821 ）。踩坑点：版本代号(focal/jammy)不对应、忘备份 sources.list、security 源替换风险。英文社区完全没有此问题。
- 【中文特有痛点2·中文乱码/编码】CSDN 高频主题：终端/文件中文乱码(https://blog.csdn.net/qq_45988641/article/details/123631126 )、Windows 上传文件名变问号(GBK vs UTF-8, https://blog.csdn.net/chenbaixing/article/details/118261406 )、unzip 解压乱码、locale-gen 报 command not found 的叠加故障。
- 【中文特有痛点3·command not found 高发】CSDN 有'终极指南'级文章（ https://blog.csdn.net/sinat_23329907/article/details/143416172 ），成因带中国特色：教程跨发行版混用(在 CentOS 上抄 apt-get)、sudo secure_path、Windows 换行符导致脚本报错(:set ff=unix)。
- 【中文特有痛点4·输入法/桌面环境】V2EX Linux 节点当前页即有 'Linux 输入法求助'、'fcitx5 拼音不自动调词频'、'腾讯会议 Linux 版 N 卡问题'、Wayland+NVIDIA 抱怨（ https://www.v2ex.com/go/linux ）。另知乎入门高赞普遍建议新手用虚拟机/WSL 起步（ https://www.zhihu.com/question/27811101 ），即中文新手多在 Windows 宿主机场景下学习，ssh/Xshell/文件互传是伴生需求。
- 【可抄内容源·许可证已核】jaywcjlove/linux-command：MIT 许可（GitHub API 确认 SPDX=MIT, https://github.com/jaywcjlove/linux-command ），36461 stars，580+ 命令的中文文档，每条结构=一句话概述/补充说明(含'打包 vs 压缩'这类概念辨析)/语法/选项逐条中文注释/实例——与 cmdtyper MIT 兼容，可直接改写引入。实测抓样 tar.md 质量良好（ https://raw.githubusercontent.com/jaywcjlove/linux-command/master/command/tar.md ）。
- 【可抄内容源·许可证已核】tldr-pages：内容 CC-BY 4.0（ https://github.com/tldr-pages/tldr/blob/main/LICENSE.md 原文确认），有官方中文翻译(pages.zh)；每条命令约8个'任务式一句话+命令'示例，署名后可改写引入 MIT 项目，天然适合做'给中文任务→默写命令'题干。
- 【不可直接抄的源】Stack Exchange 全部内容 CC BY-SA 4.0（只能抄题目分布和答案结构）；TLCL 书为 CC BY-NC-ND（禁演绎，仅参考编排, https://linuxcommand.org/tlcl.php ）；jlevy/the-art-of-command-line 为 CC BY-SA 4.0（README 确认）；cheat/cheatsheets 仓库未声明许可证（GitHub API license=null），不可用。

**对 cmdtyper 的启示**：

- 新手刚需主题优先级清单（题库扩充顺序，均有高票数据支撑）：①tar/unzip/gzip 参数组合 ②apt 全工作流(update/install/remove/purge/search/list --installed + dpkg -i) ③chmod/chown/sudo(数字+符号两种写法, -R, 为什么别777) ④cp/mv/rm/mkdir -p/ln -s ⑤重定向 > >> 2>&1 | tee ⑥PATH/.bashrc/export/source ⑦find/grep 高频姿势(含排除目录) ⑧df -h/du -sh 磁盘排查链 ⑨ps aux|grep/kill/lsof -i:端口 ⑩ssh/scp/nohup ⑪systemctl/journalctl/crontab ⑫vim 生存四键 ⑬useradd/usermod -aG/groups ⑭sed s///g + head/tail -f/wc -l ⑮中文特供：换源三步(备份 sources.list→编辑→apt update)、locale/LANG 乱码排查、unzip -O GBK。现有 273 题应对照此清单查缺补漏。
- 内容直接来源：把 jaywcjlove/linux-command(MIT, 中文) 作为讲解文本底料——其'概述/补充说明/选项逐条注释/实例'结构几乎能一对一映射到 cmdtyper 的 lesson TOML 与 token_details；tldr-pages 中文版(CC-BY 4.0, 需在关于页署名)的任务式示例直接改写为默写题干。两者许可证均已核实与 MIT 兼容。
- 跟打→默写的中间脚手架设计可直接借用高票答案的讲解解剖结构，做成三级阶梯：Level1 跟打完整命令+逐 flag 中文注释卡（复刻 2039 票 tar 答案的'f 必须在最后'式逐字讲解）；Level2 flag 级完形填空（题干给命令骨架 tar -__f，只默写参数——flag 逐字拆解是社区验证过的记忆钩子）；Level3 给中文任务描述默写全命令（题干写法抄 tldr 的任务式一句话）。这正好填补当前'跟打到默写跨度太大'的空档。
- 新增'报错卡'题型：高票问题里大量标题就是报错原文(unable to lock /var/lib/dpkg、sudo: unable to resolve host、find: missing argument to -exec、Argument list too long、Permission denied)。cmdtyper 的预置输出机制天然适合：屏幕显示预置报错→用户默写修复命令。这是教材完全没有、新手最需要的以症状为入口的编排方式。
- 讲解写法规范（从 3 条 1500-3800 票答案提炼）：命令先行（第一行就是完整可打的命令）；每个 flag 单独一行中文解释；每课设一个'常见错误直觉'纠偏点（如 2>1 为什么错、配置写进 .bashrc 还是 .profile、为什么不该 777）；只给 1-2 个高频变体不穷举；注明边界条件。建议写进内容贡献规范。
- 概念讲解卡（少打字/不打字内容）覆盖高票概念混淆区：terminal/shell/tty 区别、sh vs bash、login shell、/usr/bin vs /usr/local/bin、man 页数字、-- 双横线；再加'急救卡'：Ctrl-S 假死按 Ctrl-Q、Ctrl-C/Ctrl-Z/fg、kill -9 无效意味着什么、rm 事故预防(-i、-- 、先 ls 再 rm 的习惯)。适合放进现有'系统架构专题/符号训练'框架。
- 编排原则：按'任务/症状'组织专题而非教材式知识体系（数据表明新手全部按 How-do-I-X 和报错原文提问，没人按章节学）；鸟哥式的磁盘分割/Quota/RAID/内核编译等章节不必进题库。Reddit 型痛点（选发行版/装系统/驱动）不适合打字题，最多做一张导读讲解卡。
- 中文本地化差异化机会：换源、乱码、输入法、Windows 宿主机(WSL/虚拟机/Xshell/文件互传) 是英文教学产品完全不覆盖的刚需，做成'中文用户生存专题'可成为 cmdtyper 独有卖点；其中换源(编辑 sources.list + apt update)和乱码排查(locale/LANG/unzip -O)都是可打字练习的命令序列。

