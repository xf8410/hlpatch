#include <dlfcn.h>
#include <android/log.h>
#include <jni.h>
#include <unistd.h>
#include "il2cpp_hook.h"
#include "game_hooks.h"
#include "http_server.h"
#define LOG_TAG "UmaHook"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
static jint (*g_orig_JNI_OnLoad)(JavaVM*, void*) = nullptr;
static HookConfig g_config;
static HookConfig load_config() {
    HookConfig cfg; cfg.http_port = 18765; cfg.hook_http = true; cfg.max_response_size = 1024*1024;
    strncpy(cfg.work_dir, "/sdcard/Android/media/jp.co.cygames.umamusume", sizeof(cfg.work_dir)-1);
    return cfg;
}
JNIEXPORT jint JNI_OnLoad(JavaVM* vm, void* reserved) {
    LOGI("UmaHook v0.1 loaded");
    g_config = load_config();
    il2cpp_hook_set_on_init([](void* h) {
        LOGI("IL2CPP init!");
        if(!il2cpp_hook_init_api(h)){LOGE("API init fail");return;}
        if(!game_hooks_install(g_config)){LOGE("Hook install fail");return;}
        if(!http_server_start(g_config.http_port)){LOGE("Server fail");return;}
        LOGI("All ready on port %d", g_config.http_port);
    });
    il2cpp_hook_start();
    void* orig = dlopen("libmain_orig.so", RTLD_NOW);
    if(!orig){LOGE("Cannot load libmain_orig.so: %s", dlerror());return JNI_ERR;}
    g_orig_JNI_OnLoad = (jint(*)(JavaVM*,void*))dlsym(orig, "JNI_OnLoad");
    if(!g_orig_JNI_OnLoad){LOGE("No orig JNI_OnLoad");return JNI_ERR;}
    return g_orig_JNI_OnLoad(vm, reserved);
}
