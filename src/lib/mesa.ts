import { get, writable } from 'svelte/store';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { getPersonalAsset, putPersonalAsset } from './personal-assets';

export type CharacterData = Record<string, any>;
export type LocalCharacter = { id: string; name: string; data: CharacterData; updatedAt:number };
export type RoomInfo = { id: string; name: string; coverId?:string|null; lastOpenedAt:number };
export type RoomCard = RoomInfo & {role:'master'|'player';coverUrl?:string;connection?:ConnectionInfo};
export type Participant = { id: string; name: string; approved: boolean };
export type RoomCharacter = { id: string; owner: string; revision: number; data: CharacterData; reason?: string; canEditProficiencies?: boolean };
export type RoomMap = { id:string; name:string; kind:'narrative'|'tactical'; width:number; height:number; gridVisible:boolean; snap:boolean; cellPx:number; metresPerCell:number; backdropColor:string|null };
export type MapPoint = {x:number;y:number};
export type RoomMark = {id:string;mapId:string;kind:'drawing'|'text'|'pin'|'measure';ownerParticipantId:string;author:string;visibility:'public'|'private';data:Record<string,any>;revision:number};
export type TokenArt = { id:string; name:string; width:number; height:number; ownerParticipantId?:string; ownerName?:string };
export type RoomAudio = { id:string; name:string; ownerParticipantId:string; ownerName:string };
export type Playback = { audioId:string|null; positionSeconds:number; startedAt:number; playing:boolean };
export type PlacedToken = { id:string; mapId?:string; artId:string|null; sheetId:string|null; name:string; ownerParticipantId:string|null; creatorParticipantId:string|null; x:number; y:number; size:number; revision:number; conditions?:string[] };
export type RoomThreat = {id?:string;name:string;nd:string;kind:string;size:string;role:'solo'|'minion'|'special';initiative:number;perception:number;defense:number;fortitude:number;reflex:number;will:number;maxHp:number;maxMp:number;movement:number;attackName:string;attackFormula:string;damageFormula:string;actions:string;immunities:string;resistances:string;equipment:string;treasure:string;notes:string};
export type ThreatCondition = {name:string;unit:'manual'|'rounds';remaining?:number};
export type ThreatInstance = {tokenId:string;threatId:string;name:string;currentHp:number;currentMp:number;conditions:ThreatCondition[];revision:number};
export type CombatEntry = {id:string;name:string;tokenIds:string[];initiative:number;ownerParticipantId:string|null};
export type CombatState = {mapId:string;round:number;turn:number;paused:boolean;entries:CombatEntry[];revision:number};
export type FogState = {mapId:string;enabled:boolean;ops:import('./fog').FogOp[];revision:number};
export type RoomDocument = {id?:string;kind:'note'|'npc'|'handout';title:string;body:string;visibility:'private'|'public'};
export type HandoutOpen = {id:string;title:string;body:string};
export type MasterAssets = { maps:RoomMap[]; tokenArt:TokenArt[]; threats:RoomThreat[]; documents:RoomDocument[]; activeMap:RoomMap|null };
export type ChatLine = { id?: number; kind?: 'message'|'roll'|'action'|'system'|'handout'|'whisper'; author: string; text: string; at: number; audience?: 'all' | 'gm' | 'secret' | 'self'; details?: { formula?:string; rolls?:number[]; total?:number; label?:string; id?:string; title?:string; body?:string } };
export type DiceReveal = {id:number;formula:string;total:number;rolls:number[]};
type Measurement = { token: string; author: string; mapId?:string; x1: number; y1: number; x2: number; y2: number; shape?:'line'|'corners'; metres: number };
type Roll = { id: string; sheetId: string; formula: string; rolls: number[]; total: number; damageType?: string; magical?: boolean; lifeLoss?: boolean };
type Offer = { participantId: string; participantName: string; data: CharacterData };
export type SheetChangeRequest = {id:string;sheetId:string;baseRevision:number;previous:CharacterData;data:CharacterData;participantName:string;fields:string[]};
type HostInfo = { id: string; name: string; address: string; port: number; code: string; token: string };
export type ConnectionInfo = { address: string; port: number; code: string; name: string; token?: string };
export type RoomState = {
  status: 'idle' | 'connecting' | 'pending' | 'ready' | 'offline';
  role: 'master' | 'player' | 'pending' | '';
  room: RoomInfo | null;
  invite: HostInfo | null;
  participants: Participant[];
  characters: RoomCharacter[];
  chat: ChatLine[];
  dice: DiceReveal|null;
  offers: Offer[];
  sheetRequests: SheetChangeRequest[];
  roll: Roll | null;
  conflict: RoomCharacter | null;
  lastSheet: RoomCharacter | null;
  measure: Measurement | null;
  activeMap: RoomMap | null;
  placedTokens: PlacedToken[];
  threatInstances: ThreatInstance[];
  combat: CombatState|null;
  fog: FogState|null;
  marks: RoomMark[];
  documents: RoomDocument[];
  handout: HandoutOpen|null;
  roomArt: TokenArt[];
  roomAudio: RoomAudio[];
  playlist: string[];
  playback: Playback;
  serverTimeOffset: number;
  ping: {mapId:string;x:number;y:number;author:string;serial:number}|null;
  token: string;
  participantId: string | null;
  message: string;
};

