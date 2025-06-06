use lazy_static::lazy_static;
use regex::Regex;
use sqlx::types::chrono::{DateTime, Utc};
use std::{collections::HashMap, convert::Infallible, str::FromStr};

lazy_static! {
  static ref REGEX: Regex = Regex::new(
      r"\((?<k1>[a-zA-Z0-9_-]+)\s*,\s*(?<k2>[a-zZ-Z0-9_-]+)\)=\((?<v1>[a-zA-Z0-9_-]+)\s*,\s*[\[\(](?<v2>[^\)\]]+)[\]\)]\)",
    ).unwrap();
}

#[derive(Debug, Clone)]
pub enum ReservationConflictInfo {
  Parsed(ReservationConflict),
  UnParsed,
}

#[derive(Debug, Clone)]
pub struct ReservationConflict {
  pub new: ReservationWindow,
  pub old: ReservationWindow,
}

#[derive(Debug, Clone)]
pub struct ReservationWindow {
  pub rid: String,
  pub start: DateTime<Utc>,
  pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
  type Err = Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Ok(conflict) = s.parse() {
      Ok(ReservationConflictInfo::Parsed(conflict))
    } else {
      Ok(ReservationConflictInfo::UnParsed)
    }
  }
}

impl FromStr for ReservationConflict {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    ParsedInfo::from_str(s)?.try_into()
  }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
  type Error = ();
  fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
    Ok(Self {
      new: value.new.try_into()?,
      old: value.old.try_into()?,
    })
  }
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
  type Error = ();
  fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
    let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");

    let (start, end) = parse_timespan(&timespan_str)?;

    Ok(Self {
      rid: value.get("resource_id").ok_or(())?.to_string(),
      start,
      end,
    })
  }
}

struct ParsedInfo {
  new: HashMap<String, String>,
  old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
  type Err = ();
  //"Key (resource_id, timespan)=(ocean-view-room-713, [\"2024-01-22 00:00:00+00\",\"2024-01-23 04:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2024-01-21 11:00:00+00\",\"2024-01-22 04:00:00+00\"))."
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut caps_iter = REGEX.captures_iter(s);
    let cap_new = caps_iter.next().ok_or(())?;
    let cap_old = caps_iter.next().ok_or(())?;

    Ok(Self {
      new: HashMap::from([
        (cap_new["k1"].to_string(), cap_new["v1"].to_string()),
        (cap_new["k2"].to_string(), cap_new["v2"].to_string()),
      ]),
      old: HashMap::from([
        (cap_old["k1"].to_string(), cap_old["v1"].to_string()),
        (cap_old["k2"].to_string(), cap_old["v2"].to_string()),
      ]),
    })
  }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
  Ok(
    DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
      .map_err(|_| ())?
      .with_timezone(&Utc),
  )
}
fn parse_timespan(s: &str) -> Result<(DateTime<Utc>, DateTime<Utc>), ()> {
  let mut split_str = s.splitn(2, ',');

  let start = parse_datetime(split_str.next().ok_or(())?)?;
  let end = parse_datetime(split_str.next().ok_or(())?)?;

  Ok((start, end))
}

#[cfg(test)]
mod tests {
  use super::*;
  const ERR_MSG: &str = "Key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).";
  #[test]
  fn parse_datetime_should_work() {
    let datetime = parse_datetime("2022-12-26 22:00:00+00").unwrap();
    assert_eq!(datetime.to_rfc3339(), "2022-12-26T22:00:00+00:00")
  }

  #[test]
  fn parsed_info_should_work() {
    let info: ParsedInfo = ERR_MSG.parse().unwrap();
    println!("{:?}", info.new);
    println!("{:?}", info.old);
    assert_eq!(info.new["resource_id"], "ocean-view-room-713");
    assert_eq!(
      info.new["timespan"],
      "\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\""
    );
    assert_eq!(info.old["resource_id"], "ocean-view-room-713");
    assert_eq!(
      info.old["timespan"],
      "\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\""
    );
  }

  #[test]
  fn hash_map_to_reservation_window_should_work() {
    let mut map = HashMap::new();
    map.insert("resource_id".to_string(), "ocean-view-room-713".to_string());
    map.insert(
      "timespan".to_string(),
      "\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"".to_string(),
    );
    let window: ReservationWindow = map.try_into().unwrap();
    assert_eq!(window.rid, "ocean-view-room-713");
    assert_eq!(window.start.to_rfc3339(), "2022-12-26T22:00:00+00:00");
    assert_eq!(window.end.to_rfc3339(), "2022-12-30T19:00:00+00:00");
  }
  #[test]
  fn conflict_error_message_should_parse() {
    let info: ReservationConflictInfo = ERR_MSG.parse().unwrap();
    match info {
      ReservationConflictInfo::Parsed(conflict) => {
        assert_eq!(conflict.new.rid, "ocean-view-room-713");
        assert_eq!(
          conflict.new.start.to_rfc3339(),
          "2022-12-26T22:00:00+00:00"
        );
        assert_eq!(conflict.new.end.to_rfc3339(), "2022-12-30T19:00:00+00:00");
        assert_eq!(conflict.old.rid, "ocean-view-room-713");
        assert_eq!(
          conflict.old.start.to_rfc3339(),
          "2022-12-25T22:00:00+00:00"
        );
        assert_eq!(conflict.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
      }
      ReservationConflictInfo::UnParsed => panic!("should have parsed"),
    }
  }
}
