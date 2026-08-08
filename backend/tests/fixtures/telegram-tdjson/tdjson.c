#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define MAKOSH_QUEUE_CAPACITY 32
#define MAKOSH_PAYLOAD_CAPACITY 4096
#define MAKOSH_STARTUP_RECEIVE_DELAYS 80

typedef struct {
    char queue[MAKOSH_QUEUE_CAPACITY][MAKOSH_PAYLOAD_CAPACITY];
    size_t head;
    size_t tail;
    size_t startup_receive_delays_remaining;
    char current[MAKOSH_PAYLOAD_CAPACITY];
    int folder_7;
    int folder_9;
    int folder_11;
    int folder_reassignment_failure_emitted;
} МакошьTdJsonClient;

static int enqueue(МакошьTdJsonClient *client, const char *payload) {
    size_t next = (client->tail + 1) % MAKOSH_QUEUE_CAPACITY;
    if (next == client->head || strlen(payload) >= MAKOSH_PAYLOAD_CAPACITY) {
        return 0;
    }
    strcpy(client->queue[client->tail], payload);
    client->tail = next;
    return 1;
}

static int extract_extra(const char *request, char *extra, size_t capacity) {
    const char *key = strstr(request, "\"@extra\"");
    if (key == NULL) {
        return 0;
    }
    const char *separator = strchr(key, ':');
    if (separator == NULL) {
        return 0;
    }
    const char *start = strchr(separator, '"');
    if (start == NULL) {
        return 0;
    }
    start += 1;
    const char *end = strchr(start, '"');
    if (end == NULL) {
        return 0;
    }
    size_t length = (size_t)(end - start);
    if (length == 0 || length >= capacity) {
        return 0;
    }
    memcpy(extra, start, length);
    extra[length] = '\0';
    return 1;
}

void *td_json_client_create(void) {
    МакошьTdJsonClient *client = calloc(1, sizeof(МакошьTdJsonClient));
    if (client == NULL) {
        return NULL;
    }
    client->startup_receive_delays_remaining = MAKOSH_STARTUP_RECEIVE_DELAYS;
    client->folder_7 = 1;
    client->folder_9 = 1;
    enqueue(
        client,
        "{\"@type\":\"updateAuthorizationState\",\"authorization_state\":{\"@type\":\"authorizationStateReady\"}}"
    );
    enqueue(
        client,
        "{\"@type\":\"updateNewMessage\",\"message\":{\"id\":7001,\"chat_id\":9001,\"sender_id\":{\"@type\":\"messageSenderUser\",\"user_id\":42},\"is_outgoing\":false,\"date\":1783024000,\"content\":{\"@type\":\"messageText\",\"text\":{\"@type\":\"formattedText\",\"text\":\"managed Telegram evidence\"}}}}"
    );
    enqueue(
        client,
        "{\"@type\":\"updateCall\",\"call\":{\"id\":41,\"unique_id\":5001,\"user_id\":42,\"is_outgoing\":false,\"is_video\":false,\"state\":{\"@type\":\"callStatePending\",\"is_created\":true,\"is_received\":true}}}"
    );
    enqueue(
        client,
        "{\"@type\":\"updateCall\",\"call\":{\"id\":41,\"unique_id\":5001,\"user_id\":42,\"is_outgoing\":false,\"is_video\":false,\"state\":{\"@type\":\"callStateReady\",\"protocol\":{\"@type\":\"callProtocol\",\"udp_p2p\":true,\"udp_reflector\":true,\"min_layer\":65,\"max_layer\":92,\"library_versions\":[\"13.0.0\"]},\"servers\":[{\"@type\":\"callServer\",\"id\":4,\"ip_address\":\"127.0.0.1\",\"ipv6_address\":\"\",\"port\":443,\"type\":{\"@type\":\"callServerTypeTelegramReflector\",\"peer_tag\":\"CAgICAgICAgICAgICAgICA==\",\"is_tcp\":false}}],\"config\":\"managed-private-config\",\"encryption_key\":\""
        "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcH"
        "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcH"
        "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcH"
        "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcH"
        "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBw=="
        "\",\"custom_parameters\":\"managed-private-parameters\",\"allow_p2p\":true,\"is_group_call_supported\":false}}}"
    );
    enqueue(
        client,
        "{\"@type\":\"updateNewCallSignalingData\",\"call_id\":41,\"data\":\"aW5jb21pbmctc2lnbmFs\"}"
    );
    enqueue(
        client,
        "{\"@type\":\"updateCall\",\"call\":{\"id\":41,\"unique_id\":5001,\"user_id\":42,\"is_outgoing\":false,\"is_video\":false,\"state\":{\"@type\":\"callStateDiscarded\",\"reason\":{\"@type\":\"callDiscardReasonMissed\"},\"need_rating\":false,\"need_debug_information\":false,\"need_log\":false}}}"
    );
    return client;
}

