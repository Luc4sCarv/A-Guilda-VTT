# Ficha correta — execução e auditoria

Data: 2026-10-08. Estado: **parcial; não homologada como 100% correta**.

## Objetivo aprovado

Livro Básico Jogo do Ano e suplementos oficiais disponíveis em RPG_Livros.
Automação assistida: cálculos determinísticos automáticos, decisões narrativas
confirmadas pelo mestre. Nimb orienta o fluxo; wikis orientam a pesquisa;
livro e correções da edição correspondente resolvem divergências.
Ponytail full reativado por solicitação posterior do usuário. Reutilizar o fluxo
da mesa sem reduzir validações de segurança ou declarar cobertura inexistente.

Não incluir automaticamente Tormenta RPG antigo, material experimental,
Dragão Brasil ou homebrew. Os planos de navegação e mesa permanecem,
subordinados à confiabilidade da ficha.

## Conferência realizada

- Livro Básico local, página impressa 17 / página PDF 23: compra de 10 pontos;
  custos para -1, 0, 1, 2, 3, 4 são -1, 0, 1, 2, 4, 7.
- A compra é anterior aos bônus; não existe nesta correção um teto 4 para o
  atributo final. A mesma página descreve outro método por rolagens, ainda
  não implementado no editor atual. Não representar rolagens como compra.
- Wiki consultada: https://tormenta20.fandom.com/pt-br/wiki/Atributos_B%C3%A1sicos
- Livro Básico local, página impressa 152 / PDF 158: impossibilidade de
  sobrepor armaduras e usar dois escudos; confirma a proteção de equipamento.
- Fluxo Nimb observado: https://fichasdenimb.com.br/criar-ficha
- Documentação Nimb: https://github.com/YuriAlessandro/gerador-ficha-tormenta20
- A errata em https://jamboeditora.com.br/wp-content/uploads/2023/07/tormenta20-errata.pdf
  é internamente de julho de 2022; sua aplicabilidade ao Jogo do Ano não deve
  ser presumida pela data do endereço.

## Correções implementadas neste lote

| Falha | Proteção | Verificação |
| --- | --- | --- |
| Seis atributos em 4 eram aceitos | Orçamento na edição, salvar do navegador e validação Rust compartilhada por salvar local/enviar/aceitar/atribuir/salvar na mesa | Testes unitários e tentativa forjada por WebSocket |
| Nível textual/fracionário virava 1 | Rejeitar tipo inválido quando o campo está presente | Teste de fronteira |
| Escolhas raciais repetidas, fora da faixa ou em raça sem escolha | Validar quantidade, índices e restrição do Lefou no servidor | Teste de fronteira |
| PM e recursos temporários negativos | Rejeitar no servidor; PV atual negativo continua permitido | Teste de fronteira |
| Efeito com custo negativo ou valor inválido | Validar formato no servidor e gasto local antes de ativar | Testes Rust e JavaScript |
| Objetos/texto no lugar de listas | Rejeitar estruturas inválidas nas coleções verificadas | Teste de fronteira |
| Moedas escapavam da carga quando equipment era omitido | Conferir moedas e carga mesmo sem lista de equipamento | Teste de fronteira |
| Duas armaduras/escudos equipados somavam RD no servidor | Rejeitar múltiplos itens equipados dessas categorias | Teste de fronteira |
| Quantidade fracionária ou zero equipado | Rejeitar no servidor | Teste de fronteira |
| Valores extremos de RD/índice de atributo | Validar formato e limites de armazenamento | Teste de fronteira |

Os limites de armazenamento não são regras de jogo e não certificam um bônus.
Ativações na mesa agora usam uma ação no servidor: o custo vem da versão
armazenada, não da mensagem do jogador. A ação exige dono/mestre, revisão atual,
identificador único e PM suficientes; gasta PM temporários primeiro. Repetição
da mesma revisão não cobra novamente. Desativar não devolve PM.
Fichas antigas não são apagadas nem reduzidas automaticamente: a compra acima
do orçamento fica sinalizada, pode ser reduzida e não pode ser salva até reparo.

## Brechas e lacunas ainda abertas — não esconder como concluídas

1. **Ajustes sem origem:** atributos temporários, efeitos personalizados, PV,
   Defesa e RD continuam editáveis no rascunho. Alterações mecânicas feitas pelo
   jogador agora exigem aprovação do mestre para mudar a ficha da mesa. O pedido
   guarda a versão anterior e a proposta, persiste após reconexão e só pode ser
   aprovado pelo mestre enquanto a revisão original continuar atual. Reenviar
   substitui o pedido e invalida seu identificador anterior. Isso mitiga o bypass,
   mas não comprova a regra: falta rastrear a origem de cada bônus automaticamente.
