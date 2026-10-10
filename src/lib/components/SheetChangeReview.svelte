<script lang="ts">
  import {room,sendRoom} from '../mesa';
  const labels:Record<string,string>={baseAttributes:'Atributos base (FOR, DES, CON, INT, SAB, CAR)',temporaryAttributes:'Ajustes temporários',trainedSkills:'Perícias treinadas',skillAttributes:'Atributos das perícias',effects:'Efeitos',powers:'Poderes',spells:'Magias',equipment:'Equipamento',attacks:'Ataques',level:'Nível',characterClass:'Classe',ancestry:'Raça',ancestryChoices:'Escolhas raciais',origin:'Origem',originBenefits:'Benefícios da origem',deity:'Divindade',deityPower:'Poder concedido',currentHp:'PV atuais',maximumHp:'PV máximos',hpAdjustment:'Ajuste de PV',currentMp:'PM atuais',temporaryMp:'PM temporários',temporaryHp:'PV temporários',defenseBase:'Defesa base',defenseManual:'Ajuste de Defesa',manualRd:'Redução de dano',coins:'Moedas',activeConditionNames:'Condições',name:'Nome',description:'Descrição',cost:'Custo',value:'Valor',source:'Fonte',active:'Ativo',minimumLevel:'Nível mínimo',circle:'Círculo',quantity:'Quantidade',spaces:'Espaços',equipped:'Equipado',kind:'Tipo',attribute:'Atributo',skill:'Perícia'};
  function describe(value:unknown):string {
    if(value===undefined||value===null)return 'Não informado';
    if(typeof value==='boolean')return value?'Sim':'Não';
    if(Array.isArray(value))return value.length?value.map(describe).join('\n'):'Nenhum';
    if(typeof value==='object')return Object.entries(value).filter(([key])=>key!=='open').map(([key,v])=>`${labels[key]??key}: ${describe(v)}`).join('\n');
    return String(value);
  }
</script>

{#if $room.role==='master' && $room.sheetRequests.length}
  <section aria-label="Alterações de ficha para revisão">
    <h3>Alterações para revisar ({$room.sheetRequests.length})</h3>
    <p>Confira fontes, pré-requisitos e custos. Aprovar autoriza o uso na mesa; não certifica todas as regras automaticamente.</p>
    {#each $room.sheetRequests as request (request.id)}
      {@const stale=$room.characters.find(s=>s.id===request.sheetId)?.revision!==request.baseRevision}
      <details>
        <summary>{request.participantName} · {request.data.name} · {request.fields.length} alterações</summary>
        {#each request.fields as field}
          <h4>{labels[field]??field}</h4>
          <div class="comparison"><div><strong>Em jogo</strong><pre>{describe(request.previous[field])}</pre></div><div><strong>Proposto</strong><pre>{describe(request.data[field])}</pre></div></div>
        {/each}
        {#if stale}<p role="status">A ficha mudou. Recuse este pedido e peça uma nova revisão.</p>{/if}
        <div class="review-actions"><button disabled={stale} onclick={()=>sendRoom({type:'review_sheet_change',requestId:request.id,accept:true})}>Aprovar alterações</button><button onclick={()=>sendRoom({type:'review_sheet_change',requestId:request.id,accept:false})}>Recusar</button></div>
      </details>
    {/each}
  </section>
{/if}

<style>
  section,details {min-width:0;overflow-wrap:anywhere;}
  details {margin-block:1rem;}
  summary {cursor:pointer;}
  h4 {margin-block:1.25rem .5rem;}
  .comparison {display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:1rem;}
  pre {font:inherit;white-space:pre-wrap;overflow-wrap:anywhere;max-height:20rem;overflow:auto;}
  .review-actions {display:flex;gap:.5rem;flex-wrap:wrap;margin-block:1rem;}
  @media(max-width:600px){.comparison{grid-template-columns:1fr;}}
</style>
