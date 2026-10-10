import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

const source = await readFile(new URL('../App.svelte', import.meta.url), 'utf8');
function editor(overrides={}) {
  const context={level:1,sheetDirty:false,sheetOpen:true,compendiumOpen:true,roomSheetId:'',section:'resumo',startupError:'',confirm:()=>false,updateResources:fn=>fn(),...overrides};
  const functions=['changeLevel','closeSheet'].map(name=>source.split('\n').find(line=>line.startsWith(`  function ${name}(`))).join('\n');
  runInNewContext(ts.transpileModule(functions,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);
  return context;
}
test('nível inválido restaura o campo e explica o erro sem recalcular recursos',()=>{
  for(const value of [0,21,1.5,NaN,Infinity]) {
    const state=editor(),input={value:String(value)};
    state.changeLevel(value,input);
    assert.equal(state.level,1);assert.equal(input.value,'1');
    assert.match(state.startupError,/entre 1 e 20/);
  }
  const state=editor();state.changeLevel(20,{value:'20'});
  assert.equal(state.level,20);assert.equal(state.sheetDirty,true);
});
test('cancelar saída preserva rascunho; confirmar volta à lista ou à mesa',()=>{
  const state=editor({sheetDirty:true});state.closeSheet();
  assert.equal(state.sheetOpen,true);assert.equal(state.section,'resumo');
  state.confirm=()=>true;state.closeSheet();
  assert.equal(state.sheetOpen,false);assert.equal(state.section,'fichas');
  const room=editor({roomSheetId:'mesa'});room.closeSheet();assert.equal(room.section,'mesas');
});
test('alerta fica na ficha e todos os atalhos de saída usam a confirmação',()=>{
  assert.match(source,/class="sheet-page"[\s\S]*?\{#if startupError\}<p class="mesa-error" role="alert">/);
  assert.ok(!source.includes('onclick={()=>{sheetOpen=false;'));
  assert.ok(source.includes('changeLevel(e.currentTarget.valueAsNumber,e.currentTarget)'));
});
