/*
 * Test-only ABI fixture for managed Telegram runtime admission.
 *
 * It proves exact native artifact binding and process composition. It has no
 * audio device, network transport or production media behavior.
 */

#include "../../../src/telegram-call-media-tgcalls/native/bridge.h"

#include <stdlib.h>
#include <string.h>

typedef struct {
    int stopped;
    int event_index;
    int muted;
} МакошьTgCallsFixtureSession;

static const char *const MAKOSH_TGCALLS_VERSIONS[] = {"13.0.0", "14.0.0"};

static int32_t fill_snapshot(
    const МакошьTgCallsFixtureSession *session,
    МакошьTgCallsSnapshotV1 *snapshot_out) {
    if (session == NULL || snapshot_out == NULL) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    snapshot_out->abi_version = MAKOSH_TGCALLS_ABI_VERSION_V1;
    snapshot_out->state = MAKOSH_TGCALLS_ESTABLISHED_V1;
    snapshot_out->duration_seconds = session->stopped ? 1u : 0u;
    snapshot_out->connection_id = 7001;
    snapshot_out->failed = 0;
    return MAKOSH_TGCALLS_OK_V1;
}

uint32_t makosh_tgcalls_abi_version_v1(void) {
    return MAKOSH_TGCALLS_ABI_VERSION_V1;
}

size_t makosh_tgcalls_version_count_v1(void) {
    return sizeof(MAKOSH_TGCALLS_VERSIONS) / sizeof(MAKOSH_TGCALLS_VERSIONS[0]);
}

int32_t makosh_tgcalls_version_at_v1(
    size_t index,
    char *output,
    size_t output_capacity) {
    if (index >= makosh_tgcalls_version_count_v1() || output == NULL) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    size_t length = strlen(MAKOSH_TGCALLS_VERSIONS[index]);
    if (output_capacity <= length) {
        return MAKOSH_TGCALLS_BUFFER_TOO_SMALL_V1;
    }
    memcpy(output, MAKOSH_TGCALLS_VERSIONS[index], length + 1);
    return MAKOSH_TGCALLS_OK_V1;
}

int32_t makosh_tgcalls_max_layer_v1(void) {
    return 92;
}

int32_t makosh_tgcalls_session_create_v1(
    const МакошьTgCallsSessionConfigV1 *config,
    void **session_out) {
    if (config == NULL || session_out == NULL
        || config->abi_version != MAKOSH_TGCALLS_ABI_VERSION_V1
        || config->library_version == NULL
        || (strcmp(config->library_version, "13.0.0") != 0
            && strcmp(config->library_version, "14.0.0") != 0)
        || config->encryption_key == NULL
        || config->encryption_key_length != MAKOSH_TGCALLS_KEY_BYTES_V1
        || config->servers == NULL
        || config->server_count == 0) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    МакошьTgCallsFixtureSession *session =
        calloc(1, sizeof(МакошьTgCallsFixtureSession));
    if (session == NULL) {
        return MAKOSH_TGCALLS_NATIVE_FAILURE_V1;
    }
    session->event_index = 0;
    *session_out = session;
    return MAKOSH_TGCALLS_OK_V1;
}

int32_t makosh_tgcalls_session_receive_signaling_v1(
    void *raw_session,
    const uint8_t *data,
    size_t data_length) {
    МакошьTgCallsFixtureSession *session = raw_session;
    static const uint8_t expected[] = "incoming-signal";
    if (session == NULL || session->stopped || data == NULL || data_length == 0) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    if (data_length != sizeof(expected) - 1
        || memcmp(data, expected, sizeof(expected) - 1) != 0) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    return MAKOSH_TGCALLS_OK_V1;
}

int32_t makosh_tgcalls_session_set_muted_v1(void *raw_session, uint8_t muted) {
    МакошьTgCallsFixtureSession *session = raw_session;
    if (session == NULL || session->stopped) {
        return MAKOSH_TGCALLS_INVALID_STATE_V1;
    }
    session->muted = muted != 0;
    return MAKOSH_TGCALLS_OK_V1;
}

int32_t makosh_tgcalls_session_poll_event_v1(
    void *raw_session,
    МакошьTgCallsEventV1 *event_out,
    uint8_t *payload_out,
    size_t payload_capacity) {
    МакошьTgCallsFixtureSession *session = raw_session;
    if (session == NULL || event_out == NULL) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    if (session->event_index == 0) {
        static const uint8_t signaling[] = "outbound-signal";
        size_t signaling_length = sizeof(signaling) - 1;
        if (payload_out == NULL || payload_capacity < signaling_length) {
            return MAKOSH_TGCALLS_BUFFER_TOO_SMALL_V1;
        }
        memcpy(payload_out, signaling, signaling_length);
        session->event_index = 1;
        event_out->abi_version = MAKOSH_TGCALLS_ABI_VERSION_V1;
        event_out->kind = MAKOSH_TGCALLS_SIGNALING_EVENT_V1;
        event_out->state = MAKOSH_TGCALLS_CONNECTING_V1;
        event_out->payload_length = signaling_length;
        return MAKOSH_TGCALLS_EVENT_V1;
    }
    if (session->event_index > 1) {
        return MAKOSH_TGCALLS_OK_V1;
    }
    session->event_index = 2;
    event_out->abi_version = MAKOSH_TGCALLS_ABI_VERSION_V1;
    event_out->kind = MAKOSH_TGCALLS_STATE_EVENT_V1;
    event_out->state = MAKOSH_TGCALLS_ESTABLISHED_V1;
    event_out->payload_length = 0;
    return MAKOSH_TGCALLS_EVENT_V1;
}

int32_t makosh_tgcalls_session_snapshot_v1(
    void *raw_session,
    МакошьTgCallsSnapshotV1 *snapshot_out) {
    return fill_snapshot(raw_session, snapshot_out);
}

int32_t makosh_tgcalls_session_stop_v1(
    void *raw_session,
    МакошьTgCallsSnapshotV1 *snapshot_out) {
    МакошьTgCallsFixtureSession *session = raw_session;
    if (session == NULL) {
        return MAKOSH_TGCALLS_INVALID_ARGUMENT_V1;
    }
    session->stopped = 1;
    return fill_snapshot(session, snapshot_out);
}

int32_t makosh_tgcalls_session_destroy_v1(void *raw_session) {
    МакошьTgCallsFixtureSession *session = raw_session;
    if (session == NULL || !session->stopped) {
        return MAKOSH_TGCALLS_INVALID_STATE_V1;
    }
    free(session);
    return MAKOSH_TGCALLS_OK_V1;
}
