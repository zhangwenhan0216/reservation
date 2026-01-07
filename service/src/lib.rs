mod service;
use std::pin::Pin;

use abi::{Config, Reservation};
use futures::Stream;
use reservation::ReservationManage;
use tonic::Status;

pub struct RsvpServie {
  pub manager: reservation::ReservationManage,
}
impl RsvpServie {
  pub async fn from_config(config: &Config) -> Result<Self, abi::Error> {
    let pool = ReservationManage::from_config(&config.db).await?;
    Ok(Self {
      manager: ReservationManage::new(pool),
    })
  }
}

type ReservationStream =
  Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;
