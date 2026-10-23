// A9: a player that the camera and the HUD both change.
pub struct Player { pub hp: i32, pub inventory: Vec<String> }
pub struct Camera<'a> { pub target: &'a mut Player }
pub struct Hud<'a> { pub target: &'a mut Player }

pub fn frame() {
    let mut p = Player { hp: 100, inventory: Vec::new() };
    let cam = Camera { target: &mut p };
    let hud = Hud { target: &mut p };
    cam.target.hp -= 1;
    hud.target.inventory.push("key".to_string());
}
