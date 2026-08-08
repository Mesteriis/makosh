#include "bridge.h"

#include <algorithm>
#include <array>
#include <chrono>
#include <condition_variable>
#include <cstring>
#include <deque>
#include <iomanip>
#include <memory>
#include <mutex>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

#include "submodules/TgVoipWebrtc/tgcalls/tgcalls/Instance.h"
#include "submodules/TgVoipWebrtc/tgcalls/tgcalls/InstanceImpl.h"
#include "submodules/TgVoipWebrtc/tgcalls/tgcalls/v2/InstanceV2CompatImpl.h"
#include "submodules/TgVoipWebrtc/tgcalls/tgcalls/v2/InstanceV2Impl.h"
#include "submodules/TgVoipWebrtc/tgcalls/tgcalls/v2/InstanceV2ReferenceImpl.h"

namespace {

constexpr size_t kMaximumStringBytes = 256 * 1024;
constexpr size_t kMaximumSignalingBytes = 256 * 1024;
constexpr size_t kMaximumServers = 64;
constexpr size_t kMaximumQueuedEvents = 128;
constexpr size_t kMaximumQueuedBytes = 1024 * 1024;
constexpr auto kStopTimeout = std::chrono::seconds(10);

struct QueuedEvent {
    uint32_t kind = 0;
    uint32_t state = 0;
    std::vector<uint8_t> payload;

    ~QueuedEvent() {
        std::fill(payload.begin(), payload.end(), 0);
    }
};

struct BridgeState {
    std::mutex mutex;
    std::deque<QueuedEvent> events;
    size_t queued_bytes = 0;
    bool overflowed = false;
    uint32_t state = MAKOSH_TGCALLS_CONNECTING_V1;
    bool failed = false;
    bool established = false;
    bool stopped = false;
    std::chrono::steady_clock::time_point connected_at;
    std::chrono::steady_clock::time_point stopped_at;

    void push(QueuedEvent event) {
        std::lock_guard<std::mutex> lock(mutex);
        if (overflowed) {
            return;
        }
        if (events.size() >= kMaximumQueuedEvents
            || event.payload.size() > kMaximumSignalingBytes
            || queued_bytes + event.payload.size() > kMaximumQueuedBytes) {
            overflowed = true;
            return;
        }
        queued_bytes += event.payload.size();
        events.push_back(std::move(event));
    }

    void update(tgcalls::State next) {
        uint32_t mapped = MAKOSH_TGCALLS_CONNECTING_V1;
        switch (next) {
            case tgcalls::State::WaitInit:
            case tgcalls::State::WaitInitAck:
                mapped = MAKOSH_TGCALLS_CONNECTING_V1;
                break;
            case tgcalls::State::Established:
                mapped = MAKOSH_TGCALLS_ESTABLISHED_V1;
                break;
            case tgcalls::State::Reconnecting:
                mapped = MAKOSH_TGCALLS_RECONNECTING_V1;
                break;
            case tgcalls::State::Failed:
                mapped = MAKOSH_TGCALLS_FAILED_V1;
                break;
        }
        {
            std::lock_guard<std::mutex> lock(mutex);
            state = mapped;
            if (next == tgcalls::State::Established && !established) {
                established = true;
                connected_at = std::chrono::steady_clock::now();
            }
            if (next == tgcalls::State::Failed) {
                failed = true;
            }
        }
        push(QueuedEvent {
            .kind = MAKOSH_TGCALLS_STATE_EVENT_V1,
            .state = mapped,
            .payload = {},
        });
    }

