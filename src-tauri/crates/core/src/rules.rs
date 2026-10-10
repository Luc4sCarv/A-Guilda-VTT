use crate::model::*;

pub const ATTRIBUTE_BUDGET: i32 = 10;

pub fn base_ruleset() -> Ruleset {
    use Attribute::*;

    Ruleset {
        classes: vec![
            class("arcanista", 8, 2, 6, vec![Intelligence, Charisma]),
            class("barbaro", 24, 6, 3, vec![]),
            class("bardo", 12, 3, 4, vec![Charisma]),
            class("bucaneiro", 16, 4, 3, vec![]),
            class("cacador", 16, 4, 4, vec![]),
            class("cavaleiro", 20, 5, 3, vec![]),
            class("clerigo", 16, 4, 5, vec![Wisdom]),
            class("druida", 16, 4, 4, vec![Wisdom]),
            class("guerreiro", 20, 5, 3, vec![]),
            class("inventor", 12, 3, 4, vec![]),
            class("ladino", 12, 3, 4, vec![]),
            class("lutador", 20, 5, 3, vec![]),
            class("nobre", 16, 4, 4, vec![]),
            class("paladino", 20, 5, 3, vec![Charisma]),
        ],
    }
}

pub const fn attribute_cost(value: i32) -> Option<i32> {
    match value {
        -1 => Some(-1),
        0 => Some(0),
        1 => Some(1),
        2 => Some(2),
        3 => Some(4),
        4 => Some(7),
        _ => None,
    }
}

pub fn attribute_budget(attributes: Attributes) -> Option<i32> {
    attributes
        .values()
        .into_iter()
        .map(attribute_cost)
        .try_fold(0, |total, cost| cost.map(|cost| total + cost))
}

pub fn total_level(character: &Character) -> Option<u8> {
    character
        .classes
        .iter()
        .try_fold(0_u8, |total, class| total.checked_add(class.levels))
}

pub fn calculate(character: &Character, ruleset: &Ruleset) -> Option<DerivedStats> {
    let level = total_level(character)?;
    let attributes = character.base_attributes + character.ancestry.attributes;
    let constitution = attributes.constitution;
    let first = character.classes.first()?;
    let first_definition = find_class(ruleset, &first.class_id)?;

    let mut max_hp = first_definition.initial_hp + constitution;
    max_hp += (i32::from(first.levels) - 1) * (first_definition.hp_per_level + constitution).max(1);

    let mut max_mp = first_definition.mp_per_level * i32::from(first.levels);
    let mut mana_attributes = Vec::new();
    if let Some(attribute) = mana_attribute(first, first_definition) {
        mana_attributes.push(attribute);
    }

    for class in character.classes.iter().skip(1) {
        let definition = find_class(ruleset, &class.class_id)?;
        max_hp += i32::from(class.levels) * (definition.hp_per_level + constitution).max(1);
        max_mp += i32::from(class.levels) * definition.mp_per_level;
        if let Some(attribute) = mana_attribute(class, definition)
            && !mana_attributes.contains(&attribute)
        {
            mana_attributes.push(attribute);
        }
    }

    max_hp += character.ancestry.initial_hp
        + character.ancestry.hp_per_level * i32::from(level)
        + character.manual_hp_adjustment;
    max_mp += mana_attributes
        .into_iter()
        .map(|attribute| attributes.get(attribute))
        .sum::<i32>()
        + character.manual_mp_adjustment;

    let carrying_limit = carrying_limit(attributes.strength);
    let load = load_state(character.carried_spaces, carrying_limit);
    Some(DerivedStats {
        attributes,
        level,
        max_hp: max_hp.max(0),
        max_mp: max_mp.max(0),
        defense: 10
            + if character.equipment.heavy_armor {
                0
            } else {
                attributes.dexterity
            }
            + character.equipment.armor_defense
            + character.equipment.shield_defense
            + character.ancestry.defense
            + character.other_defense,
        movement_meters: (character.ancestry.movement_meters.unwrap_or(9)
            - if character.equipment.heavy_armor {
                3
            } else {
                0
            }
            - if load == LoadState::Overloaded { 3 } else { 0 })
        .max(0),
        carrying_limit,
        load,
    })
}

pub fn skill_bonus(
    level: u8,
    attributes: Attributes,
    equipment: EquipmentBonuses,
    check: SkillCheck,
) -> i32 {
    let half_level = i32::from(level / 2);
    let training = if check.trained {
        match level {
            1..=6 => 2,
            7..=14 => 4,
            _ => 6,
        }
    } else {
        0
    };
    let armor = if skill_uses_armor_penalty(check.skill, check.use_kind) {
        equipment.armor_penalty
    } else {
        0
    };

    attributes.get(skill_attribute(check.skill)) + half_level + training + check.other_bonus + armor
}

