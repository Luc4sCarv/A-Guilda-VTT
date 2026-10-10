<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { fly } from 'svelte/transition';
  import { prefersReducedMotion } from 'svelte/motion';
  import Icon from './Icon.svelte';
  import SheetChangeReview from './SheetChangeReview.svelte';
  import AssetThumb from './AssetThumb.svelte';
  import MesaMedia from './MesaMedia.svelte';
  import MapCanvas from './MapCanvas.svelte';
  import { compendium } from '../compendium';
  import { damageTypes, conditions } from '../sheet-rules';
  import { isFogCovered } from '../fog';
  import { room, masterAssets, assetBlobUrl, sendRoom, clearRoll, clearDiceAnimation, closeHandout, reconnectRoom, type LocalCharacter, type RoomCharacter, type PlacedToken, type ThreatInstance, type RoomMark } from '../mesa';

  let { characters, onOpen, onNew, onConfig, onSettings, onLeave }: {
    characters: LocalCharacter[];
    onOpen: (sheet: RoomCharacter) => void;
    onNew: () => void;
    onConfig: () => void;
    onSettings: () => void;
    onLeave: () => void | Promise<void>;
  } = $props();

  type Tab = 'chat' | 'combate' | 'fichas' | 'handouts' | 'compendio' | 'pessoas' | 'ameacas' | 'tokens' | 'artes' | 'musica';
  type DockPanel = 'select' | 'pan' | 'draw' | 'text' | 'measure' | 'dice' | 'pin' | 'fog' | 'fx' | null;
  let tab = $state<Tab>('chat');
  let sideGroup = $derived(['chat','combate','fichas','handouts','pessoas','ameacas'].includes(tab) ? 'jogo' : 'biblioteca');
  let sidebarOpen = $state(true);
  let tool = $state<'select'|'pan'|'draw'|'text'|'measure'|'pin'|'fog'>('select');
  let fogMode=$state<'reveal'|'hide'>('reveal'),fogShape=$state<'rect'|'polygon'>('rect'),fogOpacity=$state(40),fogPreview=$state(false);
  let fogFinishSignal=$state(0),fogCancelSignal=$state(0);
  let drawShape=$state<'freehand'|'line'|'rect'>('freehand'),drawStroke=$state('#315fd4'),drawFill=$state('transparent'),drawWidth=$state(3),drawBlend=$state<'normal'|'multiply'>('normal');
  let textDraft=$state<{x:number;y:number;text:string;color:string;font:string;size:number;bold:boolean;italic:boolean}>({x:0,y:0,text:'',color:'#ffffff',font:'Arial',size:26,bold:false,italic:false});
  let pinDraft=$state<{x:number;y:number;title:string;note:string;visibility:'public'|'private'}>({x:0,y:0,title:'',note:'',visibility:'public'});
  let selectedMarkId=$state<string|null>(null);
  let openedPinId=$state<string|null>(null);
  let turnWindow=$state(false),turnX=$state(84),turnY=$state(72),turnDrag=$state<{x:number;y:number;left:number;top:number}|null>(null);
  let boardEl:HTMLDivElement|undefined;
  let dockPanel = $state<DockPanel>(null);
  let chatText = $state(''), chatAudience = $state<'all' | 'gm'>('all');
  let rollAudience = $state<'all' | 'gm' | 'secret' | 'self'>('all'), rollFormula = $state('1d20'), rollCount = $state(1);
  let catalogQuery = $state(''), catalogGroup = $state('Todos');
  let selectedLocal = $state(''), targetSheet = $state(''), damageFormula = $state('2d6');
  let damageType = $state('Corte'), magical = $state(false), lifeLoss = $state(false);
  let measureShape = $state<'line' | 'corners'>('line'), duration = $state<'instant' | 'permanent'>('instant');
  let measureDraft=$state({x1:0,y1:0,x2:0,y2:0});
  let shareMeasure = $state(false),measureSnap=$state(false),trackTokenMove=$state(true), metresPerCell = $state(1.5);
  let clearLocalMeasures=$state(0);
  let measure = $state<{x1:number;y1:number;x2:number;y2:number} | null>(null);
  let measuring = false, measureTimer: ReturnType<typeof setTimeout> | null = null;
  let messagesEl = $state<HTMLDivElement | undefined>(undefined);
  let handoutDialog=$state<HTMLDialogElement|undefined>(undefined);
  let selectedTokenId=$state<string|null>(null);
  let tokenMenu=$state<{id:string;x:number;y:number}|null>(null);
  let mapMenu=$state<{x:number;y:number;mapX:number;mapY:number}|null>(null),tokenSettingsOpen=$state(false);
  let audioEl=$state<HTMLAudioElement|undefined>(undefined),needsAudioActivation=$state(false),volume=$state(1);
  let loadedAudio='',audioUrl='',audioRequest=0,observedPlayback='';
  let sheetArt=$state<Record<string,string>>({});
  let tokenName=$state(''),tokenSize=$state(64),tokenOwner=$state('');
  let threatHpInput=$state(''),threatMpInput=$state('');
  let conditionName=$state(''),conditionUnit=$state<'manual'|'rounds'>('manual'),conditionRounds=$state(1);
  let initiatives=$state<Record<string,string>>({});
  let combatCharacters=$derived($room.placedTokens.filter(token=>!!token.sheetId));
  let combatThreats=$derived($room.placedTokens.filter(token=>$room.threatInstances.some(threat=>threat.tokenId===token.id)));
  let currentCombatEntry=$derived($room.combat?.entries[$room.combat.turn]||null);
  let selectedToken=$derived($room.placedTokens.find(t=>t.id===selectedTokenId)||null);
  let selectedThreat=$derived($room.threatInstances.find(item=>item.tokenId===selectedTokenId)||null);
  let selectedThreatBase=$derived($masterAssets.threats.find(item=>item.id===selectedThreat?.threatId)||null);
  let selectedMark=$derived($room.marks.find(mark=>mark.id===selectedMarkId)||null);
  let openedPin=$derived($room.marks.find(mark=>mark.id===openedPinId&&mark.kind==='pin'&&visiblePin(mark))||null);
  function visiblePin(mark:RoomMark){if(fogPreview&&mark.visibility==='private')return false;return $room.role==='master'&&!fogPreview||!$room.fog?.enabled||!isFogCovered(mark.data.x,mark.data.y,$room.fog.ops);}
  $effect(()=>{selectedTokenId;threatHpInput='';threatMpInput='';});
  let canControlSelected=$derived(!!selectedToken && ($room.role==='master'||(!!selectedToken.sheetId&&$room.characters.some(c=>c.id===selectedToken.sheetId&&c.owner===$room.token))||(!selectedToken.sheetId&&selectedToken.ownerParticipantId===$room.participantId)));
  let canEditSelected=$derived(!!selectedToken && ($room.role==='master'||(!!$room.participantId&&!selectedToken.sheetId&&selectedToken.creatorParticipantId===$room.participantId)));
  $effect(()=>{if(selectedToken){tokenName=selectedToken.name;tokenSize=selectedToken.size;tokenOwner=selectedToken.ownerParticipantId||'';}});

  let catalog = $derived(compendium.filter(item =>
    (catalogGroup === 'Todos' || item.group === catalogGroup) &&
    `${item.name} ${item.summary}`.toLocaleLowerCase('pt-BR').includes(catalogQuery.toLocaleLowerCase('pt-BR').trim())
  ).slice(0, 100));
  let distance = $derived(measure ? Math.round((measureShape === 'line'
    ? Math.hypot(measure.x2-measure.x1, measure.y2-measure.y1)
    : Math.abs(measure.x2-measure.x1)+Math.abs(measure.y2-measure.y1))
    / 64 * (Number.isFinite(metresPerCell) && metresPerCell > 0 ? metresPerCell : 1.5) * 10) / 10 : 0);

  $effect(() => {
    $room.chat.length;
    void tick().then(() => { if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight; });
  });
  $effect(()=>{if($room.handout)void tick().then(()=>{if(handoutDialog&&!handoutDialog.open)handoutDialog.showModal();});});

  function chat() {
    const text = chatText.trim();
    if (!text) return;
    const command = text.match(/^\/r(?:oll)?\s+(.+)$/i);
    const sent = command
      ? sendRoom({type:'roll_chat',formula:command[1],audience:chatAudience})
      : sendRoom({type:'chat',text,audience:chatAudience});
    if (sent) chatText = '';
  }
  function roll(formula: string,label='') {
    if (sendRoom({type:'roll_chat',formula,label,audience:rollAudience})) { closeDock(); tab = 'chat'; sidebarOpen = true; }
  }
  function closeDock() { dockPanel = null; tool = 'select'; measure = null; }
  const dockNames={select:'Selecionar',pan:'Arrastar mapa',draw:'Desenhar',text:'Texto',measure:'Medir',dice:'Rolo de dados',pin:'Alfinete',fog:'Névoa',fx:'FX'};
  function editMark(mark:RoomMark){selectedMarkId=mark.id;dockPanel=mark.kind==='drawing'?'draw':mark.kind==='text'?'text':mark.kind==='pin'?'pin':'measure';tool=mark.kind==='drawing'?'draw':mark.kind==='text'?'text':mark.kind==='pin'?'pin':'measure';if(mark.kind==='drawing'){drawShape=mark.data.shape;drawStroke=mark.data.stroke;drawFill=mark.data.fill;drawWidth=mark.data.width;drawBlend=mark.data.blend;}if(mark.kind==='text')textDraft={...textDraft,...mark.data};if(mark.kind==='pin')pinDraft={...pinDraft,...mark.data,visibility:mark.visibility};if(mark.kind==='measure'){measureShape=mark.data.shape;measureDraft={x1:mark.data.points[0].x,y1:mark.data.points[0].y,x2:mark.data.points[1].x,y2:mark.data.points[1].y};}}
  function saveText(){if(!textDraft.text.trim())return;if(sendRoom({type:'mark_save',kind:'text',visibility:'public',id:selectedMark?.kind==='text'?selectedMark.id:undefined,baseRevision:selectedMark?.revision,data:textDraft})){selectedMarkId=null;textDraft={...textDraft,text:''};}}
  function savePin(){if(!pinDraft.title.trim())return;const {visibility,...data}=pinDraft;if(sendRoom({type:'mark_save',kind:'pin',visibility,id:selectedMark?.kind==='pin'?selectedMark.id:undefined,baseRevision:selectedMark?.revision,data})){selectedMarkId=null;pinDraft={...pinDraft,title:'',note:''};}}
  function saveDrawingStyle(){if(selectedMark?.kind!=='drawing')return;if(sendRoom({type:'mark_save',kind:'drawing',visibility:'public',id:selectedMark.id,baseRevision:selectedMark.revision,data:{...selectedMark.data,stroke:drawStroke,fill:drawFill,width:drawWidth,blend:drawBlend}}))selectedMarkId=null;}
  function saveMeasure(){if(selectedMark?.kind!=='measure')return;const {x1,y1,x2,y2}=measureDraft;if(sendRoom({type:'mark_save',kind:'measure',visibility:'public',id:selectedMark.id,baseRevision:selectedMark.revision,data:{points:[{x:x1,y:y1},{x:x2,y:y2}],shape:measureShape}}))selectedMarkId=null;}
  function deleteMark(mark:RoomMark){if(confirm(`Apagar ${mark.kind==='pin'?'alfinete':'marcação'}?`)){if(sendRoom({type:'mark_delete',id:mark.id,baseRevision:mark.revision})&&selectedMarkId===mark.id)selectedMarkId=null;}}
  function dragToken(event:DragEvent,source:{artId?:string;sheetId?:string;threatId?:string}){event.dataTransfer?.setData('application/x-tormenta-token',JSON.stringify(source));if(event.dataTransfer)event.dataTransfer.effectAllowed='copy';}
  function placeCenter(source:{artId?:string;sheetId?:string;threatId?:string}){const map=$room.activeMap;if(map)sendRoom({type:'place_token',...source,x:map.width/2,y:map.height/2});}
  function changeThreatResource(instance:ThreatInstance,resource:'hp'|'mp',raw:string){const value=Number(raw.trim());if(!Number.isInteger(value))return;sendRoom({type:'set_threat_resource',tokenId:instance.tokenId,resource,mode:/^[+-]/.test(raw.trim())?'delta':'set',value,baseRevision:instance.revision});if(resource==='hp')threatHpInput='';else threatMpInput='';}
  function changeThreatCondition(instance:ThreatInstance,name:string,action:'add'|'remove'){
    if(sendRoom({type:'set_threat_condition',tokenId:instance.tokenId,name,action,unit:conditionUnit,remaining:conditionUnit==='rounds'?conditionRounds:null,baseRevision:instance.revision})&&action==='add')conditionName='';
  }
  function startCombat(){
    const scores:Record<string,number>={};
    for(const token of combatCharacters){const score=Number(initiatives[token.id]);if(!Number.isInteger(score)||!initiatives[token.id]?.trim())return;scores[token.id]=score;}
    if(initiatives.threats?.trim()){const score=Number(initiatives.threats);if(!Number.isInteger(score))return;scores.threats=score;}
    sendRoom({type:'start_combat',initiatives:scores});
  }
  function combatAction(type:'combat_next'|'combat_previous'|'combat_pause'|'end_combat'){
    if($room.combat)sendRoom({type,baseRevision:$room.combat.revision});
  }
  function moveTurnWindow(event:PointerEvent){if(!turnDrag)return;turnX=Math.max(8,Math.min((boardEl?.clientWidth||500)-300,turnDrag.left+event.clientX-turnDrag.x));turnY=Math.max(8,Math.min((boardEl?.clientHeight||500)-180,turnDrag.top+event.clientY-turnDrag.y));}
  function openSide(){closeDock();sidebarOpen=!sidebarOpen;}
  function moveSelected(dx:number,dy:number){const t=selectedToken,map=$room.activeMap;if(t&&map&&canControlSelected)sendRoom({type:'move_token',id:t.id,baseRevision:t.revision,x:Math.max(0,Math.min(map.width,t.x+dx)),y:Math.max(0,Math.min(map.height,t.y+dy))});}
  function toggleDock(panel: Exclude<DockPanel, null>) {
    if (dockPanel === panel) { closeDock(); return; }
    dockPanel = panel; tool = ['select','pan','draw','text','measure','pin','fog'].includes(panel)?panel as typeof tool:'select'; measure = null;sidebarOpen=false;selectedMarkId=null;
  }
  function setVolume(value:number){volume=value;if(audioEl)audioEl.volume=value;localStorage.setItem('tormenta-vtt-volume',String(value));}
  async function activateAudio(){try{await audioEl?.play();needsAudioActivation=false;}catch{needsAudioActivation=true;}}
  async function syncAudio(){
    const audio=audioEl,playback=$room.playback;
    if(!audio)return;
    const serial=++audioRequest;
    if(!playback.audioId){audio.pause();audio.removeAttribute('src');audio.load();loadedAudio='';if(audioUrl)URL.revokeObjectURL(audioUrl);audioUrl='';return;}
    if(loadedAudio!==playback.audioId){
      try{const url=await assetBlobUrl(playback.audioId);if(serial!==audioRequest){URL.revokeObjectURL(url);return;}audio.pause();if(audioUrl)URL.revokeObjectURL(audioUrl);audioUrl=url;audio.src=url;loadedAudio=playback.audioId;audio.load();}
      catch(error){room.update(s=>({...s,message:String(error)}));return;}
    }
    if(serial!==audioRequest)return;
    const position=playback.positionSeconds+(playback.playing?Math.max(0,Date.now()+$room.serverTimeOffset-playback.startedAt)/1000:0);
    try{if(Math.abs(audio.currentTime-position)>2)audio.currentTime=position;}catch{}
    if(playback.playing){try{await audio.play();needsAudioActivation=false;}catch{needsAudioActivation=true;}}
    else audio.pause();
  }
  $effect(()=>{const key=JSON.stringify($room.playback);if(!audioEl||key===observedPlayback)return;observedPlayback=key;void syncAudio();});
  $effect(()=>{if(audioEl)audioEl.volume=volume;});
  onMount(()=>{const saved=localStorage.getItem('tormenta-vtt-volume');if(saved!==null&&Number.isFinite(Number(saved)))setVolume(Math.max(0,Math.min(1,Number(saved))));return()=>{audioRequest++;audioEl?.pause();if(audioUrl)URL.revokeObjectURL(audioUrl);};});
  function point(event: PointerEvent) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return {x: Math.max(0, Math.min(rect.width, event.clientX-rect.left)), y: Math.max(0, Math.min(rect.height, event.clientY-rect.top))};
  }
  function beginMeasure(event: PointerEvent) {
    if($room.activeMap)return;
    if (tool !== 'measure' || event.button !== 0) return;
    if (measureTimer) clearTimeout(measureTimer);
    const p = point(event);
    measure = {x1:p.x,y1:p.y,x2:p.x,y2:p.y};
    measuring = true;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }
  function moveMeasure(event: PointerEvent) {
    if($room.activeMap)return;
    if (!measuring || !measure) return;
    const p = point(event);
    measure = {...measure,x2:p.x,y2:p.y};
  }
  function endMeasure(event: PointerEvent) {
    if($room.activeMap)return;
    if (!measuring || !measure) return;
    moveMeasure(event);
    measuring = false;
    const board = event.currentTarget as HTMLElement;
    if (shareMeasure && board.clientWidth && board.clientHeight) sendRoom({type:'measure',
      x1:measure.x1/board.clientWidth,y1:measure.y1/board.clientHeight,
      x2:measure.x2/board.clientWidth,y2:measure.y2/board.clientHeight,metres:distance});
    if (duration === 'instant') measureTimer = setTimeout(() => measure = null, 4000);
  }
