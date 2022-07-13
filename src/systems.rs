use bevy_ecs::prelude::*;
use sdl2::keyboard::Keycode;
use std::collections::HashSet;

pub fn print_keys(keys: Res<HashSet<Keycode>>) {}
