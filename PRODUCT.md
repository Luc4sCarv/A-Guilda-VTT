# A Guilda VTT — produto

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Mestres preparam e conduzem sessões de Tormenta20; jogadores mantêm suas fichas e participam da mesma mesa durante a sessão.

## Product Purpose

Reunir ficha, preparação do mestre e mesa virtual jogável em um aplicativo desktop, mantendo mapa ou ficha como foco e deixando claro onde o usuário está e como voltar.

## Operating Context

O aplicativo Svelte/Tauri funciona no Windows e usa um servidor embutido para mesas locais ou por VPN. Durante uma sessão, mestre e jogadores alternam entre mapa, fichas, chat, rolagens, conteúdo e ferramentas contextuais.

## Capabilities and Constraints

- O projeto segue o MVP em `VTT_Documentacao/`; temas não mudam regras, permissões, rede nem dados de campanhas.
- Um conjunto compartilhado de componentes deve servir aos temas Moderno, Arcaico e Nimb.
- Preferências de aparência são locais ao dispositivo. Instalações anteriores preservam a escolha claro/escuro/sistema no Arcaico; novas instalações iniciam no Moderno.
- Navegação por teclado, foco visível, contraste e preferência por movimento reduzido devem continuar funcionais.

## Brand Commitments

- O nome do produto é **A Guilda VTT**.
- Os três temas usam a composição aprovada do Nimb e a arte original do início. Moderno usa cores escuras e cantos arredondados de aplicativos Windows; Arcaico preserva sua paleta e acabamento cartográfico; Nimb mantém a linguagem clara e carmesim das referências, sem reutilizar logotipo ou ilustrações de terceiros.

## Evidence on Hand

- Implementação e documentação locais em `src/` e `VTT_Documentacao/`.
- Três capturas fornecidas pelo usuário: VS Code escuro, ficha Fichas de Nimb e página inicial Fichas de Nimb.

## Product Principles

1. A tarefa da sessão permanece mais importante que o ornamento.
2. Mudança de tema preserva todos os dados e ações.
3. A interface explica localização, estado e retorno sem duplicar destinos.
