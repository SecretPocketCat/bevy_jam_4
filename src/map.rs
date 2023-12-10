use crate::{
    animation::{delay_tween, get_scale_anim, get_scale_tween},
    loading::TextureAssets,
    map_completion::CompletedMap,
    piece::{get_opposite_side_index, PieceHexData},
    reset::Resettable,
    GameState,
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
    utils::{
        petgraph::{
            adj::NodeIndex,
            algo::{astar, dijkstra},
            data::Build,
            graph::UnGraph,
        },
        HashMap, HashSet,
    },
    window::PrimaryWindow,
};
use bevy_tweening::{Animator, EaseFunction};
use hexx::{shapes, Direction, *};
use rand::{seq::SliceRandom, thread_rng, Rng};
use strum::EnumIter;

pub use self::edge_connection::EdgeConnection;

pub const MAP_RADIUS: u32 = 1;
pub const HEX_SIZE: f32 = 46.;
pub const HEX_SIZE_INNER_MULT: f32 = 0.925;
pub const HEX_SIZE_INNER: f32 = HEX_SIZE * HEX_SIZE_INNER_MULT;

// https://www.redblobgames.com/grids/hexagons/#basics
pub const HEX_WIDTH: f32 = HEX_SIZE * 1.732_050_8; // sqrt of 3
pub const HEX_HEIGHT: f32 = HEX_SIZE * 2.;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_grid);
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct WorldLayout(HexLayout);

type MapGraph = UnGraph<(), ()>;

mod edge_connection {
    use std::cmp::Ordering;

    use hexx::Hex;

    #[derive(Debug, Hash, PartialEq, Eq, Clone)]
    pub struct EdgeConnection(Hex, Hex);

    impl EdgeConnection {
        pub fn new(a: Hex, b: Hex) -> Self {
            // sort the hexes in the same way to get the same hash
            match format!("{}:{}", a.x, a.y).cmp(&format!("{}:{}", b.x, b.y)) {
                Ordering::Greater => Self(a, b),
                _ => Self(b, a),
            }
        }

        pub fn first(&self) -> Hex {
            self.0
        }

        pub fn second(&self) -> Hex {
            self.1
        }
    }
}

#[derive(Debug, Resource)]
pub struct WorldMap {
    pub hexes: HashMap<Hex, MapHex>,
    houses: HashSet<Hex>,
    graph: MapGraph,
    hex_nodes: HashMap<NodeIndex, Hex>,
    hex_edge_nodes: HashMap<NodeIndex, EdgeConnection>,
    edge_connection_nodes: HashMap<EdgeConnection, NodeIndex>,
}

impl WorldMap {
    fn get_or_add_edge_connection(&mut self, a: Hex, b: Hex) -> u32 {
        let edge_conn = EdgeConnection::new(a, b);

        *self
            .edge_connection_nodes
            .entry(edge_conn.clone())
            .or_insert_with(|| {
                let index = self.graph.add_node(()).index() as u32;
                self.hex_edge_nodes.insert(index, edge_conn);

                index
            })
    }

    fn add_hex_graph_edges(&mut self, hex: &Hex, edge_connections: &[bool; 6]) {
        let hex_data = &self.hexes[hex];
        let hex_node = hex_data.node_index;

        for (side, _) in edge_connections
            .iter()
            .enumerate()
            .filter(|(_, conn)| **conn)
        {
            let target_hex = *hex + Hex::new(1, -1).rotate_cw(side as u32);
            let edge_node = self.get_or_add_edge_connection(*hex, target_hex);

            self.graph.add_edge(hex_node.into(), edge_node.into(), ());

            // add connections to adjacent houses
            if self.houses.contains(&target_hex) {
                self.graph.add_edge(
                    edge_node.into(),
                    self.hexes[&target_hex].node_index.into(),
                    (),
                );
            }
        }
    }

    pub fn place_piece(&mut self, hex: Hex, piece_hexes: &HashMap<Hex, PieceHexData>) {
        let placed_hexes: Vec<_> = piece_hexes
            .iter()
            .map(|(key, val)| (hex + *key, (val.connections.clone(), val.entity.clone())))
            .collect();

        // place hexes
        for (hex, (connected_sides, hex_e)) in placed_hexes.iter() {
            if let Some(connected_sides) = connected_sides {
                self.add_hex_graph_edges(&hex, connected_sides);
            }

            self.hexes.entry(*hex).and_modify(|map_hex| {
                map_hex.placed_hex_e = Some(*hex_e);
            });
        }
    }

