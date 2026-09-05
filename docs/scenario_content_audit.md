# 场景实训内容审计

审计日期：2026-09-06。目录 `data/scenarios` 含 20 个案例、245 个逐条提交步骤、41 个诊断判断点。步骤数量不是独立命令数量；相同命令通过规范命令 ID 共用。每例至少八步、两个判断点，含背景、目标、环境、步骤指导、模拟输出、证据解释、修复验证和复盘。

| 案例 | 步骤 | 核心证据与能力 |
| --- | ---: | --- |
| 中断恢复后排查 | 11 | boot 时间、当前载波、历史 Link Down/Up、回程、资源、业务健康 |
| SSH 连接被拒绝 | 9 | TCP 阶段、回环监听、sshd 有效配置、带外入口与重载验证 |
| DNS 记录错误 | 10 | NSS、指定解析器、Host 保留、区域校验和重载、客户端缓存 |
| 端口冲突 | 9 | errno 98、端口 PID、进程完整参数、所属服务与健康检查 |
| 防火墙超时 | 9 | 服务监听、SYN 到达、UFW 阻断日志、限定来源例外 |
| MTU 与大包故障 | 9 | 大小包对照、IPv4 开销、隧道路由、路径 MTU、持久化 |
| TLS 证书过期 | 10 | 系统时间、SNI 叶子证书、续期演练、实际续期与重载 |
| HTTP 502/504 | 10 | 连接拒绝与读取超时的阶段差异、代理上游和实际监听 |
| TCP 抓包定位 | 9 | 两端同窗口 SYN/SYN-ACK、回程选路、精确路由修复 |
| Docker 网络 | 10 | 容器内部解析、网络归属、服务别名、命名卷保留 |
| 网站从零部署 | 49 | 软件源、镜像、FastAPI、PostgreSQL、Compose、Git、HTTPS、续期定时器 |
| Git 发布回滚 | 15 | 源码与镜像版本关联、健康验收、旧镜像回滚、期望版本 |
| 后端日志排查 | 10 | 请求 ID、连接耗尽、pg_stat_activity、事务年龄、任务归属 |
| PostgreSQL 事务修改 | 13 | 精确筛选、行锁、RETURNING、回滚演练、提交后验证 |
| 数据库备份恢复 | 10 | custom 归档、无 TTY 导出、校验、隔离恢复与关键数据 |
| 磁盘 inode | 10 | 字节容量与 inode、目录文件量、过期策略、定时清理 |
| CPU/内存/OOM | 10 | 负载与核数、available、RSS、历史 OOM、进程管理器 |
| systemd 连续失败 | 11 | 首次错误、EnvironmentFile、权限、启动限流与配置加载 |
| cron 与权限 | 10 | 实际任务身份、调度记录、路径权限、绝对路径、真实身份验收 |
| 文件恢复 | 11 | 保全现场、校验清单、恢复预览、暂存恢复、切换与内容比较 |

## 官方资料与核验点

以下官方文档、项目手册或项目维护者源码用于核对语义，不将模拟输出宣称为真实事故记录。每个案例的 `sources` 字段也保留相关链接。

