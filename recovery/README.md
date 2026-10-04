# SO 修复构建与来源

此分支直接编译 `hachimi_ura_plugin` 中已检入的源码。候选版本为
`3.28.2-recovery.3`。历史累积生成器仅用于独立目录内的基线对照，不能在
候选编译前覆盖 `src/lib.rs`。旧 53 个 workflow job 在修复分支上禁用；新的
`recovery-candidate.yml` 只上传 CI 产物，不创建 release、不提交或推送源码。

## 固定来源

`baseline-source-lock.json` 锁定 Git 仓库、基线与实际发布工作流提交，以及
43 个生成输入和最终 6 个文件的规范 LF SHA256。`reproduce_baseline.py` 从
Git 对象取这些输入；本地缺少提交时，仅在 `.recovery-cache/pinned-upstream.git`
中抓取指定提交。它不依赖旧调查目录，也不读取当前 main。

生成源码 SHA256 为 `67f1cb0ef8d0319e4c9f7e3ea026d5938b7f7833d14304594111ac414e6e76c9`。
历史发布没有保存解析后的依赖锁；`baseline-Cargo.lock` 是在固定 Rust 工具链下
重新解析并固定的构建依赖。基线构建同时采用当前 NDK、API、16 KB 链接与调试信息
配置，供同环境源码对照使用。它不承诺与历史发布 SO 的字节或依赖完全一致。

## 命令

需要 Python 3.13.7、Rust 1.97.1、cargo-ndk 4.1.2、NDK 28.2.13676358；
Windows 还需已安装 Visual Studio C++ Build Tools。脚本只初始化子进程环境。

```powershell
python scripts/recovery/run_checks.py
python scripts/recovery/reproduce_baseline.py --source-only
python scripts/recovery/build_native.py --baseline --ndk PATH_TO_NDK --cargo-ndk-dir PATH_TO_CARGO_NDK_BIN
python scripts/recovery/build_native.py --ndk PATH_TO_NDK --cargo-ndk-dir PATH_TO_CARGO_NDK_BIN
```

`--cargo-ndk-dir` 在 cargo-ndk 已在 PATH 中时可省略。生成目录和产物目录始终新建，
不会覆盖旧证据。生成源码或构建失败立即失败；候选源码或构建脚本在编译期间改变，
也会拒绝生成成功清单。

检查入口运行来源/ELF测试、生产策略宿主回归、`ramen_observation` Release契约测试、
以及独立Hook注册器并发/回滚测试。原生Hook的真实宿主ABI、安装瞬间回调窗口和回滚字节
还需设备确认；注册器不会将“API返回原地址”等同于“底层补丁恢复成功”。

每组产物包括 stripped `libhachimi_ura.so`、`symbols/arm64-v8a/libhachimi_ura.so`、
`BUILD-MANIFEST.json`、`SHA256SUMS`、`build.log`。两份 SO 必须有相同 ELF Build ID；
符号文件必须带 `.debug_info`，全部 LOAD 段必须满足 16 KB 对齐。
清单绑定实际源码与本地 crate、生成/构建脚本、锁文件、工具版本、ABI/API 和 SO 哈希。

本地构建和宿主回归不代表真机验收。候选需冷启动、不开嗅探/开嗅探、正常文字、
大 JSON、重启游戏续养以及 4 KB/16 KB 环境验证；没有这些证据时不得标为正式稳定版。
