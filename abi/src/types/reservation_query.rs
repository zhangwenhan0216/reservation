use sqlx::{
  postgres::types::PgRange,
  types::chrono::{DateTime, Utc},
};

use crate::{
  types::{get_timespan, validate_range},
  ReservationQuery, Validator,
};

impl ReservationQuery {
  pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
    get_timespan(self.start.as_ref(), self.end.as_ref())
  }
}

impl Validator for ReservationQuery {
  fn validate(&self) -> Result<(), crate::Error> {
    validate_range(self.start.as_ref(), self.end.as_ref())?;

    Ok(())
  }
}
