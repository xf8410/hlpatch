# v3.28.7 — 取证校准版（严格模块标签 + 线程实名 + 跨线程保护）

基线沿革：v3.28.6 同款链（v3.28.0 + cumulative 3.27.23 + SIGSEGV 防护 + 崩溃日志修复 + 摘 AI + 动态版本 + meta 直查 + crash-truth + pc/lr 取证）。
**本版只校准取证仪器 + 修一个跨线程 longjmp 隐患，不改任何数据行为。**

## v3.28.6 现场暴露的三个仪器缺陷

v3.28.6 装机后拿到新样本：

```
[117] S:exit
[118] S:enter
CRASH at step 118 sig=11 addr=0x1f022058000109 tid=486774850816 pc=native+1511833d0 lr=native+1511834b0 thr=init FATAL
```

1. **"native" 标签是合并出来的假货**：`libnative.so`（游戏库，base 0x7xx…）与 `libnativewindow.so`（系统图形库，base 0x1xxx…）被前缀规则并成一条，`+0x1511833d0`（≈5.6GB）这种偏移无法换算到函数——标签必须各归各行；
2. **`thr=init` 不可全信**：线程表无"最新注册优先"判定，pthread_t 回收后老标签会被新线程顶着出来——需带写入序号 + tid→标签的离线映射（写进日志）；
3. **跨线程 longjmp 隐患**（提前拆弹）：旧逻辑只要全局恢复标志为真就 `siglongjmp`，哪怕崩溃发生在**别的线程**上——会跳到别人的栈上执行，属严重 UB。必须"武装线程 == 崩溃线程"才允许回收。

## 本版修正（全部为取证/安全，零数据路径变更）

1. **模块表严格化**：`native`（libnative.so）/ `natwin`（libnativewindow.so）/ `nathelp`（libnativehelper.so）各自独立成行；同名段只有相距 ≤64MB 才合并；查表改为"最小包含区间优先"（不再被大跨度条目遮蔽）。
2. **线程实名制**：每槽带写入序号，同 tid 取**最新**注册标签；注册时把 `T:init=<tid>`、`T:push=<tid>`、`T:http=<tid>` 直接写进 predict 日志——崩溃行的 `tid=` 可离线对上真身。
3. **armed-tid 双重门**：`hl_arm_recovery()` 记录武装线程；handler 的 longjmp 与 `RECOVERED` 判词都要求"武装线程 == 崩溃线程"（两处门，grep 断言恰好 2）。
4. **新字段**：`fa=0x…`（内核记录的故障地址，与 addr 互相校验）、`c=<n>`（si_code：1=野地址 MAPERR，2=权限/执行错误 ACCERR）。
5. **标记带线程号**：`S:enter tid=… / S:exit tid=…`——/summary 读写线程身份直接印在轨迹里。
6. 模块表容量 128→256 行。

## 下一次崩溃怎么读（v3.28.7 版）

```
CRASH at step N sig=11 addr=0x… tid=… pc=<mod>+0x… lr=<mod>+0x… thr=<init|push|http|?>
      fa=0x… c=<1|2> RECOVERED|FATAL
```

- `c=1`（MAPERR）：野地址访问——坏指针/内存写坏；
- `c=2`（ACCERR）：权限或取指错误——踩 W^X 页/调用空函数指针；
- `pc=native+…` 偏移现在**可换算**（对应 libnative.so 内代码位置）；
- `thr=init` 现在**可信**（用紧邻的 `T:init=<tid>` 行核对）；
- `RECOVERED` 现在只会出现在"武装线程自己崩且被兜住"的情况，其余一律 `FATAL`（如实记录）。

## 保留项与排除项（与前版一致）

保留：全部数据端点、嗅探/发收包落盘、meta 直查控制台、纯数据管道（摘 AI）、动态版本、SIGSEGV 防护、崩溃日志落盘、4MiB 深栈。
不含：画面映射（截屏）、training_anim_skip、签名明文观测。

SHA256 见 SHA256SUMS / BUILD-MANIFEST.txt。