const empty: RoomState = {status:'idle',role:'',room:null,invite:null,participants:[],characters:[],chat:[],dice:null,offers:[],sheetRequests:[],roll:null,conflict:null,lastSheet:null,measure:null,activeMap:null,placedTokens:[],threatInstances:[],combat:null,fog:null,marks:[],documents:[],handout:null,roomArt:[],roomAudio:[],playlist:[],playback:{audioId:null,positionSeconds:0,startedAt:0,playing:false},serverTimeOffset:0,ping:null,token:'',participantId:null,message:''};
export const room = writable<RoomState>(empty);
export const masterAssets = writable<MasterAssets>({maps:[],tokenArt:[],threats:[],documents:[],activeMap:null});
export const localCopiesChanged = writable(0);
let socket: WebSocket | null = null;
let connection: ConnectionInfo | null = null;
let retry: ReturnType<typeof setTimeout> | null = null;
let attempts = 0;
let intentional = false;
let measureTimer: ReturnType<typeof setTimeout> | null = null;
let diceTimer: ReturnType<typeof setTimeout> | null = null;
const sessionKey = 'tormenta-vtt-room-session';
const historyKey='a-guilda-vtt-room-history';

export const desktopAvailable = isTauri();
export function rememberedRoom(): ConnectionInfo | null {
  try { return JSON.parse(localStorage.getItem(sessionKey) || 'null'); } catch { return null; }
}
export async function localCharacters(): Promise<LocalCharacter[]> { return invoke('list_characters'); }
export async function previousRooms(): Promise<RoomInfo[]> { return invoke('list_rooms'); }
export async function persistCharacter(id: string | undefined, data: CharacterData): Promise<LocalCharacter> { return invoke('save_character',{id,data}); }
function joinedRooms():RoomCard[]{try{return JSON.parse(localStorage.getItem(historyKey)||'[]')}catch{return[]}}
function saveJoinedRoom(info:RoomInfo,connection:ConnectionInfo){const item:RoomCard={...info,lastOpenedAt:Date.now()/1000,role:'player',connection};localStorage.setItem(historyKey,JSON.stringify([item,...joinedRooms().filter(r=>r.id!==item.id)].slice(0,20)));}
async function coverUrl(item:RoomCard):Promise<string|undefined>{
  try{
    if(item.role==='player'){const cached=await getPersonalAsset(`room-cover:${item.id}`);return cached?URL.createObjectURL(cached.file):undefined;}
    const cover=await invoke<{bytes:number[];mime:string}|null>('room_cover',{roomId:item.id});return cover?URL.createObjectURL(new Blob([new Uint8Array(cover.bytes)],{type:cover.mime})):undefined;
  }catch{return undefined}
}
export async function roomCards():Promise<RoomCard[]>{
  const hosted=(await previousRooms()).map(room=>({...room,role:'master' as const}));
  const cards=[...hosted,...joinedRooms().filter(joined=>!hosted.some(room=>room.id===joined.id))].sort((a,b)=>b.lastOpenedAt-a.lastOpenedAt);
  return Promise.all(cards.map(async card=>({...card,coverUrl:await coverUrl(card)})));
}

