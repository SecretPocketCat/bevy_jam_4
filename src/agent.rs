use bevy::prelude::*;
use hexx::Hex;

use crate::{map::WorldMap, GameState};

#[derive(Component, Default, Deref, DerefMut)]
pub struct AgentCoords(pub Hex);

pub struct AgentPlugin;
impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_agent.run_if(in_state(GameState::Playing)));
    }
}

fn move_agent(
    map: Res<WorldMap>,
    mut agent_q: Query<(&mut Transform, &AgentCoords), Changed<AgentCoords>>,
) {
    for (mut t, coords) in agent_q.iter_mut() {
        t.translation = map.layout.hex_to_world_pos(coords.0).extend(0.);
    }
}
