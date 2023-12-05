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
        app.add_systems(OnEnter(GameState::Playing), setup_grid);
    }
}

#[derive(Debug, Default, Resource)]
pub struct HighlightedHexes {
    pub selected: Option<Hex>,
    pub movement: Vec<Hex>,
}

#[derive(Debug, Resource)]
pub struct WorldMap {
    pub layout: HexLayout,
    pub entities: HashMap<Hex, Entity>,
    pub selected_material: Handle<ColorMaterial>,
    pub ring_material: Handle<ColorMaterial>,
    pub default_material: Handle<ColorMaterial>,
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
    });
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
