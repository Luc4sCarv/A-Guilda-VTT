import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { attributeCost, attributePurchaseError, canChangeAttribute } from './attribute-purchase.ts';

test('compra usa custo não linear e orçamento compartilhado', () => {
  assert.equal(attributeCost([4, 3, 0, 0, 0, -1]), 10);
  assert.equal(attributeCost([4, 4, 4, 4, 4, 4]), 42);
  assert.ok(attributePurchaseError([4, 4, 4, 4, 4, 4]));
  assert.equal(attributePurchaseError([4, 4, -1, -1, -1, -1]), null);
  assert.equal(canChangeAttribute([4, 3, 0, 0, 0, -1], 2, 1), false);
});

test('rejeita tipos, tamanho e valores inválidos sem coagir entradas', () => {
  for (const values of [null, {}, [], [0,0,0,0,0], ['4',0,0,0,0,0], [NaN,0,0,0,0,0], [Infinity,0,0,0,0,0], [1.5,0,0,0,0,0], [-2,0,0,0,0,0], [5,0,0,0,0,0]]) assert.ok(attributePurchaseError(values));
  assert.equal(canChangeAttribute([0,0,0,0,0,0], 6, 1), false);
});

test('fichas legadas podem reduzir o excesso, mas não ampliá-lo nem salvar', () => {
  assert.equal(canChangeAttribute([4,4,4,4,4,4], 0, 3), true);
  assert.equal(canChangeAttribute([3,4,4,4,4,4], 0, 4), false);
  assert.ok(attributePurchaseError([3,4,4,4,4,4]));
});