</script>

<svelte:window onpointerdown={event=>{if((tokenMenu||mapMenu)&&!(event.target as Element).closest('.mesa-token-menu,.mesa-map-menu')){tokenMenu=null;mapMenu=null;tokenSettingsOpen=false;}}} onkeydown={event=>{if(event.key==='Escape'){tokenMenu=null;mapMenu=null;tokenSettingsOpen=false;closeDock();sidebarOpen=false;}else if(event.key===' '&&$room.dice&&!(event.target as Element).closest('input,textarea,[contenteditable="true"]')){event.preventDefault();clearDiceAnimation();}}}/>

<div class="mesa-tabletop">
  <audio bind:this={audioEl} onended={()=>{if($room.role==='master')sendRoom({type:'playback_control',action:'next'});}} onloadedmetadata={()=>{if(audioEl&&$room.playback.audioId){const p=$room.playback;audioEl.currentTime=p.positionSeconds+(p.playing?Math.max(0,Date.now()+$room.serverTimeOffset-p.startedAt)/1000:0);}}}></audio>
  <header class="mesa-tabletop-head">
    <div><button class="mesa-back" onclick={onLeave} title="Sair da mesa"><Icon name="back" size={16}/> Voltar</button><span class="mesa-tabletop-mark"><Icon name="dice" size={18}/></span><strong>{$room.room?.name || 'Conectando…'}</strong><small>{$room.role === 'master' ? 'Mestre' : $room.status === 'pending' ? 'Aguardando aprovação' : 'Jogador'}</small></div>
    <div><button aria-expanded={sidebarOpen} aria-controls="mesa-side-panel" onclick={openSide}><Icon name="book" size={16}/> Painel</button>{#if needsAudioActivation}<button onclick={activateAudio}>Ativar áudio</button>{/if}{#if $room.role==='master'}<button onclick={onConfig} disabled={!$room.room}><Icon name="settings" size={16}/> Área do mestre</button>{/if}<button aria-label="Configurações do aplicativo" title="Configurações do aplicativo" onclick={onSettings}><Icon name="settings" size={16}/></button></div>
  </header>
  <div class="mesa-tabletop-main" class:sidebar-collapsed={!sidebarOpen}>
    <nav class="mesa-tools" aria-label="Ferramentas da mesa">
      <button class:active={tool === 'select'} title="Selecionar" aria-label="Selecionar" aria-pressed={tool==='select'} onclick={() => toggleDock('select')}><Icon name="cursor" size={18}/></button>
      <button class:active={tool === 'pan'} title="Arrastar mapa" aria-label="Arrastar mapa" aria-pressed={tool==='pan'} onclick={() => toggleDock('pan')}><Icon name="hand" size={18}/></button>
      <span aria-hidden="true"></span>
      <button class:active={tool === 'draw'} title="Desenhar" aria-label="Desenhar" aria-pressed={tool==='draw'} disabled={!$room.activeMap||$room.status!=='ready'} onclick={() => toggleDock('draw')}><Icon name="pen" size={18}/></button>
      <button class:active={tool === 'text'} title="Texto" aria-label="Texto" aria-pressed={tool==='text'} disabled={!$room.activeMap||$room.status!=='ready'} onclick={() => toggleDock('text')}><Icon name="text" size={18}/></button>
      <button class:active={tool === 'measure'} title="Medir" aria-label="Medir" aria-pressed={tool==='measure'} onclick={() => toggleDock('measure')}><Icon name="ruler" size={18}/></button>
      <button class:active={dockPanel === 'dice'} title="Rolar dados" aria-label="Rolar dados" aria-expanded={dockPanel === 'dice'} onclick={() => toggleDock('dice')}><Icon name="dice" size={18}/></button>
      {#if $room.role==='master'}
        <button class:active={turnWindow} title="Ordem de turno" aria-label="Ordem de turno" aria-expanded={turnWindow} onclick={()=>{turnWindow=!turnWindow;dockPanel=null;}}><Icon name="turn" size={18}/></button>
        <button class:active={tool==='pin'} title="Alfinete" aria-label="Alfinete" aria-pressed={tool==='pin'} disabled={!$room.activeMap||$room.status!=='ready'} onclick={()=>toggleDock('pin')}><Icon name="pin" size={18}/></button>
        <button class:active={tool==='fog'} title="Névoa" aria-label="Editar névoa" aria-pressed={tool==='fog'} disabled={$room.activeMap?.kind!=='tactical'||$room.status!=='ready'} onclick={()=>toggleDock('fog')}><Icon name="fog" size={18}/></button>
      {/if}
      <button class:active={dockPanel === 'fx'} title="FX" aria-label="FX" aria-expanded={dockPanel === 'fx'} onclick={() => toggleDock('fx')}><Icon name="spark" size={18}/></button>
    </nav>
    {#key dockPanel}
      {#if dockPanel}<section id="mesa-dock-panel" class="mesa-dock-panel" class:dice-panel={dockPanel === 'dice'} aria-label={dockNames[dockPanel]} in:fly={{x:prefersReducedMotion.current?0:-14,duration:prefersReducedMotion.current?120:180}} out:fly={{x:prefersReducedMotion.current?0:-14,duration:prefersReducedMotion.current?120:140}}>
        <header><strong>{dockNames[dockPanel]}</strong><button aria-label="Fechar ferramenta" onclick={closeDock}><Icon name="close" size={15}/></button></header>
        {#if dockPanel === 'draw'}
          <p class="mesa-side-hint">Arraste no mapa para desenhar. Escolha uma marcação abaixo para alterar seu estilo.</p>
          <label>Forma<select bind:value={drawShape}><option value="freehand">Livre</option><option value="line">Linha</option><option value="rect">Retângulo</option></select></label>
          <div class="mesa-color-controls"><label>Traço<input type="color" bind:value={drawStroke}/></label><label>Preenchimento<input type="color" value={drawFill==='transparent'?'#ffffff':drawFill} onchange={e=>drawFill=e.currentTarget.value}/></label></div>
          <label class="mesa-check"><input type="checkbox" checked={drawFill==='transparent'} onchange={e=>drawFill=e.currentTarget.checked?'transparent':'#ffffff'}/> Sem preenchimento</label>
          <label>Espessura<input type="number" min="1" max="24" bind:value={drawWidth}/></label>
          <label>Modo<select bind:value={drawBlend}><option value="normal">Normal</option><option value="multiply">Multiplicar</option></select></label>
          {#if selectedMark?.kind==='drawing'}<button class="primary" disabled={$room.status!=='ready'} onclick={saveDrawingStyle}>Aplicar estilo à marcação</button>{/if}
        {:else if dockPanel === 'text'}
          <p class="mesa-side-hint">Clique no mapa para posicionar um texto; selecione um existente para editar.</p>
          <label>Texto<textarea bind:value={textDraft.text} maxlength="1000" rows="3"></textarea></label>
          <div class="mesa-pair"><label>Fonte<select bind:value={textDraft.font}><option>Arial</option><option>Georgia</option><option>Segoe UI</option></select></label><label>Tamanho<input type="number" min="12" max="72" bind:value={textDraft.size}/></label></div>
          <div class="mesa-color-controls"><label>Cor<input type="color" bind:value={textDraft.color}/></label><label>Posição X<input type="number" min="0" max={$room.activeMap?.width} bind:value={textDraft.x}/></label><label>Posição Y<input type="number" min="0" max={$room.activeMap?.height} bind:value={textDraft.y}/></label></div>
          <div class="mesa-audience"><label class="mesa-check"><input type="checkbox" bind:checked={textDraft.bold}/> Negrito</label><label class="mesa-check"><input type="checkbox" bind:checked={textDraft.italic}/> Itálico</label></div>
          <button class="primary" disabled={!textDraft.text.trim()||$room.status!=='ready'||!$room.activeMap} onclick={saveText}>{selectedMark?.kind==='text'?'Salvar texto':'Adicionar texto'}</button>
        {:else if dockPanel === 'measure'}
          <label>Forma<select bind:value={measureShape}><option value="line">Linha</option><option value="corners">Ângulos retos</option></select></label>
          <label class="mesa-measure-check"><input type="checkbox" bind:checked={measureSnap}/> Ajustar à grade</label>
          <label>Tempo de exibição<select bind:value={duration}><option value="instant">4 segundos</option><option value="permanent">Permanente</option></select></label>
          <label class="mesa-measure-check"><input type="checkbox" bind:checked={shareMeasure}/> Mostrar para outros</label>
          <label class="mesa-measure-check"><input type="checkbox" bind:checked={trackTokenMove}/> Medir movimento de tokens</label>
          <button onclick={()=>clearLocalMeasures++}>Limpar minhas medidas locais</button>
          {#if $room.activeMap}<small>1 célula = {$room.activeMap.cellPx} px = {$room.activeMap.metresPerCell} m. Ajuste na Área do mestre.</small>{:else}<label>Metros por célula<input type="number" min="0.1" max="100" step="0.1" bind:value={metresPerCell}/><small>1 célula = 64 px nesta área vazia.</small></label>{/if}
          {#if selectedMark?.kind==='measure'}<div class="mesa-pair"><label>Início X<input type="number" min="0" max={$room.activeMap?.width} bind:value={measureDraft.x1}/></label><label>Início Y<input type="number" min="0" max={$room.activeMap?.height} bind:value={measureDraft.y1}/></label><label>Fim X<input type="number" min="0" max={$room.activeMap?.width} bind:value={measureDraft.x2}/></label><label>Fim Y<input type="number" min="0" max={$room.activeMap?.height} bind:value={measureDraft.y2}/></label></div><button class="primary" disabled={$room.status!=='ready'||Object.values(measureDraft).some(value=>!Number.isFinite(value))} onclick={saveMeasure}>Salvar medida</button><button class="mesa-danger" onclick={()=>deleteMark(selectedMark!)}>Apagar medida selecionada</button>{/if}
        {:else if dockPanel === 'pin'}
          <p class="mesa-side-hint">Clique no mapa para colocar um alfinete. Os privados aparecem somente ao mestre.</p>
          <label>Título<input bind:value={pinDraft.title} maxlength="80"/></label><label>Nota<textarea bind:value={pinDraft.note} maxlength="2000" rows="4"></textarea></label>
          <label>Visibilidade<select bind:value={pinDraft.visibility}><option value="public">Todos</option><option value="private">Somente mestre</option></select></label>
          <div class="mesa-pair"><label>Posição X<input type="number" min="0" max={$room.activeMap?.width} bind:value={pinDraft.x}/></label><label>Posição Y<input type="number" min="0" max={$room.activeMap?.height} bind:value={pinDraft.y}/></label></div>
          <button class="primary" disabled={!pinDraft.title.trim()||$room.status!=='ready'||!$room.activeMap} onclick={savePin}>{selectedMark?.kind==='pin'?'Salvar alfinete':'Adicionar alfinete'}</button>
        {:else if dockPanel === 'fog'}
          <p class="mesa-side-hint">Cubra a cena. Arraste um retângulo ou clique nos vértices do polígono; Enter ou duplo clique conclui.</p>
          <div class="mesa-token-actions"><button disabled={$room.status!=='ready'} onclick={()=>{if(confirm('Cobrir toda a cena e apagar as áreas reveladas?'))sendRoom({type:'fog_set',action:'cover_all',baseRevision:$room.fog?.revision||0});}}>Cobrir tudo</button><button disabled={!$room.fog?.enabled} onclick={()=>sendRoom({type:'fog_set',action:'show_all',baseRevision:$room.fog?.revision})}>Mostrar tudo</button>{#if $room.fog?.enabled===false&&$room.fog.revision>0}<button onclick={()=>sendRoom({type:'fog_set',action:'resume',baseRevision:$room.fog?.revision})}>Retomar névoa</button>{/if}</div>
          {#if $room.fog?.enabled}<div class="mesa-audience" role="group" aria-label="Ferramenta de névoa"><button class:active={fogMode==='reveal'} onclick={()=>fogMode='reveal'}>Revelar</button><button class:active={fogMode==='hide'} onclick={()=>fogMode='hide'}>Ocultar</button></div>{/if}
          <label>Forma<select bind:value={fogShape}><option value="rect">Retângulo</option><option value="polygon">Polígono</option></select></label>
          {#if fogShape==='polygon'}<div class="mesa-token-actions"><button onclick={()=>fogFinishSignal++}>Concluir polígono</button><button onclick={()=>fogCancelSignal++}>Cancelar pontos</button></div>{/if}
          <label>Opacidade para o mestre · {fogOpacity}%<input type="range" min="10" max="90" step="1" bind:value={fogOpacity}/></label>
          <label class="mesa-check"><input type="checkbox" bind:checked={fogPreview}/> Visualizar como jogador</label>
        {:else if dockPanel === 'dice'}
          <div class="mesa-audience" role="group" aria-label="Visibilidade da rolagem"><button class:active={rollAudience==='all'} onclick={() => rollAudience='all'}>Todos</button><button class:active={rollAudience==='gm'} onclick={() => rollAudience='gm'}>Mestre</button><button class:active={rollAudience==='secret'} onclick={() => rollAudience='secret'}>Oculta</button><button class:active={rollAudience==='self'} onclick={() => rollAudience='self'}>Só eu</button></div>
          <label>Quantidade<input type="number" min="1" max="100" bind:value={rollCount}/></label>
          <div class="mesa-dice-grid">{#each [4,6,8,10,12,20,100] as sides}<button disabled={$room.status!=='ready'} onclick={() => roll(`${Math.max(1,Math.min(100,Math.trunc(rollCount)||1))}d${sides}`)}>D{sides}</button>{/each}</div>
          <form class="mesa-roll-form" onsubmit={event => {event.preventDefault();roll(rollFormula);}}><label>Expressão<input bind:value={rollFormula} placeholder="2d6+3"/></label><button disabled={!rollFormula.trim() || $room.status!=='ready'}>Rolar</button></form>
          {#if $room.role === 'master'}<details class="mesa-damage-tools"><summary>Aplicar dano</summary><select bind:value={targetSheet} aria-label="Ficha-alvo"><option value="">Ficha-alvo</option>{#each $room.characters as sheet}<option value={sheet.id}>{sheet.data.name}</option>{/each}</select><div class="mesa-pair"><input bind:value={damageFormula} aria-label="Expressão de dano" placeholder="2d6+3"/><button disabled={!targetSheet || !damageFormula.trim()} onclick={() => sendRoom({type:'roll_damage',sheetId:targetSheet,formula:damageFormula,damageType,magical,lifeLoss})}>Preparar</button></div><label>Tipo<select bind:value={damageType} disabled={lifeLoss}><option>Geral</option>{#each damageTypes as kind}<option>{kind}</option>{/each}</select></label><label class="mesa-check"><input type="checkbox" bind:checked={magical} disabled={lifeLoss}/> Origem mágica</label><label class="mesa-check"><input type="checkbox" bind:checked={lifeLoss}/> Perda de PV (ignora RD e PV temporários)</label>{#if $room.roll}<div class="mesa-roll"><strong>{$room.roll.formula} = {$room.roll.total}</strong><span>Dados: {$room.roll.rolls.join(', ')} · {$room.roll.lifeLoss?'Perda de PV':`${$room.roll.damageType}${$room.roll.magical?' mágico':''}`}</span><button class="primary" onclick={() => {sendRoom({type:'apply_damage',id:$room.roll!.id});clearRoll();}}>Confirmar</button><button onclick={clearRoll}>Cancelar</button></div>{/if}</details>{/if}
        {:else if dockPanel === 'fx'}<p class="mesa-empty-panel">Efeitos visuais chegarão em uma próxima atualização. Nenhum efeito é aplicado por este menu.</p>
        {:else if dockPanel === 'pan'}<small>Arraste qualquer ponto para mover o mapa, inclusive sobre tokens.</small>
        {:else}<small>Clique em um token para selecioná-lo e arrastá-lo. Arraste uma área vazia para mover o mapa.</small>{/if}
        {#if dockPanel==='select'&&$room.marks.some(mark=>mark.kind==='pin'&&visiblePin(mark))}<div class="mesa-mark-list"><h3>Alfinetes</h3>{#each $room.marks.filter(mark=>mark.kind==='pin'&&visiblePin(mark)) as mark}<button onclick={()=>openedPinId=mark.id}>{mark.data.title}</button>{/each}</div>{/if}
        {#if dockPanel&&['draw','text','measure','pin'].includes(dockPanel)&&$room.marks.some(mark=>mark.kind===(dockPanel==='draw'?'drawing':dockPanel))}
          <div class="mesa-mark-list"><h3>Na cena</h3>{#each $room.marks.filter(mark=>mark.kind===(dockPanel==='draw'?'drawing':dockPanel)) as mark}<div class="mesa-mark-row">{#if $room.role==='master'||!!$room.participantId&&mark.ownerParticipantId===$room.participantId}<button onclick={()=>editMark(mark)} aria-label={`Editar ${mark.kind==='text'?mark.data.text:mark.kind==='pin'?mark.data.title:mark.kind==='measure'?`${mark.data.metres} metros`:'desenho'} de ${mark.author}`}><span>{mark.kind==='text'?mark.data.text:mark.kind==='pin'?mark.data.title:mark.kind==='measure'?`${mark.data.metres} m`:`${mark.data.shape} · ${mark.author}`}</span></button><button class="mesa-danger" aria-label="Apagar marcação" onclick={()=>deleteMark(mark)}><Icon name="trash" size={14}/></button>{:else}<span>{mark.kind==='text'?mark.data.text:mark.kind==='measure'?`${mark.data.metres} m`:`${mark.data.shape} · ${mark.author}`}</span>{/if}</div>{/each}</div>
        {/if}
      </section>{/if}
    {/key}
    <div class="mesa-board" bind:this={boardEl} class:measuring={tool === 'measure'} role="application" aria-label={$room.activeMap?`Mapa ${$room.activeMap.name}`:'Área do mapa vazia'}
      onpointerdown={beginMeasure} onpointermove={moveMeasure} onpointerup={endMeasure}>
      {#if $room.combat}<button class="mesa-combat-banner" onclick={()=>{tab='combate';sidebarOpen=true;}} aria-label={`Combate, rodada ${$room.combat.round}, turno de ${currentCombatEntry?.name}`}><strong>Rodada {$room.combat.round}</strong><span>{currentCombatEntry?.name}</span>{#if $room.combat.paused}<small>Pausado</small>{/if}</button>{/if}
      {#if turnWindow&&$room.role==='master'}<section class="mesa-turn-window" style:left={`${turnX}px`} style:top={`${turnY}px`} aria-label="Ordem de turno">
        <header><button class="mesa-turn-handle" aria-label="Mover ordem de turno" title="Arraste ou use as setas" onpointerdown={e=>{turnDrag={x:e.clientX,y:e.clientY,left:turnX,top:turnY};e.currentTarget.setPointerCapture(e.pointerId);}} onpointermove={moveTurnWindow} onpointerup={()=>turnDrag=null} onpointercancel={()=>turnDrag=null} onkeydown={e=>{if(e.key==='ArrowLeft')turnX=Math.max(8,turnX-20);else if(e.key==='ArrowRight')turnX=Math.min(Math.max(8,(boardEl?.clientWidth||500)-300),turnX+20);else if(e.key==='ArrowUp')turnY=Math.max(8,turnY-20);else if(e.key==='ArrowDown')turnY=Math.min(Math.max(8,(boardEl?.clientHeight||500)-180),turnY+20);else return;e.preventDefault();}}>Ordem de turno</button><button aria-label="Fechar ordem de turno" onclick={()=>turnWindow=false}><Icon name="close" size={16}/></button></header>
        <div class="mesa-turn-body">{#if $room.combat}<p>Rodada {$room.combat.round}{#if $room.combat.paused} · pausada{/if}</p><ol class="mesa-combat-order">{#each $room.combat.entries as entry,index}<li class:active={index===$room.combat.turn}><span>{entry.name}</span><strong>{entry.initiative}</strong></li>{/each}</ol><div class="mesa-token-actions"><button onclick={()=>combatAction('combat_previous')}>Anterior</button><button class="primary" disabled={$room.combat.paused} onclick={()=>combatAction('combat_next')}>Próximo</button><button onclick={()=>combatAction('combat_pause')}>{$room.combat.paused?'Retomar':'Pausar'}</button></div><button class="mesa-danger" onclick={()=>{if(confirm('Encerrar este combate?'))combatAction('end_combat');}}>Encerrar combate</button>
          {:else}<p>Coloque personagens ou ameaças na cena e informe as iniciativas.</p>{#each combatCharacters as token}<label>{token.name}<input type="number" min="-1000" max="1000" bind:value={initiatives[token.id]}/></label>{/each}{#if combatThreats.length}<label>Ameaças<input type="number" min="-1000" max="1000" placeholder="Automática se vazio" bind:value={initiatives.threats}/></label>{/if}<button class="primary" disabled={$room.status!=='ready'||$room.activeMap?.kind!=='tactical'||combatCharacters.length+combatThreats.length===0||combatCharacters.some(token=>!initiatives[token.id]?.trim())} onclick={startCombat}>Iniciar combate</button>{/if}</div>
      </section>{/if}
      {#if fogPreview}<button class="mesa-fog-preview" onclick={()=>fogPreview=false}>Visão do jogador · sair</button>{/if}
      {#if $room.activeMap}<MapCanvas map={$room.activeMap} tokens={$room.placedTokens} characters={$room.characters} {tool} {fogMode} {fogPreview} {fogShape} {fogOpacity} {fogFinishSignal} {fogCancelSignal} {clearLocalMeasures} {measureShape} {measureSnap} {trackTokenMove} {duration} {shareMeasure} {drawShape} {drawStroke} {drawFill} {drawWidth} {drawBlend} selectedId={selectedTokenId} onSelect={id=>{selectedTokenId=id;tokenMenu=null;mapMenu=null;}} onPlace={(kind,x,y)=>{selectedMarkId=null;if(kind==='text'){textDraft={...textDraft,x:Math.round(x),y:Math.round(y),text:''};dockPanel='text';}else{pinDraft={...pinDraft,x:Math.round(x),y:Math.round(y),title:'',note:''};dockPanel='pin';}}} onPinOpen={mark=>openedPinId=mark.id} onTokenMenu={(id,x,y)=>{tokenMenu={id,x,y};mapMenu=null;tokenSettingsOpen=false;}} onMapMenu={(x,y,mapX,mapY)=>{mapMenu={x,y,mapX,mapY};tokenMenu=null;}}/>{:else}
        {#if measure}<svg class="mesa-measure-line" aria-hidden="true"><line x1={measure.x1} y1={measure.y1} x2={measure.x2} y2={measure.y2}/><circle cx={measure.x1} cy={measure.y1} r="4"/><circle cx={measure.x2} cy={measure.y2} r="4"/></svg><span class="mesa-measure-label" style:left={`${measure.x2}px`} style:top={`${measure.y2}px`}>{distance} m</span>{/if}
        {#if $room.measure && $room.measure.token !== $room.token}<svg class="mesa-measure-line shared" aria-hidden="true"><line x1={`${$room.measure.x1*100}%`} y1={`${$room.measure.y1*100}%`} x2={`${$room.measure.x2*100}%`} y2={`${$room.measure.y2*100}%`}/></svg><span class="mesa-measure-shared">{$room.measure.author}: {$room.measure.metres} m</span>{/if}
      {/if}
      {#if $room.dice}<div class="mesa-dice-overlay"><button class="mesa-dice-reveal" aria-label={`Resultado ${$room.dice.total}; pular animação`} onclick={clearDiceAnimation}><span class="mesa-dice-face">{$room.dice.total}</span><small>{$room.dice.formula}</small></button></div>{/if}
      {#if openedPin}<aside class="mesa-pin-note" aria-label={`Alfinete ${openedPin.data.title}`}><header><strong>{openedPin.data.title}</strong><button aria-label="Fechar alfinete" onclick={()=>openedPinId=null}><Icon name="close" size={16}/></button></header><p>{openedPin.data.note||'Sem nota adicional.'}</p><small>{openedPin.visibility==='private'?'Somente mestre':'Compartilhado'} · {openedPin.author}</small></aside>{/if}
      {#if selectedToken && canControlSelected}<div class="mesa-token-controls" aria-label={`Mover ${selectedToken.name}`}><strong>{selectedToken.name}</strong><div><button aria-label="Mover para a esquerda" onclick={()=>moveSelected(-($room.activeMap?.cellPx||64),0)}>←</button><button aria-label="Mover para cima" onclick={()=>moveSelected(0,-($room.activeMap?.cellPx||64))}>↑</button><button aria-label="Mover para baixo" onclick={()=>moveSelected(0,$room.activeMap?.cellPx||64)}>↓</button><button aria-label="Mover para a direita" onclick={()=>moveSelected($room.activeMap?.cellPx||64,0)}>→</button></div></div>{/if}
      {#if selectedToken?.conditions?.length}<div class="mesa-token-conditions" aria-label={`Condições de ${selectedToken.name}`}>{#each selectedToken.conditions as condition}<span>{condition}</span>{/each}</div>{/if}
      {#if mapMenu}<section class="mesa-map-menu" aria-label="Ações do mapa" style:left={`${mapMenu.x}px`} style:top={`${mapMenu.y}px`}><button onclick={()=>{sendRoom({type:'ping',mapId:$room.activeMap?.id,x:mapMenu!.mapX,y:mapMenu!.mapY});mapMenu=null;}}>◉ Ping Todos</button></section>{/if}
      {#if tokenMenu && selectedToken?.id===tokenMenu.id}<section class="mesa-token-menu" aria-label={`Ações de ${selectedToken.name}`} style:left={`${tokenMenu.x}px`} style:top={`${tokenMenu.y}px`}>
        {#if !tokenSettingsOpen}<button onclick={()=>{sendRoom({type:'ping',mapId:$room.activeMap?.id,x:selectedToken!.x,y:selectedToken!.y});tokenMenu=null;}}>◉ Ping Todos</button>
        {#if canEditSelected}<button onclick={()=>tokenSettingsOpen=true}>⚙ Configurações</button><button class="mesa-danger" onclick={()=>{if(confirm(`Retirar ${selectedToken!.name} do mapa?`)){sendRoom({type:'remove_token',id:selectedToken!.id});selectedTokenId=null;tokenMenu=null;}}}>Retirar</button>{/if}
        {:else}<header><strong>Token · {selectedToken.name}</strong><button aria-label="Fechar configurações do token" onclick={()=>{tokenMenu=null;tokenSettingsOpen=false;}}>×</button></header>
        <label>Nome<input bind:value={tokenName} maxlength="80"/></label>
        <label>Tamanho (px)<input type="number" min="16" max="2048" bind:value={tokenSize}/></label>
        {#if $room.role==='master'}{#if !selectedToken.sheetId}<label>Controlado por<select bind:value={tokenOwner}><option value="">Somente mestre</option>{#each $room.participants.filter(p=>p.approved) as person}<option value={person.id}>{person.name}</option>{/each}</select></label>{:else}<small>Controle vinculado à ficha.</small>{/if}{/if}
        <div class="mesa-token-actions"><button disabled={!tokenName.trim()||!Number.isFinite(Number(tokenSize))||Number(tokenSize)<16||Number(tokenSize)>2048} onclick={()=>{sendRoom({type:'update_token',id:selectedToken!.id,name:tokenName.trim(),size:Number(tokenSize),ownerParticipantId:tokenOwner||null});tokenMenu=null;tokenSettingsOpen=false;}}>Salvar</button><button onclick={()=>tokenSettingsOpen=false}>Voltar</button></div>{/if}
      </section>{/if}
      {#if $room.status !== 'ready'}<div class="mesa-board-status"><strong>{$room.status === 'pending' ? 'Aguardando aprovação do mestre' : $room.status === 'offline' ? 'Conexão interrompida' : 'Conectando…'}</strong>{#if $room.status === 'offline'}<button onclick={reconnectRoom}>Reconectar</button>{/if}</div>{/if}
    </div>
    <aside id="mesa-side-panel" class="mesa-side" inert={!sidebarOpen} aria-hidden={!sidebarOpen}>
      <nav class="mesa-side-tabs" aria-label="Painéis da mesa">
        <div class="mesa-side-groups" role="group" aria-label="Grupos do painel"><button class:active={sideGroup==='jogo'} aria-pressed={sideGroup==='jogo'} onclick={()=>tab='chat'}>Jogo</button><button class:active={sideGroup==='biblioteca'} aria-pressed={sideGroup==='biblioteca'} onclick={()=>tab='compendio'}>Biblioteca</button></div>
        <div class="mesa-side-destinations">
          {#if sideGroup==='jogo'}{#each [['chat','book','Chat'],['fichas','users','Fichas'],['pessoas','table','Participantes']] as item}<button class:active={tab===item[0]} aria-pressed={tab===item[0]} onclick={()=>tab=item[0] as Tab}><Icon name={item[1]} size={16}/>{item[2]}</button>{/each}<button class:active={tab==='combate'} aria-pressed={tab==='combate'} onclick={()=>tab='combate'}><Icon name="sword" size={16}/>Combate</button><button class:active={tab==='handouts'} aria-pressed={tab==='handouts'} onclick={()=>tab='handouts'}><Icon name="guide" size={16}/>Handouts</button>{#if $room.role==='master'}<button class:active={tab==='ameacas'} aria-pressed={tab==='ameacas'} onclick={()=>tab='ameacas'}><Icon name="warning" size={16}/>Ameaças</button>{/if}
          {:else}{#each [['compendio','guide','Compêndio'],['artes','image','Artes'],['musica','music','Música']] as item}<button class:active={tab===item[0]} aria-pressed={tab===item[0]} onclick={()=>tab=item[0] as Tab}><Icon name={item[1]} size={16}/>{item[2]}</button>{/each}{#if $room.role==='master'}<button class:active={tab==='tokens'} aria-pressed={tab==='tokens'} onclick={()=>tab='tokens'}><Icon name="spark" size={16}/>Tokens</button>{/if}{/if}
        </div>
      </nav>
      {#if $room.message}<p class="mesa-side-notice" role="status">{$room.message}</p>{/if}
      {#if tab === 'chat'}<div class="mesa-side-content mesa-chat-panel">
        <h2>Chat</h2><div class="mesa-chat-log" bind:this={messagesEl} aria-live="polite">
          {#each $room.chat as line}<article class:mesa-roll-card={line.kind==='roll'} class:mesa-action-line={line.kind==='action'} class:mesa-system-line={line.kind==='system'} class:mesa-handout-line={line.kind==='handout'}><div><strong>{line.author}{line.kind==='roll'?` · ${line.details?.label||'Rolagem'}`:''}</strong><small>{line.audience==='gm'?'Mestre':line.audience==='secret'?'Oculta':line.audience==='self'?'Só eu':new Date(line.at*1000).toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit'})}</small></div>{#if line.kind==='roll'&&line.details}<strong class="mesa-roll-total">{line.details.total}</strong><p class="mesa-roll-formula">{line.details.formula}</p>{#if line.details.rolls?.length}<details><summary>Dados individuais</summary><span>{line.details.rolls.join(' · ')}</span></details>{/if}{:else if line.kind==='handout'&&line.details?.body}<p>{line.details.title||line.text}</p><button onclick={()=>room.update(state=>({...state,handout:{id:line.details?.id||'',title:line.details?.title||line.text,body:line.details?.body||''}}))}>Abrir handout</button>{:else}<p>{line.text}</p>{/if}</article>{:else}<p class="mesa-empty-panel">A conversa começa aqui.</p>{/each}
        </div>
        <form onsubmit={event => {event.preventDefault();chat();}}><textarea aria-label="Mensagem" bind:value={chatText} maxlength="2000" placeholder="Mensagem ou /r 1d20+5" onkeydown={event => {if(event.key==='Enter'&&!event.shiftKey){event.preventDefault();chat();}}}></textarea><div><select bind:value={chatAudience} aria-label="Destino da mensagem"><option value="all">Todos</option><option value="gm">Mestre</option></select><button class="primary" disabled={!chatText.trim() || $room.status !== 'ready'}>Enviar</button></div></form>
      </div>{/if}
      {#if tab === 'combate'}<div class="mesa-side-content mesa-combat-panel">
        <h2>Combate</h2>
        {#if $room.combat}
          <p class="mesa-side-hint">Rodada {$room.combat.round}{#if $room.combat.paused} · pausado{/if}</p>
          <ol class="mesa-combat-order">{#each $room.combat.entries as entry,index}<li class:active={index===$room.combat.turn}><span>{entry.name}</span><strong>{entry.initiative}</strong>{#if entry.tokenIds.length>1}<small>{entry.tokenIds.length} ameaças</small>{/if}</li>{/each}</ol>
          <p class="mesa-combat-current">Turno de <strong>{currentCombatEntry?.name}</strong></p>
          {#if $room.role==='master'}<div class="mesa-token-actions"><button onclick={()=>combatAction('combat_previous')}>Anterior</button><button class="primary" disabled={$room.combat.paused} onclick={()=>combatAction('combat_next')}>Próximo turno</button><button onclick={()=>combatAction('combat_pause')}>{$room.combat.paused?'Retomar':'Pausar'}</button></div><button class="mesa-danger" onclick={()=>{if(confirm('Encerrar este combate?'))combatAction('end_combat');}}>Encerrar combate</button>
          {:else if currentCombatEntry?.ownerParticipantId===$room.participantId&&!$room.combat.paused}<button class="primary" onclick={()=>combatAction('combat_next')}>Encerrar meu turno</button>{:else}<p class="mesa-side-hint">Aguarde seu turno.</p>{/if}
        {:else if $room.role==='master'}
          <p class="mesa-side-hint">Coloque fichas e ameaças na cena tática. Informe a iniciativa já rolada de cada personagem; as ameaças rolam juntas com o menor bônus.</p>
          {#each combatCharacters as token}<label>{token.name}<input type="number" min="-1000" max="1000" placeholder="Resultado da iniciativa" bind:value={initiatives[token.id]}/></label>{/each}
          {#if combatThreats.length}<label>Ameaças ({combatThreats.length})<input type="number" min="-1000" max="1000" placeholder="Automática se vazio" bind:value={initiatives.threats}/></label>{/if}
          <button class="primary" disabled={$room.status!=='ready'||$room.activeMap?.kind!=='tactical'||combatCharacters.length+combatThreats.length===0||combatCharacters.some(token=>!initiatives[token.id]?.trim())} onclick={startCombat}>Iniciar combate</button>
        {:else}<p class="mesa-empty-panel">O mestre ainda não iniciou combate nesta cena.</p>{/if}
      </div>{/if}
      {#if tab==='handouts'}<div class="mesa-side-content"><h2>Handouts</h2><p class="mesa-side-hint">Conteúdo público da campanha. O mestre pode apresentar um handout a todos.</p>{#each $room.documents.filter(document=>document.kind==='handout') as document}<div class="mesa-side-row"><button onclick={()=>room.update(state=>({...state,handout:{id:document.id||'',title:document.title,body:document.body}}))}><span><strong>{document.title}</strong><small>{document.visibility==='public'?'Disponível':'Só mestre'}</small></span><Icon name="chevron" size={16}/></button>{#if $room.role==='master'}<button disabled={$room.status!=='ready'} onclick={()=>sendRoom({type:'show_handout',id:document.id})}>Mostrar a todos</button>{/if}</div>{:else}<p class="mesa-empty-panel">Nenhum handout disponível.</p>{/each}{#if $room.role==='master'}<button class="mesa-subtle" onclick={onConfig}>Preparar conteúdo</button>{/if}</div>{/if}
      {#if tab === 'fichas'}<div class="mesa-side-content">
        <h2>Fichas</h2><p class="mesa-side-hint">O mestre vê todas; jogadores veem só as próprias.</p>
        <SheetChangeReview/>
        {#each $room.characters as sheet}<div class="mesa-side-row"><button onclick={() => onOpen(sheet)}><span><strong>{sheet.data.name}</strong><small>Revisão {sheet.revision}</small></span><Icon name="chevron" size={16}/></button>{#if $room.role==='master'}<label class="mesa-permission"><input type="checkbox" checked={!!sheet.canEditProficiencies} onchange={e=>sendRoom({type:'set_proficiency_permission',sheetId:sheet.id,allowed:e.currentTarget.checked})}/> Proficiências</label>{/if}</div>{:else}<p class="mesa-empty-panel">Nenhuma ficha atribuída.</p>{/each}
        {#if $room.role === 'player'}<h3>Enviar ao mestre</h3><select bind:value={selectedLocal} aria-label="Ficha local"><option value="">Escolha uma ficha local</option>{#each characters as c}<option value={c.id}>{c.name}</option>{/each}</select><button disabled={!selectedLocal} onclick={() => {const c=characters.find(x=>x.id===selectedLocal);if(c)sendRoom({type:'offer_sheet',data:c.data});}}>Enviar para aprovação</button><button class="mesa-subtle" onclick={onNew}>Criar ficha</button>
        {:else if $room.role === 'master'}<h3>Fichas aguardando</h3>{#each $room.offers as offer}<div class="mesa-person"><span>{offer.participantName}: {offer.data.name}</span><button onclick={() => sendRoom({type:'accept_sheet',participantId:offer.participantId})}>Aceitar</button></div>{:else}<p class="mesa-side-hint">Nenhuma ficha aguardando.</p>{/each}<button class="mesa-subtle" onclick={onConfig}>Atribuir ficha na configuração</button>{/if}
      </div>{/if}
      {#if tab==='ameacas' && $room.role==='master'}<div class="mesa-side-content mesa-token-panel"><h2>Ameaças</h2><p class="mesa-side-hint">Arraste para a cena ou coloque no centro. Selecione um token para controlar seus recursos.</p>{#each $masterAssets.threats as threat}<div class="mesa-token-source" role="group" aria-label={`Ameaça ${threat.name}`} draggable="true" ondragstart={event=>dragToken(event,{threatId:threat.id})}><strong>{threat.name} · ND {threat.nd}</strong><button disabled={!$room.activeMap} onclick={()=>placeCenter({threatId:threat.id})}>Colocar no centro</button></div>{:else}<p class="mesa-empty-panel">Crie uma ficha na Área do mestre.</p>{/each}{#if selectedThreat && selectedThreatBase}<section class="mesa-threat-live"><h3>{selectedThreat.name}</h3><small>ND {selectedThreatBase.nd} · Defesa {selectedThreatBase.defense} · Fort {selectedThreatBase.fortitude} · Ref {selectedThreatBase.reflex} · Von {selectedThreatBase.will}</small><div class="mesa-threat-resources"><form onsubmit={event=>{event.preventDefault();changeThreatResource(selectedThreat!,'hp',threatHpInput);}}><label>PV {selectedThreat.currentHp} / {selectedThreatBase.maxHp}<input bind:value={threatHpInput} placeholder="-12, +8 ou 15" aria-label="Alterar PV"/></label><button disabled={!threatHpInput.trim()}>Aplicar</button></form><form onsubmit={event=>{event.preventDefault();changeThreatResource(selectedThreat!,'mp',threatMpInput);}}><label>PM {selectedThreat.currentMp} / {selectedThreatBase.maxMp}<input bind:value={threatMpInput} placeholder="-2, +3 ou 0" aria-label="Alterar PM"/></label><button disabled={!threatMpInput.trim()}>Aplicar</button></form></div>{#if selectedThreatBase.attackFormula}<button onclick={()=>roll(selectedThreatBase!.attackFormula,`${selectedThreatBase!.name} · ${selectedThreatBase!.attackName||'Ataque'}`)}>Rolar ataque · {selectedThreatBase.attackFormula}</button>{/if}{#if selectedThreatBase.damageFormula}<button onclick={()=>roll(selectedThreatBase!.damageFormula,`${selectedThreatBase!.name} · Dano`)}>Rolar dano · {selectedThreatBase.damageFormula}</button>{/if}<div class="mesa-threat-conditions"><h4>Condições</h4>{#each selectedThreat.conditions as active}<div class="mesa-person"><span>{active.name}{#if active.unit==='rounds'} · {active.remaining} rodadas{/if}</span><button onclick={()=>changeThreatCondition(selectedThreat!,active.name,'remove')}>Remover</button></div>{:else}<small>Sem condições.</small>{/each}<div class="mesa-threat-condition-add"><select aria-label="Condição" bind:value={conditionName}><option value="">Escolha uma condição</option>{#each conditions.filter(item=>!selectedThreat?.conditions.some(active=>active.name===item.name)) as item}<option value={item.name}>{item.name}</option>{/each}</select><select aria-label="Duração" bind:value={conditionUnit}><option value="manual">Manual</option><option value="rounds">Rodadas</option></select>{#if conditionUnit==='rounds'}<input type="number" min="1" max="100" aria-label="Número de rodadas" bind:value={conditionRounds}/>{/if}<button disabled={!conditionName||conditionUnit==='rounds'&&(!Number.isInteger(conditionRounds)||conditionRounds<1||conditionRounds>100)} onclick={()=>changeThreatCondition(selectedThreat!,conditionName,'add')}>Aplicar</button></div><small>Os efeitos mecânicos são resolvidos pelo mestre; a duração em rodadas expira automaticamente.</small></div>{#if selectedThreatBase.actions}<p>{selectedThreatBase.actions}</p>{/if}{#if selectedThreatBase.notes}<details><summary>Notas táticas privadas</summary><p>{selectedThreatBase.notes}</p></details>{/if}{#if selectedThreatBase.treasure}<details><summary>Tesouro privado</summary><p>{selectedThreatBase.treasure}</p></details>{/if}</section>{/if}<button class="mesa-subtle" onclick={onConfig}>Preparar ameaças</button></div>{/if}
      {#if tab==='tokens' && $room.role==='master'}<div class="mesa-side-content mesa-token-panel">
        <h2>Tokens</h2><p class="mesa-side-hint">Arraste para o mapa ou adicione ao centro. Um token de ficha usa o dono da própria ficha.</p>
        <h3>Fichas da mesa</h3>{#each $room.characters as sheet}<div class="mesa-token-source" role="group" aria-label={`Token ${sheet.data.name}`} draggable="true" ondragstart={e=>dragToken(e,{sheetId:sheet.id,artId:sheetArt[sheet.id]||undefined})}><strong>{sheet.data.name}</strong>{#if $masterAssets.tokenArt.length}<select aria-label={`Imagem de ${sheet.data.name}`} bind:value={sheetArt[sheet.id]}><option value="">Iniciais</option>{#each $masterAssets.tokenArt as art}<option value={art.id}>{art.name}</option>{/each}</select>{/if}<button disabled={!$room.activeMap} onclick={()=>placeCenter({sheetId:sheet.id,artId:sheetArt[sheet.id]||undefined})}>Adicionar ao centro</button></div>{:else}<p class="mesa-empty-panel">Nenhuma ficha atribuída.</p>{/each}
        <h3>Tokens avulsos</h3>{#each $masterAssets.tokenArt as art (art.id)}<div class="mesa-token-source" role="group" aria-label={`Token ${art.name}`} draggable="true" ondragstart={e=>dragToken(e,{artId:art.id})}><div class="mesa-token-source-title"><AssetThumb id={art.id} alt=""/><strong>{art.name}</strong></div><button disabled={!$room.activeMap} onclick={()=>placeCenter({artId:art.id})}>Adicionar ao centro</button></div>{:else}<p class="mesa-empty-panel">Envie imagens na Área do mestre.</p>{/each}
        <button class="mesa-subtle" onclick={onConfig}>Gerenciar imagens</button>
      </div>{/if}
      {#if tab==='artes'}<MesaMedia kind="art"/>{/if}
      {#if tab==='musica'}<MesaMedia kind="audio" {needsAudioActivation} onActivate={activateAudio} {volume} onVolume={setVolume}/>{/if}
      {#if tab === 'compendio'}<div class="mesa-side-content">
        <h2>Compêndio</h2><p class="mesa-side-hint">Amostra local do Livro Básico; o catálogo do Fichas de Nimb ainda não foi importado.</p><input type="search" bind:value={catalogQuery} placeholder="Buscar item, magia ou poder" aria-label="Buscar no compêndio"/><select bind:value={catalogGroup} aria-label="Tipo de conteúdo"><option>Todos</option><option>Armas</option><option>Armaduras</option><option>Escudos</option><option>Munições</option><option>Magias</option><option>Poderes</option></select>
        <div class="mesa-catalog">{#each catalog as item}<article><strong>{item.name}</strong><small>{item.group} · {item.summary}</small><p>{item.detail}</p></article>{:else}<p class="mesa-empty-panel">Nenhum resultado.</p>{/each}</div>
      </div>{/if}
      {#if tab === 'pessoas'}<div class="mesa-side-content">
        <h2>Participantes</h2>{#each $room.participants as person}<div class="mesa-person"><span>{person.name} {person.approved?'':'· aguardando'}</span>{#if $room.role === 'master'}<button onclick={() => sendRoom({type:person.approved?'revoke':'approve',participantId:person.id})}>{person.approved?'Revogar':'Aprovar'}</button>{/if}</div>{:else}<p class="mesa-empty-panel">Ninguém entrou ainda.</p>{/each}
        {#if $room.role === 'master' && $room.invite}<h3>Convite</h3><div class="mesa-side-invite"><span>{$room.invite.address}:{$room.invite.port}</span><code>{$room.invite.code}</code></div><button class="mesa-subtle" onclick={onConfig}>Área do mestre</button>{/if}
      </div>{/if}
    </aside>
  </div>
</div>
{#if $room.handout}<dialog class="mesa-handout-dialog" bind:this={handoutDialog} onclose={closeHandout} aria-label={`Handout ${$room.handout.title}`}><header><h2>{$room.handout.title}</h2><button aria-label="Fechar handout" onclick={()=>handoutDialog?.close()}><Icon name="close" size={18}/></button></header><div class="mesa-handout-body">{$room.handout.body}</div></dialog>{/if}
