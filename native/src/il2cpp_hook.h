#pragma once
#include <functional>
struct HookConfig { int http_port; bool hook_http; int max_response_size; char work_dir[256]; };
typedef std::function<void(void*)> OnInitCallback;
void il2cpp_hook_set_on_init(OnInitCallback cb);
void il2cpp_hook_start();
bool il2cpp_hook_init_api(void* h);
void* find_method_addr(const char* asm_name, const char* ns, const char* cls, const char* method, int pc = -1);
