import assert from 'node:assert/strict';
import test from 'node:test';
import { applyAppearance, appearanceTheme, resolveTheme, themePreference, themeStyle } from './theme.ts';

test('tema salvo vence a preferência do sistema', () => {
  assert.equal(resolveTheme('light', true), 'light');
  assert.equal(resolveTheme('dark', false), 'dark');
  assert.equal(resolveTheme(null, true), 'dark');
});

test('preferência inválida volta ao sistema',()=>{
  assert.equal(themePreference(null),'system');
  assert.equal(themePreference('system'),'system');
  assert.equal(themePreference('light'),'light');
});

test('tema novo é Moderno; preferência antiga migra sem perda',()=>{
  assert.equal(themeStyle(null,null),'modern');
  assert.equal(themeStyle(null,'light'),'archaic');
  assert.equal(themeStyle(null,'system'),'archaic');
  assert.equal(themeStyle('nimb','dark'),'nimb');
  assert.equal(themeStyle('invalid','dark'),'archaic');
});

test('somente Arcaico acompanha o tom escolhido ou o sistema',()=>{
  assert.equal(appearanceTheme('modern','light',false),'dark');
  assert.equal(appearanceTheme('nimb','dark',true),'light');
  assert.equal(appearanceTheme('archaic','system',true),'dark');
  assert.equal(appearanceTheme('archaic','system',false),'light');
  assert.equal(appearanceTheme('archaic','light',true),'light');
});

test('aplicar aparência atualiza raiz, janela e avisa o mapa',()=>{
  const originalDocument=globalThis.document,originalWindow=globalThis.window;
  const root={dataset:{}},meta={value:'',setAttribute(_key,value){this.value=value;}};
  const events=[];
  globalThis.document={documentElement:root,querySelector:()=>meta};
  globalThis.window={dispatchEvent:event=>events.push(event.type)};
  try{
    applyAppearance('nimb','dark',true);
    assert.deepEqual(root.dataset,{style:'nimb',theme:'light'});
    assert.equal(meta.value,'#cf2e35');
    applyAppearance('archaic','system',false);
    assert.deepEqual(root.dataset,{style:'archaic',theme:'light'});
    assert.equal(meta.value,'#eee8dc');
    assert.deepEqual(events,['guilda-theme-change','guilda-theme-change']);
  }finally{globalThis.document=originalDocument;globalThis.window=originalWindow;}
});
