use crate::{ReservationManage, Rsvp};
use abi::{Error, ReservationId, ReservationStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, PgPool, Row};

#[async_trait]
impl Rsvp for ReservationManage {
  async fn reserve(
    &self,
    mut rsvp: abi::Reservation,
  ) -> Result<abi::Reservation, Error> {
    rsvp.validate()?;

    let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();

    let status = ReservationStatus::try_from(rsvp.status)
      .unwrap_or(ReservationStatus::Pending);

    let id: i64 = sqlx::query(
      "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1,$2,$3,$4,$5::rsvp.reservation_status) RETURNING id",
    )
    .bind(rsvp.user_id.clone())
    .bind(rsvp.resource_id.clone())
    .bind(timespan)
    .bind(rsvp.note.clone())
    .bind(status.to_string())
    .fetch_one(&self.pool)
    .await?
    .get("id");

    rsvp.id = id;

    Ok(rsvp)
  }

  async fn change_status(
    &self,
    id: ReservationId,
  ) -> Result<abi::Reservation, Error> {
    let rsvp: abi::Reservation = sqlx::query_as("UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1 AND STATUS = 'pending' RETURNING *")
    .bind(id).fetch_one(&self.pool).await?;

    Ok(rsvp.into())
  }

  async fn update_note(
    &self,
    id: ReservationId,
    note: String,
  ) -> Result<abi::Reservation, Error> {
    let rsvp: abi::Reservation = sqlx::query_as(
      "UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *",
    )
    .bind(note)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(rsvp.into())
  }

  async fn get(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
    let rsvp: abi::Reservation =
      sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1")
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

    Ok(rsvp.into())
  }