function connect(info: ConnectionInfo, invite: HostInfo | null = null) {
  if (retry) clearTimeout(retry);
  if (diceTimer) clearTimeout(diceTimer);
  intentional = false;
  connection = info;
  socket?.close();
  room.update(s => ({...s,status:'connecting',invite:invite??s.invite,message:''}));
  const ws = new WebSocket(`ws://${info.address}:${info.port}/ws`);
  let authenticated = false;
  socket = ws;
  ws.onopen = () => ws.send(JSON.stringify({type:'hello',code:info.code,token:info.token,name:info.name}));
  ws.onmessage = event => {
    let data: any;
    try { data=JSON.parse(event.data); } catch { return; }
    if (data.type==='welcome') {
      authenticated=true;
      attempts=0;
      connection={...info,token:data.token};
      localStorage.setItem(sessionKey,JSON.stringify(connection));
      if(data.role!=='master')saveJoinedRoom(data.room,connection);
      room.set({...get(room),status:data.role==='pending'?'pending':'ready',role:data.role,room:data.room,token:data.token,participantId:data.participantId||null,participants:data.participants,characters:data.characters,chat:data.chat,dice:null,offers:[],sheetRequests:data.sheetRequests||[],roll:null,conflict:null,lastSheet:null,activeMap:data.activeMap,placedTokens:data.placedTokens||[],threatInstances:data.threatInstances||[],combat:data.combat||null,fog:data.fog||null,marks:data.marks||[],documents:data.documents||[],handout:null,roomArt:data.roomArt||[],roomAudio:data.roomAudio||[],playlist:data.playlist||[],playback:data.playback||empty.playback,serverTimeOffset:Number(data.serverNowMs||Date.now())-Date.now(),ping:null,message:''});
      masterAssets.set(data.role==='master'?{maps:data.maps||[],tokenArt:data.tokenArt||[],threats:data.threats||[],documents:data.documents||[],activeMap:data.activeMap}:{maps:[],tokenArt:[],threats:[],documents:[],activeMap:null});
      if(data.role!=='master'&&data.room?.coverId)void assetBlobUrl(data.room.coverId).then(async url=>{const file=await fetch(url).then(r=>r.blob());URL.revokeObjectURL(url);await putPersonalAsset({id:`room-cover:${data.room.id}`,name:data.room.name,kind:'cover',file});}).catch(()=>{});
      if (data.role==='player') for (const sheet of data.characters as RoomCharacter[]) mirrorOwnedSheet(sheet);
    } else if (data.type==='approved') {
      room.update(s=>({...s,status:'ready',role:'player',characters:data.characters,chat:data.chat,activeMap:data.activeMap,placedTokens:data.placedTokens||[],combat:data.combat||null,fog:data.fog||null,marks:data.marks||[],documents:data.documents||[],roomArt:data.roomArt||[],roomAudio:data.roomAudio||[],playlist:data.playlist||[],playback:data.playback||empty.playback,serverTimeOffset:Number(data.serverNowMs||Date.now())-Date.now(),message:'Entrada aprovada pelo mestre.'}));
      for (const sheet of data.characters as RoomCharacter[]) mirrorOwnedSheet(sheet);
    } else if (data.type==='revoked') {
      room.update(s=>({...s,status:'offline',message:'O mestre encerrou seu acesso à mesa.'}));
      intentional=true; ws.close(); localStorage.removeItem(sessionKey);
    } else if (data.type==='roster') {
      room.update(s=>({...s,participants:data.participants}));
    } else if (data.type==='chat') {
      const dice=data.kind==='roll'&&Number.isFinite(data.details?.total)?{id:Number(data.id),formula:String(data.details.formula),total:Number(data.details.total),rolls:data.details.rolls as number[]}:null;
      room.update(s=>({...s,chat:[...s.chat,data].slice(-100),dice:dice||s.dice}));
      if(dice){if(diceTimer)clearTimeout(diceTimer);diceTimer=setTimeout(()=>room.update(s=>({...s,dice:s.dice?.id===dice.id?null:s.dice})),matchMedia('(prefers-reduced-motion: reduce)').matches?150:1000);}
    } else if (data.type==='measure') {
      if (measureTimer) clearTimeout(measureTimer);
      room.update(s=>({...s,measure:data}));
      measureTimer=setTimeout(()=>room.update(s=>({...s,measure:null})),4000);
    } else if (data.type==='map_changed') {
      room.update(s=>({...s,activeMap:data.activeMap,placedTokens:data.placedTokens||[],threatInstances:[],combat:data.combat||null,fog:data.fog||null,marks:[],measure:null}));
      if(get(room).role==='master') masterAssets.update(s=>({...s,activeMap:data.activeMap}));
    } else if (data.type==='threat_instances') {
      room.update(s=>({...s,threatInstances:data.items||[]}));
    } else if (data.type==='combat') {
      room.update(s=>({...s,combat:data.combat||null}));
    } else if (data.type==='fog') {
      room.update(s=>({...s,fog:data.fog||null}));
    } else if (data.type==='marks') {
      room.update(s=>({...s,marks:data.items||[]}));
    } else if (data.type==='documents') {
      room.update(s=>({...s,documents:data.items||[]}));
    } else if (data.type==='handout_open') {
      room.update(s=>({...s,handout:data.handout||null}));
    } else if (data.type==='token_upsert' && data.token) {
      room.update(s=>({...s,placedTokens:[...s.placedTokens.filter(t=>t.id!==data.token.id),data.token]}));
    } else if (data.type==='token_removed') {
      room.update(s=>({...s,placedTokens:s.placedTokens.filter(t=>t.id!==data.id)}));
    } else if (data.type==='token_conflict' && data.token) {
      room.update(s=>({...s,placedTokens:[...s.placedTokens.filter(t=>t.id!==data.token.id),data.token],message:'O token foi movido por outra pessoa; posição atualizada.'}));
    } else if (data.type==='ping') {
      room.update(s=>({...s,ping:{mapId:data.mapId,x:data.x,y:data.y,author:data.author,serial:Date.now()}}));
    } else if (data.type==='asset_added') {
      room.update(s=>data.kind==='art'?{...s,roomArt:[...s.roomArt.filter(a=>a.id!==data.item.id),data.item]}:{...s,roomAudio:[...s.roomAudio.filter(a=>a.id!==data.item.id),data.item]});
      if(data.kind==='art' && get(room).role==='master')masterAssets.update(s=>({...s,tokenArt:[...s.tokenArt.filter(a=>a.id!==data.item.id),data.item]}));
    } else if (data.type==='playlist') {
      room.update(s=>({...s,playlist:data.ids,playback:data.playback,serverTimeOffset:Number(data.serverNowMs||Date.now())-Date.now(),roomAudio:[...s.roomAudio.filter(a=>(s.role==='master'||a.ownerParticipantId===s.participantId)&&!data.tracks?.some((t:RoomAudio)=>t.id===a.id)),...(data.tracks||[])]}));
    } else if (data.type==='playback') {
      room.update(s=>({...s,playback:data.playback,serverTimeOffset:Number(data.serverNowMs||Date.now())-Date.now()}));
    } else if (data.type==='sheet') {
      const sheet=data as RoomCharacter;
      room.update(s=>({...s,characters:[...s.characters.filter(c=>c.id!==sheet.id),sheet],lastSheet:sheet,message:'Ficha atualizada.'}));
      if (get(room).role==='player' && sheet.owner===get(room).token) mirrorOwnedSheet(sheet);
    } else if (data.type==='sheet_permission') {
      room.update(s=>({...s,characters:s.characters.map(c=>c.id===data.id?{...c,canEditProficiencies:data.allowed}:c),message:data.allowed?'Edição de proficiências autorizada.':'Edição de proficiências revogada.'}));
    } else if (data.type==='sheet_requests') {
      room.update(s=>({...s,sheetRequests:data.requests}));
    } else if (data.type==='sheet_offer') {
      room.update(s=>({...s,offers:[...s.offers.filter(o=>o.participantId!==data.participantId),data]}));
    } else if (data.type==='offer_accepted') {
      room.update(s=>({...s,offers:s.offers.filter(o=>o.participantId!==data.participantId)}));
    } else if (data.type==='roll_ready') {
      room.update(s=>({...s,roll:data}));
    } else if (data.type==='conflict') {
      room.update(s=>({...s,conflict:data.sheet,message:'A ficha mudou na mesa. Seu rascunho foi mantido; compare antes de substituir.'}));
    } else if (data.type==='notice'||data.type==='error') {
      room.update(s=>({...s,message:data.message}));
      if (data.type==='error' && !authenticated) { intentional=true; ws.close(); }
    }
  };
  ws.onerror = () => room.update(s=>({...s,message:'Não foi possível conectar. Verifique IP, porta e código.'}));
  ws.onclose = () => {
    if (socket!==ws) return;
    room.update(s=>({...s,status:'offline'}));
    if (!intentional && connection && attempts<6) {
      attempts++;
      retry=setTimeout(()=>connect(connection!),Math.min(1000*2**attempts,15000));
    }
  };
}

