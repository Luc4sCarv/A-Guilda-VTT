#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Attributes {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl Attributes {
    pub const fn get(self, attribute: Attribute) -> i32 {
        match attribute {
            Attribute::Strength => self.strength,
            Attribute::Dexterity => self.dexterity,
            Attribute::Constitution => self.constitution,
            Attribute::Intelligence => self.intelligence,
            Attribute::Wisdom => self.wisdom,
            Attribute::Charisma => self.charisma,
        }
    }

    pub const fn values(self) -> [i32; 6] {
        [
            self.strength,
            self.dexterity,
            self.constitution,
            self.intelligence,
            self.wisdom,
            self.charisma,
        ]
    }
}

impl std::ops::Add for Attributes {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            strength: self.strength + other.strength,
            dexterity: self.dexterity + other.dexterity,
            constitution: self.constitution + other.constitution,
            intelligence: self.intelligence + other.intelligence,
            wisdom: self.wisdom + other.wisdom,
            charisma: self.charisma + other.charisma,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attribute {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    Tiny,
    Small,
    #[default]
    Medium,
    Large,
    Huge,
    Colossal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeModifiers {
    pub occupied_squares: u8,
    pub natural_reach_squares: u8,
    pub stealth: i32,
    pub maneuver: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassDefinition {
    pub id: String,
    pub initial_hp: i32,
    pub hp_per_level: i32,
    pub mp_per_level: i32,
    pub mana_attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassLevel {
    pub class_id: String,
    pub levels: u8,
    pub mana_attribute: Option<Attribute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ruleset {
    pub classes: Vec<ClassDefinition>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AncestryBonuses {
    pub attributes: Attributes,
    pub initial_hp: i32,
    pub hp_per_level: i32,
    pub movement_meters: Option<i32>,
    pub defense: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EquipmentBonuses {
    pub armor_defense: i32,
    pub shield_defense: i32,
    pub armor_penalty: i32,
    pub heavy_armor: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub base_attributes: Attributes,
    pub ancestry: AncestryBonuses,
    pub classes: Vec<ClassLevel>,
    pub equipment: EquipmentBonuses,
    pub other_defense: i32,
    pub manual_hp_adjustment: i32,
    pub manual_mp_adjustment: i32,
    pub size: Size,
    pub carried_spaces: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivedStats {
    pub attributes: Attributes,
    pub level: u8,
    pub max_hp: i32,
    pub max_mp: i32,
    pub defense: i32,
    pub movement_meters: i32,
    pub carrying_limit: u32,
    pub load: LoadState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceState {
    pub current: i32,
    pub maximum: i32,
    pub temporary: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillCheck {
    pub skill: Skill,
    pub use_kind: SkillUse,
    pub trained: bool,
    pub other_bonus: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillUse {
    General,
    Swimming,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skill {
    Acrobatics,
    AnimalHandling,
    Athletics,
    Performance,
    Riding,
    Knowledge,
    Healing,
    Diplomacy,
    Deception,
    Fortitude,
    Stealth,
    Warfare,
    Initiative,
    Intimidation,
    Insight,
    Investigation,
    Gambling,
    Thievery,
    Melee,
    Mysticism,
    Nobility,
    Craft,
    Perception,
    Piloting,
    Ranged,
    Reflexes,
    Religion,
    Survival,
    Will,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadState {
    Normal,
    Overloaded,
    OverLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpellCircle {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationCode {
    MissingId,
    MissingName,
    AttributeOutOfRange,
    AttributeBudgetExceeded,
    MissingClass,
    UnknownClass,
    DuplicateClass,
    InvalidClassLevel,
    InvalidTotalLevel,
    MissingManaAttribute,
    InvalidManaAttribute,
    LoadLimitExceeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationError {
    pub code: ValidationCode,
    pub field: &'static str,
    pub message: String,
}
