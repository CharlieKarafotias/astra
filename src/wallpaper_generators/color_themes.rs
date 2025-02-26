use std::fmt::Display;
use std::sync::LazyLock;

pub(super) struct ColorTheme {
    name: String,
    supports_dark_mode: bool,
    colors: Vec<[u8; 3]>,
    colors_dark_mode: Option<Vec<[u8; 3]>>,
}

impl ColorTheme {
    fn new(
        name: String,
        supports_dark_mode: bool,
        colors: Vec<[u8; 3]>,
        colors_dark_mode: Option<Vec<[u8; 3]>>,
    ) -> Self {
        Self {
            name,
            supports_dark_mode,
            colors,
            colors_dark_mode,
        }
    }

    pub(super) fn get_colors(&self, dark_mode: bool) -> &Vec<[u8; 3]> {
        if dark_mode && self.supports_dark_mode {
            self.colors_dark_mode.as_ref().unwrap_or(&self.colors)
        } else {
            &self.colors
        }
    }
}

impl Display for ColorTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Color Theme: {}, supports dark mode: {}, color_count: {}",
            self.name,
            self.supports_dark_mode,
            self.colors.len()
        )
    }
}

// Created with help from ChatGPT
static THEME_NEON_DREAMS: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Neon Dreams".to_string(),
        true,
        vec![
            [245, 245, 245],
            [58, 12, 163],
            [255, 0, 110],
            [0, 255, 183],
            [255, 221, 51],
        ],
        Some(vec![
            [10, 10, 30],
            [102, 0, 255],
            [255, 51, 153],
            [0, 204, 153],
            [255, 230, 80],
        ]),
    )
});

// Created with help from ChatGPT
static THEME_AURORA_GLOW: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Aurora Glow".to_string(),
        true,
        vec![
            [250, 250, 255],
            [70, 130, 180],
            [144, 238, 144],
            [255, 105, 180],
            [255, 215, 0],
        ],
        Some(vec![
            [5, 5, 20],
            [30, 144, 255],
            [60, 179, 113],
            [255, 20, 147],
            [255, 140, 0],
        ]),
    )
});

// Created with help from ChatGPT
static THEME_CYBER_SUNSET: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Cyber Sunset".to_string(),
        true,
        vec![
            [255, 245, 235],
            [255, 87, 51],
            [255, 153, 51],
            [204, 0, 102],
            [255, 255, 102],
        ],
        Some(vec![
            [20, 10, 5],
            [255, 69, 0],
            [255, 120, 0],
            [153, 0, 76],
            [204, 204, 0],
        ]),
    )
});

// Created with help from ChatGPT
pub(super) static THEME_MYSTIC_FOREST: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Mystic Forest".to_string(),
        true,
        vec![
            [240, 255, 240],
            [34, 139, 34],
            [85, 107, 47],
            [189, 183, 107],
            [152, 251, 152],
        ],
        Some(vec![
            [5, 20, 5],
            [0, 100, 0],
            [46, 64, 33],
            [139, 139, 80],
            [0, 255, 127],
        ]),
    )
});

// Created with help from ChatGPT
pub(super) static THEME_RETRO_POP: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Retro Pop".to_string(),
        true,
        vec![
            [255, 250, 240],
            [255, 69, 96],
            [255, 165, 0],
            [102, 205, 170],
            [147, 112, 219],
        ],
        Some(vec![
            [30, 10, 10],
            [255, 36, 66],
            [255, 120, 0],
            [72, 159, 139],
            [122, 88, 181],
        ]),
    )
});

// Created with help from ChatGPT
pub(super) static THEME_OCEAN_BREEZE: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Ocean Breeze".to_string(),
        true,
        vec![
            [240, 255, 255],
            [0, 191, 255],
            [70, 130, 180],
            [32, 178, 170],
            [175, 238, 238],
        ],
        Some(vec![
            [10, 25, 30],
            [0, 139, 200],
            [50, 100, 160],
            [22, 128, 130],
            [100, 190, 190],
        ]),
    )
});

// Created with help from ChatGPT
pub(super) static THEME_GALAXY_VOYAGE: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Galaxy Voyage".to_string(),
        true,
        vec![
            [245, 245, 255],
            [75, 0, 130],
            [138, 43, 226],
            [255, 20, 147],
            [240, 230, 140],
        ],
        Some(vec![
            [15, 10, 35],
            [148, 0, 211],
            [186, 85, 211],
            [255, 0, 127],
            [189, 183, 107],
        ]),
    )
});

// Created with help from ChatGPT
pub(super) static THEME_FIRE_ICE: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Fire & Ice".to_string(),
        true,
        vec![
            [255, 250, 255],
            [0, 191, 255],
            [135, 206, 250],
            [255, 69, 0],
            [255, 140, 0],
        ],
        Some(vec![
            [5, 5, 15],
            [0, 139, 200],
            [100, 149, 237],
            [255, 36, 0],
            [204, 102, 0],
        ]),
    )
});

// Created with help from ChatGPT
pub(super) static THEME_CANDY_CRUSH: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Candy Crush".to_string(),
        true,
        vec![
            [255, 250, 250],
            [255, 99, 71],
            [255, 182, 193],
            [255, 160, 122],
            [255, 218, 185],
        ],
        Some(vec![
            [30, 10, 10],
            [220, 20, 60],
            [255, 105, 180],
            [210, 105, 30],
            [255, 160, 122],
        ]),
    )
});

// Created with help from ChatGPT
pub(super) static THEME_SUNLIT_MEADOW: LazyLock<ColorTheme> = LazyLock::new(|| {
    ColorTheme::new(
        "Sunlit Meadow".to_string(),
        true,
        vec![
            [250, 255, 245],
            [173, 255, 47],
            [124, 252, 0],
            [255, 222, 173],
            [255, 239, 213],
        ],
        Some(vec![
            [10, 20, 5],
            [154, 205, 50],
            [85, 139, 47],
            [210, 180, 140],
            [245, 222, 179],
        ]),
    )
});

pub(super) static THEMES: LazyLock<[&'static ColorTheme; 10]> = LazyLock::new(|| {
    [
        &*THEME_NEON_DREAMS,
        &*THEME_AURORA_GLOW,
        &*THEME_CYBER_SUNSET,
        &*THEME_MYSTIC_FOREST,
        &*THEME_RETRO_POP,
        &*THEME_OCEAN_BREEZE,
        &*THEME_GALAXY_VOYAGE,
        &*THEME_FIRE_ICE,
        &*THEME_CANDY_CRUSH,
        &*THEME_SUNLIT_MEADOW,
    ]
});
