# Sistema de UI - A Guilda VTT

> Atualização de 8 de outubro de 2026: este documento descreve a identidade visual do tema **Arcaico**. A composição de início, ficha e mesa é compartilhada pelos temas **Moderno**, **Arcaico** e **Nimb**; suas diferenças de cor, tipografia e forma estão documentadas em `DESIGN.md` e implementadas em `src/themes.css`. A preferência legada `a-guilda-vtt-theme` continua determinando apenas a luminosidade do Arcaico; `a-guilda-vtt-style` seleciona a família visual.

> Nome de produto provisório: **A Guilda VTT**.

Complemento aos documentos de regras e arquitetura técnica. Este cobre **o vocabulário visual**: paleta (claro/escuro), tipografia, layout e — o mais importante pra não repetir o erro anterior — um **inventário fechado de componentes reutilizáveis**. A ficha inteira e o hub devem ser montados a partir desses blocos, nunca desenhados tela por tela do zero.

---

## 0. O erro mais comum nesse tipo de app

Ficha de RPG é dado denso: dezenas de números pequenos, tabelas, checkboxes. O risco não é "ficar feio", é ficar **inconsistente** — cada seção com um espaçamento diferente, um ícone novo pra cada ação parecida, bordas ora com sombra ora sem. A defesa contra isso é simples: **todo elemento repetido no app (linha de perícia, linha de ataque, linha de item) usa o mesmo componente**, só trocando dados. Se uma seção nova "quase" se encaixa num componente existente, ela deve mudar pra caber — não ganhar uma variante nova.

## 1. Paleta — tokens semânticos (claro e escuro)

Nomeie por **função**, não por cor, pra trocar de tema sem tocar em nenhum componente:

| Token | Escuro | Claro | Uso |
|---|---|---|---|
| `bg` | `#0B0C0D` | `#F1F1EE` | fundo geral |
| `surface` | `#121416` | `#FBFBF8` | painéis, linhas de tabela |
| `surface-raised` | `#1A1D20` | `#E8E9E6` | elementos sobrepostos (menus, modais) |
| `border` | `#34383D` | `#C4C6C3` | hairlines entre seções e células |
| `text` | `#F0F0ED` | `#111214` | texto principal |
| `text-muted` | `#A4A8AC` | `#5D6266` | labels, texto secundário |
| `accent` | `#DF4B52` | `#A9252D` | seleção, combate e ações contextuais |
| `vida` | `#C4675A` | `#A8483B` | PV, dano, alertas de recurso |
| `mana` | `#7477A8` | `#5B5E8F` | PM, magia |
| `positivo` | `#7A9A68` | `#5A7A49` | confirmações, "treinado" |
| `negativo` | `#C4453A` | `#A6362C` | erro, validação, remoção |

A direção visual é **Cartografia Técnica**: preto estrutura a interface, cinzas frios separam planos e vermelho marca ações ou estados importantes. A grade cartográfica pode aparecer de forma sutil no fundo geral, mas não dentro de tabelas ou campos densos.

`vida` e `mana` não são decoração — são os únicos dois lugares onde cor carrega significado funcional consistente (barra de PV sempre nessa cor, barra de PM sempre naquela, em toda a UI, sem exceção).

## 2. Tipografia

Duas famílias, papéis claramente distintos:

- **Display/cabeçalhos** (nome do personagem, títulos de seção, hub): **Bahnschrift**, com fallback para `Arial Narrow` e sans-serif. O desenho condensado sustenta a linguagem de carta técnica sem prejudicar leitura.
- **Corpo/dados/tabelas** (perícias, ataques, inventário, formulários): **Segoe UI Variable**, com fallback para `Segoe UI` e sans-serif. Números de atributos, recursos e totais usam **Cascadia Mono** ou Consolas, além de `tabular-nums`.

Escala sugerida (rem, base 16px): `12 / 13 / 14 / 16 / 20 / 28 / 40`. A maior parte da UI (tabelas, formulários) vive em `12–14`; `28–40` é reservado pra nome do personagem e números de destaque (total de PV, por exemplo).

Evite: caixa alta em labels, um "eyebrow" acima de cada título, aspas decorativas — nada disso aparece nas referências de ficha oficial e adicionaria ruído numa tela já densa.

## 3. Layout

