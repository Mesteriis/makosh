#include <atomic>
#include <chrono>
#include <condition_variable>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <limits>
#include <memory>
#include <mutex>
#include <string_view>

#include "api/task_queue/default_task_queue_factory.h"
#include "modules/audio_device/include/audio_device.h"
#include "modules/audio_device/include/audio_device_defines.h"

namespace {

constexpr std::string_view kExplicitConsent =
    "--allow-microphone-and-speaker-access";
constexpr uint64_t kRequiredCallbacks = 100;
constexpr auto kCallbackTimeout = std::chrono::seconds(10);

class AudioProbeTransport final : public webrtc::AudioTransport {
public:
    int32_t RecordedDataIsAvailable(
        const void *,
        size_t samples,
        size_t bytes_per_sample,
        size_t channels,
        uint32_t samples_per_second,
        uint32_t,
        int32_t,
        uint32_t current_microphone_level,
        bool,
        uint32_t &new_microphone_level) override {
        new_microphone_level = current_microphone_level;
        if (samples == 0
            || bytes_per_sample == 0
            || channels == 0
            || samples_per_second == 0) {
            return -1;
        }
        recording_callbacks_.fetch_add(1, std::memory_order_relaxed);
        callbacks_changed_.notify_all();
        return 0;
    }

    int32_t NeedMorePlayData(
        size_t samples,
        size_t bytes_per_sample,
        size_t channels,
        uint32_t samples_per_second,
        void *audio_samples,
        size_t &samples_out,
        int64_t *elapsed_time_milliseconds,
        int64_t *ntp_time_milliseconds) override {
        if (!write_silence(
                samples,
                bytes_per_sample,
                channels,
                samples_per_second,
                audio_samples)) {
            return -1;
        }
        samples_out = samples;
        if (elapsed_time_milliseconds != nullptr) {
            *elapsed_time_milliseconds = 0;
        }
        if (ntp_time_milliseconds != nullptr) {
            *ntp_time_milliseconds = 0;
        }
        playout_callbacks_.fetch_add(1, std::memory_order_relaxed);
        callbacks_changed_.notify_all();
        return 0;
    }

    void PullRenderData(
        int bits_per_sample,
        int samples_per_second,
        size_t channels,
        size_t frames,
        void *audio_samples,
        int64_t *elapsed_time_milliseconds,
        int64_t *ntp_time_milliseconds) override {
        const auto bytes_per_sample =
            bits_per_sample > 0 && bits_per_sample % 8 == 0
            ? static_cast<size_t>(bits_per_sample / 8)
            : 0;
        if (!write_silence(
                frames,
                bytes_per_sample,
                channels,
                samples_per_second > 0
                    ? static_cast<uint32_t>(samples_per_second)
                    : 0,
                audio_samples)) {
            return;
        }
        if (elapsed_time_milliseconds != nullptr) {
            *elapsed_time_milliseconds = 0;
        }
        if (ntp_time_milliseconds != nullptr) {
            *ntp_time_milliseconds = 0;
        }
        playout_callbacks_.fetch_add(1, std::memory_order_relaxed);
        callbacks_changed_.notify_all();
    }

    bool wait_for_full_duplex() {
        std::unique_lock<std::mutex> lock(callbacks_mutex_);
        return callbacks_changed_.wait_for(lock, kCallbackTimeout, [this] {
            return recording_callbacks() >= kRequiredCallbacks
                && playout_callbacks() >= kRequiredCallbacks;
        });
    }

    uint64_t recording_callbacks() const {
        return recording_callbacks_.load(std::memory_order_relaxed);
    }

    uint64_t playout_callbacks() const {
        return playout_callbacks_.load(std::memory_order_relaxed);
    }

private:
    static bool write_silence(
        size_t samples,
        size_t bytes_per_sample,
        size_t channels,
        uint32_t samples_per_second,
        void *audio_samples) {
        if (samples == 0
            || bytes_per_sample == 0
            || channels == 0
            || samples_per_second == 0
            || audio_samples == nullptr
            || bytes_per_sample > std::numeric_limits<size_t>::max() / channels) {
            return false;
        }
        const auto bytes_per_frame = bytes_per_sample * channels;
        if (samples > std::numeric_limits<size_t>::max() / bytes_per_frame) {
            return false;
        }
        std::memset(audio_samples, 0, samples * bytes_per_frame);
        return true;
    }

