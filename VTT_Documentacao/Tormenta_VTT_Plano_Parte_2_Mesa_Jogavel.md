# Plano Parte 2 — mesa completa e jogável

**Estado:** plano funcional e de experiência  
**Data:** 5 de outubro de 2026  
**Continuação de:** `Tormenta_VTT_Objetivo_Navegacao_Movimento.md`  
**Escopo:** preparação, execução e encerramento de uma sessão de Tormenta20 para mestre e jogadores

> Implementação de 9 de outubro de 2026 — ferramentas por papel: a mesa usa uma barra comum nos três temas, com Selecionar, Arrastar, Desenhar, Texto, Medir, Dados e FX para jogadores; Ordem de turno, Alfinete e Névoa são adicionais do mestre. Desenhos, textos, alfinetes e medidas públicas permanentes pertencem ao mapa e são restaurados na reconexão. Medidas privadas permanentes são locais à sessão. A névoa aceita retângulos e polígonos, conserva as operações antigas e cobre o mapa com textura original azul-acinzentada opaca para o jogador. O mestre pode ajustar a transparência da prévia. A cor externa ao mapa é amostrada da imagem ou escolhida manualmente. FX só informa “em breve”; camadas e efeitos visuais ficam fora desta alteração. A névoa é proteção visual, não criptográfica: o arquivo original do mapa ainda chega ao cliente.

## 1. Objetivo

Entregar uma mesa em que um mestre consiga preparar uma sessão, convidar o grupo, apresentar cenas, controlar ameaças, arbitrar testes, conduzir combate, revelar informação e distribuir recompensas sem depender de outra ferramenta além da chamada de voz.

O jogador deve conseguir entrar, reconhecer imediatamente sua ficha e token, declarar ações, rolar a partir da ficha, acompanhar o combate, conversar, receber itens e enxergar apenas o que seu personagem poderia perceber.

### Definição de “mesa jogável”

A mesa estará jogável quando os seguintes ciclos funcionarem de ponta a ponta:

1. **Preparar:** criar cena, colocar mapa, ameaças, paredes, luzes, notas e handouts.
2. **Reunir:** abrir mesa, compartilhar convite, aprovar jogadores e vincular fichas.
3. **Explorar:** mover tokens, medir, apontar, revelar o mapa e alternar cenas.
4. **Interpretar:** conversar, sussurrar, mostrar imagens/textos e consultar regras.
5. **Resolver:** pedir um teste, rolar dados, identificar autor, propósito e visibilidade do resultado.
6. **Combater:** iniciar encontro, ordenar iniciativa, controlar turnos, PV, PM e condições.
7. **Recompensar:** entregar itens ou tesouro e registrar a alteração na ficha.
8. **Encerrar:** salvar toda a mesa, preservar o histórico e retomar de onde parou.

Vídeo e voz integrados não são necessários para esse objetivo. O grupo pode continuar usando Discord ou ferramenta equivalente.

## 2. O que a pesquisa indica

