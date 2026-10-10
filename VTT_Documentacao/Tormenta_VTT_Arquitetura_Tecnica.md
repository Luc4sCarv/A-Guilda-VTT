# Arquitetura técnica — Tormenta20 VTT

Complemento ao `Tormenta_VTT_Regras_e_Logica.md`. Aquele documento cobre o **domínio** (regras, ficha, validação). Este cobre **como estruturar o software**: leve, rápido, cliente-servidor, com a ficha seguindo o design da ficha oficial (imagens de referência: layout de atributos, PV/PM, ataques com rolagem, perícias, poderes/magias/inventário em abas).

Prioridade explícita: **leveza e velocidade acima de paridade de recursos com o Foundry**. Não copiar a stack do Foundry (Node + Electron); copiar só o *modelo* cliente-servidor onde ele resolve os requisitos (mesa hospedada, estado em tempo real).

---

## 1. Modelo de arquitetura: local por padrão, servidor sob demanda

A operação normal é totalmente local. O servidor embutido só existe durante uma mesa hospedada pelo mestre; todo mundo — inclusive o mestre — usa a mesma UI.

- **Uso local**: UI → comandos Tauri → `core`/SQLite. Sem servidor, porta ou WebSocket.
- **Mestre abre uma mesa**: o app inicia o servidor embutido e passa a coordenar o estado compartilhado.
- **Jogador entra na mesa**: o app conecta ao servidor do mestre pela LAN.
- **Mestre fecha a mesa**: conexões são encerradas, o estado pendente é persistido e o servidor para.
- **Estado compartilhado em tempo real**: movimento de token, rolagens, atualização de ficha — tudo via WebSocket, não polling.

Um único binário cobre os dois papéis, mas não mantém infraestrutura de rede ociosa. O diagrama abaixo representa apenas uma mesa ativa; no uso local, UI, `core` e SQLite se comunicam diretamente dentro do app.

```
┌───────────────────────────────┐
│   App (Tauri)                 │
│  ┌─────────────┐              │
│  │  Servidor    │  localhost  │ ◄── jogadores conectam aqui
│  │  (Rust/axum) │  ou LAN/IP  │     quando o mestre expõe a mesa
│  └──────┬───────┘              │
│         │ WebSocket             │
│  ┌──────▼───────┐               │
│  │  Cliente      │              │
│  │  (webview)    │              │
│  └───────────────┘              │
└───────────────────────────────┘
```

## 2. Stack

| Camada | Escolha | Por quê |
|---|---|---|
| Empacotamento desktop | **Tauri** | Sem Chromium embutido — usa o webview do SO. Binário de poucos MB, RAM baixa, start quase instantâneo. Compatível com o alvo Windows/`.exe` do `.md` original. |
| Servidor embutido | **Rust** (`axum`, usando seu suporte a WebSocket) | Compila junto no mesmo binário do Tauri, sem runtime externo pra instalar. Rápido e com baixo consumo, mesmo rodando servidor + UI na mesma máquina. |
| Núcleo de regras (`core`) | **Rust**, crate própria, sem I/O | Cálculo de PV/PM/Defesa/perícias, validação de atributos, multiclasse — tudo do `Tormenta_VTT_Regras_e_Logica.md` vira funções puras testáveis. Usada tanto pelo servidor (autoridade — evita cliente trapacear) quanto exposta ao frontend via Tauri commands (feedback instantâneo na UI). |
| Comunicação em tempo real | **WebSocket** | Sincroniza ficha, ataques, dados rolados, posição de token, fog of war entre servidor e clientes conectados. |
| Persistência | **SQLite** (`rusqlite`) | Um arquivo por instalação/mundo. Simples de fazer backup, importar/exportar em JSON como o `.md` já define. |
| UI (cliente) | **Svelte + TypeScript**, DOM comum | Ficha, compêndio, hub de mesas/personagens/livros são formulários e listas — DOM puro, sem necessidade de canvas. |
| Mapa da mesa | **PixiJS**, isolado numa view própria | Único ponto que usa canvas/WebGL pesado (tokens, grid, fog of war, luz). Fica contido na tela "Mesa", não contamina o resto da UI — reduz o risco de inconsistência de webview entre SOs. |
| Rolagem de dados | Parser de expressão em Rust (`1d20+5`, `2d6`), exposto ao frontend | Único módulo, usado tanto no "dado solto" (d4–d100) quanto no dado vinculado ao ataque da ficha — sem duplicar lógica. |

