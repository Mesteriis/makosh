//! Event Hub catalog resolution and contract compatibility checks.

mod contracts;
mod entries;

use makosh_kernel_control_store::ModuleRegistryStore;
use makosh_kernel_control_store_sqlite::StoreError;

pub use contracts::{EventCatalogContractV1, EventCatalogParticipantV1};
pub use entries::EventCatalogEntryV1;

pub fn resolve<S>(store: &S) -> Result<Vec<EventCatalogEntryV1>, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    entries::resolve(store)
}

pub fn resolve_contracts<S>(store: &S) -> Result<Vec<EventCatalogContractV1>, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    let entries = resolve(store)?;
    contracts::build(entries).map_err(|error| format!("Event catalog is incompatible: {error:?}"))
}