pub const fn skill_attribute(skill: Skill) -> Attribute {
    use Attribute::*;
    use Skill::*;
    match skill {
        Athletics | Melee => Strength,
        Acrobatics | Riding | Stealth | Initiative | Thievery | Piloting | Ranged | Reflexes => {
            Dexterity
        }
        Fortitude => Constitution,
        Knowledge | Warfare | Investigation | Mysticism | Nobility | Craft => Intelligence,
        Healing | Insight | Perception | Religion | Survival | Will => Wisdom,
        AnimalHandling | Performance | Diplomacy | Deception | Intimidation | Gambling => Charisma,
    }
}

pub const fn skill_requires_training(skill: Skill) -> bool {
    use Skill::*;
    matches!(
        skill,
        AnimalHandling
            | Performance
            | Knowledge
            | Warfare
            | Gambling
            | Thievery
            | Mysticism
            | Nobility
            | Craft
            | Piloting
            | Religion
    )
}

pub const fn skill_uses_armor_penalty(skill: Skill, use_kind: SkillUse) -> bool {
    matches!(skill, Skill::Acrobatics | Skill::Stealth | Skill::Thievery)
        || matches!((skill, use_kind), (Skill::Athletics, SkillUse::Swimming))
}

pub fn carrying_limit(strength: i32) -> u32 {
    if strength >= 0 {
        10_u32.saturating_add((strength as u32).saturating_mul(2))
    } else {
        strength.saturating_add(10).max(0) as u32
    }
}

pub const fn load_state(spaces: u32, limit: u32) -> LoadState {
    if spaces <= limit {
        LoadState::Normal
    } else if spaces <= limit * 2 {
        LoadState::Overloaded
    } else {
        LoadState::OverLimit
    }
}

pub const fn size_modifiers(size: Size) -> SizeModifiers {
    match size {
        Size::Tiny => SizeModifiers {
            occupied_squares: 1,
            natural_reach_squares: 1,
            stealth: 5,
            maneuver: -5,
        },
        Size::Small => SizeModifiers {
            occupied_squares: 1,
            natural_reach_squares: 1,
            stealth: 2,
            maneuver: -2,
        },
        Size::Medium => SizeModifiers {
            occupied_squares: 1,
            natural_reach_squares: 1,
            stealth: 0,
            maneuver: 0,
        },
        Size::Large => SizeModifiers {
            occupied_squares: 2,
            natural_reach_squares: 2,
            stealth: -2,
            maneuver: 2,
        },
        Size::Huge => SizeModifiers {
            occupied_squares: 3,
            natural_reach_squares: 3,
            stealth: -5,
            maneuver: 5,
        },
        Size::Colossal => SizeModifiers {
            occupied_squares: 6,
            natural_reach_squares: 6,
            stealth: -10,
            maneuver: 10,
        },
    }
}

pub const fn spell_base_cost(circle: SpellCircle) -> u8 {
    match circle {
        SpellCircle::First => 1,
        SpellCircle::Second => 3,
        SpellCircle::Third => 6,
        SpellCircle::Fourth => 10,
        SpellCircle::Fifth => 15,
    }
}

pub const fn spell_dc(level: u8, key_attribute: i32, other_bonus: i32) -> i32 {
    10 + level as i32 / 2 + key_attribute + other_bonus
}

pub const fn arcane_armor_check_dc(mana_cost: u8) -> i32 {
    20 + mana_cost as i32
}

pub const fn can_spend_mana(class_level: u8, mana_cost: u8) -> bool {
    mana_cost <= class_level
}

pub fn adjust_resource(old: ResourceState, new_maximum: i32) -> ResourceState {
    let maximum = new_maximum.max(0);
    ResourceState {
        current: if old.current == old.maximum {
            maximum
        } else {
            old.current.min(maximum)
        },
        maximum,
        temporary: old.temporary.max(0),
    }
}

pub fn take_damage(mut state: ResourceState, damage: i32) -> ResourceState {
    let damage = damage.max(0);
    let absorbed = state.temporary.min(damage);
    state.temporary -= absorbed;
    state.current -= damage - absorbed;
    state
}

pub fn spend_mana(mut state: ResourceState, cost: i32) -> Option<ResourceState> {
    if cost < 0 || state.current + state.temporary < cost {
        return None;
    }
    let temporary = state.temporary.min(cost);
    state.temporary -= temporary;
    state.current -= cost - temporary;
    Some(state)
}

pub(crate) fn find_class<'a>(ruleset: &'a Ruleset, id: &str) -> Option<&'a ClassDefinition> {
    ruleset.classes.iter().find(|class| class.id == id)
}

pub(crate) fn mana_attribute(
    class: &ClassLevel,
    definition: &ClassDefinition,
) -> Option<Attribute> {
    match definition.mana_attributes.as_slice() {
        [] => None,
        [attribute] => Some(*attribute),
        choices => class
            .mana_attribute
            .filter(|attribute| choices.contains(attribute)),
    }
}

fn class(
    id: &str,
    initial_hp: i32,
    hp_per_level: i32,
    mp_per_level: i32,
    mana_attributes: Vec<Attribute>,
) -> ClassDefinition {
    ClassDefinition {
        id: id.into(),
        initial_hp,
        hp_per_level,
        mp_per_level,
        mana_attributes,
    }
}
