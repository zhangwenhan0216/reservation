use abi::{
  reservation_service_server::ReservationService, CancelRequest,
  CancelResponse, ConfirmRequest, ConfirmResponse, FilterRequest,
  FilterResponse, GetRequest, GetResponse, QueryRequest, ReserveRequest,
  ReserveResponse, UpdateRequest, UpdateResponse,
};
use reservation::Rsvp;
use tonic::{Request, Response, Status};

use crate::{ReservationStream, RsvpServie};

#[tonic::async_trait]
impl ReservationService for RsvpServie {
  async fn reserve(
    &self,
    request: Request<ReserveRequest>,
  ) -> Result<Response<ReserveResponse>, Status> {
    let request = request.into_inner();
    if request.reservation.is_none() {
      return Err(Status::invalid_argument("reservation is required"));
    }
    let reservation =
      self.manager.reserve(request.reservation.unwrap()).await?;

    Ok(Response::new(ReserveResponse {
      reservation: Some(reservation),
    }))
  }
  /// update status to CONFIRMED
  async fn confirm(
    &self,
    _request: Request<ConfirmRequest>,
  ) -> Result<Response<ConfirmResponse>, Status> {
    todo!()
  }
  /// update only note
  async fn update(
    &self,
    _request: Request<UpdateRequest>,
  ) -> Result<Response<UpdateResponse>, Status> {
    todo!()
  }
  /// cancel reservation
  async fn cancel(
    &self,
    _request: Request<CancelRequest>,
  ) -> Result<Response<CancelResponse>, Status> {
    todo!()
  }
  /// get reservation by id
  async fn get(
    &self,
    _request: Request<GetRequest>,
  ) -> Result<Response<GetResponse>, Status> {
    todo!()
  }
  /// Server streaming response type for the query method.
  type queryStream = ReservationStream;
  /// query reservations with pagination
  async fn query(
    &self,
    _request: Request<QueryRequest>,
  ) -> Result<Response<Self::queryStream>, Status> {
    todo!()
  }
  /// filter reservations order by reservation id
  async fn filter(
    &self,
    _request: Request<FilterRequest>,
  ) -> Result<Response<FilterResponse>, Status> {
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use std::ops::Deref;

  use super::*;
  use abi::{convert_local_time_to_utc, Config, Reservation};
  use sqlx_db_tester::TestDb;

  struct TestConfig {
    pub _tdb: TestDb,
    pub config: Config,
  }
  impl Deref for TestConfig {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
      &self.config
    }
  }
  impl TestConfig {
    fn new() -> Self {
      let mut config = Config::from_file("../service/fixtures/config.yml")
        .expect("failed to read config file");

      let test_db = TestDb::new(
        &config.db.host,
        config.db.port,
        &config.db.username,
        &config.db.password,
        "../migrations",
      );

      config.db.database = test_db.database.clone();

      Self {
        _tdb: test_db,
        config,
      }
    }
  }
  #[tokio::test]
  async fn rpc_reserve_should_work() {
    let config = TestConfig::new();
    let service = RsvpServie::from_config(&config).await.unwrap();

    let request = Request::new(ReserveRequest {
      reservation: Some(Reservation::new_pending(
        "xiaozhangId",
        "testResourceId",
        convert_local_time_to_utc("2024-01-21 19:00:00"),
        convert_local_time_to_utc("2024-01-22 12:00:00"),
        "test_reserve_should_work_for_valid_window",
      )),
    });

    let response = service.reserve(request).await.unwrap().into_inner();

    let reservation_res = response.reservation.unwrap();
    assert_eq!(reservation_res.user_id, "xiaozhangId");
    assert_eq!(reservation_res.resource_id, "testResourceId");
    assert_eq!(
      reservation_res.start.unwrap(),
      "2024-01-21T11:00:00.000Z".parse().unwrap()
    );
    assert_eq!(
      reservation_res.end.unwrap(),
      "2024-01-22T04:00:00.000Z".parse().unwrap()
    );
    assert_eq!(
      reservation_res.note,
      "test_reserve_should_work_for_valid_window"
    );
  }
}
