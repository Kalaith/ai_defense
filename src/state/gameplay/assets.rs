//! Embedded gameplay art with nearest-neighbour sampling.

use macroquad::prelude::*;

pub struct GameplayAssets {
    pub tiles: Texture2D,
    pub pads: Texture2D,
    pub breaches: Texture2D,
    pub core: Texture2D,
    pub towers: Texture2D,
    pub enemies: [Texture2D; 5],
    pub machines: [Texture2D; 9],
}

fn texture(bytes: &[u8]) -> Texture2D {
    let image = Image::from_file_with_format(bytes, Some(ImageFormat::Png))
        .expect("embedded gameplay texture must be valid PNG");
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    texture
}

impl GameplayAssets {
    pub fn load() -> Self {
        Self {
            tiles: texture(include_bytes!("../../../assets/map/factory_tiles.png")),
            pads: texture(include_bytes!("../../../assets/map/tower_pads.png")),
            breaches: texture(include_bytes!("../../../assets/map/breach_markers.png")),
            core: texture(include_bytes!("../../../assets/buildings/factory_core.png")),
            towers: texture(include_bytes!("../../../assets/towers/towers.png")),
            enemies: [
                texture(include_bytes!("../../../assets/enemies/scout_drone.png")),
                texture(include_bytes!("../../../assets/enemies/assault_drone.png")),
                texture(include_bytes!("../../../assets/enemies/siege_walker.png")),
                texture(include_bytes!("../../../assets/enemies/infiltrator.png")),
                texture(include_bytes!("../../../assets/enemies/command_node.png")),
            ],
            machines: [
                texture(include_bytes!("../../../assets/buildings/scrap_converter.png")),
                texture(include_bytes!("../../../assets/buildings/water_reclaimer.png")),
                texture(include_bytes!("../../../assets/buildings/nutrient_vats.png")),
                texture(include_bytes!("../../../assets/buildings/power_turbine.png")),
                texture(include_bytes!("../../../assets/buildings/power_distribution_spine.png")),
                texture(include_bytes!("../../../assets/buildings/assembly_hall_core.png")),
                texture(include_bytes!("../../../assets/buildings/logistics_hub_core.png")),
                texture(include_bytes!("../../../assets/buildings/robotics_bay_core.png")),
                texture(include_bytes!("../../../assets/buildings/research_core.png")),
            ],
        }
    }
}

pub fn draw_frame(texture: &Texture2D, frame: usize, frame_size: Vec2, center: Vec2, size: Vec2, tint: Color) {
    draw_texture_ex(
        texture,
        center.x - size.x * 0.5,
        center.y - size.y * 0.5,
        tint,
        DrawTextureParams {
            dest_size: Some(size),
            source: Some(Rect::new(frame as f32 * frame_size.x, 0.0, frame_size.x, frame_size.y)),
            ..Default::default()
        },
    );
}

pub fn machine_index(kind: &str) -> usize {
    match kind {
        "scrap_converter" => 0,
        "water_reclaimer" => 1,
        "nutrient_vats" => 2,
        "power_turbine" => 3,
        "power_distribution_spine" => 4,
        "assembly_hall_core" => 5,
        "logistics_hub_core" => 6,
        "robotics_bay_core" => 7,
        "research_core" => 8,
        _ => 0,
    }
}