async function mirrorOwnedSheet(sheet:RoomCharacter) {
  const key=`tormenta-vtt-local-copy:${sheet.id}`;
  try {
    const id=localStorage.getItem(key)||undefined;
    const previous=localStorage.getItem(`${key}:last-data`);
    if (id) {
      const existing=(await localCharacters()).find(c=>c.id===id);
      if (existing && JSON.stringify(existing.data)!==previous && JSON.stringify(existing.data)!==JSON.stringify(sheet.data)) {
        room.update(s=>({...s,message:'Sua cópia local tem alterações próprias e foi preservada. Abra a ficha da mesa para comparar.'}));
        return;
      }
    }
    const saved=await persistCharacter(id,sheet.data);
    localStorage.setItem(key,saved.id);
    localStorage.setItem(`${key}:last-data`,JSON.stringify(saved.data));
    localCopiesChanged.update(n=>n+1);
  } catch (error) { room.update(s=>({...s,message:`Ficha recebida, mas a cópia local falhou: ${error}`})); }
}

export async function hostRoom(name:string, roomId?:string) {
  if (!desktopAvailable) throw new Error('A Mesa requer o aplicativo desktop.');
  const host=await invoke<HostInfo>('start_room',{name,roomId:roomId||null});
  connect({address:'127.0.0.1',port:host.port,code:host.code,name:'Mestre',token:host.token},host);
}
export function joinRoom(address:string,port:number,code:string,name:string) {
  if (!desktopAvailable) throw new Error('A Mesa requer o aplicativo desktop.');
  if (!address.trim()||!Number.isInteger(port)||port<1||port>65535||!code.trim()||!name.trim()) throw new Error('Preencha IP, porta, código e nome.');
  connect({address:address.trim(),port,code:code.trim().toUpperCase(),name:name.trim()});
}
export function reconnectRoom(){const saved=rememberedRoom();if(saved)connect(saved);}
export function openRoomCard(card:RoomCard){if(card.role==='player'&&card.connection)connect(card.connection);}
export function sendRoom(message:Record<string,unknown>) {
  if(socket?.readyState!==WebSocket.OPEN) {room.update(s=>({...s,message:'Sem conexão com a mesa.'}));return false;}
  socket.send(JSON.stringify(message));return true;
}
export async function loadMasterAssets(roomId:string):Promise<MasterAssets>{
  const data=await invoke<MasterAssets>('list_master_assets',{roomId});
  masterAssets.set(data);
  room.update(state=>state.role==='master'&&state.room?.id===roomId?{...state,documents:data.documents}:state);
  return data;
}
export async function importRoomAsset(roomId:string,kind:'map'|'token',name:string){
  const result=await invoke<RoomMap|TokenArt>('import_asset',{roomId,kind,name});
  await loadMasterAssets(roomId);
  if(kind==='token')room.update(s=>({...s,roomArt:[...s.roomArt,result as TokenArt]}));
  return result;
}
export async function createRoomScene(roomId:string,name:string){
  const scene=await invoke<RoomMap>('create_scene',{roomId,name});
  await loadMasterAssets(roomId);
  return scene;
}
export async function saveRoomThreat(roomId:string,data:RoomThreat){
  const threat=await invoke<RoomThreat>('save_threat',{roomId,id:data.id||null,data});
  await loadMasterAssets(roomId);
  return threat;
}
export async function saveRoomDocument(roomId:string,data:RoomDocument){
  const document=await invoke<RoomDocument>('save_document',{roomId,...data,id:data.id||null});
  await loadMasterAssets(roomId);
  return document;
}
export async function deleteRoomDocument(roomId:string,id:string){
  await invoke('delete_document',{roomId,id});
  await loadMasterAssets(roomId);
}
export function closeHandout(){room.update(state=>({...state,handout:null}));}
export async function importRoomCover(roomId:string):Promise<RoomCard>{const room=await invoke<RoomInfo>('import_room_cover',{roomId});const card={...room,role:'master' as const};return {...card,coverUrl:await coverUrl(card)};}
export async function saveRoomMap(roomId:string,map:RoomMap){
  await invoke('update_map',{roomId,mapId:map.id,name:map.name,gridVisible:map.gridVisible,snap:map.snap,cellPx:map.cellPx,metresPerCell:map.metresPerCell,backdropColor:map.backdropColor||null});
  await loadMasterAssets(roomId);
}
export async function showRoomMap(roomId:string,mapId:string|null){
  await invoke('activate_map',{roomId,mapId});
  await loadMasterAssets(roomId);
}
export async function assetBlobUrl(id:string):Promise<string>{
  if(!connection || !get(room).token) throw new Error('Sem conexão com a mesa');
  const response=await fetch(`http://${connection.address}:${connection.port}/assets/${encodeURIComponent(id)}`,{headers:{Authorization:`Bearer ${get(room).token}`}});
  if(!response.ok) throw new Error('Arquivo indisponível ou sem permissão');
  return URL.createObjectURL(await response.blob());
}
export async function sharePersonalAsset(kind:'art'|'audio',name:string,file:Blob):Promise<TokenArt|RoomAudio>{
  if(!connection || !get(room).token || get(room).status!=='ready')throw new Error('Sem conexão com a mesa');
  const response=await fetch(`http://${connection.address}:${connection.port}/assets`,{method:'POST',headers:{Authorization:`Bearer ${get(room).token}`,'X-Asset-Kind':kind,'X-Asset-Name':encodeURIComponent(name)},body:file});
  if(!response.ok)throw new Error(await response.text());
  return response.json();
}
export function clearRoll(){room.update(s=>({...s,roll:null}));}
export function clearDiceAnimation(){if(diceTimer)clearTimeout(diceTimer);room.update(s=>({...s,dice:null}));}
export function clearConflict(){room.update(s=>({...s,conflict:null}));}
export async function leaveRoom() {
  intentional=true;if(retry)clearTimeout(retry);
  if(diceTimer)clearTimeout(diceTimer);
  socket?.close();socket=null;
  if(get(room).role==='master') await invoke('stop_room');
  room.set({...empty});
  masterAssets.set({maps:[],tokenArt:[],threats:[],documents:[],activeMap:null});
}
