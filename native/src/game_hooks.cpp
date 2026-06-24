#include "game_hooks.h"
#include "il2cpp_hook.h"
#include "il2cpp_api.h"
#include <android/log.h>
#include <sys/time.h>
#include "Dobby/include/dobby.h"
#define LOG_TAG "UmaHook"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
std::mutex g_data_mutex; CapturedHttpData g_latest_http; GameState g_game_state;
static int64_t now_ms(){struct timeval tv;gettimeofday(&tv,nullptr);return(int64_t)tv.tv_sec*1000+tv.tv_usec/1000;}
struct Il2CppArray{void*klass;void*monitor;void*bounds;size_t max_length;};
static uint8_t* arr_data(void*a){return a?(uint8_t*)a+sizeof(void*)*2+sizeof(void*)+sizeof(size_t):nullptr;}
static size_t arr_len(void*a){return a?((Il2CppArray*)a)->max_length:0;}
typedef void*(*Decomp_t)(void*); static Decomp_t orig_Decompress=nullptr;
static void* hook_Decompress(void* data){
    void* r=orig_Decompress(data);
    if(r){uint8_t*d=arr_data(r);size_t n=arr_len(r);
    if(d&&n>0&&n<4*1024*1024){std::lock_guard<std::mutex>lk(g_data_mutex);
    g_latest_http.response_data.assign(d,d+n);g_latest_http.timestamp_ms=now_ms();g_latest_http.has_response=true;
    if(n>0&&(d[0]=='{'||d[0]=='[')){g_game_state.raw_json.assign(d,d+n);g_game_state.valid=true;}
    LOGI("Captured: %zu bytes",n);}}
    return r;
}
typedef void*(*Comp_t)(void*); static Comp_t orig_Compress=nullptr;
static void* hook_Compress(void* data){
    if(data){uint8_t*d=arr_data(data);size_t n=arr_len(data);
    if(d&&n>0&&n<4*1024*1024){std::lock_guard<std::mutex>lk(g_data_mutex);
    g_latest_http.request_data.assign(d,d+n);g_latest_http.has_request=true;}}
    return orig_Compress(data);
}
static bool do_hook(const char*a,const char*ns,const char*c,const char*m,int pc,void*hf,void**of){
    void*addr=find_method_addr(a,ns,c,m,pc);if(!addr)return false;return DobbyHook(addr,hf,of)==0;
}
bool game_hooks_install(const HookConfig& cfg){
    int ok=0;
    if(cfg.hook_http){
    if(do_hook("Gallop","Gallop","HttpHelper","DecompressResponse",1,(void*)hook_Decompress,(void**)&orig_Decompress))ok++;
    if(do_hook("Gallop","Gallop","HttpHelper","CompressRequest",1,(void*)hook_Compress,(void **)&orig_Compress))ok++;}
    LOGI("Hooks: %d installed",ok);return ok>0;
}
