# Objetivo de produto — navegação limpa e movimento suave

**Estado:** contexto de objetivo e plano de melhoria  
**Data:** 5 de outubro de 2026  
**Escopo:** shell do aplicativo, fichas, mesa virtual, painéis, modais e transições  
**Fora de escopo:** regras de Tormenta 20, persistência, rede, criação de novas funcionalidades e uma troca ampla da identidade visual

**Continuação funcional:** `Tormenta_VTT_Plano_Parte_2_Mesa_Jogavel.md`

## Objetivo

Fazer o usuário sempre entender **onde está, o que mudou e como voltar**, mantendo o mapa ou a ficha como foco principal. A interface deve reagir de forma imediata; o movimento apenas explica uma mudança de contexto, sem atrasar ações frequentes.

### Resultado esperado

- uma navegação global clara para Resumo, Mesas, Fichas e Livros;
- um cabeçalho local consistente ao abrir uma ficha ou mesa;
- no máximo um painel contextual aberto por vez;
- transições curtas, previsíveis e interrompíveis;
- comportamento equivalente com movimento reduzido;
- nenhuma dependência nova de animação.

## Leitura do produto atual

Hoje existem três modelos de navegação que parecem pertencer a produtos diferentes:

1. o hub usa abas horizontais abaixo do cabeçalho;
2. a ficha troca essas abas por uma barra própria;
3. a mesa usa cabeçalho, trilho lateral, painel direito e uma configuração com outro conjunto de abas.

Cada área funciona isoladamente, mas a troca entre elas exige que o usuário reaprenda onde ficam “voltar”, ferramentas e ações primárias. O movimento também não forma um sistema: há entradas animadas sem saída correspondente, durações e curvas diferentes, trocas secas de conteúdo e uma animação Svelte que não é coberta pelas regras CSS de movimento reduzido.

## Referências comparadas

