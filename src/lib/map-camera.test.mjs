import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { readFile } from 'node:fs/promises';
import { compile } from 'svelte/compiler';
import { effect_root, state, get, set } from 'svelte/internal/client';
import { flushSync } from 'svelte';

test('selecionar e desmarcar token preserva zoom e permite mover a câmera', async () => {
  // Exercise the real component script without mounting a window or a GPU renderer.
  const component = await readFile(new URL('./components/MapCanvas.svelte', import.meta.url), 'utf8');
  let script = component.match(/<script lang="ts">([\s\S]*?)<\/script>/)[1]
    .replace("import { onMount, untrack } from 'svelte';", "import { untrack } from 'svelte'; const onMount = () => {};")
    .replace(/import \{ assetBlobUrl,[^\n]+/, `
      import { writable } from 'svelte/store';
      import { Texture as TestTexture } from 'pixi.js';
      const room = writable({role:'master', token:'master', measure:null, marks:[]});
      const sendRoom = () => true;
      const assetBlobUrl = async () => { throw new Error('Unexpected image request'); };
      const requestAnimationFrame = () => 1;
    `)
    .replace("from '../fog';", `from '${new URL('./fog.ts', import.meta.url).href}';`);
  script += `
    export function initializeTest() {
      host = {clientWidth:1000,clientHeight:700,getBoundingClientRect:()=>({left:0,top:0}),setPointerCapture:()=>{},hasPointerCapture:()=>false} as unknown as HTMLDivElement;
      app={renderer:{background:{color:0}}} as unknown as Application;
      scene=new Container(); backgroundLayer=new Container(); tokenLayer=new Container(); markLayer=new Container(); overlayLayer=new Container(); fogLayer=new Container(); fogGuideLayer=new Container();
      textures.set(map.id,TestTexture.EMPTY); fitMap(); draw();
    }
    export function cameraTest() { return {scale,offsetX,offsetY}; }
    export function tokenPositionTest(id:string) { const node=tokenViews.get(id)?.node; return node&&{x:node.position.x,y:node.position.y}; }
    export function zoomTest() { zoom(1.5,500,350); }
    export function pointerTest(kind:string,x:number,y:number) {
      const event={clientX:x,clientY:y,button:0,pointerId:1,target:{closest:()=>null}} as unknown as PointerEvent;
      if(kind==='down')down(event); else if(kind==='move')move(event); else up(event);
    }
    export function contextTest(x:number,y:number) {
      const event={clientX:x,clientY:y,button:2,pointerId:2,target:{closest:()=>null},preventDefault:()=>{}} as unknown as PointerEvent;
      down(event); contextMenu(event);
    }
    export function rightDragTest(x:number,y:number) {
      const event={clientX:x,clientY:y,button:2,buttons:2,pointerId:3,target:{closest:()=>null},preventDefault:()=>{}} as unknown as PointerEvent;
      down(event);move({...event,clientX:x+30,clientY:y+20});up({...event,clientX:x+30,clientY:y+20});contextMenu({...event,clientX:x+30,clientY:y+20});
    }
  `;
  const compiled = compile(`<script lang="ts">${script}</script>`, {generate:'client', filename:'MapCameraTest.svelte'}).js.code;
  const resolved = compiled.replace(/(from\s+|import\s+)(['"])(svelte[^'"]*|pixi\.js)\2/g,
    (_, prefix, quote, specifier) => `${prefix}${quote}${import.meta.resolve(specifier)}${quote}`);
  const { default: Canvas } = await import(`data:text/javascript;base64,${Buffer.from(resolved).toString('base64')}`);
  const selection = state(null);
  const mapState = state({id:'map',name:'Mapa',width:1200,height:800,gridVisible:false,cellPx:64,metresPerCell:1.5});
  const tokenState = state([{id:'token',name:'AB',artId:null,x:600,y:400,size:64,revision:1}]);
  const menus=[],mapMenus=[];
  const props = {
    get map(){return get(mapState);},
    get tokens(){return get(tokenState);},
    characters:[],tool:'select',fogMode:'reveal',fogPreview:false,measureShape:'line',duration:'instant',shareMeasure:false,
    get selectedId(){return get(selection);}, onSelect:id=>set(selection,id), onTokenMenu:(...args)=>menus.push(args),onMapMenu:(...args)=>mapMenus.push(args),
  };
  let canvas;
  const dispose = effect_root(() => { canvas=Canvas(null,props); canvas.initializeTest(); });
  try {
    flushSync();
    canvas.zoomTest(); flushSync();
    const zoomed=canvas.cameraTest();
    assert.equal(zoomed.scale,(952/1200)*1.5);
    const x=zoomed.offsetX+600*zoomed.scale, y=zoomed.offsetY+400*zoomed.scale;
    canvas.pointerTest('down',x,y); flushSync(); canvas.pointerTest('up',x,y); flushSync();
    assert.equal(get(selection),'token');
    assert.deepEqual(canvas.cameraTest(),zoomed,'seleção não deve reenquadrar');
    canvas.pointerTest('down',20,20); flushSync();
    assert.equal(get(selection),null);
    assert.deepEqual(canvas.cameraTest(),zoomed,'desmarcar não deve reenquadrar');
    canvas.pointerTest('move',100,70); flushSync(); canvas.pointerTest('up',100,70); flushSync();
    assert.deepEqual(canvas.cameraTest(),{scale:zoomed.scale,offsetX:zoomed.offsetX+80,offsetY:zoomed.offsetY+50});
    canvas.zoomTest(); flushSync();
    assert.equal(canvas.cameraTest().scale,zoomed.scale*1.5,'zoom deve continuar funcionando');
    const camera=canvas.cameraTest();
    canvas.contextTest(camera.offsetX+600*camera.scale,camera.offsetY+400*camera.scale); flushSync();
    assert.equal(menus.length,1,'clique direito no token abre as configurações');
    assert.equal(menus[0][0],'token');
    canvas.contextTest(5,5); flushSync();
    assert.equal(menus.length,1,'clique direito fora do token não abre o menu');
    const previousMapMenus=mapMenus.length;
    const blankX=camera.offsetX+300*camera.scale,blankY=camera.offsetY+300*camera.scale;
    canvas.contextTest(blankX,blankY); flushSync();
    assert.equal(mapMenus.length,previousMapMenus+1,'clique direito no mapa abre ações do mapa');
    assert.deepEqual(canvas.cameraTest(),camera,'menu de contexto não reenquadra o mapa');
    canvas.pointerTest('up',5,5); flushSync();
    const before=canvas.cameraTest();
    const fromX=before.offsetX+600*before.scale, fromY=before.offsetY+400*before.scale;
    canvas.pointerTest('down',fromX,fromY); flushSync();
    canvas.pointerTest('move',fromX+60,fromY+30); flushSync();
    const moved={x:600+60/before.scale,y:400+30/before.scale};
    assert.deepEqual(canvas.tokenPositionTest('token'),moved,'token deve acompanhar o ponteiro no mesmo evento');
    assert.deepEqual(canvas.cameraTest(),before,'arrastar token não reenquadra o mapa');
    canvas.pointerTest('up',fromX+60,fromY+30); flushSync();
    assert.deepEqual(canvas.tokenPositionTest('token'),moved,'token não deve voltar à posição antiga ao soltar');
    set(tokenState,[{...get(tokenState)[0],...moved,revision:2}]); flushSync();
    assert.deepEqual(canvas.tokenPositionTest('token'),moved);
    set(mapState,{...get(mapState)}); flushSync();
    assert.deepEqual(canvas.cameraTest(),before,'atualização da mesa não deve ajustar o zoom');
    const menuCount=mapMenus.length;
    canvas.rightDragTest(before.offsetX+200*before.scale,before.offsetY+200*before.scale);flushSync();
    assert.equal(mapMenus.length,menuCount,'arrastar com botão direito não abre menu');
    assert.deepEqual(canvas.cameraTest(),{scale:before.scale,offsetX:before.offsetX+30,offsetY:before.offsetY+20});
  } finally { dispose(); }
});
