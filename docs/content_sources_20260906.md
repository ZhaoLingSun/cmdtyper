# 2026-09-06 题库扩充与教学口径核验

本轮独立编写 37 个专题，其中前 24 个专门训练网络诊断；489 条新增专题命令的 ID 保存在 `content_expansion_audit.json`。这些数量不含拆开的旧工作流、旧讲解的规范引用，也不含另行补充的基础短练习。

基础用法分组覆盖 174 个命令讲解 L1 示例、55 个符号条目和 52 个系统章节，共 281 组。每组保留一个教学示例，再引用三道不同的简单题，共 843 个题目引用。同一道命令在不同上下文复用时只定义一次；L1、L3、L5 是同一题目的训练方式，不增加题目数。

独立复核后，137 个题组按具体操作、选项及输入方向重新选择练习，并补充 209 条针对性短题。最终共有 690 条独立练习命令，其中 553 条为本轮新增且实际被练习组使用的短题；全局规范命令总数为 2467。旧命令迁移及未进入练习组的命令仍按各自清单独立核算。最终逐组复核进一步收紧了域名 Ping、PATH、目录方向、权限位、进程选择、归档格式、链接创建与解析、分页与行号、文本变换方向、服务启停、会话列表等核心操作；所有 281 组都有核心能力和三题对应理由。完整可读清单为 `foundation_semantic_review_20260906.md`，机器契约为同名 JSON。

40 个既有分步流程的 216 步都已记录与命令顺序一致的上下文、输出和具体说明。其中 176 步实际静默，使用空字符串表示；文件创建、变量赋值、目录变化和重定向的状态变化写在说明中，不伪装成程序打印的“成功”消息。97 个复用原子命令均有具体用途说明；同一 `printf` 在不同流程中读取本流程自己的归档或密钥路径。交互命令交代预置回答，SSH 流程交代独立核验主机身份及已有公钥的位置。

`data/practice` 中的源标识与用法标识将练习连接到学习中心。示例路径 `~/ops-lab`、实验数据库 `appdb`、PID 4242、网卡 ens33，以及示例地址必须结合题目理解；应用不会建立 SSH 会话、抓取真实报文或修改系统。无可靠固定结果的检查不填造假的成功输出。少量模拟输出只表示具体实验条件，不作为真实外部服务的当前状态。

## 官方来源与本轮核验要点

| 内容 | 官方资料 | 已落实的教学约束 |
| --- | --- | --- |
| 套接字和端口 | [iproute2 ss 手册源](https://github.com/iproute2/iproute2/blob/main/man/man8/ss.8) | 监听、连接状态、IPv4/IPv6 和进程归属分开查看，过滤表达式引用完整参数。 |
| 内核网络参数 | [Linux 网络 sysctl 文档](https://www.kernel.org/doc/Documentation/networking/ip-sysctl.txt) | 查询转发、反向路径过滤、SYN 重试和 MTU 参数；观察值不自动转成调参建议。 |
| DNS | [BIND 9 工具手册](https://bind9.readthedocs.io/en/stable/manpages.html) | 区分记录类型、指定解析器、UDP/TCP 与超时重试；系统 NSS 解析另用 getent。 |
| HTTP 与代理 | [curl 官方手册](https://curl.se/docs/manpage.html) | DNS、TCP、TLS、首字节指标按累计时间解释；分别限制连接和总时长，保留域名验证。 |
| 抓包 | [tcpdump 手册源](https://github.com/the-tcpdump-group/tcpdump/blob/master/tcpdump.1.in) | 采集设置报文数上限，离线读取 pcap；TLS 加密正文不以 HTTP 明文讲解。 |
| 日志 | [systemd journalctl 手册源](https://github.com/systemd/systemd/blob/main/man/journalctl.xml) | 启动、单元、时间范围和格式分别限定；上一启动记录需要 journal 实际留存。 |
| TLS | [OpenSSL s_client](https://docs.openssl.org/3.0/man1/openssl-s_client/) | 保留 SNI，区分握手观察和验证失败返回，不把关闭证书验证当作修复。 |
| Docker | [Docker Compose CLI](https://docs.docker.com/reference/cli/docker/compose/) | 区分 build、up、restart、exec 和容器日志；配置变更不能只靠 restart 应用。 |
| 发布时的代理解析 | [Compose 网络更新](https://docs.docker.com/compose/how-tos/networking/)、[Nginx proxy_pass](https://nginx.org/en/docs/http/ngx_http_proxy_module.html) | API 容器重建可能改变地址；本课程采用静态代理地址解析，因此每次发布和回滚后校验并重载 Nginx，再从 HTTPS 用户入口验证。 |
| Git | [Git revert](https://git-scm.com/docs/git-revert) | 通过新提交反做变更，保留已有公开历史；使用前检查工作区状态。 |
| 数据库 | [PostgreSQL psql](https://www.postgresql.org/docs/current/app-psql.html) | `-c` 中保持事务处于同一连接；更新用明确条件、RETURNING 和 ROLLBACK 演练。 |
| Python | [Python ipaddress](https://docs.python.org/3/library/ipaddress.html) | 使用标准库进行接口网络和地址归属判断，不发出网络探测。 |
| 文件与 Shell | [GNU Coreutils](https://www.gnu.org/software/coreutils/manual/html_node/index.html)、[GNU Bash](https://www.gnu.org/software/bash/manual/bash.html) | 保留引号、重定向顺序和变量展开语义；符号课中的条件执行不机械拆开。 |

网络题把载波、地址、路由、邻居、监听、握手、TLS 和应用响应作为不同证据层。Ping 或中间跳未响应不能单独证明应用不可达；UDP 测试使用真实协议应答；累计错误计数需看时间窗口内增量。MTU 题说明 IPv4 无选项 ICMP 的 28 字节头部，并注明固定 IPv6 报文偏移过滤不覆盖扩展头情形。

## 可重复核验

运行 `python3 scripts/audit_practice_content.py` 检查全局命令和 ID 去重、引用有效性、每组教学与三题不同、全部基础用法覆盖、489 条专题新增清单、24 网络专题及 1098 条新题 Bash 语法。45 项操作语义约束检查排序、重定向与 SCP 方向、版本或帮助、只读编辑、locale、文件边界、精确端口、域名解析、PATH、符号链接和历史展开。此外逐题运行全部 281 个核心能力契约，核对 843 条对应理由的命令 ID 与实际命令一致；正则符号从 grep 实际模式参数中取证，避免只在整条命令中碰巧找到字符。脚本还检查发布/回滚的代理刷新顺序，以及 40 流程 216 步的上下文顺序、静默输出、共享变量结果、重定向错误、tee 追加和 stat 字节数。此脚本调用 `bash -n`，仅解析语法，不运行训练命令。

489 条独立新增命令都不同于原 554 条完整命令。进一步把原命令的任何字面子串也保守排除后，仍有 488 条，超过至少 400 条的要求；被保守排除的一条是旧 `nohup` 流程中出现过的 Python HTTP server 子串。这项检查排除按字面拆分凑数，但语义教学质量仍需逐项人工复核，不能仅由数量或 Bash 语法证明。

Rust 内容检查使用 `cargo test --test parse_all --test tokens_consistency --test id_uniqueness`。模拟教学不能以在当前真实主机上执行故障注入或数据库写入来验证；命令适用的工具、权限和预置实验对象在题目与专题说明中交代。