- **Sem cards com sombra e cantos muito arredondados.** Painéis são retângulos com hairline de 1px (`border`) e raio de 2px. É o mesmo padrão da ficha de referência: blocos com título no topo, conteúdo em lista/tabela, borda fina.
- **Números são o protagonista, label é quieto.** Nas caixas de atributo e recursos: número grande, label pequeno abaixo ou ao lado — nunca o contrário.
- **Alinhamento**: conteúdo denso (tabelas, formulários) alinhado à esquerda; valores numéricos à direita dentro da própria coluna. Nada centralizado por padrão — centralizar é pra poucos elementos de destaque (ex. o nome do personagem no cabeçalho).
- **Densidade controlada por espaçamento vertical**, não por reduzir fonte: altura de linha consistente em tabelas (ex. 32px por linha de perícia/item), com padding interno fixo — isso é o que faz a ficha parecer "profissional" em vez de espremida.

## 4. Inventário de componentes

Esses componentes formam a base da ficha e do hub. Novos componentes são aceitáveis quando representam comportamento ou semântica realmente diferente; não se cria uma variante apenas para mudar espaçamento ou cor.

| Componente | Onde aparece | Comportamento |
|---|---|---|
| **StatBox** | Atributos (FOR–CAR) | número grande + label pequeno, borda fina, sem interação (ou clique abre detalhe) |
| **ResourceBar** | PV, PM | Máximo/Atual/Temporário + barra de preenchimento na cor do token (`vida`/`mana`); editar "Atual" direto no número |
| **DataRow** | Listas simples e tabelas estreitas | Linha padrão: ícone opcional à esquerda (dado/livro), label, valores intermediários, total à direita. Listas com edição ou colunas semanticamente diferentes podem ter componentes próprios reutilizando os mesmos tokens e controles. |
| **DiceButton** | Toda linha com rolagem possível | Ícone de dado consistente; clique resolve a rolagem e dispara evento (rola local + envia pro WebSocket quando em mesa) |
| **Toggle/Checkbox** | "Treinado" em perícia, "Equipado" em item | Estado visual único reaproveitado em qualquer lugar que precise de on/off |
| **Panel** | Cada bloco da ficha (Ataques, Perícias, Magias, Inventário, Anotações) | Título + hairline + conteúdo; colapsável — em telas estreitas, vira acordeão em vez de forçar scroll horizontal |
| **FormulaLine** | Defesa, CD de magia | Mostra a soma (`10 + DES + armadura + escudo + outros`) com cada termo clicável/editável quando aplicável |
| **Tag** | Tamanho, deslocamento, proficiências, tipo de dano | Texto curto com borda, sem preenchimento sólido — não é "badge" colorido, é rótulo discreto |

**Campos calculados vs. editáveis** precisam ser visualmente diferentes sempre: calculado = texto sem borda, levemente `text-muted`; editável = input com borda visível. Isso evita o erro comum de o jogador não saber se pode clicar num número ou não.

## 5. Tema claro/escuro na prática

Implementar como **um único conjunto de componentes**, tema resolvido via variáveis CSS (`data-theme="dark"` / `"light"` na raiz, com os tokens da seção 1 como custom properties). Nenhum componente deve ter cor "hardcoded" — só referências a `var(--bg)`, `var(--accent)`, etc. Isso também é o que garante que os dois temas não divirjam com o tempo (mudar um token muda os dois temas juntos, quando fizer sentido).

## 6. Ordem de construção

Baseado no que travou da última vez: comece pelos componentes isolados, não pela tela inteira.

1. Tokens (cores + tipografia) como CSS variables, com toggle de tema funcionando numa tela em branco.
2. `StatBox`, `ResourceBar`, `DataRow`, `Toggle` isolados — cada um com dados falsos, sem lógica de negócio ainda.
3. Montar **um** bloco real da ficha (ex. Atributos + PV/PM) usando esses componentes.
4. Só depois expandir pros blocos restantes (Ataques, Perícias, Magias, Inventário) — a essa altura é composição, não design novo.
5. Hub (Mesas/Personagens/Livros) por último, reaproveitando `Panel` e `Tag` já existentes.

Se uma tela nova quase cabe num componente existente, primeiro tente compor os controles existentes. Generalize o componente apenas quando houver pelo menos dois usos reais com o mesmo comportamento; componentes genéricos especulativos ficam fora do projeto.
