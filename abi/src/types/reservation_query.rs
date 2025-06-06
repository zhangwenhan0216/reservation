use sqlx::{
  postgres::types::PgRange,
  types::chrono::{DateTime, Utc},
};

use crate::{types::get_timespan, ReservationQuery};

impl ReservationQuery {
  pub fn new(user_id: &str, resource_id: &str) -> Self {
    todo!()
  }

  pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
    get_timespan(self.start.as_ref(), self.end.as_ref())
  }
}
