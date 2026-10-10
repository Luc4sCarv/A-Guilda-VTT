import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createServer } from 'vite';

test('compêndio local inclui poderes e magias nas categorias corretas', async () => {
  const server = await createServer({ server: { middlewareMode: true }, appType: 'custom' });
  try {
    const { compendium } = await server.ssrLoadModule('/src/lib/compendium.ts');
    assert.ok(compendium.some(item => item.group === 'Poderes' && item.name === 'Acuidade com Arma'));
    assert.ok(compendium.some(item => item.group === 'Magias'));
    assert.ok(compendium.every(item => item.name && item.group && item.page));
    const power = name => compendium.find(item => item.group === 'Poderes' && item.name === name);
    assert.match(power('Sortudo').detail, /3 PM/);
    assert.match(power('Vontade de Ferro').detail, /a cada dois níveis/);
    assert.equal(power('Vitalidade').category, 'Combate');
    assert.match(power('Ataque Poderoso').detail, /Requisitos: For 1$/);
    assert.match(power('Saque Rápido').detail, /Requisitos: Iniciativa treinada$/);
    assert.match(power('Acuidade com Arma').detail, /ataque e dano/);
    assert.equal(power('Aumento de Atributo'), undefined);
  } finally {
    await server.close();
  }
});
