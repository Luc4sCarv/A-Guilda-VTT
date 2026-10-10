<script lang="ts">
  import { onMount } from 'svelte';
  import AssetThumb from './AssetThumb.svelte';
  import { room, assetBlobUrl, sendRoom, sharePersonalAsset, type RoomAudio } from '../mesa';
  import { addPersonalAsset, listPersonalAssets, type PersonalAsset } from '../personal-assets';

  let { kind, needsAudioActivation = false, onActivate = () => {}, volume = 1, onVolume = () => {} }: {
    kind:'art'|'audio'; needsAudioActivation?:boolean; onActivate?:()=>void; volume?:number; onVolume?:(value:number)=>void;
  }=$props();
  let local=$state<PersonalAsset[]>([]), urls=$state<Record<string,string>>({}), shared=$state<Record<string,string>>({});
  let query=$state(''), busy=$state(''), error=$state(''), preview=$state(''), previewName=$state(''), previewOwned=false;
  let fileInput:HTMLInputElement;
  let filtered=$derived(local.filter(item=>item.kind===kind&&item.name.toLocaleLowerCase('pt-BR').includes(query.toLocaleLowerCase('pt-BR'))));
  let library=$derived(kind==='art'?$room.roomArt:$room.roomAudio);
  const shareKey=()=>`tormenta-vtt-shared:${$room.room?.id||''}`;
  function loadShared(){try{shared=JSON.parse(localStorage.getItem(shareKey())||'{}');}catch{shared={};}}
  async function loadLocal(){const items=await listPersonalAssets();for(const url of Object.values(urls))URL.revokeObjectURL(url);local=items;urls=Object.fromEntries(items.filter(i=>i.kind==='art').map(i=>[i.id,URL.createObjectURL(i.file)]));}
  onMount(()=>{loadShared();void loadLocal().catch(e=>error=String(e));return()=>{for(const url of Object.values(urls))URL.revokeObjectURL(url);if(previewOwned&&preview)URL.revokeObjectURL(preview);};});
  async function importFiles(event:Event){
    const files=Array.from((event.currentTarget as HTMLInputElement).files||[]);
    try{for(const file of files)await addPersonalAsset(file,kind);await loadLocal();}catch(e){error=String(e);}
    (event.currentTarget as HTMLInputElement).value='';
  }
  async function share(item:PersonalAsset){
    busy=item.id;error='';
    try{const uploaded=await sharePersonalAsset(kind,item.name,item.file);shared={...shared,[item.id]:uploaded.id};localStorage.setItem(shareKey(),JSON.stringify(shared));}
    catch(e){error=String(e);}finally{busy='';}
  }
  function isShared(item:PersonalAsset){return library.some(a=>a.id===shared[item.id]);}
  function drag(event:DragEvent,id:string){event.dataTransfer?.setData('application/x-tormenta-token',JSON.stringify({artId:id}));if(event.dataTransfer)event.dataTransfer.effectAllowed='copy';}
  function place(id:string){const map=$room.activeMap;if(map)sendRoom({type:'place_token',artId:id,x:map.width/2,y:map.height/2});}
  async function openPreview(id:string,name:string){
    if(previewOwned&&preview)URL.revokeObjectURL(preview);
    preview='';previewName=name;previewOwned=false;
    try{preview=await assetBlobUrl(id);previewOwned=true;}catch(e){error=String(e);}
  }
  function closePreview(){if(previewOwned&&preview)URL.revokeObjectURL(preview);preview='';previewOwned=false;}
  function updateQueue(ids:string[]){sendRoom({type:'set_playlist',ids});}
  function moveQueue(id:string,step:number){const ids=[...$room.playlist],from=ids.indexOf(id),to=from+step;if(from<0||to<0||to>=ids.length)return;[ids[from],ids[to]]=[ids[to],ids[from]];updateQueue(ids);}
  function track(id:string):RoomAudio|undefined{return $room.roomAudio.find(item=>item.id===id);}
</script>

