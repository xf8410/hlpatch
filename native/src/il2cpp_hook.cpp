#include "il2cpp_hook.h"
#include "il2cpp_api.h"
#include <dlfcn.h>
#include <android/log.h>
#include "dobby.h"

#define LOG_TAG "UmaHook"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

Il2CppApi g_il2cpp = {};
static OnInitCallback g_cb = nullptr;
static void* g_handle = nullptr;
static bool g_init = false;

static void* (*orig_dlopen)(const char*, int) = nullptr;
static void (*orig_il2cpp_init)(void*) = nullptr;

// il2cpp_init hook: called after il2cpp is initialized
static void hook_il2cpp_init(void* arg) {
    if (orig_il2cpp_init) orig_il2cpp_init(arg);
    g_init = true;
    LOGI("il2cpp_init done, calling callback");
    if (g_cb) g_cb(g_handle);
}

static void* hook_dlopen(const char* fn, int fl) {
    void* h = orig_dlopen(fn, fl);
    if (fn && strstr(fn, "libil2cpp.so")) {
        LOGI("libil2cpp.so loaded! h=%p", h);
        g_handle = h;
        void* a = dlsym(h, "il2cpp_init");
        if (a) {
            DobbyHook(a, (dobby_dummy_func_t)hook_il2cpp_init, (dobby_dummy_func_t*)&orig_il2cpp_init);
        }
    }
    return h;
}

void il2cpp_hook_set_on_init(OnInitCallback cb) { g_cb = cb; }

void il2cpp_hook_start() {
    DobbyHook((void*)dlopen, (dobby_dummy_func_t)hook_dlopen, (dobby_dummy_func_t*)&orig_dlopen);
}

bool il2cpp_hook_init_api(void* h) {
    if (!h) return false;
    #define R(n) g_il2cpp.n = (decltype(g_il2cpp.n))dlsym(h, #n); if (!g_il2cpp.n) { LOGE("Missing: %s", #n); return false; }
    R(domain_get) R(domain_get_assemblies) R(assembly_get_image) R(image_get_name)
    R(class_from_name) R(method_from_name) R(runtime_class_init)
    R(string_new) R(class_instance_size)
    #undef R
    return true;
}

void* find_method_addr(const char* an, const char* ns, const char* cn, const char* mn, int pc) {
    if (!g_init) return nullptr;
    Il2CppDomain* d = g_il2cpp.domain_get();
    if (!d) return nullptr;
    size_t cnt = 0;
    Il2CppAssembly** aa = g_il2cpp.domain_get_assemblies(d, &cnt);
    Il2CppImage* img = nullptr;
    for (size_t i = 0; i < cnt; i++) {
        Il2CppImage* m = g_il2cpp.assembly_get_image(aa[i]);
        const char* n = g_il2cpp.image_get_name(m);
        if (n && strstr(n, an)) { img = m; break; }
    }
    if (!img) return nullptr;
    Il2CppClass* k = g_il2cpp.class_from_name(img, ns, cn);
    if (!k) return nullptr;
    g_il2cpp.runtime_class_init(k);
    Il2CppMethodInfo* mi = g_il2cpp.method_from_name(k, mn, pc);
    if (!mi) return nullptr;
    LOGI("Found %s.%s::%s = %p", ns, cn, mn, mi->methodPointer);
    return mi->methodPointer;
}
