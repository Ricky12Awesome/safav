# Simple Audio For Audio Visualizers (safav)

Simple way to handle audio for audio visualizers

**only listens to audio, does not do any playback of audio**

# Features

- Very Fast
- Loopback (Windows / Linux, Mac might not be as simple, also I don't own one)
- minimal dependencies / lightweight


# Example

```rust
// Create host
let mut host = Host::new().unwrap();

// Get list of devices provided by host
let devices = host.devices();

// Creates a new listener that can be shared between threads since host itself can't be shared
let listener = host.create_listener();

// Starts the listener to listen to audio
host.listen().unwrap();

loop {
  // Polls data from listener
  let data = listener.poll();
  
  // ...
}
```