/*
 * Kestrel - a modern instant-messaging service written in Rust
 * Copyright (C) 2026 Kestrel Chat
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use rocket::futures::StreamExt;
use rocket_ws::WebSocket;

use crate::{gateway::router, protocol::packet::Packet};

pub fn gateway(ws: WebSocket) -> rocket_ws::Channel<'static> {
    ws.channel(|mut stream| {
        Box::pin(async move {
            while let Some(message) = stream.next().await {
                let msg = match message {
                    Ok(m) => m,
                    Err(_) => break,
                };

                if let rocket_ws::Message::Text(text) = msg
                    && let Ok(packet) = serde_json::from_str::<Packet>(&text)
                {
                    router::route(packet).await;
                }
            }

            Ok(())
        })
    })
}
