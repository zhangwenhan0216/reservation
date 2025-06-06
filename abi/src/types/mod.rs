mod reservation;
mod reservation_query;
mod reservation_status;

pub type ReservationId = i64;
use std::ops::Bound;

use prost_types::Timestamp;
use sqlx::{
  postgres::types::PgRange,
  types::{
    chrono::{DateTime, Utc},
    Type,
  },
};

use crate::convert_to_utc_time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
  Unkonwn,
  Confirmed,
  Pending,
  Blocked,
}

fn get_timespan(
  start: Option<&Timestamp>,
  end: Option<&Timestamp>,
) -> PgRange<DateTime<Utc>> {
  let start = convert_to_utc_time(start.unwrap().clone());
  let end = convert_to_utc_time(end.unwrap().clone());

  PgRange {
    start: Bound::Included(start),
    end: Bound::Excluded(end),
  }
}
