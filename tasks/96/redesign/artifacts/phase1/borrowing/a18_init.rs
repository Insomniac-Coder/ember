// A18: build an object step by step, calling a method before every field is set.
pub struct Renderer { pub width: u32 }
pub struct Game { pub renderer: Renderer, pub assets: Vec<String>, pub frame: u64 }

impl Game {
    fn load_assets(&self) -> Vec<String> {
        vec![format!("atlas@{}px", self.renderer.width)]
    }
    pub fn new(width: u32) -> Game {
        let mut g: Game;
        g.renderer = Renderer { width };
        g.frame = 0;
        g.assets = g.load_assets();
        g
    }
}
