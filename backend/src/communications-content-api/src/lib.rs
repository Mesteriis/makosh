//! Public generated contracts for bounded canonical Communications body reads.

pub const PACKAGE: &str = "makosh-communications-content-api";
pub const CONTENT_TICKET_CONTRACT_NAME_V1: &str = "communications.content.ticket";
pub const CONTENT_READ_CONTRACT_NAME_V1: &str = "communications.content.read";
pub const CONTENT_TICKET_CONNECT_PATH_V1: &str = "/makosh.communications.content.ticket.v1.CommunicationsContentTicketService/IssueMessageBodyRead";
pub const CONTENT_READ_BLOB_PATH_V1: &str = "/api/blobs/communications/v1/message-body";
pub const CONTENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const CONTENT_CONTRACT_REVISION_V1: u32 = 1;
pub const MAX_MESSAGE_BODY_BYTES_V1: u64 = 256 * 1024;

mod ticket_wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.content.ticket.v1.rs"
    ));
}

mod read_wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.content.read.v1.rs"
    ));
}

pub use read_wire::ReadMessageBodyRequestV1;
pub use ticket_wire::{IssueMessageBodyReadRequestV1, IssueMessageBodyReadResponseV1};

include!(concat!(
    env!("OUT_DIR"),
    "/communications_content_ticket_schema.rs"
));
include!(concat!(
    env!("OUT_DIR"),
    "/communications_content_read_schema.rs"
));

pub const COMMUNICATIONS_CONTENT_TICKET_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communications-content-ticket-v1.bin"
));
pub const COMMUNICATIONS_CONTENT_READ_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communications-content-read-v1.bin"
));
