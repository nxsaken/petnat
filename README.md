A Petri net plugin for [Bevy Engine](https://github.com/bevyengine/bevy). 🍾

## About

`petnat` will equip you with [Petri nets](https://en.wikipedia.org/wiki/Petri_net) to use in your Bevy projects.
It's a powerful way to model states, processes, resources, and more.

This is a very experimental project, and actual Bevy integration is unavailable yet.
Feedback and thoughts are welcome!

## Rough idea

1. Model something using places and transitions.
2. Define a `PetriNet<NetId>` resource.
3. Add a `Token<NetId>` component to an entity.
4. Mark places with the `Token` according to the context.
5. Transitions will be scheduled or triggered with Events (needs experimenting).
6. Implement game logic based on the current marking of the `Token`.
7. Hopefully add static analysis on the net somehow via Rust's type system?

## Example

Here is an example of how Petri nets are defined and used.

```rust
// impl NetId
enum Choice {}

// impl Place<Choice>
enum P0 {}
enum P1 {}
enum P2 {}
enum P3 {}

// impl Trans<Choice>
enum T0 {}
enum T1 {}

/// Two transitions sharing a preset place (P1).
/// When one of them fires, the other ceases to be enabled.
/// 
/// ## Before (asterisks denote the current marking):
/// 
/// (P0)* --> |T0| -\
/// (P1)* -<         >-> (P3)
/// (P2)* --> |T1| -/
/// 
/// ## After T0 fires:
/// 
/// (P0)  --> |T0| -\
/// (P1)  -<         >-> (P3)*
/// (P2)* --> |T1| -/
fn choice() -> PetriNet<Choice> {
    PetriNet::empty()
        .add_place::<P0>()
        .add_place::<P1>()
        .add_place::<P2>()
        .add_place::<P3>()
        .add_trans::<
            T0, 
            ((P0, W1), (P1, W1)),
            (P3, W1) // W1, W2, W3, ... – arc weights
        >()
        .add_trans::<
            T1, 
            ((P1, W1), (P2, W1)), 
            (P3, W1)
        >()
}

fn test_choice() {
    let net = choice();
    let mut token = net.spawn_token();
    token.mark::<P0>(1);
    token.mark::<P1>(1);
    token.mark::<P2>(1);
    assert!(net.is_enabled::<T0>(&token));
    assert!(net.is_enabled::<T1>(&token));
    assert!(net.fire::<T0>(&mut token).is_some());
    assert!(!net.is_enabled::<T1>(&token));
}
```