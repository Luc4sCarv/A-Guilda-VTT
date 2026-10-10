# A Guilda VTT

Prévia para testes de mesa de Tormenta20 no Windows 10/11. Um mesmo aplicativo serve para mestre e jogadores.

## Instalar e jogar

1. Na página **Releases**, baixe o instalador Windows que termina em `_x64-setup.exe` e execute-o. Não é preciso instalar Node.js ou Rust.
2. O mestre cria ou abre uma mesa e compartilha o endereço, a porta e o código de convite mostrados no aplicativo.
3. Cada jogador instala o mesmo programa, escolhe **Entrar com convite** e aguarda a aprovação do mestre.

Por enquanto, a conexão é direta por rede local ou VPN entre amigos; não há servidor público de partidas. Se o Windows solicitar acesso à rede para o aplicativo do mestre, permita apenas nas redes em que pretende jogar. Os dados da mesa ficam no computador do mestre. Faça uma cópia dos seus dados antes de reinstalar o Windows ou trocar de computador.

Esta é uma prévia não assinada digitalmente. O Windows pode mostrar um aviso do SmartScreen ao baixar ou abrir o instalador. Confirme que o arquivo veio desta página de Releases; se não reconhecer a origem, não o execute.

## Estado desta prévia

Há fichas, cenas e mapas, chat e dados, combate, ameaças, desenhos, textos, medidas, alfinetes e névoa manual. Efeitos FX e camadas ainda não estão ativos. A névoa evita revelações na interface normal, mas não protege o arquivo do mapa contra inspeção deliberada.

Projeto de fãs não oficial. Livros e PDFs de Tormenta20 não são distribuídos neste repositório nem no instalador.

## Desenvolvimento

Requer Node.js, Rust e os pré-requisitos do Tauri para Windows. Instale as dependências com `npm ci`, execute `npm run check`, `npm test` e `cargo test -p tormenta20-vtt`; `npm run tauri build` gera o instalador NSIS em `target/release/bundle/nsis/`.
