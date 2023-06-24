#[derive(Clone)]
pub struct Bot {
    /* General Map Info */
    pub name: String,
    pub class: String,
    pub weapons: Vec<String>,
    pub difficulty: i64,
    pub behavior: String,
    pub weapon_restriction: String,
    pub bot_attributes: Vec<String>,
    pub tags: Vec<String>,
    pub health: String,
    pub scale: f64,
    pub max_vision_range: i64,
    pub class_icon: String,
    pub auto_jump_min: i64,
    pub auto_jump_max: i64,
    pub attributes: Vec<[String; 2]>,
    pub is_boss: bool,
    pub is_giant: bool,
    pub currency_weight: i64,
    pub count: i64,
    pub max_active: i64,
    pub spawn_per_timer: i64,
    pub time_before_spawn: i64,
    pub time_between_spawn: i64,
}
impl Default for Bot {
    fn default() -> Self {
        Bot {
            name: "Scout".to_string(),
            class: "scout".to_string(),
            weapons: vec![],
            difficulty: 1,
            behavior: "".to_string(),
            weapon_restriction: "".to_string(),
            bot_attributes: vec![],
            tags: vec![],
            health: "125".to_string(),
            scale: 1.0,
            max_vision_range: 0,
            class_icon: "scout".to_string(),
            auto_jump_min: 0,
            auto_jump_max: 0,
            attributes: vec![],
            is_boss: false,
            is_giant: false,
            currency_weight: 1,
            count: 10,
            max_active: 10,
            spawn_per_timer: 2,
            time_before_spawn: 0,
            time_between_spawn: 5,
        }
    }
}