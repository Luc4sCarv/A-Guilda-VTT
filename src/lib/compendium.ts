import rules from '../../VTT_Documentacao/tormenta20_core_rules-v6.md?raw';

const catalogText = rules.replace(/\r\n/g, '\n');

export type CatalogGroup = 'Armas' | 'Armaduras' | 'Escudos' | 'Munições' | 'Magias' | 'Poderes';
export type CatalogItem = {
  name: string;
  group: CatalogGroup;
  category: string;
  summary: string;
  detail: string;
  page: string;
  circle?: number;
  tradition?: string;
  damage?: string;
  critical?: string;
  damageType?: string;
  range?: string;
  price?: string;
  spaces?: number;
  defense?: string;
  penalty?: string;
  execution?: string;
  duration?: string;
  resistance?: string;
  cost?: string;
  enhancements?: string;
  quantity?: number;
};

function table(heading: string): string[][] {
  const marker = heading.startsWith('7. Poderes') || heading.startsWith('8. Magias') ? '## ' : '### ';
  const start = catalogText.indexOf(`${marker}${heading}\n`);
  if (start < 0) throw new Error(`Tabela não encontrada: ${heading}`);
  const tail = catalogText.slice(start + marker.length + heading.length + 1);
  const next = tail.search(/\n(?:### |---\n|## )/);
  const body = next < 0 ? tail : tail.slice(0, next);
  return body.split('\n').filter(line => line.startsWith('| **')).map(line =>
    line.split('|').slice(1, -1).map(cell => cell.trim().replace(/^\*\*|\*\*$/g, ''))
  );
}

const weapons: CatalogItem[] = table('Armas').map(([name, category, grip, damage, critical, range, damageType, price, spaces, , , page]) => ({
  name, group: 'Armas', category, summary: `${damage} · ${critical} · ${range === '—' ? 'corpo a corpo' : range.toLowerCase()}`,
  detail: `${category} · ${grip} · ${damageType} · ${price}`, page, damage, critical, range,
  damageType, price, spaces: Number(spaces),
}));

const armor: CatalogItem[] = table('Armaduras e Escudos').map(([name, category, defense, penalty, , price, spaces, , , page]) => ({
  name, group: category === 'Escudo' ? 'Escudos' : 'Armaduras', category,
  summary: `Defesa ${defense} · penalidade ${penalty}`, detail: `${category} · ${price} · ${spaces} espaço(s)`,
  page, defense, penalty, price, spaces: Number(spaces),
}));

const ammunition: CatalogItem[] = table('Munições').map(([name, quantity, price, spaces, , , page]) => ({
  name, group: 'Munições', category: 'Munição', summary: `${quantity} unidades · ${price}`,
  detail: `${spaces} espaço(s) por pacote`, page, price, spaces: Number(spaces), quantity: Number(quantity),
}));

const spells: CatalogItem[] = table('8. Magias — amostra parcial (`magias.md`)').map(([
  name, circle, tradition, execution, range, duration, resistance, cost, effect, enhancements, , , page,
]) => ({
  name, group: 'Magias', category: tradition, circle: Number.parseInt(circle, 10), tradition,
  summary: `${circle} círculo · ${cost} · ${tradition.toLowerCase()}`,
  detail: effect, page, range, execution, duration, resistance, cost, enhancements,
}));

const powers: CatalogItem[] = table('7. Poderes — amostra parcial (`poderes.md`)')
  .filter(([, , , , , , source]) => source === 'Livro Básico')
  .map(([name, category, className, level, requirements, effect, , , page]) => ({
    name, group: 'Poderes', category,
    summary: `${category} · ${className} · ${level}`,
    detail: `${effect} · Requisitos: ${requirements}`, page,
  }));

export const compendium: CatalogItem[] = [...weapons, ...armor, ...ammunition, ...spells, ...powers];
