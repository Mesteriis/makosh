#![forbid(unsafe_code)]

mod admission;
mod client;
mod managed_runtime;
mod scheduler;
mod scheduler_io;

pub use admission::{
    calendar_module_descriptor_v1, calendar_settings_schema_bytes_v1, calendar_settings_schema_v1,
    scheduler_job_contract_v1, scheduler_receipt_contract_v1,
    scheduler_schedule_control_contract_v1,
};
pub use client::{CalendarClientRuntimeContextV1, dispatch_calendar_client_request_v1};
pub use managed_runtime::{
    CalendarManagedRuntimeErrorV1, CalendarManagedRuntimeV1, CalendarRuntimeAdmissionV1,
};
pub use scheduler::{
    CalendarSchedulerEnvelopeContextV1, CalendarSchedulerEnvelopeErrorV1,
    build_cancel_reminder_schedule_v1, build_ensure_reminder_schedule_v1,
    calendar_schedule_control_message_id_v1,
};
pub use scheduler_io::{
    CalendarSchedulerRuntimeContextV1, CalendarSchedulerRuntimeErrorV1,
    consume_calendar_reminder_due_once_v1, consume_calendar_schedule_result_once_v1,
    relay_calendar_outbox_once_v1,
};

pub const PACKAGE: &str = "makosh-calendar-runtime";
