use bevy::prelude::*;
use rand::distributions::WeightedIndex;

#[derive(Debug, Clone)]
pub(super) struct RouteHexBlueprint {
    pub(super) connected_sides: [bool; 6],
    pub(super) atlas_index: usize,
    pub(super) weight: u8,
}

#[derive(Debug, Resource)]
pub(super) struct HexBlueprints {
    pub(super) hexes: Vec<RouteHexBlueprint>,
    pub(super) weighted_index: WeightedIndex<u8>,
    pub(super) size_weighted_index: WeightedIndex<u8>,
}

impl Default for HexBlueprints {
    fn default() -> Self {
        // these go clockwise from the top-right edge (pointy hexes)
        let blueprints: Vec<_> = [
            ([false, true, true, false, false, false], 3),
            ([false, true, false, true, false, false], 8),
            ([false, true, false, false, true, false], 10),
            ([false, true, true, true, false, false], 2),
            ([false, true, false, true, true, false], 4),
            ([false, true, true, false, true, false], 4),
            ([false, true, false, true, false, true], 3),
            ([false, true, true, true, true, false], 1),
            ([false, true, true, false, true, true], 1),
            ([true, true, true, true, true, true], 0),
        ]
        .into_iter()
        .enumerate()
        .map(
            |(atlas_index, (connected_sides, weight))| RouteHexBlueprint { connected_sides, atlas_index, weight },
        )
        .collect();

        let weighted_index = WeightedIndex::new(blueprints.iter().map(|h| h.weight)).unwrap();

        Self {
            hexes: blueprints,
            weighted_index,
            // todo: tweak when triples work properly
            size_weighted_index: WeightedIndex::new([3, 4 /*3*/]).unwrap(),
            // size_weighted_index: WeightedIndex::new([2, 3, 1]).unwrap(),
        }
    }
}

#[derive(Component)]
pub struct PieceHexData {
    pub entity: Entity,
    pub(super) side_index: u8,
    pub connections: Option<[bool; 6]>,
}