## 3. Estrutura de projeto

```
tormenta20-vtt/
├── src-tauri/                 # projeto Rust (Tauri)
│   ├── crates/
│   │   ├── core/              # regras puras: atributos, PV/PM, Defesa,
│   │   │                      # perícias, multiclasse, validação de ficha
│   │   ├── server/            # axum + websocket: mesas, sync, persistência
│   │   └── dice/              # parser e resolução de expressões de dado
│   ├── src/                   # entrypoint Tauri, commands expostos ao frontend
│   └── Cargo.toml
├── src/                       # frontend (Svelte + TypeScript)
│   ├── views/
│   │   ├── hub/                # Mesas / Personagens / Livros / Estatísticas
│   │   ├── ficha/               # componentes da ficha (seção 4)
│   │   └── mesa/                 # canvas PixiJS + HUD da mesa
│   └── lib/                      # chamadas aos Tauri commands, tipos
└── compendio/                     # dados de conteúdo (armas, poderes,
                                    # magias, ancestralidades, origens, classes)
```

`core` isolado do resto é o ponto mais importante: mesma lógica de cálculo rodando no servidor (autoridade) e espelhada na UI (feedback instantâneo ao editar a ficha), sem duplicar regra em duas linguagens.

O cliente local consulta `core` por comandos Tauri para pré-visualizar alterações, mas esse resultado não é autoritativo. Em uma mesa, toda mutação é validada novamente pelo `core` do servidor do mestre.

### Sincronização mínima

O servidor mantém uma revisão inteira crescente por mesa. O cliente envia comandos com `id`, `tipo`, `revisao_base` e `dados`; o servidor valida, grava em uma transação SQLite e publica o evento confirmado. Comandos repetidos com o mesmo `id` não são reaplicados. Na v1, a última alteração aceita pelo servidor vence. Ao perder uma revisão ou reconectar, o cliente solicita um snapshot completo. CRDT e sincronização offline de mesa ficam fora do MVP.

### Identidade e segurança da mesa

- Ao criar a mesa, o servidor gera um segredo aleatório de alta entropia. O QR carrega endereço local, porta e esse segredo.
- O código curto localiza uma mesa anunciada na LAN; ele não substitui o segredo. Tentativas de pareamento são limitadas.
- Cada jogador recebe um token de sessão revogável associado ao seu usuário e personagem. O servidor verifica permissão em todo comando.
- Nenhuma porta é aberta no uso local. Durante uma mesa, o servidor escuta na rede local. Internet, TLS, relay e contas remotas ficam fora do MVP.

## 4. Estrutura da ficha (baseada no design de referência)

Componentes mapeados diretamente das imagens de referência, cada um consumindo `core` só para exibir valores já calculados — o componente não calcula, só chama:

