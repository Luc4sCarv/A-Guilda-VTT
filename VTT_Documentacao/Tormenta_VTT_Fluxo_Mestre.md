# Fluxo do Mestre — Tormenta20 VTT

Complemento aos documentos de regras, arquitetura e UI. Este cobre **o que o mestre precisa fazer, do zero até a primeira sessão rodando**, comparando com Roll20 e Foundry para decidir o que copiar, o que simplificar e o que deixar de fora do MVP.

Princípio geral: **o app já é 100% dedicado a Tormenta20**. Isso elimina de cara a maior fonte de complexidade inicial dos dois concorrentes — escolher/configurar sistema de jogo, importar compêndio de terceiros, decidir qual ficha usar. O mestre nunca escolhe "sistema"; ele só cria a mesa.

---

## 1. O que Foundry e Roll20 fazem hoje (resumo por área)

| Área | Foundry | Roll20 |
|---|---|---|
| Convite de jogadores | Link de convite gerado automaticamente: um IP de rede local e um IP público, com o link público ofuscado por padrão (evita vazar em stream) | Link de convite da sessão via navegador, sem configuração de rede — Roll20 é hospedado na nuvem deles |
| Permissões | 5 níveis (bloqueado, jogador, confiável, mestre assistente, mestre), configuráveis por documento (ficha, item, cena) | Modelo mais simples: dono do personagem vs. mestre, com toggle de "jogadores podem criar personagem" |
| Organização de conteúdo | Cenas, atores, itens, journals — tudo em pastas manuais | Journal como "home base": fichas, handouts e notas do mestre num lugar só, com tags |
| Cenas/mapas | Navegação de cenas, controle de "GM only" pra não poluir a barra dos jogadores, iluminação dinâmica, fog of war | Page Toolbar pra alternar mapas, grid ajustável, fog of war, iluminação dinâmica |
| Combate | Combat tracker: clique direito no token → adiciona à iniciativa | Similar, com rolagem de iniciativa integrada à ficha |
| Comunicação | Chat com sussurro (`/w`) | Chat com sussurro pro mestre especificamente (`/w gm`), dados 3D opcionais |
| Conteúdo pronto | Compêndios de terceiros, tokens com anel dinâmico, imagens "wildcard" (várias variações de um mesmo NPC) | Compendium sharing (se o mestre possui o conteúdo, libera consulta aos jogadores), Charactermancer (assistente de criação de personagem) |

O padrão claro: **os dois tratam "convidar jogador" e "montar o conteúdo da mesa" como as duas tarefas centrais do mestre**, e é exatamente aí que a fricção aparece — IP manual no Foundry, escolha de sistema/compêndio no Roll20.

## 2. Fluxo de criação de mesa — proposta simplificada

Wizard de 3 passos, sem nenhuma tela de configuração solta antes disso:

1. **Nome da mesa** — único campo obrigatório. Sistema já é Tormenta20, não é uma escolha.
2. **Convite gerado automaticamente** — gerar um **código curto** (tipo "URSO-4821") + QR code. Na LAN, o app do jogador descobre as mesas por anúncio local e usa o código para selecionar a correta; o QR contém também endereço, porta e segredo de pareamento. Sem descoberta automática, o jogador pode ler o QR. O código não promete conexão pela internet.
3. **Pronto** — mesa criada e já aberta, vazia, pronta pra receber conteúdo. Não force o mestre a subir mapa/token antes de poder abrir a mesa; isso pode ser feito com jogadores já dentro.

Isso resolve o maior ponto de fricção real do Foundry dentro da mesma rede sem exigir infraestrutura de nuvem. Relay, túnel e descoberta fora da LAN permanecem uma fase futura.

## 3. Populando a mesa — o que reduzir do Foundry

O Foundry tem token com 5 abas de configuração (identidade, aparência, luz, visão, resistências...). Isso é o oposto do que você quer. Para o mestre leigo:

