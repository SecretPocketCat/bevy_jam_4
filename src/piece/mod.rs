use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;

use crate::{map::WorldMap, map_completion::CompletedMap, GameState};

use self::{
    drag::{drag_piece, drag_piece_end, dragged, out_piece, over_piece, HoveredPiece},
    hex::HexBlueprints,
    piece::{rotate_piece, spawn_pieces},
};

mod drag;
mod hex;
mod piece;

pub use hex::PieceHexData;
pub use piece::{Piece};

pub struct PiecePlugin;
impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HexBlueprints>()
            .init_resource::<HoveredPiece>()
            .add_plugins(DefaultPickingPlugins)
            .add_systems(
                Update,
                (
                    spawn_pieces,
                    dragged,
                    drag_piece,
                    drag_piece_end.after(spawn_pieces),
                    rotate_piece,
                    over_piece.after(out_piece),
                    out_piece,
                )
                    .distributive_run_if(
                        in_state(GameState::Game)
                            .and_then(resource_exists::<WorldMap>())
                            .and_then(not(resource_exists::<CompletedMap>())),
                    ),
            );
    }
}
