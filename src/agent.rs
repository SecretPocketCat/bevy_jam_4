use crate::animation::get_relative_translation_anim;
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
    mut cmd: Commands,
    map: Res<WorldMap>,
    agent_q: Query<(Entity, &AgentCoords), Changed<AgentCoords>>,
) {
    for (e, coords) in agent_q.iter() {
        let position = map.layout.hex_to_world_pos(coords.0).extend(0.);
        cmd.entity(e)
            .insert(get_relative_translation_anim(position, 250));
    }
}