    pub fn get_completed_routes(&mut self) -> Option<CompletedMap> {
        if let Some(hex) = self.houses.iter().next() {
            let start_node = self.hexes[hex].node_index;
            let res = dijkstra(&self.graph, start_node.into(), None, |_| 1);

            let other_houses: Vec<_> = self.houses.iter().filter(|h| *h != hex).cloned().collect();

            let all_reachable = other_houses
                .iter()
                .all(|h| res.contains_key(&self.hexes[h].node_index.into()));

            if all_reachable {
                info!("reached all houses reached from {hex:?}");

                Some(CompletedMap {
                    routes: other_houses
                        .iter()
                        .map(|house| {
                            let end = self.hexes[house].node_index;
                            let (_, path) = astar(
                                &self.graph,
                                start_node.into(),
                                |n| n == end.into(),
                                |_| 1,
                                |n| {
                                    let node_index = n.index() as u32;
                                    let hex =
                                        self.hex_nodes.get(&node_index).cloned().unwrap_or_else(
                                            || self.hex_edge_nodes[&node_index].first(),
                                        );
                                    house.unsigned_distance_to(hex)
                                },
                            )
                            .unwrap();
                            info!("Path from {hex:?} to {house:?}: {path:?}");

                            path.iter()
                                .map(|n| self.hex_nodes.get(&(n.index() as u32)))
                                .flatten()
                                .cloned()
                                .collect()
                        })
                        .collect(),
                    dead_ends: self
                        .graph
                        .node_indices()
                        .filter(|n| self.graph.neighbors_undirected(*n).count() == 1)
                        .map(|n| self.hex_edge_nodes.get(&(n.index() as u32)))
                        .flatten()
                        .cloned()
                        .collect(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct MapHex {
    pub placed_hex_e: Option<Entity>,
    node_index: NodeIndex,
}

impl MapHex {
    pub fn new(graph: &mut MapGraph) -> Self {
        Self {
            placed_hex_e: None,
            node_index: graph.add_node(()).index() as u32,
        }
    }

    pub fn occupied(hex_e: Entity, graph: &mut MapGraph) -> Self {
        let mut hex = Self::new(graph);
        hex.placed_hex_e = Some(hex_e);

        hex
    }
}

pub fn spawn_grid(
    mut cmd: Commands,
    sprites: Res<TextureAssets>,
    completed_map: Option<Res<CompletedMap>>,
) {
    if completed_map.is_some() {
        cmd.remove_resource::<CompletedMap>();
    }

    let mut rng = thread_rng();

    let layout = HexLayout {
        hex_size: Vec2::splat(HEX_SIZE),
        orientation: HexOrientation::Pointy,
        ..default()
    };

    let mut graph = MapGraph::new_undirected();
    let mut hexes: HashMap<Hex, MapHex> = shapes::hexagon(Hex::ZERO, MAP_RADIUS)
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let hex_len = hex.ulength() as u64;
            cmd.spawn((
                SpriteSheetBundle {
                    transform: Transform {
                        translation: pos.extend(0.),
                        scale: Vec2::ZERO.extend(1.),
                        ..default()
                    },
                    sprite: TextureAtlasSprite::new(12),
                    texture_atlas: sprites.tiles.clone(),
                    ..default()
                },
                Animator::new(delay_tween(
                    get_scale_tween(
                        None,
                        Vec3::ONE,
                        350,
                        if hex_len == MAP_RADIUS as u64 {
                            EaseFunction::BackOut
                        } else {
                            EaseFunction::QuadraticOut
                        },
                    ),
                    hex_len * 80,
                )),
                Resettable,
            ))
            .with_children(|b| {
                b.spawn(Text2dBundle {
                    text: Text::from_section(
                        format!("{},{}", hex.x, hex.y),
                        TextStyle {
                            font_size: 17.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, 0.0, 10.0),
                    ..default()
                });
            });
            (hex, MapHex::new(&mut graph))
        })
        .collect();

    // houses
    let count = 3;
    let mut house_hexes = HashSet::with_capacity(count);
    let mut wedge_indices = HashSet::with_capacity(count);

    let direction_group = [
        [
            Direction::Top,
            Direction::BottomLeft,
            Direction::BottomRight,
        ],
        [Direction::Bottom, Direction::TopLeft, Direction::TopRight],
    ]
    .choose(&mut rng)
    .unwrap();

    for dir in direction_group {
        'wedge: loop {
            for (i, hex) in Hex::ZERO
                .corner_wedge(
                    ((MAP_RADIUS - 2.min(MAP_RADIUS))..=(MAP_RADIUS + 1)).rev(),
                    *dir,
                )
                .enumerate()
            {
                if house_hexes.contains(&hex) || wedge_indices.contains(&i) {
                    continue;
                }

                if rng.gen_bool(0.25) {
                    // todo: tween
                    let entity = cmd
                        .spawn((
                            SpriteSheetBundle {
                                transform: Transform {
                                    translation: layout.hex_to_world_pos(hex).extend(1.),
                                    scale: Vec2::ZERO.extend(1.),
                                    ..default()
                                },
                                sprite: TextureAtlasSprite::new(11),
                                texture_atlas: sprites.tiles.clone(),
                                ..default()
                            },
                            Animator::new(delay_tween(
                                get_scale_tween(None, Vec3::ONE, 400, EaseFunction::BackOut),
                                500 + house_hexes.len() as u64 * 80,
                            )),
                            Resettable,
                        ))
                        .id();

                    hexes
                        .entry(hex)
                        .and_modify(|h| h.placed_hex_e = Some(entity.clone()))
                        .or_insert_with(|| MapHex::occupied(entity, &mut graph));
                    house_hexes.insert(hex);
                    wedge_indices.insert(i);

                    break 'wedge;
                }
            }
        }
    }

    let world_map = WorldMap {
        houses: house_hexes.iter().map(|h| *h).collect(),
        hex_nodes: hexes
            .iter()
            .map(|(h, map_hex)| (map_hex.node_index, *h))
            .collect(),
        hexes,
        graph,
        edge_connection_nodes: HashMap::new(),
        hex_edge_nodes: HashMap::new(),
    };

    cmd.insert_resource(WorldLayout(layout));
    cmd.insert_resource(world_map);
}
