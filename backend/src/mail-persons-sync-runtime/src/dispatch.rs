use makosh_mail_address_book_contract::wire_person_source::{
    MailPersonSourceObservedV1, MailPersonSourceRemovedV1, MailPersonSourceUpdatedV1,
};
use makosh_mail_persons_sync_core::{
    MailPersonsSyncCoreErrorV1, map_observed_to_persons_v1, map_removed_to_persons_v1,
    map_updated_to_persons_v1,
};
use makosh_persons_api::wire::PersonsCommandV1;

pub enum MailPersonSourceInputV1 {
    Observed(MailPersonSourceObservedV1),
    Updated(MailPersonSourceUpdatedV1),
    Removed(MailPersonSourceRemovedV1),
}

pub fn dispatch_mail_person_source_v1(
    input: MailPersonSourceInputV1,
) -> Result<PersonsCommandV1, MailPersonsSyncCoreErrorV1> {
    match input {
        MailPersonSourceInputV1::Observed(value) => map_observed_to_persons_v1(&value),
        MailPersonSourceInputV1::Updated(value) => map_updated_to_persons_v1(&value),
        MailPersonSourceInputV1::Removed(value) => map_removed_to_persons_v1(&value),
    }
}
