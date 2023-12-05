use crate::{
    agent::AgentCoords,
    player::{self, Player},
    GameState,
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashMap,
    window::PrimaryWindow,
};
use hexx::{shapes, *};

pub struct MapPlugin;

pub const HEX_SIZE: Vec2 = Vec2::splat(33.0);

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_grid)
            .add_systems(Update, handle_input.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Debug, Default, Resource)]
struct HighlightedHexes {
    pub selected: Hex,
    pub ring: Vec<Hex>,
    pub line: Vec<Hex>,
}

#[derive(Debug, Resource)]
pub struct WorldMap {
    pub layout: HexLayout,
    entities: HashMap<Hex, Entity>,
    selected_material: Handle<ColorMaterial>,
    line_material: Handle<ColorMaterial>,
    ring_material: Handle<ColorMaterial>,
    default_material: Handle<ColorMaterial>,
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let layout = HexLayout {
        hex_size: HEX_SIZE,
        orientation: HexOrientation::Pointy,
        ..default()
    };

    // materials
    let selected_material = materials.add(Color::ORANGE.into());
    let ring_material = materials.add(Color::LIME_GREEN.into());
    let line_material = materials.add(Color::GRAY.into());
    let default_material = materials.add(Color::WHITE.into());

    // mesh
    let mesh = hexagonal_plane(&layout);
    let mesh_handle = meshes.add(mesh);

    let entities = shapes::hexagon(Hex::ZERO, 6)
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let id = commands
                .spawn(ColorMesh2dBundle {
                    transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                    mesh: mesh_handle.clone().into(),
                    material: default_material.clone(),
                    ..default()
                })
                .with_children(|b| {
                    // b.spawn(Text2dBundle {
                    //     text: Text::from_section(
                    //         format!("{},{}", hex.x, hex.y),
                    //         TextStyle {
                    //             font_size: 7.0,
                    //             color: Color::BLACK,
                    //             ..default()
                    //         },
                    //     ),
                    //     transform: Transform::from_xyz(0.0, 0.0, 10.0),
                    //     ..default()
                    // });
                })
                .id();
            (hex, id)
        })
        .collect();

    commands.insert_resource(WorldMap {
        layout,
        entities,
        selected_material,
        ring_material,
        default_material,
        line_material,
    });
}

fn handle_input(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<Input<MouseButton>>,
    mut player_q: Query<&mut AgentCoords, With<Player>>,
    map: Res<WorldMap>,
    mut highlighted_hexes: Local<HighlightedHexes>,
) {
    let window = windows.single();
    let (camera, cam_transform) = cameras.single();
    if let Some(pos) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(cam_transform, p))
    {
        let coord = map.layout.world_pos_to_hex(pos);

        if let Some(entity) = map.entities.get(&coord) {
            if mouse.just_pressed(MouseButton::Left) {
                let mut player_coords = player_q.single_mut();
                player_coords.0 = coord;
                warn!("Player: {:?}", player_coords.0);
            }

            if coord == highlighted_hexes.selected {
                return;
            }

            // Clear highlighted hexes materials
            for vec in [&highlighted_hexes.ring, &highlighted_hexes.line] {
                for entity in vec.iter().filter_map(|h| map.entities.get(h)) {
                    commands
                        .entity(*entity)
                        .insert(map.default_material.clone());
                }
            }
            commands
                .entity(map.entities[&highlighted_hexes.selected])
                .insert(map.default_material.clone());

            // Draw a  line
            highlighted_hexes.line = Hex::ZERO.line_to(coord).collect();
            // Draw a ring
            highlighted_hexes.ring = Hex::ZERO.ring(coord.ulength()).collect();

            for (vec, mat) in [
                (&highlighted_hexes.ring, &map.ring_material),
                (&highlighted_hexes.line, &map.line_material),
            ] {
                for h in vec {
                    if let Some(e) = map.entities.get(h) {
                        commands.entity(*e).insert(mat.clone());
                    }
                }
            }

            // Make the selected tile red
            commands
                .entity(*entity)
                .insert(map.selected_material.clone());
            highlighted_hexes.selected = coord;
        }
    }
}

/// Compute a bevy mesh from the layout
fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout)
        .facing(Vec3::Z)
        .with_scale(Vec3::splat(0.95))
        .build();
    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
        .with_indices(Some(Indices::U16(mesh_info.indices)))
}