| Referência | O que funciona | O que não copiar |
|---|---|---|
| VTT atual | Identidade cartográfica própria, mapa dominante na mesa, boa densidade para ficha | Barras de navegação concorrentes, troca abrupta de contexto, muitos destinos no painel direito |
| [Fichas de Nimb](https://fichasdenimb.com.br/) | Categorias nomeadas, ícones com texto e atalhos de ferramentas visíveis | Menu muito longo e o mesmo destino exposto em vários lugares; isso aumenta ruído e manutenção |
| [Owlbear Rodeo](https://docs.owlbear.rodeo/docs/getting-started/) | Canvas como centro, ferramentas nas bordas, painel apenas quando necessário e conteúdo próximo da ação | Não reduzir o VTT a um mapa simples: ficha, regras e materiais continuam sendo parte essencial deste produto |

A direção recomendada combina a descoberta explícita do Fichas de Nimb com o foco espacial do Owlbear, sem reproduzir toda a arquitetura de nenhum deles.

## Melhorias priorizadas

| Prioridade | Antes | Depois | Por quê |
|---|---|---|---|
| P0 | Cabeçalho global, abas do hub e barras locais competem | Um shell global no hub e um único cabeçalho local em superfícies de trabalho | Reduz a troca de modelo mental |
| P0 | Entradas animadas, saídas abruptas e curvas diferentes | Quatro padrões de movimento compartilhados | Torna a mudança de estado previsível |
| P0 | `prefers-reduced-motion` cobre CSS, mas não a transição `fly` do Svelte nem todos os efeitos | Movimento reduzido tratado tanto no CSS quanto no Svelte | Acessibilidade precisa funcionar por inteiro |
| P1 | Sete destinos disputam a faixa do painel direito da mesa | Destinos agrupados em Jogo e Biblioteca, com conteúdo específico dentro do grupo | Menos decisões simultâneas sem esconder recursos |
| P1 | Mapas e Área do Mestre aparecem como ações paralelas embora se relacionem com preparação | Ações de preparação ficam no mesmo contexto de mestre | Elimina caminhos duplicados |
| P1 | Painéis vazios ocupam grande área no hub | Estado vazio compacto, ação primária e itens recentes quando existirem | Melhora a orientação sem inventar conteúdo |
| P2 | A mudança de aba ou superfície é um corte seco | Conteúdo faz uma transição discreta de opacidade e 4–8 px | Mantém continuidade espacial |
| P2 | A lateral da mesa anima `grid-template-columns` | Painel se move com `transform` e `opacity`; o layout não é animado | Evita trabalho de layout durante a animação |

## Arquitetura de navegação proposta

```text
Hub
└─ Cabeçalho global
   ├─ Marca
   ├─ Resumo · Mesas · Fichas · Livros
   └─ Configurações

Superfície de trabalho
└─ Cabeçalho local único
   ├─ Voltar
   ├─ Título + estado de salvamento/conexão
   └─ Uma ação primária e ações secundárias essenciais
      ├─ Ficha: conteúdo + ajudante opcional
      └─ Mesa: mapa + ferramentas + um painel contextual
```

O cabeçalho global não precisa permanecer dentro da mesa: a mesa deve continuar imersiva. O padrão que precisa permanecer é uma saída sempre visível no mesmo lado, um título/estado legível e ações previsíveis à direita.

### Mesa virtual

- manter o mapa como maior área visual;
- manter o trilho de ferramentas na borda;
- abrir somente um painel contextual à direita;
- agrupar o painel em **Jogo** (Chat, Fichas, Participantes) e **Biblioteca** (Compêndio, Artes, Música, Tokens);
- deixar opções de preparação e mapa no contexto de mestre, evitando comandos duplicados no topo;
- mostrar rótulo em tooltip para ferramentas apenas com ícone e preservar foco por teclado.

### Ficha

- usar o mesmo padrão de “Voltar / título e estado / ações” da mesa;
- manter o compêndio como ajudante lateral, não como segunda navegação;
- preservar a densidade atual e o estado dos campos ao trocar de seção;
- evitar movimento em edição, rolagem e comandos de teclado repetidos.

## Sistema mínimo de movimento

Não é necessário instalar uma biblioteca. CSS e as transições nativas do Svelte já cobrem o objetivo.

| Padrão | Duração | Uso |
|---|---:|---|
| Instantâneo | 0 ms | teclado, seleção repetida de ferramenta e estados que pedem resposta imediata |
| Micro | 120–140 ms | hover, pressionar, foco e indicador ativo |
| Contexto | 160–200 ms | troca de aba, estado vazio/conteúdo e ajudante contextual |
| Painel | 180–240 ms na entrada; 120–180 ms na saída | drawers, modais e painel da mesa |

Regras:

- entrada usa desaceleração forte, por exemplo `cubic-bezier(.16, 1, .3, 1)`;
- movimento dentro da tela usa `ease-in-out`;
- saída é mais curta que a entrada;
- animar apenas `transform` e `opacity` nas transições principais;
- distância máxima de deslocamento: 8–16 px para conteúdo, largura do próprio painel para drawers;
- não animar toda propriedade com `transition: all`;
- em movimento reduzido, remover escala e deslocamento, mantendo somente a mudança instantânea ou uma opacidade curta;
- controlar transições Svelte com `prefersReducedMotion` de `svelte/motion`, pois a preferência do sistema não desativa automaticamente uma diretiva `transition:`.

## Plano de execução

### Entrega 1 — fundação de movimento

1. Definir tokens de duração, distância e easing em `src/app.css`.
2. Aplicar esses tokens aos componentes já existentes, sem criar uma camada nova de animação.
3. Corrigir movimento reduzido no `fly` do painel da mesa e no ping do mapa.
4. Remover a animação de propriedade de layout do painel lateral.

**Aceite:** nenhuma ação rotineira leva mais de 240 ms; abrir e fechar têm continuidade; movimento reduzido não produz translação ou escala relevante.

### Entrega 2 — shell e transições do hub

1. Integrar Resumo, Mesas, Fichas e Livros ao cabeçalho global no desktop.
2. Remover a segunda faixa de navegação do hub.
3. Usar uma transição discreta de contexto entre áreas, preservando foco e estado.
4. Compactar estados vazios e destacar uma ação primária por tela.

**Aceite:** qualquer área do hub abre com um clique; a área atual é óbvia; Tab e Shift+Tab percorrem os controles em ordem visual.

### Entrega 3 — ficha e mesa

1. Padronizar o cabeçalho local da ficha e da mesa.
2. Consolidar o painel direito da mesa em Jogo e Biblioteca.
3. Mover ações duplicadas de mapa/preparação para o contexto de mestre.
4. Garantir apenas um overlay, drawer ou painel transitório por vez.

**Aceite:** “voltar” está sempre visível; o mapa continua dominante; não há duas camadas bloqueando a mesma superfície.

### Entrega 4 — verificação

1. Testar mouse, teclado, Esc, restauração de foco e leitor de tela básico.
2. Testar movimento normal e reduzido.
3. Verificar animações no painel de desempenho do navegador, procurando trabalho de layout e pintura durante a transição.
4. Capturar antes/depois nas larguras usadas pelo aplicativo desktop.

**Aceite:** nenhuma regressão em criação/edição de ficha, entrada/criação de mesa ou troca de mapa; transições principais usam apenas composição; nenhuma dependência foi adicionada.

## Arquivos prováveis

- `src/app.css`: tokens e padrões de movimento;
- `src/App.svelte`: shell, troca de áreas, ficha e overlays globais;
- `src/lib/components/Mesa.svelte`: navegação e configuração da mesa;
- `src/lib/components/MesaTable.svelte`: painel contextual, movimento reduzido e controles da mesa.

## Guardas de escopo

- não trocar tipografia, cores ou linguagem visual nesta rodada;
- não criar um framework interno de animação;
- não adotar View Transition API antes de CSS/Svelte se mostrarem insuficientes;
- não alterar regras, formato de dados ou rede;
- não antecipar recursos além do MVP documentado.

## Leituras para implementação

- [Material Design — Duration & easing](https://m1.material.io/motion/duration-easing.html): referência prática para movimento curto e responsivo em desktop.
- [Material Design — Understanding motion](https://m2.material.io/design/motion/understanding-motion/): movimento como explicação de relação, disponibilidade e resultado.
- [web.dev — Animations and performance](https://web.dev/articles/animations-and-performance/): por que priorizar `transform` e `opacity`.
- [web.dev — How to create high-performance CSS animations](https://web.dev/articles/animations-guide): roteiro de medição e depuração.
- [Svelte — transition](https://svelte.dev/docs/svelte/transition): ciclo das transições e observação sobre movimento reduzido.
- [MDN — prefers-reduced-motion](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/%40media/prefers-reduced-motion): implementação da preferência do sistema.
- [W3C — Animation from Interactions](https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions): requisito de permitir desativação de movimento disparado por interação.
- [Nielsen Norman Group — Recognition Rather Than Recall](https://www.nngroup.com/articles/recognition-and-recall/): base para rótulos visíveis, agrupamento e navegação reconhecível.

## Decisão recomendada

Começar pela **Entrega 1**. Ela corrige inconsistências e acessibilidade com o menor risco, cria a régua de qualidade para as etapas seguintes e evita redesenhar a navegação enquanto o comportamento básico ainda varia entre componentes.