    uint32_t duration_seconds() const {
        if (!established) {
            return 0;
        }
        const auto end = stopped ? stopped_at : std::chrono::steady_clock::now();
        const auto seconds =
            std::chrono::duration_cast<std::chrono::seconds>(end - connected_at).count();
        if (seconds <= 0) {
            return 0;
        }
        return static_cast<uint32_t>(
            std::min<int64_t>(seconds, static_cast<int64_t>(UINT32_MAX)));
    }
};

struct StopCompletion {
    std::mutex mutex;
    std::condition_variable changed;
    bool completed = false;
};

struct Session {
    std::unique_ptr<tgcalls::Instance> instance;
    std::shared_ptr<std::array<uint8_t, MAKOSH_TGCALLS_KEY_BYTES_V1>> key;
    std::shared_ptr<BridgeState> bridge;
    std::shared_ptr<StopCompletion> stop_completion;
    bool stopped = false;
    int64_t connection_id = 0;
};

void register_implementations() {
    static std::once_flag once;
    std::call_once(once, [] {
        tgcalls::Register<tgcalls::InstanceImpl>();
        tgcalls::Register<tgcalls::InstanceV2Impl>();
        tgcalls::Register<tgcalls::InstanceV2CompatImpl>();
        tgcalls::Register<tgcalls::InstanceV2ReferenceImpl>();
        tgcalls::SetLoggingFunction([](const std::string &) {
            // Native debug output may contain provider-derived data. Макошь
            // deliberately exposes no bridge logging callback.
        });
    });
}

bool bounded_string(const char *value, bool required) {
    if (value == nullptr) {
        return !required;
    }
    const auto length = strnlen(value, kMaximumStringBytes + 1);
    return length <= kMaximumStringBytes && (!required || length > 0);
}

std::string peer_tag_hex(const uint8_t peer_tag[MAKOSH_TGCALLS_PEER_TAG_BYTES_V1]) {
    std::ostringstream output;
    output << std::hex << std::setfill('0');
    for (size_t index = 0; index < MAKOSH_TGCALLS_PEER_TAG_BYTES_V1; ++index) {
        output << std::setw(2) << static_cast<unsigned int>(peer_tag[index]);
    }
    return output.str();
}

bool valid_server(const МакошьTgCallsServerV1 &server) {
    if (server.abi_version != MAKOSH_TGCALLS_ABI_VERSION_V1
        || server.port == 0
        || !bounded_string(server.host, true)) {
        return false;
    }
    if (server.kind == MAKOSH_TGCALLS_TELEGRAM_REFLECTOR_V1) {
        return true;
    }
    if (server.kind == MAKOSH_TGCALLS_WEBRTC_V1) {
        return bounded_string(server.username, false)
            && bounded_string(server.password, false)
            && (server.supports_stun != 0 || server.supports_turn != 0);
    }
    return false;
}

std::vector<tgcalls::RtcServer> map_servers(
    const МакошьTgCallsServerV1 *servers,
    size_t count) {
    std::vector<tgcalls::RtcServer> mapped;
    mapped.reserve(count);
    for (size_t index = 0; index < count; ++index) {
        const auto &source = servers[index];
        tgcalls::RtcServer server;
        server.id = source.reflector_id;
        server.host = source.host;
        server.port = source.port;
        server.isTcp = source.is_tcp != 0;
        if (source.kind == MAKOSH_TGCALLS_TELEGRAM_REFLECTOR_V1) {
            server.login = "reflector";
            server.password = peer_tag_hex(source.peer_tag);
            server.isTurn = true;
        } else {
            server.login = source.username == nullptr ? "" : source.username;
            server.password = source.password == nullptr ? "" : source.password;
            server.isTurn = source.supports_turn != 0;
        }
        mapped.push_back(std::move(server));
        if (source.kind == MAKOSH_TGCALLS_WEBRTC_V1
            && source.supports_stun != 0
            && source.supports_turn != 0) {
            auto stun = mapped.back();
            stun.isTurn = false;
            stun.isTcp = false;
            mapped.push_back(std::move(stun));
        }
    }
    return mapped;
}

void fill_snapshot(Session &session, МакошьTgCallsSnapshotV1 *output) {
    std::lock_guard<std::mutex> lock(session.bridge->mutex);
    output->abi_version = MAKOSH_TGCALLS_ABI_VERSION_V1;
    output->state = session.bridge->state;
    output->duration_seconds = session.bridge->duration_seconds();
    output->connection_id = session.connection_id;
    output->failed = session.bridge->failed ? 1 : 0;
}

} // namespace

extern "C" uint32_t makosh_tgcalls_abi_version_v1(void) {
    return MAKOSH_TGCALLS_ABI_VERSION_V1;
}

extern "C" size_t makosh_tgcalls_version_count_v1(void) {
    register_implementations();
    return tgcalls::Meta::Versions().size();
}

extern "C" int32_t makosh_tgcalls_version_at_v1(
    size_t index,
    char *output,
    size_t output_capacity) {
    register_implementations();
    const auto versions = tgcalls::Meta::Versions();
    if (index >= versions.size() || output == nullptr) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    const auto &version = versions[index];
    if (output_capacity <= version.size()) {
        return MAKOSH_TGCALLS_BUFFER_TOO_SMALL_V1;
    }
    std::memcpy(output, version.data(), version.size());
    output[version.size()] = '\0';
    return MAKOSH_TGCALLS_OK_V1;
}

extern "C" int32_t makosh_tgcalls_max_layer_v1(void) {
    register_implementations();
    return tgcalls::Meta::MaxLayer();
}

