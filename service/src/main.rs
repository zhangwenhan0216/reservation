use std::path::Path;

use abi::{reservation_service_server::ReservationServiceServer, Config};
use reservation_service::RsvpServie;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // we would first try RESERVATION_CONFIG envar, then try "./reservation.yml" then try "~/config/reservation.yml", then try "etc/reservation.yml"

  let filename = std::env::var("RESERVATION_CONFIG").unwrap_or_else(|_| {
    let path = "./reservation.yml";
    if Path::new(path).exists() {
      return path.to_string();
    }

    let path = shellexpand::tilde("~/.config/reservation.yml").to_string();
    if Path::new(&path).exists() {
      return path.to_string();
    }

    "/etc/reservation.yml".to_string()
  });

  let config = Config::from_file(&filename)?;

  let addr =
    format!("{}:{}", config.server.host, config.server.port).parse()?;

  let rsvp_service = RsvpServie::from_config(&config).await?;

  println!("Server listening on {}", addr);

  tonic::transport::Server::builder()
    .add_service(ReservationServiceServer::new(rsvp_service))
    .serve(addr)
    .await?;

  Ok(())
}
