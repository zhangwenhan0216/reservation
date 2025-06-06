mod conflict;
pub use conflict::{
  ReservationConflict, ReservationConflictInfo, ReservationWindow,
};
use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("time is error")]
  InvalidTime,

  #[error("user_id is invalid, user_id={0}")]
  InvalidUserId(String),

  #[error("resource_id is invalid, user_id={0}")]
  InvalidResourceId(String),

  #[error("sqlx query error")]
  DbError(sqlx::Error),

  #[error("conflict reservation")]
  ConflictReservation(ReservationConflictInfo),

  #[error("no reservation found by the given condition")]
  NotFound,

  #[error("unknown error")]
  Unknown,
}

impl PartialEq for Error {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Error::DbError(_), Error::DbError(_)) => true,
      (Error::InvalidTime, Error::InvalidTime) => true,
      (Error::ConflictReservation(_), Error::ConflictReservation(_)) => true,
      (Error::NotFound, Error::NotFound) => true,
      (Error::InvalidUserId(_), Error::InvalidUserId(_)) => true,
      (Error::InvalidResourceId(_), Error::InvalidResourceId(_)) => true,
      (Error::Unknown, Error::Unknown) => true,
      _ => false,
    }
  }
}

impl From<sqlx::Error> for Error {
  fn from(e: sqlx::Error) -> Self {
    match e {
      sqlx::Error::Database(e) => {
        let err: &PgDatabaseError = e.downcast_ref();
        match (err.code(), err.schema(), err.table()) {
          ("23P01", Some("rsvp"), Some("reservations")) => {
            Error::ConflictReservation(
              err
                .detail()
                .unwrap()
                .parse::<ReservationConflictInfo>()
                .unwrap(),
            )
          }
          _ => Error::DbError(sqlx::Error::Database(e)),
        }
      }
      sqlx::Error::RowNotFound => Error::NotFound,
      _ => Error::DbError(e),
    }
  }
}
