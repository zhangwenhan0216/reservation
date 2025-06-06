use std::ops::{Bound, Range};

use sqlx::{
  postgres::{types::PgRange, PgRow},
  types::chrono::{DateTime, Utc},
  FromRow, Row,
};

use crate::{
  utils::{convert_to_timestamp, convert_to_utc_time},
  Error, Reservation, ReservationStatus, RsvpStatus,
};

impl Reservation {
  pub fn new_pending(
    user_id: impl Into<String>,
    resource_id: impl Into<String>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    note: impl Into<String>,
  ) -> Self {
    Self {
      id: 0,
      user_id: user_id.into(),
      resource_id: resource_id.into(),
      start: Some(convert_to_timestamp(start)),
      end: Some(convert_to_timestamp(end)),
      note: note.into(),
      status: ReservationStatus::Pending as i32,
    }
  }
  pub fn validate(&self) -> Result<(), Error> {
    if self.user_id.is_empty() {
      return Err(Error::InvalidUserId(self.user_id.clone()));
    }

    if self.resource_id.is_empty() {
      return Err(Error::InvalidResourceId(self.resource_id.clone()));
    }

    if self.start.is_none() || self.end.is_none() {
      return Err(Error::InvalidTime);
    }

    let start = self.start.unwrap();
    let end = self.end.unwrap();

    if start.seconds >= end.seconds {
      return Err(Error::InvalidTime);
    }
    Ok(())
  }

  pub fn get_timespan(&self) -> Range<DateTime<Utc>> {
    let start = convert_to_utc_time(self.start.unwrap().clone());
    let end = convert_to_utc_time(self.end.unwrap().clone());

    Range { start, end }
  }
}

struct NaiveRange<T> {
  start: Option<T>,
  end: Option<T>,
}

impl<T> From<PgRange<T>> for NaiveRange<T> {
  fn from(range: PgRange<T>) -> Self {
    let f = |r| match r {
      Bound::Included(v) => Some(v),
      Bound::Excluded(v) => Some(v),
      Bound::Unbounded => None,
    };
    Self {
      start: f(range.start),
      end: f(range.end),
    }
  }
}

impl FromRow<'_, PgRow> for Reservation {
  fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
    let timespan: PgRange<DateTime<Utc>> = row.get("timespan");
    let NaiveRange { start, end } = timespan.into();

    // real time range should be included
    assert!(start.is_some() && end.is_some());

    let status: RsvpStatus = row.get("status");

    Ok(Self {
      id: row.get("id"),
      user_id: row.get("user_id"),
      resource_id: row.get("resource_id"),
      start: Some(convert_to_timestamp(start.unwrap())),
      end: Some(convert_to_timestamp(end.unwrap())),
      status: ReservationStatus::from(status) as i32,
      note: row.get("note"),
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::utils::convert_local_time_to_utc;

  use super::*;

  #[test]
  fn test_reservation_validate_fn() -> Result<(), Error> {
    let rsvp = Reservation::new_pending(
      "user_id",
      "resource_id",
      convert_local_time_to_utc("2024-01-21 19:00:00"),
      convert_local_time_to_utc("2024-01-22 12:00:00"),
      "",
    );
    rsvp.validate()?;

    Ok(())
  }
}
