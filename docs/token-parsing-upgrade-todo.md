# cmdtyper Token 解析系统改造 TODO（v1）

> 目标：让 **学习模式** 中的每条命令都具备**细致、完整、统一、可复用**的 token 拆解说明；同时建立一套可扩展的通用 token 解析库与审计机制，避免继续出现“命令参数，用于指定要处理的对象或行为细节”这类空泛解释。

## 0. 改造目标与验收线

- [ ] 明确本次改造的产品目标：**学习模式优先保证“逐 token 教学质量”**
- [ ] 明确设计原则：**先可解释，再可复用，最后再自动化**
- [ ] 明确最低验收标准：
  - [ ] 学习模式中，示例命令的**关键 token 不允许漏解释**
  - [ ] 不允许出现空泛模板解释（如“用于指定对象或行为细节”）
  - [ ] 高频/关键 token 必须进入通用解析库
  - [ ] lesson 中缺失 token 细解时，fallback 也要足够具体
- [ ] 明确“关键 token”的定义：
  - [ ] 命令词
  - [ ] 选项
  - [ ] 选项值
  - [ ] 路径
  - [ ] 占位符
  - [ ] 操作符 / 管道 / 重定向
  - [ ] 权限值 / 数字模式
  - [ ] 变量与替换
  - [ ] 模式串 / 正则 / 搜索关键词
- [ ] 明确“完整拆解”的定义：
  - [ ] 说明 token 是什么
  - [ ] 说明它在该命令中的作用
  - [ ] 说明为什么这里需要它
  - [ ] 对组合写法（如 `-lhS`）说明拆分含义

## 1. 全库审计基线整理

### 1.1 命令库总量与现状
- [ ] 固化当前审计基线数据到文档中：
  - [ ] 命令总数：273
  - [ ] 题库 token 总数：1021
  - [ ] 唯一 token 文本：519
  - [ ] lesson `token_details` 总数：597
  - [ ] 无 lesson 细拆命令数：254 / 273
- [ ] 记录当前主要问题：
  - [ ] 大量命令没有 lesson 级 token 细拆
  - [ ] 过度依赖 command-level `tokens.desc`
  - [ ] token 粒度不统一
  - [ ] 同类 token 解释风格不统一
  - [ ] 高频 token 缺少统一语义模板

### 1.2 现有 token 分类盘点
- [ ] 将当前粗分类结果写入审计文档：
  - [ ] `word`
  - [ ] `short_option_or_bundle`
  - [ ] `other`
  - [ ] `operator_or_placeholder`
  - [ ] `path`
  - [ ] `long_option`
  - [ ] `octal_mode`
  - [ ] `variable_or_subst`
- [ ] 针对每类，整理代表样例与风险点
- [ ] 识别“混合 token”问题：
  - [ ] `-la`
  - [ ] `-lhS`
  - [ ] `-czf`
  - [ ] `-xzf`
  - [ ] `-sh`
  - [ ] `-type`
  - [ ] `-name`
- [ ] 识别“语义强但当前归类模糊”的 token：
  - [ ] `f`
  - [ ] `d`
  - [ ] `{}`
  - [ ] `+`
  - [ ] `-exec`
  - [ ] `-F:`
  - [ ] `'{print $1}'`
  - [ ] `'*.log'`

## 2. 数据模型升级设计

### 2.1 Token 结构扩展
- [ ] 设计新的 token 数据结构（兼容旧字段）
- [ ] 为 token 新增 `kind`
- [ ] 为 token 新增可选 `role`
- [ ] 为 token 新增可选 `group`
- [ ] 为 token 新增可选 `applies_to`
- [ ] 为 token 新增可选 `meaning`
- [ ] 为 token 新增可选 `notes`
- [ ] 为 token 新增可选 `examples`
- [ ] 为 token 新增可选 `split_into`（用于 bundle 拆分）
- [ ] 明确旧字段 `desc` 的兼容策略
- [ ] 明确 lesson `token_details` 与 command `tokens` 的字段对齐策略

