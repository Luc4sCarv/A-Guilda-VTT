export const damageTypes = ['Ácido','Corte','Eletricidade','Essência','Fogo','Frio','Impacto','Luz','Perfuração','Psíquico','Trevas'] as const;
export const sizes = ['Minúsculo','Pequeno','Médio','Grande','Enorme','Colossal'] as const;
export type Size = typeof sizes[number];
export type Item = {name:string; itemType?:string; quantity?:number; spaces?:number; equipped?:boolean; defense?:number; penalty?:number; heavy?:boolean; material?:string; category?:string};
export type Condition = {name:string; note:string; group?:'cansaço'|'metabolismo'|'veneno'; imposes?:string[]; skill?:number; physical?:number; mental?:number; attack?:number; meleeAttack?:number; defense?:number; reflex?:number; initiative?:number; perception?:number; movement?:'zero'|'half'|'prone'; rd?:number};

// Apêndice do Livro Básico JdA, pp. 394–395; texto resumido, sem reproduzir o livro.
export const conditions: Condition[] = [
  {name:'Abalado',skill:-2,note:'Medo; uma segunda aplicação torna apavorado.'},
  {name:'Agarrado',imposes:['Desprevenido','Imóvel'],attack:-2,note:'Só ataca com armas leves.'},
  {name:'Alquebrado',note:'Habilidades custam +1 PM.'},
  {name:'Apavorado',skill:-5,note:'Não se aproxima da fonte do medo.'},
  {name:'Atordoado',imposes:['Desprevenido'],note:'Não pode realizar ações.'},
  {name:'Caído',meleeAttack:-5,movement:'prone',note:'Defesa −5 contra corpo a corpo e +5 contra ataques à distância.'},
  {name:'Cego',imposes:['Desprevenido','Lento'],physical:-5,note:'Não observa; alvos têm camuflagem total.'},
  {name:'Confuso',note:'Comportamento decidido por 1d6 a cada turno.'},
  {name:'Debilitado',physical:-5,note:'Nova aplicação torna inconsciente.'},
  {name:'Desprevenido',defense:-5,reflex:-5,note:'−5 na Defesa e em Reflexos.'},
  {name:'Doente',group:'metabolismo',note:'Efeito depende da doença.'},
  {name:'Em Chamas',note:'1d6 dano de fogo no início de cada turno; aplicar manualmente.'},
  {name:'Enfeitiçado',note:'Fonte recebe +10 em Diplomacia contra você.'},
  {name:'Enjoado',group:'metabolismo',note:'Só uma ação padrão ou de movimento por rodada.'},
  {name:'Enredado',imposes:['Lento','Vulnerável'],attack:-2,note:'Movimento restringido.'},
  {name:'Envenenado',group:'veneno',note:'Efeito depende do veneno; perdas recorrentes acumulam.'},
  {name:'Esmorecido',mental:-5,note:'−5 em testes mentais.'},
  {name:'Exausto',group:'cansaço',imposes:['Debilitado','Lento','Vulnerável'],note:'Nova aplicação torna inconsciente.'},
  {name:'Fascinado',perception:-5,note:'Só pode observar o objeto da fascinação.'},
  {name:'Fatigado',group:'cansaço',imposes:['Fraco','Vulnerável'],note:'Nova aplicação torna exausto.'},
  {name:'Fraco',physical:-2,note:'Nova aplicação torna debilitado.'},
  {name:'Frustrado',mental:-2,note:'Nova aplicação torna esmorecido.'},
  {name:'Imóvel',movement:'zero',note:'Todos os deslocamentos ficam em 0 m.'},
  {name:'Inconsciente',imposes:['Indefeso'],note:'Não pode agir, inclusive reagir.'},
  {name:'Indefeso',defense:-10,note:'Falha automaticamente em Reflexos.'},
  {name:'Lento',movement:'half',note:'Deslocamentos pela metade; não corre nem investe.'},
  {name:'Ofuscado',attack:-2,perception:-2,note:'Sentidos prejudicados.'},
  {name:'Paralisado',imposes:['Imóvel','Indefeso'],note:'Só ações puramente mentais.'},
  {name:'Pasmo',note:'Não pode realizar ações.'},
  {name:'Petrificado',imposes:['Inconsciente'],rd:8,note:'Recebe RD 8.'},
  {name:'Sangrando',group:'metabolismo',note:'Teste de Constituição CD 15 por turno; falha perde 1d6 PV, sem RD.'},
  {name:'Sobrecarregado',movement:'prone',note:'Penalidade de armadura −5 e deslocamento −3 m.'},
  {name:'Surdo',initiative:-5,note:'Não ouve; conjuração em condição ruim.'},
  {name:'Surpreendido',imposes:['Desprevenido'],note:'Não age na primeira rodada.'},
  {name:'Vulnerável',defense:-2,note:'Defesa −2.'},
];
export const conditionByName = Object.fromEntries(conditions.map(c=>[c.name,c])) as Record<string,Condition>;
export function activeConditions(names:string[], ancestry:string):Condition[] {
  const seen=new Set<string>();
  const add=(name:string)=>{const c=conditionByName[name];if(!c||seen.has(name)||(ancestry==='Golem'&&c.group))return;seen.add(name);c.imposes?.forEach(add);};
  names.forEach(add);
  return [...seen].map(name=>conditionByName[name]);
}
export function conditionPenalty(active:Condition[], key:'skill'|'physical'|'mental'|'attack'|'meleeAttack'|'defense'|'reflex'|'initiative'|'perception'):number {
  return Math.min(0,...active.map(c=>c[key]??0));
}
export function carryingLimit(strength:number):number {return Math.max(0,10+(strength>=0?2*strength:strength));}
export function itemSpaces(items:Item[],coins:{to:number;ts:number;tc:number}):number {
  return items.reduce((n,item)=>n+Math.max(0,Number(item.quantity??1))*Math.max(0,Number(item.spaces??0)),0)+Math.ceil((coins.to+coins.ts+coins.tc)/1000);
}
export function sizeModifiers(size:Size){return ({'Minúsculo':[5,-5,-1],'Pequeno':[2,-2,0],'Médio':[0,0,0],'Grande':[-2,2,1],'Enorme':[-5,5,1],'Colossal':[-10,10,2]} as const)[size];}
// Tabela 3-2, Livro Básico JdA, p. 143. Cada linha é −2, −1, normal, +1, +2.
const damageSteps:Record<string,string[]>={
  '1d2':['1','1','1d2','1d3','1d4'],
  '1d3':['1','1d2','1d3','1d4','1d6'],
  '1d4':['1d2','1d3','1d4','1d6','1d8'],
  '1d6':['1d3','1d4','1d6','1d8','1d10'],
  '1d8':['1d4','1d6','1d8','1d10','1d12'],
  '2d4':['1d4','1d6','2d4','1d10','1d12'],
  '1d10':['1d6','1d8','1d10','1d12','3d6'],
  '1d12':['1d8','1d10','1d12','3d6','4d6'],
  '2d6':['1d8','1d10','2d6','3d6','4d6'],
  '3d4':['1d8','1d10','3d4','3d6','4d6'],
  '2d8':['1d10','2d6','2d8','3d8','4d8'],
  '2d10':['2d6','2d8','2d10','3d10','4d10']
};
export function resizeWeaponDamage(damage:string,steps:number):string {return damageSteps[damage.trim()]?.[Math.max(0,Math.min(4,2+steps))]??damage;}
export const classProficiencies:Record<string,string[]>={Arcanista:[],Bárbaro:['Armas marciais','Escudos'],Bardo:['Armas marciais'],Bucaneiro:['Armas marciais'],Caçador:['Armas marciais','Escudos'],Cavaleiro:['Armas marciais','Armaduras pesadas','Escudos'],Clérigo:['Armaduras pesadas','Escudos'],Druida:['Escudos'],Guerreiro:['Armas marciais','Armaduras pesadas','Escudos'],Inventor:[],Ladino:[],Lutador:[],Nobre:['Armas marciais','Armaduras pesadas','Escudos'],Paladino:['Armas marciais','Armaduras pesadas','Escudos']};
export const proficiencyOptions=['Armas simples','Armas marciais','Armas marciais discretas','Armas exóticas','Armas de fogo','Armaduras leves','Armaduras pesadas','Escudos'];
export const ancestrySizes:Record<string,Size>={Goblin:'Pequeno',Hynne:'Pequeno','Sílfide':'Minúsculo'};
export function spellBaseCost(circle:number):number {return [0,1,3,6,10,15][circle]??0;}

export function spendMana(current:number, temporary:number, cost:number, maximum:number): {current:number;temporary:number}|null {
  if ([current,temporary,cost,maximum].some(n=>!Number.isSafeInteger(n)||n<0) || current>maximum || current+temporary<cost) return null;
  const fromTemporary=Math.min(temporary,cost);
  return {current:current-(cost-fromTemporary),temporary:temporary-fromTemporary};
}