- **Ficha de ameaça (NPC/monstro)**: vem do compêndio já pronta — arrastar da lista pro mapa cria o token automaticamente com os dados da ficha vinculados. Nenhuma configuração manual de token separada da ficha.
- **Ficha de jogador**: dois caminhos, como o Roll20 permite — (a) mestre cria e atribui a um jogador, ou (b) mestre convida e o próprio jogador cria a ficha (o app já valida tudo via `core`, então não tem risco de ficha inválida entrar na mesa).
- **Mapa**: upload de imagem + grid — sem "camadas" separadas como no Foundry. Paredes (pro sistema de visão) são desenhadas na mesma tela do mapa, uma ferramenta só.
- **Token = ficha, sempre.** Não existe token "solto" sem ficha por trás. Isso elimina uma categoria inteira de configuração que o Foundry expõe (token desvinculado de actor).

## 4. Permissões — dois papéis, não cinco

Foundry tem 5 níveis (Bloqueado / Jogador / Confiável / Mestre Assistente / Mestre) e permissão por documento em cima disso. Isso é poder demais pra quem só quer jogar. Para o MVP:

- **Mestre**: controla tudo — todas as fichas, mapa, paredes, combate.
- **Jogador**: controla só a própria ficha (edita, rola dados, move o próprio token quando é sua vez ou quando permitido).

Na v1, o mestre escolhe por mesa entre movimento **livre** ou **somente no turno do jogador**. O servidor aplica essa regra; esconder ou desabilitar o controle apenas na interface não é considerado permissão.

Nada de granularidade por documento, nada de "confiável" intermediário. Se no futuro aparecer demanda real de um "mestre assistente" (co-mestre), isso entra como uma terceira opção depois — não precisa existir na v1.

## 5. Combate — automatizar o que hoje é manual nos dois concorrentes

Nos dois, adicionar alguém à iniciativa é uma ação manual (clique direito → adicionar). Dá pra simplificar:

- Ao entrar em modo de combate, todos os tokens **na cena atual** entram automaticamente na iniciativa (mestre pode remover individualmente, não precisa adicionar).
- Rolagem de iniciativa é um botão único "Rolar iniciativa de todos", que usa o bônus já calculado de cada ficha (`core`).
- Ordem de turno e "próximo turno" como uma barra fixa, sempre visível durante combate — sem precisar abrir um painel separado.

## 6. O que fica de fora do MVP (mas existe nos concorrentes)

Pra não inflar o escopo antes de ter uma mesa jogável:

- Compêndio de conteúdo de terceiros / marketplace — o compêndio aqui é só o conteúdo oficial de Tormenta20 que você já modelou.
- Jukebox/playlist de música.
- Vídeo/voz integrado (jogadores usam Discord por fora, como a maioria já faz mesmo com Foundry/Roll20 ligados).
- Imagens "wildcard" (variações aleatórias de arte por NPC) e anel dinâmico de token — são polimento visual, não afetam se a mesa funciona.
- Hospedagem via internet fora da LAN (relay/túnel) — v1 assume mestre e jogadores na mesma rede ou VPN externa; suporte a convite via internet é uma fase posterior, não bloqueia lançar algo jogável.

## 7. Resumo do fluxo completo (mestre leigo, do zero à mesa rodando)

1. Abre o app → "Criar mesa" → dá um nome → recebe código de convite.
2. Compartilha o código com os jogadores (voz, texto, QR).
3. Sobe um mapa (ou usa um já existente no compêndio, se você incluir alguns prontos).
4. Arrasta ameaças do compêndio pro mapa quando precisar.
5. Jogadores entram com o código, criam ou recebem suas fichas.
6. Sessão roda: rolagem pela ficha, movimento no mapa, combate com iniciativa automática.

Nenhum desses passos exige o mestre entender rede, permissão granular ou configuração de token — é a régua pra medir qualquer tela nova que for desenhada: se ela adiciona uma decisão que não está nesses 6 passos, ela deveria ser opcional/avançada, não parte do caminho principal.
