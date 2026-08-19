//! Embedded gameplay art with nearest-neighbour sampling.

use macroquad::prelude::*;

/// The production atlas contract. Keeping the names beside the runtime lookup
/// prevents the 8x8 sheet from becoming an undocumented pile of coordinates.
pub const TILE_NAMES: [&str; 64] = [
    "floor_plate_intact_a",
    "floor_plate_intact_b",
    "floor_plate_intact_c",
    "floor_plate_cracked_a",
    "floor_plate_cracked_b",
    "floor_plate_rusted_a",
    "floor_plate_rusted_b",
    "floor_plate_oil_stain",
    "floor_plate_shallow_water",
    "floor_grate",
    "floor_hazard_stripe",
    "floor_collapsed_void",
    "wall_isolated",
    "wall_end_n",
    "wall_end_e",
    "wall_end_s",
    "wall_end_w",
    "wall_straight_ns",
    "wall_straight_ew",
    "wall_corner_ne",
    "wall_corner_es",
    "wall_corner_sw",
    "wall_corner_wn",
    "wall_t_nes",
    "wall_t_esw",
    "wall_t_swn",
    "wall_t_wne",
    "wall_cross",
    "route_straight_ns",
    "route_straight_ew",
    "route_corner_ne",
    "route_corner_es",
    "route_corner_sw",
    "route_corner_wn",
    "route_t_nes",
    "route_t_esw",
    "route_t_swn",
    "route_t_wne",
    "route_cross",
    "route_broken",
    "decal_rubble_light",
    "decal_rubble_medium",
    "decal_rubble_heavy",
    "decal_broken_pipes",
    "decal_loose_cables",
    "decal_scrap_pile",
    "decal_water_leak",
    "decal_scorch_marks",
    "decal_moss",
    "decal_collapsed_beam",
    "decal_repair_scaffold",
    "decal_power_cable",
    "section_scrap_processing",
    "section_water_filtration",
    "section_power_spine",
    "section_assembly_hall",
    "section_logistics_hub",
    "section_hydroponics",
    "section_robotics_bay",
    "section_research_core",
    "section_ai_vault",
    "section_factory_heart",
    "breach_sealed_floor",
    "breach_open_floor",
];

pub const FLOOR_VARIANTS: [&str; 12] = [
    "floor_plate_intact_a",
    "floor_plate_intact_b",
    "floor_plate_intact_c",
    "floor_plate_cracked_a",
    "floor_plate_cracked_b",
    "floor_plate_rusted_a",
    "floor_plate_rusted_b",
    "floor_plate_oil_stain",
    "floor_plate_shallow_water",
    "floor_grate",
    "floor_hazard_stripe",
    "floor_collapsed_void",
];

pub struct GameplayAssets {
    pub tiles: Texture2D,
    pub pads: Texture2D,
    pub core: Texture2D,
    pub towers: Texture2D,
    pub enemies: [Texture2D; 5],
    pub machines: [Texture2D; 9],
}

#[cfg(not(test))]
fn texture(bytes: &[u8]) -> Texture2D {
    let image = Image::from_file_with_format(bytes, Some(ImageFormat::Png))
        .expect("embedded gameplay texture must be valid PNG");
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    texture
}

impl GameplayAssets {
    #[cfg(not(test))]
    pub fn load() -> Option<Self> {
        Some(Self {
            tiles: texture(include_bytes!("../../../assets/map/factory_tiles.png")),
            pads: texture(include_bytes!("../../../assets/map/tower_pads.png")),
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
                texture(include_bytes!(
                    "../../../assets/buildings/scrap_converter.png"
                )),
                texture(include_bytes!(
                    "../../../assets/buildings/water_reclaimer.png"
                )),
                texture(include_bytes!(
                    "../../../assets/buildings/nutrient_vats.png"
                )),
                texture(include_bytes!(
                    "../../../assets/buildings/power_turbine.png"
                )),
                texture(include_bytes!(
                    "../../../assets/buildings/power_distribution_spine.png"
                )),
                texture(include_bytes!(
                    "../../../assets/buildings/assembly_hall_core.png"
                )),
                texture(include_bytes!(
                    "../../../assets/buildings/logistics_hub_core.png"
                )),
                texture(include_bytes!(
                    "../../../assets/buildings/robotics_bay_core.png"
                )),
                texture(include_bytes!(
                    "../../../assets/buildings/research_core.png"
                )),
            ],
        })
    }

    /// Simulation tests do not create a Macroquad graphics context. Keeping
    /// GPU assets absent there lets them exercise the actual gameplay state
    /// instead of failing during texture upload before their first assertion.
    #[cfg(test)]
    pub fn load() -> Option<Self> {
        None
    }
}

pub fn draw_frame(
    texture: &Texture2D,
    frame: usize,
    frame_size: Vec2,
    center: Vec2,
    size: Vec2,
    tint: Color,
) {
    draw_texture_ex(
        texture,
        center.x - size.x * 0.5,
        center.y - size.y * 0.5,
        tint,
        DrawTextureParams {
            dest_size: Some(size),
            source: Some(Rect::new(
                frame as f32 * frame_size.x,
                0.0,
                frame_size.x,
                frame_size.y,
            )),
            ..Default::default()
        },
    );
}

pub fn tile_rect(name: &str) -> Rect {
    let cell = TILE_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .unwrap_or(0);
    Rect::new(
        (cell % 8) as f32 * 64.0,
        (cell / 8) as f32 * 64.0,
        64.0,
        64.0,
    )
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
