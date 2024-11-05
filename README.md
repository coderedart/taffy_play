# Taffy Play

[![dependency status](https://deps.rs/repo/github/coderedart/taffy_play/status.svg)](https://deps.rs/repo/github/coderedart/taffy_play)
[![Build Status](https://github.com/coderedart/taffy_play/workflows/CI/badge.svg)](https://github.com/coderedart/taffy_play/actions?workflow=CI)

playground to play with taffy styles and see the layout results. Inspired by [Taffy Cpp Playground](https://inobelar.github.io/emscripten_samples/sample_Taffy_cpp_Playground.html).

Link to online playground: https://coderedart.github.io/taffy_play/

I could use some help with:
1. Making the UI prettier and more intuitive to use.
2. Adding docs of each style attribute as tooltips (or with a help icon).
3. Providing a default set of example taffy trees, to showcase how taffy layout works.
4. Add the grid related style attributes to UI.

### Usage
There's basically two windows:
1. Node Visuals: This displays the taffy nodes as rectangles, with the focused node using red color.
    1. If you hover over any node, you will see a tooltip text that shows its layout values (location, size, margins, border etc..)
    2. If you click any node, it will become the focused node and you can edit its attributes in the editor window.
2. Node Editor: This is where you can browse nodes and edit their style values.
    1. The left side panel shows a tree view of nodes, and the focused node is selected.
    2. The style attributes displayed in the window belong to the focused node.
        1. Read the docs at https://docs.rs/taffy to understand what they mean. 
        2. The taffy nodes are laid out every frame, so all changes should be immediately visible. 
        3. You can delete the node by clicking delete button.
        4. You can add a child node by clicking add node 


### Testing locally

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.

