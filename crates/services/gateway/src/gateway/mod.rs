pub mod connection;
pub mod router;

use rocket_ws::WebSocket;

#[get("/")]
pub fn gateway_route(ws: WebSocket) -> rocket_ws::Channel<'static> {
  connection::gateway(ws)
}
