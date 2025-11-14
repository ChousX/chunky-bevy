use bevy::prelude::*;
use chunky::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ChunkyPlugin::default());
    app.run();
}
