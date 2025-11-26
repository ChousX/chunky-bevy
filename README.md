# Chunky Bevy

A simple and efficient chunk management system for Bevy game engine, perfect for voxel games, procedural worlds, and any application that needs spatial partitioning.

## Features

- 🎯 **Simple API** - Easy to use chunk management with minimal boilerplate
- 🔄 **Automatic Loading** - Optional chunk loader component for automatic chunk spawning around entities
- 👁️ **Visualization** - Built-in debug visualization for chunk boundaries
- ⚡ **Efficient** - HashMap-based chunk lookup with O(1) access
- 🎮 **Bevy Integration** - First-class Bevy ECS integration with hooks and resources

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.17"
chunky-bevy = "0.1"
```

Basic usage:

```rust
use bevy::prelude::*;
use chunky-bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ChunkyPlugin::default()) // 10x10x10 chunks
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a chunk loader that generates chunks around it
    commands.spawn((
        Transform::default(),
        ChunkLoader(IVec3::new(2, 1, 2)), // Load 5x3x5 chunks
    ));
    
    // Manually spawn a specific chunk
    commands.spawn((
        Chunk,
        ChunkPos(IVec3::new(0, 0, 0)),
    ));
}
```

## Features

### Default Features
- `chunk_visualizer` - Enables debug visualization of chunk boundaries
- `chunk_loader` - Enables automatic chunk loading around ChunkLoader entities

### Optional Features
- `chunk_info` - Logs chunk spawn/despawn events

### Disable default features:
```toml
chunky-bevy = { version = "0.1", default-features = false }
```

## Components

### `Chunk`
Marks an entity as a chunk. Automatically registers/unregisters with ChunkManager.

### `ChunkPos(IVec3)`
The chunk's position in chunk-space coordinates. Automatically updates the entity's `Transform`.

### `ChunkLoader(IVec3)`
Automatically loads chunks in a radius around the entity. The IVec3 defines the loading radius in each direction.

Example:
- `ChunkLoader(IVec3::ZERO)` - Loads only the chunk the entity is in
- `ChunkLoader(IVec3::ONE)` - Loads a 3x3x3 cube of chunks
- `ChunkLoader(IVec3::new(5, 0, 5))` - Loads a 11x1x11 flat area

## Resources

### `ChunkManager`
The main resource for querying and managing chunks.

```rust
fn my_system(chunk_manager: Res<ChunkManager>) {
    // Convert world position to chunk position
    let chunk_pos = chunk_manager.get_chunk_pos(&world_pos);
    
    // Get chunk entity if it exists
    if let Some(entity) = chunk_manager.get_chunk(&chunk_pos) {
        // Do something with the chunk entity
    }
    
    // Check if a chunk is loaded
    if chunk_manager.is_loaded(&chunk_pos) {
        // Chunk exists
    }
}
```

## Visualization

Enable chunk boundary visualization:

```rust
fn setup(mut visualizer: ResMut<NextState<ChunkBoundryVisualizer>>) {
    visualizer.set(ChunkBoundryVisualizer::On);
}
```

## Helpers

Spawn multiple chunks at once:

```rust
use chunky-bevy::helpers::*;

fn setup(mut commands: Commands) {
    // Spawn chunks from (0,0,0) to (5,5,5)
    spawn_chunks_rect(
        &mut commands,
        IVec3::ZERO,
        IVec3::new(5, 5, 5)
    );
    
    // Or from world positions
    spawn_chunks_rect_from_world_pos(
        &mut commands,
        Vec3::ZERO,
        Vec3::new(50.0, 50.0, 50.0)
    );
}
```

## Examples

Run the basic example:

```bash
cargo run --example basic
```

Controls:
- **WASD** - Move camera
- **Q/E** - Move camera down/up
- **HJKL** - Move cube (chunk loader)
- **Y/I** - Move cube down/up
- **Left Mouse Button** - Look around

## Custom Chunk Sizes

```rust
use chunky-bevy::ChunkyPlugin;

App::new()
    .add_plugins(ChunkyPlugin::THREE_DIMETION) // 10x10x10 (default)
    // Or custom size:
    .add_plugins(ChunkyPlugin {
        chunk_size: Vec3::new(16.0, 256.0, 16.0),
    })
```

## Bevy Version Compatibility

| Chunky Bevy | Bevy  |
|-------------|-------|
| 0.1         | 0.17  |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE.Apache-2.0) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE.MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
