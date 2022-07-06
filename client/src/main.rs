use std::net::SocketAddr;

use bevy::{prelude::*, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, math::vec3, reflect::Uuid};
use bevy_spicy_networking::{ClientPlugin, NetworkClient, NetworkSettings, ClientNetworkEvent, NetworkData};
use shared::{NewChatMessage, client_register_network_messages, Position, Spawn, Connected, Actions};

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

#[derive(Component)]
pub struct Me(Uuid);

#[derive(Component)]
struct OnlinePlayer(Uuid);

#[derive(Component)]
struct Keys {
    w: i16,
    a: i16,
    s: i16,
    d: i16
}

fn main() {
    let mut app = App::new();
        app.insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Rusty Crabs".to_string(),
            ..Default::default()
        })
        .insert_resource(Keys { w: 0, a: 0, s: 0, d: 0 })
        .add_plugins(DefaultPlugins)
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(ClientPlugin);
    client_register_network_messages(&mut app);

    app.add_startup_system(spawn_camera)
        //.add_startup_system(spawn_player)
        .add_startup_system(connect)
        .add_system(handle_network_events)
        .add_system(receive_messages)
        .add_system(send_messages)
        .add_system(movement)

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

fn connect(
    mut net: ResMut<NetworkClient>,
) {
    let ip_addr = "127.0.0.1".parse().expect("Couldn't parse IP");
    let socket_addr = SocketAddr::new(ip_addr, 9000);

    net.connect(
        socket_addr,
        NetworkSettings {
            max_packet_length: 10 * 1024 * 1024,
        }
    );
}

fn handle_network_events(
    mut net_events: EventReader<ClientNetworkEvent>,
) {
    for event in net_events.iter() {
        match event {
            ClientNetworkEvent::Connected => {
                info!("Connected to server!");
            },
            _ => {}
        }
    }
}

fn receive_messages(
    mut commands: Commands,
    mut messages: EventReader<NetworkData<NewChatMessage>>,
    mut positions: EventReader<NetworkData<Position>>,
    mut spawns: EventReader<NetworkData<Spawn>>,
    mut connected : EventReader<NetworkData<Connected>>,
    mut players: Query<(&mut Transform, &mut OnlinePlayer), With<OnlinePlayer>>,
    my_id: Option<Res<Me>>
) {
    for msg in messages.iter() {
        info!("New message: {}, {}", &msg.name, &msg.message);
    }
    for pos in positions.iter() {
        info!("Position: [x: {}, y: {}], ID: {:?}", pos.x, pos.y, pos.id);
        for (mut transform, player) in players.iter_mut() {
            if pos.id == player.0 {
               transform.translation.x = pos.x; 
               transform.translation.y = pos.y; 
            }
        }

    }
    for spawn in spawns.iter() {
        let mut command = commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 0.47, 1.),
                custom_size: Some(Vec2::new(UNIT, UNIT)),
                ..Default::default()
            },
            transform: Transform {
                translation: vec3(
                    spawn.x,
                    spawn.y,
                    0.),
                ..Default::default()
            },
            ..Default::default()
        });
        command.insert(Player);
        command.insert(OnlinePlayer(spawn.id));
        
        match my_id {
            Some(ref my_id) => {
                if spawn.id == my_id.0 {
                    command.insert(Me(spawn.id));
                }
            },
            None => ()
        }
    }
    for conn in connected.iter() {
        commands.insert_resource(Me(conn.id));
        info!("My id is: {}", conn.id);
    }
}

fn send_messages(
    net: Res<NetworkClient>,
    mut actions: ResMut<Keys>
) {
    let _ = net.send_message(Actions {
        w: actions.w,
        a: actions.a,
        s: actions.s,
        d: actions.d
    }); 
    actions.w = 0;
    actions.a = 0;
    actions.s = 0;
    actions.d = 0;
}

fn movement(
    mut actions: ResMut<Keys>,
    input: Res<Input<KeyCode>>,
) {
    let up = input.any_pressed([KeyCode::W, KeyCode::Up, KeyCode::K]);
    let left = input.any_pressed([KeyCode::A, KeyCode::Left, KeyCode::H]);
    let down = input.any_pressed([KeyCode::S, KeyCode::Down, KeyCode::J]);
    let right = input.any_pressed([KeyCode::D, KeyCode::Right, KeyCode::L]);

    if up {
        actions.w += 1;
        info!("UP pressed!");
    }
    if left {
        actions.a += 1;
        info!("LEFT pressed!");
    }
    if down {
        actions.s += 1;
        info!("DOWN pressed!");
    }
    if right {
        actions.d += 1;
        info!("RIGHT pressed!");
    }
}