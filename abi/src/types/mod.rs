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

use crate::{convert_to_utc_time, Error, Validator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
  Unkonwn,
  Confirmed,
  Pending,
  Blocked,
}

fn validate_range(
  start: Option<&Timestamp>,
  end: Option<&Timestamp>,
) -> Result<(), Error> {
  if start.is_none() || end.is_none() {
    return Err(Error::InvalidTime);
  }

  let start = start.unwrap();
  let end = end.unwrap();

  if start.seconds >= end.seconds {
    return Err(Error::InvalidTime);
  }
  Ok(())
}

fn get_timespan(
  start: Option<&Timestamp>,
  end: Option<&Timestamp>,
) -> PgRange<DateTime<Utc>> {
  let start = convert_to_utc_time(*start.unwrap());
  let end = convert_to_utc_time(*end.unwrap());

  PgRange {
    start: Bound::Included(start),
    end: Bound::Excluded(end),
  }
}

impl Validator for ReservationId {
  fn validate(&self) -> Result<(), Error> {
    if *self <= 0 {
      return Err(Error::InvalidReservationId(*self));
    }
    Ok(())
  }
}
