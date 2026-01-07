mod conflict;
pub use conflict::{
  ReservationConflict, ReservationConflictInfo, ReservationWindow,
};
use sqlx::postgres::PgDatabaseError;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum Error {
  #[error("time is error")]
  InvalidTime,

  #[error("Failed to read configuration file")]
  ConfigReadError,

  #[error("Failed to parse configuration file")]
  ConfigParseError,

  #[error("id is invalid, id={0}")]
  InvalidReservationId(i64),

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
    matches!(
      (self, other),
      (Error::DbError(_), Error::DbError(_))
        | (Error::InvalidTime, Error::InvalidTime)
        | (Error::ConflictReservation(_), Error::ConflictReservation(_))
        | (Error::NotFound, Error::NotFound)
        | (Error::InvalidUserId(_), Error::InvalidUserId(_))
        | (Error::InvalidResourceId(_), Error::InvalidResourceId(_))
        | (Error::Unknown, Error::Unknown)
    )
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

impl From<Error> for Status {
  fn from(value: Error) -> Self {
    match value {
      Error::InvalidTime => Status::invalid_argument(value.to_string()),
      Error::ConfigReadError | Error::ConfigParseError => {
        Status::internal(value.to_string())
      }
      Error::InvalidReservationId(id) => {
        Status::invalid_argument(format!("Invalid reservation id: {}", id))
      }
      Error::InvalidUserId(user_id) => {
        Status::invalid_argument(format!("Invalid user id: {}", user_id))
      }
      Error::InvalidResourceId(resource_id) => Status::invalid_argument(
        format!("Invalid resource id: {}", resource_id),
      ),
      Error::DbError(e) => Status::internal(e.to_string()),
      Error::ConflictReservation(info) => {
        Status::already_exists(format!("Conflict reservation: {:?}", info))
      }
      Error::NotFound => Status::not_found(value.to_string()),
      Error::Unknown => Status::unknown(value.to_string()),
    }
  }
}
