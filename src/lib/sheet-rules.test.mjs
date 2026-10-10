import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { activeConditions, carryingLimit, conditionPenalty, itemSpaces, resizeWeaponDamage, sizeModifiers, spendMana, classProficiencies } from './sheet-rules.ts';

test('carga: Força positiva, negativa, meio espaço e moedas', () => {
  assert.equal(carryingLimit(2), 14);
  assert.equal(carryingLimit(-2), 8);
  assert.equal(itemSpaces([{name:'Poção',quantity:3,spaces:0.5}],{to:1000,ts:1,tc:0}),3.5);
});

test('tamanho altera Furtividade, manobras e passos de dano', () => {
  assert.deepEqual(sizeModifiers('Minúsculo'),[5,-5,-1]);
  assert.deepEqual(sizeModifiers('Colossal'),[-10,10,2]);
  assert.equal(resizeWeaponDamage('1d8',-1),'1d6');
  assert.equal(resizeWeaponDamage('2d6',2),'4d6');
  assert.equal(resizeWeaponDamage('1d10',1),'1d12');
});

test('condições impostas não duplicam penalidades; golem ignora metabolismo', () => {
  const statuses=activeConditions(['Enredado','Vulnerável','Enredado'],'Humano');
  assert.equal(statuses.filter(s=>s.name==='Vulnerável').length,1);
  assert.equal(conditionPenalty(statuses,'defense'),-2);
  assert.equal(activeConditions(['Sangrando','Fatigado','Envenenado'],'Golem').length,0);
});

test('gasto de mana não aceita custo negativo, NaN ou recursos forjados acima do máximo', () => {
  assert.deepEqual(spendMana(5,2,3,5),{current:4,temporary:0});
  assert.deepEqual(spendMana(0,3,3,5),{current:0,temporary:0});
  assert.equal(spendMana(2,0,3,5),null);
  assert.equal(spendMana(6,0,3,5),null);
  for(const value of [-1,NaN,Infinity,0.5]) {
    assert.equal(spendMana(5,0,value,5),null);
    assert.equal(spendMana(5,value,1,5),null);
    assert.equal(spendMana(value,0,1,5),null);
  }
});

test('proficiências adicionais do clérigo e ladino seguem Jogo do Ano pp. 57 e 73',()=>{
  assert.deepEqual(classProficiencies.Clérigo,['Armaduras pesadas','Escudos']);
  assert.deepEqual(classProficiencies.Ladino,[]);
});
