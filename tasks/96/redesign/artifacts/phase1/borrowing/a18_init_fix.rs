// A18 workaround: compute the parts as locals in dependency order, then build once.
pub struct Renderer { pub width: u32 }
pub struct Game { pub renderer: Renderer, pub assets: Vec<String>, pub frame: u64 }

fn load_assets(r: &Renderer) -> Vec<String> {
    vec![format!("atlas@{}px", r.width)]
}
impl Game {
    pub fn new(width: u32) -> Game {
        let renderer = Renderer { width };
        let assets = load_assets(&renderer);
        Game { renderer, assets, frame: 0 }
    }
}