### 2.2 `kind` 枚举设计
- [ ] 定义 `command`
- [ ] 定义 `subcommand`
- [ ] 定义 `short_option`
- [ ] 定义 `short_option_bundle`
- [ ] 定义 `long_option`
- [ ] 定义 `option_value`
- [ ] 定义 `path`
- [ ] 定义 `filename`
- [ ] 定义 `directory`
- [ ] 定义 `pattern`
- [ ] 定义 `regex`
- [ ] 定义 `literal`
- [ ] 定义 `quoted_string`
- [ ] 定义 `variable`
- [ ] 定义 `substitution`
- [ ] 定义 `operator`
- [ ] 定义 `pipe`
- [ ] 定义 `redirection`
- [ ] 定义 `placeholder`
- [ ] 定义 `permission_mode`
- [ ] 定义 `number`
- [ ] 定义 `service_name`
- [ ] 定义 `url`
- [ ] 定义 `unknown` / `other`

### 2.3 组合 token 拆分策略
- [ ] 设计 `-la` 这种 bundle 的表示法
- [ ] 设计 `-lhS` 的逐项展开表示法
- [ ] 设计 `-czf` / `-xzf` 这类 tar 习惯写法的解释模板
- [ ] 设计 `-F:` 这种带参数短选项的结构化表示
- [ ] 设计 `-i 's/...'` 这种 option + quoted expression 的关联关系
- [ ] 设计 `find -type f` 这种 option + value 的绑定关系
- [ ] 设计 `find -exec ... {} +` 这种跨 token 结构关系

## 3. 通用 Token 解析库（Lexicon）建设

### 3.1 解析库文件组织
- [ ] 设计通用解析库存放位置
- [ ] 决定采用 TOML / JSON / Rust 常量表
- [ ] 设计按领域拆分的文件结构：
  - [ ] `shell_operators`
  - [ ] `paths_and_placeholders`
  - [ ] `common_options`
  - [ ] `find_family`
  - [ ] `grep_family`
  - [ ] `sed_awk_family`
  - [ ] `permissions`
  - [ ] `git_family`
  - [ ] `system_admin`
  - [ ] `network_http`
- [ ] 规定 lexicon 条目的唯一键格式
- [ ] 规定同 token 多语境解释的覆盖机制

### 3.2 第一优先级：Shell 通用操作符
- [ ] `|`
- [ ] `||`
- [ ] `&&`
- [ ] `;`
- [ ] `>`
- [ ] `>>`
- [ ] `<`
- [ ] `<<`
- [ ] `2>`
- [ ] `2>&1`
- [ ] `&`
- [ ] `{}`（占位符）
- [ ] `+`（批量执行终止符）
- [ ] `[]`
- [ ] `()`
- [ ] `$()`

### 3.3 第一优先级：路径与位置 token
- [ ] `.`
- [ ] `..`
- [ ] `~`
- [ ] `/etc`
- [ ] `/var/log`
- [ ] `/var/www`
- [ ] `/srv/www/`
- [ ] `/etc/passwd`
- [ ] `/backup/www/`
- [ ] `/usr/local/share/...`
- [ ] `./src`
- [ ] `../`
- [ ] `src/main.rs`
- [ ] `README.md`
- [ ] `app.log`
- [ ] `access.log`
- [ ] `notes.txt`

### 3.4 第一优先级：高频命令词
- [ ] `find`
- [ ] `grep`
- [ ] `sed`
- [ ] `awk`
- [ ] `cut`
- [ ] `sort`
- [ ] `uniq`
- [ ] `xargs`
- [ ] `tar`
- [ ] `chmod`
- [ ] `chown`
- [ ] `systemctl`
- [ ] `journalctl`
- [ ] `docker`
- [ ] `git`
- [ ] `curl`
- [ ] `ps`
- [ ] `du`
- [ ] `df`
- [ ] `ls`
- [ ] `cat`
- [ ] `head`
- [ ] `tail`
- [ ] `echo`
- [ ] `cd`
- [ ] `pwd`
- [ ] `mkdir`
- [ ] `cp`
- [ ] `mv`
- [ ] `rm`
- [ ] `sudo`

### 3.5 第一优先级：高频选项
- [ ] `-n`
- [ ] `-r`
- [ ] `-R`
- [ ] `-f`
- [ ] `-h`
- [ ] `-c`
- [ ] `-i`
- [ ] `-u`
- [ ] `-p`
- [ ] `-s`
- [ ] `-l`
- [ ] `-a`
- [ ] `-S`
- [ ] `-t`
- [ ] `-d`
- [ ] `-F:`
- [ ] `-type`
- [ ] `-name`
- [ ] `-exec`
- [ ] `-x`
- [ ] `-e`
- [ ] `-v`
- [ ] `--since`
- [ ] `--delete`
- [ ] `--oneline`
- [ ] `--format`
- [ ] `--graph`
- [ ] `--no-edit`
- [ ] `--amend`
- [ ] `--staged`

