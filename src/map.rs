use crate::{agent::AgentCoords, GameState};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashMap,
    window::PrimaryWindow,
};
use hexx::{shapes, *};

pub const HEX_SIZE: f32 = 50.;
pub const HEX_SIZE_INNER_MULT: f32 = 0.95;
pub const HEX_SIZE_INNER: f32 = HEX_SIZE * HEX_SIZE_INNER_MULT;

// https://www.redblobgames.com/grids/hexagons/#basics
pub const HEX_WIDTH: f32 = HEX_SIZE * 1.732_050_8; // sqrt of 3
pub const HEX_HEIGHT: f32 = HEX_SIZE * 2.;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlacedHexes>()
            .add_systems(OnEnter(GameState::Playing), setup_grid);
    }
}

#[derive(Debug, Resource)]
pub struct WorldMap {
    pub layout: HexLayout,
    pub entities: HashMap<Hex, Entity>,
    pub selected_material: Handle<ColorMaterial>,
    pub ring_material: Handle<ColorMaterial>,
    pub default_material: Handle<ColorMaterial>,
}

// todo
#[derive(Clone)]
pub enum Ingredient {
    Honey,
    Ginger,
    Sugar,
}

// // todo:
// pub enum HexColor {

// }

#[derive(Component, Clone)]
pub struct HexData {
    pub ingredient: Ingredient,
    pub color: Color,
}

#[derive(Resource, Default)]
pub struct PlacedHexes {
    pub placed: HashMap<Hex, HexData>,
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let layout = HexLayout {
        hex_size: Vec2::splat(HEX_SIZE),
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

    let entities = shapes::hexagon(Hex::ZERO, 3)
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let id = commands
                .spawn(ColorMesh2dBundle {
                    transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                    mesh: mesh_handle.clone().into(),
                    material: default_material.clone(),
                    ..default()
                })
                // .with_children(|b| {
                //     b.spawn(Text2dBundle {
                //         text: Text::from_section(
                //             format!("{},{}", hex.x, hex.y),
                //             TextStyle {
                //                 font_size: 7.0,
                //                 color: Color::BLACK,
                //                 ..default()
                //             },
                //         ),
                //         transform: Transform::from_xyz(0.0, 0.0, 10.0),
                //         ..default()
                //     });
                // })
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

    commands.insert_resource(PlacedHexes::default());
}

/// Compute a bevy mesh from the layout
fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout)
        .facing(Vec3::Z)
        .with_scale(Vec3::splat(HEX_SIZE_INNER_MULT))
        .build();

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
        .with_indices(Some(Indices::U16(mesh_info.indices)))
}
