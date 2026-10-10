import assert from 'node:assert/strict';
import test from 'node:test';
import { deities, origins, skills, skillTotal } from '../src/lib/character-rules.ts';

test('catálogos básicos e progressão de perícias', () => {
  assert.equal(Object.keys(deities).length, 20);
  assert.equal(Object.keys(origins).length, 35);
  assert.equal(skills.length, 29);
  assert.ok(deities.Valkaria.powers.includes('Armas da Ambição'));
  assert.deepEqual(origins.Soldado.skills, ['Fortitude', 'Guerra', 'Luta', 'Pontaria']);
  assert.equal(skillTotal(1, 0, true), 2);
  assert.equal(skillTotal(7, 0, true), 7);
  assert.equal(skillTotal(15, 0, true), 13);
  assert.equal(skillTotal(2, 1, false, -5), -3);
});
