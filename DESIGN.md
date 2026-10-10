---
name: A Guilda VTT
description: Três linguagens visuais para uma mesma mesa de RPG
colors:
  modern-bg: "#181818"
  modern-surface: "#1f1f1f"
  modern-accent: "#3996dd"
  archaic-dark-bg: "#0b0c0d"
  archaic-light-bg: "#eee8dc"
  archaic-accent: "#df4b52"
  nimb-bg: "#f2f0ee"
  nimb-surface: "#fff"
  nimb-accent: "#c92d35"
  fog-base: "#5b6f80"
typography:
  modern-body:
    fontFamily: "Segoe UI Variable, Segoe UI, sans-serif"
  archaic-display:
    fontFamily: "Bahnschrift, Arial Narrow, sans-serif"
  nimb-display:
    fontFamily: "Cinzel Decorative, Georgia, serif"
  nimb-body:
    fontFamily: "Alegreya, Georgia, serif"
rounded:
  modern-panel: "11px"
  archaic-panel: "2px"
  nimb-card: "13px"
---

# Design System: A Guilda VTT

## Overview

**Creative North Star: "Uma mesa, três identidades"**

Os três temas usam a mesma composição aprovada do Nimb: navegação superior, cartões, ficha com perícias à direita, PV/PM circulares e controles contextuais na mesa. Moderno usa cores escuras e cantos Windows; Arcaico conserva a cartografia técnica; Nimb mantém o fichário claro e carmesim. A lógica, os dados e as ações permanecem iguais. O mapa real e a névoa não são filtros de tema.

## Colors

Os valores normativos estão no frontmatter e em `src/themes.css`. Azul identifica o foco e a ação do Moderno; o Arcaico usa sua paleta anterior; vermelho carmesim orienta o Nimb. PV, PM, perigo e estados de erro mantêm significado funcional em todos os temas.

## Typography

Moderno usa a família de interface do Windows e números monoespaçados onde há medição. Arcaico mantém os tipos anteriores. Nimb reserva Cinzel Decorative para títulos e Alegreya para texto, formulários e números; controles densos não recebem ornamento extra.

## Layout

A composição do Nimb é comum aos três temas. Todos abrem com a mesma ilustração original e, na ficha ampla, põem perícias à direita e atributos/recursos à esquerda. Em largura estreita, as colunas empilham sem alterar a ordem lógica do DOM. Não há trilho lateral ou Explorador exclusivo de um tema.

## Elevation & Depth

As superfícies dos cartões e a intensidade das sombras acompanham a paleta: escura no Moderno, cartográfica no Arcaico e branca no Nimb. O mapa não recebe filtro nem sombra de tema.

## Shapes

Moderno usa cantos arredondados de 10–12px, Arcaico mantém painéis quase retos e Nimb usa cartões de 13px. Os anéis de PV/PM permanecem nos três temas. A moldura vetorial ornamental dos atributos é exclusiva do Nimb; Moderno usa blocos arredondados e Arcaico, recortes cartográficos.

## Components

As mesmas fichas, campos, botões, chat e ferramentas de mapa recebem tokens de cor, tipo e forma sem mudar de posição. Configurações apresenta a mesma geometria nas três prévias; somente Arcaico expõe a escolha Sistema/Claro/Escuro. Os estados de foco, hover, erro, carregamento e seleção devem continuar identificáveis sem depender apenas da cor.

Na mesa, a barra vertical mantém a mesma ordem nos três temas. Jogadores têm Selecionar, Arrastar, Desenhar, Texto, Medir, Dados e FX; o mestre recebe ainda Ordem de turno, Alfinete e Névoa. A cobertura usa textura procedural azul-acinzentada original, opaca para jogadores e ajustável apenas na prévia do mestre. A cor externa ao mapa vem da imagem ou do ajuste manual do mestre; nenhuma dessas escolhas recolore a arte.

## Do's and Don'ts

### Do:

- **Do** manter mapa, ficha e ações principais visíveis ao trocar de tema.
- **Do** usar arte própria e fontes locais com licenças incluídas no pacote.
- **Do** honrar movimento reduzido e contraste de texto em todo tema.

### Don't:

- **Don't** copiar logotipo, ilustrações ou conteúdo de terceiros.
- **Don't** duplicar árvores inteiras de componentes para sustentar os três visuais.
- **Don't** mudar regras, permissões ou dados ao mudar aparência.
