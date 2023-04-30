use diamond::{run, DiamondApp, DiamondContext};

struct App {}

impl DiamondApp for App {
    fn start(
        &mut self,
        _event_loop: &winit::event_loop::EventLoop<()>,
        _context: &mut DiamondContext,
    ) {
        println!("Hello, world print!");
        log::info!("Hello, world log!");
    }
}

fn main() {
    pollster::block_on(run(App {}));
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use winit::{event::Event, window::Window};

    #[wasm_bindgen(start)]
    pub fn run() {
        #[allow(clippy::main_recursion)]
        super::main();
    }
}
