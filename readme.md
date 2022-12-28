# jil.im

A link shortener published at <https://jil.im>.

Written in Rust, using Axum, meant to be deployed on Render.

Creating, editing, and deleting shortlinks is protected by a web password, but seeing and navigating to shortlinks is available to everyone who can access the web service.

Data is stored in memory and synced with an API (specifically [this API](https://github.com/jameslittle230/jil-api), though nothing prevents you from publishing your own API and using that instead, or not using an API at all and storing it all in memory if you like to live dangerously). This means it's fast, since all lookups are in-memory, but you can't have too many shortlinks since they're all stored in memory.

The link resolution algorithm is published as a standalone crate: <https://lib.rs/crates/golink>