### 3.6 第一优先级：特殊值与语义值
- [ ] `f`（普通文件）
- [ ] `d`（目录）
- [ ] `l`（符号链接）
- [ ] `644`
- [ ] `755`
- [ ] `600`
- [ ] `700`
- [ ] `$HOME`
- [ ] `$SHELL`
- [ ] `ERROR`
- [ ] `TODO`
- [ ] `main`
- [ ] `net`
- [ ] `'*.log'`
- [ ] `'[n]ginx'`
- [ ] `'{print $1}'`
- [ ] `'s/foo/bar/'`
- [ ] `'s/localhost/127.0.0.1/'`

## 4. Lesson 级“全 token 拆解”补全计划

### 4.1 先定质量规范
- [ ] 制定 `token_details` 编写规范
- [ ] 规定每个 example 必须覆盖的 token 范围
- [ ] 规定禁止使用的空泛描述模板
- [ ] 规定 bundle 选项必须展开解释
- [ ] 规定操作符、路径、值、占位符必须单独说明
- [ ] 规定含管道的命令必须解释数据流方向
- [ ] 规定含 `find -exec` / `xargs` / `sed` / `awk` 的命令必须做语义级讲解

### 4.2 优先命令家族（第一批）
- [ ] `find` 家族
- [ ] `grep` 家族
- [ ] `chmod` / `chown` 家族
- [ ] `tar` 家族
- [ ] `sed` 家族
- [ ] `awk` 家族
- [ ] `cut` / `sort` / `uniq` / `xargs` 家族
- [ ] `systemctl` / `journalctl` 家族
- [ ] `git` 家族
- [ ] `curl` 家族
- [ ] `docker` 家族

### 4.3 第二批命令家族
- [ ] `ls`
- [ ] `cp`
- [ ] `mv`
- [ ] `rm`
- [ ] `mkdir`
- [ ] `cat`
- [ ] `head`
- [ ] `tail`
- [ ] `ps`
- [ ] `du`
- [ ] `df`
- [ ] `echo`
- [ ] `cd`
- [ ] `pwd`

### 4.4 对每个 example 的补全要求
- [ ] 补齐命令词说明
- [ ] 补齐全部关键选项说明
- [ ] 补齐每个选项值说明
- [ ] 补齐路径/目标对象说明
- [ ] 补齐操作符说明
- [ ] 补齐占位符说明
- [ ] 补齐数字模式说明
- [ ] 补齐变量/环境值说明
- [ ] 对复杂表达式增加语境解释
- [ ] 对组合命令补充“执行顺序/数据流”解释

## 5. 低质量描述清理专项

### 5.1 禁止的描述模板
- [ ] 搜索并清理“命令参数，用于指定要处理的对象或行为细节”
- [ ] 搜索并清理“指定要处理的对象或行为细节”
- [ ] 搜索并清理“目标对象”
- [ ] 搜索并清理“某种选项”
- [ ] 搜索并清理“用于控制输出”
- [ ] 搜索并清理其他模板化空话

### 5.2 替换规则
- [ ] 路径类 token 改为“目标目录/目标文件 + 用途”
- [ ] 选项类 token 改为“具体行为变化”
- [ ] 选项值类 token 改为“给前一个选项提供什么值”
- [ ] 占位符类 token 改为“在该命令结构中被替换成什么”
- [ ] 操作符类 token 改为“如何传递/组合命令语义”
- [ ] 数值类 token 改为“这个数字在该命令中的具体含义”

## 6. 审计与生成工具

### 6.1 审计脚本
- [ ] 编写脚本：统计全库 token 频率
- [ ] 编写脚本：统计 lesson `token_details` 覆盖率
- [ ] 编写脚本：识别 command 与 lesson 的覆盖差距
- [ ] 编写脚本：检测低质量描述模板
- [ ] 编写脚本：识别高频但未纳入 lexicon 的 token
- [ ] 编写脚本：识别 bundle 选项未拆解的案例
- [ ] 编写脚本：识别操作符未解释的案例
- [ ] 编写脚本：识别 `find -exec` / `sed` / `awk` 等复杂命令的解释缺失

### 6.2 质量报告输出
- [ ] 输出 token 覆盖率报告
- [ ] 输出高频 token 缺口报告
- [ ] 输出空泛描述报告
- [ ] 输出按命令家族的整改优先级报告
- [ ] 输出“已修 / 未修”进度报表

## 7. UI 与展示层改造（可选但推荐）

