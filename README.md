# Chunky Bevy

A simple and efficient chunk management system for Bevy game engine, perfect for voxel games, procedural worlds, and any application that needs spatial partitioning.

## Features

- 🎯 **Simple API** - Easy to use chunk management with minimal boilerplate
- 🔄 **Automatic Loading** - Optional chunk loader component for automatic chunk spawning around entities
- 🗑️ **Automatic Unloading** - Configurable strategies for chunk lifecycle management (distance, limit, or hybrid)
- 👁️ **Visualization** - Built-in debug visualization for chunk boundaries
- ⚡ **Efficient** - HashMap-based chunk lookup with O(1) access
- 🎮 **Bevy Integration** - First-class Bevy ECS integration with hooks and resources

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.17"
chunky-bevy = "0.2"
```

Basic usage:

```rust
use bevy::prelude::*;
use chunky_bevy::prelude::*;

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
- `chunk_unloader` - Enables automatic chunk unloading with configurable strategies
- `reflect` - Enables Bevy reflection for all types

### Optional Features
- `chunk_info` - Logs chunk spawn/despawn events

### Disable default features:
```toml
chunky-bevy = { version = "0.2", default-features = false }
```

## Components

### `Chunk`
Marks an entity as a chunk. Automatically registers/unregisters with ChunkManager.

### `ChunkPos(IVec3)`
The chunk's position in chunk-space coordinates. Automatically updates the entity's `Transform`.

### `ChunkLoader(IVec3)`
Automatically loads chunks in a radius around the entity. The IVec3 defines the loading radius in each direction.

Examples:
- `ChunkLoader(IVec3::ZERO)` - Loads only the chunk the entity is in
- `ChunkLoader(IVec3::ONE)` - Loads a 3x3x3 cube of chunks
- `ChunkLoader(IVec3::new(5, 0, 5))` - Loads an 11x1x11 flat area

### `ChunkPinned`
Prevents a chunk from being automatically unloaded. Useful for spawn areas or quest locations.

### `ChunkUnloadRadius(IVec3)`
Defines the unload radius for a specific `ChunkLoader`. If absent, defaults to the loader's load radius.

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

### Unload Strategy Resources

Chunk unloading is opt-in. Insert resources to enable different strategies:

```rust
fn setup(mut commands: Commands) {
    // Distance-based: unload chunks beyond loader radius
    commands.insert_resource(ChunkUnloadByDistance);
    
    // Limit-based: LRU eviction when chunk count exceeds max
    commands.insert_resource(ChunkUnloadLimit { max_chunks: 1000 });
    
    // Hybrid (both resources): chunks must be out of range AND over limit
}
```

## Events

### `ChunkUnloadEvent`
Sent when a chunk is about to be despawned. Read with `MessageReader<ChunkUnloadEvent>` to save data before removal.

```rust
fn save_chunks(mut events: MessageReader<ChunkUnloadEvent>) {
    for event in events.read() {
        println!("Chunk {:?} unloading: {:?}", event.chunk_pos, event.reason);
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
use chunky_bevy::helpers::*;

fn setup(mut commands: Commands) {
    // Spawn chunks from chunk position (0,0,0) to (5,5,5)
    spawn_chunks_rect(&mut commands, IVec3::ZERO, IVec3::new(5, 5, 5));
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

Run the chunk unloading example:

```bash
cargo run --example chunk_unloading
```

Controls:
- **WASD** - Move horizontally
- **Q/E** - Move down/up
- **Space** - Cycle through unload strategies
- **Right-click + drag** - Look around

## Bevy Version Compatibility

| Chunky Bevy | Bevy  |
|-------------|-------|
| 0.2         | 0.17  |
| 0.1         | 0.15  |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE.Apache-2.0) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE.MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
