use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "type")]
    pub doc_type: Option<String>,
    pub id: String,
    pub path: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Book {
    #[serde(flatten)]
    pub doc: Document,
    pub label: String,
}

#[derive(Serialize, Deserialize)]
pub struct Picture {
    pub id: String,
    pub url: String,
    pub label: String,
    #[serde(rename = "hubUrl")]
    pub hub_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BookInfo {
    pub id: Option<String>,
    pub path: Option<Vec<String>>,
    pub fasc: Option<String>,
    pub page: Option<String>,
    pub detail: String,
}

#[derive(Serialize, Deserialize)]
pub struct Description {
    #[serde(rename = "descriptorId")]
    pub descriptor_id: String,
    #[serde(rename = "statesIds")]
    pub states_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SpecimenLocation {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Measurement {
    pub min: f64,
    pub max: f64,
    pub character: String,
}

#[derive(Serialize, Deserialize)]
pub struct NamedEntity {
    pub name: Option<String>,
    #[serde(rename = "nameEN")]
    pub name_en: Option<String>,
    #[serde(rename = "nameCN")]
    pub name_cn: Option<String>,
    pub photos: Vec<Picture>,
    #[serde(rename = "vernacularName")]
    pub vernacular_name: Option<String>,
    pub color: Option<String>,
    pub detail: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(flatten)]
    pub doc: Document,
    #[serde(flatten)]
    pub entity: NamedEntity,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Hierarchical {
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "topLevel")]
    pub top_level: bool,
    pub children: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Character {
    #[serde(flatten)]
    pub doc: Document,
    #[serde(flatten)]
    pub entity: NamedEntity,
    #[serde(flatten)]
    pub hierarchical: Hierarchical,
    pub preset: Option<serde_json::Value>,
    #[serde(rename = "mapFile")]
    pub map_file: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub unit: Option<String>,
    pub states: Vec<String>,
    #[serde(rename = "characterType")]
    pub character_type: Option<String>,
    #[serde(rename = "inherentStateId")]
    pub inherent_state_id: Option<String>,
    #[serde(rename = "inapplicableStatesIds")]
    pub inapplicable_states_ids: Vec<String>,
    #[serde(rename = "requiredStatesIds")]
    pub required_states_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Taxon {
    #[serde(flatten)]
    pub doc: Document,
    #[serde(flatten)]
    pub entity: NamedEntity,
    #[serde(flatten)]
    pub hierarchical: Hierarchical,
    #[serde(rename = "bookInfoByIds")]
    pub book_info_by_ids: Option<HashMap<String, BookInfo>>,
    #[serde(rename = "specimenLocations")]
    pub specimen_locations: Option<Vec<SpecimenLocation>>,
    pub descriptions: Vec<Description>,
    pub measurements: Option<Vec<Measurement>>,
    pub author: String,
    #[serde(rename = "vernacularName2")]
    pub vernacular_name2: Option<String>,
    #[serde(rename = "name2")]
    pub name2: Option<String>,
    pub meaning: String,
    #[serde(rename = "herbariumPicture")]
    pub herbarium_picture: String,
    pub website: String,
    #[serde(rename = "noHerbier")]
    pub no_herbier: Option<String>,
    pub fasc: Option<String>,
    pub page: Option<String>,
    pub extra: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Dataset {
    pub version: Option<u32>,
    pub id: String,
    pub taxons: Vec<Taxon>,
    pub characters: Vec<Character>,
    pub states: Vec<State>,
    pub books: Vec<Book>,
    #[serde(rename = "extraFields")]
    pub extra_fields: Option<Vec<serde_json::Value>>,
}

fn diff_book(a: &Book, b: &Book) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if let Some(doc_diff) = diff_document(&a.doc, &b.doc) {
        changes.insert("doc".to_string(), json!(doc_diff));
    }
    if a.label != b.label {
        changes.insert("label".to_string(), json!({"old": a.label, "new": b.label}));
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_picture(a: &Picture, b: &Picture) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if a.id != b.id {
        changes.insert("id".to_string(), json!({"old": a.id, "new": b.id}));
    }
    if a.url != b.url {
        changes.insert("url".to_string(), json!({"old": a.url, "new": b.url}));
    }
    if a.label != b.label {
        changes.insert("label".to_string(), json!({"old": a.label, "new": b.label}));
    }
    if a.hub_url != b.hub_url {
        changes.insert(
            "hubUrl".to_string(),
            json!({"old": a.hub_url, "new": b.hub_url}),
        );
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_book_info(a: &BookInfo, b: &BookInfo) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if a.id != b.id {
        changes.insert("id".to_string(), json!({"old": a.id, "new": b.id}));
    }
    if a.path != b.path {
        changes.insert("path".to_string(), json!({"old": a.path, "new": b.path}));
    }
    if a.fasc != b.fasc {
        changes.insert("fasc".to_string(), json!({"old": a.fasc, "new": b.fasc}));
    }
    if a.page != b.page {
        changes.insert("page".to_string(), json!({"old": a.page, "new": b.page}));
    }
    if a.detail != b.detail {
        changes.insert(
            "detail".to_string(),
            json!({"old": a.detail, "new": b.detail}),
        );
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_description(
    a: &Description,
    b: &Description,
) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if a.descriptor_id != b.descriptor_id {
        changes.insert(
            "descriptorId".to_string(),
            json!({"old": a.descriptor_id, "new": b.descriptor_id}),
        );
    }
    if a.states_ids != b.states_ids {
        changes.insert(
            "statesIds".to_string(),
            json!({"old": a.states_ids, "new": b.states_ids}),
        );
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_specimen_location(
    a: &SpecimenLocation,
    b: &SpecimenLocation,
) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if a.lat != b.lat {
        changes.insert("lat".to_string(), json!({"old": a.lat, "new": b.lat}));
    }
    if a.lng != b.lng {
        changes.insert("lng".to_string(), json!({"old": a.lng, "new": b.lng}));
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_measurement(
    a: &Measurement,
    b: &Measurement,
) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if a.min != b.min {
        changes.insert("min".to_string(), json!({"old": a.min, "new": b.min}));
    }
    if a.max != b.max {
        changes.insert("max".to_string(), json!({"old": a.max, "new": b.max}));
    }
    if a.character != b.character {
        changes.insert(
            "character".to_string(),
            json!({"old": a.character, "new": b.character}),
        );
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_document(a: &Document, b: &Document) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if a.doc_type != b.doc_type {
        changes.insert(
            "type".to_string(),
            json!({"old": a.doc_type, "new": b.doc_type}),
        );
    }
    if a.id != b.id {
        changes.insert("id".to_string(), json!({"old": a.id, "new": b.id}));
    }
    if a.path != b.path {
        changes.insert("path".to_string(), json!({"old": a.path, "new": b.path}));
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_named_entity(
    a: &NamedEntity,
    b: &NamedEntity,
) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if a.name != b.name {
        changes.insert("name".to_string(), json!({"old": a.name, "new": b.name}));
    }
    if a.name_en != b.name_en {
        changes.insert(
            "nameEN".to_string(),
            json!({"old": a.name_en, "new": b.name_en}),
        );
    }
    if a.name_cn != b.name_cn {
        changes.insert(
            "nameCN".to_string(),
            json!({"old": a.name_cn, "new": b.name_cn}),
        );
    }
    if a.photos.len() != b.photos.len()
        || a.photos
            .iter()
            .zip(&b.photos)
            .any(|(p1, p2)| diff_picture(p1, p2).is_some())
    {
        let photos_diff: Vec<serde_json::Value> = a
            .photos
            .iter()
            .zip(&b.photos)
            .map(|(p1, p2)| {
                diff_picture(p1, p2)
                    .map(|d| json!(d))
                    .unwrap_or(json!({"kept": p1}))
            })
            .collect();
        changes.insert("photos".to_string(), json!({"diff": photos_diff}));
    }
    if a.vernacular_name != b.vernacular_name {
        changes.insert(
            "vernacularName".to_string(),
            json!({"old": a.vernacular_name, "new": b.vernacular_name}),
        );
    }
    if a.color != b.color {
        changes.insert("color".to_string(), json!({"old": a.color, "new": b.color}));
    }
    if a.detail != b.detail {
        changes.insert(
            "detail".to_string(),
            json!({"old": a.detail, "new": b.detail}),
        );
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_hierarchical(
    a: &Hierarchical,
    b: &Hierarchical,
) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    if a.parent_id != b.parent_id {
        changes.insert(
            "parentId".to_string(),
            json!({"old": a.parent_id, "new": b.parent_id}),
        );
    }
    if a.top_level != b.top_level {
        changes.insert(
            "topLevel".to_string(),
            json!({"old": a.top_level, "new": b.top_level}),
        );
    }
    if a.children != b.children {
        changes.insert(
            "children".to_string(),
            json!({"old": a.children, "new": b.children}),
        );
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_character(a: &Character, b: &Character) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();

    if let Some(doc_diff) = diff_document(&a.doc, &b.doc) {
        changes.extend(doc_diff);
    }
    if let Some(entity_diff) = diff_named_entity(&a.entity, &b.entity) {
        changes.extend(entity_diff);
    }
    if let Some(hier_diff) = diff_hierarchical(&a.hierarchical, &b.hierarchical) {
        changes.extend(hier_diff);
    }
    if a.preset != b.preset {
        changes.insert(
            "preset".to_string(),
            json!({"old": a.preset, "new": b.preset}),
        );
    }
    if a.map_file != b.map_file {
        changes.insert(
            "mapFile".to_string(),
            json!({"old": a.map_file, "new": b.map_file}),
        );
    }
    if a.min != b.min {
        changes.insert("min".to_string(), json!({"old": a.min, "new": b.min}));
    }
    if a.max != b.max {
        changes.insert("max".to_string(), json!({"old": a.max, "new": b.max}));
    }
    if a.unit != b.unit {
        changes.insert("unit".to_string(), json!({"old": a.unit, "new": b.unit}));
    }
    if a.states != b.states {
        changes.insert(
            "states".to_string(),
            json!({"old": a.states, "new": b.states}),
        );
    }
    if a.character_type != b.character_type {
        changes.insert(
            "characterType".to_string(),
            json!({"old": a.character_type, "new": b.character_type}),
        );
    }
    if a.inherent_state_id != b.inherent_state_id {
        changes.insert(
            "inherentStateId".to_string(),
            json!({"old": a.inherent_state_id, "new": b.inherent_state_id}),
        );
    }
    if a.inapplicable_states_ids != b.inapplicable_states_ids {
        changes.insert(
            "inapplicableStatesIds".to_string(),
            json!({"old": a.inapplicable_states_ids, "new": b.inapplicable_states_ids}),
        );
    }
    if a.required_states_ids != b.required_states_ids {
        changes.insert(
            "requiredStatesIds".to_string(),
            json!({"old": a.required_states_ids, "new": b.required_states_ids}),
        );
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_book_info_map(
    a: &Option<HashMap<String, BookInfo>>,
    b: &Option<HashMap<String, BookInfo>>,
) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();
    match (a, b) {
        (Some(a_map), Some(b_map)) => {
            for (k, v) in a_map {
                match b_map.get(k) {
                    Some(bv) => {
                        if let Some(diff) = diff_book_info(v, bv) {
                            changes.insert(k.clone(), json!(diff));
                        }
                    }
                    None => {
                        changes.insert(k.clone(), json!({"removed": k}));
                    }
                }
            }
            for (k, v) in b_map {
                if !a_map.contains_key(k) {
                    changes.insert(k.clone(), json!({"added": v}));
                }
            }
            if changes.is_empty() {
                None
            } else {
                Some(changes)
            }
        }
        (None, None) => None,
        (Some(a_map), None) => {
            for k in a_map.keys() {
                changes.insert(k.clone(), json!({"removed": k}));
            }
            Some(changes)
        }
        (None, Some(b_map)) => {
            for (k, v) in b_map {
                changes.insert(k.clone(), json!({"added": v}));
            }
            Some(changes)
        }
    }
}

fn diff_state(a: &State, b: &State) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();

    if let Some(doc_diff) = diff_document(&a.doc, &b.doc) {
        changes.extend(doc_diff);
    }
    if let Some(entity_diff) = diff_named_entity(&a.entity, &b.entity) {
        changes.extend(entity_diff);
    }
    if a.description != b.description {
        changes.insert(
            "description".to_string(),
            json!({"old": a.description, "new": b.description}),
        );
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_vec<T, F>(a: &[T], b: &[T], diff_fn: F) -> Option<Vec<serde_json::Value>>
where
    T: Serialize,
    F: Fn(&T, &T) -> Option<HashMap<String, serde_json::Value>>,
{
    let mut changes = Vec::new();
    for (av, bv) in a.iter().zip(b.iter()) {
        if let Some(diff) = diff_fn(av, bv) {
            changes.push(json!(diff));
        }
    }
    if !changes.is_empty() {
        Some(changes)
    } else {
        None
    }
}

fn diff_option<T, F>(a: &Option<T>, b: &Option<T>, diff_fn: F) -> Option<serde_json::Value>
where
    T: Serialize,
    F: Fn(&T, &T) -> Option<serde_json::Value>,
{
    match (a.as_ref(), b.as_ref()) {
        (Some(a_val), Some(b_val)) => diff_fn(a_val, b_val),
        (None, None) => None,
        (Some(a_val), None) => Some(json!({"removed": a_val})),
        (None, Some(b_val)) => Some(json!({"added": b_val})),
    }
}

fn diff_option_vec<T, F>(
    a: &Option<Vec<T>>,
    b: &Option<Vec<T>>,
    diff_fn: F,
) -> Option<serde_json::Value>
where
    T: Serialize,
    F: Fn(&T, &T) -> Option<HashMap<String, serde_json::Value>>,
{
    diff_option(a, b, |av, bv| diff_vec(av, bv, &diff_fn).map(|v| json!(v)))
}

fn diff_taxon(a: &Taxon, b: &Taxon) -> Option<HashMap<String, serde_json::Value>> {
    let mut changes = HashMap::new();

    if let Some(doc_diff) = diff_document(&a.doc, &b.doc) {
        changes.extend(doc_diff);
    }
    if let Some(entity_diff) = diff_named_entity(&a.entity, &b.entity) {
        changes.extend(entity_diff);
    }
    if let Some(hier_diff) = diff_hierarchical(&a.hierarchical, &b.hierarchical) {
        changes.extend(hier_diff);
    }
    if let Some(book_info_diff) = diff_book_info_map(&a.book_info_by_ids, &b.book_info_by_ids) {
        changes.insert("bookInfoByIds".to_string(), json!(book_info_diff));
    }
    if let Some(specimen_diff) = diff_option_vec(
        &a.specimen_locations,
        &b.specimen_locations,
        diff_specimen_location,
    ) {
        changes.insert("specimenLocations".to_string(), specimen_diff);
    }
    if let Some(desc_diff) = diff_vec(&a.descriptions, &b.descriptions, diff_description) {
        changes.insert("descriptions".to_string(), json!(desc_diff));
    }
    if let Some(measurement_diff) =
        diff_option_vec(&a.measurements, &b.measurements, diff_measurement)
    {
        changes.insert("measurements".to_string(), measurement_diff);
    }
    if a.author != b.author {
        changes.insert(
            "author".to_string(),
            json!({"old": a.author, "new": b.author}),
        );
    }
    if a.vernacular_name2 != b.vernacular_name2 {
        changes.insert(
            "vernacularName2".to_string(),
            json!({"old": a.vernacular_name2, "new": b.vernacular_name2}),
        );
    }
    if a.name2 != b.name2 {
        changes.insert("name2".to_string(), json!({"old": a.name2, "new": b.name2}));
    }
    if a.meaning != b.meaning {
        changes.insert(
            "meaning".to_string(),
            json!({"old": a.meaning, "new": b.meaning}),
        );
    }
    if a.herbarium_picture != b.herbarium_picture {
        changes.insert(
            "herbariumPicture".to_string(),
            json!({"old": a.herbarium_picture, "new": b.herbarium_picture}),
        );
    }
    if a.website != b.website {
        changes.insert(
            "website".to_string(),
            json!({"old": a.website, "new": b.website}),
        );
    }
    if a.no_herbier != b.no_herbier {
        changes.insert(
            "noHerbier".to_string(),
            json!({"old": a.no_herbier, "new": b.no_herbier}),
        );
    }
    if a.fasc != b.fasc {
        changes.insert("fasc".to_string(), json!({"old": a.fasc, "new": b.fasc}));
    }
    if a.page != b.page {
        changes.insert("page".to_string(), json!({"old": a.page, "new": b.page}));
    }
    if a.extra != b.extra {
        changes.insert("extra".to_string(), json!({"old": a.extra, "new": b.extra}));
    }
    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

pub fn diff_datasets(old: &Dataset, new: &Dataset) -> serde_json::Value {
    fn diff_vec_by_id<T, F, D>(old: &[T], new: &[T], get_id: F, diff_item: D) -> serde_json::Value
    where
        T: Serialize,
        F: Fn(&T) -> &str,
        D: Fn(&T, &T) -> Option<HashMap<String, serde_json::Value>>,
    {
        let old_map: HashMap<&str, &T> = old.iter().map(|x| (get_id(x), x)).collect();
        let new_map: HashMap<&str, &T> = new.iter().map(|x| (get_id(x), x)).collect();

        let old_ids: HashSet<&str> = old_map.keys().copied().collect();
        let new_ids: HashSet<&str> = new_map.keys().copied().collect();

        let added: Vec<&T> = new_ids.difference(&old_ids).map(|id| new_map[id]).collect();
        let removed: Vec<&str> = old_ids.difference(&new_ids).copied().collect();

        let mut modified = Vec::new();

        for id in new_ids.intersection(&old_ids) {
            let old_item = old_map[id];
            let new_item = new_map[id];
            if let Some(diff) = diff_item(old_item, new_item) {
                modified.push(json!({
                    "id": id,
                    "changes": diff
                }));
            }
        }

        json!({
            "added": added,
            "removed": removed,
            "modified": modified,
        })
    }

    json!({
        "taxons": diff_vec_by_id(&old.taxons, &new.taxons, |t| &t.doc.id, diff_taxon),
        "characters": diff_vec_by_id(&old.characters, &new.characters, |c| &c.doc.id, diff_character),
        "states": diff_vec_by_id(&old.states, &new.states, |s| &s.doc.id, diff_state),
        "books": diff_vec_by_id(&old.books, &new.books, |b| &b.doc.id, diff_book),
    })
}

pub fn diff_json_strs(old_json: &str, new_json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let old_dataset: Dataset = serde_json::from_str(old_json)?;
    let new_dataset: Dataset = serde_json::from_str(new_json)?;
    let diff = diff_datasets(&old_dataset, &new_dataset);
    Ok(serde_json::to_string_pretty(&diff)?)
}