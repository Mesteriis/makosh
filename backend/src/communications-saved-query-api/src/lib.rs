//! Generated public contract for owner-local Communications saved searches.

pub const PACKAGE: &str = "makosh-communications-saved-query-api";
pub const SAVED_SEARCH_CONTRACT_NAME_V1: &str = "communications.saved-search";
pub const SAVED_SEARCH_CONNECT_PATH_V1: &str =
    "/makosh.communications.saved_search.v1.CommunicationsSavedSearchService/Manage";
pub const SAVED_SEARCH_CONTRACT_MAJOR_V1: u32 = 1;
pub const SAVED_SEARCH_CONTRACT_REVISION_V1: u32 = 1;

mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.saved_search.v1.rs"
    ));
}

pub use wire::*;

include!(concat!(
    env!("OUT_DIR"),
    "/communications_saved_search_schema.rs"
));

pub const COMMUNICATIONS_SAVED_SEARCH_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communications-saved-search-v1.bin"
));
