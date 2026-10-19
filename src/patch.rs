//! Shared sparse-update folding. Publish a batch only after every patch succeeds.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::element::{Element, types};

pub(crate) enum PatchError {
    UnknownRef(String),
    Malformed(String),
}

pub(crate) fn apply(
    elements: &mut [Element],
    patches: &[Map<String, Value>],
) -> Result<(), PatchError> {
    // Stage only touched elements. Repeated refs see the preceding staged patch,
    // while an error leaves the original turn intact on both sides of the wire.
    let mut changed = BTreeMap::new();
    for patch in patches {
        let target = patch
            .get("ref")
            .and_then(Value::as_str)
            .ok_or_else(|| PatchError::Malformed("patch ref must be a string".into()))?;
        let index = elements
            .iter()
            .position(|element| element.r#ref == target)
            .ok_or_else(|| PatchError::UnknownRef(target.into()))?;
        let original = changed.get(&index).unwrap_or(&elements[index]);
        changed.insert(index, merge(original, patch)?);
    }
    for (index, element) in changed {
        elements[index] = element;
    }
    Ok(())
}

fn merge(element: &Element, patch: &Map<String, Value>) -> Result<Element, PatchError> {
    let Value::Object(mut state) =
        serde_json::to_value(element).expect("Element fields are JSON-serializable")
    else {
        unreachable!("Element serializes as an object");
    };
    let folded_append = fold_log_append(&mut state, patch)?;
    for (key, value) in patch {
        if key == "ref" || (folded_append && key == "append") {
            continue;
        }
        if value.is_null() {
            state.remove(key);
        } else {
            state.insert(key.clone(), value.clone());
        }
    }
    serde_json::from_value(Value::Object(state))
        // Error text must not echo a secret or another caller-provided value.
        .map_err(|_| PatchError::Malformed("patch produces invalid element fields".into()))
}

fn fold_log_append(
    state: &mut Map<String, Value>,
    patch: &Map<String, Value>,
) -> Result<bool, PatchError> {
    if state.get("type").and_then(Value::as_str) != Some(types::LOG) {
        return Ok(false);
    }
    let Some(append) = patch.get("append") else {
        return Ok(false);
    };
    let added = append
        .as_array()
        .ok_or_else(|| PatchError::Malformed("log append must be an array".into()))?;
    let lines = state
        .entry("lines")
        .or_insert_with(|| Value::Array(Vec::new()));
    let lines = lines
        .as_array_mut()
        .ok_or_else(|| PatchError::Malformed("log lines must be an array".into()))?;
    lines.extend(added.iter().cloned());
    Ok(true)
}
