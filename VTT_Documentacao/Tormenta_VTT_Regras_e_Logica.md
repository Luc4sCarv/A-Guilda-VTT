# visão geral do projeto

## Ideia geral

Um aplicativo desktop livre para criar, manter e consultar fichas de personagens do RPG **Tormenta20**. O objetivo atual não é substituir o jogador nem automatizar todas as regras narrativas: é reduzir trabalho repetitivo, manter os valores da ficha coerentes e disponibilizar conteúdo estruturado com referência ao livro de origem.


## Linguagens e tecnologias definidas

O alvo inicial é Windows 10/11, distribuído como um único `.exe`. O aplicativo usa **Rust** no núcleo de regras, servidor e integração desktop via Tauri. A interface usa **Svelte + TypeScript**, compilada como arquivos estáticos carregados pelo webview do sistema. Não há runtime externo para o usuário instalar.

## Dados e persistência

Cada ficha possui um identificador único e guarda, entre outros dados, identidade do personagem, ancestralidade, origem, classes e níveis, atributos, recursos, ataques, perícias, inventário, poderes, magias e anotações. As fichas são salvas no diretório de dados local do aplicativo.

Ao salvar, o núcleo valida os campos e grava o registro usando o identificador da ficha: se ele já existir, os dados são atualizados; se não existir, uma nova ficha é criada. A listagem é ordenada pela data de atualização.

Há importação e exportação em JSON. Na importação, a ficha recebe novo identificador e nova data de atualização, evitando sobrescrever uma ficha existente. O programa também reconhece formatos antigos de inventário e poderes compostos somente por texto, convertendo-os para registros personalizados.

Uma migração única preserva o rascunho produzido pela versão inicial. Depois de concluída, ela é marcada no banco para que uma ficha antiga removida não volte a ser criada.


## Regras automatizadas

### Atributos

Os seis atributos básicos são Força, Destreza, Constituição, Inteligência, Sabedoria e Carisma. Cada um pode ir de –1 a 4. A compra usa orçamento máximo de 10 pontos, com os custos abaixo:

| Valor | Custo |
| --- | ---: |
| –1 | –1 |
| 0 | 0 |
| 1 | 1 |
| 2 | 2 |
| 3 | 4 |
| 4 | 7 |

Os modificadores de ancestralidade são aplicados separadamente ao atributo base. Algumas ancestralidades exigem escolhas de atributos; nessas situações, a quantidade definida pela regra deve ser escolhida e atributos excluídos não podem ser usados.

### Ancestralidade, origem e classe

O conjunto atual contém 17 ancestralidades, 35 origens e 14 classes do livro básico. A ancestralidade pode alterar atributos, PV inicial, PV por nível e deslocamento. A origem exige a seleção de exatamente dois benefícios; benefícios que também são perícias passam a ser treinados automaticamente.

Cada classe define PV inicial, PV por nível, PM por nível, perícias obrigatórias, possíveis escolhas condicionais, quantidade de perícias livres, proficiências e, quando aplicável, progressão de magia. Uma classe não pode ser repetida na multiclasse. Os níveis adicionais devem somar menos que o nível total, preservando ao menos um nível na classe inicial.

Arcanista exige caminho; se o caminho for Feiticeiro, exige linhagem. Algumas classes exigem a escolha de uma perícia entre alternativas específicas.

### PV, PM, Defesa e deslocamento

Para a classe inicial, os PV máximos são calculados a partir de PV inicial, Constituição e níveis na classe. Classes adicionais acrescentam seus PV por nível, também considerando Constituição. Bônus de ancestralidade são somados. O feiticeiro de linhagem dracônica recebe adicional de Carisma nos PV. Ajustes manuais de PV são mantidos à parte do cálculo automático.

Os PM máximos somam a progressão da classe inicial e das classes adicionais. Arcanista adiciona Inteligência ou Carisma, conforme o caminho; Bardo e Paladino adicionam Carisma; Clérigo e Druida adicionam Sabedoria. Ajustes manuais de PM também são preservados separadamente.

A Defesa é calculada como `10 + Destreza + armadura + escudo + raça + outros`. Armadura pesada remove a Destreza do cálculo. O deslocamento parte do valor da ancestralidade, ou 9 metros quando ela não o modifica; armadura pesada reduz 3 metros. Valores nunca ficam abaixo de zero.