2. **Recursos pelo protocolo:** mudanças diretas de recursos via salvar ficam em
   revisão. Ativação básica de magia/efeito já debita PM no servidor e não aceita
   custo adulterado no pedido. Ainda faltam recuperação, modificadores de custo,
   manutenção, exceções ao limite básico de PM e histórico/desfazer de ações.
3. **Perícias:** marcação livre e troca livre do atributo-chave; faltam seleção
   por classe, orçamento de treinamento e exceções comprovadas por habilidade.
4. **Poderes e devoção:** catálogo parcial, repetição e pré-requisitos sem
   cobertura integral; restrições visuais não bastam como validação de servidor.
5. **Magias:** círculos acessíveis, aprendizado, escolas, aprimoramentos,
   limites de gasto, duração e sustentação ainda não estão completos.
6. **Inventário:** preços, espaços, bônus e materiais manuais não são derivados
   de IDs oficiais; faltam empunhadura, requisitos e interações completas.
7. **Progressão:** editor usa classe única e nível editável; histórico de escolhas,
   multiclasse e aumento de atributos por fontes distintas permanecem pendentes.
8. **Criação:** assistente guiado continua desabilitado; cadastro avançado não
   equivale a ficha validada. Faltam métodos de atributos por rolagem, idade,
   opções raciais e suplementos completos.
9. **Legados:** migração preserva dados sem validá-los como ficha oficial;
   revisão deve ser explícita, sem apagar o original ou perder campos desconhecidos.
10. **Cálculos duplicados:** interface e Rust ainda divergem em cobertura;
    centralizar gradualmente e manter testes de regressão entre os caminhos.

## Continuação do plano, por ordem de dependência

1. Inventariar edição/revisão dos PDFs e cada regra relevante: fonte, página,
   escolhas, exemplos esperados e status de conferência.
2. Consolidar cálculos e validação no núcleo existente, separando escolhas,
   valores derivados e estado temporário. Usar IDs estáveis e versão de regras.
3. Criação guiada e evolução nível a nível com rascunho, voltar sem perda,
   pré-requisitos e explicação dos totais; manter editor avançado sem selo falso.
4. Equipamento, magias, efeitos, recursos e ações de sessão verificadas.
5. Suplementos locais, pacote por pacote, incluindo interações e exceções.
6. Migração versionada com cópia de segurança e homologação das fichas.

Critério final: inventário integral coberto, nenhuma divergência mecânica
pendente, exemplos independentes, testes de cada regra e casos de limite,
comparações com Nimb resolvidas nos livros, persistência sem perda e fluxo
completo acessível. Concordância com o Nimb isoladamente não prova correção.

## Verificações deste lote

- npm test: 12 testes aprovados.
- npm run check: zero erros e avisos.
- cargo test -p tormenta20-core: 4 testes aprovados.
- cargo test --manifest-path src-tauri/Cargo.toml --lib: 16 testes aprovados,
  incluindo protocolo da mesa e regressões de permissões/migração/dano.
- Build de produção aprovado; aviso de tamanho de bundle existente permanece.
- Detector Impeccable: nenhum apontamento nos quatro componentes modificados.
- Revisão visual pendente: navegador integrado não conseguiu acessar o servidor
  local (timeout). O servidor temporário de teste foi encerrado; testes de
  lógica/interface compilada não substituem a conferência visual.

## Correções de dados conferidos no Livro Básico

- Proficiências adicionais: Clérigo recebe armaduras pesadas/escudos, não armas
  marciais (p. 57); Ladino não recebe proficiências adicionais (p. 73).
- Armadura Arcana básica fornece +5 Defesa (p. 181), corrigido de +4 na interface.
- Catálogo parcial: corrigidos Acuidade, Ataque Poderoso, Esquiva, Foco em Arma,
  Saque Rápido, Vitalidade, Vontade de Ferro, Sortudo, Ataque Especial e Fúria,
  incluindo referências de página. Teste verifica dados importados no compêndio.
- Removida do catálogo parcial a entrada incorreta de Aumento de Atributo como
  poder geral de Destino de 1º nível. Não se apagaram poderes de fichas existentes.
  Reintroduzir com aquisição por classe e restrição por patamar devidamente modeladas.
- Corrigir estas entradas não certifica o restante da tabela, os suplementos ou
  a aplicação automática de todos os poderes. A revisão permanece parcial.

O pedido de desligamento está condicionado à conclusão das correções. Como há
lacunas mecânicas abertas acima, nenhum desligamento foi executado/agendado.
