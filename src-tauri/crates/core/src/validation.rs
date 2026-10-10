use crate::{model::*, rules::*};

pub fn validate(character: &Character, ruleset: &Ruleset) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if character.id.trim().is_empty() {
        push(
            &mut errors,
            ValidationCode::MissingId,
            "id",
            "identificador obrigatório",
        );
    }
    if character.name.trim().is_empty() {
        push(
            &mut errors,
            ValidationCode::MissingName,
            "name",
            "nome obrigatório",
        );
    }

    match attribute_budget(character.base_attributes) {
        None => push(
            &mut errors,
            ValidationCode::AttributeOutOfRange,
            "base_attributes",
            "atributos base devem estar entre -1 e 4",
        ),
        Some(cost) if cost > ATTRIBUTE_BUDGET => push(
            &mut errors,
            ValidationCode::AttributeBudgetExceeded,
            "base_attributes",
            "custo dos atributos excede 10 pontos",
        ),
        _ => {}
    }

    if character.classes.is_empty() {
        push(
            &mut errors,
            ValidationCode::MissingClass,
            "classes",
            "ao menos uma classe é obrigatória",
        );
    }

    for (index, class) in character.classes.iter().enumerate() {
        if class.levels == 0 {
            push(
                &mut errors,
                ValidationCode::InvalidClassLevel,
                "classes",
                format!("classe {index} deve ter ao menos um nível"),
            );
        }
        if find_class(ruleset, &class.class_id).is_none() {
            push(
                &mut errors,
                ValidationCode::UnknownClass,
                "classes",
                format!("classe desconhecida: {}", class.class_id),
            );
        }
        if let Some(definition) = find_class(ruleset, &class.class_id) {
            if definition.mana_attributes.len() > 1 && class.mana_attribute.is_none() {
                push(
                    &mut errors,
                    ValidationCode::MissingManaAttribute,
                    "classes",
                    format!("classe {} exige atributo de mana", class.class_id),
                );
            }
            if class.mana_attribute.is_some_and(|attribute| {
                definition.mana_attributes.len() <= 1
                    || !definition.mana_attributes.contains(&attribute)
            }) {
                push(
                    &mut errors,
                    ValidationCode::InvalidManaAttribute,
                    "classes",
                    format!("atributo de mana inválido para {}", class.class_id),
                );
            }
        }
        if character.classes[..index]
            .iter()
            .any(|previous| previous.class_id == class.class_id)
        {
            push(
                &mut errors,
                ValidationCode::DuplicateClass,
                "classes",
                format!("classe repetida: {}", class.class_id),
            );
        }
    }

    if !matches!(total_level(character), Some(1..=20)) {
        push(
            &mut errors,
            ValidationCode::InvalidTotalLevel,
            "classes",
            "nível total deve estar entre 1 e 20",
        );
    }

    let attributes = character.base_attributes + character.ancestry.attributes;
    if character.carried_spaces > carrying_limit(attributes.strength) * 2 {
        push(
            &mut errors,
            ValidationCode::LoadLimitExceeded,
            "carried_spaces",
            "carga excede o dobro do limite",
        );
    }

    errors
}

fn push(
    errors: &mut Vec<ValidationError>,
    code: ValidationCode,
    field: &'static str,
    message: impl Into<String>,
) {
    errors.push(ValidationError {
        code,
        field,
        message: message.into(),
    });
}
