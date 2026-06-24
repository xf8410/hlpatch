#pragma once
#include "il2cpp_hook.h"
#include <cstdint>
#include <vector>
#include <mutex>
struct CapturedHttpData { std::vector<uint8_t> request_data; std::vector<uint8_t> response_data; int64_t timestamp_ms=0; bool has_request=false; bool has_response=false; };
struct GameState { std::string raw_json; int turn=-1; int energy=-1; bool valid=false; };
extern std::mutex g_data_mutex; extern CapturedHttpData g_latest_http; extern GameState g_game_state;
bool game_hooks_install(const HookConfig& config);
