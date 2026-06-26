#include "http_server.h"
#include "game_hooks.h"
#include <android/log.h>
#include <thread>
#include <sstream>
#include "httplib.h"
#define LOG_TAG "UmaHook"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
static httplib::Server* g_srv=nullptr; static bool g_run=false;
static std::string hex_prev(const std::vector<uint8_t>&d,size_t mx=128){
    std::string r;char b[4];for(size_t i=0;i<std::min(d.size(),mx);i++){snprintf(b,4,"%02X ",d[i]);r+=b;}return r;
}
static void srv_loop(int port){
    httplib::Server s;g_srv=&s;
    s.Get("/status",[&](const httplib::Request&,httplib::Response& r){
        std::lock_guard<std::mutex>lk(g_data_mutex);std::ostringstream o;
        o<<"{\"hook_active\":true,\"resp_size\":"<<g_latest_http.response_data.size()<<",\"ts\":"<<g_latest_http.timestamp_ms<<"}";r.set_content(o.str(),"application/json");});
    s.Get("/game_state",[&](const httplib::Request&,httplib::Response& r){
        std::lock_guard<std::mutex>lk(g_data_mutex);std::ostringstream o;
        o<<"{\"valid\":"<<(g_game_state.valid?"true":"false")<<",\"ts\":"<<g_latest_http.timestamp_ms<<"}";r.set_content(o.str(),"application/json");});
    s.Get("/raw_response",[&](const httplib::Request&,httplib::Response& r){
        std::lock_guard<std::mutex>lk(g_data_mutex);auto&d=g_latest_http.response_data;
        if(d.empty()){r.set_content("{\"error\":\"no data\"}","application/json");return;}
        if(d[0]=='{'||d[0]=='[')r.set_content(std::string(d.begin(),d.end()),"application/json");
        else{std::ostringstream o;o<<"{\"size\":"<<d.size()<<",\"hex\":\""<<hex_prev(d)<<"\"}";r.set_content(o.str(),"application/json");}});
    if(!s.bind_to_port("127.0.0.1",port)){g_run=false;return;}
    g_run=true;s.listen_after_bind();g_run=false;
}
bool http_server_start(int port){if(g_run)return true;new std::thread(srv_loop,port);
for(int i=0;i<50;i++){if(g_run)return true;usleep(100000);}return false;}
void http_server_stop(){if(g_srv){g_srv->stop();g_srv=nullptr;}g_run=false;}
