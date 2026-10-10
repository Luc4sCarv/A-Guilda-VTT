// Livro Básico Jogo do Ano, p. 17, tabela 1-1. Apenas atributos base
// comprados: bônus de raça, idade, poderes e efeitos são fontes separadas.
export const ATTRIBUTE_BUDGET = 10;
export function attributeCost(values: unknown): number | null {
  if (!Array.isArray(values) || values.length !== 6) return null;
  const costs = [-1, 0, 1, 2, 4, 7];
  if (values.some(value => !Number.isInteger(value) || value < -1 || value > 4)) return null;
  return values.reduce((total, value) => total + costs[value + 1], 0);
}

export function attributePurchaseError(values: unknown): string | null {
  const cost = attributeCost(values);
  if (cost === null) return 'Informe seis atributos base inteiros entre −1 e 4.';
  return cost > ATTRIBUTE_BUDGET ? `Compra de atributos: ${cost}/${ATTRIBUTE_BUDGET} pontos. Reduza os atributos base para continuar.` : null;
}

export function canChangeAttribute(values: number[], index: number, value: number): boolean {
  if (!Number.isInteger(index) || index < 0 || index >= 6) return false;
  const next = values.map((previous, position) => position === index ? value : previous);
  const cost = attributeCost(next);
  const previous = attributeCost(values);
  // Permite reparar uma ficha antiga acima do orçamento sem apagar seus dados.
  return cost !== null && (cost <= ATTRIBUTE_BUDGET || (previous !== null && cost < previous));
}