extern "C" int32_t makosh_tgcalls_session_create_v1(
    const МакошьTgCallsSessionConfigV1 *config,
    void **session_out) {
    if (config == nullptr
        || session_out == nullptr
        || config->abi_version != MAKOSH_TGCALLS_ABI_VERSION_V1
        || !bounded_string(config->library_version, true)
        || !bounded_string(config->call_config, false)
        || !bounded_string(config->custom_parameters, false)
        || !bounded_string(config->input_device_id, false)
        || !bounded_string(config->output_device_id, false)
        || config->initialization_timeout_seconds <= 0.0
        || config->receive_timeout_seconds <= 0.0
        || config->encryption_key == nullptr
        || config->encryption_key_length != MAKOSH_TGCALLS_KEY_BYTES_V1
        || config->servers == nullptr
        || config->server_count == 0
        || config->server_count > kMaximumServers * 2) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    for (size_t index = 0; index < config->server_count; ++index) {
        if (!valid_server(config->servers[index])) {
            return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
        }
    }

    register_implementations();
    const std::string requested_version(config->library_version);
    const auto versions = tgcalls::Meta::Versions();
    if (std::find(versions.begin(), versions.end(), requested_version) == versions.end()) {
        return MAKOSH_TGCALLS_UNSUPPORTED_VERSION_V1;
    }

    auto key =
        std::make_shared<std::array<uint8_t, MAKOSH_TGCALLS_KEY_BYTES_V1>>();
    std::copy_n(config->encryption_key, MAKOSH_TGCALLS_KEY_BYTES_V1, key->begin());
    auto bridge = std::make_shared<BridgeState>();
    auto descriptor = tgcalls::Descriptor {
        .version = requested_version,
        .config = {
            .initializationTimeout = config->initialization_timeout_seconds,
            .receiveTimeout = config->receive_timeout_seconds,
            .dataSaving = tgcalls::DataSaving::Never,
            .enableP2P = config->enable_p2p != 0,
            .allowTCP = config->allow_tcp != 0,
            .enableStunMarking = false,
            .enableAEC = false,
            .enableNS = true,
            .enableAGC = true,
            .enableCallUpgrade = false,
            .enableVolumeControl = false,
            .logPath = {""},
            .statsLogPath = {""},
            .maxApiLayer = tgcalls::Meta::MaxLayer(),
            .enableHighBitrateVideo = false,
            .preferredVideoCodecs = {},
            .protocolVersion = tgcalls::ProtocolVersion::V0,
            .customParameters =
                config->custom_parameters == nullptr ? "" : config->custom_parameters,
        },
        .persistentState = {},
        .endpoints = {},
        .proxy = nullptr,
        .rtcServers = map_servers(config->servers, config->server_count),
        .initialNetworkType = tgcalls::NetworkType::Unknown,
        .encryptionKey = tgcalls::EncryptionKey(key, config->is_outgoing != 0),
        .mediaDevicesConfig = {
            .audioInputId =
                config->input_device_id == nullptr ? "" : config->input_device_id,
            .audioOutputId =
                config->output_device_id == nullptr ? "" : config->output_device_id,
        },
        .videoCapture = nullptr,
        .stateUpdated = [bridge](tgcalls::State state) {
            bridge->update(state);
        },
        .signalBarsUpdated = [](int) {},
        .audioLevelUpdated = [](float) {},
        .remoteBatteryLevelIsLowUpdated = [](bool) {},
        .remoteMediaStateUpdated = [](tgcalls::AudioState, tgcalls::VideoState) {},
        .remotePrefferedAspectRatioUpdated = [](float) {},
        .signalingDataEmitted = [bridge](const std::vector<uint8_t> &data) {
            bridge->push(QueuedEvent {
                .kind = MAKOSH_TGCALLS_SIGNALING_EVENT_V1,
                .state = 0,
                .payload = data,
            });
        },
        .createAudioDeviceModule = {},
        .createWrappedAudioDeviceModule = {},
        .initialInputDeviceId =
            config->input_device_id == nullptr ? "" : config->input_device_id,
        .initialOutputDeviceId =
            config->output_device_id == nullptr ? "" : config->output_device_id,
        .directConnectionChannel = nullptr,
    };

    // Current pinned tgcalls exposes no server-config hook; Telegram's own
    // macOS wrapper also accepts TDLib call config without applying it. The
    // bounded value is deliberately neither logged nor retained here.
    auto instance = tgcalls::Meta::Create(requested_version, std::move(descriptor));
    if (!instance) {
        std::fill(key->begin(), key->end(), 0);
        return MAKOSH_TGCALLS_NATIVE_FAILURE_V1;
    }
    auto session = std::make_unique<Session>();
    session->instance = std::move(instance);
    session->key = std::move(key);
    session->bridge = std::move(bridge);
    *session_out = session.release();
    return MAKOSH_TGCALLS_OK_V1;
}