- 网络 socket 与抓包：[ss 手册](https://man7.org/linux/man-pages/man8/ss.8.html)、[tcpdump 手册](https://man7.org/linux/man-pages/man8/tcpdump.8.html)。核验监听过滤、PID、接口、包数、pcap 读写和 TCP 标志解释。
- SSH：[OpenSSH sshd_config](https://man.openbsd.org/sshd_config)。核验 ListenAddress 与 Include；Ubuntu ssh.service/ssh.socket 差异在课程中明确限定。
- DNS：[BIND 9 工具手册](https://bind9.readthedocs.io/en/latest/manpages.html)。核验 dig、区域文件检查和 rndc reload；解析器缓存与权威记录分开处理。
- 防火墙：[Ubuntu Server UFW 文档](https://ubuntu.com/server/docs/how-to/security/firewalls/)。案例明确使用原生主机服务，Docker 发布端口另按其转发路径分析。
- Docker：[Ubuntu 安装](https://docs.docker.com/engine/install/ubuntu/)、[Compose 启动顺序](https://docs.docker.com/compose/how-tos/startup-order/)、[Compose 生产运行](https://docs.docker.com/compose/how-tos/production/)。核验官方签名软件源、Compose 插件、健康依赖、目标服务发布和持久数据。
- 网站：[FastAPI 容器部署](https://fastapi.tiangolo.com/deployment/docker/)、[Nginx proxy 模块](https://nginx.org/en/docs/http/ngx_http_proxy_module.html)。应用 Dockerfile 使用 exec 形式启动；区分 TCP connect 失败与上游读取超时。
- TLS：[OpenSSL s_client](https://docs.openssl.org/master/man1/openssl-s_client/)、[Certbot 使用指南](https://eff-certbot.readthedocs.io/en/stable/using.html)。核验 SNI、webroot、自定义配置目录、dry-run、renew 和成功后 deploy hook。
- Git：[git tag](https://git-scm.com/docs/git-tag)、[git diff](https://git-scm.com/docs/git-diff)。版本关联及回滚采用本地标签和保留镜像；课程不推送外部仓库。
- PostgreSQL：[事务](https://www.postgresql.org/docs/current/tutorial-transactions.html)、[pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html)、[pg_restore](https://www.postgresql.org/docs/current/app-pgrestore.html)。核验事务边界、custom 归档、隔离库恢复、归档目录和错误即退出。运行环境固定 PostgreSQL 17，所用操作在该版本支持。
- systemd：[journalctl](https://www.freedesktop.org/software/systemd/man/255/journalctl.html)、[EnvironmentFile 等执行配置源码](https://github.com/systemd/systemd/blob/v255/man/systemd.exec.xml)、[资源限制源码](https://github.com/systemd/systemd/blob/v255/man/systemd.resource-control.xml)、[timer 源码](https://github.com/systemd/systemd/blob/v255/man/systemd.timer.xml)。部分 freedesktop HTML 页面返回 403，改用同项目 v255 文档源码核对。
- 文件与任务：[df](https://www.gnu.org/s/coreutils/manual/html_node/df-invocation.html)、[find 删除](https://www.gnu.org/software/findutils/manual/html_node/find_html/Delete-Files.html)、[SHA2](https://www.gnu.org/s/coreutils/manual/html_node/sha2-utilities.html)、[crontab](https://man7.org/linux/man-pages/man5/crontab.5.html)、[rsync](https://download.samba.org/pub/rsync/rsync.1)、[Python JSON](https://docs.python.org/3/library/json.html)。核验 inode、预览再删除、校验清单、系统 crontab 用户列、相对目录和 JSON 校验。

## 环境与一致性

- 默认使用 Ubuntu 24.04 LTS；主机 web-01、ops-01 和文档保留地址有明确含义。每例独立还原初始状态，不把案例 A 的修改偷偷带入案例 B。
- 网站系列项目路径固定 `/srv/ops-site`，服务为 `nginx`、`api`、`db`，数据库 `app`、表 `orders`、目标订单 ID `42`。稳定 HTTPS 基线明确标记为 Git 与镜像 `v1.0.1`。
- PostgreSQL 17 的数据路径为 `/var/lib/postgresql/data`。口令通过单独 secret 文件提供；初始化脚本仅在空卷第一次运行；备份、证书和口令均排除出 Git 和构建上下文。
- `example.test` 与模拟 CA 仅用于课程。实际公共证书签发需要用户拥有且可验证的真实域名，教学不会对保留域名发起签发请求。
- `nano` 步骤展示并模拟保存完整配置文件，讲清实际编辑器保存方式。应用不会打开编辑器或运行命令。SQL 步骤显示 `app=#`，在退出 `\q` 后回到 shell。
- 诊断错误分支展示具体原因并返回当前证据；正确分支进入下一步，不能略过未回答的判断。阅读和判断本身不写入跟打记录。

## 验证

`cargo test --test scenario_training` 包含八项测试，验证所有案例的最低内容数量与来源、错误/正确分支及到达复盘、独立持久进度与重做、规范引用加载及冲突、未完成回车、输出续练防重复记录、首字符错误退出统计、微小窗口与灰字渲染。

新增真实 App 按键端到端测试逐一完成全部 20 个案例的 245 条命令，在 41 个判断点先选错误答案返回证据，再选正确答案。它同时验证命令 ID、稳定记录 ID、无重复历史及复盘可达。

未运行这些运维命令，不把静态教学代码当作真实网站部署验收。仓库整体的解析、ID、token、一致性检查和发布构建由整体验收统一执行。
