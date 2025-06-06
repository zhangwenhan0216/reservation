use prost_types::Timestamp;
use sqlx::types::chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};

/// conversion
pub fn convert_to_utc_time(ts: Timestamp) -> DateTime<Utc> {
  DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as _).unwrap()
}

pub fn convert_to_timestamp(time: DateTime<Utc>) -> Timestamp {
  Timestamp {
    seconds: time.timestamp(),
    nanos: time.timestamp_subsec_nanos() as _,
  }
}

pub fn convert_local_time_to_utc(time: &str) -> DateTime<Utc> {
  let native_time =
    NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S").unwrap();
  Local
    .from_local_datetime(&native_time)
    .single()
    .expect("convert_local_time_to_utc error")
    .with_timezone(&Utc)
}
