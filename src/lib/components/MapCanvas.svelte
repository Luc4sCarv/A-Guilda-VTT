<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { Application, Assets, Container, Graphics, Sprite, Text, Texture, TilingSprite } from 'pixi.js';
  import { assetBlobUrl, room, sendRoom, type PlacedToken, type RoomMap, type RoomCharacter, type RoomMark, type MapPoint } from '../mesa';
  import { isFogCovered, type FogOp } from '../fog';

  let { map, tokens, characters, tool, fogMode, fogPreview, fogShape, fogOpacity, fogFinishSignal, fogCancelSignal, clearLocalMeasures, measureShape, measureSnap, trackTokenMove, duration, shareMeasure, drawShape, drawStroke, drawFill, drawWidth, drawBlend, selectedId, onSelect, onPlace, onPinOpen, onTokenMenu, onMapMenu }: {
    map:RoomMap; tokens:PlacedToken[]; characters:RoomCharacter[]; tool:'select'|'pan'|'draw'|'text'|'measure'|'pin'|'fog';fogMode:'reveal'|'hide';fogPreview:boolean;fogShape:'rect'|'polygon';fogOpacity:number;fogFinishSignal:number;fogCancelSignal:number;clearLocalMeasures:number;
    measureShape:'line'|'corners';measureSnap:boolean;trackTokenMove:boolean; duration:'instant'|'permanent'; shareMeasure:boolean;
    drawShape:'freehand'|'line'|'rect';drawStroke:string;drawFill:string;drawWidth:number;drawBlend:'normal'|'multiply';
    selectedId:string|null; onSelect:(id:string|null)=>void;onPlace:(kind:'text'|'pin',x:number,y:number)=>void;onPinOpen:(mark:RoomMark)=>void; onTokenMenu:(id:string,x:number,y:number)=>void; onMapMenu:(x:number,y:number,mapX:number,mapY:number)=>void;
  } = $props();
  let host:HTMLDivElement;
  let app:Application|null=null, scene:Container|null=null;
  let backgroundLayer:Container|null=null, tokenLayer:Container|null=null, markLayer:Container|null=null, overlayLayer:Container|null=null, fogLayer:Container|null=null, fogGuideLayer:Container|null=null;
  let backgroundLayout='', backgroundTexture:Texture|undefined;
  let fogLayout='';
  let fittedMap='';
  let drawFrame=0;
  const tokenViews=new Map<string,{node:Container;ring:Graphics;name:string;size:number;artId:string|null;texture?:Texture;selected:boolean|null;conditionKey:string}>();
  let scale=$state(1), offsetX=$state(0), offsetY=$state(0);
  let zoomLabel=$state('100%'), imageError=$state('');
  let gesture:{kind:'pan'|'token'|'measure'|'fog'|'draw';id?:string;revision?:number;startX:number;startY:number;offsetX:number;offsetY:number;worldX?:number;worldY?:number}|null=null;
  let preview:{id:string;x:number;y:number}|null=null;
  let waiting:{id:string;revision:number}|null=null;
  let pendingTimer:ReturnType<typeof setTimeout>|null=null;
  let line=$state<{x1:number;y1:number;x2:number;y2:number}|null>(null);
  let fogRect=$state<{x1:number;y1:number;x2:number;y2:number}|null>(null);
  let fogPolygon=$state<MapPoint[]>([]);
  let drawing=$state<MapPoint[]>([]);
  let pendingDrawing:{mapId:string;points:MapPoint[];shape:'freehand'|'line'|'rect';stroke:string;fill:string;width:number;blend:'normal'|'multiply'}|null=null;
  let localMeasures=$state<{points:MapPoint[];shape:'line'|'corners';metres:number}[]>([]);
  let localMapId='';
  let fogTexture:Texture|null=null;
  let fogMaskTexture:Texture|null=null;
  let lineTimer:ReturnType<typeof setTimeout>|null=null;
  let rightDragged=false;
  let visiblePing=$state<{x:number;y:number;author:string;serial:number}|null>(null);
  let pingTimer:ReturnType<typeof setTimeout>|null=null;
  const textures=new Map<string,Texture>();
  const urls=new Map<string,string>();
  const loading=new Set<string>();
  const failedImages=new Map<string,number>();
  const backdropColors=new Map<string,number>();
  let canvasAccent=0xe7bb72, canvasMeasure=0xe1ab62, canvasShared=0xb28cff;
  function updateCanvasTheme(){
    const styles=getComputedStyle(host);
    const read=(name:string,fallback:number)=>{const value=styles.getPropertyValue(name).trim();return /^#[0-9a-f]{6}$/i.test(value)?Number.parseInt(value.slice(1),16):fallback;};
    canvasAccent=read('--map-selection',0xe7bb72);
    canvasMeasure=read('--map-measure',0xe1ab62);
    canvasShared=read('--map-shared',0xb28cff);
    for(const visual of tokenViews.values())visual.selected=null;
    scheduleDraw();
  }

  function cloudTexture():Texture{
    const canvas=document.createElement('canvas');canvas.width=canvas.height=256;
    const context=canvas.getContext('2d')!;
    const pixels=context.createImageData(256,256);
    for(let y=0;y<256;y++)for(let x=0;x<256;x++){
      const angleX=x*Math.PI/128,angleY=y*Math.PI/128;
      const mist=Math.sin(angleX+Math.sin(angleY)*1.8)*9+Math.cos(angleY*2-angleX)*7+Math.sin(angleX*3+angleY*2)*4;
      const i=(y*256+x)*4;
      pixels.data[i]=Math.round(91+mist);pixels.data[i+1]=Math.round(111+mist);pixels.data[i+2]=Math.round(128+mist);pixels.data[i+3]=255;
    }
    context.putImageData(pixels,0,0);
    return Texture.from(canvas);
  }

  async function sampleBackdrop(url:string,id:string){
    try{
      const bitmap=await createImageBitmap(await (await fetch(url)).blob());
      const canvas=document.createElement('canvas');canvas.width=canvas.height=64;
      const context=canvas.getContext('2d',{willReadFrequently:true})!;
      context.drawImage(bitmap,0,0,64,64);bitmap.close();
      const data=context.getImageData(0,0,64,64).data;
      const bins=new Map<number,{count:number;r:number;g:number;b:number}>();
      for(let i=0;i<data.length;i+=4){if(data[i+3]<128)continue;
        const key=(data[i]>>4)<<8|(data[i+1]>>4)<<4|(data[i+2]>>4);
        const bin=bins.get(key)||{count:0,r:0,g:0,b:0};bin.count++;bin.r+=data[i];bin.g+=data[i+1];bin.b+=data[i+2];bins.set(key,bin);
      }
      const winner=[...bins.values()].sort((a,b)=>b.count-a.count)[0];
      if(winner){backdropColors.set(id,(Math.round(winner.r/winner.count)<<16)|(Math.round(winner.g/winner.count)<<8)|Math.round(winner.b/winner.count));scheduleDraw();}
    }catch{ /* A cor neutra permanece se a amostra falhar. */ }
  }

  function view(){if(scene){scene.scale.set(scale);scene.position.set(offsetX,offsetY);}zoomLabel=`${Math.round(scale*100)}%`;}
  function fit(){if(!host||!map)return;scale=Math.min((host.clientWidth-48)/map.width,(host.clientHeight-48)/map.height,2);scale=Math.max(0.05,scale);offsetX=(host.clientWidth-map.width*scale)/2;offsetY=(host.clientHeight-map.height*scale)/2;view();}
  function fitMap(){const key=`${map.id}:${map.width}:${map.height}`;if(key===fittedMap)return;fittedMap=key;fit();}
  function point(event:MouseEvent){const r=host.getBoundingClientRect();return {x:(event.clientX-r.left-offsetX)/scale,y:(event.clientY-r.top-offsetY)/scale};}
  function zoom(factor:number,clientX=host.getBoundingClientRect().left+host.clientWidth/2,clientY=host.getBoundingClientRect().top+host.clientHeight/2){
    const r=host.getBoundingClientRect(),x=clientX-r.left,y=clientY-r.top,old=scale;
    scale=Math.max(0.05,Math.min(4,scale*factor));offsetX=x-(x-offsetX)*scale/old;offsetY=y-(y-offsetY)*scale/old;view();
  }
  async function image(id:string){
    if(textures.has(id)||loading.has(id)||Date.now()-(failedImages.get(id)??0)<5000)return;
    loading.add(id);
    try{
      const url=await assetBlobUrl(id);
      urls.set(id,url);
      const texture=await Assets.load<Texture>({src:url,parser:'texture'});
      if(!texture)throw new Error('Não foi possível decodificar a imagem.');
      textures.set(id,texture);
      if(id===map.id)void sampleBackdrop(url,id);
      failedImages.delete(id);
      imageError='';draw();
    }
    catch(error){
      const url=urls.get(id);
      if(url){URL.revokeObjectURL(url);urls.delete(id);}
      failedImages.set(id,Date.now());
      imageError=String(error);
    }
    finally{loading.delete(id);}
  }
  function clear(layer:Container){layer.removeChildren().forEach(child=>child.destroy());}
  function path(graphics:Graphics,points:MapPoint[]){graphics.moveTo(points[0].x,points[0].y);for(const point of points.slice(1))graphics.lineTo(point.x,point.y);return graphics;}
  function rulerPath(graphics:Graphics,a:MapPoint,b:MapPoint,shape:'line'|'corners'){graphics.moveTo(a.x,a.y);if(shape==='corners')graphics.lineTo(b.x,a.y);return graphics.lineTo(b.x,b.y);}
  function drawMark(mark:RoomMark){
    if(!markLayer)return;
    const data=mark.data;
    if(mark.kind==='drawing'){
      const points=data.points as MapPoint[];
      if(!points?.length)return;
      const graphics=new Graphics();graphics.blendMode=data.blend==='multiply'?'multiply':'normal';
      if(data.shape==='rect'){
        const x=Math.min(points[0].x,points[1].x),y=Math.min(points[0].y,points[1].y);
        graphics.rect(x,y,Math.abs(points[1].x-points[0].x),Math.abs(points[1].y-points[0].y));
        if(data.fill!=='transparent')graphics.fill(data.fill);
      }else path(graphics,points);
      graphics.stroke({color:data.stroke,width:data.width,cap:'round',join:'round'});markLayer.addChild(graphics);
    }else if(mark.kind==='text'){
      const label=new Text({text:data.text,style:{fill:data.color,fontFamily:data.font,fontSize:data.size,fontWeight:data.bold?'bold':'normal',fontStyle:data.italic?'italic':'normal',wordWrap:true,wordWrapWidth:Math.max(80,map.width-data.x)}});
      label.position.set(data.x,data.y);markLayer.addChild(label);
    }else if(mark.kind==='pin'){
      const pin=new Graphics().circle(0,-9,10).fill({color:mark.visibility==='private'?0xb889e0:canvasAccent}).circle(0,-9,3).fill(0xffffff).moveTo(-7,-3).lineTo(0,10).lineTo(7,-3).fill({color:mark.visibility==='private'?0xb889e0:canvasAccent});
      pin.position.set(data.x,data.y);markLayer.addChild(pin);
    }else if(mark.kind==='measure'){
      const points=data.points as MapPoint[];const graphics=new Graphics();rulerPath(graphics,points[0],points[1],data.shape);graphics.stroke({color:canvasShared,width:3});markLayer.addChild(graphics);
      const label=new Text({text:`${data.metres} m`,style:{fill:'#ffffff',fontSize:15,stroke:{color:'#202733',width:3}}});label.position.set(points[1].x+5,points[1].y-20);markLayer.addChild(label);
    }
  }
  function buildFogMask(ops:FogOp[]):Texture{
    // shortcut: máscara de até 2048 px; aumentar resolução adaptativa se mapas enormes exigirem bordas mais finas.
    const ratio=Math.min(1,2048/Math.max(map.width,map.height));
    const canvas=document.createElement('canvas');canvas.width=Math.max(1,Math.round(map.width*ratio));canvas.height=Math.max(1,Math.round(map.height*ratio));
    const context=canvas.getContext('2d')!;context.scale(ratio,ratio);context.fillStyle='#ffffff';context.fillRect(0,0,map.width,map.height);
    for(const op of ops){
      context.globalCompositeOperation=op.mode==='reveal'?'destination-out':'source-over';
      context.beginPath();
      if(op.shape==='polygon'){context.moveTo(op.points[0].x,op.points[0].y);for(const point of op.points.slice(1))context.lineTo(point.x,point.y);context.closePath();context.fill('evenodd');}
      else context.rect(op.x,op.y,op.width,op.height);
      if(op.shape!=='polygon')context.fill();
    }
    return Texture.from(canvas);
  }
  function draw(){
    if(!scene||!backgroundLayer||!tokenLayer||!markLayer||!overlayLayer||!fogLayer||!fogGuideLayer)return;
    app!.renderer.background.color=map.backdropColor||backdropColors.get(map.id)||0x282924;
    const background=textures.get(map.id);
    const layout=`${map.id}:${map.width}:${map.height}:${map.gridVisible}:${map.cellPx}`;
    if(layout!==backgroundLayout||background!==backgroundTexture){
      clear(backgroundLayer);
      backgroundLayout=layout;backgroundTexture=background;
      if(background){const sprite=new Sprite(background);sprite.width=map.width;sprite.height=map.height;backgroundLayer.addChild(sprite);}
      else backgroundLayer.addChild(new Graphics().rect(0,0,map.width,map.height).fill(0x252721));
      if(map.gridVisible){
        const grid=new Graphics();
        for(let x=0;x<=map.width;x+=map.cellPx)grid.moveTo(x,0).lineTo(x,map.height);
        for(let y=0;y<=map.height;y+=map.cellPx)grid.moveTo(0,y).lineTo(map.width,y);
        grid.stroke({color:0xffffff,alpha:0.2,width:1});backgroundLayer.addChild(grid);
      }
    }
    if(map.kind==='tactical'&&!background)void image(map.id);
    const present=new Set(tokens.map(token=>token.id));
    for(const [id,visual] of tokenViews)if(!present.has(id)){tokenLayer.removeChild(visual.node);visual.node.destroy({children:true});tokenViews.delete(id);}
    for(const [index,token] of tokens.entries()){
      const x=preview?.id===token.id?preview.x:token.x,y=preview?.id===token.id?preview.y:token.y;
      const art=token.artId?textures.get(token.artId):undefined;
      if(token.artId&&!art)void image(token.artId);
      let visual=tokenViews.get(token.id);
      const conditionKey=token.conditions?.join(',')||'';
      if(visual&&(visual.name!==token.name||visual.size!==token.size||visual.artId!==token.artId||visual.texture!==art||visual.conditionKey!==conditionKey)){
        tokenLayer.removeChild(visual.node);visual.node.destroy({children:true});tokenViews.delete(token.id);visual=undefined;
      }
      if(!visual){
        const node=new Container();
        if(art){const sprite=new Sprite(art);sprite.anchor.set(0.5);sprite.width=token.size;sprite.height=token.size;node.addChild(sprite);}
        else {node.addChild(new Graphics().circle(0,0,token.size/2).fill(0x9a7134));const label=new Text({text:token.name.slice(0,2).toUpperCase(),style:{fill:'#ffffff',fontSize:Math.max(12,token.size/3),fontWeight:'bold'}});label.anchor.set(0.5);node.addChild(label);}
        if(token.conditions?.length){const radius=Math.max(10,Math.min(16,token.size/5));const badge=new Container();badge.position.set(token.size/2-radius/2,-token.size/2+radius/2);badge.addChild(new Graphics().circle(0,0,radius).fill(0x9f392f).stroke({color:0xffffff,width:2}));const count=new Text({text:String(token.conditions.length),style:{fill:'#ffffff',fontSize:radius*1.1,fontWeight:'bold'}});count.anchor.set(0.5);badge.addChild(count);node.addChild(badge);}
        const ring=new Graphics();node.addChild(ring);tokenLayer.addChild(node);
        visual={node,ring,name:token.name,size:token.size,artId:token.artId,texture:art,selected:null,conditionKey};tokenViews.set(token.id,visual);
      }
      const selected=token.id===selectedId;
      if(visual.selected!==selected){visual.ring.clear().circle(0,0,token.size/2).stroke({color:selected?canvasAccent:0x111111,width:selected?4:2});visual.selected=selected;}
      visual.node.position.set(x,y);
      if(tokenLayer.getChildIndex(visual.node)!==index)tokenLayer.setChildIndex(visual.node,index);
    }
    clear(markLayer);
    for(const mark of $room.marks)if(mark.mapId===map.id&&!(fogPreview&&mark.visibility==='private'))drawMark(mark);
    if(drawing.length>1&&(!pendingDrawing||pendingDrawing.mapId===map.id))drawMark({id:'preview',mapId:map.id,kind:'drawing',ownerParticipantId:'',author:'',visibility:'public',revision:0,data:{points:drawing,shape:drawShape,stroke:drawStroke,fill:drawFill,width:drawWidth,blend:drawBlend}});
    clear(overlayLayer);
    const shared=$room.measure;
    if(shared && shared.mapId===map.id && shared.token!==$room.token){const g=new Graphics();rulerPath(g,{x:shared.x1*map.width,y:shared.y1*map.height},{x:shared.x2*map.width,y:shared.y2*map.height},shared.shape||'line');g.stroke({color:canvasShared,width:3});overlayLayer.addChild(g);}
    if(line){const g=new Graphics();rulerPath(g,{x:line.x1,y:line.y1},{x:line.x2,y:line.y2},measureShape);g.stroke({color:canvasMeasure,width:3});overlayLayer.addChild(g);}
    for(const local of localMeasures){const g=new Graphics();rulerPath(g,local.points[0],local.points[1],local.shape);g.stroke({color:canvasMeasure,width:3});overlayLayer.addChild(g);}
    if(trackTokenMove&&gesture?.kind==='token'&&preview&&gesture.worldX!==undefined&&gesture.worldY!==undefined){overlayLayer.addChild(new Graphics().moveTo(gesture.worldX,gesture.worldY).lineTo(preview.x,preview.y).stroke({color:canvasMeasure,width:2}));}
    clear(fogGuideLayer);
    if(fogRect){const x=Math.min(fogRect.x1,fogRect.x2),y=Math.min(fogRect.y1,fogRect.y2),width=Math.abs(fogRect.x2-fogRect.x1),height=Math.abs(fogRect.y2-fogRect.y1);fogGuideLayer.addChild(new Graphics().rect(x,y,width,height).stroke({color:canvasAccent,width:3}));}
    if(fogPolygon.length){const guide=new Graphics();path(guide,fogPolygon);guide.stroke({color:canvasAccent,width:3});for(const point of fogPolygon)guide.circle(point.x,point.y,4).fill(canvasAccent);fogGuideLayer.addChild(guide);}
    const fog=$room.fog;
    const nextFogLayout=`${map.id}:${map.width}:${map.height}:${fog?.revision}:${fog?.enabled}:${$room.role}:${fogPreview}:${fogOpacity}`;
    if(fogLayout!==nextFogLayout){
      clear(fogLayer);fogMaskTexture?.destroy();fogMaskTexture=null;fogLayout=nextFogLayout;
      if(fog?.enabled&&fog.mapId===map.id&&fogTexture){
        fogMaskTexture=buildFogMask(fog.ops);
        const mask=new Sprite(fogMaskTexture);mask.width=map.width;mask.height=map.height;
        const cloud=new TilingSprite({texture:fogTexture,width:map.width,height:map.height});cloud.mask=mask;
        fogLayer.addChild(cloud,mask);fogLayer.alpha=$room.role==='master'&&!fogPreview?fogOpacity/100:1;
      }
    }
  }
  function fogHidden(x:number,y:number){
    const fog=$room.fog;if(!fog?.enabled||fog.mapId!==map.id||$room.role==='master'&&!fogPreview)return false;
    return isFogCovered(x,y,fog.ops);
  }
  function scheduleDraw(){if(drawFrame)return;drawFrame=requestAnimationFrame(()=>{drawFrame=0;draw();});}
  function clipped(point:MapPoint):MapPoint{return {x:Math.max(0,Math.min(map.width,point.x)),y:Math.max(0,Math.min(map.height,point.y))};}
  function measurePoint(point:MapPoint):MapPoint{const p=clipped(point);return measureSnap?{x:Math.min(map.width,Math.round(p.x/map.cellPx)*map.cellPx),y:Math.min(map.height,Math.round(p.y/map.cellPx)*map.cellPx)}:p;}
  function finishFogPolygon(){if(fogPolygon.length<3){room.update(s=>({...s,message:'Marque ao menos três pontos para concluir o polígono.'}));return;}if(sendRoom({type:'fog_set',action:'polygon',mode:fogMode,points:fogPolygon,baseRevision:$room.fog?.revision})){fogPolygon=[];scheduleDraw();}}
  function canMove(token:PlacedToken){return $room.role==='master'||(!!token.sheetId&&characters.some(c=>c.id===token.sheetId&&c.owner===$room.token))||(!token.sheetId&&token.ownerParticipantId===$room.participantId);}
  function hit(x:number,y:number){return [...tokens].reverse().find(t=>!fogHidden(t.x,t.y)&&Math.hypot(x-(preview?.id===t.id?preview.x:t.x),y-(preview?.id===t.id?preview.y:t.y))<=t.size/2);}
  function down(event:PointerEvent){
    if((event.target as HTMLElement).closest('button'))return;
    if(event.button!==0&&event.button!==1&&event.button!==2)return;
    if(event.button===2)rightDragged=false;
    if($room.status!=='ready'&&['draw','text','pin','fog'].includes(tool))return;
    const p=point(event);
    if(tool==='select'&&event.button===0){const pin=[...$room.marks].reverse().find(mark=>mark.kind==='pin'&&!(fogPreview&&mark.visibility==='private')&&!fogHidden(mark.data.x,mark.data.y)&&Math.hypot(p.x-mark.data.x,p.y-mark.data.y)<16);if(pin){onPinOpen(pin);return;}}
    if(event.button===0&&(tool==='text'||tool==='pin')){if(p.x>=0&&p.y>=0&&p.x<=map.width&&p.y<=map.height)onPlace(tool,p.x,p.y);return;}
    if(tool==='fog'&&fogShape==='polygon'&&event.button===0&&$room.role==='master'&&$room.fog?.enabled){
      if(p.x>=0&&p.y>=0&&p.x<=map.width&&p.y<=map.height){const next=clipped(p),last=fogPolygon.at(-1);if(!last||Math.hypot(last.x-next.x,last.y-next.y)>2)fogPolygon=[...fogPolygon,next];if(event.detail>=2||fogPolygon.length>=64)finishFogPolygon();else scheduleDraw();}return;
    }
    if(event.button===2&&hit(p.x,p.y))return;
    if(tool==='fog'&&event.button===0&&$room.role==='master'&&$room.fog?.enabled){if(p.x<0||p.y<0||p.x>map.width||p.y>map.height)return;gesture={kind:'fog',startX:event.clientX,startY:event.clientY,offsetX,offsetY};fogRect={x1:p.x,y1:p.y,x2:p.x,y2:p.y};}
    else if(tool==='measure'&&event.button===0){if(p.x<0||p.y<0||p.x>map.width||p.y>map.height)return;const start=measurePoint(p);gesture={kind:'measure',startX:event.clientX,startY:event.clientY,offsetX,offsetY};line={x1:start.x,y1:start.y,x2:start.x,y2:start.y};if(lineTimer)clearTimeout(lineTimer);}
    else if(tool==='draw'&&event.button===0){if(p.x<0||p.y<0||p.x>map.width||p.y>map.height)return;gesture={kind:'draw',startX:event.clientX,startY:event.clientY,offsetX,offsetY};drawing=[clipped(p),clipped(p)];}
    else {const token=tool==='select'&&event.button===0?hit(p.x,p.y):undefined;if(tool==='select')onSelect(token?.id||null);
      if(token&&canMove(token))gesture={kind:'token',id:token.id,revision:token.revision,startX:event.clientX,startY:event.clientY,offsetX,offsetY,worldX:token.x,worldY:token.y};
      else gesture={kind:'pan',startX:event.clientX,startY:event.clientY,offsetX,offsetY};
    }
    host.setPointerCapture(event.pointerId);draw();
  }
  function move(event:PointerEvent){
    if(!gesture)return;
    if(event.buttons===2 && Math.hypot(event.clientX-gesture.startX,event.clientY-gesture.startY)>5)rightDragged=true;
    if(gesture.kind==='pan'){offsetX=gesture.offsetX+event.clientX-gesture.startX;offsetY=gesture.offsetY+event.clientY-gesture.startY;view();}
    else if(gesture.kind==='token'&&gesture.id){const p=point(event);preview={id:gesture.id,x:Math.max(0,Math.min(map.width,p.x)),y:Math.max(0,Math.min(map.height,p.y))};const node=tokenViews.get(gesture.id)?.node;if(node)node.position.set(preview.x,preview.y);else scheduleDraw();}
    else if(gesture.kind==='measure'&&line){const p=measurePoint(point(event));line={...line,x2:p.x,y2:p.y};scheduleDraw();}
    else if(gesture.kind==='fog'&&fogRect){const p=point(event);fogRect={...fogRect,x2:Math.max(0,Math.min(map.width,p.x)),y2:Math.max(0,Math.min(map.height,p.y))};scheduleDraw();}
    else if(gesture.kind==='draw'&&drawing.length){const p=clipped(point(event));if(drawShape==='freehand'){const last=drawing[drawing.length-1];if(Math.hypot(p.x-last.x,p.y-last.y)>=2&&drawing.length<1024)drawing=[...drawing,p];}else drawing=[drawing[0],p];scheduleDraw();}
  }
  function up(event:PointerEvent){
    if(host.hasPointerCapture(event.pointerId))host.releasePointerCapture(event.pointerId);
    if(!gesture)return;
    move(event);
    if(gesture.kind==='token'&&preview){
      if(trackTokenMove&&shareMeasure&&gesture.worldX!==undefined&&gesture.worldY!==undefined){const distance=Math.round(Math.hypot(preview.x-gesture.worldX,preview.y-gesture.worldY)/map.cellPx*map.metresPerCell*10)/10;sendRoom({type:'measure',mapId:map.id,x1:gesture.worldX/map.width,y1:gesture.worldY/map.height,x2:preview.x/map.width,y2:preview.y/map.height,metres:distance});}
      if(sendRoom({type:'move_token',id:gesture.id,baseRevision:gesture.revision,x:preview.x,y:preview.y})){
        const pending=waiting={id:gesture.id!,revision:gesture.revision!};
        if(pendingTimer)clearTimeout(pendingTimer);
        pendingTimer=setTimeout(()=>{if(waiting!==pending)return;waiting=null;if(gesture?.kind!=='token'){preview=null;draw();}},2000);
      }else preview=null;
    }
    if(gesture.kind==='measure'&&line){
      const dx=Math.abs(line.x2-line.x1),dy=Math.abs(line.y2-line.y1);
      const distance=(measureShape==='line'?Math.hypot(dx,dy):dx+dy)/map.cellPx*map.metresPerCell;
      if(duration==='permanent'){
        const points=[{x:line.x1,y:line.y1},{x:line.x2,y:line.y2}];
        if(shareMeasure)sendRoom({type:'mark_save',kind:'measure',visibility:'public',data:{points,shape:measureShape}});
        else localMeasures=[...localMeasures,{points,shape:measureShape,metres:Math.round(distance*10)/10}];
        line=null;
      }else{
        if(shareMeasure)sendRoom({type:'measure',mapId:map.id,x1:Math.max(0,Math.min(1,line.x1/map.width)),y1:Math.max(0,Math.min(1,line.y1/map.height)),x2:Math.max(0,Math.min(1,line.x2/map.width)),y2:Math.max(0,Math.min(1,line.y2/map.height)),shape:measureShape});
        lineTimer=setTimeout(()=>{line=null;draw();},4000);
      }
    }
    if(gesture.kind==='draw'&&drawing.length>=2){const points=drawing;const length=Math.hypot(points.at(-1)!.x-points[0].x,points.at(-1)!.y-points[0].y);if(length>=1){const data={points,shape:drawShape,stroke:drawStroke,fill:drawFill,width:drawWidth,blend:drawBlend};if(sendRoom({type:'mark_save',kind:'drawing',visibility:'public',data}))drawing=[];else pendingDrawing={...data,mapId:map.id};}else drawing=[];}
    if(gesture.kind==='fog'&&fogRect){const x=Math.min(fogRect.x1,fogRect.x2),y=Math.min(fogRect.y1,fogRect.y2),width=Math.abs(fogRect.x2-fogRect.x1),height=Math.abs(fogRect.y2-fogRect.y1);if(width>=1&&height>=1)sendRoom({type:'fog_set',action:'rect',mode:fogMode,x,y,width,height,baseRevision:$room.fog?.revision});fogRect=null;}
    if(!waiting)preview=null;
    gesture=null;draw();
  }
  function cancel(event:PointerEvent){if(host.hasPointerCapture(event.pointerId))host.releasePointerCapture(event.pointerId);gesture=null;preview=null;waiting=null;if(pendingTimer)clearTimeout(pendingTimer);line=null;fogRect=null;drawing=[];draw();}
  function contextMenu(event:MouseEvent){
    event.preventDefault();
    if((event.target as Element).closest('button'))return;
    if(rightDragged){rightDragged=false;return;}
    const world=point(event),token=hit(world.x,world.y);
    const rect=host.getBoundingClientRect();
    const x=Math.max(8,Math.min(event.clientX-rect.left+8,host.clientWidth-252));
    const y=Math.max(8,Math.min(event.clientY-rect.top+8,host.clientHeight-180));
    if(token){onSelect(token.id);onTokenMenu(token.id,x,y);}
    else if(world.x>=0&&world.y>=0&&world.x<=map.width&&world.y<=map.height){onSelect(null);onMapMenu(x,y,world.x,world.y);}
  }
  function drop(event:DragEvent){
    event.preventDefault();if($room.status!=='ready')return;
    try{const source=JSON.parse(event.dataTransfer?.getData('application/x-tormenta-token')||'null');if(!source||(!source.artId&&!source.sheetId&&!source.threatId))return;const p=point(event);sendRoom({type:'place_token',...source,x:Math.max(0,Math.min(map.width,p.x)),y:Math.max(0,Math.min(map.height,p.y))});}catch{}
  }
  onMount(()=>{
    let disposed=false;let observer:ResizeObserver;
    window.addEventListener('guilda-theme-change',updateCanvasTheme);
    void (async()=>{const instance=new Application();await instance.init({resizeTo:host,background:0x282924,antialias:true});if(disposed){instance.destroy(true);return;}app=instance;fogTexture=cloudTexture();host.prepend(instance.canvas);scene=new Container();backgroundLayer=new Container();tokenLayer=new Container();markLayer=new Container();overlayLayer=new Container();fogLayer=new Container();fogGuideLayer=new Container();scene.addChild(backgroundLayer,tokenLayer,markLayer,overlayLayer,fogLayer,fogGuideLayer);instance.stage.addChild(scene);fitMap();draw();observer=new ResizeObserver(()=>{instance.resize();view();});observer.observe(host);})();
    updateCanvasTheme();
    return ()=>{disposed=true;window.removeEventListener('guilda-theme-change',updateCanvasTheme);observer?.disconnect();if(lineTimer)clearTimeout(lineTimer);if(pingTimer)clearTimeout(pingTimer);if(drawFrame)cancelAnimationFrame(drawFrame);app?.destroy(true);fogMaskTexture?.destroy();fogTexture?.destroy();scene=null;tokenViews.clear();for(const url of urls.values())URL.revokeObjectURL(url);};
  });
  // Pixi reads reactive state internally; only the dependencies listed here may trigger each effect.
  $effect(()=>{map.id;map.width;map.height;map.gridVisible;map.cellPx;map.backdropColor;tokens;selectedId;$room.measure;$room.fog;$room.marks;fogPreview;fogOpacity;line;fogRect;fogPolygon;drawing;localMeasures;untrack(()=>{if(scene){if(waiting){const token=tokens.find(t=>t.id===waiting!.id);if(!token||token.revision!==waiting.revision){waiting=null;if(token&&gesture?.kind==='token'&&gesture.id===token.id)gesture.revision=token.revision;else preview=null;if(pendingTimer)clearTimeout(pendingTimer);}}draw();}});});
  $effect(()=>{const token=tokens.find(item=>item.id===selectedId);$room.fog;fogPreview;if(token&&fogHidden(token.x,token.y))onSelect(null);});
  $effect(()=>{map.id;map.width;map.height;untrack(()=>{if(scene){fitMap();draw();}});});
  $effect(()=>{fogFinishSignal;if(fogFinishSignal>0)untrack(finishFogPolygon);});
  $effect(()=>{fogCancelSignal;if(fogCancelSignal>0)untrack(()=>{fogPolygon=[];scheduleDraw();});});
  $effect(()=>{tool;fogShape;if(tool!=='fog'||fogShape!=='polygon')fogPolygon=[];});
  $effect(()=>{clearLocalMeasures;if(clearLocalMeasures>0)untrack(()=>{localMeasures=[];scheduleDraw();});});
  $effect(()=>{const id=map.id;if(localMapId&&localMapId!==id){localMeasures=[];fogPolygon=[];}localMapId=id;});
  $effect(()=>{if($room.status==='ready'&&pendingDrawing&&map.id===pendingDrawing.mapId)untrack(()=>{if(sendRoom({type:'mark_save',kind:'drawing',visibility:'public',data:pendingDrawing})){pendingDrawing=null;drawing=[];}});});
  $effect(()=>{const ping=$room.ping;$room.fog;if(!ping||ping.mapId!==map.id||fogHidden(ping.x,ping.y))return;visiblePing=ping;if(pingTimer)clearTimeout(pingTimer);pingTimer=setTimeout(()=>visiblePing=null,2200);});
</script>

<div class="mesa-map-canvas" bind:this={host} role="application" aria-label={`Mapa ${map.name}`} onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={cancel} oncontextmenu={contextMenu} onwheel={e=>{e.preventDefault();zoom(e.deltaY<0?1.15:1/1.15,e.clientX,e.clientY);}} ondragover={e=>{if($room.status==='ready')e.preventDefault();}} ondrop={drop}>
  <div class="mesa-map-zoom"><button aria-label="Aproximar mapa" onclick={e=>{e.stopPropagation();zoom(1.25);}}>+</button><span>{zoomLabel}</span><button aria-label="Afastar mapa" onclick={e=>{e.stopPropagation();zoom(1/1.25);}}>−</button><button onclick={e=>{e.stopPropagation();fit();}}>Ajustar</button></div>
  {#if map.kind==='narrative'}<div class="mesa-narrative-scene"><strong>{map.name}</strong><span>Cena narrativa</span></div>{/if}
  {#if imageError && map.kind==='tactical'}<p class="mesa-map-error" role="alert">{imageError}</p>{/if}
  {#if line}<span class="mesa-map-distance" style:left={`${offsetX+line.x2*scale}px`} style:top={`${offsetY+line.y2*scale}px`}>{(Math.round((measureShape==='line'?Math.hypot(line.x2-line.x1,line.y2-line.y1):Math.abs(line.x2-line.x1)+Math.abs(line.y2-line.y1))/map.cellPx*map.metresPerCell*10)/10)} m</span>{/if}
  {#if $room.measure && $room.measure.mapId===map.id && $room.measure.token!==$room.token}<span class="mesa-measure-shared">{$room.measure.author}: {$room.measure.metres} m</span>{/if}
  {#if visiblePing}<span class="mesa-map-ping" style:left={`${offsetX+visiblePing.x*scale}px`} style:top={`${offsetY+visiblePing.y*scale}px`}><span></span><small>{visiblePing.author}</small></span>{/if}
</div>
