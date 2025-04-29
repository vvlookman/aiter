use std::sync::LazyLock;

use time::{
    format_description::{well_known::Rfc3339, BorrowedFormatItem},
    macros::format_description,
    OffsetDateTime, PrimitiveDateTime, UtcOffset,
};

static LOCAL_OFFSET: LazyLock<UtcOffset> =
    LazyLock::new(|| UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
static DATETIME_FORMAT_DESC: LazyLock<&[BorrowedFormatItem]> =
    LazyLock::new(|| format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));

pub fn iso_to_local_datetime_string(iso_str: &str) -> String {
    match PrimitiveDateTime::parse(iso_str, &Rfc3339) {
        Ok(datetime) => datetime
            .assume_utc()
            .to_offset(*LOCAL_OFFSET)
            .format(&DATETIME_FORMAT_DESC)
            .unwrap_or(iso_str.to_string()),
        Err(_) => iso_str.to_string(),
    }
}

pub fn now_iso_datetime_string() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default()
}

pub fn now_local_datetime_string() -> String {
    format_to_local(&OffsetDateTime::now_utc())
}

pub fn utc_to_iso_datetime_string(utc_str: &str) -> String {
    match PrimitiveDateTime::parse(utc_str, &DATETIME_FORMAT_DESC) {
        Ok(datetime) => datetime
            .assume_utc()
            .format(&Rfc3339)
            .unwrap_or(utc_str.to_string()),
        Err(_) => utc_str.to_string(),
    }
}

fn format_to_local(time: &OffsetDateTime) -> String {
    time.to_offset(*LOCAL_OFFSET)
        .format(*DATETIME_FORMAT_DESC)
        .unwrap_or(time.to_string())
}
