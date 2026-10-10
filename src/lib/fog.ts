export type FogRect = {x:number;y:number;width:number;height:number};
export type FogOp = ({shape?:'rect'} & FogRect | {shape:'polygon';points:{x:number;y:number}[]}) & {mode:'reveal'|'hide'};

function insidePolygon(x:number,y:number,points:{x:number;y:number}[]):boolean{
  let inside=false;
  for(let i=0,j=points.length-1;i<points.length;j=i++){
    const a=points[i],b=points[j];
    if((a.y>y)!==(b.y>y)&&x<(b.x-a.x)*(y-a.y)/(b.y-a.y)+a.x)inside=!inside;
  }
  return inside;
}

function subtract(rect:FogRect,cut:FogRect):FogRect[]{
  const x1=Math.max(rect.x,cut.x),y1=Math.max(rect.y,cut.y);
  const x2=Math.min(rect.x+rect.width,cut.x+cut.width),y2=Math.min(rect.y+rect.height,cut.y+cut.height);
  if(x1>=x2||y1>=y2)return [rect];
  return [
    {x:rect.x,y:rect.y,width:rect.width,height:y1-rect.y},
    {x:rect.x,y:y2,width:rect.width,height:rect.y+rect.height-y2},
    {x:rect.x,y:y1,width:x1-rect.x,height:y2-y1},
    {x:x2,y:y1,width:rect.x+rect.width-x2,height:y2-y1}
  ].filter(part=>part.width>0&&part.height>0);
}

export function coveredRectangles(width:number,height:number,ops:Extract<FogOp,FogRect & {mode:'reveal'|'hide'}>[]):FogRect[]{
  let covered:FogRect[]=[{x:0,y:0,width,height}];
  for(const op of ops){
    if(op.mode==='reveal')covered=covered.flatMap(rect=>subtract(rect,op));
    else{
      let additions:FogRect[]=[op];
      for(const rect of covered)additions=additions.flatMap(part=>subtract(part,rect));
      covered.push(...additions);
    }
  }
  return covered;
}

export function isFogCovered(x:number,y:number,ops:FogOp[]):boolean{
  let covered=true;
  for(const op of ops){
    const inside=op.shape==='polygon'?insidePolygon(x,y,op.points):x>=op.x&&y>=op.y&&x<op.x+op.width&&y<op.y+op.height;
    if(inside)covered=op.mode==='hide';
  }
  return covered;
}