  async fn delete(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
    let rsvp: abi::Reservation =
      sqlx::query_as("DELETE FROM rsvp.reservations WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

    Ok(rsvp.into())
  }

  async fn query(
    &self,
    query: abi::ReservationQuery,
  ) -> Result<Vec<abi::Reservation>, Error> {
    let user_id = str_to_option(&query.user_id);
    let resource_id = str_to_option(&query.resource_id);
    let status = ReservationStatus::try_from(query.status)
      .unwrap_or(ReservationStatus::Pending);

    let rsvps =
      sqlx::query_as("SELECT * FROM rsvp.query($1, $2, $3, $4, $5, $6, $7)")
        .bind(user_id)
        .bind(resource_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

    Ok(rsvps)
  }
}

fn str_to_option(s: &str) -> Option<&str> {
  if s.is_empty() {
    None
  } else {
    Some(s)
  }
}

impl ReservationManage {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use abi::{
    convert_local_time_to_utc, Reservation, ReservationConflict,
    ReservationConflictInfo, ReservationWindow,
  };
  #[sqlx_database_tester::test(pool(
    variable = "migrated_pool",
    migrations = "../migrations"
  ))]
  async fn reserve_should_work_for_valid_window() {
    let pool = ReservationManage::new(migrated_pool);

    let rsvp = Reservation::new_pending(
      "xiaozhangId",
      "testResourceId",
      convert_local_time_to_utc("2024-01-21 19:00:00"),
      convert_local_time_to_utc("2024-01-22 12:00:00"),
      "test_reserve_should_work_for_valid_window",
    );

    let rsvp_new = pool.reserve(rsvp).await.unwrap();

    println!("{:?}", rsvp_new);

    assert!(rsvp_new.id != 0);
  }

  #[sqlx_database_tester::test(pool(
    variable = "migrated_pool",
    migrations = "../migrations"
  ))]
  async fn reserve_conflict_reservation_should_reject() {
    let pool = ReservationManage::new(migrated_pool);

    let rsvp = Reservation::new_pending(
      "xiaozhangId",
      "ocean-view-room-713",
      convert_local_time_to_utc("2024-01-21 19:00:00"),
      convert_local_time_to_utc("2024-01-22 12:00:00"),
      "test_reserve_should_work_for_valid_window",
    );

    let rsvp1 = Reservation::new_pending(
      "xiaonanId",
      "ocean-view-room-713",
      convert_local_time_to_utc("2024-01-22 8:00:00"),
      convert_local_time_to_utc("2024-01-23 12:00:00"),
      "test_reserve_should_work_for_valid_window",
    );

    let _rsvp1 = pool.reserve(rsvp).await.unwrap();
    let reserve_conflict = pool.reserve(rsvp1).await.unwrap_err();

    let info = ReservationConflictInfo::Parsed(ReservationConflict {
      new: ReservationWindow {
        rid: "ocean-view-room-713".to_string(),
        start: convert_local_time_to_utc("2024-01-21 19:00:00"),
        end: convert_local_time_to_utc("2024-01-22 12:00:00"),
      },
      old: ReservationWindow {
        rid: "ocean-view-room-713".to_string(),
        start: convert_local_time_to_utc("2024-01-22 8:00:00"),
        end: convert_local_time_to_utc("2024-01-23 12:00:00"),
      },
    });

    assert_eq!(reserve_conflict, Error::ConflictReservation(info))
  }

  #[sqlx_database_tester::test(pool(
    variable = "migrated_pool",
    migrations = "../migrations"
  ))]
  async fn reserve_change_status_reservation_should_work() {
    let pool = ReservationManage::new(migrated_pool);

    // test change status
    let rsvp = Reservation::new_pending(
      "testUserId",
      "testResourceId",
      convert_local_time_to_utc("2024-01-21 19:00:00"),
      convert_local_time_to_utc("2024-01-22 12:00:00"),
      "test_change_status_reservation_should_work",
    );

    let rsvp = pool.reserve(rsvp).await.unwrap();

    // 将状态从Pending改为Confirmed
    let updated_rsvp = pool.change_status(rsvp.id).await.unwrap();

    assert_eq!(updated_rsvp.status, ReservationStatus::Confirmed as i32);
  }

  #[sqlx_database_tester::test(pool(
    variable = "migrated_pool",
    migrations = "../migrations"
  ))]
  async fn reserve_change_status_not_pending_should_donothing() {
    let pool = ReservationManage::new(migrated_pool);

    // test change status
    let rsvp = Reservation::new_pending(
      "testUserId",
      "testResourceId",
      convert_local_time_to_utc("2024-01-21 19:00:00"),
      convert_local_time_to_utc("2024-01-22 12:00:00"),
      "test_change_status_not_pending_should_donothing",
    );

    let rsvp = pool.reserve(rsvp).await.unwrap();
    let rsvp = pool.change_status(rsvp.id).await.unwrap();

    let ret = pool.change_status(rsvp.id).await;

    assert_eq!(ret, Err(Error::NotFound))
  }

  #[sqlx_database_tester::test(pool(
    variable = "migrated_pool",
    migrations = "../migrations"
  ))]
  async fn reserve_update_note_reservation_should_work() {
    let pool = ReservationManage::new(migrated_pool);

    let rsvp = Reservation::new_pending(
      "testUserId",
      "testResourceId",
      convert_local_time_to_utc("2024-01-21 19:00:00"),
      convert_local_time_to_utc("2024-01-22 12:00:00"),
      "",
    );

    let rsvp = pool.reserve(rsvp).await.unwrap();
    let updated_rsvp = pool
      .update_note(
        rsvp.id,
        "test_update_note_reservation_should_work".to_string(),
      )
      .await
      .unwrap();
    assert_eq!(
      updated_rsvp.note,
      "test_update_note_reservation_should_work"
    )
  }

  #[sqlx_database_tester::test(pool(
    variable = "migrated_pool",
    migrations = "../migrations"
  ))]
  async fn reserve_get_reservation_should_work() {
    let pool = ReservationManage::new(migrated_pool);

    let rsvp = Reservation::new_pending(
      "testUserId",
      "testResourceId",
      convert_local_time_to_utc("2024-01-21 19:00:00"),
      convert_local_time_to_utc("2024-01-22 12:00:00"),
      "",
    );

    let rsvp = pool.reserve(rsvp).await.unwrap();
    let rsvp1 = pool.get(rsvp.id.clone()).await.unwrap();

    assert_eq!(rsvp1, rsvp)
  }

  #[sqlx_database_tester::test(pool(
    variable = "migrated_pool",
    migrations = "../migrations"
  ))]
  async fn reserve_delete_reservation_should_work() {
    let pool = ReservationManage::new(migrated_pool);

    let rsvp = Reservation::new_pending(
      "testUserId",
      "testResourceId",
      convert_local_time_to_utc("2024-01-21 19:00:00"),
      convert_local_time_to_utc("2024-01-22 12:00:00"),
      "",
    );

    let rsvp = pool.reserve(rsvp).await.unwrap();
    let rsvp1 = pool.delete(rsvp.id.clone()).await.unwrap();

    let ret = pool.get(rsvp1.id.clone()).await;
    assert_eq!(ret, Err(Error::NotFound))
  }
}