void td_json_client_send(void *raw_client, const char *request) {
    МакошьTdJsonClient *client = raw_client;
    char extra[512];
    char response[MAKOSH_PAYLOAD_CAPACITY];
    if (client == NULL || request == NULL ||
        !extract_extra(request, extra, sizeof(extra))) {
        return;
    }
    const char *format;
    if (strstr(request, "\"@type\":\"getChat\"") != NULL) {
        const char *positions;
        if (client->folder_7 && client->folder_9 && client->folder_11) {
            positions =
                "[{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":7},\"order\":7,\"is_pinned\":false},{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":9},\"order\":9,\"is_pinned\":false},{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":11},\"order\":11,\"is_pinned\":false}]";
        } else if (client->folder_7 && client->folder_9) {
            positions =
                "[{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":7},\"order\":7,\"is_pinned\":false},{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":9},\"order\":9,\"is_pinned\":false}]";
        } else if (client->folder_9 && client->folder_11) {
            positions =
                "[{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":9},\"order\":9,\"is_pinned\":false},{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":11},\"order\":11,\"is_pinned\":false}]";
        } else {
            positions = "[]";
        }
        int written = snprintf(
            response,
            sizeof(response),
            "{\"@type\":\"chat\",\"id\":9002,\"positions\":%s,\"@extra\":\"%s\"}",
            positions,
            extra
        );
        if (written > 0 && (size_t)written < sizeof(response)) {
            enqueue(client, response);
        }
        return;
    } else if (strstr(request, "\"@type\":\"getChatFolder\"") != NULL) {
        format =
            "{\"@type\":\"chatFolder\",\"name\":{\"@type\":\"chatFolderName\",\"text\":\"Managed folder\"},\"icon\":{\"@type\":\"chatFolderIcon\",\"name\":\"Custom\"},\"included_chat_ids\":[9002],\"@extra\":\"%s\"}";
    } else if (strstr(request, "\"@type\":\"addChatToList\"") != NULL
        && strstr(request, "\"chat_folder_id\":11") != NULL) {
        client->folder_11 = 1;
        enqueue(
            client,
            "{\"@type\":\"updateChatPosition\",\"chat_id\":9002,\"position\":{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":11},\"order\":11,\"is_pinned\":false}}"
        );
        if (strstr(extra, "managed-telegram-folder-reassign-retry:add:11") != NULL
            && !client->folder_reassignment_failure_emitted) {
            client->folder_reassignment_failure_emitted = 1;
            format =
                "{\"@type\":\"error\",\"code\":503,\"message\":\"private fixture failure\",\"@extra\":\"%s\"}";
        } else {
            format = "{\"@type\":\"ok\",\"@extra\":\"%s\"}";
        }
    } else if (strstr(request, "\"@type\":\"editChatFolder\"") != NULL
        && strstr(request, "\"chat_folder_id\":7") != NULL) {
        client->folder_7 = 0;
        enqueue(
            client,
            "{\"@type\":\"updateChatPosition\",\"chat_id\":9002,\"position\":{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":7},\"order\":0,\"is_pinned\":false}}"
        );
        format = "{\"@type\":\"ok\",\"@extra\":\"%s\"}";
    } else if (strstr(request, "\"@type\":\"getMe\"") != NULL) {
        format = "{\"@type\":\"user\",\"id\":777,\"@extra\":\"%s\"}";
    } else if (strstr(request, "\"@type\":\"getChatHistory\"") != NULL) {
        format =
            "{\"@type\":\"messages\",\"total_count\":1,\"messages\":[{\"@type\":\"message\",\"id\":7200,\"chat_id\":9002,\"sender_id\":{\"@type\":\"messageSenderUser\",\"user_id\":44},\"is_outgoing\":false,\"date\":1783024100,\"content\":{\"@type\":\"messageText\",\"text\":{\"@type\":\"formattedText\",\"text\":\"managed Telegram history fixture\"}}}],\"@extra\":\"%s\"}";
    } else if (strstr(request, "\"@type\":\"createCall\"") != NULL) {
        format = "{\"@type\":\"callId\",\"id\":52,\"@extra\":\"%s\"}";
    } else if (strstr(request, "\"@type\":\"sendCallSignalingData\"") != NULL
        && strstr(request, "\"data\":\"b3V0Ym91bmQtc2lnbmFs\"") == NULL) {
        format = "{\"@type\":\"error\",\"code\":400,\"message\":\"invalid fixture signaling\",\"@extra\":\"%s\"}";
    } else {
        format = strstr(request, "\"@type\":\"sendMessage\"") == NULL
            ? "{\"@type\":\"ok\",\"@extra\":\"%s\"}"
            : "{\"@type\":\"message\",\"id\":8001,\"@extra\":\"%s\"}";
    }
    int written = snprintf(response, sizeof(response), format, extra);
    if (written > 0 && (size_t)written < sizeof(response)) {
        enqueue(client, response);
    }
    if (strstr(request, "\"@type\":\"createCall\"") != NULL) {
        enqueue(
            client,
            "{\"@type\":\"updateCall\",\"call\":{\"id\":52,\"unique_id\":6001,\"user_id\":43,\"is_outgoing\":true,\"is_video\":false,\"state\":{\"@type\":\"callStatePending\",\"is_created\":true,\"is_received\":true}}}"
        );
    }
    if (strstr(request, "\"@type\":\"discardCall\"") != NULL
        && strstr(request, "\"call_id\":52") != NULL) {
        enqueue(
            client,
            "{\"@type\":\"updateCall\",\"call\":{\"id\":52,\"unique_id\":6001,\"user_id\":43,\"is_outgoing\":true,\"is_video\":false,\"state\":{\"@type\":\"callStateDiscarded\",\"reason\":{\"@type\":\"callDiscardReasonHungUp\"},\"need_rating\":false,\"need_debug_information\":false,\"need_log\":false}}}"
        );
    }
    if (strstr(request, "outage replay trigger") != NULL) {
        enqueue(
            client,
            "{\"@type\":\"updateNewMessage\",\"message\":{\"id\":7002,\"chat_id\":9001,\"sender_id\":{\"@type\":\"messageSenderUser\",\"user_id\":42},\"is_outgoing\":false,\"date\":1783024001,\"content\":{\"@type\":\"messageText\",\"text\":{\"@type\":\"formattedText\",\"text\":\"managed Telegram outage replay evidence\"}}}}"
        );
    }
    if (strstr(request, "operational fixture trigger") != NULL) {
        enqueue(
            client,
            "{\"@type\":\"updateNewMessage\",\"message\":{\"id\":7100,\"chat_id\":9002,\"sender_id\":{\"@type\":\"messageSenderUser\",\"user_id\":44},\"is_outgoing\":false,\"date\":1783024101,\"content\":{\"@type\":\"messageDocument\",\"document\":{\"file_name\":\"report.pdf\",\"mime_type\":\"application/pdf\",\"document\":{\"@type\":\"file\",\"id\":42}},\"caption\":{\"@type\":\"formattedText\",\"text\":\"managed operational document\"}}}}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateFile\",\"file\":{\"@type\":\"file\",\"id\":42,\"size\":100,\"expected_size\":100,\"local\":{\"downloaded_size\":100,\"is_downloading_active\":false,\"is_downloading_completed\":true},\"remote\":{\"unique_id\":\"managed-file-42\"}}}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateMessageContent\",\"chat_id\":9002,\"message_id\":7100,\"new_content\":{\"@type\":\"messageText\",\"text\":{\"@type\":\"formattedText\",\"text\":\"edited operational fixture\"}}}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateMessageIsPinned\",\"chat_id\":9002,\"message_id\":7100,\"is_pinned\":true}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateMessageInteractionInfo\",\"chat_id\":9002,\"message_id\":7100,\"interaction_info\":{\"reactions\":{\"recent_reactions\":[{\"sender_id\":{\"@type\":\"messageSenderUser\",\"user_id\":44},\"type\":{\"@type\":\"reactionTypeEmoji\",\"emoji\":\"ok\"},\"is_outgoing\":false}]}}}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateChatPosition\",\"chat_id\":9002,\"position\":{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":7},\"order\":9,\"is_pinned\":true}}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateChatPosition\",\"chat_id\":9002,\"position\":{\"list\":{\"@type\":\"chatListFolder\",\"chat_folder_id\":9},\"order\":8,\"is_pinned\":false}}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateChatNotificationSettings\",\"chat_id\":9002,\"notification_settings\":{\"use_default_mute_for\":false,\"mute_for\":3600}}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateNewMessage\",\"message\":{\"id\":7101,\"chat_id\":9002,\"sender_id\":{\"@type\":\"messageSenderUser\",\"user_id\":44},\"is_outgoing\":false,\"date\":1783024102,\"content\":{\"@type\":\"messageText\",\"text\":{\"@type\":\"formattedText\",\"text\":\"managed tombstone fixture\"}}}}"
        );
        enqueue(
            client,
            "{\"@type\":\"updateDeleteMessages\",\"chat_id\":9002,\"message_ids\":[7101],\"is_permanent\":true}"
        );
    }
}

const char *td_json_client_receive(void *raw_client, double timeout) {
    МакошьTdJsonClient *client = raw_client;
    if (client == NULL) {
        return NULL;
    }
    if (client->startup_receive_delays_remaining > 0) {
        client->startup_receive_delays_remaining -= 1;
        if (timeout > 0.0) {
            double bounded = timeout > 0.05 ? 0.05 : timeout;
            usleep((useconds_t)(bounded * 1000000.0));
        }
        return NULL;
    }
    if (client->head == client->tail) {
        if (timeout > 0.0) {
            double bounded = timeout > 0.05 ? 0.05 : timeout;
            usleep((useconds_t)(bounded * 1000000.0));
        }
        return NULL;
    }
    strcpy(client->current, client->queue[client->head]);
    client->head = (client->head + 1) % MAKOSH_QUEUE_CAPACITY;
    return client->current;
}

const char *td_json_client_execute(void *raw_client, const char *request) {
    (void)raw_client;
    (void)request;
    return "{\"@type\":\"ok\"}";
}

void td_json_client_destroy(void *raw_client) {
    free(raw_client);
}