### 7.1 token 展示增强
- [ ] 根据 `kind` 给不同 token 上色
- [ ] 命令词、选项、路径、操作符使用不同视觉样式
- [ ] 对 bundle token 提供展开视图
- [ ] 对 `find -exec ... {} +` 这类结构提供分段展示
- [ ] 对 lesson token_details 与 lexicon fallback 做视觉区分
- [ ] 在 UI 中标记“通用解释” vs “该命令专属解释”

### 7.2 可读性增强
- [ ] 长命令多行对齐显示
- [ ] token explanation 支持折叠/展开
- [ ] 增加“为什么这样写”提示区域
- [ ] 增加“执行顺序”提示区域（针对管道/组合命令）
- [ ] 增加“常见误区”快捷跳转

## 8. 测试与验证

### 8.1 数据层测试
- [ ] 测试新 token 结构可反序列化
- [ ] 测试旧 token 结构兼容
- [ ] 测试 lexicon 条目可加载
- [ ] 测试 lesson token_details 优先级正确
- [ ] 测试 fallback 链路：
  - [ ] lesson token_details
  - [ ] lexicon
  - [ ] command tokens

### 8.2 内容质量测试
- [ ] 检查每条高优先级命令都有关键 token 解释
- [ ] 检查高频 token 都已进入 lexicon
- [ ] 检查禁用模板描述不再出现
- [ ] 检查 bundle 选项解释完整
- [ ] 检查操作符和占位符解释完整

### 8.3 人工验收
- [ ] 人工抽检 `find` 家族
- [ ] 人工抽检 `grep` 家族
- [ ] 人工抽检 `chmod` 家族
- [ ] 人工抽检 `tar` 家族
- [ ] 人工抽检 `sed/awk` 家族
- [ ] 人工抽检 `git` 家族
- [ ] 人工抽检 `systemctl/journalctl` 家族
- [ ] 确认学习模式中不再出现明显空泛解释

## 9. 优先级排序（建议施工顺序）

### P0：必须先做
- [ ] 定义 token kind / role 模型
- [ ] 建立第一版通用 token 解析库
- [ ] 清理空泛描述模板
- [ ] 补 `find` / `grep` / `chmod` / `tar` / `sed` / `awk` 的 lesson 细拆
- [ ] 增加覆盖率审计脚本

### P1：高收益
- [ ] 补 `systemctl` / `journalctl` / `git` / `curl` / `docker`
- [ ] 给 UI 增加 token kind 的展示区分
- [ ] 为 bundle token 增加展开解释
- [ ] 输出进度报告

### P2：增强项
- [ ] 自动建议 token 解释草稿
- [ ] 更智能的 token 语法拆分
- [ ] 半自动生成 lesson token_details
- [ ] 复杂命令的语法树可视化

## 10. 建议第一批具体施工清单

### 10.1 先做基础设施
- [ ] 新增 token kind 模型
- [ ] 新增 lexicon 文件结构
- [ ] 新增 fallback 解析链路
- [ ] 新增审计脚本雏形
- [ ] 新增低质量描述检测规则

### 10.2 再做高优先级内容
- [ ] `find /var/www -type f -exec chmod 644 {} +`
- [ ] `find . -name '*.log'`
- [ ] `find /var/log -type f`
- [ ] `grep -R 'TODO' src`
- [ ] `grep -n 'main' src/main.rs`
- [ ] `tar -czf backup.tar.gz project`
- [ ] `tar -xzf backup.tar.gz`
- [ ] `chmod 644 notes.txt`
- [ ] `chmod +x deploy.sh`
- [ ] `chmod -R 755 scripts`
- [ ] `awk -F: '{print $1}' /etc/passwd`
- [ ] `sed -i 's/localhost/127.0.0.1/' .env`
- [ ] `cat ids.txt | xargs -n 1 echo`

### 10.3 然后扩散到通用命令
- [ ] `ls` 家族
- [ ] `cp/mv/rm` 家族
- [ ] `head/tail/cat` 家族
- [ ] `du/df/ps` 家族
- [ ] `curl/systemctl/journalctl` 家族

## 11. 最终交付标准
- [ ] 学习模式中的高优先级命令全部具备细粒度拆解
- [ ] 高频 token 已纳入解析库
- [ ] lesson 与 command 的解释风格统一
- [ ] 审计脚本可持续发现解释缺口
- [ ] 仓库内不再存在明显空泛 token 描述
- [ ] 有一份持续维护的 token 改造基线文档