<div class="mesa-side-content mesa-library">
  <h2>{kind==='art'?'Artes':'Música'}</h2>
  <p class="mesa-side-hint">Sua biblioteca fica neste computador. Compartilhe apenas os arquivos que quiser usar nesta mesa.</p>
  {#if error}<p class="mesa-side-notice" role="alert">{error}</p>{/if}
  <input type="search" bind:value={query} placeholder={kind==='art'?'Buscar arte':'Buscar música'} aria-label="Buscar biblioteca"/>
  <input class="mesa-file-input" type="file" bind:this={fileInput} accept={kind==='art'?'.png,.jpg,.jpeg,.webp':'.mp3,.ogg,.wav'} multiple onchange={importFiles}/>
  <button class="mesa-subtle" onclick={()=>fileInput?.click()}>+ Importar {kind==='art'?'artes':'músicas'}</button>
  <h3>Minha biblioteca</h3>
  {#each filtered as item (item.id)}<div class="mesa-library-row">
    {#if kind==='art'}<button class="mesa-library-thumb" aria-label={`Prévia de ${item.name}`} onclick={()=>{closePreview();preview=urls[item.id];previewName=item.name;}}><img src={urls[item.id]} alt=""/></button>{/if}
    <strong title={item.name}>{item.name}</strong>
    {#if isShared(item)}<small>Compartilhado</small>{:else}<button disabled={busy===item.id||$room.status!=='ready'} onclick={()=>share(item)}>{busy===item.id?'Enviando…':'Compartilhar'}</button>{/if}
  </div>{:else}<p class="mesa-empty-panel">Nenhum arquivo pessoal.</p>{/each}
  <h3>{kind==='art'?'Artes desta mesa':'Faixas desta mesa'}</h3>
  {#if kind==='art'}
    {#each [...new Set($room.roomArt.map(a=>a.ownerName||'Mestre'))] as owner}<h4>{owner}</h4>
      {#each $room.roomArt.filter(a=>(a.ownerName||'Mestre')===owner) as art (art.id)}<div class="mesa-library-row" role="group" aria-label={`Arte ${art.name}`} draggable={$room.role==='master'||art.ownerParticipantId===$room.participantId} ondragstart={e=>drag(e,art.id)}>
        <button class="mesa-library-thumb" aria-label={`Prévia de ${art.name}`} onclick={()=>openPreview(art.id,art.name)}><AssetThumb id={art.id} alt=""/></button><strong title={art.name}>{art.name}</strong>
        {#if $room.role==='master'||art.ownerParticipantId===$room.participantId}<button disabled={!$room.activeMap} title="Adicionar ao centro" aria-label={`Adicionar ${art.name} ao centro`} onclick={()=>place(art.id)}>+</button>{/if}
      </div>{/each}
    {:else}<p class="mesa-empty-panel">Nenhuma arte compartilhada.</p>{/each}
  {:else}
    {#each library as audio (audio.id)}<div class="mesa-library-row"><strong title={audio.name}>{audio.name}</strong><small>{audio.ownerName}</small>{#if $room.role==='master' && !$room.playlist.includes(audio.id)}<button aria-label={`Adicionar ${audio.name} à fila`} onclick={()=>updateQueue([...$room.playlist,audio.id])}>+</button>{/if}</div>{:else}<p class="mesa-empty-panel">Nenhuma música compartilhada.</p>{/each}
    <h3>Fila da mesa</h3>
    {#each $room.playlist as id,index (id)}<div class="mesa-library-row"><strong>{index+1}. {track(id)?.name||'Faixa'}</strong>{#if $room.role==='master'}<button aria-label="Tocar esta faixa" onclick={()=>sendRoom({type:'playback_control',action:'play',audioId:id})}>▶</button><button aria-label="Subir na fila" disabled={index===0} onclick={()=>moveQueue(id,-1)}>↑</button><button aria-label="Descer na fila" disabled={index===$room.playlist.length-1} onclick={()=>moveQueue(id,1)}>↓</button><button aria-label="Retirar da fila" onclick={()=>updateQueue($room.playlist.filter(v=>v!==id))}>×</button>{/if}</div>{:else}<p class="mesa-empty-panel">Fila vazia.</p>{/each}
    <div class="mesa-music-controls"><strong>Tocando: {track($room.playback.audioId||'')?.name||'Nada'}</strong>
      {#if $room.role==='master'}<div><button onclick={()=>sendRoom({type:'playback_control',action:$room.playback.playing?'pause':'resume'})} disabled={!$room.playback.audioId}>{$room.playback.playing?'Pausar':'Continuar'}</button><button onclick={()=>sendRoom({type:'playback_control',action:'next'})} disabled={!$room.playlist.length}>Próxima</button><button onclick={()=>sendRoom({type:'playback_control',action:'stop'})} disabled={!$room.playback.audioId}>Parar</button></div>{/if}
      <label>Volume local <input type="range" min="0" max="1" step="0.01" value={volume} oninput={e=>onVolume(Number(e.currentTarget.value))}/></label><button onclick={()=>onVolume(volume>0?0:1)}>{volume>0?'Silenciar':'Ativar som'}</button>
      {#if needsAudioActivation}<button class="primary" onclick={onActivate}>Ativar áudio</button>{/if}
    </div>
  {/if}
</div>

{#if preview}<div class="mesa-preview-overlay"><button class="mesa-preview-backdrop" aria-label="Fechar prévia" onclick={closePreview}></button><div class="mesa-preview" role="dialog" aria-modal="true" aria-label={`Prévia de ${previewName}`} tabindex="-1"><header><strong>{previewName}</strong><button aria-label="Fechar prévia" onclick={closePreview}>×</button></header><img src={preview} alt={previewName}/></div></div>{/if}