extern "C" int32_t makosh_tgcalls_session_receive_signaling_v1(
    void *session,
    const uint8_t *data,
    size_t data_length) {
    auto *typed = static_cast<Session *>(session);
    if (typed == nullptr
        || typed->stopped
        || data == nullptr
        || data_length == 0
        || data_length > kMaximumSignalingBytes) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    typed->instance->receiveSignalingData(std::vector<uint8_t>(data, data + data_length));
    return MAKOSH_TGCALLS_OK_V1;
}

extern "C" int32_t makosh_tgcalls_session_set_muted_v1(void *session, uint8_t muted) {
    auto *typed = static_cast<Session *>(session);
    if (typed == nullptr || typed->stopped) {
        return MAKOSH_TGCALLS_INVALID_STATE_V1;
    }
    typed->instance->setMuteMicrophone(muted != 0);
    return MAKOSH_TGCALLS_OK_V1;
}

extern "C" int32_t makosh_tgcalls_session_poll_event_v1(
    void *session,
    МакошьTgCallsEventV1 *event_out,
    uint8_t *payload_out,
    size_t payload_capacity) {
    auto *typed = static_cast<Session *>(session);
    if (typed == nullptr || event_out == nullptr) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    std::lock_guard<std::mutex> lock(typed->bridge->mutex);
    if (typed->bridge->overflowed) {
        return MAKOSH_TGCALLS_QUEUE_OVERFLOW_V1;
    }
    if (typed->bridge->events.empty()) {
        return MAKOSH_TGCALLS_OK_V1;
    }
    const auto &event = typed->bridge->events.front();
    event_out->abi_version = MAKOSH_TGCALLS_ABI_VERSION_V1;
    event_out->kind = event.kind;
    event_out->state = event.state;
    event_out->payload_length = event.payload.size();
    if (event.payload.size() > payload_capacity
        || (!event.payload.empty() && payload_out == nullptr)) {
        return MAKOSH_TGCALLS_BUFFER_TOO_SMALL_V1;
    }
    if (!event.payload.empty()) {
        std::copy(event.payload.begin(), event.payload.end(), payload_out);
    }
    typed->bridge->queued_bytes -= event.payload.size();
    typed->bridge->events.pop_front();
    return MAKOSH_TGCALLS_EVENT_V1;
}

extern "C" int32_t makosh_tgcalls_session_snapshot_v1(
    void *session,
    МакошьTgCallsSnapshotV1 *snapshot_out) {
    auto *typed = static_cast<Session *>(session);
    if (typed == nullptr || snapshot_out == nullptr) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    if (!typed->stopped) {
        typed->connection_id = typed->instance->getPreferredRelayId();
    }
    fill_snapshot(*typed, snapshot_out);
    return MAKOSH_TGCALLS_OK_V1;
}

extern "C" int32_t makosh_tgcalls_session_stop_v1(
    void *session,
    МакошьTgCallsSnapshotV1 *snapshot_out) {
    auto *typed = static_cast<Session *>(session);
    if (typed == nullptr || snapshot_out == nullptr) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    if (!typed->stopped) {
        typed->connection_id = typed->instance->getPreferredRelayId();
        if (!typed->stop_completion) {
            typed->stop_completion = std::make_shared<StopCompletion>();
            const auto completion = typed->stop_completion;
            typed->instance->stop([completion](tgcalls::FinalState) {
                {
                    std::lock_guard<std::mutex> lock(completion->mutex);
                    completion->completed = true;
                }
                completion->changed.notify_one();
            });
        }
        const auto completion = typed->stop_completion;
        std::unique_lock<std::mutex> lock(completion->mutex);
        if (!completion->changed.wait_for(lock, kStopTimeout, [completion] {
                return completion->completed;
            })) {
            return MAKOSH_TGCALLS_NATIVE_FAILURE_V1;
        }
        lock.unlock();
        typed->instance.reset();
        std::fill(typed->key->begin(), typed->key->end(), 0);
        typed->key.reset();
        typed->stop_completion.reset();
        typed->stopped = true;
        {
            std::lock_guard<std::mutex> bridge_lock(typed->bridge->mutex);
            typed->bridge->stopped = true;
            typed->bridge->stopped_at = std::chrono::steady_clock::now();
        }
    }
    fill_snapshot(*typed, snapshot_out);
    return MAKOSH_TGCALLS_OK_V1;
}

extern "C" int32_t makosh_tgcalls_session_destroy_v1(void *session) {
    auto *typed = static_cast<Session *>(session);
    if (typed == nullptr || !typed->stopped) {
        return MAKOSH_TGCALLS_INVALID_STATE_V1;
    }
    delete typed;
    return MAKOSH_TGCALLS_OK_V1;
}
