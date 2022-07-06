use std::{net::SocketAddr, time::Duration};

use bevy::{prelude::*, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, math::vec3, app::ScheduleRunnerSettings};
use bevy_spicy_networking::{ServerPlugin, NetworkServer, ServerNetworkEvent, ConnectionId, NetworkData};
use rand::Rng;
use shared::{Connected, Spawn, Actions, server_register_network_messages, Position, NewChatMessage};


#[derive(Component)]
pub struct Player;

const WIDTH: f32 = 800.;
const HEIGHT: f32 = 600.;
const UNIT: f32 = 26.;
const MARGIN: f32 = UNIT/2. + 40.;
const STEP: f32 = 10.;
const TOP_BORDER: f32   =  HEIGHT/2. - MARGIN;
const BOT_BORDER: f32   = -HEIGHT/2. + MARGIN;
const RIGHT_BORDER: f32 =  WIDTH/2. - MARGIN;
const LEFT_BORDER: f32  = -WIDTH/2. + MARGIN;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Rusty Crabs".to_string(),
            ..Default::default()
        })
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0/60.0
        )))
        .add_plugins(DefaultPlugins)
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(spawn_camera)
        //.add_startup_system(spawn_player)
        .add_plugin(ServerPlugin);
    server_register_network_messages(&mut app);

    app.add_startup_system(setup_networking)
        .add_system(handle_connection_events)
        .add_system(send_messages)
        .add_system(receive_messages)

        // LOCAL
        //.add_system_set(
        //    SystemSet::on_update(CoreStage::PreUpdate)
        //        .with_run_criteria(FixedTimestep::steps_per_second(60.))    
        //        .with_system(move_player)
        //)

        // SERVER
        //.add_system_set(
        //    SystemSet::on_update(CoreStage::PreUpdate)
        //        .with_run_criteria(FixedTimestep::steps_per_second(1.))
        //        .with_system(hello)
        //)

        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

#[allow(dead_code)]
fn spawn_player(mut commands: Commands) {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0., 0.47, 1.),
            custom_size: Some(Vec2::new(UNIT, UNIT)),
            ..Default::default()
        },
        transform: Transform {
            translation: vec3(
                -WIDTH/2.  + MARGIN,
                 HEIGHT/2. - MARGIN,
                0.),
            ..Default::default()
        },
        ..Default::default()
    }).insert(Player);
}

#[allow(dead_code)]
fn move_player(
    mut query: Query<&mut Transform, With<Player>>
) {
    for mut transform in query.iter_mut() {
        if transform.translation.x < RIGHT_BORDER
        && transform.translation.y >= TOP_BORDER
        {
            transform.translation.x += STEP;
        } else if transform.translation.x >= RIGHT_BORDER
        && transform.translation.y > BOT_BORDER
        {
            transform.translation.y -= STEP;
        } else if transform.translation.x > LEFT_BORDER
        && transform.translation.y <= BOT_BORDER
        {
            transform.translation.x -= STEP;
        } else if transform.translation.x <= LEFT_BORDER
        && transform.translation.y < TOP_BORDER
        {
            transform.translation.y += STEP;
        }
    }
}

#[allow(dead_code)]
fn hello () {
    info!("Hello from server!");
}

fn setup_networking(mut net: ResMut<NetworkServer>) {
    let ip_addr = "127.0.0.1".parse().expect("Couldn't parse IP");
    let socket_addr = SocketAddr::new(ip_addr, 9000);
    match net.listen(socket_addr) {
        Ok (_) => (),
        Err(_err) => {
            error!("Couldn't listen!");
        }
    }
    info!("Listening for connections!");
}

#[derive(Component)]
struct OnlinePlayer(ConnectionId);

fn handle_connection_events(
    mut commands: Commands,
    net: Res<NetworkServer>,
    mut network_events: EventReader<ServerNetworkEvent>
) {
    for event in network_events.iter() {
        if let ServerNetworkEvent::Connected(conn_id) = event {
            let x = rand::thread_rng().gen_range(-WIDTH/2. + MARGIN..WIDTH/2. - MARGIN);
            let y = rand::thread_rng().gen_range(-HEIGHT/2. + MARGIN..HEIGHT/2. - MARGIN);
            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1., 0.47, 0.),
                    custom_size: Some(Vec2::new(UNIT, UNIT)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: vec3(x, y, 0.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Player)
            .insert(OnlinePlayer(*conn_id));

            let _ = net.send_message(*conn_id, Connected {
                id: conn_id.uuid()
            });

            net.broadcast(NewChatMessage {
                name: String::from("Server"),
                message: format!("New user connected; {}", conn_id)
            });
            info!("New player connected: {}", conn_id);

            net.broadcast(Spawn {
                id: conn_id.uuid(),
                x,
                y
            })
        }
    }
}

fn send_messages(
    net: Res<NetworkServer>,
    mut positions: Query<(&mut Transform, &mut OnlinePlayer), With<OnlinePlayer>>
) {
    for (pos, player) in positions.iter_mut() {
        net.broadcast(Position {
            id: player.0.uuid(),
            x: pos.translation.x,
            y: pos.translation.y, 
        });
    } 
}

fn receive_messages(
    mut actions: EventReader<NetworkData<Actions>>,
    mut query: Query<(&mut Transform, &mut OnlinePlayer), With<OnlinePlayer>>,
) {
    for action in actions.iter() {
        let source = action.source();
        for (mut tranform, player) in query.iter_mut() {
            info!("Source: {}", source.uuid());
            info!("Player: {}", source.uuid());
            if source.uuid() == player.0.uuid() {
                info!("Received action: {:?}", action);
                tranform.translation.x += (action.d - action.a) as f32 * STEP;
                tranform.translation.y += (action.w - action.s) as f32 * STEP;
            }
        }
    }
}