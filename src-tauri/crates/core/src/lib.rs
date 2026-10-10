mod model;
mod rules;
mod validation;

pub use model::*;
pub use rules::*;
pub use validation::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn ruleset() -> Ruleset {
        Ruleset {
            classes: vec![
                ClassDefinition {
                    id: "guerreiro".into(),
                    initial_hp: 20,
                    hp_per_level: 5,
                    mp_per_level: 3,
                    mana_attributes: vec![],
                },
                ClassDefinition {
                    id: "arcanista".into(),
                    initial_hp: 8,
                    hp_per_level: 2,
                    mp_per_level: 6,
                    mana_attributes: vec![Attribute::Intelligence, Attribute::Charisma],
                },
            ],
        }
    }

    fn character() -> Character {
        Character {
            id: "1".into(),
            name: "Lisandra".into(),
            base_attributes: Attributes {
                strength: 2,
                dexterity: 1,
                constitution: 2,
                intelligence: 1,
                wisdom: 0,
                charisma: -1,
            },
            ancestry: AncestryBonuses::default(),
            classes: vec![ClassLevel {
                class_id: "guerreiro".into(),
                levels: 1,
                mana_attribute: None,
            }],
            equipment: EquipmentBonuses::default(),
            other_defense: 0,
            manual_hp_adjustment: 0,
            manual_mp_adjustment: 0,
            size: Size::Medium,
            carried_spaces: 0,
        }
    }

    #[test]
    fn validates_attribute_purchase() {
        let mut character = character();
        assert!(validate(&character, &ruleset()).is_empty());

        character.base_attributes.charisma = 4;
        assert!(
            validate(&character, &ruleset())
                .iter()
                .any(|error| error.code == ValidationCode::AttributeBudgetExceeded)
        );
    }

    #[test]
    fn calculates_multiclass_resources_and_heavy_armor() {
        let mut character = character();
        character.classes[0].levels = 2;
        character.classes.push(ClassLevel {
            class_id: "arcanista".into(),
            levels: 1,
            mana_attribute: Some(Attribute::Intelligence),
        });
        character.equipment = EquipmentBonuses {
            armor_defense: 6,
            armor_penalty: -2,
            heavy_armor: true,
            ..EquipmentBonuses::default()
        };

        let stats = calculate(&character, &ruleset()).unwrap();
        assert_eq!(stats.level, 3);
        assert_eq!(stats.max_hp, 33);
        assert_eq!(stats.max_mp, 13);
        assert_eq!(stats.defense, 16);
        assert_eq!(stats.movement_meters, 6);
    }

    #[test]
    fn calculates_skills_and_preserves_full_resources() {
        let bonus = skill_bonus(
            7,
            Attributes {
                dexterity: 3,
                ..Attributes::default()
            },
            EquipmentBonuses {
                armor_penalty: -2,
                ..EquipmentBonuses::default()
            },
            SkillCheck {
                skill: Skill::Acrobatics,
                use_kind: SkillUse::General,
                trained: true,
                other_bonus: 1,
            },
        );
        assert_eq!(bonus, 9);

        assert_eq!(
            adjust_resource(
                ResourceState {
                    current: 20,
                    maximum: 20,
                    temporary: 3
                },
                25
            ),
            ResourceState {
                current: 25,
                maximum: 25,
                temporary: 3
            }
        );
    }

    #[test]
    fn applies_inventory_magic_size_and_temporary_resource_rules() {
        assert_eq!(carrying_limit(2), 14);
        assert_eq!(load_state(20, 14), LoadState::Overloaded);
        assert_eq!(load_state(29, 14), LoadState::OverLimit);
        assert_eq!(size_modifiers(Size::Large).maneuver, 2);
        assert_eq!(spell_base_cost(SpellCircle::Fourth), 10);
        assert_eq!(spell_dc(8, 5, 0), 19);
        assert_eq!(arcane_armor_check_dc(3), 23);
        assert!(!can_spend_mana(2, 3));

        let resource = ResourceState {
            current: 10,
            maximum: 10,
            temporary: 4,
        };
        assert_eq!(
            take_damage(resource, 6),
            ResourceState {
                current: 8,
                maximum: 10,
                temporary: 0,
            }
        );
        assert_eq!(
            spend_mana(resource, 12).unwrap(),
            ResourceState {
                current: 2,
                maximum: 10,
                temporary: 0,
            }
        );
    }
}
