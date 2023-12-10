macro_rules! relative_lens_struct {
    ($lens:ident, $value:ty) => {
        #[derive(Default)]
        pub struct $lens {
            pub(super) start: Option<$value>,
            pub(super) end: $value,
        }

        impl $lens {
            #[allow(dead_code)]
            pub fn relative(end: $value) -> Self {
                Self { start: None, end }
            }

            #[allow(dead_code)]
            pub fn new(start: $value, end: $value) -> Self {
                Self {
                    start: Some(start),
                    end,
                }
            }
        }
    };
}

pub(super) use relative_lens_struct;

macro_rules! color_lens {
    ($component:ty, $lens:ident, $field:tt) => {
        relative_lens_struct!($lens, Color);

        impl Lens<$component> for $lens {
            fn lerp(&mut self, target: &mut $component, ratio: f32) {
                target.$field = lerp_color(
                    self.start
                        .expect("Lerping has started so initial values should have been set"),
                    self.end,
                    ratio,
                );
            }

            fn update_on_tween_start(
                &mut self,
                target: &$component,
                _direction: TweeningDirection,
                _times_completed: i32,
            ) {
                self.start.get_or_insert_with(|| target.$field);
            }
        }
    };
}

pub(super) use color_lens;

macro_rules! relative_lens {
    ($component:ty, $value:ty, $lens:ident, $field:tt) => {
        relative_lens_struct!($lens, $value);

        impl Lens<$component> for $lens {
            fn lerp(&mut self, target: &mut $component, ratio: f32) {
                let start = self.start.unwrap();
                let value = start + (self.end - start) * ratio;
                target.$field = value;
            }

            fn update_on_tween_start(
                &mut self,
                target: &$component,
                _direction: TweeningDirection,
                _times_completed: i32,
            ) {
                self.start.get_or_insert_with(|| target.$field);
            }
        }
    };
}

pub(super) use relative_lens;

macro_rules! relative_tween_fns {
    ($name:ident, $component:ty, $lens:ty, $value_start:ty, $value_end:ty) => {
        paste::paste! {
            pub fn [<get_absolute_ $name _tween>](
                start: $value_start,
                end: $value_end,
                duration_ms: u64,
            ) -> Tween<$component> {
                [<get_ $name _tween>](
                    Some(start),
                    end,
                    duration_ms,
                    EaseFunction::QuadraticInOut,
                )
            }

            pub fn [<get_relative_ $name _tween>](
                end: $value_end,
                duration_ms: u64,
            ) -> Tween<$component> {
                [<get_ $name _tween>](
                    None,
                    end,
                    duration_ms,
                    EaseFunction::QuadraticInOut,
                )
            }

            pub fn [<get_absolute_ $name _anim>](
                start: $value_start,
                end: $value_end,
                duration_ms: u64,
            ) -> Animator<$component> {
                Animator::new([<get_absolute_ $name _tween>](
                    start,
                    end,
                    duration_ms,
                ))
            }

            pub fn [<get_relative_ $name _anim>](
                end: $value_end,
                duration_ms: u64,
            ) -> Animator<$component> {
                Animator::new([<get_relative_ $name _tween>](
                    end,
                    duration_ms,
                ))
            }

            pub fn [<get_ $name _anim>](
                start: Option<$value_start>,
                end: $value_end,
                duration_ms: u64,
                ease: EaseFunction,
            ) -> Animator<$component> {
                Animator::new([<get_ $name _tween>](
                    start,
                    end,
                    duration_ms,
                    ease,
                ))

            }

            pub fn [<get_ $name _tween>](
                start: Option<$value_start>,
                end: $value_end,
                duration_ms: u64,
                ease: EaseFunction,
            ) -> Tween<$component> {
                Tween::new(
                    ease,
                    Duration::from_millis(duration_ms),
                    $lens {
                        start,
                        end,
                    },
                ).with_completed_event(0)
            }
        }
    };
}

pub(super) use relative_tween_fns;