    std::atomic<uint64_t> recording_callbacks_ {0};
    std::atomic<uint64_t> playout_callbacks_ {0};
    std::mutex callbacks_mutex_;
    std::condition_variable callbacks_changed_;
};

class AudioDeviceGuard {
public:
    explicit AudioDeviceGuard(
        rtc::scoped_refptr<webrtc::AudioDeviceModule> device)
        : device_(std::move(device)) {}

    ~AudioDeviceGuard() {
        if (device_->Recording()) {
            device_->StopRecording();
        }
        if (device_->Playing()) {
            device_->StopPlayout();
        }
        device_->RegisterAudioCallback(nullptr);
        if (device_->Initialized()) {
            device_->Terminate();
        }
    }

    webrtc::AudioDeviceModule *get() const {
        return device_.get();
    }

private:
    rtc::scoped_refptr<webrtc::AudioDeviceModule> device_;
};

class MicrophoneMuteGuard {
public:
    MicrophoneMuteGuard(
        webrtc::AudioDeviceModule *device,
        bool original_mute,
        bool enabled)
        : device_(device)
        , original_mute_(original_mute)
        , armed_(enabled) {}

    ~MicrophoneMuteGuard() {
        restore();
    }

    bool restore() {
        if (!armed_) {
            return true;
        }
        if (device_->SetMicrophoneMute(original_mute_) != 0) {
            return false;
        }
        armed_ = false;
        return true;
    }

private:
    webrtc::AudioDeviceModule *device_;
    bool original_mute_;
    bool armed_;
};

int fail(const char *reason) {
    std::cerr << "audio-device-conformance: failed: " << reason << '\n';
    return 1;
}

} // namespace

int main(int argc, char **argv) {
    if (argc != 2 || std::string_view(argv[1]) != kExplicitConsent) {
        std::cerr
            << "usage: makosh_tgcalls_audio_device_conformance "
            << kExplicitConsent << '\n';
        return 2;
    }

    auto task_queue_factory = webrtc::CreateDefaultTaskQueueFactory();
    auto device = webrtc::AudioDeviceModule::Create(
        webrtc::AudioDeviceModule::kPlatformDefaultAudio,
        task_queue_factory.get());
    if (!device) {
        return fail("platform audio device is unavailable");
    }

    AudioProbeTransport transport;
    AudioDeviceGuard guard(std::move(device));
    auto *audio = guard.get();
    if (audio->Init() != 0) {
        return fail("audio device initialization failed");
    }
    if (audio->PlayoutDevices() <= 0 || audio->RecordingDevices() <= 0) {
        return fail("default input or output device is unavailable");
    }
    if (audio->SetPlayoutDevice(0) != 0
        || audio->SetRecordingDevice(0) != 0
        || audio->RegisterAudioCallback(&transport) != 0
        || audio->InitPlayout() != 0
        || audio->InitRecording() != 0) {
        return fail("full-duplex audio device preparation failed");
    }

    bool original_microphone_mute = false;
    bool microphone_mute_available = false;
    if (audio->MicrophoneMuteIsAvailable(&microphone_mute_available) != 0) {
        return fail("microphone mute capability query failed");
    }
    if (microphone_mute_available
        && audio->MicrophoneMute(&original_microphone_mute) != 0) {
        return fail("microphone mute state query failed");
    }
    MicrophoneMuteGuard microphone_mute_guard(
        audio,
        original_microphone_mute,
        microphone_mute_available);

    if (audio->StartPlayout() != 0 || audio->StartRecording() != 0) {
        return fail("full-duplex audio device start failed");
    }
    if (!transport.wait_for_full_duplex()) {
        return fail("bounded full-duplex callback threshold was not reached");
    }

    if (microphone_mute_available) {
        const bool test_mute = !original_microphone_mute;
        bool observed_mute = original_microphone_mute;
        if (audio->SetMicrophoneMute(test_mute) != 0
            || audio->MicrophoneMute(&observed_mute) != 0
            || observed_mute != test_mute) {
            return fail("microphone mute transition or restoration failed");
        }
        if (!microphone_mute_guard.restore()) {
            return fail("microphone mute restoration failed");
        }
    }

    if (audio->StopRecording() != 0 || audio->StopPlayout() != 0) {
        return fail("full-duplex audio device stop failed");
    }
    std::cout
        << "audio-device-conformance: ok recording_callbacks="
        << transport.recording_callbacks()
        << " playout_callbacks="
        << transport.playout_callbacks()
        << '\n';
    return 0;
}
