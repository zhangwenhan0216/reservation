fn main() {
  tonic_build::configure()
    .out_dir("src/pb")
    .type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]")
    .compile_protos(&["protos/reservation.proto"], &["protos"])
    .unwrap();

  println!("cargo:rerun-if-changed=protos/reservation.proto");
}
