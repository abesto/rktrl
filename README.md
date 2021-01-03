# `rktrl`

Play in your browser: https://abesto.github.io/rktrl/

A toy roguelike built by following https://bfnightly.bracketproductions.com/rustbook/, with a few twists. Major ones:

* Using the [`legion`](https://github.com/amethyst/legion) ECS system instead of Specs
  * See [this](https://github.com/amethyst/legion/issues/217) for my experience on the Specs -> Legion migration
  * Using maximal magic for serialization using [`legion_typeuuid`](https://github.com/TomGillen/legion_typeuuid) (limited by the lack of WASM support in the `ctor` crate)
* [Cause-and-effect pattern](https://www.reddit.com/r/roguelikedev/comments/kl8xop/introducing_the_causeandeffect_pattern/) instead of `...Intent` components
* `RunState` is managed through a Deque
* Development features controlled with Cargo features
* Rendering is done in an ECS system, using `DrawBatch`es instead of directly writing to the console