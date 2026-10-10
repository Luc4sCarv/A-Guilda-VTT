import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { coveredRectangles, isFogCovered } from './fog.ts';

test('névoa suporta revelações e ocultações sobrepostas sem buracos falsos', () => {
  const ops=[
    {mode:'reveal',x:10,y:10,width:40,height:40},
    {mode:'reveal',x:40,y:10,width:40,height:40},
    {mode:'hide',x:45,y:20,width:10,height:10},
    {mode:'reveal',x:48,y:22,width:4,height:4}
  ];
  const rects=coveredRectangles(100,100,ops);
  for(let y=0;y<100;y+=2)for(let x=0;x<100;x+=2){
    const overlaps=rects.filter(rect=>x>=rect.x&&y>=rect.y&&x<rect.x+rect.width&&y<rect.y+rect.height).length;
    assert.equal(overlaps,isFogCovered(x,y,ops)?1:0,`posição ${x},${y}`);
  }
});

test('névoa poligonal respeita a ordem e mantém a compatibilidade com retângulos antigos', () => {
  const ops=[
    {mode:'reveal',x:10,y:10,width:20,height:20},
    {mode:'hide',shape:'polygon',points:[{x:0,y:0},{x:40,y:0},{x:0,y:40}]},
    {mode:'reveal',shape:'polygon',points:[{x:5,y:5},{x:15,y:5},{x:5,y:15}]}
  ];
  assert.equal(isFogCovered(8,8,ops),false);
  assert.equal(isFogCovered(20,15,ops),true);
  assert.equal(isFogCovered(25,25,ops),false);
  assert.equal(isFogCovered(90,90,ops),true);
});
