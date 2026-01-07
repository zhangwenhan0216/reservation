fn main() {
  tonic_build::configure()
    .out_dir("src/pb")
    .with_sql_type(&["reservation.ReservationStatus"])
    .with_builder(&[
      "reservation.ReservationQuery",
      "reservation.ReservationFilter",
    ])
    .with_bulider_into(
      "reservation.ReservationQuery",
      &[
        "user_id",
        "resource_id",
        "status",
        "page",
        "page_size",
        "desc",
      ],
    )
    .with_bulider_into(
      "reservation.ReservationFilter",
      &[
        "user_id",
        "resource_id",
        "status",
        "cursor",
        "page_size",
        "desc",
      ],
    )
    .with_builder_option("reservation.ReservationQuery", &["start", "end"])
    .compile_protos(&["protos/reservation.proto"], &["protos"])
    .unwrap();

  println!("cargo:rerun-if-changed=protos/reservation.proto");
}

trait BuilderExt {
  fn with_sql_type(self, paths: &[&str]) -> Self;
  fn with_builder(self, paths: &[&str]) -> Self;
  fn with_bulider_into(self, path: &str, fields: &[&str]) -> Self;
  fn with_builder_option(self, path: &str, fields: &[&str]) -> Self;
}

impl BuilderExt for tonic_build::Builder {
  fn with_sql_type(self, paths: &[&str]) -> Self {
    paths.iter().fold(self, |builder, path| {
      builder.type_attribute(path, "#[derive(sqlx::Type)]")
    })
  }

  fn with_builder(self, paths: &[&str]) -> Self {
    paths.iter().fold(self, |builder, path| {
      builder.type_attribute(path, "#[derive(derive_builder::Builder)]")
    })
  }

  fn with_bulider_into(self, path: &str, fields: &[&str]) -> Self {
    fields.iter().fold(self, |builder, field| {
      builder.field_attribute(
        format!("{}.{}", path, field),
        "#[builder(setter(into), default)]",
      )
    })
  }

  fn with_builder_option(self, path: &str, fields: &[&str]) -> Self {
    fields.iter().fold(self, |builder, field| {
      builder.field_attribute(
        format!("{}.{}", path, field),
        "#[builder(setter(into, strip_option))]",
      )
    })
  }
}