- **Cabeçalho**: nome do personagem, jogador, raça/origem, classe (+ multiclasse), XP, nível, divindade, distinção.
- **Atributos**: 6 caixas fixas (FOR, DES, CON, INT, SAB, CAR) com o valor final (base + ancestralidade já aplicada).
- **Recursos**: PV e PM, cada um com Máximo / Atual / Temporário. Subir de nível recalcula o máximo via `core`; se o recurso estava cheio, permanece cheio após o recálculo (regra já definida no `.md`).
- **Ataques**: tabela com Bônus de Ataque, Dano, Dano Extra, Dados Extra, Crítico, e ícone de dado (🎲) por linha — clique dispara a rolagem vinculada (ver seção 5).
- **Defesa**: fórmula visível `10 + DES + Bônus de Armadura + Bônus de Escudo + Outros`, com armadura pesada removendo DES do cálculo.
- **Armadura e Escudo**: nome do item, Defesa concedida, Penalidade — linkado ao inventário.
- **Tamanho** e **Deslocamento**: campos derivados de ancestralidade + armadura pesada.
- **Perícias**: lista das 29, cada linha com Total / Atributo / Treino / Outros, checkbox de treinada (reflete progressão de treinamento por nível).
- **Habilidades e Poderes Gerais**: lista com ícone de "página" (fonte/descrição) por item, botão de adicionar, botão de lista (visão expandida).
- **Magias**: agrupadas por círculo (1º a 5º), com CD calculado (`10 + atributo + equip. + poder + outros`) e custo em PM por círculo exibido no cabeçalho de cada grupo.
- **Equipamentos/Inventário**: colunas de Vestido/Equipado (checkbox), Quantidade, Nome, Espaços — com totais de carga (Tibar, T. Ouro, Espaços, E.Extra, E.Limite, E.Máxima) calculados como `espaços × quantidade` somado.
- **Anotações**: texto livre, sem cálculo.

Cada bloco acima deve virar um componente de frontend isolado, recebendo dados via prop/state e chamando os Tauri commands do `core` para qualquer valor derivado — nunca calculando localmente em JS/TS.

## 5. Pontos específicos de comportamento

### Compêndio com drag & drop
Conteúdo (`compendio/`) como registros com schema fixo por tipo (arma, poder, magia, item, ancestralidade, origem, classe). O compêndio é uma lista filtrável; arrastar até a ficha usa a API HTML5 Drag and Drop, e o `drop` chama a mesma validação do `core` que uma edição manual chamaria — o item entra pré-preenchido, mas validado do mesmo jeito.

### Rolagem direto da ficha
Cada linha de ataque já carrega a expressão de dano resolvida a partir dos dados da arma. Clique no dado: (1) resolve a rolagem via `dice`, (2) envia o resultado pro servidor via WebSocket, (3) servidor retransmite pro chat de todos os conectados e, se houver token-alvo selecionado no mapa, aplica a redução de PV automaticamente. O "alvo" é apenas o token clicado no mapa guardado como contexto antes de rolar.

### Visão e sombra (fog of war + cone de visão)
1. Mestre desenha **paredes** como segmentos de linha sobre o mapa.
2. Para cada token, calcula-se um **polígono de visibilidade** por raycasting: raios da posição do token até os extremos de cada parede visível formam o polígono iluminado (técnica padrão de "2D visibility polygon" / shadow casting — não precisa reinventar, há referências prontas).
3. Fora do polígono: fog of war (escurecido se já explorado, oculto se nunca visto).
4. Cone de visão à frente = mesmo cálculo com ângulo de varredura limitado (ex.: 90–120°), orientado pela direção do token.
5. O cálculo roda no servidor; cada cliente recebe só a máscara de visibilidade de seus tokens. Na v1 isso evita revelação acidental pela UI, mas não protege contra um cliente modificado que inspecione a imagem completa. Servir o mapa em recortes autorizados só entra se houver necessidade real contra clientes hostis.

## 6. Ordem de implementação sugerida

1. `core` em Rust: atributos, PV/PM/Defesa, perícias, multiclasse, validação — com testes cobrindo as regras do `.md`.
2. Ficha completa rodando **local**, sem iniciar servidor ou abrir porta.
3. Compêndio + drag & drop na ficha.
4. WebSocket + conceito de "mesa": múltiplos clientes conectados, ficha e chat compartilhados, ainda sem mapa.
5. Canvas do mapa (PixiJS): tokens, grid, movimentação sincronizada.
6. Paredes + visibilidade/fog of war.
7. Rolagem vinculada a ataques da ficha com aplicação automática de dano no alvo.

Cada etapa entrega um app utilizável sozinho — ajuda a não travar num escopo grande demais antes de ter algo funcional pra testar.

O gerador de dados usa a aleatoriedade do sistema operacional. Fora de uma mesa, o resultado fica apenas no histórico local; durante uma mesa, o servidor registra expressão, resultado, autor e horário no chat. Dados 3D, provas criptográficas e replay verificável ficam fora do MVP.
