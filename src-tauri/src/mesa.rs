use std::{collections::HashMap, fs, io::Cursor, net::UdpSocket, path::{Path as FilePath, PathBuf}, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use axum::{body::{to_bytes, Body}, extract::{Path, State, ws::{Message, WebSocket, WebSocketUpgrade}}, http::{header, HeaderMap, StatusCode}, response::Response, routing::{get, post}, Router};
use futures_util::{SinkExt, StreamExt};
use image::{ImageFormat, ImageReader};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{Manager, State as TauriState};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use tormenta20_core::{take_damage, ResourceState};
use uuid::Uuid;

#[derive(Default)]
pub struct MesaState {
    running: Mutex<Option<RunningRoom>>,
}

struct RunningRoom {
    shutdown: oneshot::Sender<()>,
    server: Arc<RoomServer>,
}

struct RoomServer {
    id: String,
    name: String,
    code: String,
    master_token: String,
    db: PathBuf,
    assets: PathBuf,
    live: Mutex<LiveRoom>,
    stop: watch::Sender<bool>,
}

#[derive(Default)]
struct LiveRoom {
    clients: HashMap<String, mpsc::UnboundedSender<Message>>,
    offers: HashMap<String, Value>,
    rolls: HashMap<String, PendingRoll>,
}

struct PendingRoll {
    sheet_id: String,
    formula: String,
    rolls: Vec<u32>,
    total: i32,
    damage_type: String,
    magical: bool,
    life_loss: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalCharacter {
    id: String,
    name: String,
    data: Value,
    updated_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfo {
    id: String,
    name: String,
    cover_id: Option<String>,
    last_opened_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomCover {
    bytes: Vec<u8>,
    mime: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    id: String,
    name: String,
    address: String,
    port: u16,
    code: String,
    token: String,
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}
fn now_ms() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64 }

fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("tormenta-vtt.sqlite"))
}

fn assets_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("assets");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn open_db(path: &PathBuf) -> Result<Connection, String> {
    let db = Connection::open(path).map_err(|e| e.to_string())?;
    db.execute_batch("PRAGMA foreign_keys=ON;
        CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS local_characters(id TEXT PRIMARY KEY, name TEXT NOT NULL, data TEXT NOT NULL, updated_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS rooms(id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS participants(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, name TEXT NOT NULL, token TEXT NOT NULL UNIQUE, approved INTEGER NOT NULL DEFAULT 0, revoked INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS room_characters(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, owner_token TEXT NOT NULL, data TEXT NOT NULL, revision INTEGER NOT NULL DEFAULT 1);
        CREATE TABLE IF NOT EXISTS sheet_change_requests(id TEXT PRIMARY KEY, sheet_id TEXT NOT NULL UNIQUE, room_id TEXT NOT NULL, owner_token TEXT NOT NULL, base_revision INTEGER NOT NULL, previous TEXT NOT NULL, proposed TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS chat(id INTEGER PRIMARY KEY AUTOINCREMENT, room_id TEXT NOT NULL, author TEXT NOT NULL, text TEXT NOT NULL, created_at INTEGER NOT NULL, audience TEXT NOT NULL DEFAULT 'all', owner_token TEXT NOT NULL DEFAULT '', kind TEXT NOT NULL DEFAULT 'message', details TEXT NOT NULL DEFAULT '{}');
        CREATE TABLE IF NOT EXISTS room_maps(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, name TEXT NOT NULL, file_name TEXT NOT NULL, width INTEGER NOT NULL, height INTEGER NOT NULL, grid_visible INTEGER NOT NULL DEFAULT 0, snap INTEGER NOT NULL DEFAULT 0, cell_px INTEGER NOT NULL DEFAULT 64, metres_per_cell REAL NOT NULL DEFAULT 1.5, kind TEXT NOT NULL DEFAULT 'tactical');
        CREATE TABLE IF NOT EXISTS room_token_art(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, name TEXT NOT NULL, file_name TEXT NOT NULL, width INTEGER NOT NULL, height INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS room_audio(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, owner_token TEXT NOT NULL, name TEXT NOT NULL, file_name TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS room_playlist(room_id TEXT NOT NULL, position INTEGER NOT NULL, audio_id TEXT NOT NULL, PRIMARY KEY(room_id,position));
        CREATE TABLE IF NOT EXISTS room_playback(room_id TEXT PRIMARY KEY, audio_id TEXT, position_seconds REAL NOT NULL DEFAULT 0, started_at INTEGER NOT NULL DEFAULT 0, playing INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS room_tokens(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, map_id TEXT NOT NULL, art_id TEXT, sheet_id TEXT, name TEXT NOT NULL, owner_participant_id TEXT, creator_participant_id TEXT, x REAL NOT NULL, y REAL NOT NULL, size REAL NOT NULL, revision INTEGER NOT NULL DEFAULT 1);
        CREATE TABLE IF NOT EXISTS room_threats(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, data TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS room_threat_instances(token_id TEXT PRIMARY KEY, room_id TEXT NOT NULL, threat_id TEXT NOT NULL, current_hp INTEGER NOT NULL, current_mp INTEGER NOT NULL, conditions TEXT NOT NULL DEFAULT '[]', revision INTEGER NOT NULL DEFAULT 1);
        CREATE TABLE IF NOT EXISTS room_combat(room_id TEXT PRIMARY KEY, map_id TEXT NOT NULL, state TEXT NOT NULL, revision INTEGER NOT NULL DEFAULT 1);
        CREATE TABLE IF NOT EXISTS room_fog(map_id TEXT PRIMARY KEY, room_id TEXT NOT NULL, state TEXT NOT NULL, revision INTEGER NOT NULL DEFAULT 1);
        CREATE TABLE IF NOT EXISTS room_marks(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, map_id TEXT NOT NULL, kind TEXT NOT NULL, owner_token TEXT NOT NULL, author TEXT NOT NULL, visibility TEXT NOT NULL DEFAULT 'public', data TEXT NOT NULL, revision INTEGER NOT NULL DEFAULT 1);
        CREATE TABLE IF NOT EXISTS room_documents(id TEXT PRIMARY KEY, room_id TEXT NOT NULL, kind TEXT NOT NULL, title TEXT NOT NULL, body TEXT NOT NULL, visibility TEXT NOT NULL DEFAULT 'private');")
        .map_err(|e| e.to_string())?;
    for (column,definition) in [("audience","TEXT NOT NULL DEFAULT 'all'"),("owner_token","TEXT NOT NULL DEFAULT ''"),("kind","TEXT NOT NULL DEFAULT 'message'"),("details","TEXT NOT NULL DEFAULT '{}'")] {
        let exists:i64=db.query_row("SELECT COUNT(*) FROM pragma_table_info('chat') WHERE name=?1",[column],|r|r.get(0)).map_err(|e|e.to_string())?;
        if exists==0 {
            db.execute_batch(&format!("ALTER TABLE chat ADD COLUMN {column} {definition}")).map_err(|e|e.to_string())?;
            if column=="kind" {db.execute("UPDATE chat SET audience='self' WHERE audience='secret'",[]).map_err(|e|e.to_string())?;}
        }
    }
    let has_permission:i64=db.query_row("SELECT COUNT(*) FROM pragma_table_info('room_characters') WHERE name='can_edit_proficiencies'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if has_permission==0 {db.execute_batch("ALTER TABLE room_characters ADD COLUMN can_edit_proficiencies INTEGER NOT NULL DEFAULT 0").map_err(|e|e.to_string())?;}
    let has_active:i64=db.query_row("SELECT COUNT(*) FROM pragma_table_info('rooms') WHERE name='active_map_id'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if has_active==0 {db.execute_batch("ALTER TABLE rooms ADD COLUMN active_map_id TEXT").map_err(|e|e.to_string())?;}
    for (column,definition) in [("cover_file_name","TEXT"),("last_opened_at","INTEGER NOT NULL DEFAULT 0")] {
        let exists:i64=db.query_row("SELECT COUNT(*) FROM pragma_table_info('rooms') WHERE name=?1",[column],|r|r.get(0)).map_err(|e|e.to_string())?;
        if exists==0 {db.execute_batch(&format!("ALTER TABLE rooms ADD COLUMN {column} {definition}")).map_err(|e|e.to_string())?;}
    }
    db.execute("UPDATE rooms SET last_opened_at=created_at WHERE last_opened_at=0",[]).map_err(|e|e.to_string())?;
    let has_art_owner:i64=db.query_row("SELECT COUNT(*) FROM pragma_table_info('room_token_art') WHERE name='owner_token'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if has_art_owner==0 {db.execute_batch("ALTER TABLE room_token_art ADD COLUMN owner_token TEXT NOT NULL DEFAULT ''").map_err(|e|e.to_string())?;}
    let has_creator:i64=db.query_row("SELECT COUNT(*) FROM pragma_table_info('room_tokens') WHERE name='creator_participant_id'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if has_creator==0 {db.execute_batch("ALTER TABLE room_tokens ADD COLUMN creator_participant_id TEXT").map_err(|e|e.to_string())?;}
    let has_scene_kind:i64=db.query_row("SELECT COUNT(*) FROM pragma_table_info('room_maps') WHERE name='kind'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if has_scene_kind==0 {db.execute_batch("ALTER TABLE room_maps ADD COLUMN kind TEXT NOT NULL DEFAULT 'tactical'").map_err(|e|e.to_string())?;}
    let has_backdrop:i64=db.query_row("SELECT COUNT(*) FROM pragma_table_info('room_maps') WHERE name='backdrop_color'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
    if has_backdrop==0 {db.execute_batch("ALTER TABLE room_maps ADD COLUMN backdrop_color TEXT").map_err(|e|e.to_string())?;}
    Ok(db)
}

fn validate_sheet(data: &Value) -> Result<String, String> {
    let obj = data.as_object().ok_or("Ficha inválida")?;
    let name = obj.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() || name.chars().count() > 100 { return Err("Nome da ficha deve ter de 1 a 100 caracteres".into()); }
    let level = match obj.get("level") { Some(value) => value.as_i64().ok_or("Nível inválido")?, None => 1 };
    if !(1..=20).contains(&level) { return Err("Nível inválido".into()); }
    if let Some(attrs) = obj.get("baseAttributes") {
        let attrs = attrs.as_array().ok_or("Atributos inválidos")?;
        if attrs.len() != 6 || attrs.iter().any(|v| !matches!(v.as_i64(), Some(-1..=4))) { return Err("Atributos inválidos".into()); }
        let cost: i32 = attrs.iter().map(|v| tormenta20_core::attribute_cost(v.as_i64().unwrap() as i32).unwrap()).sum();
        if cost > tormenta20_core::ATTRIBUTE_BUDGET { return Err("A compra de atributos excede os 10 pontos disponíveis".into()); }
    }
    if let Some(choices) = obj.get("ancestryChoices") {
        let choices = choices.as_array().ok_or("Escolhas raciais inválidas")?;
        let ancestry = obj.get("ancestry").and_then(Value::as_str).unwrap_or("");
        let limit = if ["Humano", "Lefou", "Sereia/Tritão"].contains(&ancestry) { 3 } else { 0 };
        if choices.len() > limit || choices.iter().enumerate().any(|(i, v)|
            !matches!(v.as_i64(), Some(0..=5)) || choices[..i].contains(v) || (ancestry == "Lefou" && v.as_i64() == Some(5))) {
            return Err("Escolhas de atributos raciais inválidas".into());
        }
    }
    if let Some(values) = obj.get("temporaryAttributes") {
        let values = values.as_array().ok_or("Ajustes temporários inválidos")?;
        if values.len() != 6 || values.iter().any(|v| !matches!(v.as_i64(), Some(-20..=20))) { return Err("Ajustes temporários inválidos".into()); }
    }
    for key in ["currentHp", "temporaryHp", "currentMp", "temporaryMp"] {
        if let Some(value) = obj.get(key) {
            let n = value.as_i64().ok_or_else(|| format!("{key} inválido"))?;
            let minimum = if key == "currentHp" { -1_000_000 } else { 0 };
            if !(minimum..=1_000_000).contains(&n) { return Err(format!("{key} fora do limite")); }
        }
    }
    if data.get("maximumHp").is_some_and(|n|n.as_i64().is_none_or(|n|!(0..=1_000_000).contains(&n))){return Err("PV máximo inválido".into());}
    for key in ["manualRd", "otherMovement", "skillAttributes"] {
        if let Some(value) = obj.get(key) {
            let entries = value.as_object().ok_or_else(|| format!("{key} inválido"))?;
            let maximum = if key == "skillAttributes" { 5 } else { 1_000_000 };
            if entries.len() > 100 || entries.values().any(|v| v.as_i64().is_none_or(|n| !(0..=maximum).contains(&n))) { return Err(format!("{key} inválido")); }
        }
    }
    if data["ancestry"]=="Golem" && !["","Água","Ar","Fogo","Terra"].contains(&data["golemElement"].as_str().unwrap_or("")){return Err("Fonte elemental inválida".into());}
    if ["notes","diary"].iter().any(|key|obj.get(*key).and_then(Value::as_str).is_some_and(|s|s.len()>20_000)) { return Err("Anotações muito longas".into()); }
    for key in ["attacks", "equipment", "powers", "spells", "effects"] {
        if let Some(value) = obj.get(key) {
            let items = value.as_array().ok_or_else(|| format!("{key} deve ser uma lista"))?;
            if items.len() > 200 || items.iter().any(|item| !item.is_object()) { return Err(format!("{key} inválido ou excede o limite")); }
        }
    }
    if let Some(items) = obj.get("effects").and_then(Value::as_array) {
        for item in items {
            for (key, minimum, maximum) in [("cost",0,1_000_000),("minimumLevel",1,20),("value",-1_000_000,1_000_000)] {
                let value = item.get(key).and_then(Value::as_i64).ok_or_else(|| format!("Efeito: {key} inválido"))?;
                if !(minimum..=maximum).contains(&value) { return Err(format!("Efeito: {key} fora do limite")); }
            }
            if item["kind"] == "Atributo" && !matches!(item["attribute"].as_i64(), Some(0..=5)) { return Err("Atributo do efeito inválido".into()); }
            if !item["active"].is_boolean() { return Err("Estado do efeito inválido".into()); }
            if item["active"] == true && item["minimumLevel"].as_i64().unwrap() > level { return Err("Efeito ativo exige nível maior".into()); }
        }
    }
    {
        let empty = Vec::new();
        let items = obj.get("equipment").and_then(Value::as_array).unwrap_or(&empty);
        let mut spaces=0.0;
        for kind in ["Armadura", "Escudo"] {
            if items.iter().filter(|item| item["equipped"] == true && item["itemType"] == kind).count() > 1 { return Err(format!("Apenas um item de {kind} pode estar equipado")); }
        }
        for item in items {
            let quantity=if item.get("quantity").is_some(){item["quantity"].as_f64().ok_or("Quantidade inválida")?}else{1.0};
            let size=if item.get("spaces").is_some(){item["spaces"].as_f64().ok_or("Espaços inválidos")?}else{0.0};
            if !quantity.is_finite()||!size.is_finite()||quantity.fract()!=0.0||quantity<0.0||size<0.0||quantity>10_000.0||size>1_000.0 {return Err("Carga inválida".into());}
            if quantity == 0.0 && item["equipped"] == true { return Err("Item sem quantidade não pode estar equipado".into()); }
            spaces+=quantity*size;
        }
        let coins=obj.get("coins").unwrap_or(&Value::Null);
        if obj.contains_key("coins") && !coins.is_object() { return Err("Moedas inválidas".into()); }
        let mut coin_count=0_i64;
        for key in ["to","ts","tc"] {let count=if coins.get(key).is_some(){coins[key].as_i64().ok_or("Moedas inválidas")?}else{0};if !(0..=1_000_000).contains(&count){return Err("Moedas inválidas".into());}coin_count+=count;}
        spaces+=((coin_count+999)/1000) as f64;
        let ancestry=obj.get("ancestry").and_then(Value::as_str).unwrap_or("");
        let racial_strength=match ancestry {"Minotauro"|"Golem"=>2,"Sílfide"=>-2,"Hynne"|"Kliren"=>-1,"Trog"=>1,_=>0};
        let choice=obj.get("ancestryChoices").and_then(Value::as_array).is_some_and(|v|v.iter().any(|n|n.as_i64()==Some(0))) as i64;
        let base=obj.get("baseAttributes").and_then(Value::as_array).and_then(|v|v.first()).and_then(Value::as_i64).unwrap_or(0);
        let temporary=obj.get("temporaryAttributes").and_then(Value::as_array).and_then(|v|v.first()).and_then(Value::as_i64).unwrap_or(0);
        let effects=obj.get("effects").and_then(Value::as_array).map(|items|items.iter().filter(|e|e["active"]==true&&e["kind"]=="Atributo"&&e["attribute"]==0).map(|e|e["value"].as_i64().unwrap_or(0)).sum::<i64>()).unwrap_or(0);
        let strength=base+racial_strength+choice+temporary+effects;
        let limit=(10+if strength>=0 {2*strength}else{strength}).max(0) as f64;
        if spaces>2.0*limit {return Err("Carga acima do dobro do limite".into());}
    }
    if data.to_string().len() > 250_000 { return Err("Ficha muito grande".into()); }
    Ok(name.to_owned())
}

// ponytail: catálogo parcial; mudanças mecânicas não comprovadas exigem revisão
// do mestre. Substituir por validações automáticas apenas com regras e testes.
fn mechanical_changes(previous:&Value, proposed:&Value, can_edit_proficiencies:bool)->Vec<String> {
    fn mechanics(data:&Value, can_edit:bool)->serde_json::Map<String,Value> {
        let mut result=data.as_object().cloned().unwrap_or_default();
        for key in ["name","player","notes","diary"] {result.remove(key);}
        if can_edit {for key in ["proficiencyAdds","proficiencyRemoves","proficiencyChoice"] {result.remove(key);}}
        for key in ["attacks","equipment","powers","spells","effects"] {
            if let Some(items)=result.get_mut(key).and_then(Value::as_array_mut) {
                for item in items {if let Some(item)=item.as_object_mut(){item.remove("open");}}
            }
        }
        result
    }
    let before=mechanics(previous,can_edit_proficiencies);
    let after=mechanics(proposed,can_edit_proficiencies);
    let mut keys:Vec<_>=before.keys().chain(after.keys()).filter(|key|before.get(*key)!=after.get(*key)).cloned().collect();
    keys.sort();keys.dedup();keys
}

fn sheet_requests(db:&Connection,room_id:&str)->Result<Vec<Value>,String> {
    let mut query=db.prepare("SELECT q.id,q.sheet_id,q.base_revision,q.previous,q.proposed,p.name FROM sheet_change_requests q JOIN participants p ON p.token=q.owner_token AND p.room_id=q.room_id WHERE q.room_id=?1 AND p.approved=1 AND p.revoked=0").map_err(|e|e.to_string())?;
    let rows=query.query_map([room_id],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,i64>(2)?,row.get::<_,String>(3)?,row.get::<_,String>(4)?,row.get::<_,String>(5)?))).map_err(|e|e.to_string())?;
    rows.map(|row|{
        let (id,sheet_id,base,previous,proposed,name)=row.map_err(|e|e.to_string())?;
        let previous:Value=serde_json::from_str(&previous).map_err(|e|e.to_string())?;
        let proposed:Value=serde_json::from_str(&proposed).map_err(|e|e.to_string())?;
        Ok(json!({"id":id,"sheetId":sheet_id,"baseRevision":base,"previous":previous,"data":proposed,"participantName":name,"fields":mechanical_changes(&previous,&proposed,false)}))
    }).collect()
}

fn activate_sheet_entry(data:&Value, collection:&str, entry_id:i64, active:bool)->Result<Value,String> {
    if !["spells","effects"].contains(&collection){return Err("Tipo de habilidade inválido".into());}
    validate_sheet(data)?;
    let items=data[collection].as_array().ok_or("Habilidade não encontrada")?;
    let matches:Vec<_>=items.iter().enumerate().filter(|(_,item)|item["id"].as_i64()==Some(entry_id)).collect();
    if matches.len()!=1{return Err("Identificador de habilidade ausente ou duplicado".into());}
    let (index,item)=matches[0];
    if item["active"].as_bool().unwrap_or(false)==active{return Err("A habilidade já está neste estado".into());}
    let mut next=data.clone();
    if active {
        let level=data["level"].as_i64().ok_or("Nível inválido")?;
        let cost=if collection=="spells" {
            use tormenta20_core::SpellCircle::*;
            let circle=match item["circle"].as_i64(){Some(1)=>First,Some(2)=>Second,Some(3)=>Third,Some(4)=>Fourth,Some(5)=>Fifth,_=>return Err("Círculo inválido".into())};
            i64::from(tormenta20_core::spell_base_cost(circle))
        }else{
            let minimum=item["minimumLevel"].as_i64().ok_or("Nível mínimo inválido")?;
            if level<minimum{return Err("Nível insuficiente para esta habilidade".into());}
            item["cost"].as_i64().ok_or("Custo inválido")?
        };
        if cost<0||cost>level{return Err("Custo excede o limite básico de PM por habilidade. Exceções exigem revisão do mestre.".into());}
        let current=data["currentMp"].as_i64().unwrap_or(0);
        let temporary=data["temporaryMp"].as_i64().unwrap_or(0);
        if current+temporary<cost{return Err("PM insuficientes".into());}
        let spent_temporary=temporary.min(cost);
        next["temporaryMp"]=json!(temporary-spent_temporary);
        next["currentMp"]=json!(current-(cost-spent_temporary));
    }
    let instantaneous=collection=="spells"&&item["duration"].as_str().is_some_and(|s|s.trim().eq_ignore_ascii_case("Instantânea"));
    next[collection][index]["active"]=json!(active&&!instantaneous);
    validate_sheet(&next)?;
    Ok(next)
}

fn damage_reduction(data:&Value,kind:&str)->i32 {
    let mut reduction=data["manualRd"]["Geral"].as_i64().unwrap_or(0).max(0)
        +if kind=="Geral" {0}else{data["manualRd"][kind].as_i64().unwrap_or(0).max(0)};
    if let Some(items)=data["equipment"].as_array() {
        for item in items.iter().filter(|i|i["equipped"]==true) {
            let material=item["material"].as_str().unwrap_or("");
            let item_type=item["itemType"].as_str().unwrap_or("");
            if material=="Adamante" && item_type=="Armadura" {reduction+=if item["heavy"]==true {5}else{2};}
            if material=="Adamante" && item_type=="Escudo" {reduction+=2;}
            if material=="Gelo Eterno" && kind=="Fogo" && (item_type=="Armadura"||item_type=="Escudo") {reduction+=if item["heavy"]==true {10}else{5};}
        }
    }
    if data["spells"].as_array().is_some_and(|spells|spells.iter().any(|s|s["active"]==true&&s["name"]=="Pele de Pedra")){reduction+=5;}
    if data["activeConditionNames"].as_array().is_some_and(|names|names.iter().any(|n|n=="Petrificado")){reduction+=8;}
    if let Some(effects)=data["effects"].as_array(){reduction+=effects.iter().filter(|e|e["active"]==true&&e["kind"]=="Redução de dano").map(|e|e["value"].as_i64().unwrap_or(0)).sum::<i64>();}
    reduction.clamp(0,1_000_000) as i32
}

fn golem_immunity(data:&Value)->Option<&'static str>{
    if data["ancestry"]!="Golem" {return None;}
    match data["golemElement"].as_str()? {"Água"=>Some("Frio"),"Ar"=>Some("Eletricidade"),"Fogo"=>Some("Fogo"),"Terra"=>Some("Ácido"),_=>None}
}

#[tauri::command]
pub fn migrate_legacy(app: tauri::AppHandle, legacy: Option<String>) -> Result<Vec<LocalCharacter>, String> {
    let path = db_path(&app)?;
    let mut db = open_db(&path)?;
    migrate_legacy_db(&mut db,legacy)?;
    list_local(&db)
}

fn migrate_legacy_db(db:&mut Connection,legacy:Option<String>)->Result<(),String>{
    let tx=db.transaction().map_err(|e|e.to_string())?;
    let migrated:bool=tx.query_row("SELECT 1 FROM meta WHERE key='legacy_migrated'",[],|_|Ok(true)).optional().map_err(|e|e.to_string())?.unwrap_or(false);
    if !migrated {
        if let Some(raw)=legacy.filter(|s|!s.is_empty()) {
            let data:Value=serde_json::from_str(&raw).map_err(|_|"Rascunho antigo inválido; ele não foi apagado".to_string())?;
            let name=data.get("name").and_then(Value::as_str).filter(|s|!s.trim().is_empty()).unwrap_or("Personagem sem nome");
            tx.execute("INSERT INTO local_characters VALUES (?1,?2,?3,?4)",params![Uuid::new_v4().to_string(),name,raw,now()]).map_err(|e|e.to_string())?;
        }
        tx.execute("INSERT INTO meta VALUES ('legacy_migrated','1')",[]).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e|e.to_string())
}

fn list_local(db: &Connection) -> Result<Vec<LocalCharacter>, String> {
    let mut stmt = db.prepare("SELECT id,name,data,updated_at FROM local_characters ORDER BY updated_at DESC").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,i64>(3)?))).map_err(|e| e.to_string())?;
    rows.map(|row| {
        let (id,name,raw,updated_at)=row.map_err(|e| e.to_string())?;
        Ok(LocalCharacter{id,name,data:serde_json::from_str(&raw).map_err(|e| e.to_string())?,updated_at})
    }).collect()
}

#[tauri::command]
pub fn list_characters(app: tauri::AppHandle) -> Result<Vec<LocalCharacter>, String> {
    list_local(&open_db(&db_path(&app)?)?)
}

#[tauri::command]
pub fn save_character(app: tauri::AppHandle, id: Option<String>, data: Value) -> Result<LocalCharacter, String> {
    let name = validate_sheet(&data)?;
    let id = id.filter(|s| !s.is_empty()).unwrap_or_else(|| Uuid::new_v4().to_string());
    let db = open_db(&db_path(&app)?)?;
    db.execute("INSERT INTO local_characters VALUES (?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,data=excluded.data,updated_at=excluded.updated_at",
        params![id,name,data.to_string(),now()]).map_err(|e| e.to_string())?;
    Ok(LocalCharacter{id,name,data,updated_at:now()})
}

#[tauri::command]
pub fn delete_character(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let db=open_db(&db_path(&app)?)?;
    if db.execute("DELETE FROM local_characters WHERE id=?1",[id]).map_err(|e|e.to_string())?==0 {return Err("Ficha local não encontrada".into());}
    Ok(())
}

#[tauri::command]
pub fn list_rooms(app: tauri::AppHandle) -> Result<Vec<RoomInfo>, String> {
    let db = open_db(&db_path(&app)?)?;
    let mut stmt = db.prepare("SELECT id,name,cover_file_name,last_opened_at FROM rooms ORDER BY last_opened_at DESC").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {let cover:Option<String>=row.get(2)?;Ok(RoomInfo{id:row.get(0)?,name:row.get(1)?,cover_id:cover.and_then(|f|FilePath::new(&f).file_stem()?.to_str().map(str::to_owned)),last_opened_at:row.get(3)?})}).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn room_cover(app:tauri::AppHandle,room_id:String)->Result<Option<RoomCover>,String>{
    let db=open_db(&db_path(&app)?)?;
    let file:Option<String>=db.query_row("SELECT cover_file_name FROM rooms WHERE id=?1",[room_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?.flatten();
    let Some(file)=file else{return Ok(None)};
    let mime=if file.ends_with(".png"){"image/png"}else if file.ends_with(".jpg"){"image/jpeg"}else if file.ends_with(".webp"){"image/webp"}else{return Err("Capa inválida".into())};
    Ok(Some(RoomCover{bytes:fs::read(assets_path(&app)?.join(file)).map_err(|e|e.to_string())?,mime:mime.into()}))
}

#[tauri::command]
pub fn import_room_cover(app:tauri::AppHandle,room_id:String)->Result<RoomInfo,String>{
    let db=open_db(&db_path(&app)?)?;
    if !room_exists(&db,&room_id)?{return Err("Mesa não encontrada".into())}
    let chosen=app.dialog().file().add_filter("Imagens", &["png","jpg","jpeg","webp"]).blocking_pick_file().ok_or("Importação cancelada")?;
    let source=chosen.as_path().ok_or("Selecione um arquivo local")?;
    let (ext,_,_)=inspect_image(source,"cover")?;
    let file_name=format!("{}.{}",Uuid::new_v4(),ext);
    let dest=assets_path(&app)?.join(&file_name);
    fs::copy(source,&dest).map_err(|e|e.to_string())?;
    let old:Option<String>=db.query_row("SELECT cover_file_name FROM rooms WHERE id=?1",[&room_id],|r|r.get(0)).map_err(|e|e.to_string())?;
    if let Err(error)=db.execute("UPDATE rooms SET cover_file_name=?1 WHERE id=?2",params![file_name,room_id]){let _=fs::remove_file(dest);return Err(error.to_string())}
    if let Some(old)=old{if FilePath::new(&old).file_stem().and_then(|v|v.to_str()).is_some_and(|v|Uuid::parse_str(v).is_ok()){let _=fs::remove_file(assets_path(&app)?.join(old));}}
    list_rooms(app)?.into_iter().find(|r|r.id==room_id).ok_or("Mesa não encontrada".into())
}

fn delete_room_db(db:&mut Connection,room_id:&str)->Result<Vec<String>,String>{
    let tx=db.transaction().map_err(|e|e.to_string())?;
    if !room_exists(&tx,room_id)? {return Err("Mesa não encontrada".into());}
    let files={
        let mut stmt=tx.prepare("SELECT file_name FROM room_maps WHERE room_id=?1 UNION ALL SELECT file_name FROM room_token_art WHERE room_id=?1 UNION ALL SELECT file_name FROM room_audio WHERE room_id=?1 UNION ALL SELECT cover_file_name FROM rooms WHERE id=?1 AND cover_file_name IS NOT NULL").map_err(|e|e.to_string())?;
        stmt.query_map([room_id],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?.map(|r|r.map_err(|e|e.to_string())).collect::<Result<Vec<_>,_>>()?
    };
    for table in ["sheet_change_requests","room_documents","room_marks","room_fog","room_combat","room_threat_instances","room_threats","room_tokens","room_playlist","room_playback","room_audio","room_token_art","room_maps","room_characters","participants","chat","rooms"] {
        let column=if table=="rooms"{"id"}else{"room_id"};
        tx.execute(&format!("DELETE FROM {table} WHERE {column}=?1"),[room_id]).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e|e.to_string())?;
    Ok(files)
}

#[tauri::command]
pub async fn delete_room(app:tauri::AppHandle,state:TauriState<'_,MesaState>,room_id:String)->Result<usize,String>{
    let running=state.running.lock().await;
    if running.as_ref().is_some_and(|room|room.server.id==room_id){return Err("Encerre esta mesa antes de apagá-la".into());}
    let assets=assets_path(&app)?;
    let mut db=open_db(&db_path(&app)?)?;
    let files=delete_room_db(&mut db,&room_id)?;
    let mut remaining=0;
    for file in files.into_iter().collect::<std::collections::HashSet<_>>() {
        let path=FilePath::new(&file);
        if path.file_stem().and_then(|v|v.to_str()).is_none_or(|stem|Uuid::parse_str(stem).is_err())||path.components().count()!=1{remaining+=1;continue;}
        match db.query_row("SELECT 1 FROM room_maps WHERE file_name=?1 UNION SELECT 1 FROM room_token_art WHERE file_name=?1 UNION SELECT 1 FROM room_audio WHERE file_name=?1 UNION SELECT 1 FROM rooms WHERE cover_file_name=?1 LIMIT 1",[&file],|_|Ok(true)).optional(){Ok(Some(_))=>continue,Err(_)=>{remaining+=1;continue;},Ok(None)=>{}}
        if let Err(error)=fs::remove_file(assets.join(path)) {if error.kind()!=std::io::ErrorKind::NotFound{remaining+=1;}}
    }
    Ok(remaining)
}

fn room_exists(db:&Connection,room_id:&str)->Result<bool,String>{
    db.query_row("SELECT 1 FROM rooms WHERE id=?1",[room_id],|_|Ok(true)).optional().map(|v|v.unwrap_or(false)).map_err(|e|e.to_string())
}

fn maps(db:&Connection,room_id:&str)->Result<Vec<Value>,String>{
    let mut stmt=db.prepare("SELECT id,name,width,height,grid_visible,snap,cell_px,metres_per_cell,kind,backdrop_color FROM room_maps WHERE room_id=?1 ORDER BY rowid").map_err(|e|e.to_string())?;
    stmt.query_map([room_id],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"width":r.get::<_,i64>(2)?,"height":r.get::<_,i64>(3)?,"gridVisible":r.get::<_,i64>(4)?!=0,"snap":r.get::<_,i64>(5)?!=0,"cellPx":r.get::<_,i64>(6)?,"metresPerCell":r.get::<_,f64>(7)?,"kind":r.get::<_,String>(8)?,"backdropColor":r.get::<_,Option<String>>(9)?}))).map_err(|e|e.to_string())?.map(|r|r.map_err(|e|e.to_string())).collect()
}

fn validate_threat(data:&Value)->Result<(),String>{
    let obj=data.as_object().ok_or("Ficha de ameaça inválida")?;
    if data.to_string().len()>20_000{return Err("Ficha de ameaça grande demais".into());}
    for (key,max,required) in [("name",80,true),("nd",20,true),("kind",80,false),("size",40,false),("attackName",80,false),("attackFormula",30,false),("damageFormula",30,false),("actions",4000,false),("immunities",1000,false),("resistances",1000,false),("equipment",1000,false),("treasure",1000,false),("notes",4000,false)]{
        let value=obj.get(key).and_then(Value::as_str).unwrap_or("").trim();
        if (required&&value.is_empty())||value.chars().count()>max{return Err(format!("Campo {key} inválido"));}
    }
    if !["solo","minion","special"].contains(&obj.get("role").and_then(Value::as_str).unwrap_or("")){return Err("Papel de combate inválido".into());}
    for key in ["initiative","perception","fortitude","reflex","will"]{
        if !obj.get(key).and_then(Value::as_i64).is_some_and(|n|(-1000..=1000).contains(&n)){return Err(format!("Campo {key} inválido"));}
    }
    for (key,min,max) in [("defense",0,1000),("maxHp",1,1_000_000),("maxMp",0,1_000_000),("movement",0,1000)]{
        if !obj.get(key).and_then(Value::as_i64).is_some_and(|n|(min..=max).contains(&n)){return Err(format!("Campo {key} inválido"));}
    }
    Ok(())
}

fn threats(db:&Connection,room_id:&str)->Result<Vec<Value>,String>{
    let mut stmt=db.prepare("SELECT id,data FROM room_threats WHERE room_id=?1 ORDER BY rowid").map_err(|e|e.to_string())?;
    let rows=stmt.query_map([room_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(|e|e.to_string())?;
    rows.map(|row|{let (id,raw)=row.map_err(|e|e.to_string())?;let mut data:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;data["id"]=json!(id);Ok(data)}).collect()
}

fn documents(db:&Connection,room_id:&str,master:bool)->Result<Vec<Value>,String>{
    let mut stmt=db.prepare("SELECT id,kind,title,body,visibility FROM room_documents WHERE room_id=?1 AND (?2=1 OR visibility='public') ORDER BY rowid").map_err(|e|e.to_string())?;
    stmt.query_map(params![room_id,master as i64],|r|Ok(json!({"id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"title":r.get::<_,String>(2)?,"body":r.get::<_,String>(3)?,"visibility":r.get::<_,String>(4)?}))).map_err(|e|e.to_string())?.map(|r|r.map_err(|e|e.to_string())).collect()
}

#[tauri::command]
pub async fn save_document(app:tauri::AppHandle,state:TauriState<'_,MesaState>,room_id:String,id:Option<String>,kind:String,title:String,body:String,visibility:String)->Result<Value,String>{
    let title=title.trim();let body=body.trim();
    if !["note","npc","handout"].contains(&kind.as_str())||!["private","public"].contains(&visibility.as_str())||title.is_empty()||title.chars().count()>80||body.chars().count()>20_000{return Err("Documento inválido ou grande demais".into());}
    let (document,public)={
        let db=open_db(&db_path(&app)?)?;
        if !room_exists(&db,&room_id)?{return Err("Mesa não encontrada".into());}
        let id=if let Some(id)=id{
            if db.execute("UPDATE room_documents SET kind=?1,title=?2,body=?3,visibility=?4 WHERE id=?5 AND room_id=?6",params![kind,title,body,visibility,id,room_id]).map_err(|e|e.to_string())?==0{return Err("Documento não encontrado".into());}
            id
        }else{
            let id=Uuid::new_v4().to_string();
            db.execute("INSERT INTO room_documents(id,room_id,kind,title,body,visibility) VALUES (?1,?2,?3,?4,?5,?6)",params![id,room_id,kind,title,body,visibility]).map_err(|e|e.to_string())?;
            id
        };
        (json!({"id":id,"kind":kind,"title":title,"body":body,"visibility":visibility}),documents(&db,&room_id,false)?)
    };
    let server=state.running.lock().await.as_ref().filter(|r|r.server.id==room_id).map(|r|r.server.clone());
    if let Some(server)=server{broadcast(&server,json!({"type":"documents","items":public})).await;}
    Ok(document)
}

#[tauri::command]
pub async fn delete_document(app:tauri::AppHandle,state:TauriState<'_,MesaState>,room_id:String,id:String)->Result<(),String>{
    let public={
        let db=open_db(&db_path(&app)?)?;
        if db.execute("DELETE FROM room_documents WHERE id=?1 AND room_id=?2",params![id,room_id]).map_err(|e|e.to_string())?==0{return Err("Documento não encontrado".into());}
        documents(&db,&room_id,false)?
    };
    let server=state.running.lock().await.as_ref().filter(|r|r.server.id==room_id).map(|r|r.server.clone());
    if let Some(server)=server{broadcast(&server,json!({"type":"documents","items":public})).await;}
    Ok(())
}

fn threat_instances(db:&Connection,room_id:&str,map_id:&str)->Result<Vec<Value>,String>{
    let mut stmt=db.prepare("SELECT i.token_id,i.threat_id,i.current_hp,i.current_mp,i.conditions,i.revision,t.name FROM room_threat_instances i JOIN room_tokens t ON t.id=i.token_id AND t.room_id=i.room_id WHERE i.room_id=?1 AND t.map_id=?2 ORDER BY t.rowid").map_err(|e|e.to_string())?;
    stmt.query_map(params![room_id,map_id],|r|{let raw:String=r.get(4)?;Ok(json!({"tokenId":r.get::<_,String>(0)?,"threatId":r.get::<_,String>(1)?,"currentHp":r.get::<_,i64>(2)?,"currentMp":r.get::<_,i64>(3)?,"conditions":serde_json::from_str::<Value>(&raw).unwrap_or(json!([])),"revision":r.get::<_,i64>(5)?,"name":r.get::<_,String>(6)?}))}).map_err(|e|e.to_string())?.map(|r|r.map_err(|e|e.to_string())).collect()
}

fn tick_threat_conditions(db:&Connection,room_id:&str,map_id:&str)->Result<Vec<String>,String>{
    let mut stmt=db.prepare("SELECT i.token_id,i.conditions FROM room_threat_instances i JOIN room_tokens t ON t.id=i.token_id AND t.room_id=i.room_id WHERE i.room_id=?1 AND t.map_id=?2").map_err(|e|e.to_string())?;
    let rows=stmt.query_map(params![room_id,map_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    drop(stmt);
    let mut changed=Vec::new();
    for (id,raw) in rows {
        let mut items:Vec<Value>=serde_json::from_str(&raw).map_err(|_|"Condições inválidas")?;
        let before=items.clone();
        items.retain_mut(|item|{
            if item["unit"]!="rounds"{return true;}
            let remaining=item["remaining"].as_i64().unwrap_or(1)-1;
            item["remaining"]=json!(remaining);
            remaining>0
        });
        if items!=before{db.execute("UPDATE room_threat_instances SET conditions=?1,revision=revision+1 WHERE token_id=?2 AND room_id=?3",params![json!(items).to_string(),id,room_id]).map_err(|e|e.to_string())?;changed.push(id);}
    }
    Ok(changed)
}

fn combat_state(db:&Connection,room_id:&str,map_id:&str)->Result<Option<Value>,String>{
    let row:Option<(String,i64)>=db.query_row("SELECT state,revision FROM room_combat WHERE room_id=?1 AND map_id=?2",params![room_id,map_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
    row.map(|(raw,revision)|{let mut value:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;value["revision"]=json!(revision);Ok(value)}).transpose()
}

fn save_combat(db:&Connection,room_id:&str,map_id:&str,state:&Value)->Result<(),String>{
    let changed=db.execute("INSERT OR IGNORE INTO room_combat(room_id,map_id,state) VALUES (?1,?2,?3)",params![room_id,map_id,state.to_string()]).map_err(|e|e.to_string())?;
    if changed==0{return Err("Já existe um combate em andamento; encerre-o antes de iniciar outro".into());}
    Ok(())
}

fn update_combat(db:&Connection,room_id:&str,map_id:&str,state:&Value,revision:i64)->Result<(),String>{
    let changed=db.execute("UPDATE room_combat SET state=?1,revision=revision+1 WHERE room_id=?2 AND map_id=?3 AND revision=?4",params![state.to_string(),room_id,map_id,revision]).map_err(|e|e.to_string())?;
    if changed==0{return Err("O combate mudou; tente novamente".into());}
    Ok(())
}

fn fog_state(db:&Connection,room_id:&str,map_id:&str)->Result<Value,String>{
    let row:Option<(String,i64)>=db.query_row("SELECT state,revision FROM room_fog WHERE room_id=?1 AND map_id=?2",params![room_id,map_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
    if let Some((raw,revision))=row{
        let mut value:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;
        value["mapId"]=json!(map_id);value["revision"]=json!(revision);Ok(value)
    }else{Ok(json!({"mapId":map_id,"enabled":false,"ops":[],"revision":0}))}
}

fn valid_color(color:&str)->bool{
    color.len()==7&&color.starts_with('#')&&color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

fn mark_point(value:&Value,map:&Value)->Result<Value,String>{
    let x=value["x"].as_f64().ok_or("Ponto inválido")?;
    let y=value["y"].as_f64().ok_or("Ponto inválido")?;
    if !x.is_finite()||!y.is_finite()||x<0.0||y<0.0||x>map["width"].as_f64().unwrap_or(0.0)||y>map["height"].as_f64().unwrap_or(0.0){return Err("Ponto fora do mapa".into());}
    Ok(json!({"x":x,"y":y}))
}

fn validate_mark(kind:&str,data:&Value,map:&Value)->Result<Value,String>{
    match kind {
        "drawing"|"measure"=>{
            let points=data["points"].as_array().ok_or("Pontos ausentes")?;
            let shape=data["shape"].as_str().ok_or("Forma ausente")?;
            let max=if kind=="drawing"&&shape=="freehand"{1024}else{2};
            if points.len()<2||points.len()>max||!(if kind=="drawing"{["freehand","line","rect"].contains(&shape)}else{["line","corners"].contains(&shape)}){return Err("Forma ou pontos inválidos".into());}
            let points=points.iter().map(|point|mark_point(point,map)).collect::<Result<Vec<_>,_>>()?;
            if kind=="measure"{
                let (a,b)=(&points[0],&points[1]);
                let dx=(b["x"].as_f64().unwrap()-a["x"].as_f64().unwrap()).abs();
                let dy=(b["y"].as_f64().unwrap()-a["y"].as_f64().unwrap()).abs();
                let pixels=if shape=="line"{dx.hypot(dy)}else{dx+dy};
                let metres=(pixels/map["cellPx"].as_f64().unwrap_or(64.0)*map["metresPerCell"].as_f64().unwrap_or(1.5)*10.0).round()/10.0;
                return Ok(json!({"points":points,"shape":shape,"metres":metres}));
            }
            let stroke=data["stroke"].as_str().filter(|s|valid_color(s)).ok_or("Cor do traço inválida")?;
            let fill=data["fill"].as_str().filter(|s|*s=="transparent"||valid_color(s)).ok_or("Cor de preenchimento inválida")?;
            let width=data["width"].as_f64().filter(|n|n.is_finite()&&(1.0..=24.0).contains(n)).ok_or("Espessura inválida")?;
            let blend=data["blend"].as_str().filter(|s|["normal","multiply"].contains(s)).ok_or("Modo de desenho inválido")?;
            Ok(json!({"points":points,"shape":shape,"stroke":stroke,"fill":fill,"width":width,"blend":blend}))
        }
        "text"=>{
            let point=mark_point(data,map)?;
            let content=data["text"].as_str().map(str::trim).filter(|s|!s.is_empty()&&s.chars().count()<=1000).ok_or("Texto inválido")?;
            let color=data["color"].as_str().filter(|s|valid_color(s)).ok_or("Cor do texto inválida")?;
            let font=data["font"].as_str().filter(|s|["Arial","Georgia","Segoe UI"].contains(s)).ok_or("Fonte inválida")?;
            let size=data["size"].as_u64().filter(|n|(12..=72).contains(n)).ok_or("Tamanho de texto inválido")?;
            let bold=data["bold"].as_bool().ok_or("Estilo de texto inválido")?;
            let italic=data["italic"].as_bool().ok_or("Estilo de texto inválido")?;
            Ok(json!({"x":point["x"],"y":point["y"],"text":content,"color":color,"font":font,"size":size,"bold":bold,"italic":italic}))
        }
        "pin"=>{
            let point=mark_point(data,map)?;
            let title=data["title"].as_str().map(str::trim).filter(|s|!s.is_empty()&&s.chars().count()<=80).ok_or("Título inválido")?;
            let note=data["note"].as_str().filter(|s|s.chars().count()<=2000).ok_or("Nota longa demais")?;
            Ok(json!({"x":point["x"],"y":point["y"],"title":title,"note":note}))
        }
        _=>Err("Tipo de marcação inválido".into())
    }
}

fn marks(db:&Connection,room_id:&str,map_id:&str,master:bool)->Result<Vec<Value>,String>{
    let mut stmt=db.prepare("SELECT m.id,m.kind,COALESCE(p.id,''),m.author,m.visibility,m.data,m.revision FROM room_marks m LEFT JOIN participants p ON p.token=m.owner_token AND p.room_id=m.room_id WHERE m.room_id=?1 AND m.map_id=?2 AND (?3=1 OR m.visibility='public') ORDER BY m.rowid").map_err(|e|e.to_string())?;
    let rows=stmt.query_map(params![room_id,map_id,master as i64],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,i64>(6)?))).map_err(|e|e.to_string())?;
    rows.map(|row|{let (id,kind,owner,author,visibility,raw,revision)=row.map_err(|e|e.to_string())?;let data:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;Ok(json!({"id":id,"mapId":map_id,"kind":kind,"ownerParticipantId":owner,"author":author,"visibility":visibility,"data":data,"revision":revision}))}).collect()
}

fn mark_lists(db:&Connection,room_id:&str,map_id:&str)->Result<(Vec<Value>,Vec<Value>),String>{
    Ok((marks(db,room_id,map_id,false)?,marks(db,room_id,map_id,true)?))
}

async fn broadcast_marks(server:&RoomServer,public:Vec<Value>,all:Vec<Value>){
    broadcast(server,json!({"type":"marks","items":public})).await;
    private(server,&server.master_token,json!({"type":"marks","items":all})).await;
}

fn new_combat(db:&Connection,room_id:&str,map_id:&str,initiatives:&Value)->Result<Value,String>{
    let supplied=initiatives.as_object().ok_or("Informe as iniciativas dos personagens")?;
    let tokens=placed_tokens(db,room_id,map_id)?;
    let mut entries=Vec::new();
    let mut threats_on_map=Vec::new();
    let mut threat_bonus=None::<i64>;
    for placed in tokens {
        let id=placed["id"].as_str().ok_or("Token inválido")?;
        if let Some(sheet_id)=placed["sheetId"].as_str(){
            let score=supplied.get(id).and_then(Value::as_i64).filter(|n|(-1000..=1000).contains(n)).ok_or("Informe a iniciativa de cada personagem")?;
            let owner:Option<String>=db.query_row("SELECT p.id FROM room_characters c LEFT JOIN participants p ON p.token=c.owner_token AND p.room_id=c.room_id WHERE c.id=?1 AND c.room_id=?2",params![sheet_id,room_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?.flatten();
            entries.push(json!({"id":id,"name":placed["name"],"tokenIds":[id],"initiative":score,"ownerParticipantId":owner}));
        }else{
            let threat_id:Option<String>=db.query_row("SELECT threat_id FROM room_threat_instances WHERE token_id=?1 AND room_id=?2",params![id,room_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
            if let Some(threat_id)=threat_id {
                let raw:String=db.query_row("SELECT data FROM room_threats WHERE id=?1 AND room_id=?2",params![threat_id,room_id],|r|r.get(0)).map_err(|e|e.to_string())?;
                let threat:Value=serde_json::from_str(&raw).map_err(|_|"Ameaça inválida")?;
                let bonus=threat["initiative"].as_i64().ok_or("Iniciativa da ameaça inválida")?;
                threat_bonus=Some(threat_bonus.map_or(bonus,|previous|previous.min(bonus)));
                threats_on_map.push(json!(id));
            }
        }
        if entries.len()+threats_on_map.len()>100{return Err("Combate limitado a 100 tokens".into());}
    }
    if !threats_on_map.is_empty(){
        let score=if let Some(value)=supplied.get("threats") {value.as_i64().filter(|n|(-1000..=1000).contains(n)).ok_or("Iniciativa das ameaças inválida")?}else{roll_dice("1d20")?.1 as i64+threat_bonus.unwrap_or(0)};
        entries.push(json!({"id":"threats","name":"Ameaças","tokenIds":threats_on_map,"initiative":score,"ownerParticipantId":Value::Null}));
    }
    if entries.is_empty(){return Err("Coloque personagens ou ameaças na cena antes de iniciar combate".into());}
    entries.sort_by(|a,b|b["initiative"].as_i64().cmp(&a["initiative"].as_i64()));
    Ok(json!({"mapId":map_id,"round":1,"turn":0,"paused":false,"entries":entries}))
}

#[tauri::command]
pub fn save_threat(app:tauri::AppHandle,room_id:String,id:Option<String>,data:Value)->Result<Value,String>{
    validate_threat(&data)?;
    let db=open_db(&db_path(&app)?)?;
    if !room_exists(&db,&room_id)?{return Err("Mesa não encontrada".into());}
    let id=if let Some(id)=id{
        if db.execute("UPDATE room_threats SET data=?1 WHERE id=?2 AND room_id=?3",params![data.to_string(),id,room_id]).map_err(|e|e.to_string())?==0{return Err("Ameaça não encontrada".into());}
        id
    }else{
        let id=Uuid::new_v4().to_string();
        db.execute("INSERT INTO room_threats(id,room_id,data) VALUES (?1,?2,?3)",params![id,room_id,data.to_string()]).map_err(|e|e.to_string())?;
        id
    };
    let mut result=data;result["id"]=json!(id);Ok(result)
}

fn token_art(db:&Connection,room_id:&str)->Result<Vec<Value>,String>{
    room_art(db,room_id,None)
}

fn room_art(db:&Connection,room_id:&str,owner:Option<&str>)->Result<Vec<Value>,String>{
    let mut stmt=db.prepare("SELECT a.id,a.name,a.width,a.height,COALESCE(p.id,''),COALESCE(p.name,'Mestre') FROM room_token_art a LEFT JOIN participants p ON p.token=a.owner_token AND p.room_id=a.room_id WHERE a.room_id=?1 AND (?2 IS NULL OR a.owner_token=?2) ORDER BY a.rowid").map_err(|e|e.to_string())?;
    stmt.query_map(params![room_id,owner],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"width":r.get::<_,i64>(2)?,"height":r.get::<_,i64>(3)?,"ownerParticipantId":r.get::<_,String>(4)?,"ownerName":r.get::<_,String>(5)?}))).map_err(|e|e.to_string())?.map(|r|r.map_err(|e|e.to_string())).collect()
}

fn room_audio(db:&Connection,room_id:&str,owner:Option<&str>)->Result<Vec<Value>,String>{
    let mut stmt=db.prepare("SELECT a.id,a.name,COALESCE(p.id,''),COALESCE(p.name,'Mestre') FROM room_audio a LEFT JOIN participants p ON p.token=a.owner_token AND p.room_id=a.room_id WHERE a.room_id=?1 AND (?2 IS NULL OR a.owner_token=?2 OR a.id IN (SELECT audio_id FROM room_playlist WHERE room_id=?1)) ORDER BY a.rowid").map_err(|e|e.to_string())?;
    stmt.query_map(params![room_id,owner],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"ownerParticipantId":r.get::<_,String>(2)?,"ownerName":r.get::<_,String>(3)?}))).map_err(|e|e.to_string())?.map(|r|r.map_err(|e|e.to_string())).collect()
}

fn playlist(db:&Connection,room_id:&str)->Result<Vec<String>,String>{
    let mut stmt=db.prepare("SELECT audio_id FROM room_playlist WHERE room_id=?1 ORDER BY position").map_err(|e|e.to_string())?;
    stmt.query_map([room_id],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?.map(|r|r.map_err(|e|e.to_string())).collect()
}

fn playback(db:&Connection,room_id:&str)->Result<Value,String>{
    let state:Option<(Option<String>,f64,i64,bool)>=db.query_row("SELECT audio_id,position_seconds,started_at,playing FROM room_playback WHERE room_id=?1",[room_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get::<_,i64>(3)?!=0))).optional().map_err(|e|e.to_string())?;
    let (id,position,started,playing)=state.unwrap_or((None,0.0,0,false));
    Ok(json!({"audioId":id,"positionSeconds":position,"startedAt":started,"playing":playing}))
}

fn active_map(db:&Connection,room_id:&str)->Result<Option<Value>,String>{
    let id:Option<String>=db.query_row("SELECT active_map_id FROM rooms WHERE id=?1",[room_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?.flatten();
    Ok(maps(db,room_id)?.into_iter().find(|m|m["id"].as_str()==id.as_deref()))
}

fn placed_tokens(db:&Connection,room_id:&str,map_id:&str)->Result<Vec<Value>,String>{
    let mut stmt=db.prepare("SELECT t.id,t.art_id,t.sheet_id,t.name,t.owner_participant_id,t.x,t.y,t.size,t.revision,t.creator_participant_id,COALESCE(i.conditions,'[]') FROM room_tokens t LEFT JOIN room_threat_instances i ON i.token_id=t.id AND i.room_id=t.room_id WHERE t.room_id=?1 AND t.map_id=?2 ORDER BY t.rowid").map_err(|e|e.to_string())?;
    stmt.query_map(params![room_id,map_id],|r|{let raw:String=r.get(10)?;let conditions:Vec<String>=serde_json::from_str::<Vec<Value>>(&raw).unwrap_or_default().into_iter().filter_map(|condition|condition["name"].as_str().map(str::to_owned)).collect();Ok(json!({"id":r.get::<_,String>(0)?,"artId":r.get::<_,Option<String>>(1)?,"sheetId":r.get::<_,Option<String>>(2)?,"name":r.get::<_,String>(3)?,"ownerParticipantId":r.get::<_,Option<String>>(4)?,"x":r.get::<_,f64>(5)?,"y":r.get::<_,f64>(6)?,"size":r.get::<_,f64>(7)?,"revision":r.get::<_,i64>(8)?,"creatorParticipantId":r.get::<_,Option<String>>(9)?,"conditions":conditions}))}).map_err(|e|e.to_string())?.map(|r|r.map_err(|e|e.to_string())).collect()
}

#[tauri::command]
pub fn list_master_assets(app:tauri::AppHandle,room_id:String)->Result<Value,String>{
    let db=open_db(&db_path(&app)?)?;
    if !room_exists(&db,&room_id)? {return Err("Mesa não encontrada".into());}
    Ok(json!({"maps":maps(&db,&room_id)?,"tokenArt":token_art(&db,&room_id)?,"threats":threats(&db,&room_id)?,"documents":documents(&db,&room_id,true)?,"activeMap":active_map(&db,&room_id)?}))
}

#[tauri::command]
pub fn create_scene(app:tauri::AppHandle,room_id:String,name:String)->Result<Value,String>{
    let name=name.trim();
    if name.is_empty()||name.chars().count()>80{return Err("Nome da cena deve ter de 1 a 80 caracteres".into());}
    let db=open_db(&db_path(&app)?)?;
    if !room_exists(&db,&room_id)?{return Err("Mesa não encontrada".into());}
    let id=Uuid::new_v4().to_string();
    db.execute("INSERT INTO room_maps(id,room_id,name,file_name,width,height,kind) VALUES (?1,?2,?3,'',1600,900,'narrative')",params![id,room_id,name]).map_err(|e|e.to_string())?;
    maps(&db,&room_id)?.into_iter().find(|scene|scene["id"]==id).ok_or("Cena não encontrada".into())
}

fn inspect_image(source:&FilePath,kind:&str)->Result<(&'static str,u32,u32),String>{
    let limit=if kind=="map" {20_000_000}else{3_000_000};
    let size=fs::metadata(source).map_err(|e|e.to_string())?.len();
    if size==0||size>limit {return Err(format!("Arquivo deve ter até {} MB",limit/1_000_000));}
    let reader=ImageReader::open(source).map_err(|e|e.to_string())?.with_guessed_format().map_err(|e|e.to_string())?;
    let format=reader.format().ok_or("Imagem inválida")?;
    let ext=match format {ImageFormat::Png=>"png",ImageFormat::Jpeg=>"jpg",ImageFormat::WebP=>"webp",_=>return Err("Use PNG, JPEG ou WebP".into())};
    let (header_width,header_height)=image::image_dimensions(source).map_err(|_|"Imagem inválida")?;
    if header_width==0||header_height==0||if kind=="map" {u64::from(header_width)*u64::from(header_height)>40_000_000}else{header_width>2048||header_height>2048} {return Err("Dimensões da imagem excedem o limite".into());}
    let image=reader.decode().map_err(|_|"Não foi possível ler a imagem")?;
    let (width,height)=(image.width(),image.height());
    if (width,height)!=(header_width,header_height){return Err("Imagem mudou durante a importação".into());}
    Ok((ext,width,height))
}

fn audio_extension(bytes:&[u8])->Option<&'static str>{
    if bytes.len()>=27 && bytes.starts_with(b"OggS") && bytes[4]==0 && (bytes.windows(8).any(|w|w==b"OpusHead") || bytes.windows(7).any(|w|w==b"\x01vorbis")) {return Some("ogg");}
    if bytes.len()>=44 && bytes.starts_with(b"RIFF") && &bytes[8..12]==b"WAVE" && bytes.windows(4).any(|w|w==b"fmt ") && bytes.windows(4).any(|w|w==b"data") {return Some("wav");}
    let start=if bytes.len()>=10 && bytes.starts_with(b"ID3") {
        if bytes[6..10].iter().any(|b|b&0x80!=0){return None;}
        10+((bytes[6] as usize)<<21)+((bytes[7] as usize)<<14)+((bytes[8] as usize)<<7)+bytes[9] as usize
    }else{0};
    if start>=bytes.len(){return None;}
    let end=(start+4096).min(bytes.len().saturating_sub(3));
    if (start..end).any(|i|bytes[i]==0xff && bytes[i+1]&0xe0==0xe0 && (bytes[i+1]>>3)&3!=1 && (bytes[i+1]>>1)&3!=0 && bytes[i+2]>>4!=0 && bytes[i+2]>>4!=15 && (bytes[i+2]>>2)&3!=3){Some("mp3")}else{None}
}

fn decode_asset_name(value:&str)->Option<String>{
    let mut bytes=Vec::with_capacity(value.len());let raw=value.as_bytes();let mut i=0;
    while i<raw.len(){if raw[i]==b'%' {let hex=raw.get(i+1..i+3)?;let hex=std::str::from_utf8(hex).ok()?;bytes.push(u8::from_str_radix(hex,16).ok()?);i+=3;}else{bytes.push(raw[i]);i+=1;}}
    let name=String::from_utf8(bytes).ok()?;let name=name.trim();
    if name.is_empty()||name.chars().count()>80 {None}else{Some(name.to_string())}
}

#[tauri::command]
pub fn import_asset(app:tauri::AppHandle,room_id:String,kind:String,name:String)->Result<Value,String>{
    if kind!="map"&&kind!="token" {return Err("Tipo de arquivo inválido".into());}
    let db=open_db(&db_path(&app)?)?;
    if !room_exists(&db,&room_id)? {return Err("Mesa não encontrada".into());}
    let chosen=app.dialog().file().add_filter("Imagens", &["png","jpg","jpeg","webp"]).blocking_pick_file().ok_or("Importação cancelada")?;
    let source=chosen.as_path().ok_or("Selecione um arquivo local")?;
    let (ext,width,height)=inspect_image(source,&kind)?;
    let title=if name.trim().is_empty(){source.file_stem().and_then(|s|s.to_str()).unwrap_or("Imagem")}else{name.trim()};
    if title.chars().count()>80 {return Err("Nome muito longo".into());}
    let id=Uuid::new_v4().to_string();
    let file_name=format!("{id}.{ext}");
    let dest=assets_path(&app)?.join(&file_name);
    fs::copy(source,&dest).map_err(|e|e.to_string())?;
    let result=(||->Result<Value,String>{if kind=="map" {
        db.execute("INSERT INTO room_maps(id,room_id,name,file_name,width,height) VALUES (?1,?2,?3,?4,?5,?6)",params![id,room_id,title,file_name,width,height]).map_err(|e|e.to_string())?;
        maps(&db,&room_id)?.into_iter().find(|m|m["id"]==id).ok_or("Mapa não encontrado".to_string())
    }else{
        db.execute("INSERT INTO room_token_art(id,room_id,name,file_name,width,height) VALUES (?1,?2,?3,?4,?5,?6)",params![id,room_id,title,file_name,width,height]).map_err(|e|e.to_string())?;
        token_art(&db,&room_id)?.into_iter().find(|a|a["id"]==id).ok_or("Token não encontrado".to_string())
    }})();
    if result.is_err(){let _=fs::remove_file(dest);}
    result
}

#[tauri::command]
pub async fn update_map(app:tauri::AppHandle,state:TauriState<'_,MesaState>,room_id:String,map_id:String,name:String,grid_visible:bool,snap:bool,cell_px:i64,metres_per_cell:f64,backdrop_color:Option<String>)->Result<Value,String>{
    let name=name.trim();
    if name.is_empty()||name.chars().count()>80||!(16..=512).contains(&cell_px)||!metres_per_cell.is_finite()||!(0.1..=100.0).contains(&metres_per_cell){return Err("Configuração de mapa inválida".into());}
    if backdrop_color.as_deref().is_some_and(|color|!valid_color(color)){return Err("Cor do fundo inválida".into());}
    let (result,active,tokens,combat,fog)={
        let db=open_db(&db_path(&app)?)?;
        let kind:Option<String>=db.query_row("SELECT kind FROM room_maps WHERE id=?1 AND room_id=?2",params![map_id,room_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
        if kind.as_deref()==Some("narrative")&&(grid_visible||snap){return Err("Cena narrativa não usa grade".into());}
        let changed=db.execute("UPDATE room_maps SET name=?1,grid_visible=?2,snap=?3,cell_px=?4,metres_per_cell=?5,backdrop_color=?6 WHERE id=?7 AND room_id=?8",params![name,grid_visible as i64,snap as i64,cell_px,metres_per_cell,backdrop_color,map_id,room_id]).map_err(|e|e.to_string())?;
        if changed==0{return Err("Mapa não encontrado".into());}
        let result=maps(&db,&room_id)?.into_iter().find(|m|m["id"]==map_id).ok_or("Mapa não encontrado")?;
        let active=active_map(&db,&room_id)?.is_some_and(|m|m["id"]==map_id);
        let tokens=if active {placed_tokens(&db,&room_id,&map_id)?}else{Vec::new()};
        let combat=if active {combat_state(&db,&room_id,&map_id)?}else{None};
        let fog=if active {Some(fog_state(&db,&room_id,&map_id)?)}else{None};
        (result,active,tokens,combat,fog)
    };
    if active {
        let server=state.running.lock().await.as_ref().filter(|r|r.server.id==room_id).map(|r|r.server.clone());
        if let Some(server)=server {broadcast(&server,json!({"type":"map_changed","activeMap":result,"placedTokens":tokens,"combat":combat,"fog":fog})).await;let (public,all)={let db=db(&server)?;mark_lists(&db,&room_id,&map_id)?};broadcast_marks(&server,public,all).await;}
    }
    Ok(result)
}

#[tauri::command]
pub async fn activate_map(app:tauri::AppHandle,state:TauriState<'_,MesaState>,room_id:String,map_id:Option<String>)->Result<Value,String>{
    let (active,tokens,instances,combat,fog,notice)={
        let db=open_db(&db_path(&app)?)?;
        if !room_exists(&db,&room_id)? {return Err("Mesa não encontrada".into());}
        if let Some(ref id)=map_id {
            let exists:bool=db.query_row("SELECT 1 FROM room_maps WHERE id=?1 AND room_id=?2",params![id,room_id],|_|Ok(true)).optional().map_err(|e|e.to_string())?.unwrap_or(false);
            if !exists{return Err("Mapa não encontrado".into());}
        }
        db.execute("UPDATE rooms SET active_map_id=?1 WHERE id=?2",params![map_id,room_id]).map_err(|e|e.to_string())?;
        let active=active_map(&db,&room_id)?;
        let tokens=if let Some(ref map)=active {placed_tokens(&db,&room_id,map["id"].as_str().unwrap_or(""))?}else{Vec::new()};
        let instances=if let Some(ref map)=active {threat_instances(&db,&room_id,map["id"].as_str().unwrap_or(""))?}else{Vec::new()};
        let combat=if let Some(ref map)=active {combat_state(&db,&room_id,map["id"].as_str().unwrap_or(""))?}else{None};
        let fog=if let Some(ref map)=active {Some(fog_state(&db,&room_id,map["id"].as_str().unwrap_or(""))?)}else{None};
        let text=active.as_ref().and_then(|scene|scene["name"].as_str()).map(|name|format!("Cena ativa: {name}")).unwrap_or_else(||"Mesa sem cena ativa.".to_string());
        let at=now();
        db.execute("INSERT INTO chat(room_id,author,text,created_at,kind) VALUES (?1,'Sistema',?2,?3,'system')",params![room_id,text,at]).map_err(|e|e.to_string())?;
        let notice=json!({"type":"chat","id":db.last_insert_rowid(),"kind":"system","author":"Sistema","text":text,"at":at,"audience":"all"});
        (active,tokens,instances,combat,fog,notice)
    };
    let event=json!({"type":"map_changed","activeMap":active,"placedTokens":tokens,"combat":combat,"fog":fog});
    let server=state.running.lock().await.as_ref().filter(|r|r.server.id==room_id).map(|r|r.server.clone());
    if let Some(server)=server {broadcast(&server,event.clone()).await;private(&server,&server.master_token,json!({"type":"threat_instances","items":instances})).await;if let Some(ref map)=active{let (public,all)={let db=db(&server)?;mark_lists(&db,&room_id,map["id"].as_str().unwrap_or(""))?};broadcast_marks(&server,public,all).await;}broadcast(&server,notice).await;}
    Ok(event)
}

fn suggested_ip() -> String {
    UdpSocket::bind("0.0.0.0:0").ok().and_then(|s| {
        s.connect("192.0.2.1:80").ok()?;
        Some(s.local_addr().ok()?.ip().to_string())
    }).filter(|s| s != "0.0.0.0").unwrap_or_else(|| "127.0.0.1".into())
}

#[tauri::command]
pub async fn start_room(app: tauri::AppHandle, state: TauriState<'_, MesaState>, name: String, room_id: Option<String>) -> Result<HostInfo, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 { return Err("Nome da mesa deve ter de 1 a 80 caracteres".into()); }
    let mut running = state.running.lock().await;
    if running.is_some() { return Err("Feche a mesa atual antes de abrir outra".into()); }
    let path = db_path(&app)?;
    let db = open_db(&path)?;
    let id = if let Some(id) = room_id {
        let old: Option<String> = db.query_row("SELECT name FROM rooms WHERE id=?1", [&id], |r| r.get(0)).optional().map_err(|e| e.to_string())?;
        if old.is_none() { return Err("Mesa não encontrada".into()); }
        db.execute("UPDATE rooms SET name=?1,last_opened_at=?2 WHERE id=?3",params![name,now(),id]).map_err(|e|e.to_string())?;
        id
    } else {
        let id = Uuid::new_v4().to_string();
        db.execute("INSERT INTO rooms(id,name,created_at,last_opened_at) VALUES (?1,?2,?3,?3)", params![id,name,now()]).map_err(|e| e.to_string())?;
        id
    };
    let code = Uuid::new_v4().simple().to_string().to_uppercase();
    let token = Uuid::new_v4().to_string();
    let (stop, _)=watch::channel(false);
    let server = Arc::new(RoomServer{id:id.clone(),name:name.into(),code:code.clone(),master_token:token.clone(),db:path,assets:assets_path(&app)?,live:Mutex::new(LiveRoom::default()),stop});
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let router = Router::new().route("/ws", get(websocket)).route("/assets", post(upload_asset).options(asset_options)).route("/assets/{id}", get(asset).options(asset_options)).with_state(server.clone());
    let (tx,rx)=oneshot::channel();
    tauri::async_runtime::spawn(async move { let _=axum::serve(listener,router).with_graceful_shutdown(async { let _=rx.await; }).await; });
    *running=Some(RunningRoom{shutdown:tx,server});
    Ok(HostInfo{id,name:name.into(),address:suggested_ip(),port,code,token})
}

#[tauri::command]
pub async fn stop_room(state: TauriState<'_, MesaState>) -> Result<(), String> {
    if let Some(room)=state.running.lock().await.take() { room.server.stop.send_replace(true); let _=room.shutdown.send(()); }
    Ok(())
}

async fn websocket(State(server):State<Arc<RoomServer>>, ws:WebSocketUpgrade) -> axum::response::Response {
    ws.max_message_size(300_000).on_upgrade(move |socket| connection(server,socket))
}

fn asset_response(status:StatusCode,body:Body,content_type:Option<&str>)->Response{
    let mut builder=Response::builder().status(status)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN,"*")
        .header(header::CACHE_CONTROL,"no-store");
    if let Some(content_type)=content_type {builder=builder.header(header::CONTENT_TYPE,content_type);}
    builder.body(body).unwrap()
}

async fn asset_options()->Response{
    Response::builder().status(StatusCode::NO_CONTENT)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN,"*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS,"GET, POST, OPTIONS")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS,"Authorization, Content-Type, X-Asset-Kind, X-Asset-Name")
        .body(Body::empty()).unwrap()
}

async fn asset(State(server):State<Arc<RoomServer>>,Path(id):Path<String>,headers:HeaderMap)->Response{
    let unauthorized=||asset_response(StatusCode::FORBIDDEN,Body::empty(),None);
    let token=headers.get(header::AUTHORIZATION).and_then(|h|h.to_str().ok()).and_then(|h|h.strip_prefix("Bearer ")).unwrap_or("");
    let db=match db(&server){Ok(db)=>db,Err(_)=>return asset_response(StatusCode::INTERNAL_SERVER_ERROR,Body::empty(),None)};
    let master=token==server.master_token;
    if !master && !participant(&db,&server.id,token).ok().flatten().is_some_and(|(_,_,approved,revoked)|approved&&!revoked){return unauthorized();}
    let map_file:Option<String>=db.query_row("SELECT file_name FROM room_maps WHERE id=?1 AND room_id=?2",params![id,server.id],|r|r.get(0)).optional().ok().flatten();
    let art_file:Option<(String,String)>=db.query_row("SELECT file_name,owner_token FROM room_token_art WHERE id=?1 AND room_id=?2",params![id,server.id],|r|Ok((r.get(0)?,r.get(1)?))).optional().ok().flatten();
    let audio_file:Option<(String,String)>=db.query_row("SELECT file_name,owner_token FROM room_audio WHERE id=?1 AND room_id=?2",params![id,server.id],|r|Ok((r.get(0)?,r.get(1)?))).optional().ok().flatten();
    let cover_file:Option<String>=db.query_row("SELECT cover_file_name FROM rooms WHERE id=?1",[&server.id],|r|r.get(0)).optional().ok().flatten().flatten().filter(|f|FilePath::new(f).file_stem().and_then(|s|s.to_str())==Some(id.as_str()));
    let allowed=if master||cover_file.is_some(){true}else if map_file.is_some(){
        db.query_row("SELECT active_map_id=?1 FROM rooms WHERE id=?2",params![id,server.id],|r|r.get::<_,bool>(0)).unwrap_or(false)
    }else if let Some((_,owner))=&audio_file {
        owner==token || db.query_row("SELECT 1 FROM room_playlist WHERE room_id=?1 AND audio_id=?2 LIMIT 1",params![server.id,id],|_|Ok(true)).optional().ok().flatten().unwrap_or(false)
    }else{
        art_file.as_ref().is_some_and(|(_,owner)|owner==token) || db.query_row("SELECT 1 FROM room_tokens t JOIN rooms r ON r.id=t.room_id WHERE t.art_id=?1 AND t.room_id=?2 AND t.map_id=r.active_map_id LIMIT 1",params![id,server.id],|_|Ok(true)).optional().ok().flatten().unwrap_or(false)
    };
    if !allowed{return unauthorized();}
    let Some(file)=cover_file.or(map_file).or_else(||art_file.map(|a|a.0)).or_else(||audio_file.map(|a|a.0)) else {return asset_response(StatusCode::NOT_FOUND,Body::empty(),None)};
    let content_type=if file.ends_with(".png"){"image/png"}else if file.ends_with(".jpg"){"image/jpeg"}else if file.ends_with(".webp"){"image/webp"}else if file.ends_with(".mp3"){"audio/mpeg"}else if file.ends_with(".ogg"){"audio/ogg"}else if file.ends_with(".wav"){"audio/wav"}else{return asset_response(StatusCode::NOT_FOUND,Body::empty(),None)};
    match fs::read(server.assets.join(file)){
        Ok(bytes)=>asset_response(StatusCode::OK,Body::from(bytes),Some(content_type)),
        Err(_)=>asset_response(StatusCode::NOT_FOUND,Body::empty(),None)
    }
}

async fn upload_asset(State(server):State<Arc<RoomServer>>,headers:HeaderMap,body:Body)->Response{
    let result=async {
        let token=headers.get(header::AUTHORIZATION).and_then(|h|h.to_str().ok()).and_then(|h|h.strip_prefix("Bearer ")).ok_or((StatusCode::FORBIDDEN,"Sem autorização".to_string()))?;
        let db=db(&server).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e))?;
        let master=token==server.master_token;
        if !master && !participant(&db,&server.id,token).ok().flatten().is_some_and(|(_,_,approved,revoked)|approved&&!revoked){return Err((StatusCode::FORBIDDEN,"Sem permissão".into()));}
        let kind=headers.get("x-asset-kind").and_then(|v|v.to_str().ok()).unwrap_or("");
        if kind!="art"&&kind!="audio" {return Err((StatusCode::BAD_REQUEST,"Tipo inválido".into()));}
        let name=decode_asset_name(headers.get("x-asset-name").and_then(|v|v.to_str().ok()).unwrap_or("")).ok_or((StatusCode::BAD_REQUEST,"Nome inválido".to_string()))?;
        let limit=if kind=="audio"{50_000_000}else{3_000_000};
        let bytes=to_bytes(body,limit).await.map_err(|_|(StatusCode::PAYLOAD_TOO_LARGE,"Arquivo grande demais".to_string()))?;
        if bytes.is_empty(){return Err((StatusCode::BAD_REQUEST,"Arquivo vazio".into()));}
        let (ext,width,height)=if kind=="art" {
            let reader=ImageReader::new(Cursor::new(&bytes)).with_guessed_format().map_err(|_|(StatusCode::BAD_REQUEST,"Imagem inválida".to_string()))?;
            let ext=match reader.format(){Some(ImageFormat::Png)=>"png",Some(ImageFormat::Jpeg)=>"jpg",Some(ImageFormat::WebP)=>"webp",_=>return Err((StatusCode::BAD_REQUEST,"Use PNG, JPEG ou WebP".into()))};
            let (w,h)=reader.into_dimensions().map_err(|_|(StatusCode::BAD_REQUEST,"Imagem inválida".to_string()))?;
            if w==0||h==0||w>2048||h>2048{return Err((StatusCode::BAD_REQUEST,"Dimensões inválidas".into()));}
            image::load_from_memory(&bytes).map_err(|_|(StatusCode::BAD_REQUEST,"Imagem corrompida".to_string()))?;
            (ext,w,h)
        }else{
            let ext=audio_extension(&bytes).ok_or((StatusCode::BAD_REQUEST,"Use MP3, Ogg ou WAV válidos".to_string()))?;
            (ext,0,0)
        };
        let id=Uuid::new_v4().to_string();let file=format!("{id}.{ext}");let path=server.assets.join(&file);
        fs::write(&path,&bytes).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))?;
        let inserted=if kind=="art"{db.execute("INSERT INTO room_token_art(id,room_id,name,file_name,width,height,owner_token) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![id,server.id,name,file,width,height,if master{""}else{token}])}else{db.execute("INSERT INTO room_audio(id,room_id,owner_token,name,file_name) VALUES (?1,?2,?3,?4,?5)",params![id,server.id,if master{""}else{token},name,file])};
        if let Err(error)=inserted{let _=fs::remove_file(path);return Err((StatusCode::INTERNAL_SERVER_ERROR,error.to_string()));}
        let item=if kind=="art"{room_art(&db,&server.id,None).ok().and_then(|v|v.into_iter().find(|a|a["id"]==id))}else{room_audio(&db,&server.id,None).ok().and_then(|v|v.into_iter().find(|a|a["id"]==id))}.ok_or((StatusCode::INTERNAL_SERVER_ERROR,"Arquivo não encontrado".to_string()))?;
        private(&server,token,json!({"type":"asset_added","kind":kind,"item":item})).await;
        Ok::<Value,(StatusCode,String)>(item)
    }.await;
    match result {Ok(item)=>asset_response(StatusCode::CREATED,Body::from(item.to_string()),Some("application/json")),Err((status,message))=>asset_response(status,Body::from(message),Some("text/plain; charset=utf-8"))}
}

fn text(value: Value) -> Message { Message::Text(value.to_string().into()) }
fn send(tx:&mpsc::UnboundedSender<Message>, value:Value) { let _=tx.send(text(value)); }
fn db(server:&RoomServer) -> Result<Connection,String> { open_db(&server.db) }

fn participant(db:&Connection,room:&str,token:&str) -> Result<Option<(String,String,bool,bool)>,String> {
    db.query_row("SELECT id,name,approved,revoked FROM participants WHERE room_id=?1 AND token=?2",params![room,token],|r|Ok((r.get(0)?,r.get(1)?,r.get::<_,i64>(2)?!=0,r.get::<_,i64>(3)?!=0)))
        .optional().map_err(|e|e.to_string())
}

fn roster(db:&Connection,room:&str,master:bool) -> Result<Vec<Value>,String> {
    let mut stmt=db.prepare("SELECT id,name,approved FROM participants WHERE room_id=?1 AND revoked=0 ORDER BY rowid").map_err(|e|e.to_string())?;
    let rows=stmt.query_map([room],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?!=0))).map_err(|e|e.to_string())?;
    rows.filter_map(|r| match r {Ok((id,name,approved)) if approved||master=>Some(Ok(json!({"id":id,"name":name,"approved":approved}))),Ok(_)=>None,Err(e)=>Some(Err(e.to_string()))}).collect()
}

fn sheets(db:&Connection,room:&str,owner:Option<&str>) -> Result<Vec<Value>,String> {
    let mut stmt=db.prepare("SELECT id,owner_token,data,revision,can_edit_proficiencies FROM room_characters WHERE room_id=?1 ORDER BY rowid").map_err(|e|e.to_string())?;
    let rows=stmt.query_map([room],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?!=0))).map_err(|e|e.to_string())?;
    rows.filter_map(|r|match r {Ok((id,owner_token,raw,revision,can_edit_proficiencies)) if owner.is_none_or(|o|o==owner_token)=>Some(serde_json::from_str::<Value>(&raw).map(|data|json!({"id":id,"owner":owner_token,"revision":revision,"data":data,"canEditProficiencies":can_edit_proficiencies})).map_err(|e|e.to_string())),Ok(_)=>None,Err(e)=>Some(Err(e.to_string()))}).collect()
}

fn history(db:&Connection,room:&str,token:&str,master:bool) -> Result<Vec<Value>,String> {
    let mut stmt=db.prepare("SELECT id,author,text,created_at,audience,kind,details FROM chat WHERE room_id=?1 AND (audience='all' OR (owner_token=?2 AND audience IN ('gm','self')) OR (?3=1 AND audience IN ('gm','secret'))) ORDER BY id DESC LIMIT 100").map_err(|e|e.to_string())?;
    let rows=stmt.query_map(params![room,token,master as i64],|r|{
        let raw:String=r.get(6)?;
        Ok(json!({"id":r.get::<_,i64>(0)?,"author":r.get::<_,String>(1)?,"text":r.get::<_,String>(2)?,"at":r.get::<_,i64>(3)?,"audience":r.get::<_,String>(4)?,"kind":r.get::<_,String>(5)?,"details":serde_json::from_str::<Value>(&raw).unwrap_or(json!({}))}))
    }).map_err(|e|e.to_string())?;
    let mut items:Vec<Value>=rows.map(|r|r.map_err(|e|e.to_string())).collect::<Result<_,_>>()?;
    items.reverse();Ok(items)
}

async fn private(server:&RoomServer,owner:&str,value:Value) {
    let live=server.live.lock().await;
    for token in [server.master_token.as_str(),owner].into_iter().collect::<std::collections::HashSet<_>>() { if let Some(tx)=live.clients.get(token) { send(tx,value.clone()); } }
}

async fn broadcast(server:&RoomServer,value:Value) {
    let recipients=if let Ok(db)=db(server) {
        let mut stmt=match db.prepare("SELECT token FROM participants WHERE room_id=?1 AND approved=1 AND revoked=0") {Ok(s)=>s,Err(_)=>return};
        stmt.query_map([&server.id],|r|r.get::<_,String>(0)).ok().map(|r|r.filter_map(Result::ok).collect::<Vec<_>>()).unwrap_or_default()
    } else { Vec::new() };
    let live=server.live.lock().await;
    for token in recipients.iter().chain(std::iter::once(&server.master_token)) { if let Some(tx)=live.clients.get(token) { send(tx,value.clone()); } }
}

async fn update_roster(server:&RoomServer) {
    if let Ok(db)=db(server) {
        if let Ok(list)=roster(&db,&server.id,false) { broadcast(server,json!({"type":"roster","participants":list})).await; }
        if let Ok(list)=roster(&db,&server.id,true) { private(server,&server.master_token,json!({"type":"roster","participants":list})).await; }
    }
}

async fn connection(server:Arc<RoomServer>, mut socket:WebSocket) {
    let mut stop=server.stop.subscribe();
    if *stop.borrow() { return; }
    let first=match tokio::time::timeout(std::time::Duration::from_secs(10),socket.recv()).await {Ok(Some(Ok(Message::Text(raw))))=>raw, _=>return};
    let hello:Value=match serde_json::from_str(&first) {Ok(v)=>v,Err(_)=>return};
    if hello["type"]!="hello" { return; }
    let code=hello["code"].as_str().unwrap_or("");
    let old_token=hello["token"].as_str().unwrap_or("");
    let (token,role,name)=if old_token==server.master_token {
        (server.master_token.clone(),"master".to_string(),"Mestre".to_string())
    } else {
        let db=match db(&server) {Ok(v)=>v,Err(_)=>return};
        match participant(&db,&server.id,old_token) {
            Ok(Some((_,name,approved,false)))=>(old_token.to_string(),if approved{"player"}else{"pending"}.to_string(),name),
            Ok(Some(_))=>return,
            _=>{
                if code!=server.code { let _=socket.send(text(json!({"type":"error","message":"Código inválido"}))).await; return; }
                let name=hello["name"].as_str().unwrap_or("").trim();
                if name.is_empty()||name.chars().count()>60 {let _=socket.send(text(json!({"type":"error","message":"Informe um nome de até 60 caracteres"}))).await;return;}
                let token=Uuid::new_v4().to_string();
                if db.execute("INSERT INTO participants VALUES (?1,?2,?3,?4,0,0)",params![Uuid::new_v4().to_string(),server.id,name,token]).is_err(){return;}
                (token,"pending".to_string(),name.to_string())
            }
        }
    };
    let (mut writer,mut reader)=socket.split();
    let (tx,mut rx)=mpsc::unbounded_channel();
    server.live.lock().await.clients.insert(token.clone(),tx.clone());
    let send_task=tokio::spawn(async move {while let Some(message)=rx.recv().await {if writer.send(message).await.is_err(){break;}}});
    if let Ok(db)=db(&server) {
        let is_master=role=="master";
        let cover_id:Option<String>=db.query_row("SELECT cover_file_name FROM rooms WHERE id=?1",[&server.id],|r|r.get::<_,Option<String>>(0)).ok().flatten().and_then(|f|FilePath::new(&f).file_stem()?.to_str().map(str::to_owned));
        send(&tx,json!({"type":"welcome","token":token,"role":role,"room":{"id":server.id,"name":server.name,"coverId":cover_id,"lastOpenedAt":now()},
            "serverNowMs":now_ms(),
            "sheetRequests":if is_master{sheet_requests(&db,&server.id).unwrap_or_default()}else{Vec::new()},
            "participantId":if is_master{None}else{participant(&db,&server.id,&token).ok().flatten().map(|p|p.0)},
            "participants":roster(&db,&server.id,is_master).unwrap_or_default(),
            "characters":if role=="pending"{Vec::new()}else{sheets(&db,&server.id,if is_master{None}else{Some(&token)}).unwrap_or_default()},
            "chat":if role=="pending"{Vec::new()}else{history(&db,&server.id,&token,is_master).unwrap_or_default()},
            "maps":if is_master{maps(&db,&server.id).unwrap_or_default()}else{Vec::new()},
            "tokenArt":if is_master{token_art(&db,&server.id).unwrap_or_default()}else{Vec::new()},
            "threats":if is_master{threats(&db,&server.id).unwrap_or_default()}else{Vec::new()},
            "documents":if role=="pending"{Vec::new()}else{documents(&db,&server.id,is_master).unwrap_or_default()},
            "threatInstances":if is_master{active_map(&db,&server.id).ok().flatten().and_then(|m|threat_instances(&db,&server.id,m["id"].as_str().unwrap_or("")).ok()).unwrap_or_default()}else{Vec::new()},
            "combat":if role=="pending"{None}else{active_map(&db,&server.id).ok().flatten().and_then(|m|combat_state(&db,&server.id,m["id"].as_str().unwrap_or("")).ok().flatten())},
            "fog":if role=="pending"{None}else{active_map(&db,&server.id).ok().flatten().and_then(|m|fog_state(&db,&server.id,m["id"].as_str().unwrap_or("")).ok())},
            "marks":if role=="pending"{Vec::new()}else{active_map(&db,&server.id).ok().flatten().and_then(|m|marks(&db,&server.id,m["id"].as_str().unwrap_or(""),is_master).ok()).unwrap_or_default()},
            "roomArt":if role=="pending"{Vec::new()}else{room_art(&db,&server.id,if is_master{None}else{Some(&token)}).unwrap_or_default()},
            "roomAudio":if role=="pending"{Vec::new()}else{room_audio(&db,&server.id,if is_master{None}else{Some(&token)}).unwrap_or_default()},
            "playlist":if role=="pending"{Vec::new()}else{playlist(&db,&server.id).unwrap_or_default()},
            "playback":if role=="pending"{Value::Null}else{playback(&db,&server.id).unwrap_or(Value::Null)},
            "activeMap":if role=="pending"{None}else{active_map(&db,&server.id).unwrap_or_default()},
            "placedTokens":if role=="pending"{Vec::new()}else{active_map(&db,&server.id).ok().flatten().and_then(|m|placed_tokens(&db,&server.id,m["id"].as_str().unwrap_or("")).ok()).unwrap_or_default()}}));
    }
    update_roster(&server).await;
    loop {
        let message=tokio::select! { _=stop.changed()=>break, next=reader.next()=>next };
        let Some(Ok(message))=message else {break};
        let Message::Text(raw)=message else {continue};
        let Ok(input)=serde_json::from_str::<Value>(&raw) else {send(&tx,json!({"type":"error","message":"Mensagem inválida"}));continue};
        if role!="master" {
            let allowed=db(&server).ok().and_then(|d|participant(&d,&server.id,&token).ok().flatten()).is_some_and(|(_,_,approved,revoked)|approved&&!revoked);
            if !allowed {send(&tx,json!({"type":"error","message":"Aguarde aprovação do mestre"}));continue;}
        }
        if let Err(err)=handle(&server,&tx,&token,&name,role=="master",&input).await {send(&tx,json!({"type":"error","message":err}));}
    }
    let mut live=server.live.lock().await;
    if live.clients.get(&token).is_some_and(|current|current.same_channel(&tx)) {live.clients.remove(&token);}
    drop(live);
    send_task.abort();
}

fn string_field<'a>(input:&'a Value,key:&str,max:usize)->Result<&'a str,String>{
    let value=input[key].as_str().ok_or_else(||format!("{key} ausente"))?.trim();
    if value.is_empty()||value.chars().count()>max {return Err(format!("{key} inválido"));}Ok(value)
}

fn map_point(input:&Value,map:&Value)->Result<(f64,f64),String>{
    let (x,y)=(input["x"].as_f64().ok_or("Posição inválida")?,input["y"].as_f64().ok_or("Posição inválida")?);
    let (width,height)=(map["width"].as_f64().unwrap_or(0.0),map["height"].as_f64().unwrap_or(0.0));
    if !x.is_finite()||!y.is_finite()||x<0.0||y<0.0||x>width||y>height{return Err("Posição fora do mapa".into());}
    let cell=map["cellPx"].as_f64().unwrap_or(64.0);
    Ok(if map["snap"]==true {((x/cell).round().mul_add(cell,0.0).min(width),(y/cell).round().mul_add(cell,0.0).min(height))}else{(x,y)})
}

fn token_record(db:&Connection,room:&str,id:&str)->Result<Option<Value>,String>{
    db.query_row("SELECT t.id,t.art_id,t.sheet_id,t.name,t.owner_participant_id,t.x,t.y,t.size,t.revision,t.map_id,t.creator_participant_id,COALESCE(i.conditions,'[]') FROM room_tokens t LEFT JOIN room_threat_instances i ON i.token_id=t.id AND i.room_id=t.room_id WHERE t.id=?1 AND t.room_id=?2",params![id,room],|r|{let raw:String=r.get(11)?;let conditions:Vec<String>=serde_json::from_str::<Vec<Value>>(&raw).unwrap_or_default().into_iter().filter_map(|condition|condition["name"].as_str().map(str::to_owned)).collect();Ok(json!({"id":r.get::<_,String>(0)?,"artId":r.get::<_,Option<String>>(1)?,"sheetId":r.get::<_,Option<String>>(2)?,"name":r.get::<_,String>(3)?,"ownerParticipantId":r.get::<_,Option<String>>(4)?,"x":r.get::<_,f64>(5)?,"y":r.get::<_,f64>(6)?,"size":r.get::<_,f64>(7)?,"revision":r.get::<_,i64>(8)?,"mapId":r.get::<_,String>(9)?,"creatorParticipantId":r.get::<_,Option<String>>(10)?,"conditions":conditions}))}).optional().map_err(|e|e.to_string())
}

async fn handle(server:&RoomServer,tx:&mpsc::UnboundedSender<Message>,token:&str,name:&str,master:bool,input:&Value)->Result<(),String>{
    let mut db=db(server)?;
    match input["type"].as_str().unwrap_or("") {
        "approve"|"revoke" if master=>{
            let id=string_field(input,"participantId",80)?;
            let approve=input["type"]=="approve";
            let count=db.execute("UPDATE participants SET approved=?1,revoked=?2 WHERE id=?3 AND room_id=?4",params![approve as i64,(!approve) as i64,id,server.id]).map_err(|e|e.to_string())?;
            if count==0{return Err("Jogador não encontrado".into());}
            let target:Option<String>=db.query_row("SELECT token FROM participants WHERE id=?1",[id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
            if let Some(target)=target {
                let peer=if approve {server.live.lock().await.clients.get(&target).cloned()} else {server.live.lock().await.clients.remove(&target)};
                if let Some(peer)=peer {
                    let active=active_map(&db,&server.id)?;
                    let tokens=if let Some(ref map)=active {placed_tokens(&db,&server.id,map["id"].as_str().unwrap_or(""))?}else{Vec::new()};
                    let combat=if let Some(ref map)=active {combat_state(&db,&server.id,map["id"].as_str().unwrap_or(""))?}else{None};
                    let fog=if let Some(ref map)=active {Some(fog_state(&db,&server.id,map["id"].as_str().unwrap_or(""))?)}else{None};
                    let visible_marks=if let Some(ref map)=active{marks(&db,&server.id,map["id"].as_str().unwrap_or(""),false)?}else{Vec::new()};
                    send(&peer,if approve{json!({"type":"approved","serverNowMs":now_ms(),"characters":sheets(&db,&server.id,Some(&target))?,"chat":history(&db,&server.id,&target,false)?,"activeMap":active,"placedTokens":tokens,"combat":combat,"fog":fog,"marks":visible_marks,"documents":documents(&db,&server.id,false)?,"roomArt":room_art(&db,&server.id,Some(&target))?,"roomAudio":room_audio(&db,&server.id,Some(&target))?,"playlist":playlist(&db,&server.id)?,"playback":playback(&db,&server.id)?})}else{json!({"type":"revoked"})});
                }
            }
            update_roster(server).await;
        }
        "chat"=>{
            let content=string_field(input,"text",2000)?;
            let audience=input["audience"].as_str().unwrap_or("all");
            if audience!="all"&&audience!="gm" {return Err("Destino de chat inválido".into());}
            let at=now();
            db.execute("INSERT INTO chat(room_id,author,text,created_at,audience,owner_token) VALUES (?1,?2,?3,?4,?5,?6)",params![server.id,name,content,at,audience,token]).map_err(|e|e.to_string())?;
            let event=json!({"type":"chat","id":db.last_insert_rowid(),"kind":"message","author":name,"text":content,"at":at,"audience":audience});
            if audience=="all" {broadcast(server,event).await;}else{private(server,token,event).await;}
        }
        "show_handout" if master=>{
            let id=string_field(input,"id",80)?;
            let (kind,title,body):(String,String,String)=db.query_row("SELECT kind,title,body FROM room_documents WHERE id=?1 AND room_id=?2",params![id,server.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|e|e.to_string())?.ok_or("Handout não encontrado")?;
            if kind!="handout"{return Err("Este documento não é um handout".into());}
            let at=now();let details=json!({"id":id,"title":title,"body":body});
            db.execute("INSERT INTO chat(room_id,author,text,created_at,kind,details) VALUES (?1,'Mestre',?2,?3,'handout',?4)",params![server.id,title,at,details.to_string()]).map_err(|e|e.to_string())?;
            broadcast(server,json!({"type":"chat","id":db.last_insert_rowid(),"kind":"handout","author":"Mestre","text":title,"at":at,"audience":"all","details":details})).await;
            broadcast(server,json!({"type":"handout_open","handout":details})).await;
        }
        "roll_chat"=>{
            let formula=string_field(input,"formula",30)?;
            let audience=input["audience"].as_str().unwrap_or("all");
            if !["all","gm","secret","self"].contains(&audience) {return Err("Destino de rolagem inválido".into());}
            let label=input["label"].as_str().unwrap_or("").trim();
            if label.chars().count()>80 {return Err("Nome da rolagem muito longo".into());}
            let (rolls,total)=roll_dice(formula)?;
            let report=format!("{} → {:?} = {}",formula,rolls,total);
            let details=json!({"formula":formula,"rolls":rolls,"total":total,"label":label});
            let at=now();
            db.execute("INSERT INTO chat(room_id,author,text,created_at,audience,owner_token,kind,details) VALUES (?1,?2,?3,?4,?5,?6,'roll',?7)",params![server.id,name,report,at,audience,token,details.to_string()]).map_err(|e|e.to_string())?;
            let event=json!({"type":"chat","id":db.last_insert_rowid(),"kind":"roll","author":name,"text":report,"at":at,"audience":audience,"details":details});
            match audience {
                "all"=>broadcast(server,event).await,
                "gm"=>private(server,token,event).await,
                "self"=>send(tx,event),
                _=>{
                    if let Some(master_tx)=server.live.lock().await.clients.get(&server.master_token).cloned(){send(&master_tx,event);}
                    if token!=server.master_token {
                        let confirmation="Rolagem oculta enviada ao mestre.";
                        db.execute("INSERT INTO chat(room_id,author,text,created_at,audience,owner_token,kind) VALUES (?1,'Sistema',?2,?3,'self',?4,'system')",params![server.id,confirmation,at,token]).map_err(|e|e.to_string())?;
                        send(tx,json!({"type":"chat","id":db.last_insert_rowid(),"kind":"system","author":"Sistema","text":confirmation,"at":at,"audience":"self"}));
                    }
                }
            }
        }
        "measure"=>{
            let current=active_map(&db,&server.id)?.ok_or("Medição exige mapa ativo")?;
            if current["id"]!=input["mapId"]{return Err("Medição fora do mapa ativo".into());}
            let mut points=[0.0;4];
            for (index,key) in ["x1","y1","x2","y2"].iter().enumerate() {
                let value=input[*key].as_f64().ok_or("Medição inválida")?;
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {return Err("Medição inválida".into());}
                points[index]=value;
            }
            let shape=input["shape"].as_str().unwrap_or("line");
            if !["line","corners"].contains(&shape){return Err("Forma de medição inválida".into());}
            let dx=(points[2]-points[0]).abs()*current["width"].as_f64().unwrap_or(0.0);
            let dy=(points[3]-points[1]).abs()*current["height"].as_f64().unwrap_or(0.0);
            let pixels=if shape=="line"{dx.hypot(dy)}else{dx+dy};
            let metres=(pixels/current["cellPx"].as_f64().unwrap_or(64.0)*current["metresPerCell"].as_f64().unwrap_or(1.5)*10.0).round()/10.0;
            broadcast(server,json!({"type":"measure","token":token,"author":name,"mapId":input["mapId"],"x1":points[0],"y1":points[1],"x2":points[2],"y2":points[3],"shape":shape,"metres":metres})).await;
        }
        "ping"=>{
            let map=active_map(&db,&server.id)?.ok_or("Nenhum mapa ativo")?;
            if input["mapId"]!=map["id"]{return Err("Mapa incorreto".into());}
            let (x,y)=map_point(input,&map)?;
            broadcast(server,json!({"type":"ping","mapId":map["id"],"x":x,"y":y,"author":name})).await;
        }
        "place_token"=>{
            let map=active_map(&db,&server.id)?.ok_or("Ative um mapa primeiro")?;
            let (x,y)=map_point(input,&map)?;
            let art_id=input["artId"].as_str().filter(|s|!s.is_empty());
            let sheet_id=input["sheetId"].as_str().filter(|s|!s.is_empty());
            let threat_id=input["threatId"].as_str().filter(|s|!s.is_empty());
            if art_id.is_none()&&sheet_id.is_none()&&threat_id.is_none(){return Err("Escolha uma ficha, ameaça ou imagem".into());}
            if sheet_id.is_some()&&threat_id.is_some(){return Err("Token não pode representar ficha e ameaça".into());}
            if !master && (art_id.is_none()||sheet_id.is_some()||threat_id.is_some()){return Err("Jogadores só podem colocar artes próprias".into());}
            let art_name=if let Some(id)=art_id {db.query_row("SELECT name FROM room_token_art WHERE id=?1 AND room_id=?2 AND (?3=1 OR owner_token=?4)",params![id,server.id,master as i64,token],|r|r.get::<_,String>(0)).optional().map_err(|e|e.to_string())?.ok_or("Imagem indisponível")?}else{String::new()};
            let sheet_name=if let Some(id)=sheet_id {let raw:String=db.query_row("SELECT data FROM room_characters WHERE id=?1 AND room_id=?2",params![id,server.id],|r|r.get(0)).optional().map_err(|e|e.to_string())?.ok_or("Ficha não encontrada")?;serde_json::from_str::<Value>(&raw).ok().and_then(|v|v["name"].as_str().map(str::to_owned)).unwrap_or("Personagem".into())}else{String::new()};
            let threat_data=if let Some(id)=threat_id {let raw:String=db.query_row("SELECT data FROM room_threats WHERE id=?1 AND room_id=?2",params![id,server.id],|r|r.get(0)).optional().map_err(|e|e.to_string())?.ok_or("Ameaça não encontrada")?;Some(serde_json::from_str::<Value>(&raw).map_err(|_|"Ameaça inválida")?)}else{None};
            let name=if let Some(ref threat)=threat_data{threat["name"].as_str().unwrap_or("Ameaça").to_string()}else if sheet_id.is_some(){sheet_name}else{art_name};
            let id=Uuid::new_v4().to_string();
            let size=map["cellPx"].as_f64().unwrap_or(64.0);
            let owner=if master{None}else{participant(&db,&server.id,token)?.map(|p|p.0)};
            {
                let txdb=db.transaction().map_err(|e|e.to_string())?;
                txdb.execute("INSERT INTO room_tokens(id,room_id,map_id,art_id,sheet_id,name,x,y,size,owner_participant_id,creator_participant_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![id,server.id,map["id"].as_str(),art_id,sheet_id,name,x,y,size,owner,owner]).map_err(|e|e.to_string())?;
                if let (Some(threat_id),Some(threat))=(threat_id,threat_data.as_ref()){
                    txdb.execute("INSERT INTO room_threat_instances(token_id,room_id,threat_id,current_hp,current_mp) VALUES (?1,?2,?3,?4,?5)",params![id,server.id,threat_id,threat["maxHp"].as_i64().unwrap_or(1),threat["maxMp"].as_i64().unwrap_or(0)]).map_err(|e|e.to_string())?;
                }
                txdb.commit().map_err(|e|e.to_string())?;
            }
            broadcast(server,json!({"type":"token_upsert","token":token_record(&db,&server.id,&id)?})).await;
            if threat_id.is_some(){private(server,&server.master_token,json!({"type":"threat_instances","items":threat_instances(&db,&server.id,map["id"].as_str().unwrap_or(""))?})).await;}
        }
        "move_token"=>{
            let id=string_field(input,"id",80)?;
            let placed=token_record(&db,&server.id,id)?.ok_or("Token não encontrado")?;
            let map=active_map(&db,&server.id)?.ok_or("Nenhum mapa ativo")?;
            if placed["mapId"]!=map["id"]{return Err("Token fora do mapa ativo".into());}
            if !master {
                let owns=if let Some(sheet_id)=placed["sheetId"].as_str(){
                    db.query_row("SELECT owner_token=?1 FROM room_characters WHERE id=?2 AND room_id=?3",params![token,sheet_id,server.id],|r|r.get::<_,bool>(0)).unwrap_or(false)
                }else{
                    db.query_row("SELECT p.token=?1 FROM participants p WHERE p.id=?2 AND p.room_id=?3 AND p.approved=1 AND p.revoked=0",params![token,placed["ownerParticipantId"].as_str().unwrap_or(""),server.id],|r|r.get::<_,bool>(0)).unwrap_or(false)
                };
                if !owns{return Err("Sem permissão para mover este token".into());}
            }
            let base=input["baseRevision"].as_i64().ok_or("Revisão ausente")?;
            let (x,y)=map_point(input,&map)?;
            let changed=db.execute("UPDATE room_tokens SET x=?1,y=?2,revision=revision+1 WHERE id=?3 AND room_id=?4 AND revision=?5",params![x,y,id,server.id,base]).map_err(|e|e.to_string())?;
            if changed==0 {send(tx,json!({"type":"token_conflict","token":token_record(&db,&server.id,id)?}));}
            else {broadcast(server,json!({"type":"token_upsert","token":token_record(&db,&server.id,id)?})).await;}
        }
        "update_token"=>{
            let id=string_field(input,"id",80)?;
            let placed=token_record(&db,&server.id,id)?.ok_or("Token não encontrado")?;
            if !master && (placed["sheetId"].as_str().is_some()||placed["creatorParticipantId"]!=participant(&db,&server.id,token)?.map(|p|json!(p.0)).unwrap_or(Value::Null)){return Err("Sem permissão para editar este token".into());}
            let name=string_field(input,"name",80)?;
            let size=input["size"].as_f64().ok_or("Tamanho inválido")?;
            if !size.is_finite()||!(16.0..=2048.0).contains(&size){return Err("Tamanho inválido".into());}
            let owner=input["ownerParticipantId"].as_str().filter(|s|!s.is_empty());
            if !master && owner!=placed["ownerParticipantId"].as_str(){return Err("Só o mestre pode mudar o responsável".into());}
            if !placed["sheetId"].is_null()&&owner.is_some(){return Err("O dono deste token vem da ficha".into());}
            if let Some(owner)=owner {
                let exists:bool=db.query_row("SELECT 1 FROM participants WHERE id=?1 AND room_id=?2 AND approved=1 AND revoked=0",params![owner,server.id],|_|Ok(true)).optional().map_err(|e|e.to_string())?.unwrap_or(false);
                if !exists{return Err("Jogador não aprovado".into());}
            }
            db.execute("UPDATE room_tokens SET name=?1,size=?2,owner_participant_id=?3,revision=revision+1 WHERE id=?4 AND room_id=?5",params![name,size,owner,id,server.id]).map_err(|e|e.to_string())?;
            if active_map(&db,&server.id)?.is_some_and(|m|m["id"]==placed["mapId"]){broadcast(server,json!({"type":"token_upsert","token":token_record(&db,&server.id,id)?})).await;}
        }
        "remove_token"=>{
            let id=string_field(input,"id",80)?;
            let placed=token_record(&db,&server.id,id)?.ok_or("Token não encontrado")?;
            if !master && (placed["sheetId"].as_str().is_some()||placed["creatorParticipantId"]!=participant(&db,&server.id,token)?.map(|p|json!(p.0)).unwrap_or(Value::Null)){return Err("Sem permissão para retirar este token".into());}
            let was_threat:bool=db.query_row("SELECT 1 FROM room_threat_instances WHERE token_id=?1 AND room_id=?2",params![id,server.id],|_|Ok(true)).optional().map_err(|e|e.to_string())?.unwrap_or(false);
            {
                let txdb=db.transaction().map_err(|e|e.to_string())?;
                txdb.execute("DELETE FROM room_threat_instances WHERE token_id=?1 AND room_id=?2",params![id,server.id]).map_err(|e|e.to_string())?;
                txdb.execute("DELETE FROM room_tokens WHERE id=?1 AND room_id=?2",params![id,server.id]).map_err(|e|e.to_string())?;
                txdb.commit().map_err(|e|e.to_string())?;
            }
            if active_map(&db,&server.id)?.is_some_and(|m|m["id"]==placed["mapId"]){broadcast(server,json!({"type":"token_removed","id":id})).await;if was_threat{private(server,&server.master_token,json!({"type":"threat_instances","items":threat_instances(&db,&server.id,placed["mapId"].as_str().unwrap_or(""))?})).await;}}
        }
        "set_threat_resource" if master=>{
            let id=string_field(input,"tokenId",80)?;
            let resource=input["resource"].as_str().ok_or("Recurso ausente")?;
            if resource!="hp"&&resource!="mp"{return Err("Recurso inválido".into());}
            let mode=input["mode"].as_str().ok_or("Modo ausente")?;
            if mode!="delta"&&mode!="set"{return Err("Modo inválido".into());}
            let value=input["value"].as_i64().filter(|n|(-1_000_000..=1_000_000).contains(n)).ok_or("Valor inválido")?;
            let base=input["baseRevision"].as_i64().ok_or("Revisão ausente")?;
            let (hp,mp,revision,name,raw):(i64,i64,i64,String,String)=db.query_row("SELECT i.current_hp,i.current_mp,i.revision,t.name,b.data FROM room_threat_instances i JOIN room_tokens t ON t.id=i.token_id AND t.room_id=i.room_id JOIN room_threats b ON b.id=i.threat_id AND b.room_id=i.room_id WHERE i.token_id=?1 AND i.room_id=?2",params![id,server.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(|e|e.to_string())?.ok_or("Instância de ameaça não encontrada")?;
            if revision!=base{return Err("Ameaça alterada por outra ação; atualize antes de salvar".into());}
            let template:Value=serde_json::from_str(&raw).map_err(|_|"Ameaça inválida")?;
            let maximum=template[if resource=="hp"{"maxHp"}else{"maxMp"}].as_i64().ok_or("Recurso máximo inválido")?;
            let current=if resource=="hp"{hp}else{mp};
            let next=if mode=="delta"{current.saturating_add(value)}else{value};
            let minimum=if resource=="hp"{-1_000_000}else{0};
            if !(minimum..=maximum).contains(&next){return Err("Valor fora do limite da ameaça".into());}
            let text=format!("{name}: {} {current} → {next}",if resource=="hp"{"PV"}else{"PM"});
            let at=now();
            let message_id={
                let txdb=db.transaction().map_err(|e|e.to_string())?;
                let changed=txdb.execute(if resource=="hp"{"UPDATE room_threat_instances SET current_hp=?1,revision=revision+1 WHERE token_id=?2 AND room_id=?3 AND revision=?4"}else{"UPDATE room_threat_instances SET current_mp=?1,revision=revision+1 WHERE token_id=?2 AND room_id=?3 AND revision=?4"},params![next,id,server.id,base]).map_err(|e|e.to_string())?;
                if changed==0{return Err("Ameaça alterada por outra ação; atualize antes de salvar".into());}
                txdb.execute("INSERT INTO chat(room_id,author,text,created_at,audience,owner_token,kind) VALUES (?1,'Mestre',?2,?3,'gm',?4,'action')",params![server.id,text,at,server.master_token]).map_err(|e|e.to_string())?;
                let message_id=txdb.last_insert_rowid();
                txdb.commit().map_err(|e|e.to_string())?;
                message_id
            };
            private(server,&server.master_token,json!({"type":"chat","id":message_id,"kind":"action","author":"Mestre","text":text,"at":at,"audience":"gm"})).await;
            if let Some(map)=active_map(&db,&server.id)?{private(server,&server.master_token,json!({"type":"threat_instances","items":threat_instances(&db,&server.id,map["id"].as_str().unwrap_or(""))?})).await;}
        }
        "set_threat_condition" if master=>{
            let id=string_field(input,"tokenId",80)?;
            let condition=string_field(input,"name",60)?;
            let action=input["action"].as_str().ok_or("Ação ausente")?;
            if action!="add"&&action!="remove"{return Err("Ação inválida".into());}
            let base=input["baseRevision"].as_i64().ok_or("Revisão ausente")?;
            let (raw,revision,name,map_id):(String,i64,String,String)=db.query_row("SELECT i.conditions,i.revision,t.name,t.map_id FROM room_threat_instances i JOIN room_tokens t ON t.id=i.token_id AND t.room_id=i.room_id WHERE i.token_id=?1 AND i.room_id=?2",params![id,server.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(|e|e.to_string())?.ok_or("Ameaça não encontrada")?;
            if revision!=base{return Err("Ameaça alterada por outra ação; atualize antes de salvar".into());}
            let mut items:Vec<Value>=serde_json::from_str(&raw).map_err(|_|"Condições inválidas")?;
            items.retain(|item|item["name"]!=condition);
            if action=="add"{
                let unit=input["unit"].as_str().ok_or("Duração ausente")?;
                if unit!="manual"&&unit!="rounds"{return Err("Duração inválida".into());}
                let remaining=if unit=="rounds"{Some(input["remaining"].as_i64().filter(|n|(1..=100).contains(n)).ok_or("Duração inválida")?)}else{None};
                if items.len()>=20{return Err("Limite de 20 condições por ameaça".into());}
                items.push(json!({"name":condition,"unit":unit,"remaining":remaining}));
            }
            let report=format!("{name}: {} {condition}",if action=="add"{"recebeu"}else{"removeu"});
            let at=now();
            let message_id={
                let txdb=db.transaction().map_err(|e|e.to_string())?;
                if txdb.execute("UPDATE room_threat_instances SET conditions=?1,revision=revision+1 WHERE token_id=?2 AND room_id=?3 AND revision=?4",params![json!(items).to_string(),id,server.id,base]).map_err(|e|e.to_string())?==0{return Err("Ameaça alterada por outra ação; atualize antes de salvar".into());}
                txdb.execute("INSERT INTO chat(room_id,author,text,created_at,audience,owner_token,kind) VALUES (?1,'Mestre',?2,?3,'gm',?4,'action')",params![server.id,report,at,server.master_token]).map_err(|e|e.to_string())?;
                let message_id=txdb.last_insert_rowid();txdb.commit().map_err(|e|e.to_string())?;message_id
            };
            private(server,&server.master_token,json!({"type":"chat","id":message_id,"kind":"action","author":"Mestre","text":report,"at":at,"audience":"gm"})).await;
            if active_map(&db,&server.id)?.is_some_and(|map|map["id"]==map_id){private(server,&server.master_token,json!({"type":"threat_instances","items":threat_instances(&db,&server.id,&map_id)?})).await;broadcast(server,json!({"type":"token_upsert","token":token_record(&db,&server.id,id)?})).await;}
        }
        "start_combat" if master=>{
            let map=active_map(&db,&server.id)?.ok_or("Ative uma cena antes do combate")?;
            let map_id=map["id"].as_str().ok_or("Cena inválida")?;
            if map["kind"]!="tactical"{return Err("Combate precisa de uma cena tática".into());}
            let combat=new_combat(&db,&server.id,map_id,&input["initiatives"])?;
            save_combat(&db,&server.id,map_id,&combat)?;
            broadcast(server,json!({"type":"combat","combat":combat_state(&db,&server.id,map_id)?})).await;
        }
        "combat_next"|"combat_previous"|"combat_pause"=>{
            let map=active_map(&db,&server.id)?.ok_or("Nenhuma cena ativa")?;
            let map_id=map["id"].as_str().ok_or("Cena inválida")?;
            let mut combat=combat_state(&db,&server.id,map_id)?.ok_or("Nenhum combate em andamento nesta cena")?;
            let revision=combat["revision"].as_i64().ok_or("Combate inválido")?;
            if input["baseRevision"].as_i64()!=Some(revision){return Err("O combate mudou; tente novamente".into());}
            let entries=combat["entries"].as_array().ok_or("Combate inválido")?;
            let count=entries.len();
            let current=combat["turn"].as_u64().ok_or("Turno inválido")? as usize;
            if count==0||current>=count{return Err("Turno inválido".into());}
            if input["type"]=="combat_next"&&combat["paused"]==true{return Err("O combate está pausado".into());}
            if !master {
                if input["type"]!="combat_next"{return Err("Só o mestre pode alterar este combate".into());}
                let player_id=participant(&db,&server.id,token)?.map(|p|p.0).ok_or("Jogador não encontrado")?;
                if entries[current]["ownerParticipantId"].as_str()!=Some(player_id.as_str()){return Err("Aguarde seu turno".into());}
            }
            let mut round_advanced=false;
            match input["type"].as_str().unwrap_or(""){
                "combat_next"=>{let next=(current+1)%count;combat["turn"]=json!(next);if next==0{round_advanced=true;combat["round"]=json!(combat["round"].as_u64().unwrap_or(1)+1);}},
                "combat_previous"=>{let previous=if current==0{count-1}else{current-1};combat["turn"]=json!(previous);if current==0{combat["round"]=json!(combat["round"].as_u64().unwrap_or(1).saturating_sub(1).max(1));}},
                _=>combat["paused"]=json!(!combat["paused"].as_bool().unwrap_or(false)),
            }
            combat.as_object_mut().ok_or("Combate inválido")?.remove("revision");
            let changed_conditions={
                let txdb=db.transaction().map_err(|e|e.to_string())?;
                update_combat(&txdb,&server.id,map_id,&combat,revision)?;
                let changed=if round_advanced{tick_threat_conditions(&txdb,&server.id,map_id)?}else{Vec::new()};
                txdb.commit().map_err(|e|e.to_string())?;
                changed
            };
            broadcast(server,json!({"type":"combat","combat":combat_state(&db,&server.id,map_id)?})).await;
            if !changed_conditions.is_empty(){private(server,&server.master_token,json!({"type":"threat_instances","items":threat_instances(&db,&server.id,map_id)?})).await;for id in changed_conditions{broadcast(server,json!({"type":"token_upsert","token":token_record(&db,&server.id,&id)?})).await;}}
        }
        "end_combat" if master=>{
            let map=active_map(&db,&server.id)?.ok_or("Nenhuma cena ativa")?;
            let map_id=map["id"].as_str().ok_or("Cena inválida")?;
            let changed=db.execute("DELETE FROM room_combat WHERE room_id=?1 AND map_id=?2",params![server.id,map_id]).map_err(|e|e.to_string())?;
            if changed==0{return Err("Nenhum combate em andamento nesta cena".into());}
            broadcast(server,json!({"type":"combat","combat":Value::Null})).await;
        }
        "mark_save"=>{
            let map=active_map(&db,&server.id)?.ok_or("Nenhuma cena ativa")?;
            let map_id=map["id"].as_str().ok_or("Cena inválida")?;
            let kind=input["kind"].as_str().ok_or("Tipo de marcação ausente")?;
            if kind=="pin"&&!master{return Err("Apenas o mestre pode colocar alfinetes".into());}
            let visibility=input["visibility"].as_str().unwrap_or("public");
            if visibility!="public"&&!(kind=="pin"&&master&&visibility=="private"){return Err("Visibilidade inválida".into());}
            let data=validate_mark(kind,&input["data"],&map)?;
            let requested_id=match input.get("id") {None|Some(Value::Null)=>None,Some(Value::String(id))=>Some(id.as_str()),_=>return Err("Identificador inválido".into())};
            if let Some(id)=requested_id{
                if Uuid::parse_str(id).is_err(){return Err("Identificador inválido".into());}
                let existing:Option<(String,String,i64)>=db.query_row("SELECT owner_token,kind,revision FROM room_marks WHERE id=?1 AND room_id=?2 AND map_id=?3",params![id,server.id,map_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|e|e.to_string())?;
                let (owner,old_kind,revision)=existing.ok_or("Marcação não encontrada")?;
                if !master&&owner!=token{return Err("Você só pode editar suas marcações".into());}
                if old_kind!=kind||input["baseRevision"].as_i64()!=Some(revision){return Err("Marcação alterada; tente novamente".into());}
                if db.execute("UPDATE room_marks SET data=?1,visibility=?2,revision=revision+1 WHERE id=?3 AND room_id=?4 AND map_id=?5 AND revision=?6",params![data.to_string(),visibility,id,server.id,map_id,revision]).map_err(|e|e.to_string())?==0{return Err("Marcação alterada; tente novamente".into());}
            }else{
                let count:i64=db.query_row("SELECT COUNT(*) FROM room_marks WHERE room_id=?1 AND map_id=?2",params![server.id,map_id],|r|r.get(0)).map_err(|e|e.to_string())?;
                if count>=500{return Err("Limite de 500 marcações nesta cena".into());}
                db.execute("INSERT INTO room_marks(id,room_id,map_id,kind,owner_token,author,visibility,data) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![Uuid::new_v4().to_string(),server.id,map_id,kind,token,name,visibility,data.to_string()]).map_err(|e|e.to_string())?;
            }
            let (public,all)=mark_lists(&db,&server.id,map_id)?;
            broadcast_marks(server,public,all).await;
        }
        "mark_delete"=>{
            let map=active_map(&db,&server.id)?.ok_or("Nenhuma cena ativa")?;
            let map_id=map["id"].as_str().ok_or("Cena inválida")?;
            let id=string_field(input,"id",80)?;
            let existing:Option<(String,i64)>=db.query_row("SELECT owner_token,revision FROM room_marks WHERE id=?1 AND room_id=?2 AND map_id=?3",params![id,server.id,map_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
            let (owner,revision)=existing.ok_or("Marcação não encontrada")?;
            if !master&&owner!=token{return Err("Você só pode apagar suas marcações".into());}
            if input["baseRevision"].as_i64()!=Some(revision){return Err("Marcação alterada; tente novamente".into());}
            db.execute("DELETE FROM room_marks WHERE id=?1 AND room_id=?2 AND map_id=?3 AND revision=?4",params![id,server.id,map_id,revision]).map_err(|e|e.to_string())?;
            let (public,all)=mark_lists(&db,&server.id,map_id)?;
            broadcast_marks(server,public,all).await;
        }
        "fog_set" if master=>{
            let map=active_map(&db,&server.id)?.ok_or("Nenhuma cena ativa")?;
            if map["kind"]!="tactical"{return Err("Névoa só funciona em cena tática".into());}
            let map_id=map["id"].as_str().ok_or("Cena inválida")?;
            let mut fog=fog_state(&db,&server.id,map_id)?;
            let revision=fog["revision"].as_i64().ok_or("Névoa inválida")?;
            if input["baseRevision"].as_i64()!=Some(revision){return Err("A névoa mudou; tente novamente".into());}
            match input["action"].as_str().unwrap_or(""){
                "cover_all"=>{fog["enabled"]=json!(true);fog["ops"]=json!([]);},
                "show_all"=>fog["enabled"]=json!(false),
                "resume"=>fog["enabled"]=json!(true),
                "rect"=>{
                    if fog["enabled"]!=true{return Err("Cubra a cena antes de editar a névoa".into());}
                    let mode=input["mode"].as_str().ok_or("Modo de névoa ausente")?;
                    if mode!="reveal"&&mode!="hide"{return Err("Modo de névoa inválido".into());}
                    let mut values=[0.0;4];
                    for (index,key) in ["x","y","width","height"].iter().enumerate(){values[index]=input[*key].as_f64().filter(|v|v.is_finite()).ok_or("Área de névoa inválida")?;}
                    let (x,y,width,height)=(values[0],values[1],values[2],values[3]);
                    if x<0.0||y<0.0||width<1.0||height<1.0||x+width>map["width"].as_f64().unwrap_or(0.0)+0.01||y+height>map["height"].as_f64().unwrap_or(0.0)+0.01{return Err("Área fora da cena".into());}
                    let ops=fog["ops"].as_array_mut().ok_or("Névoa inválida")?;
                    if ops.len()>=500{return Err("Limite de 500 áreas atingido; simplifique a névoa".into());}
                    ops.push(json!({"mode":mode,"x":x,"y":y,"width":width,"height":height}));
                }
                "polygon"=>{
                    if fog["enabled"]!=true{return Err("Cubra a cena antes de editar a névoa".into());}
                    let mode=input["mode"].as_str().filter(|s|["reveal","hide"].contains(s)).ok_or("Modo de névoa inválido")?;
                    let points=input["points"].as_array().filter(|p|(3..=64).contains(&p.len())).ok_or("Polígono deve ter de 3 a 64 pontos")?;
                    let points=points.iter().map(|p|mark_point(p,&map)).collect::<Result<Vec<_>,_>>()?;
                    let area:f64=points.iter().enumerate().map(|(i,p)|{let q=&points[(i+1)%points.len()];p["x"].as_f64().unwrap()*q["y"].as_f64().unwrap()-q["x"].as_f64().unwrap()*p["y"].as_f64().unwrap()}).sum::<f64>().abs()/2.0;
                    if area<1.0{return Err("Polígono muito pequeno".into());}
                    let ops=fog["ops"].as_array_mut().ok_or("Névoa inválida")?;
                    if ops.len()>=500{return Err("Limite de 500 áreas atingido; simplifique a névoa".into());}
                    ops.push(json!({"mode":mode,"shape":"polygon","points":points}));
                }
                _=>return Err("Ação de névoa inválida".into()),
            }
            fog.as_object_mut().ok_or("Névoa inválida")?.remove("revision");
            db.execute("INSERT OR IGNORE INTO room_fog(map_id,room_id,state,revision) VALUES (?1,?2,'{}',0)",params![map_id,server.id]).map_err(|e|e.to_string())?;
            let changed=db.execute("UPDATE room_fog SET state=?1,revision=revision+1 WHERE map_id=?2 AND room_id=?3 AND revision=?4",params![fog.to_string(),map_id,server.id,revision]).map_err(|e|e.to_string())?;
            if changed==0{return Err("A névoa mudou; tente novamente".into());}
            broadcast(server,json!({"type":"fog","fog":fog_state(&db,&server.id,map_id)?})).await;
        }
        "set_playlist" if master=>{
            let ids=input["ids"].as_array().ok_or("Fila inválida")?;
            if ids.len()>100{return Err("Fila grande demais".into());}
            let mut unique=std::collections::HashSet::new();
            for id in ids {
                let id=id.as_str().ok_or("Faixa inválida")?;
                if !unique.insert(id){return Err("Faixa repetida".into());}
                let exists:bool=db.query_row("SELECT 1 FROM room_audio WHERE id=?1 AND room_id=?2",params![id,server.id],|_|Ok(true)).optional().map_err(|e|e.to_string())?.unwrap_or(false);
                if !exists{return Err("Faixa não encontrada".into());}
            }
            {
                let txdb=db.transaction().map_err(|e|e.to_string())?;
                txdb.execute("DELETE FROM room_playlist WHERE room_id=?1",[&server.id]).map_err(|e|e.to_string())?;
                for (position,id) in ids.iter().enumerate(){txdb.execute("INSERT INTO room_playlist(room_id,position,audio_id) VALUES (?1,?2,?3)",params![server.id,position as i64,id.as_str()]).map_err(|e|e.to_string())?;}
                if let Some(current)=playback(&txdb,&server.id)?["audioId"].as_str(){if !unique.contains(current){txdb.execute("DELETE FROM room_playback WHERE room_id=?1",[&server.id]).map_err(|e|e.to_string())?;}}
                txdb.commit().map_err(|e|e.to_string())?;
            }
            let public_ids=playlist(&db,&server.id)?;
            let tracks=room_audio(&db,&server.id,None)?.into_iter().filter(|a|public_ids.iter().any(|id|a["id"]==*id)).collect::<Vec<_>>();
            broadcast(server,json!({"type":"playlist","serverNowMs":now_ms(),"ids":public_ids,"tracks":tracks,"playback":playback(&db,&server.id)?})).await;
        }
        "playback_control" if master=>{
            let action=string_field(input,"action",20)?;
            let previous=playback(&db,&server.id)?;
            let ids=playlist(&db,&server.id)?;
            let current=previous["audioId"].as_str();
            let elapsed=if previous["playing"]==true{(now_ms()-previous["startedAt"].as_i64().unwrap_or(0)).max(0) as f64/1000.0}else{0.0};
            let position=previous["positionSeconds"].as_f64().unwrap_or(0.0)+elapsed;
            let (next,pos,playing)=match action {
                "play"=>{let id=string_field(input,"audioId",80)?;if !ids.iter().any(|v|v==id){return Err("Faixa fora da fila".into());}(Some(id.to_string()),0.0,true)},
                "resume"=>{let id=current.ok_or("Nenhuma faixa selecionada")?;(Some(id.to_string()),position,true)},
                "pause"=>{let id=current.ok_or("Nenhuma faixa selecionada")?;(Some(id.to_string()),position,false)},
                "stop"=>(None,0.0,false),
                "next"=>{let index=current.and_then(|id|ids.iter().position(|v|v==id)).map(|i|i+1).unwrap_or(0);(ids.get(index).cloned(),0.0,ids.get(index).is_some())},
                _=>return Err("Controle inválido".into())
            };
            db.execute("INSERT INTO room_playback(room_id,audio_id,position_seconds,started_at,playing) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(room_id) DO UPDATE SET audio_id=excluded.audio_id,position_seconds=excluded.position_seconds,started_at=excluded.started_at,playing=excluded.playing",params![server.id,next,pos,now_ms(),playing as i64]).map_err(|e|e.to_string())?;
            broadcast(server,json!({"type":"playback","serverNowMs":now_ms(),"playback":playback(&db,&server.id)?})).await;
        }
        "offer_sheet" if !master=>{
            let data=input.get("data").ok_or("Ficha ausente")?;
            validate_sheet(data)?;
            server.live.lock().await.offers.insert(token.into(),data.clone());
            let id: String=db.query_row("SELECT id FROM participants WHERE token=?1",[token],|r|r.get(0)).map_err(|e|e.to_string())?;
            private(server,&server.master_token,json!({"type":"sheet_offer","participantId":id,"participantName":name,"data":data})).await;
            send(tx,json!({"type":"notice","message":"Ficha enviada; aguarde o mestre aceitar"}));
        }
        "accept_sheet"|"assign_sheet" if master=>{
            let id=string_field(input,"participantId",80)?;
            let owner:Option<String>=db.query_row("SELECT token FROM participants WHERE id=?1 AND room_id=?2 AND approved=1 AND revoked=0",params![id,server.id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
            let owner=owner.ok_or("Jogador não aprovado")?;
            let data=if input["type"]=="accept_sheet" {server.live.lock().await.offers.remove(&owner).ok_or("Não há ficha enviada")?}else{input.get("data").cloned().ok_or("Ficha ausente")?};
            validate_sheet(&data)?;
            let sheet_id=Uuid::new_v4().to_string();
            db.execute("INSERT INTO room_characters(id,room_id,owner_token,data,revision) VALUES (?1,?2,?3,?4,1)",params![sheet_id,server.id,owner,data.to_string()]).map_err(|e|e.to_string())?;
            private(server,&owner,json!({"type":"sheet","id":sheet_id,"owner":owner,"revision":1,"data":data,"canEditProficiencies":false})).await;
            if input["type"]=="accept_sheet" { send(tx,json!({"type":"offer_accepted","participantId":id})); }
        }
        "save_sheet"=>{
            let id=string_field(input,"id",80)?;
            let base=input["baseRevision"].as_i64().ok_or("Revisão ausente")?;
            let data=input.get("data").ok_or("Ficha ausente")?;
            validate_sheet(data)?;
            let stored:Option<(String,String,bool,i64)>=db.query_row("SELECT owner_token,data,can_edit_proficiencies,revision FROM room_characters WHERE id=?1 AND room_id=?2",params![id,server.id],|r|Ok((r.get(0)?,r.get(1)?,r.get::<_,i64>(2)?!=0,r.get(3)?))).optional().map_err(|e|e.to_string())?;
            let (owner,previous,can_edit,revision)=stored.ok_or("Ficha não encontrada")?;
            if !master&&owner!=token{return Err("Sem permissão para editar esta ficha".into());}
            if base!=revision {
                let current=sheets(&db,&server.id,if master{None}else{Some(token)})?.into_iter().find(|s|s["id"]==id).ok_or("Ficha não encontrada")?;
                send(tx,json!({"type":"conflict","sheet":current}));return Ok(());
            }
            if !master&&!can_edit {
                let previous:Value=serde_json::from_str(&previous).map_err(|e|e.to_string())?;
                for field in ["proficiencyAdds","proficiencyRemoves","proficiencyChoice"] {
                    let fallback=if field=="proficiencyChoice" {json!("Armas marciais")}else{json!([])};
                    if data.get(field).unwrap_or(&fallback)!=previous.get(field).unwrap_or(&fallback) {return Err("O mestre não autorizou mudanças nas proficiências".into());}
                }
            }
            let old:Value=serde_json::from_str(&previous).map_err(|e|e.to_string())?;
            if !master&&!mechanical_changes(&old,data,can_edit).is_empty() {
                let request_id=Uuid::new_v4().to_string();
                db.execute("INSERT INTO sheet_change_requests VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(sheet_id) DO UPDATE SET id=excluded.id,owner_token=excluded.owner_token,base_revision=excluded.base_revision,previous=excluded.previous,proposed=excluded.proposed",params![request_id,id,server.id,owner,base,previous,data.to_string()]).map_err(|e|e.to_string())?;
                private(server,&server.master_token,json!({"type":"sheet_requests","requests":sheet_requests(&db,&server.id)?})).await;
                send(tx,json!({"type":"notice","message":"Alterações mecânicas enviadas para revisão do mestre. A ficha em jogo não foi alterada."}));
                return Ok(());
            }
            let changed=db.execute("UPDATE room_characters SET data=?1,revision=revision+1 WHERE id=?2 AND room_id=?3 AND revision=?4",params![data.to_string(),id,server.id,base]).map_err(|e|e.to_string())?;
            if changed==0 {
                let current=sheets(&db,&server.id,if master{None}else{Some(token)})?.into_iter().find(|s|s["id"]==id).ok_or("Ficha não encontrada")?;
                send(tx,json!({"type":"conflict","sheet":current}));
            } else { private(server,&owner,json!({"type":"sheet","id":id,"owner":owner,"revision":base+1,"data":data,"canEditProficiencies":can_edit})).await; }
        }
        "activate_sheet_entry"=>{
            let id=string_field(input,"id",80)?;
            let base=input["baseRevision"].as_i64().ok_or("Revisão ausente")?;
            let collection=string_field(input,"collection",20)?;
            let entry_id=input["entryId"].as_i64().ok_or("Habilidade ausente")?;
            let active=input["active"].as_bool().ok_or("Estado inválido")?;
            let (owner,data,revision,can_edit)={
                let transaction=db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
                let (owner,raw,revision,can_edit):(String,String,i64,bool)=transaction.query_row("SELECT owner_token,data,revision,can_edit_proficiencies FROM room_characters WHERE id=?1 AND room_id=?2",params![id,server.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get::<_,i64>(3)?!=0))).optional().map_err(|e|e.to_string())?.ok_or("Ficha não encontrada")?;
                if !master&&owner!=token{return Err("Sem permissão para usar esta ficha".into());}
                if base!=revision{return Err("A ficha mudou. Carregue a versão atual antes de usar a habilidade.".into());}
                let previous:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;
                let data=activate_sheet_entry(&previous,collection,entry_id,active)?;
                transaction.execute("UPDATE room_characters SET data=?1,revision=revision+1 WHERE id=?2",params![data.to_string(),id]).map_err(|e|e.to_string())?;
                transaction.commit().map_err(|e|e.to_string())?;
                (owner,data,revision,can_edit)
            };
            private(server,&owner,json!({"type":"sheet","id":id,"owner":owner,"revision":revision+1,"data":data,"canEditProficiencies":can_edit,"reason":"action"})).await;
        }
        "review_sheet_change" if master=>{
            let request_id=string_field(input,"requestId",80)?;
            let accept=input["accept"].as_bool().ok_or("Decisão inválida")?;
            let (id,owner,base,data)={
            let transaction=db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
            let stored:Option<(String,String,i64,String)>=transaction.query_row("SELECT sheet_id,owner_token,base_revision,proposed FROM sheet_change_requests WHERE id=?1 AND room_id=?2",params![request_id,server.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(|e|e.to_string())?;
            let (id,owner,base,raw)=stored.ok_or("Solicitação não encontrada; atualize a revisão")?;
            let data:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;
            if accept {
                if !participant(&transaction,&server.id,&owner)?.is_some_and(|(_,_,approved,revoked)|approved&&!revoked){return Err("Jogador não aprovado".into());}
                validate_sheet(&data)?;
                let changed=transaction.execute("UPDATE room_characters SET data=?1,revision=revision+1 WHERE id=?2 AND room_id=?3 AND owner_token=?4 AND revision=?5",params![raw,id,server.id,owner,base]).map_err(|e|e.to_string())?;
                if changed==0{return Err("A ficha mudou depois do pedido. Recuse este pedido e solicite uma nova revisão.".into());}
            }
            transaction.execute("DELETE FROM sheet_change_requests WHERE id=?1",[request_id]).map_err(|e|e.to_string())?;
            transaction.commit().map_err(|e|e.to_string())?;
            (id,owner,base,data)
            };
            if accept {
                let sheet=sheets(&db,&server.id,None)?.into_iter().find(|s|s["id"]==id).ok_or("Ficha não encontrada")?;
                private(server,&owner,json!({"type":"sheet","id":id,"owner":owner,"revision":base+1,"data":data,"canEditProficiencies":sheet["canEditProficiencies"],"reason":"approved"})).await;
            } else {private(server,&owner,json!({"type":"notice","message":"O mestre recusou as alterações. Seu rascunho não foi apagado."})).await;}
            send(tx,json!({"type":"sheet_requests","requests":sheet_requests(&db,&server.id)?}));
        }
        "set_proficiency_permission" if master=>{
            let id=string_field(input,"sheetId",80)?;
            let allowed=input["allowed"].as_bool().ok_or("Permissão inválida")?;
            let changed=db.execute("UPDATE room_characters SET can_edit_proficiencies=?1 WHERE id=?2 AND room_id=?3",params![allowed as i64,id,server.id]).map_err(|e|e.to_string())?;
            if changed==0{return Err("Ficha não encontrada".into());}
            let sheet=sheets(&db,&server.id,None)?.into_iter().find(|s|s["id"]==id).ok_or("Ficha não encontrada")?;
            private(server,sheet["owner"].as_str().unwrap_or(""),json!({"type":"sheet_permission","id":id,"allowed":allowed})).await;
        }
        "roll_damage" if master=>{
            let id=string_field(input,"sheetId",80)?.to_string();
            let exists:bool=db.query_row("SELECT 1 FROM room_characters WHERE id=?1 AND room_id=?2",params![id,server.id],|_|Ok(true)).optional().map_err(|e|e.to_string())?.unwrap_or(false);
            if !exists{return Err("Ficha não encontrada".into());}
            let formula=string_field(input,"formula",30)?.to_string();
            let damage_type=input["damageType"].as_str().unwrap_or("Geral");
            if !["Geral","Ácido","Corte","Eletricidade","Essência","Fogo","Frio","Impacto","Luz","Perfuração","Psíquico","Trevas"].contains(&damage_type){return Err("Tipo de dano inválido".into());}
            let magical=input["magical"].as_bool().unwrap_or(false);
            let life_loss=input["lifeLoss"].as_bool().unwrap_or(false);
            let (rolls,total)=roll_dice(&formula)?;
            let total=total.max(0);
            let roll_id=Uuid::new_v4().to_string();
            server.live.lock().await.rolls.insert(roll_id.clone(),PendingRoll{sheet_id:id.clone(),formula:formula.clone(),rolls:rolls.clone(),total,damage_type:damage_type.into(),magical,life_loss});
            send(tx,json!({"type":"roll_ready","id":roll_id,"sheetId":id,"formula":formula,"rolls":rolls,"total":total,"damageType":damage_type,"magical":magical,"lifeLoss":life_loss}));
        }
        "apply_damage" if master=>{
            let roll_id=string_field(input,"id",80)?;
            let roll=server.live.lock().await.rolls.remove(roll_id).ok_or("Rolagem expirada")?;
            let (owner,data,revision,reduction,applied,immune,healed)={
                let transaction=db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
                let (owner,raw,revision):(String,String,i64)=transaction.query_row("SELECT owner_token,data,revision FROM room_characters WHERE id=?1 AND room_id=?2",params![roll.sheet_id,server.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|e.to_string())?;
                let mut data:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;
                let current=data["currentHp"].as_i64().unwrap_or(0).clamp(-1_000_000,1_000_000) as i32;
                let temporary=data["temporaryHp"].as_i64().unwrap_or(0).clamp(0,1_000_000) as i32;
                let immune=!roll.life_loss&&golem_immunity(&data)==Some(roll.damage_type.as_str());
                let reduction=if roll.life_loss||immune {0}else{damage_reduction(&data,&roll.damage_type)};
                let applied=if immune {0}else{(roll.total-reduction).max(0)};
                let maximum=data["maximumHp"].as_i64().unwrap_or(current as i64).clamp(current as i64,1_000_000) as i32;
                let healed=if immune&&roll.magical {(roll.total/2).min(maximum.saturating_sub(current))}else{0};
                let after=if roll.life_loss {ResourceState{current:current.saturating_sub(roll.total),maximum:0,temporary}}
                    else if immune {ResourceState{current:current.saturating_add(healed),maximum,temporary}}
                    else{take_damage(ResourceState{current,maximum:0,temporary},applied)};
                data["currentHp"]=json!(after.current);data["temporaryHp"]=json!(after.temporary);
                transaction.execute("UPDATE room_characters SET data=?1,revision=revision+1 WHERE id=?2",params![data.to_string(),roll.sheet_id]).map_err(|e|e.to_string())?;
                transaction.commit().map_err(|e|e.to_string())?;
                (owner,data,revision,reduction,applied,immune,healed)
            };
            private(server,&owner,json!({"type":"sheet","reason":"damage","id":roll.sheet_id,"owner":owner,"revision":revision+1,"data":data,"canEditProficiencies":sheets(&db,&server.id,Some(&owner))?.into_iter().find(|s|s["id"]==roll.sheet_id).map(|s|s["canEditProficiencies"].clone()).unwrap_or(json!(false))})).await;
            let target=data["name"].as_str().unwrap_or("Personagem");
            let report=if roll.life_loss {format!("{target} perdeu {} PV ({} → {:?}; sem RD ou PV temporários)",roll.total,roll.formula,roll.rolls)}
                else if immune {format!("{target} é imune a {}: {} de dano anulado{} ({} → {:?})",roll.damage_type,roll.total,if roll.magical{format!("; curou {healed} PV")}else{String::new()},roll.formula,roll.rolls)}
                else{format!("{target} sofreu {} de dano {}{} ({} → {:?}; RD {}; aplicado {})",roll.total,roll.damage_type,if roll.magical{" mágico"}else{""},roll.formula,roll.rolls,reduction,applied)};
            let at=now();
            db.execute("INSERT INTO chat(room_id,author,text,created_at,kind) VALUES (?1,'Mestre',?2,?3,'action')",params![server.id,report,at]).map_err(|e|e.to_string())?;
            broadcast(server,json!({"type":"chat","id":db.last_insert_rowid(),"kind":"action","author":"Mestre","text":report,"at":at})).await;
        }
        _=>return Err("Ação não permitida".into()),
    }
    Ok(())
}

fn roll_dice(formula:&str)->Result<(Vec<u32>,i32),String>{
    let formula=formula.trim().to_ascii_lowercase().replace(' ',"");
    let (count,suffix)=formula.split_once('d').ok_or("Use uma expressão como 2d6+3")?;
    let count:u32=count.parse().map_err(|_|"Quantidade de dados inválida")?;
    let sign=suffix.char_indices().find(|(i,c)|*i>0&&(*c=='+'||*c=='-'));
    let (sides,modifier)=if let Some((pos,_))=sign {(&suffix[..pos],suffix[pos..].parse::<i32>().map_err(|_|"Modificador inválido")?)}else{(suffix,0)};
    let sides:u32=sides.parse().map_err(|_|"Número de faces inválido")?;
    if !(1..=100).contains(&count)||!(2..=1000).contains(&sides)||!(-10_000..=10_000).contains(&modifier){return Err("Rolagem fora dos limites".into());}
    let mut rolls=Vec::with_capacity(count as usize);
    for _ in 0..count {
        let ceiling=u32::MAX-(u32::MAX%sides);
        let value=loop {let mut bytes=[0u8;4];getrandom::fill(&mut bytes).map_err(|e|e.to_string())?;let n=u32::from_le_bytes(bytes);if n<ceiling{break n%sides+1}};
        rolls.push(value);
    }
    let total=(rolls.iter().map(|n|*n as i32).sum::<i32>()+modifier).max(0);
    Ok((rolls,total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::{connect_async, tungstenite::Message as ClientMessage, MaybeTlsStream, WebSocketStream};

    type Client=WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
    async fn send_client(client:&mut Client,value:Value){client.send(ClientMessage::Text(value.to_string().into())).await.unwrap();}
    async fn receive(client:&mut Client,kind:&str)->Value{
        tokio::time::timeout(std::time::Duration::from_secs(5),async {
            loop {if let Some(Ok(ClientMessage::Text(raw)))=client.next().await {let value:Value=serde_json::from_str(&raw).unwrap();if value["type"]==kind{return value;}}}
        }).await.unwrap()
    }
    async fn roster_with(client:&mut Client,name:&str)->Value{
        loop {
            let roster=receive(client,"roster").await;
            if roster["participants"].as_array().unwrap().iter().any(|p|p["name"]==name){return roster;}
        }
    }

    #[test]
    fn validates_sheet_and_dice_limits(){
        assert!(validate_sheet(&json!({"name":"Heroína","level":1,"baseAttributes":[0,0,0,0,0,0]})).is_ok());
        assert!(validate_sheet(&json!({"name":"Legada","level":1,"notes":"Sem diário antigo"})).is_ok());
        assert!(validate_sheet(&json!({"name":"Diário","level":1,"diary":"x".repeat(20_001)})).is_err());
        assert!(validate_sheet(&json!({"name":"","level":1})).is_err());
        assert!(roll_dice("2d6+3").is_ok());
        assert!(roll_dice("1000d6").is_err());
    }

    #[test]
    fn threat_validation_rejects_forged_resources(){
        let base=json!({"name":"Goblin","nd":"1/4","role":"minion","initiative":2,"perception":0,"fortitude":0,"reflex":1,"will":0,"defense":12,"maxHp":6,"maxMp":0,"movement":9});
        assert!(validate_threat(&base).is_ok());
        for (key,value) in [("maxHp",json!(0)),("maxMp",json!(-1)),("defense",json!("999")),("initiative",json!(1001)),("role",json!("player"))]{let mut changed=base.clone();changed[key]=value;assert!(validate_threat(&changed).is_err(),"{key}");}
    }

    #[test]
    fn temporary_hp_absorbs_damage(){
        let after=take_damage(ResourceState{current:20,maximum:20,temporary:4},6);
        assert_eq!((after.current,after.temporary),(18,0));
    }

    #[test]
    fn mechanical_changes_default_to_review_not_a_client_approval_flag() {
        let previous=json!({"name":"Heroína","level":1,"spells":[{"id":1,"name":"Luz","open":false}]});
        let mut harmless=previous.clone();harmless["notes"]=json!("Anotação");harmless["spells"][0]["open"]=json!(true);
        assert!(mechanical_changes(&previous,&harmless,false).is_empty());
        for field in ["trainedSkills","skillAttributes","powers","deityPower","spells","temporaryAttributes","effects","currentMp","temporaryMp","level","equipment","approved","unknownFutureRule"] {
            let mut changed=previous.clone();changed[field]=json!(["forjado"]);
            assert!(mechanical_changes(&previous,&changed,false).contains(&field.into()), "{field}");
        }
        let mut changed=previous.clone();changed["proficiencyAdds"]=json!(["Escudos"]);
        assert!(!mechanical_changes(&previous,&changed,false).is_empty());
        assert!(mechanical_changes(&previous,&changed,true).is_empty());
    }

    #[test]
    fn activation_uses_stored_cost_and_preserves_resources_on_failure() {
        let original=json!({"name":"Teste","level":3,"currentMp":3,"temporaryMp":1,"spells":[{"id":1,"name":"Teste","circle":2,"active":false}],"effects":[{"id":2,"name":"Efeito","cost":2,"minimumLevel":2,"value":1,"active":false}]});
        let used=activate_sheet_entry(&original,"spells",1,true).unwrap();
        assert_eq!(used["currentMp"],1);assert_eq!(used["temporaryMp"],0);
        assert!(activate_sheet_entry(&used,"spells",1,true).is_err());
        let stopped=activate_sheet_entry(&used,"spells",1,false).unwrap();
        assert_eq!(stopped["currentMp"],1);
        assert!(activate_sheet_entry(&stopped,"spells",1,true).is_err());
        assert!(activate_sheet_entry(&original,"powers",1,true).is_err());
        assert!(activate_sheet_entry(&original,"effects",999,true).is_err());
        let mut invalid=original.clone();invalid["effects"][0]["cost"]=json!(-10);
        assert!(activate_sheet_entry(&invalid,"effects",2,true).is_err());
        invalid=original.clone();invalid["spells"][0]["circle"]=json!(5);
        assert!(activate_sheet_entry(&invalid,"spells",1,true).is_err());
        invalid=original.clone();invalid["spells"].as_array_mut().unwrap().push(original["spells"][0].clone());
        assert!(activate_sheet_entry(&invalid,"spells",1,true).is_err());
        let mut instant=original.clone();instant["spells"][0]["duration"]=json!("Instantânea");
        let result=activate_sheet_entry(&instant,"spells",1,true).unwrap();
        assert_eq!(result["spells"][0]["active"],false);assert_eq!(result["currentMp"],1);
        assert_eq!(original["currentMp"],3);
    }

    #[tokio::test]
    async fn sheet_review_requires_master_exact_request_and_current_revision() {
        let path=std::env::temp_dir().join(format!("vtt-review-{}.sqlite",Uuid::new_v4()));
        let db=open_db(&path).unwrap();
        db.execute("INSERT INTO rooms VALUES ('room','Teste',0,NULL,NULL,0)",[]).unwrap();
        db.execute("INSERT INTO participants VALUES ('p','room','Ana','player',1,0)",[]).unwrap();
        let original=json!({"name":"Teste","level":1,"baseAttributes":[0,0,0,0,0,0],"currentMp":3,"spells":[{"id":1,"name":"Luz","circle":1,"active":false}]});
        db.execute("INSERT INTO room_characters VALUES ('sheet','room','player',?1,1,0)",[original.to_string()]).unwrap();
        let (stop,_)=watch::channel(false);
        let server=RoomServer{id:"room".into(),name:"Teste".into(),code:"code".into(),master_token:"master".into(),db:path.clone(),assets:std::env::temp_dir(),live:Mutex::new(LiveRoom::default()),stop};
        let (tx,_)=mpsc::unbounded_channel();
        let mut proposed=original.clone();proposed["trainedSkills"]=json!(["Luta","Misticismo"]);
        let input=json!({"type":"save_sheet","id":"sheet","baseRevision":1,"data":proposed});
        assert!(handle(&server,&tx,"other","Outro",false,&input).await.is_err());
        handle(&server,&tx,"player","Ana",false,&input).await.unwrap();
        assert_eq!(sheets(&db,"room",None).unwrap()[0]["data"],original);
        let first=sheet_requests(&db,"room").unwrap()[0]["id"].clone();
        handle(&server,&tx,"player","Ana",false,&input).await.unwrap();
        let request=sheet_requests(&open_db(&path).unwrap(),"room").unwrap()[0].clone();
        assert_ne!(request["id"],first);
        assert!(handle(&server,&tx,"master","Mestre",true,&json!({"type":"review_sheet_change","requestId":first,"accept":true})).await.is_err());
        let approve=json!({"type":"review_sheet_change","requestId":request["id"],"accept":true});
        assert!(handle(&server,&tx,"player","Ana",false,&approve).await.is_err());
        handle(&server,&tx,"master","Mestre",true,&approve).await.unwrap();
        assert_eq!(sheets(&db,"room",None).unwrap()[0]["data"],proposed);
        assert_eq!(sheets(&db,"room",None).unwrap()[0]["revision"],2);
        assert!(sheet_requests(&db,"room").unwrap().is_empty());
        assert!(handle(&server,&tx,"master","Mestre",true,&approve).await.is_err());
        proposed["currentMp"]=json!(100);
        handle(&server,&tx,"player","Ana",false,&json!({"type":"save_sheet","id":"sheet","baseRevision":2,"data":proposed})).await.unwrap();
        let stale=sheet_requests(&db,"room").unwrap()[0]["id"].clone();
        db.execute("UPDATE room_characters SET revision=3 WHERE id='sheet'",[]).unwrap();
        assert!(handle(&server,&tx,"master","Mestre",true,&json!({"type":"review_sheet_change","requestId":stale,"accept":true})).await.is_err());
        assert_eq!(sheets(&db,"room",None).unwrap()[0]["data"]["currentMp"],3);
        handle(&server,&tx,"master","Mestre",true,&json!({"type":"review_sheet_change","requestId":stale,"accept":false})).await.unwrap();
        assert!(sheet_requests(&db,"room").unwrap().is_empty());
        let activation=json!({"type":"activate_sheet_entry","id":"sheet","baseRevision":3,"collection":"spells","entryId":1,"active":true,"cost":-100,"currentMp":999});
        assert!(handle(&server,&tx,"other","Outro",false,&activation).await.is_err());
        handle(&server,&tx,"player","Ana",false,&activation).await.unwrap();
        assert_eq!(sheets(&db,"room",None).unwrap()[0]["data"]["currentMp"],2);
        assert!(handle(&server,&tx,"player","Ana",false,&activation).await.is_err());
        assert_eq!(sheets(&db,"room",None).unwrap()[0]["data"]["currentMp"],2);
        drop(db);std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn sheet_rejects_rule_bypasses_at_persistence_boundary() {
        let base = json!({"name":"Auditoria","level":1,"baseAttributes":[4,3,0,0,0,-1]});
        assert!(validate_sheet(&base).is_ok());
        for (field, value) in [
            ("baseAttributes", json!([4,4,4,4,4,4])),
            ("baseAttributes", json!(["4",0,0,0,0,0])),
            ("level", json!(1.5)), ("level", json!("20")), ("level", Value::Null),
            ("currentMp", json!(-1)), ("temporaryHp", json!(-1)), ("temporaryMp", json!(-1)),
            ("equipment", json!({})), ("powers", json!([null])), ("spells", json!("magia")),
            ("temporaryAttributes", json!([i64::MAX,0,0,0,0,0])),
            ("manualRd", json!({"Geral":i64::MAX})),
            ("skillAttributes", json!({"Luta":6})),
            ("coins", json!({"to":-1})), ("coins", json!({"to":37000})),
            ("effects", json!([{"cost":-1,"minimumLevel":1,"value":1,"active":true}])),
            ("equipment", json!([{"quantity":0.5}])),
            ("equipment", json!([{"quantity":0,"equipped":true}])),
            ("equipment", json!([{"equipped":true,"itemType":"Armadura"},{"equipped":true,"itemType":"Armadura"}])),
        ] {
            let mut sheet=base.clone(); sheet[field]=value.clone();
            assert!(validate_sheet(&sheet).is_err(), "accepted {field}: {value}");
        }
        let mut sheet=base.clone(); sheet["currentHp"]=json!(-5);
        assert!(validate_sheet(&sheet).is_ok(), "PV negativo é permitido");
        for (ancestry, choices) in [("Humano",json!([0,0])),("Humano",json!([0,1,2,3])),("Lefou",json!([5])),("Anão",json!([0])),("Humano",json!([6]))] {
            let mut sheet=base.clone(); sheet["ancestry"]=json!(ancestry);sheet["ancestryChoices"]=choices;
            assert!(validate_sheet(&sheet).is_err());
        }
        let mut sheet=base;sheet["ancestry"]=json!("Humano");sheet["ancestryChoices"]=json!([0,1,2]);
        assert!(validate_sheet(&sheet).is_ok(), "bônus racial não consome orçamento base");
    }

    #[test]
    fn load_rd_and_golem_rules(){
        let sheet=json!({"name":"Teste","level":1,"baseAttributes":[0,0,0,0,0,0],"equipment":[{"spaces":10.5,"quantity":2}]});
        assert!(validate_sheet(&sheet).is_err());
        let sheet=json!({"name":"Teste","level":1,"baseAttributes":[1,0,0,0,0,0],"equipment":[{"spaces":12,"quantity":2}]});
        assert!(validate_sheet(&sheet).is_ok());
        let sheet=json!({"manualRd":{"Geral":3,"Fogo":2},"equipment":[{"equipped":true,"itemType":"Armadura","material":"Adamante","heavy":false}],"activeConditionNames":["Petrificado"]});
        assert_eq!(damage_reduction(&sheet,"Geral"),13);
        assert_eq!(damage_reduction(&sheet,"Fogo"),15);
        assert_eq!(golem_immunity(&json!({"ancestry":"Golem","golemElement":"Ar"})),Some("Eletricidade"));
    }

    #[test]
    fn map_coordinates_validate_and_snap(){
        let map=json!({"width":1000,"height":600,"cellPx":64,"snap":true});
        assert_eq!(map_point(&json!({"x":70.0,"y":130.0}),&map).unwrap(),(64.0,128.0));
        assert!(map_point(&json!({"x":1001.0,"y":3.0}),&map).is_err());
        assert!(map_point(&json!({"x":null,"y":3.0}),&map).is_err());
    }

    #[test]
    fn image_import_checks_format_and_dimensions(){
        let dir=std::env::temp_dir().join(format!("tormenta-image-test-{}",Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let valid=dir.join("valid.png");
        image::RgbaImage::new(2,3).save(&valid).unwrap();
        assert_eq!(inspect_image(&valid,"map").unwrap(),("png",2,3));
        let invalid=dir.join("invalid.png");fs::write(&invalid,b"not an image").unwrap();
        assert!(inspect_image(&invalid,"map").is_err());
        let wide=dir.join("wide.png");image::RgbaImage::new(2049,1).save(&wide).unwrap();
        assert!(inspect_image(&wide,"token").is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn audio_signature_rejects_fake_files(){
        assert_eq!(audio_extension(b"ID3\0\0\0\0\0\0\0"),None);
        assert_eq!(audio_extension(b"plain text disguised as audio"),None);
        assert_eq!(audio_extension(&[0xff,0xfb,0x90,0x00,0,0,0,0]),Some("mp3"));
        let mut wav=b"RIFF\0\0\0\0WAVEfmt ".to_vec();wav.resize(40,0);wav.extend_from_slice(b"data\0\0\0\0");
        assert_eq!(audio_extension(&wav),Some("wav"));
        assert_eq!(decode_asset_name("M%C3%BAsica%20de%20A%C3%A7%C3%A3o"),Some("Música de Ação".into()));
        assert_eq!(decode_asset_name("%GG"),None);
    }

    #[test]
    fn legacy_migration_keeps_one_copy(){
        let mut db=Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE meta(key TEXT PRIMARY KEY,value TEXT);CREATE TABLE local_characters(id TEXT PRIMARY KEY,name TEXT,data TEXT,updated_at INTEGER);").unwrap();
        let raw=json!({"name":"Antiga","level":1}).to_string();
        migrate_legacy_db(&mut db,Some(raw.clone())).unwrap();
        migrate_legacy_db(&mut db,Some(raw)).unwrap();
        assert_eq!(list_local(&db).unwrap().len(),1);
        assert_eq!(list_local(&db).unwrap()[0].data["name"],"Antiga");
    }

    #[test]
    fn upgrades_existing_chat_without_losing_messages(){
        let path=std::env::temp_dir().join(format!("tormenta-chat-test-{}.sqlite",Uuid::new_v4()));
        let old=Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE chat(id INTEGER PRIMARY KEY,room_id TEXT,author TEXT,text TEXT,created_at INTEGER);INSERT INTO chat VALUES (1,'mesa','Ana','Mensagem antiga',1);").unwrap();
        drop(old);
        let upgraded=open_db(&path).unwrap();
        assert_eq!(history(&upgraded,"mesa","jogador",false).unwrap()[0]["text"],"Mensagem antiga");
        assert_eq!(history(&upgraded,"mesa","jogador",false).unwrap()[0]["kind"],"message");
        drop(upgraded);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn legacy_secret_rolls_keep_their_original_owner_only_visibility(){
        let path=std::env::temp_dir().join(format!("tormenta-secret-test-{}.sqlite",Uuid::new_v4()));
        let old=Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE chat(id INTEGER PRIMARY KEY,room_id TEXT,author TEXT,text TEXT,created_at INTEGER,audience TEXT,owner_token TEXT);INSERT INTO chat VALUES (1,'mesa','Ana','1d20 = 17',1,'secret','ana');").unwrap();
        drop(old);
        let upgraded=open_db(&path).unwrap();
        assert_eq!(history(&upgraded,"mesa","ana",false).unwrap()[0]["audience"],"self");
        assert!(history(&upgraded,"mesa","mestre",true).unwrap().is_empty());
        drop(upgraded);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn existing_maps_become_tactical_scenes(){
        let path=std::env::temp_dir().join(format!("tormenta-scene-test-{}.sqlite",Uuid::new_v4()));
        let old=Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE room_maps(id TEXT PRIMARY KEY,room_id TEXT NOT NULL,name TEXT NOT NULL,file_name TEXT NOT NULL,width INTEGER NOT NULL,height INTEGER NOT NULL,grid_visible INTEGER NOT NULL DEFAULT 0,snap INTEGER NOT NULL DEFAULT 0,cell_px INTEGER NOT NULL DEFAULT 64,metres_per_cell REAL NOT NULL DEFAULT 1.5);INSERT INTO room_maps(id,room_id,name,file_name,width,height) VALUES ('mapa','mesa','Mapa antigo','mapa.png',800,600);").unwrap();
        drop(old);
        let upgraded=open_db(&path).unwrap();
        assert_eq!(maps(&upgraded,"mesa").unwrap()[0]["kind"],"tactical");
        drop(upgraded);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn combat_groups_threats_and_persists_turns(){
        let path=std::env::temp_dir().join(format!("tormenta-combat-test-{}.sqlite",Uuid::new_v4()));
        let db=open_db(&path).unwrap();
        db.execute_batch("INSERT INTO rooms(id,name,created_at) VALUES ('room','Teste',1);
            INSERT INTO room_maps(id,room_id,name,file_name,width,height) VALUES ('map','room','Mapa','map.png',800,600);
            INSERT INTO participants(id,room_id,name,token,approved) VALUES ('ana','room','Ana','ana-token',1);
            INSERT INTO room_characters(id,room_id,owner_token,data) VALUES ('sheet','room','ana-token','{\"name\":\"Heroína\"}');
            INSERT INTO room_tokens(id,room_id,map_id,sheet_id,name,x,y,size) VALUES ('hero','room','map','sheet','Heroína',20,20,64);
            INSERT INTO room_tokens(id,room_id,map_id,name,x,y,size) VALUES ('goblin-1','room','map','Goblin',80,20,64),('goblin-2','room','map','Goblin',140,20,64);
            INSERT INTO room_threats(id,room_id,data) VALUES ('goblin','room','{\"initiative\":3}');
            INSERT INTO room_threat_instances(token_id,room_id,threat_id,current_hp,current_mp) VALUES ('goblin-1','room','goblin',6,0),('goblin-2','room','goblin',6,0);").unwrap();
        assert!(new_combat(&db,"room","map",&json!({})).is_err());
        let combat=new_combat(&db,"room","map",&json!({"hero":18,"threats":12})).unwrap();
        assert_eq!(combat["entries"].as_array().unwrap().len(),2);
        assert_eq!(combat["entries"][0]["ownerParticipantId"],"ana");
        assert_eq!(combat["entries"][1]["tokenIds"],json!(["goblin-1","goblin-2"]));
        save_combat(&db,"room","map",&combat).unwrap();
        let loaded=combat_state(&db,"room","map").unwrap().unwrap();
        assert_eq!(loaded["revision"],1);
        assert!(save_combat(&db,"room","map",&combat).is_err());
        let mut moved=combat.clone();moved["turn"]=json!(1);
        update_combat(&db,"room","map",&moved,1).unwrap();
        assert!(update_combat(&db,"room","map",&combat,1).is_err());
        assert_eq!(combat_state(&db,"room","map").unwrap().unwrap()["turn"],1);
        db.execute("UPDATE room_threat_instances SET conditions=?1 WHERE token_id='goblin-1'",[json!([{"name":"Atordoado","unit":"rounds","remaining":2},{"name":"Agarrado","unit":"manual"}]).to_string()]).unwrap();
        let public_token=token_record(&db,"room","goblin-1").unwrap().unwrap();
        assert_eq!(public_token["conditions"],json!(["Atordoado","Agarrado"]));
        assert!(public_token.get("currentHp").is_none());
        assert!(public_token.get("currentMp").is_none());
        tick_threat_conditions(&db,"room","map").unwrap();
        assert_eq!(threat_instances(&db,"room","map").unwrap()[0]["conditions"][0]["remaining"],1);
        tick_threat_conditions(&db,"room","map").unwrap();
        assert_eq!(threat_instances(&db,"room","map").unwrap()[0]["conditions"],json!([{"name":"Agarrado","unit":"manual"}]));
        drop(db);std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn private_notes_are_not_in_player_document_lists(){
        let path=std::env::temp_dir().join(format!("tormenta-docs-test-{}.sqlite",Uuid::new_v4()));
        let db=open_db(&path).unwrap();
        db.execute_batch("INSERT INTO rooms(id,name,created_at) VALUES ('room','Teste',1);
            INSERT INTO room_documents VALUES ('private','room','note','Segredo','Não revelar','private');
            INSERT INTO room_documents VALUES ('public','room','handout','Carta','Texto da carta','public');").unwrap();
        assert_eq!(documents(&db,"room",true).unwrap().len(),2);
        let player=documents(&db,"room",false).unwrap();
        assert_eq!(player.len(),1);assert_eq!(player[0]["title"],"Carta");
        drop(db);std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn upgrades_room_cards_without_losing_rooms(){
        let path=std::env::temp_dir().join(format!("tormenta-room-card-test-{}.sqlite",Uuid::new_v4()));
        let old=Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE rooms(id TEXT PRIMARY KEY,name TEXT NOT NULL,created_at INTEGER NOT NULL);INSERT INTO rooms VALUES ('mesa','Campanha antiga',42);").unwrap();
        drop(old);
        let upgraded=open_db(&path).unwrap();
        let row:(String,i64,Option<String>)=upgraded.query_row("SELECT name,last_opened_at,cover_file_name FROM rooms WHERE id='mesa'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!(row,("Campanha antiga".into(),42,None));
        drop(upgraded);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn deleting_room_keeps_other_rooms_and_local_sheets(){
        let path=std::env::temp_dir().join(format!("tormenta-delete-test-{}.sqlite",Uuid::new_v4()));
        let mut db=open_db(&path).unwrap();
        db.execute_batch("INSERT INTO rooms(id,name,created_at) VALUES ('old','Antiga',1),('keep','Atual',2);
            INSERT INTO local_characters VALUES ('local','Herói','{}',1);
            INSERT INTO room_maps(id,room_id,name,file_name,width,height) VALUES ('map','old','Mapa','map.png',100,100),('other-map','keep','Outro','other.png',100,100);
            INSERT INTO room_token_art(id,room_id,name,file_name,width,height) VALUES ('art','old','Arte','art.png',32,32);
            INSERT INTO room_audio(id,room_id,owner_token,name,file_name) VALUES ('song','old','','Tema','song.mp3');
            INSERT INTO room_tokens(id,room_id,map_id,name,x,y,size) VALUES ('token','old','map','Peça',1,1,32);
            INSERT INTO room_playlist VALUES ('old',0,'song');INSERT INTO room_playback(room_id,audio_id) VALUES ('old','song');
            INSERT INTO participants(id,room_id,name,token) VALUES ('player','old','Ana','token');
            INSERT INTO room_characters(id,room_id,owner_token,data) VALUES ('sheet','old','token','{}');
            INSERT INTO chat(room_id,author,text,created_at) VALUES ('old','Ana','Oi',1);").unwrap();
        let files=delete_room_db(&mut db,"old").unwrap();
        assert_eq!(files.len(),3);
        for table in ["rooms","room_maps","room_token_art","room_audio","room_tokens","room_playlist","room_playback","participants","room_characters","chat"]{
            let column=if table=="rooms"{"id"}else{"room_id"};
            let count:i64=db.query_row(&format!("SELECT COUNT(*) FROM {table} WHERE {column}='old'"),[],|r|r.get(0)).unwrap();assert_eq!(count,0,"{table}");
        }
        assert_eq!(db.query_row("SELECT COUNT(*) FROM rooms WHERE id='keep'",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(db.query_row("SELECT COUNT(*) FROM local_characters",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert!(delete_room_db(&mut db,"old").is_err());
        drop(db);fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn upload_validates_files_and_keeps_private_assets_private(){
        let root=std::env::temp_dir().join(format!("tormenta-upload-test-{}",Uuid::new_v4()));fs::create_dir(&root).unwrap();
        let path=root.join("test.sqlite");let database=open_db(&path).unwrap();
        database.execute("INSERT INTO rooms(id,name,created_at) VALUES ('room','Teste',1)",[]).unwrap();
        database.execute("INSERT INTO participants(id,room_id,name,token,approved) VALUES ('ana','room','Ana','ana-token',1)",[]).unwrap();drop(database);
        let (stop,_)=watch::channel(false);
        let server=Arc::new(RoomServer{id:"room".into(),name:"Teste".into(),code:"CODE".into(),master_token:"master-token".into(),db:path,assets:root.join("assets"),live:Mutex::new(LiveRoom::default()),stop});
        fs::create_dir(&server.assets).unwrap();
        let mut headers=HeaderMap::new();headers.insert(header::AUTHORIZATION,"Bearer ana-token".parse().unwrap());headers.insert("x-asset-kind","art".parse().unwrap());headers.insert("x-asset-name","A%C3%A7%C3%A3o".parse().unwrap());
        assert_eq!(upload_asset(State(server.clone()),headers.clone(),Body::from("not an image")).await.status(),StatusCode::BAD_REQUEST);
        assert_eq!(upload_asset(State(server.clone()),headers.clone(),Body::from(vec![0;3_000_001])).await.status(),StatusCode::PAYLOAD_TOO_LARGE);
        assert!(room_art(&db(&server).unwrap(),"room",None).unwrap().is_empty());
        let mut png=Cursor::new(Vec::new());image::DynamicImage::new_rgba8(2,3).write_to(&mut png,ImageFormat::Png).unwrap();
        let response=upload_asset(State(server.clone()),headers.clone(),Body::from(png.into_inner())).await;
        assert_eq!(response.status(),StatusCode::CREATED);
        let body=to_bytes(response.into_body(),1000).await.unwrap();let item:Value=serde_json::from_slice(&body).unwrap();
        assert_eq!(item["name"],"Ação");assert_eq!(item["ownerParticipantId"],"ana");assert!(item.get("ownerToken").is_none());
        let id=item["id"].as_str().unwrap().to_string();
        assert_eq!(asset(State(server.clone()),Path(id.clone()),headers.clone()).await.status(),StatusCode::OK);
        let mut stranger=HeaderMap::new();stranger.insert(header::AUTHORIZATION,"Bearer stranger".parse().unwrap());
        assert_eq!(asset(State(server.clone()),Path(id.clone()),stranger).await.status(),StatusCode::FORBIDDEN);
        headers.insert("x-asset-kind","audio".parse().unwrap());
        assert_eq!(upload_asset(State(server.clone()),headers,Body::from("ID3\0\0\0\0\0\0\0")).await.status(),StatusCode::BAD_REQUEST);
        drop(server);fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn two_clients_permissions_conflict_and_damage(){
        let path=std::env::temp_dir().join(format!("tormenta-mesa-test-{}.sqlite",Uuid::new_v4()));
        let db=open_db(&path).unwrap();
        let room_id=Uuid::new_v4().to_string();
        db.execute("INSERT INTO rooms(id,name,created_at) VALUES (?1,'Teste',?2)",params![room_id,now()]).unwrap();
        drop(db);
        let (stop, _)=watch::channel(false);
        let server=Arc::new(RoomServer{id:room_id.clone(),name:"Teste".into(),code:"CODIGO-SECRETO".into(),master_token:"mestre-secreto".into(),db:path,assets:std::env::temp_dir(),live:Mutex::new(LiveRoom::default()),stop});
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address=format!("ws://127.0.0.1:{}/ws",listener.local_addr().unwrap().port());
        let service=Router::new().route("/ws",get(websocket)).with_state(server.clone());
        let task=tokio::spawn(async move {let _=axum::serve(listener,service).await;});

        let (mut wrong,_)=connect_async(&address).await.unwrap();
        send_client(&mut wrong,json!({"type":"hello","name":"Intruso","code":"ERRADO"})).await;
        assert_eq!(receive(&mut wrong,"error").await["message"],"Código inválido");

        let (mut master,_)=connect_async(&address).await.unwrap();
        send_client(&mut master,json!({"type":"hello","token":"mestre-secreto"})).await;
        assert_eq!(receive(&mut master,"welcome").await["role"],"master");

        let (mut player,_)=connect_async(&address).await.unwrap();
        send_client(&mut player,json!({"type":"hello","name":"Ana","code":"CODIGO-SECRETO"})).await;
        let joined=receive(&mut player,"welcome").await;
        assert_eq!(joined["role"],"pending");
        send_client(&mut player,json!({"type":"chat","text":"antes da aprovação"})).await;
        assert_eq!(receive(&mut player,"error").await["message"],"Aguarde aprovação do mestre");
        let token=joined["token"].as_str().unwrap().to_string();
        let roster=roster_with(&mut master,"Ana").await;
        let participant=roster["participants"][0]["id"].as_str().unwrap();
        send_client(&mut master,json!({"type":"approve","participantId":participant})).await;
        receive(&mut player,"approved").await;
        send_client(&mut player,json!({"type":"chat","text":"Olá, mesa"})).await;
        assert_eq!(receive(&mut master,"chat").await["text"],"Olá, mesa");

        let sheet=json!({"name":"Heroína","level":1,"baseAttributes":[0,0,0,0,0,0],"currentHp":20,"temporaryHp":4});
        send_client(&mut player,json!({"type":"offer_sheet","data":sheet})).await;
        let offer=receive(&mut master,"sheet_offer").await;
        assert_eq!(offer["data"]["name"],"Heroína");
        send_client(&mut master,json!({"type":"accept_sheet","participantId":participant})).await;
        let assigned=receive(&mut player,"sheet").await;
        let sheet_id=assigned["id"].as_str().unwrap().to_string();
        assert_eq!(assigned["owner"],token);

        let (mut other,_)=connect_async(&address).await.unwrap();
        send_client(&mut other,json!({"type":"hello","name":"Beto","code":"CODIGO-SECRETO"})).await;
        let other_join=receive(&mut other,"welcome").await;
        let other_token=other_join["token"].as_str().unwrap().to_string();
        assert_eq!(other_join["characters"].as_array().unwrap().len(),0);
        let all=roster_with(&mut master,"Beto").await;
        let other_id=all["participants"].as_array().unwrap().iter().find(|p|p["name"]=="Beto").unwrap()["id"].as_str().unwrap();
        send_client(&mut master,json!({"type":"approve","participantId":other_id})).await;
        let approved=receive(&mut other,"approved").await;
        assert_eq!(approved["characters"].as_array().unwrap().len(),0);
        assert_eq!(approved["chat"][0]["text"],"Olá, mesa");
        let map_id=Uuid::new_v4().to_string();
        let art_id=Uuid::new_v4().to_string();
        let db=open_db(&server.db).unwrap();
        db.execute("INSERT INTO room_maps(id,room_id,name,file_name,width,height) VALUES (?1,?2,'Mapa','map.png',800,600)",params![map_id,server.id]).unwrap();
        db.execute("INSERT INTO room_token_art(id,room_id,name,file_name,width,height) VALUES (?1,?2,'Marcador','token.png',64,64)",params![art_id,server.id]).unwrap();
        db.execute("INSERT INTO room_documents VALUES ('secret-letter',?1,'handout','Carta secreta','Pista revelada','private')",params![server.id]).unwrap();
        db.execute("UPDATE rooms SET active_map_id=?1 WHERE id=?2",params![map_id,server.id]).unwrap();
        drop(db);
        let text_mark=json!({"type":"mark_save","kind":"text","visibility":"public","data":{"x":40,"y":50,"text":"Rota","color":"#ffffff","font":"Arial","size":20,"bold":false,"italic":false}});
        send_client(&mut player,text_mark).await;
        let text_id=receive(&mut other,"marks").await["items"][0]["id"].as_str().unwrap().to_string();
        let received=receive(&mut master,"marks").await;
        assert_eq!(received["items"][0]["ownerParticipantId"],participant);
        assert!(received.to_string().find(&token).is_none());
        send_client(&mut other,json!({"type":"mark_delete","id":text_id,"baseRevision":1})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Você só pode apagar suas marcações");
        send_client(&mut player,json!({"type":"mark_save","id":text_id,"baseRevision":1,"kind":"text","visibility":"public","data":{"x":40,"y":50,"text":"Rota segura","color":"#ffffff","font":"Arial","size":20,"bold":false,"italic":false}})).await;
        assert_eq!(receive(&mut other,"marks").await["items"][0]["revision"],2);
        send_client(&mut player,json!({"type":"mark_save","kind":"pin","visibility":"private","data":{"x":10,"y":10,"title":"Segredo","note":"Plano"}})).await;
        assert_eq!(receive(&mut player,"error").await["message"],"Apenas o mestre pode colocar alfinetes");
        send_client(&mut master,json!({"type":"mark_save","kind":"pin","visibility":"private","data":{"x":10,"y":10,"title":"Segredo","note":"Plano"}})).await;
        assert_eq!(receive(&mut other,"marks").await["items"].as_array().unwrap().len(),1);
        assert_eq!(marks(&open_db(&server.db).unwrap(),&server.id,&map_id,false).unwrap().len(),1);
        assert_eq!(marks(&open_db(&server.db).unwrap(),&server.id,&map_id,true).unwrap().len(),2);
        assert_eq!(asset(State(server.clone()),Path(map_id.clone()),HeaderMap::new()).await.status(),StatusCode::FORBIDDEN);
        let mut headers=HeaderMap::new();headers.insert(header::AUTHORIZATION,format!("Bearer {token}").parse().unwrap());
        assert_eq!(asset(State(server.clone()),Path(map_id.clone()),headers.clone()).await.status(),StatusCode::NOT_FOUND);
        assert_eq!(asset(State(server.clone()),Path(art_id.clone()),headers.clone()).await.status(),StatusCode::FORBIDDEN);
        send_client(&mut other,json!({"type":"place_token","sheetId":sheet_id,"x":30,"y":30})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Jogadores só podem colocar artes próprias");
        send_client(&mut master,json!({"type":"place_token","sheetId":sheet_id,"x":70,"y":130})).await;
        let placed=receive(&mut player,"token_upsert").await["token"].clone();
        assert_eq!((placed["x"].as_f64(),placed["y"].as_f64()),(Some(70.0),Some(130.0)));
        let placed_id=placed["id"].as_str().unwrap();
        let mut initiatives=serde_json::Map::new();initiatives.insert(placed_id.to_string(),json!(15));
        send_client(&mut player,json!({"type":"fog_set","action":"cover_all","baseRevision":0})).await;
        assert_eq!(receive(&mut player,"error").await["message"],"Ação não permitida");
        send_client(&mut master,json!({"type":"fog_set","action":"cover_all","baseRevision":0})).await;
        assert_eq!(receive(&mut player,"fog").await["fog"]["enabled"],true);
        send_client(&mut master,json!({"type":"fog_set","action":"polygon","mode":"reveal","points":[{"x":10,"y":10},{"x":80,"y":10},{"x":40,"y":80}],"baseRevision":1})).await;
        assert_eq!(receive(&mut player,"fog").await["fog"]["ops"][0]["shape"],"polygon");
        send_client(&mut player,json!({"type":"show_handout","id":"secret-letter"})).await;
        assert_eq!(receive(&mut player,"error").await["message"],"Ação não permitida");
        send_client(&mut master,json!({"type":"show_handout","id":"secret-letter"})).await;
        assert_eq!(receive(&mut player,"chat").await["details"]["body"],"Pista revelada");
        assert_eq!(receive(&mut master,"chat").await["kind"],"handout");
        assert_eq!(receive(&mut player,"handout_open").await["handout"]["title"],"Carta secreta");
        send_client(&mut player,json!({"type":"start_combat","initiatives":initiatives})).await;
        assert_eq!(receive(&mut player,"error").await["message"],"Ação não permitida");
        send_client(&mut master,json!({"type":"start_combat","initiatives":initiatives})).await;
        let combat=receive(&mut player,"combat").await;
        assert_eq!(combat["combat"]["entries"][0]["initiative"],15);
        send_client(&mut other,json!({"type":"combat_next","baseRevision":1})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Aguarde seu turno");
        send_client(&mut player,json!({"type":"combat_next","baseRevision":1})).await;
        assert_eq!(receive(&mut player,"combat").await["combat"]["round"],2);
        send_client(&mut master,json!({"type":"end_combat"})).await;
        assert!(receive(&mut player,"combat").await["combat"].is_null());
        send_client(&mut other,json!({"type":"move_token","id":placed_id,"baseRevision":1,"x":80,"y":140})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Sem permissão para mover este token");
        send_client(&mut player,json!({"type":"move_token","id":placed_id,"baseRevision":1,"x":80,"y":140})).await;
        assert_eq!(receive(&mut player,"token_upsert").await["token"]["revision"],2);
        receive(&mut other,"token_upsert").await;
        send_client(&mut player,json!({"type":"move_token","id":placed_id,"baseRevision":1,"x":90,"y":150})).await;
        assert_eq!(receive(&mut player,"token_conflict").await["token"]["revision"],2);
        send_client(&mut master,json!({"type":"place_token","artId":art_id,"x":200,"y":200})).await;
        let free=receive(&mut other,"token_upsert").await["token"].clone();
        assert_eq!(asset(State(server.clone()),Path(art_id.clone()),headers).await.status(),StatusCode::NOT_FOUND);
        let free_id=free["id"].as_str().unwrap();
        send_client(&mut master,json!({"type":"update_token","id":free_id,"name":"Marcador","size":64,"ownerParticipantId":other_id})).await;
        assert_eq!(receive(&mut other,"token_upsert").await["token"]["ownerParticipantId"],other_id);
        send_client(&mut other,json!({"type":"move_token","id":free_id,"baseRevision":2,"x":220,"y":220})).await;
        assert_eq!(receive(&mut other,"token_upsert").await["token"]["revision"],3);
        send_client(&mut other,json!({"type":"remove_token","id":free_id})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Sem permissão para retirar este token");
        let personal_art=Uuid::new_v4().to_string();
        let db=open_db(&server.db).unwrap();
        db.execute("INSERT INTO room_token_art(id,room_id,name,file_name,width,height,owner_token) VALUES (?1,?2,'Arte da Ana','ana.png',64,64,?3)",params![personal_art,server.id,token]).unwrap();
        let metadata=room_art(&db,&server.id,None).unwrap().into_iter().find(|a|a["id"]==personal_art).unwrap();
        assert_eq!(metadata["ownerParticipantId"],participant);
        assert!(metadata.get("ownerToken").is_none());
        drop(db);
        send_client(&mut other,json!({"type":"place_token","artId":personal_art,"x":25,"y":25})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Imagem indisponível");
        send_client(&mut player,json!({"type":"place_token","artId":personal_art,"x":25,"y":25})).await;
        let owned=loop{let item=receive(&mut player,"token_upsert").await["token"].clone();if item["artId"]==personal_art{break item;}};
        assert_eq!(owned["ownerParticipantId"],participant);
        let owned_id=owned["id"].as_str().unwrap();
        send_client(&mut other,json!({"type":"remove_token","id":owned_id})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Sem permissão para retirar este token");
        send_client(&mut player,json!({"type":"update_token","id":owned_id,"name":"Nova arte","size":80,"ownerParticipantId":participant})).await;
        let changed=loop{let item=receive(&mut player,"token_upsert").await["token"].clone();if item["id"]==owned_id{break item;}};
        assert_eq!(changed["name"],"Nova arte");
        send_client(&mut player,json!({"type":"remove_token","id":owned_id})).await;
        assert_eq!(receive(&mut player,"token_removed").await["id"],owned_id);
        send_client(&mut player,json!({"type":"ping","mapId":map_id,"x":100,"y":100})).await;
        assert_eq!(receive(&mut other,"ping").await["x"],100.0);
        let audio_id=Uuid::new_v4().to_string();
        let db=open_db(&server.db).unwrap();
        db.execute("INSERT INTO room_audio(id,room_id,owner_token,name,file_name) VALUES (?1,?2,?3,'Tema','tema.mp3')",params![audio_id,server.id,token]).unwrap();
        drop(db);
        let mut other_headers=HeaderMap::new();other_headers.insert(header::AUTHORIZATION,format!("Bearer {other_token}").parse().unwrap());
        assert_eq!(asset(State(server.clone()),Path(audio_id.clone()),other_headers.clone()).await.status(),StatusCode::FORBIDDEN);
        send_client(&mut other,json!({"type":"set_playlist","ids":[audio_id]})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Ação não permitida");
        send_client(&mut master,json!({"type":"set_playlist","ids":[audio_id]})).await;
        assert_eq!(receive(&mut other,"playlist").await["tracks"][0]["name"],"Tema");
        assert_eq!(asset(State(server.clone()),Path(audio_id.clone()),other_headers).await.status(),StatusCode::NOT_FOUND);
        send_client(&mut master,json!({"type":"playback_control","action":"play","audioId":audio_id})).await;
        assert_eq!(receive(&mut other,"playback").await["playback"]["audioId"],audio_id);
        send_client(&mut player,json!({"type":"roll_chat","formula":"1d20+2","audience":"gm"})).await;
        let gm_roll=receive(&mut master,"chat").await;
        assert_eq!(gm_roll["audience"],"gm");
        assert_eq!(gm_roll["kind"],"roll");
        assert_eq!(gm_roll["details"]["formula"],"1d20+2");
        receive(&mut player,"chat").await;
        let db=open_db(&server.db).unwrap();
        assert!(!history(&db,&server.id,&other_token,false).unwrap().iter().any(|line|line["audience"]=="gm"));
        send_client(&mut player,json!({"type":"roll_chat","formula":"1d6","audience":"secret"})).await;
        assert_eq!(receive(&mut master,"chat").await["audience"],"secret");
        assert_eq!(receive(&mut player,"chat").await["kind"],"system");
        assert!(history(&db,&server.id,&server.master_token,true).unwrap().iter().any(|line|line["audience"]=="secret"));
        assert!(!history(&db,&server.id,&token,false).unwrap().iter().any(|line|line["audience"]=="secret"));
        drop(db);
        send_client(&mut player,json!({"type":"measure","mapId":map_id,"x1":0.1,"y1":0.2,"x2":0.4,"y2":0.5,"metres":3.2})).await;
        assert_eq!(receive(&mut other,"measure").await["metres"],7.0);
        send_client(&mut player,json!({"type":"measure","mapId":map_id,"x1":-1,"y1":0.2,"x2":0.4,"y2":0.5,"metres":3.2})).await;
        assert_eq!(receive(&mut player,"error").await["message"],"Medição inválida");
        let created=json!({"name":"Criado pelo mestre","level":1,"currentHp":8});
        send_client(&mut master,json!({"type":"assign_sheet","participantId":other_id,"data":created})).await;
        assert_eq!(receive(&mut other,"sheet").await["data"]["name"],"Criado pelo mestre");
        send_client(&mut other,json!({"type":"save_sheet","id":sheet_id,"baseRevision":1,"data":sheet})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Sem permissão para editar esta ficha");

        let mut forged=sheet.clone();forged["baseAttributes"]=json!([4,4,4,4,4,4]);
        send_client(&mut player,json!({"type":"save_sheet","id":sheet_id,"baseRevision":1,"data":forged})).await;
        assert_eq!(receive(&mut player,"error").await["message"],"A compra de atributos excede os 10 pontos disponíveis");
        // A tentativa rejeitada não pode alterar a ficha nem consumir sua revisão.
        send_client(&mut player,json!({"type":"save_sheet","id":sheet_id,"baseRevision":1,"data":sheet})).await;
        assert_eq!(receive(&mut player,"sheet").await["revision"],2);
        send_client(&mut master,json!({"type":"save_sheet","id":sheet_id,"baseRevision":1,"data":sheet})).await;
        assert_eq!(receive(&mut master,"conflict").await["sheet"]["revision"],2);

        let mut edited=sheet.clone();edited["proficiencyAdds"]=json!(["Armaduras pesadas"]);
        send_client(&mut player,json!({"type":"save_sheet","id":sheet_id,"baseRevision":2,"data":edited})).await;
        assert_eq!(receive(&mut player,"error").await["message"],"O mestre não autorizou mudanças nas proficiências");
        send_client(&mut master,json!({"type":"set_proficiency_permission","sheetId":sheet_id,"allowed":true})).await;
        assert_eq!(receive(&mut player,"sheet_permission").await["allowed"],true);
        send_client(&mut player,json!({"type":"save_sheet","id":sheet_id,"baseRevision":2,"data":edited})).await;
        assert_eq!(receive(&mut player,"sheet").await["revision"],3);

        send_client(&mut master,json!({"type":"roll_damage","sheetId":sheet_id,"formula":"2d6+3"})).await;
        let roll=receive(&mut master,"roll_ready").await;
        send_client(&mut master,json!({"type":"apply_damage","id":roll["id"]})).await;
        let damaged=receive(&mut player,"sheet").await;
        assert_eq!(damaged["reason"],"damage");
        assert_eq!(damaged["data"]["temporaryHp"],0);
        assert_eq!(damaged["data"]["currentHp"].as_i64().unwrap(),24-roll["total"].as_i64().unwrap());
        assert_eq!(receive(&mut other,"chat").await["author"],"Mestre");

        let mut elemental=damaged["data"].clone();
        elemental["ancestry"]=json!("Golem");elemental["golemElement"]=json!("Fogo");elemental["maximumHp"]=json!(20);
        elemental["temporaryHp"]=json!(4);elemental["manualRd"]=json!({"Geral":2});
        send_client(&mut master,json!({"type":"save_sheet","id":sheet_id,"baseRevision":4,"data":elemental})).await;
        assert_eq!(receive(&mut player,"sheet").await["revision"],5);
        send_client(&mut master,json!({"type":"roll_damage","sheetId":sheet_id,"formula":"1d2+3","damageType":"Fogo","magical":true})).await;
        let fire=receive(&mut master,"roll_ready").await;
        send_client(&mut master,json!({"type":"apply_damage","id":fire["id"]})).await;
        let healed=receive(&mut player,"sheet").await;
        assert_eq!(healed["data"]["temporaryHp"],4);
        assert_eq!(healed["data"]["currentHp"].as_i64().unwrap(),(damaged["data"]["currentHp"].as_i64().unwrap()+fire["total"].as_i64().unwrap()/2).min(20));
        send_client(&mut master,json!({"type":"roll_damage","sheetId":sheet_id,"formula":"1d2+3","lifeLoss":true})).await;
        let loss=receive(&mut master,"roll_ready").await;
        send_client(&mut master,json!({"type":"apply_damage","id":loss["id"]})).await;
        let lost=receive(&mut player,"sheet").await;
        assert_eq!(lost["data"]["temporaryHp"],4);
        assert_eq!(lost["data"]["currentHp"].as_i64().unwrap(),healed["data"]["currentHp"].as_i64().unwrap()-loss["total"].as_i64().unwrap());
        send_client(&mut master,json!({"type":"roll_damage","sheetId":sheet_id,"formula":"1d2+3","damageType":"Frio"})).await;
        let cold=receive(&mut master,"roll_ready").await;
        send_client(&mut master,json!({"type":"apply_damage","id":cold["id"]})).await;
        let resisted=receive(&mut player,"sheet").await;
        assert_eq!(resisted["data"]["temporaryHp"].as_i64().unwrap(),4-(cold["total"].as_i64().unwrap()-2));
        assert_eq!(resisted["data"]["currentHp"],lost["data"]["currentHp"]);

        send_client(&mut master,json!({"type":"revoke","participantId":other_id})).await;
        assert_eq!(receive(&mut other,"revoked").await["type"],"revoked");
        let mut revoked_headers=HeaderMap::new();revoked_headers.insert(header::AUTHORIZATION,format!("Bearer {other_token}").parse().unwrap());
        assert_eq!(asset(State(server.clone()),Path(map_id.clone()),revoked_headers).await.status(),StatusCode::FORBIDDEN);
        send_client(&mut other,json!({"type":"chat","text":"Não autorizado"})).await;
        assert_eq!(receive(&mut other,"error").await["message"],"Aguarde aprovação do mestre");

        player.close(None).await.unwrap();
        let (mut rejoined,_)=connect_async(&address).await.unwrap();
        send_client(&mut rejoined,json!({"type":"hello","token":token})).await;
        let snapshot=receive(&mut rejoined,"welcome").await;
        assert_eq!(snapshot["role"],"player");
        assert_eq!(snapshot["characters"].as_array().unwrap().len(),1);
        assert_eq!(snapshot["playback"]["audioId"],audio_id);
        receive(&mut rejoined,"roster").await;
        server.stop.send_replace(true);
        let closed=tokio::time::timeout(std::time::Duration::from_secs(2),rejoined.next()).await.unwrap();
        assert!(!matches!(closed,Some(Ok(ClientMessage::Text(_)))));
        task.abort();
    }
}