O [Resumo de Regras oficial de Tormenta20](https://jamboeditora.com.br/wp-content/uploads/2023/06/Tormenta20-Resumo-de-Regras.pdf) descreve a sessão como um ciclo de cena, declaração de ação, arbitragem, rolagem e consequência. Isso significa que o VTT não pode ser apenas um mapa: precisa ajudar o mestre a estabelecer cenas e reagir rapidamente às escolhas dos jogadores.

O capítulo de ameaças e a [wiki de Ameaças T20](https://tsrd.fandom.com/pt-br/wiki/Amea%C3%A7as_T20) mostram que uma ameaça usa um bloco próprio, menor que uma ficha de personagem: ND, tipo, tamanho, papel de combate, iniciativa, percepção, Defesa, resistências, PV, deslocamento, PM quando necessário, ações, atributos, perícias, equipamento e tesouro. Reaproveitar a ficha completa de jogador para monstros criaria trabalho desnecessário para o mestre.

O método de preparação do [Sly Flourish](https://slyflourish.com/eight_steps_2023.html) reforça que a ferramenta do mestre deve favorecer personagens, começo forte, cenas, segredos, locais, NPCs, monstros e recompensas. A Área do Mestre deve seguir essas tarefas reais, não categorias técnicas do banco de dados.

O [Foundry](https://foundryvtt.com/kb/) comprova a utilidade de cenas, atores, itens, diários, playlists, combate, paredes e iluminação; mas também demonstra o custo de expor cada configuração separadamente. Este projeto deve oferecer os mesmos resultados essenciais com decisões e papéis reduzidos.

O [Owlbear Rodeo](https://docs.owlbear.rodeo/docs/fog/) mostra uma boa alternativa para início rápido: fog manual continua útil mesmo quando iluminação dinâmica existe. O mestre precisa poder preencher o mapa com névoa, recortar uma sala e visualizar a perspectiva do jogador sem primeiro desenhar todas as paredes.

O sistema de dados do [Foundry](https://foundryvtt.com/article/dice/) também confirma uma boa convenção para o chat: toda rolagem vira um evento próprio, com visibilidade e detalhes expansíveis, em vez de ser tratada como texto comum.

## 3. Estado atual do projeto

| Área | Já existe | Falta para ficar completa |
|---|---|---|
| Sala | criar, entrar, reconectar, aprovar, revogar, histórico recente | código/QR simplificado, estado de prontidão e retomada explícita da sessão |
| Fichas | ficha de jogador, envio, aprovação, atribuição, cópia local | ficha própria de ameaça, recursos rápidos e ações clicáveis durante a sessão |
| Mapa | upload, grade, escala, snap, troca, câmera | conceito de cena, duplicação, preview do jogador e estado persistido de preparação |
| Tokens | colocar, mover, redimensionar, atribuir controle, retirar | PV/PM visíveis, condições, alvo, ocultação, vínculo com ameaça e estado por instância |
| Ferramentas | seleção, medida, ping | desenho, fog, paredes, portas e luzes |
| Chat | texto público/GM, rolagem por comando e dados básicos | mensagens tipadas, cards de rolagem, pedidos de teste, mensagens de sistema e privacidade clara |
| Combate | rolagem e aplicação confirmada de dano | encontro, iniciativa, rodadas, turnos, condições, cura e ações ligadas às fichas |
| Conteúdo | compêndio parcial, artes, áudio e playlist | ameaças, itens compartilháveis, tesouro, handouts e notas do mestre |
| Visão | mapa completo para participantes | fog manual, exploração, paredes, iluminação, visão por token e sombras |

## 4. Arquitetura de experiência

### 4.1 Dois momentos para o mestre

```text
FORA DA SESSÃO: PREPARAR
Área do Mestre em página inteira
├─ Hoje
├─ Cenas
├─ Ameaças e encontros
├─ Itens e handouts
└─ Jogadores

DURANTE A SESSÃO: CONDUZIR
Mapa dominante + painel rápido do mestre
├─ Trocar cena
├─ Revelar/ocultar
├─ Iniciar ou avançar combate
├─ Mostrar handout
├─ Colocar ameaça
└─ Ver jogador como...
```

A preparação completa não deve cobrir a mesa durante a sessão. O mestre terá um painel lateral rápido, usando os mesmos registros preparados anteriormente.

### 4.2 Área do Mestre proposta

#### Hoje

- cena ativa e próxima cena;
- participantes conectados, ficha vinculada e estado de conexão;
- encontro preparado ou combate em andamento;
- atalhos para “Mostrar handout”, “Colocar ameaça” e “Iniciar combate”;
- checklist curto: mapa, jogadores, fichas e cena prontos.

#### Cenas

- lista com miniatura, nome e estado: privada, pronta ou ativa;
- criar cena narrativa sem grade ou cena tática com mapa;
- duplicar cena para reutilizar paredes, luz e grade;
- editar mapa, escala, grade, fog, paredes, portas e iluminação;
- vincular música, handouts e encontro;
- botão permanente “Visualizar como jogador”.

#### Ameaças e encontros

- buscar ameaça pronta ou criar uma ameaça personalizada;
- organizar por recentes, favoritas e cena, sem sistema genérico de pastas;
- montar encontro arrastando ameaças e definindo quantidade;
- exibir ND do encontro como apoio, sem impedir decisão manual do mestre;
- colocar todo o encontro no mapa com uma ação.

#### Itens e handouts

- buscar item no compêndio;
- criar item personalizado simples;
- preparar tesouro e entregar a uma ficha ou criar pilha de saque;
- cadastrar texto ou imagem e decidir quando mostrar aos jogadores;
- manter notas privadas separadas do conteúdo compartilhável.

#### Jogadores

- convite, aprovações e conexão;
- ficha e token atribuídos;
- regra de movimento: livre ou somente no turno;
- visão compartilhada do grupo ou individual;
- remover acesso e recuperar uma ficha enviada.

## 5. Ficha de ameaça

### 5.1 Não reutilizar a ficha de jogador

A ameaça precisa de uma ficha própria, baseada na ordem de leitura do bloco oficial:

1. **Identidade:** nome, ND, tipo/subtipo, tamanho e papel de combate (solo, lacaio ou especial).
2. **Leitura rápida:** Iniciativa, Percepção/sentidos, Defesa, Fortitude, Reflexos e Vontade.
3. **Recursos:** PV atual/máximo, PM somente quando usados e deslocamentos.
4. **Ações:** ataques, habilidades, reações e recargas, cada uma com custo e tipo de ação.
5. **Atributos e perícias relevantes.**
6. **Defesas especiais:** imunidades, resistências e redução de dano.
7. **Equipamento, tesouro e notas táticas privadas.**

### 5.2 Experiência durante a sessão

- abrir como painel lateral, nunca como página que esconde o mapa;
- ações importantes ficam no topo e podem ser roladas com um clique;
- PV pode ser alterado com `-12`, `+8` ou valor direto;
- aplicar condição usa busca pelo nome e mostra resumo do efeito;
- habilidades com recarga mostram “Disponível” ou a ação necessária para recuperar;
- notas táticas e tesouro permanecem invisíveis para jogadores;
- ameaça derrotada pode permanecer no mapa como marcador ou ser retirada.

### 5.3 Modelo necessário

- **Ameaça base:** dados reutilizáveis e imutáveis durante o encontro.
- **Instância de ameaça:** PV, PM, condições, nome e estado próprios de cada token.

Isso permite colocar quatro lacaios iguais sem compartilhar PV entre eles, mantendo uma única ficha de origem.

### 5.4 Aceite

- mestre cria uma ameaça personalizada em menos de dois minutos;
- arrastar uma ameaça cria token e instância vinculados;
- duplicar ameaça não duplica trabalho de edição;
- ataque, dano, teste de resistência e habilidade geram rolagens identificadas no chat;
- nenhum campo exclusivo de personagem é exigido.

## 6. Encontros, iniciativa e combate

### 6.1 Começar combate

“Iniciar combate” usa os tokens da cena como candidatos e abre uma confirmação compacta. Personagens e ameaças são pré-selecionados; marcadores e NPCs neutros ficam de fora.

Por padrão, seguir Tormenta20:

- cada personagem tem sua iniciativa;
- ameaças entram como um grupo usando uma única iniciativa, com o menor bônus entre elas;
- o mestre pode dividir ameaças em grupos ou informar um valor manual quando necessário.

### 6.2 Barra de combate

- aparece somente quando existe encontro ativo;
- mostra rodada, participante atual, próximo e condições relevantes;
- possui “Anterior”, “Próximo turno”, “Pausar” e “Encerrar combate”;
- destaca o token atual sem mover a câmera à força;
- jogador pode encerrar apenas o próprio turno;
- entrada tardia adiciona o combatente para a rodada seguinte.

### 6.3 Ações e resolução

Fluxo recomendado:

```text
Selecionar ação → escolher/alvo já selecionado → rolar ataque ou teste
→ chat mostra resultado → rolar dano/efeito → mestre confirma aplicação
```

A confirmação humana deve continuar existindo. Cobertura, imunidades, manobras, crítico e efeitos situacionais tornam a aplicação totalmente automática frágil.

### 6.4 Estado do combatente

- PV, PV temporário e PM;
- condições com duração em turnos, rodadas, cena ou manual;
- Defesa e resistências visíveis ao mestre;
- anel de alvo e destaque do turno;
- barras de recurso configuradas por um único padrão da mesa: dono, todos ou ocultas;
- histórico de dano, cura, condição e gasto de recurso.

### 6.5 Fora do combate

O VTT não deve impor turnos durante exploração ou interpretação. Medição, movimento e rolagens continuam livres conforme a permissão da mesa.

## 7. Cenas, mapas e preparação

O registro atual de mapa deve evoluir para **cena**, agregando:

- imagem ou cor de fundo;
- grade, escala e snap;
- posição inicial de câmera;
- tokens e marcadores;
- fog explorado;
- paredes, portas e janelas;
- nível de luz ambiente e fontes de luz/escuridão;
- encontro preparado;
- música, handouts e notas vinculadas.

### Tipos de cena

- **Narrativa:** arte, música e handouts; sem exigir grade ou token.
- **Tática:** mapa, escala, tokens, fog e combate.

O mestre não deve ser obrigado a configurar uma cena tática para interpretar uma conversa, viagem ou investigação.

### Ações essenciais

- criar, renomear, duplicar e apagar;
- manter privada ou ativar para todos;
- pré-carregar no cliente antes da troca;
- colocar tokens no ponto inicial;
- salvar automaticamente câmera e preparação;
- visualizar exatamente a versão do jogador.

## 8. Fog, paredes, iluminação e sombras

### 8.1 Camadas funcionais

1. **Fog manual:** preencher tudo, revelar/ocultar por retângulo, polígono ou pincel.
2. **Exploração:** área já vista permanece escurecida; área nunca vista fica oculta.
3. **Paredes:** segmentos que bloqueiam visão e luz.
4. **Portas:** parede com estado aberta/fechada; o mestre controla por padrão.
5. **Luz ambiente:** claro, penumbra ou escuro por cena.
6. **Fontes:** ponto de luz ou escuridão com raio, intensidade e cor opcional.
7. **Visão do token:** normal, visão na penumbra ou visão no escuro.
8. **Sombra:** máscara de visibilidade gerada pelas paredes entre token e fonte.

### 8.2 Ordem de implementação

Fog manual vem primeiro e sempre continua disponível. Ele permite jogar sem preparação técnica, como demonstra o [Owlbear Rodeo](https://docs.owlbear.rodeo/docs/fog/).

Depois entram paredes e portas. Iluminação dinâmica usa o mesmo conjunto de segmentos para calcular polígonos de visibilidade. O tutorial interativo [Sight & Light](https://ncase.me/sight-and-light/) descreve o raycasting aos extremos dos segmentos e serve como referência técnica.

O render já usa PixiJS. Máscaras `Graphics` e containers existentes cobrem o MVP; filtros pesados e centenas de máscaras devem ser evitados, conforme a [documentação do PixiJS](https://pixijs.com/8.x/guides/components/scene-objects).

### 8.3 Decisões de produto

- visão padrão é 360°, coerente com o RPG de mesa; cone direcional fica fora do MVP;
- o mestre escolhe visão compartilhada pelo grupo ou individual para toda a mesa, sem configuração por documento;
- fog manual pode complementar ou sobrescrever a visão dinâmica;
- o mestre vê fog em transparência e ameaças ocultas;
- luz e fog protegem contra revelação acidental, não contra um cliente modificado que inspecione o arquivo completo do mapa;
- “Visualizar como jogador” precisa estar disponível durante toda a preparação.

### 8.4 Aceite

- uma masmorra simples pode ser preparada apenas cobrindo o mapa e revelando salas;
- uma cena com paredes atualiza visão ao mover token ou abrir porta;
- o mestre nunca perde acesso visual ao conteúdo oculto;
- jogador não vê token, luz ou ping em área que não pode perceber;
- iluminação pode ser desativada por cena sem remover paredes.

## 9. Itens, tesouro e inventário

### 9.1 Fluxos essenciais

- arrastar item do compêndio para uma ficha;
- mestre entregar item diretamente a um jogador;
- criar pilha de saque na cena e permitir coleta confirmada;
- transferir item entre personagens;
- consumir item e reduzir quantidade;
- equipar/desequipar usando as regras já existentes da ficha;
- registrar no chat quem recebeu, transferiu ou consumiu;
- manter item personalizado com nome, quantidade, espaços, preço, descrição e fonte.

### 9.2 Limites

- não criar mercado, leilão, crafting ou economia automatizada agora;
- não exigir ilustração para item;
- item copiado para a ficha passa a ser uma cópia própria, sem mudar quando o compêndio for editado;
- o servidor valida quantidade, carga e permissão antes de persistir.

## 10. Notas, handouts e apoio à narrativa

O mestre precisa preparar informação que não é mapa nem ficha:

- nota privada de sessão;
- segredo ou pista curta;
- NPC sem bloco de combate: nome, aparência, voz/jeito e objetivo;
- handout de texto ou imagem;
- lembrete de CD ou decisão tomada;
- resumo da sessão anterior.

O formato inicial pode ser texto simples com listas e uma imagem opcional. Não é necessário um editor rico completo.

### Compartilhamento

- privado do mestre;
- visível a todos;
- mostrado momentaneamente a todos;
- enviado a jogadores selecionados.

Ao mostrar um handout, ele abre em primeiro plano para o jogador, mas pode ser fechado sem interromper a mesa. A inspiração é o fluxo de páginas compartilhadas do [Journal do Foundry](https://foundryvtt.com/article/journal/), sem replicar permissões por documento.

## 11. Dados animados e chat

### 11.1 Regra de autoridade

O servidor gera e registra o resultado primeiro. A animação apenas representa esse resultado. Física visual nunca decide o número e a rolagem continua válida se a animação falhar.

### 11.2 Animação recomendada

Primeira versão:

- overlay PixiJS sobre o mapa;
- dados 2D/2.5D com rotação, salto curto, colisão visual simples e repouso na face correta;
- duração de 700 a 1.100 ms;
- não bloqueia chat, ficha ou próxima rolagem;
- rolagens simultâneas são condensadas em uma única área, sem fila longa;
- clique, Espaço ou nova rolagem pula a animação atual;
- movimento reduzido troca a animação por uma revelação curta do resultado;
- rolagem privada não produz dado, som ou placeholder para quem não pode vê-la.

Uma implementação 3D com física, materiais e efeitos deve ser uma etapa opcional. O [Dice So Nice](https://foundryvtt.com/packages/dice-so-nice) mostra o potencial e também o tamanho desse produto: temas, materiais, sons, modelos e efeitos formam um sistema próprio. Não é necessário para tornar a mesa jogável.

### 11.3 Tipos de mensagem

O chat precisa deixar de salvar toda rolagem como texto. Cada entrada terá um tipo e dados estruturados:

- `message`: conversa normal;
- `roll`: fórmula, dados individuais, total, autor, origem e visibilidade;
- `action`: ataque, dano, cura, uso de item ou habilidade;
- `system`: entrada, saída, mudança de cena ou turno;
- `handout`: conteúdo compartilhado;
- `whisper`: conversa privada.

### 11.4 Diferença visual

| Mensagem normal | Rolagem |
|---|---|
| linha compacta, avatar/nome e texto | card com ícone de dado e borda/acento próprio |
| fundo neutro | total em destaque tipográfico |
| sem fórmula | nome da ação, fórmula e alvo |
| horário discreto | dados individuais expansíveis |
| não ocupa toda a largura sem necessidade | crítico e falha com cor, ícone e texto, nunca só cor |

Exemplo de hierarquia de card:

```text
[d20] Arthen • Ataque com Espada Longa     TODOS
      24
      1d20 (18) + 6  · alvo: Guarda Purista
      [Rolar dano]
```

### 11.5 Visibilidade

- **Todos:** todos veem animação e card.
- **Mestre:** autor e mestre veem resultado.
- **Oculta:** somente o mestre vê o resultado; autor recebe apenas confirmação do envio.
- **Só eu:** somente o autor vê.

“Segredo” deve ser substituído por um desses nomes explícitos. A distinção segue o princípio de modos públicos, privados, cegos e pessoais usado pelo [Foundry](https://foundryvtt.com/article/dice/).

### 11.6 Pedidos de teste

O mestre pode pedir “Percepção CD 20” a todos ou a jogadores selecionados. Cada destinatário recebe um botão que usa a própria ficha; os resultados chegam agrupados no mesmo card para o mestre. Isso atende o fluxo real de arbitragem sem exigir macros.

## 12. Experiência do jogador

O jogador precisa ter sempre disponíveis:

- mapa ou arte da cena;
- token próprio, foco e estado de turno;
- botão para abrir a ficha sem abandonar a mesa;
- PV, PM, condições e ações favoritas;
- chat, rolagens e sussurro ao mestre;
- régua e ping;
- alvo atual;
- inventário e itens recebidos;
- handouts compartilhados;
- lista curta de participantes e conexão;
- indicação clara de “aguardando mestre”, “seu turno”, “sem conexão” e “reconectado”.

O jogador não precisa ver ferramentas de parede, luz, preparação, biblioteca de ameaças ou dados privados de NPCs.

## 13. Estados e falhas que precisam existir

- jogador reconecta e recupera cena, combate, chat, áudio e ficha atuais;
- troca de cena espera o novo mapa estar disponível antes de remover o anterior;
- falha ao carregar arte usa iniciais sem quebrar o token;
- rolagem salva mesmo se a animação falhar;
- conflito de movimento mantém a posição confirmada pelo servidor;
- alteração de PV/PM mostra quem fez e permite correção pelo mestre;
- sair no meio do combate não remove o combatente automaticamente;
- apagar cena, ameaça ou handout exige confirmação e não remove arquivo ainda usado em outro lugar;
- mestre consegue exportar uma cópia da mesa antes de apagar conteúdo material.

## 14. Plano de implementação

### Entrega 2.1 — base de sessão

1. Transformar chat em histórico tipado mantendo mensagens antigas compatíveis.
2. Transformar mapa em cena sem alterar o fluxo atual de upload.
3. Persistir estado ativo de sessão e combate.
4. Criar preview do jogador.

**Aceite:** reconexão restaura a mesma cena e o histórico distingue conversa, rolagem e sistema.

### Entrega 2.2 — fatia vertical de combate

1. Ficha de ameaça.
2. Ameaça base e instâncias independentes.
3. Encontro e iniciativa de Tormenta20.
4. Barra de rodada/turno.
5. PV, PM, condições, alvo, ataque, dano e cura com confirmação.

**Aceite:** um mestre conduz do início ao fim um combate com quatro jogadores e múltiplas cópias da mesma ameaça.

### Entrega 2.3 — preparação e informação

1. Nova Área do Mestre orientada a Hoje, Cenas, Ameaças, Conteúdo e Jogadores.
2. Cena narrativa e cena tática.
3. Notas, pistas e handouts.
4. Entrega, coleta e transferência de itens.
5. Preparação de encontro e colocação em lote.

**Aceite:** a próxima sessão pode ser preparada sem manter anotações em outro aplicativo.

### Entrega 2.4 — visão, fog e iluminação

1. Fog manual e exploração.
2. Paredes e portas.
3. Visão por token.
4. Luz ambiente, fontes de luz/escuridão e sombras.
5. Testes de desempenho com mapas e quantidades realistas de paredes.

**Aceite:** mestre prepara e executa uma exploração de masmorra sem revelar salas ou ameaças antes da hora.

### Entrega 2.5 — dados e chat final

1. Cards de rolagem e ações.
2. Quatro modos de visibilidade.
3. Pedidos de teste.
4. Animação PixiJS 2D/2.5D.
5. Movimento reduzido, pular animação e privacidade de som/efeito.

**Aceite:** uma rolagem é reconhecida como rolagem sem ler o texto, nunca atrasa o jogo e não vaza informação privada.

### Entrega 2.6 — endurecimento

1. Testes de autorização no servidor para todas as novas ações.
2. Testes de reconexão no meio de combate e troca de cena.
3. Navegação por teclado e foco de drawers/handouts.
4. Limites de arquivo, quantidade e texto.
5. Backup/exportação da mesa e teste de restauração.
6. Teste completo mestre + dois jogadores em processos separados.

## 15. Dependências entre entregas

```text
Histórico tipado + Cena
        ├─ Ficha de ameaça → Encontro → Combate
        ├─ Fog → Paredes → Visão → Luz e sombras
        ├─ Itens → Tesouro → Transferência
        └─ Rolagem estruturada → Card → Animação dos dados
```

Não começar iluminação ou animação 3D antes de cena e rolagem estruturada estarem estáveis.

## 16. Design ou assets que precisam vir de fora

| Necessidade | Precisa que você envie? | Alternativa sem asset externo |
|---|---|---|
| Retratos e tokens de ameaças oficiais | **Sim**, se devem vir incluídos no app; precisam ser licenciados/autorizados | iniciais e upload feito pelo mestre |
| Mapas e artes de cena | **Sim**, para conteúdo pronto incluído | upload do próprio usuário |
| Imagens de itens e handouts | Somente se quiser biblioteca ilustrada | lista textual e imagem opcional |
| Conjunto visual de condições | Recomendado se quiser ícones próprios de Tormenta | abreviação textual + cor + tooltip |
| Aparência dos dados 2D | Não para a primeira versão | formas e números procedurais no PixiJS |
| Dados 3D com materiais próprios | **Sim**, caso queira identidade forte: referência, modelos/texturas e direção de material | manter dados 2D/2.5D |
| Sons de dados, portas e interface | **Sim**, com arquivos licenciados e preferência de intensidade | versão silenciosa |
| Efeitos de magia/ataque | **Sim**, se forem desejados no futuro | feedback por token, chat e cor; VFX fora do MVP |
| Layout exato inspirado no livro | Envie referência aprovada se quiser proximidade visual | ficha original baseada apenas na hierarquia dos campos |

### Materiais que seria útil você fornecer antes da implementação visual final

1. Uma referência do estilo desejado para a ficha de ameaça.
2. Uma referência de dados: sóbrios e físicos, mágicos, metálicos ou no estilo atual da Guilda.
3. Um pequeno conjunto licenciado de 6–10 tokens e 2 mapas para teste visual real.
4. Caso deseje ícones de condições próprios, a lista ou o pacote visual correspondente.

Esses materiais não bloqueiam a implementação funcional. Fallbacks sem arte permitem construir e testar todos os fluxos primeiro.

## 17. Fora do objetivo desta parte

- vídeo e voz;
- marketplace e módulos de terceiros;
- macros programáveis;
- automação completa de todas as regras narrativas;
- co-mestre e permissões por documento;
- dados 3D como requisito de lançamento;
- proteção criptográfica do mapa contra clientes hostis;
- efeitos visuais de cada magia;
- internet pública com relay próprio, mantendo o escopo de LAN/VPN definido no projeto.

## 18. Critério final de lançamento

Convidar quatro jogadores, preparar uma cena narrativa e uma tática, colocar ameaças, explorar com fog, iniciar combate, rolar ataques e testes, aplicar dano e condições, entregar um item, compartilhar um handout, trocar de cena, perder e recuperar uma conexão e reabrir a mesa no dia seguinte sem perda de estado.

Se esse roteiro funcionar com interface clara e sem instruções externas, a mesa atingiu o objetivo desta Parte 2.

## 19. Referências de implementação e produto

- [Tormenta20 — Resumo de Regras oficial](https://jamboeditora.com.br/wp-content/uploads/2023/06/Tormenta20-Resumo-de-Regras.pdf)
- [Ameaças T20 — Tormenta RPG Wiki](https://tsrd.fandom.com/pt-br/wiki/Amea%C3%A7as_T20)
- [Fichas de Nimb — repositório e inventário de ferramentas](https://github.com/YuriAlessandro/gerador-ficha-tormenta20)
- [Sly Flourish — Eight Steps of the Lazy DM](https://slyflourish.com/eight_steps_2023.html)
- [Foundry — Knowledge Base](https://foundryvtt.com/kb/)
- [Foundry — Combat Encounters](https://foundryvtt.com/article/combat/)
- [Foundry — Tokens](https://foundryvtt.com/article/tokens/)
- [Foundry — Items](https://foundryvtt.com/article/items/)
- [Foundry — Journal Entries](https://foundryvtt.com/article/journal/)
- [Foundry — Lighting](https://foundryvtt.com/article/lighting/)
- [Foundry — Basic Dice](https://foundryvtt.com/article/dice/)
- [Roll20 — Dynamic Lighting](https://help.roll20.net/hc/en-us/sections/360008317214-How-to-Use-Dynamic-Lighting)
- [Owlbear Rodeo — Fog](https://docs.owlbear.rodeo/docs/fog/)
- [Sight & Light — polígonos de visibilidade 2D](https://ncase.me/sight-and-light/)
- [PixiJS v8 — Scene Objects, máscaras e filtros](https://pixijs.com/8.x/guides/components/scene-objects)
- [Dice So Nice — referência de produto para dados 3D](https://foundryvtt.com/packages/dice-so-nice)

