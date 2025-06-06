use std::fmt::Display;

use crate::ReservationStatus;

use super::RsvpStatus;

impl Display for ReservationStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ReservationStatus::Unknown => write!(f, "unkonwn"),
      ReservationStatus::Pending => write!(f, "pending"),
      ReservationStatus::Confirmed => write!(f, "confirmed"),
      ReservationStatus::Blocked => write!(f, "blocked"),
    }
  }
}

impl From<RsvpStatus> for ReservationStatus {
  fn from(value: RsvpStatus) -> Self {
    match value {
      RsvpStatus::Unkonwn => ReservationStatus::Unknown,
      RsvpStatus::Confirmed => ReservationStatus::Confirmed,
      RsvpStatus::Pending => ReservationStatus::Pending,
      RsvpStatus::Blocked => ReservationStatus::Blocked,
    }
  }
}
