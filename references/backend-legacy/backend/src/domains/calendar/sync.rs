use thiserror::Error;

/// Basic ICS export for a single event
pub fn export_event_ics(
    title: &str,
    description: Option<&str>,
    location: Option<&str>,
    start_at: &str,
    end_at: &str,
    timezone: Option<&str>,
) -> String {
    let tz = timezone.unwrap_or("Europe/Madrid");
    let desc = description.unwrap_or("");
    let loc = location.unwrap_or("");
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Макошь//Calendar//EN\r\nBEGIN:VEVENT\r\nDTSTART;TZID={tz}:{start_at}\r\nDTEND;TZID={tz}:{end_at}\r\nSUMMARY:{title}\r\nDESCRIPTION:{desc}\r\nLOCATION:{loc}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

/// Export event as markdown
pub fn export_event_md(
    title: &str,
    description: Option<&str>,
    location: Option<&str>,
    start_at: &str,
    end_at: &str,
    participants: &[String],
) -> String {
    let mut md = format!("# {title}\n\n**When:** {start_at} - {end_at}\n\n");
    if let Some(loc) = location
        && !loc.is_empty()
    {
        md.push_str(&format!("**Where:** {loc}\n\n"));
    }
    if let Some(desc) = description
        && !desc.is_empty()
    {
        md.push_str(&format!("{desc}\n\n"));
    }
    if !participants.is_empty() {
        md.push_str("## Participants\n\n");
        for p in participants {
            md.push_str(&format!("- {p}\n"));
        }
    }
    md
}

#[derive(Debug, Error)]
pub enum CalendarSyncError {
    #[error("sync failed: {0}")]
    SyncFailed(String),
    #[error("import failed: {0}")]
    ImportFailed(String),
}
