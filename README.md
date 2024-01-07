[![Rust](https://github.com/nxsaken/petnat/actions/workflows/rust.yml/badge.svg)](https://github.com/nxsaken/petnat/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/petnat.svg)](https://crates.io/crates/petnat)
[![Crates.io](https://img.shields.io/crates/d/petnat.svg)](https://crates.io/crates/petnat)
[![docs.rs](https://img.shields.io/docsrs/petnat)](https://docs.rs/petnat/latest/petnat/)

A Petri net plugin for [Bevy Engine](https://github.com/bevyengine/bevy). 🍾

## About

`petnat` equips you with [Petri nets](https://en.wikipedia.org/wiki/Petri_net) to use in your Bevy projects.
It's a powerful way to model states, processes, resources, and more.

This is a very experimental project, and I mostly started it because I wanted to play with Petri nets 
and improve my Rust. I am not sure about the possible usefulness of this plugin, but I hope to discover 
how I can improve it with time and usage.

## Rough idea

1. Build a model using places and transitions.
2. Define a `PetriNet<NetId>` resource.
3. Add a `Token<NetId>` component to an entity.
4. Mark some (probably initial) places with the `Token` according to the model.
5. Fire transitions when it makes sense according to the model.
6. Implement game logic based on the current marking of the `Token`.

## Examples

Examples can be found in the [`examples`](examples) directory.