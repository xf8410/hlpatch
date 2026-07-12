# 反作弊与插件注入分析

**分析日期**: 2026-07-12
**分析对象**: libil2cpp.so + libnative.so + libunity.so + APK 结构

---

## 一、反作弊系统

### 1.1 _Cyan.dll
- 出现在 IL2CPP DLL 列表中: `umamusume.dll _Cyan.dll`
- **不在 APK 静态文件中** — 运行时由 IL2CPP 加载
- 用途不明，可能是 Cygames 自研反作弊模块
- 搜索 `_Cyan` 在 SO 中只有 DLL 名称字符串，无方法名

### 1.2 Google.Play.Integrity
- DLL: `Google.Play.Integrity.dll`
- Google 官方反作弊 API — 检查设备完整性、应用真实性
- 可能用于：检测 root/模拟器/篡改/重打包

### 1.3 DebugHook.dll
- 调试钩子 DLL
- 可能用于开发环境调试，生产环境应禁用

### 1.4 libnative.so 安全功能
| 功能 | 实现方式 |
|------|---------|
| SQLite 加密 | sqlite3_key / sqlite3_rekey (SQLCipher) |
| TLS 通信 | mbedTLS (AES/RSA/SHA/HMAC) |
| 证书验证 | CA cert path + cert pinning |
| 完整性检查 | integrity_check (SQLite 内置) |
| 签名验证 | id-kp-codeSigning / RSASSA-PS[SYSTEM_NOTE: Content compressed. Read the full version if help_

### 2.4 关键地址
- IL2CPP API 导出表: .dynsym at file offset 0x330
- 导出函数地址前缀: 0x3ff0xxxx / 0x3ff1xxxx
- JNI_OnLoad: libil2cpp.so 入口

---

## 三、对插件注入有用的发现

### 3.1 IL2CPP API 完整导出 (241个函数)

插件可以通过 dlsym 获取这些函数:

#### 类操作 (58个)
```c
il2cpp_class_from_name(image, namespace, name)  // 按名查类
il2cpp_class_get_fields(klass, &iter)          // 遍历字段
il2cpp_class_get_methods(klass, &iter)         // 遍历方法
il2cpp_class_get_field_from_name(klass, name)  // 按名查字段
il2cpp_class_get_method_from_name(klass, name, argc)  // 按名查方法
il2cpp_class_get_image(klass)                  // 获取所属image
il2cpp_class_get_declaring_type(klass)         // 获取声明类型
il2cpp_class_get_element_class(klass)          // 获取元素类型
il2cpp_class_get_flags(klass)                  // 获取标志
il2cpp_class_get_name(klass)                   // 获取类名
il2cpp_class_get_namespace(klass)              // 获取命名空间
il2cpp_class_get_parent(klass)                 // 获取父类
il2cpp_class_get_size(klass)                   // 获取大小
il2cpp_class_get_type(klass)                   // 获取类型
il2cpp_class_is_enum(klass)                    // 是否枚举
il2cpp_class_is_interface(klass)               // 是否接口
il2cpp_class_is_abstract(klass)                // 是否抽象
il2cpp_class_is_blittable(klass)               // 是否blittable
il2cpp_class_is_array(klass)                   // 是否数组
il2cpp_class_for_each(klass, callback, arg)    // 遍历所有类
```

#### 方法操作 (19个)
```c
il2cpp_method_get_name(method)                 // 方法名
il2cpp_method_get_declaring_type(method)       // 声明类型
il2cpp_method_get_return_type(method)          // 返回类型
il2cpp_method_get_param_count(method)          // 参数数量
il2cpp_method_get_param(method, index)         // 获取参数
il2cpp_method_get_param_name(method, index)    // 参数名
il2cpp_method_get_flags(method, *iflags)       // 方法标志
il2cpp_method_get_token(method)                // 方法token
il2cpp_method_is_generic(method)               // 是否泛型
il2cpp_method_is_instance(method)              // 是否实例方法
il2cpp_method_has_attribute(method, attr)      // 是否有特性
il2cpp_object_get_virtual_method(obj, method)  // 获取虚方法
il2cpp_property_get_get_method(prop)           // 属性的get方法
il2cpp_property_get_set_method(prop)           // 属性的set方法
```

#### 字段操作 (15个)
```c
il2cpp_field_get_name(field)                   // 字段名
il2cpp_field_get_offset(field)                 // ★ 字段偏移量
il2cpp_field_get_type(field)                   // 字段类型
il2cpp_field_get_value(obj, field, *value)     // 读取字段值
il2cpp_field_set_value(obj, field, *value)     // 设置字段值
il2cpp_field_get_value_object(obj, field)      // 读取对象字段
il2cpp_field_set_value_object(obj, field, value)  // 设置对象字段
il2cpp_field_static_get_value(klass, field, *value)  // ★ 读静态字段
il2cpp_field_static_set_value(klass, field, *value)  // ★ 写静态字段
il2cpp_field_get_flags(field)                  // 字段标志
il2cpp_field_is_literal(field)                 // 是否常量
il2cpp_field_get_parent(field)                 // 父类型
```

#### 运行时调用 (5个)
```c
il2cpp_runtime_invoke(method, obj, params, *exc)  // ★ 调用任意方法
il2cpp_runtime_invoke_convert_args(method, obj, params, ...)  // 带类型转换调用
il2cpp_runtime_object_init(obj)                   // 初始化对象
il2cpp_runtime_object_init_exception(obj, *exc)   // 带异常初始化
il2cpp_runtime_unhandled_exception_policy_set(callback)  // 异常策略
```

#### 内存操作
```c
il2cpp_alloc(size)                    // 分配内存
il2cpp_free(ptr)                      // 释放内存
il2cpp_array_new(klass, size)         // 创建数组
il2cpp_array_new_specific(klass, size)  // 创建特定数组
il2cpp_object_new(klass)              // 创建对象
il2cpp_value_box(klass, data)         // 装箱
```

#### 线程操作 (15个)
```c
il2cpp_thread_attach(domain)          // ★ 附加到IL2CPP线程
il2cpp_thread_detach(thread)          // 分离线程
il2cpp_thread_current()               // 当前线程
il2cpp_is_vm_thread(thread)           // 是否VM线程
il2cpp_thread_get_all_attached_threads(**begin, **end)  // 所有附加线程
```

#### Image/Assembly (8个)
```c
il2cpp_domain_get()                   // ★ 获取默认domain
il2cpp_domain_get_assemblies(domain, **size)  // ★ 获取所有程序集
il2cpp_assembly_get_image(assembly)   // 获取image
il2cpp_image_get_name(image)          // image名
il2cpp_image_get_filename(image)      // image文件名
il2cpp_image_get_class(image, index)  // 按索引获取类
il2cpp_image_get_class_count(image)   // 类数量
il2cpp_image_get_entry_point(image)   // 入口点
```

### 3.2 字符串操作
```c
il2cpp_string_new(utf8)               // 创建字符串
il2cpp_string_new_len(utf8, len)      // 指定长度创建
il2cpp_string_new_wrapper(utf8)       // 包装创建
il2cpp_string_new_utf16(utf16, len)   // UTF16创建
il2cpp_string_chars(str)              // 获取字符数组
il2cpp_string_length(str)             // 获取长度
il2cpp_string_intern(str)             // 内联字符串
il2cpp_string_is_interned(str)        // 检查是否已内联
```

### 3.3 内存快照
```c
il2cpp_capture_memory_snapshot()      // ★ 捕获内存快照
il2cpp_free_memory_snapshot(snapshot)  // 释放快照
```

---

## 四、插件注入可行路径

### 4.1 Hachimi 模式（当前使用）
- Hachimi 通过 dlopen 拦截加载自己的 .so
- Hachimi 提供 Plugin API vtable
- URA Plugin 通过 Hachimi API 获取 IL2CPP 函数指针

### 4.2 直接 dlopen + dlsym 模式
```c
// 在 .so 的 JNI_OnLoad 中:
void* libil2cpp = dlopen("libil2cpp.so", RTLD_NOW);
auto il2cpp_class_from_name = dlsym(libil2cpp, "il2cpp_class_from_name");
auto il2cpp_class_get_methods = dlsym(libil2cpp, "il2cpp_class_get_methods");
auto il2cpp_runtime_invoke = dlsym(libil2cpp, "il2cpp_runtime_invoke");
// ...
```

### 4.3 内存读取模式（当前 URA Plugin 使用）
- 不需要调用 IL2CPP API
- 直接通过偏移量读内存: read_ptr_at, read_obscured_int_at
- 零 invoke — 避免 SIGSEGV

### 4.4 混合模式
- 内存读取用于热路径 (/summary)
- IL2CPP API 用于初始化和低频操作
- icall 注册用于添加自定义端点

---

## 五、反作弊绕过分析

### 5.1 检测点
| 检测 | 位置 | 绕过方式 |
|------|------|---------|
| Root检测 | Java层 + _Cyan.dll | Magisk DenyList / Zygisk |
| Frida检测 | _Cyan.dll(推测) | frida-server 改名 / 使用 stalker |
| Hook检测 | _Cyan.dll(推测) | 不 hook 游戏函数，只读取内存 |
| 篡改检测 | APK签名验证 | 不修改APK，通过Zygisk注入 |
| 模拟器检测 | Java层 | 不使用模拟器 |
| Play Integrity | Google API | 设备通过 Play 认证 |

### 5.2 当前方案的安全性
Hachimi + URA Plugin 的注入方式:
1. ✅ 不修改 APK — 通过 Zygisk 注入
2. ✅ 不 hook 游戏函数 — 只读内存
3. ✅ 不调用 il2cpp_runtime_invoke 在热路径 — 避免 SIGSEGV
4. ⚠️ Hachimi 本身 hook 了 dlopen — 可能被检测
5. ⚠️ URA Plugin 运行 HTTP 服务器 — 本地端口可能被扫描

### 5.3 风险点
1. **_Cyan.dll** — 未知功能，可能是反作弊
2. **Google.Play.Integrity** — 可能在启动时检查
3. **libnative.so 的证书验证** — 可能 pin 服务器证书
4. **/proc/self/maps 读取** — libil2cpp.so 有此字符串，可能扫描加载的库

---

## 六、libunity.so 关键发现

- 18MB, stripped, ARM64
- 35,093 个字符串
- `Failed to load native plugin: Unable to load library '%s'` — Unity 原生插件加载
- `ptrace` 字符串存在 — 可能有反调试
- `wait-for-managed-debugger` — 调试器等待机制
- FMOD 音频引擎集成
- Unity 渲染管线 (Vulkan/OpenGL)

*分析: 2026-07-12*