Quando o máximo de PV ou PM muda, o valor atual é ajustado para não ultrapassar o novo máximo. Se o personagem estava com o recurso completo, ele permanece completo após o recálculo.

### Perícias

Existem 29 perícias. Perícias obrigatórias da classe, escolhas obrigatórias e benefícios de origem compatíveis são automáticas. As demais são limitadas pela classe: número de escolhas livres da classe mais Inteligência final, nunca reduzindo o limite por Inteligência negativa.

O bônus de uma perícia é o atributo final mais o bônus de treinamento, quando treinada, ou metade do nível, quando não treinada. A progressão de treinamento por nível é: 2, 3, 3, 4, 4, 5, 7, 8, 8, 9, 9, 10, 10, 11, 13, 14, 14, 15, 15 e 16. Penalidades de armaduras e escudos equipados se aplicam a Acrobacia, Furtividade e Ladinagem.

### Magias, poderes, ataques e inventário

Os círculos disponíveis dependem da classe e do nível. Classes de progressão completa alcançam até o 5º círculo, avançando a cada quatro níveis a partir do 1º; classes de progressão limitada chegam ao 2º círculo no nível 6, 3º no 10 e 4º no 14. Magias só são aceitas quando tradição, classe, nível e círculo são compatíveis.

Poderes de classe exigem a classe correspondente, e requisitos de nível explícitos são verificados. Armas preenchem ataque e inventário com dano, crítico, alcance e tipo de dano. Armaduras e escudos equipados atualizam Defesa, penalidade e condição de armadura pesada. Equipamentos registram preço, quantidade e espaços; a carga mostrada é a soma de `espaços × quantidade` de todos os itens.

## Distribuição e modelo de uso

O app é um único executável — não existe versão "cliente" e versão "mestre" separadas. Todo mundo baixa o mesmo `.exe`.

Cada pessoa usa o app localmente para criar e manter suas próprias fichas. Enquanto não há mesa ativa, **nenhum servidor é iniciado**: a interface chama o `core` diretamente pelos comandos Tauri e acessa o banco SQLite local. Compêndio, criação de ficha e regras funcionam 100% offline.

Uma ficha chega ao mestre de duas formas, e as duas usam o mesmo caminho por baixo:

- **Fora da mesa**: o jogador exporta a ficha em JSON (recurso que já existe no núcleo) e envia por qualquer meio externo — WhatsApp, Discord, etc. O mestre importa o arquivo no seu próprio app. Como a importação já gera novo identificador e nova data de atualização, isso nunca sobrescreve uma ficha existente por engano.
- **Dentro da mesa**: quando o jogador entra numa mesa hospedada pelo mestre (usando o código de convite), o cliente dele se conecta no servidor embutido do mestre. Nesse momento, enviar a ficha é a mesma exportação/importação de sempre, só que transportada pelo WebSocket já aberto em vez de um arquivo solto — o jogador manda a ficha, o servidor do mestre recebe e importa automaticamente, associando a ficha àquele usuário na mesa.

Ou seja: exportar/importar via arquivo é o caminho universal (funciona sempre, com ou sem mesa); o envio pela mesa é só esse mesmo mecanismo automatizado quando já existe uma conexão ativa. Isso evita manter duas lógicas diferentes de "receber uma ficha" — uma pra arquivo e outra pra rede.

## Limites e validação

Antes de gravar, o núcleo rejeita fichas sem nome ou identificador, atributos fora da faixa, orçamento de atributos acima de 10, nível fora de 1 a 20, moedas negativas ou não numéricas, classes duplicadas e multiclasses inválidas. Também há limites de tamanho para textos e de quantidade para coleções, como ataques, perícias, itens, poderes e magias. Magias devem estar entre o 1º e o 5º círculo.

Essas verificações no núcleo protegem os dados mesmo quando uma ficha importada não tiver sido produzida pelo próprio programa.

## Conteúdo e licenciamento

O software pode modelar as regras necessárias aos cálculos, mas textos, imagens e dados editoriais do compêndio só entram na distribuição quando houver licença ou permissão compatível. Até isso ser confirmado, o app aceita compêndios importados pelo usuário e inclui apenas conteúdo próprio ou autorizado. Cada registro guarda `fonte`, `edição` e, quando aplicável, `página`.
