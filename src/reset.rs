use bevy::{ecs::system::SystemId, prelude::*};
use bevy_trauma_shake::TraumaCommands;



use crate::{
    animation::{get_relative_scale_anim, DespawnOnTweenCompleted},
    map::spawn_grid,
    score::UpdateTimerEv,
};

#[derive(Component)]
pub struct ResettableGrid;

#[derive(Resource)]
pub struct RegisteredSystems {
    pub reset: SystemId,
    pub spawn_board: SystemId,
    pub skip_board: SystemId,
}

#[derive(Component)]
pub struct Resettable;

pub struct ResetPlugin;
impl Plugin for ResetPlugin {
    fn build(&self, app: &mut App) {
        let systems = RegisteredSystems {
            reset: app.world.register_system(reset_board),
            spawn_board: app.world.register_system(spawn_grid),
            skip_board: app.world.register_system(skip_board),
        };

        app.insert_resource(systems);
    }
}

fn skip_board(
    mut cmd: Commands,
    systems: Res<RegisteredSystems>,
    mut ev_w: EventWriter<UpdateTimerEv>,
) {
    cmd.run_system(systems.reset);
    cmd.add_trauma(0.7);
    ev_w.send(UpdateTimerEv(-5.));
}

fn reset_board(
    mut cmd: Commands,
    reset_q: Query<Entity, With<ResettableGrid>>,
    systems: Res<RegisteredSystems>,
) {
    for e in reset_q.iter() {
        cmd.entity(e).despawn_recursive();
    }

    cmd.run_system(systems.spawn_board);
}

pub fn tween_reset(
    mut cmd: Commands,
    reset_q: Query<Entity, Or<(With<Resettable>, With<ResettableGrid>)>>,
) {
    for e in reset_q.iter() {
        cmd.entity(e).try_insert((
            get_relative_scale_anim(Vec2::ZERO.extend(1.), 350),
            DespawnOnTweenCompleted,
        ));
    }
}
