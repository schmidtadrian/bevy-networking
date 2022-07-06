use bevy::{prelude::*, reflect::Uuid};
use serde::{Deserialize, Serialize};

use bevy_spicy_networking::{ClientMessage, NetworkMessage, ServerMessage};

/////////////////////////////////////////////////////////////////////
// In this example the client sends `UserChatMessage`s to the server,
// the server then broadcasts to all connected clients.
//
// We use two different types here, because only the server should
// decide the identity of a given connection and thus also sends a
// name.
//
// You can have a single message be sent both ways, it simply needs
// to implement both `ClientMessage` and `ServerMessage`.
/////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserChatMessage {
    pub message: String,
}

#[typetag::serde]
impl NetworkMessage for UserChatMessage {}

impl ServerMessage for UserChatMessage {
    const NAME: &'static str = "example:UserChatMessage";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewChatMessage {
    pub name: String,
    pub message: String,
}

#[typetag::serde]
impl NetworkMessage for NewChatMessage {}

impl ClientMessage for NewChatMessage {
    const NAME: &'static str = "example:NewChatMessage";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Spawn {
    pub id: Uuid,
    pub x: f32,
    pub y: f32
}

#[typetag::serde]
impl NetworkMessage for Spawn{}

impl ClientMessage for Spawn {
    const NAME: &'static str = "Spawn";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Connected {
    pub id: Uuid
}

#[typetag::serde]
impl NetworkMessage for Connected{}

impl ClientMessage for Connected {
    const NAME: &'static str = "Connected";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub id: Uuid,
    pub x: f32,
    pub y: f32
}

#[typetag::serde]
impl NetworkMessage for Position{}

impl ClientMessage for Position {
    const NAME: &'static str = "Position";
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Actions {
    pub w: i16,
    pub a: i16,
    pub s: i16,
    pub d: i16
}

#[typetag::serde]
impl NetworkMessage for Actions{}

impl ServerMessage for Actions {
    const NAME: &'static str = "Action";
}


#[allow(unused)]
pub fn client_register_network_messages(app: &mut App) {
    use bevy_spicy_networking::AppNetworkClientMessage;

    // The client registers messages that arrives from the server, so that
    // it is prepared to handle them. Otherwise, an error occurs.
    &app.listen_for_client_message::<NewChatMessage>();
    &app.listen_for_client_message::<Position>();
    &app.listen_for_client_message::<Spawn>();
    &app.listen_for_client_message::<Connected>();
}

#[allow(unused)]
pub fn server_register_network_messages(app: &mut App) {
    use bevy_spicy_networking::AppNetworkServerMessage;

    // The server registers messages that arrives from a client, so that
    // it is prepared to handle them. Otherwise, an error occurs.
    app.listen_for_server_message::<UserChatMessage>();
    app.listen_for_server_message::<Actions>();
}