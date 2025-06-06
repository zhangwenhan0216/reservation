mod manage;
use abi::Error;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct ReservationManage {
  pool: PgPool,
}

#[async_trait]
pub trait Rsvp {
  async fn reserve(
    &self,
    rsvp: abi::Reservation,
  ) -> Result<abi::Reservation, Error>;
  async fn change_status(
    &self,
    id: abi::ReservationId,
  ) -> Result<abi::Reservation, Error>;
  async fn update_note(
    &self,
    id: abi::ReservationId,
    note: String,
  ) -> Result<abi::Reservation, Error>;
  async fn get(
    &self,
    id: abi::ReservationId,
  ) -> Result<abi::Reservation, Error>;
  async fn delete(
    &self,
    id: abi::ReservationId,
  ) -> Result<abi::Reservation, Error>;
  async fn query(
    &self,
    query: abi::ReservationQuery,
  ) -> Result<Vec<abi::Reservation>, Error>;
}
