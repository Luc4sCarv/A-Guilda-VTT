# Tormenta20: Jogo do Ano — Catálogo Estrutural das Regras do Núcleo

Este documento é uma base estrutural para o `core` de **Tormenta20 (Edição Jogo do Ano - JdA)**. As tabelas de poderes, magias e suplementos são amostras parciais e não devem ser tratadas como compêndio exaustivo. Linhas confirmadas no Livro Básico usam a paginação impressa; conteúdo de suplementos ainda exige validação na respectiva fonte.

---

## 1. Classes e Caminhos (`classes.md`)

| classe | PV inicial | PV/nível | PM/nível | perícias obrigatórias | escolhas de perícia | perícias livres | proficiências | atributo de mana | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Arcanista (Bruxo)** | 8 + Con | 2 + Con | 6 | Misticismo, Vontade | 2 da lista da classe | Int x Perícias | Nenhuma | Inteligência | Livro Básico | Jogo do Ano | p. 36 | N/A |
| **Arcanista (Feiticeiro)** | 8 + Con | 2 + Con | 6 | Misticismo, Vontade | 2 da lista da classe | Int x Perícias | Nenhuma | Carisma | Livro Básico | Jogo do Ano | p. 36 | N/A |
| **Arcanista (Mago)** | 8 + Con | 2 + Con | 6 | Misticismo, Vontade | 2 da lista da classe | Int x Perícias | Nenhuma | Inteligência | Livro Básico | Jogo do Ano | p. 36 | N/A |
| **Arcanista (Necromante)** | 8 + Con | 2 + Con | 6 | Misticismo, Vontade | 2 da lista da classe | Int x Perícias | Nenhuma | Inteligência | Heróis de Arton | 1ª Edição | p. 54 | N/A |
| **Bárbaro** | 24 + Con | 6 + Con | 3 | Fortitude, Luta | 4 da lista da classe | Int x Perícias | Armas marciais, escudos | N/A | Livro Básico | Jogo do Ano | p. 40 | N/A |
| **Bardo** | 12 + Con | 3 + Con | 4 | Atuação, Reflexos | 6 da lista da classe | Int x Perícias | Armas marciais | Carisma | Livro Básico | Jogo do Ano | p. 43 | N/A |
| **Bucaneiro** | 16 + Con | 4 + Con | 3 | Luta ou Pontaria, Reflexos | 4 da lista da classe | Int x Perícias | Armas marciais | N/A | Livro Básico | Jogo do Ano | p. 46 | N/A |
| **Caçador** | 16 + Con | 4 + Con | 4 | Sobrevivência, Luta ou Pontaria | 6 da lista da classe | Int x Perícias | Armas marciais, escudos | N/A | Livro Básico | Jogo do Ano | p. 50 | N/A |
| **Cavaleiro** | 20 + Con | 5 + Con | 3 | Fortitude, Luta | 2 da lista da classe | Int x Perícias | Armas marciais, armaduras pesadas, escudos | N/A | Livro Básico | Jogo do Ano | p. 52 | N/A |
| **Clérigo** | 16 + Con | 4 + Con | 5 | Religiao, Vontade | 2 da lista da classe | Int x Perícias | Armas marciais, armaduras pesadas, escudos | Sabedoria | Livro Básico | Jogo do Ano | p. 56 | N/A |
| **Druida** | 16 + Con | 4 + Con | 4 | Sobrevivência, Vontade | 4 da lista da classe | Int x Perícias | Escudos | Sabedoria | Livro Básico | Jogo do Ano | p. 60 | N/A |
| **Guerreiro** | 20 + Con | 5 + Con | 3 | Fortitude ou Reflexos, Luta ou Pontaria | 2 da lista da classe | Int x Perícias | Armas marciais, armaduras pesadas, escudos | N/A | Livro Básico | Jogo do Ano | p. 64 | N/A |
| **Inventor** | 12 + Con | 3 + Con | 4 | Ofício, Vontade | 4 da lista da classe | Int x Perícias | Armas de fogo | N/A | Livro Básico | Jogo do Ano | p. 68 | N/A |
| **Ladino** | 12 + Con | 3 + Con | 4 | Ladinagem, Furtividade | 8 da lista da classe | Int x Perícias | Armas marciais discretas | N/A | Livro Básico | Jogo do Ano | p. 72 | N/A |
| **Lutador** | 20 + Con | 5 + Con | 3 | Fortitude, Luta | 4 da lista da classe | Int x Perícias | Nenhuma | N/A | Livro Básico | Jogo do Ano | p. 75 | N/A |
| **Nobre** | 16 + Con | 4 + Con | 4 | Diplomacia ou Intimidação, Vontade | 4 da lista da classe | Int x Perícias | Armas marciais, armaduras pesadas, escudos | N/A | Livro Básico | Jogo do Ano | p. 78 | N/A |
| **Paladino** | 20 + Con | 5 + Con | 3 | Luta, Vontade | 2 da lista da classe | Int x Perícias | Armas marciais, armaduras pesadas, escudos | Carisma | Livro Básico | Jogo do Ano | p. 81 | N/A |

### Estrutura de Caminhos e Subclasses Obrigatórias:
- **Arcanista → Caminho**:
  - *Bruxo*: Lança magias através de um Foco Arcano. Atributo: Inteligência.
  - *Feiticeiro → Linhagem*: Lança magias por herança sobrenatural. Atributo: Carisma. Linhagens: **Dracônica** (PV e Defesa), **Feérica** (Enganação e encantos), **Rubra** (Poderes da Tormenta).
  - *Mago*: Aprende magias por estudo e as registra em um Grimório. Atributo: Inteligência.
  - *Necromante*: Lança magias arcanas e divinas de necromancia. Paga PVs para alimentar magias de morte. Atributo: Inteligência.
- **Bardo → Escolas de Magia**: Escolhe 3 escolas de magia arcanas ao criar o personagem.
- **Clérigo → Divindade e Canalizar Energia**: Obrigatório escolher uma Divindade Padroeira e escolher Canalizar Energia Positiva (Cura/Dano em mortos-vivos) ou Negativa (Dano/Cura em mortos-vivos).
- **Druida → Forma Selvagem / Caminho dos Deuses**: Escolhe 3 escolas de magia divinas e pode assumir Formas Selvagens (Ágil, Resistente, Veloz, Feroz, Voadora, Gigante).
- **Paladino → Código do Herói & Divindade**: Segue o Código de Conduta (Honra, Ajuda, Justiça) e canaliza o Golpe Divino através de Carisma.

---

## 2. Ancestralidades (`ancestralidades.md`)

| ancestralidade | modificadores fixos | quantidade de escolhas | atributos proibidos | tamanho | deslocamento | PV inicial | PV/nível | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Humano** | +1 em 3 atributos diferentes | 3 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 18 | N/A |
| **Anão** | Con +2, Sab +1, Des -1 | 0 | Nenhum | Médio | 6m | +3 PV | +1 PV | Livro Básico | Jogo do Ano | p. 19 | N/A |
| **Dahllan** | Sab +2, Des +1, Int -1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 20 | N/A |
| **Elfo** | Int +2, Des +1, Con -1 | 0 | Nenhum | Médio | 12m | Base | Base | Livro Básico | Jogo do Ano | p. 21 | N/A |
| **Goblin** | Des +2, Int +1, Car -1 | 0 | Nenhum | Pequeno | 9m, escalada 9m | Base | Base | Livro Básico | Jogo do Ano | p. 22 | N/A |
| **Lefou** | +1 em 3 atributos (exceto Car) | 3 | Carisma | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 23 | N/A |
| **Minotauro** | For +2, Con +1, Sab -1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 24 | N/A |
| **Qareen** | Car +2, Int +1, Sab -1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 25 | N/A |
| **Golem** | For +2, Con +1, Car -1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 26 | N/A |
| **Hynne** | Des +2, Car +1, For -1 | 0 | Nenhum | Pequeno | 6m | Base | Base | Livro Básico | Jogo do Ano | p. 27 | N/A |
| **Kliren** | Int +2, Car +1, For -1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 28 | N/A |
| **Medusa** | Des +2, Car +1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 29 | N/A |
| **Osteon** | +1 em 3 atributos (exceto Con) | 3 | Constituição (-1) | Conforme Raça | Conforme Raça | Base | Base | Livro Básico | Jogo do Ano | p. 29 | N/A |
| **Aggelus (Suraggel)** | Sab +2, Car +1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 30 | N/A |
| **Sulfure (Suraggel)** | Des +2, Int +1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 30 | N/A |
| **Sereia/Tritão** | +1 em 3 atributos | 3 | Nenhum | Médio | 9m, natação 12m | Base | Base | Livro Básico | Jogo do Ano | p. 30 | N/A |
| **Silfide** | Car +2, Des +1, For -2 | 0 | Nenhum | Minúsculo | 9m (pairar), voo 12m | Base | Base | Livro Básico | Jogo do Ano | p. 30 | N/A |
| **Trog** | Con +2, For +1, Int -1 | 0 | Nenhum | Médio | 9m | Base | Base | Livro Básico | Jogo do Ano | p. 31 | N/A |
| **Galokk** | For +1, Con +1, +1 livre | 1 | Carisma (-1) | Grande | 9m | +4 PV | +1 PV | Heróis de Arton | 1ª Edição | p. 135 | N/A |
| **Duende** | Car +2, Des +1, For -1 | 0 | Nenhum | Pequeno | 9m | Base | Base | Heróis de Arton | 1ª Edição | p. 8 | N/A |
| **Eiradaan** | Sab +2, Int +1, Con -1 | 0 | Nenhum | Médio | 9m | Base | Base | Heróis de Arton | 1ª Edição | p. 12 | N/A |
| **Moreau (Lobo)** | For +1, Des +1, Sab +1 | 0 | Nenhum | Médio | 9m | Base | Base | Heróis de Arton | 1ª Edição | p. 18 | N/A |
| **Moreau (Coruja)** | Int +2, Sab +1, For -1 | 0 | Nenhum | Médio | 9m | Base | Base | Heróis de Arton | 1ª Edição | p. 19 | N/A |
| **Moreau (Leão)** | For +2, Car +1, Sab -1 | 0 | Nenhum | Médio | 9m | Base | Base | Heróis de Arton | 1ª Edição | p. 20 | N/A |
| **Moreau (Urso)** | Con +2, For +1, Des -1 | 0 | Nenhum | Médio | 9m | Base | Base | Heróis de Arton | 1ª Edição | p. 21 | N/A |
| **Finntroll** | Int +2, Con +1, For -1 | 0 | Nenhum | Médio | 9m | Base | Base | Heróis de Arton | 1ª Edição | p. 25 | N/A |
| **Mashin** | Con +2, Int +1, Car -1 | 0 | Nenhum | Médio | 9m | Base | Base | Heróis de Arton | 1ª Edição | p. 28 | N/A |
| **Kaijin** | For +2, Con +1, Car -2 | 0 | Nenhum | Médio | 9m | +2 PV | Base | Heróis de Arton | 1ª Edição | p. 32 | N/A |

---

## 3. Perícias (`pericias.md`)

| perícia | atributo-chave | exige treinamento | sofre penalidade de armadura | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Acrobacia** | Destreza | Não | Sim | Livro Básico | Jogo do Ano | p. 115 | N/A |
| **Adestramento** | Carisma | Sim | Não | Livro Básico | Jogo do Ano | p. 115 | N/A |
| **Atletismo** | Força | Não | Não* | Livro Básico | Jogo do Ano | p. 116 | N/A |
| **Atuação** | Carisma | Sim | Não | Livro Básico | Jogo do Ano | p. 116 | N/A |
| **Cavalgar** | Destreza | Não | Não | Livro Básico | Jogo do Ano | p. 117 | N/A |
| **Conhecimento** | Inteligência | Sim | Não | Livro Básico | Jogo do Ano | p. 117 | N/A |
| **Cura** | Sabedoria | Não | Não | Livro Básico | Jogo do Ano | p. 117 | N/A |
| **Diplomacia** | Carisma | Não | Não | Livro Básico | Jogo do Ano | p. 118 | N/A |
| **Enganação** | Carisma | Não | Não | Livro Básico | Jogo do Ano | p. 118 | N/A |
| **Fortitude** | Constituição | Não | Não | Livro Básico | Jogo do Ano | p. 119 | N/A |
| **Furtividade** | Destreza | Não | Sim | Livro Básico | Jogo do Ano | p. 119 | N/A |
| **Guerra** | Inteligência | Sim | Não | Livro Básico | Jogo do Ano | p. 120 | N/A |
| **Iniciativa** | Destreza | Não | Não | Livro Básico | Jogo do Ano | p. 120 | N/A |
| **Intimidação** | Carisma | Não | Não | Livro Básico | Jogo do Ano | p. 120 | N/A |
| **Intuição** | Sabedoria | Não | Não | Livro Básico | Jogo do Ano | p. 121 | N/A |
| **Investigação** | Inteligência | Não | Não | Livro Básico | Jogo do Ano | p. 121 | N/A |
| **Jogatina** | Carisma | Sim | Não | Livro Básico | Jogo do Ano | p. 121 | N/A |
| **Ladinagem** | Destreza | Sim | Sim | Livro Básico | Jogo do Ano | p. 121 | N/A |
| **Luta** | Força | Não | Não | Livro Básico | Jogo do Ano | p. 122 | N/A |
| **Misticismo** | Inteligência | Sim | Não | Livro Básico | Jogo do Ano | p. 122 | N/A |
| **Nobreza** | Inteligência | Sim | Não | Livro Básico | Jogo do Ano | p. 122 | N/A |
| **Ofício** | Inteligência | Sim | Não | Livro Básico | Jogo do Ano | p. 123 | N/A |
| **Percepção** | Sabedoria | Não | Não | Livro Básico | Jogo do Ano | p. 123 | N/A |
| **Pilotagem** | Destreza | Sim | Não | Livro Básico | Jogo do Ano | p. 124 | N/A |
| **Pontaria** | Destreza | Não | Não | Livro Básico | Jogo do Ano | p. 124 | N/A |
| **Reflexos** | Destreza | Não | Não | Livro Básico | Jogo do Ano | p. 124 | N/A |
| **Religião** | Sabedoria | Sim | Não | Livro Básico | Jogo do Ano | p. 124 | N/A |
| **Sobrevivência** | Sabedoria | Não | Não | Livro Básico | Jogo do Ano | p. 125 | N/A |
| **Vontade** | Sabedoria | Não | Não | Livro Básico | Jogo do Ano | p. 125 | N/A |

*\*Nota: Penalidade de armadura aplica-se em Atletismo especificamente para testes de Natação e Escalada.*

---

## 4. Progressão Mágica (`progressao_magica.md`)

| classe/caminho | nível | círculo máximo | tradição | quantidade de magias | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Arcanista** | 1º ao 4º | 1º Círculo | Arcana | 3 iniciais (+1 por nível adicional) | Livro Básico | Jogo do Ano | p. 36 | N/A |
| **Arcanista** | 5º ao 8º | 2º Círculo | Arcana | +1 magia por nível | Livro Básico | Jogo do Ano | p. 36 | N/A |
| **Arcanista** | 9º ao 12º | 3º Círculo | Arcana | +1 magia por nível | Livro Básico | Jogo do Ano | p. 36 | N/A |
| **Arcanista** | 13º ao 16º | 4º Círculo | Arcana | +1 magia por nível | Livro Básico | Jogo do Ano | p. 36 | N/A |
| **Arcanista** | 17º ao 20º | 5º Círculo | Arcana | +1 magia por nível | Livro Básico | Jogo do Ano | p. 36 | N/A |
| **Clérigo** | 1º ao 4º | 1º Círculo | Divina | 3 iniciais (+1 por nível adicional) | Livro Básico | Jogo do Ano | p. 56 | N/A |
| **Clérigo** | 5º ao 8º | 2º Círculo | Divina | +1 magia por nível | Livro Básico | Jogo do Ano | p. 56 | N/A |
| **Clérigo** | 9º ao 12º | 3º Círculo | Divina | +1 magia por nível | Livro Básico | Jogo do Ano | p. 56 | N/A |
| **Clérigo** | 13º ao 16º | 4º Círculo | Divina | +1 magia por nível | Livro Básico | Jogo do Ano | p. 56 | N/A |
| **Clérigo** | 17º ao 20º | 5º Círculo | Divina | +1 magia por nível | Livro Básico | Jogo do Ano | p. 56 | N/A |
| **Druida** | 1º ao 5º | 1º Círculo | Divina | 3 iniciais (3 escolas) (+1 por nível) | Livro Básico | Jogo do Ano | p. 60 | N/A |
| **Druida** | 6º ao 9º | 2º Círculo | Divina | +1 magia por nível | Livro Básico | Jogo do Ano | p. 60 | N/A |
| **Druida** | 10º ao 13º | 3º Círculo | Divina | +1 magia por nível | Livro Básico | Jogo do Ano | p. 60 | N/A |
| **Druida** | 14º ao 20º | 4º Círculo | Divina | +1 magia por nível | Livro Básico | Jogo do Ano | p. 60 | N/A |
| **Bardo** | 1º ao 5º | 1º Círculo | Arcana | 2 iniciais (3 escolas) (+1 por nível) | Livro Básico | Jogo do Ano | p. 43 | N/A |
| **Bardo** | 6º ao 9º | 2º Círculo | Arcana | +1 magia por nível | Livro Básico | Jogo do Ano | p. 43 | N/A |
| **Bardo** | 10º ao 13º | 3º Círculo | Arcana | +1 magia por nível | Livro Básico | Jogo do Ano | p. 43 | N/A |
| **Bardo** | 14º ao 20º | 4º Círculo | Arcana | +1 magia por nível | Livro Básico | Jogo do Ano | p. 43 | N/A |
| **Paladino** | 6º ao 9º | 1º Círculo | Divina | 2 magias divinas | Livro Básico | Jogo do Ano | p. 81 | N/A |
| **Paladino** | 10º ao 20º | 2º Círculo | Divina | +1 magia a cada 2 níveis | Livro Básico | Jogo do Ano | p. 81 | N/A |

---

## 5. Origens — rascunho parcial (`origens.md`)

Rascunho ainda não validado linha a linha. Cada origem concede 2 benefícios à escolha (perícias ou poderes). Nomes de suplementos não podem ser atribuídos ao Livro Básico sem confirmação.

| origem | quantidade de benefícios | perícias disponíveis | poderes disponíveis | outros benefícios | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Acólito** | 2 escolhas | Cura, Religião | Membro da Igreja, Vontade de Ferro | Símbolo sagrado | Livro Básico | Jogo do Ano | p. 86 | N/A |
| **Amigo dos Animais** | 2 escolhas | Adestramento, Percepção | Amigo dos Animais, Cão de Guarda | Cão de guarda ou cavalo | Livro Básico | Jogo do Ano | p. 86 | N/A |
| **Amnésico** | 2 escolhas | 2 perícias quaisquer | Lembranças Graduais, 1 poder geral | Nível inicial de incerteza | Livro Básico | Jogo do Ano | p. 86 | N/A |
| **Aristocrata** | 2 escolhas | Enganação, Nobreza | Comandar, Sangue Azul | Joia no valor de T$ 90 | Livro Básico | Jogo do Ano | p. 86 | N/A |
| **Artesão** | 2 escolhas | Ofício, Vontade | Frutos do Trabalho, Sortudo | Ferramentas de ofício | Livro Básico | Jogo do Ano | p. 87 | N/A |
| **Artista** | 2 escolhas | Atuação, Enganação | Dom Artístico, Torcida | Instrumento musical | Livro Básico | Jogo do Ano | p. 87 | N/A |
| **Assistente de Laboratório** | 2 escolhas | Ofício (alquimia), Misticismo | Alquimista Iniciado, Esse Cheiro... | Kit de alquimia | Livro Básico | Jogo do Ano | p. 87 | N/A |
| **Caçador de Recompensas** | 2 escolhas | Investigação, Sobrevivência | Olho Clínico, Rastreador | Algemas e rede | Livro Básico | Jogo do Ano | p. 88 | N/A |
| **Capanga** | 2 escolhas | Intimidação, Luta | Confiança de Bruto, Duro de Matar | Arma simples | Livro Básico | Jogo do Ano | p. 89 | N/A |
| **Charlatão** | 2 escolhas | Enganação, Jogatina | Alquimista Iniciado, Efeito Cênico | Kit de enganação | Livro Básico | Jogo do Ano | p. 89 | N/A |
| **Circense** | 2 escolhas | Acrobacia, Atuação | Truque de Mágica, Acrobata | Vestuário vistoso | Livro Básico | Jogo do Ano | p. 89 | N/A |
| **Criminoso** | 2 escolhas | Enganação, Ladinagem | Pobreza Esclarecida, Punguista | Gazua e adaga | Livro Básico | Jogo do Ano | p. 90 | N/A |
| **Discípulo de Arcanista** | 2 escolhas | Conhecimento, Misticismo | Prodígio Arcano, Mente Analítica | Grimório ou Foco | Livro Básico | Jogo do Ano | p. 90 | N/A |
| **Escravo** | 2 escolhas | Atletismo, Fortitude | Desesperado, Estoico | Correntes quebradas | Livro Básico | Jogo do Ano | p. 90 | N/A |
| **Estudante da Academia Arcana**| 2 escolhas | Conhecimento, Misticismo | Segredo Arcano, Mente Analítica | Livro de estudos | Livro Básico | Jogo do Ano | p. 90 | N/A |
| **Gladiador** | 2 escolhas | Atletismo, Luta | Pão e Circo, Sangue nos Olhos | Arma marcial | Livro Básico | Jogo do Ano | p. 91 | N/A |
| **Guarda** | 2 escolhas | Investigação, Percepção | Detetive, Investigador | Apito e lanterna | Livro Básico | Jogo do Ano | p. 91 | N/A |
| **Herdeiro** | 2 escolhas | Nobreza, Ofício | Herança, Riqueza | Item de família (T$ 1.000) | Livro Básico | Jogo do Ano | p. 91 | N/A |
| **Herói Camponês** | 2 escolhas | Adestramento, Sobrevivência | Coração Heróico, Torcida | Ferramenta agrícola | Livro Básico | Jogo do Ano | p. 92 | N/A |
| **Marujo** | 2 escolhas | Atletismo, Pilotagem | Acrobata, Passagem de Navio | Corda e faca | Livro Básico | Jogo do Ano | p. 92 | N/A |
| **Matuto** | 2 escolhas | Adestramento, Sobrevivência | Viver do Terreno, Comum | Chapéu de palha | Livro Básico | Jogo do Ano | p. 92 | N/A |
| **Mercador** | 2 escolhas | Diplomacia, Ofício | Negociante, Barganhar | Carroça e mulas | Livro Básico | Jogo do Ano | p. 92 | N/A |
| **Minerador** | 2 escolhas | Atletismo, Ofício | Escavação, Duro de Matar | Picareta e lanterna | Livro Básico | Jogo do Ano | p. 92 | N/A |
| **Nômade** | 2 escolhas | Cavalgar, Sobrevivência | Mochileiro, Lobo Solitário | Montaria | Livro Básico | Jogo do Ano | p. 93 | N/A |
| **Pivete** | 2 escolhas | Acrobacia, Ladinagem | Chamar Atenção, Quebra-Nozes | Bumerangue | Livro Básico | Jogo do Ano | p. 93 | N/A |
| **Refugiado** | 2 escolhas | Fortitude, Sobrevivência | Determinação, Estoico | Mochila de viagem | Livro Básico | Jogo do Ano | p. 93 | N/A |
| **Religioso** | 2 escolhas | Religião, Vontade | Autoridade Feudal, Membro da Igreja | Livro sagrado | Livro Básico | Jogo do Ano | p. 93 | N/A |
| **Selvagem** | 2 escolhas | Atletismo, Sobrevivência | Vida Selvagem, Duro de Matar | Bordão de osso | Livro Básico | Jogo do Ano | p. 93 | N/A |
| **Soldado** | 2 escolhas | Guerra, Luta | Tática de Combate, Ataque Reflexo | Uniforme e arma marcial | Livro Básico | Jogo do Ano | p. 93 | N/A |
| **Taverneiro** | 2 escolhas | Diplomacia, Jogatina | Bêbado de Guerra, Bons Frutos | Caneca de metal | Livro Básico | Jogo do Ano | p. 94 | N/A |
| **Trabalhador** | 2 escolhas | Atletismo, Fortitude | Esforçado, Pobreza Esclarecida | Ferramenta de trabalho | Livro Básico | Jogo do Ano | p. 94 | N/A |
| **Vagabundo** | 2 escolhas | Enganação, Furtividade | Sorriso Sarcástico, Sortudo | Baralho comum | Livro Básico | Jogo do Ano | p. 94 | N/A |

---

## 6. Equipamentos, Armas e Armaduras (`equipamentos.md`)

Revisão: armas da Tabela 3-3 e armaduras/escudos da Tabela 3-5 do Livro Básico (paginação impressa). Esta seção ainda **não** inclui equipamentos gerais, consumíveis nem itens mágicos; não é um catálogo completo de itens. Munições estão listadas separadamente para não serem confundidas com armas.

### Armas

| nome | categoria | empunhadura | dano | crítico | alcance | tipo de dano | preço | espaços | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Adaga** | Simples | Leve | 1d4 | 19/x2 | Curto | Perfuração | T$ 2 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Espada Curta** | Simples | Leve | 1d6 | 19/x2 | — | Perfuração | T$ 10 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Foice** | Simples | Leve | 1d6 | x3 | — | Corte | T$ 4 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Clava** | Simples | Uma Mão | 1d6 | x2 | — | Impacto | — | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Lança** | Simples | Uma Mão | 1d6 | x2 | Curto | Perfuração | T$ 2 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Maça** | Simples | Uma Mão | 1d8 | x2 | — | Impacto | T$ 12 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Bordão** | Simples | Duas Mãos | 1d6/1d6 | x2 | — | Impacto | — | 2 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Pique** | Simples | Duas Mãos | 1d8 | x2 | — | Perfuração | T$ 2 | 2 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Tacape** | Simples | Duas Mãos | 1d10 | x2 | — | Impacto | — | 2 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Azagaia** | Simples | Uma Mão | 1d6 | x2 | Médio | Perfuração | T$ 1 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Besta Leve** | Simples | Uma Mão | 1d8 | 19/x2 | Médio | Perfuração | T$ 35 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Funda** | Simples | Uma Mão | 1d4 | x2 | Médio | Impacto | — | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Arco Curto** | Simples | Duas Mãos | 1d6 | x3 | Médio | Perfuração | T$ 30 | 2 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Machadinha** | Marcial | Leve | 1d6 | x3 | Curto | Corte | T$ 6 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Cimitarra** | Marcial | Uma Mão | 1d6 | 18/x2 | — | Corte | T$ 15 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Espada Longa** | Marcial | Uma Mão | 1d8 | 19/x2 | — | Corte | T$ 15 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Florete** | Marcial | Uma Mão | 1d6 | 18/x2 | — | Perfuração | T$ 20 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Machado de Batalha** | Marcial | Uma Mão | 1d8 | x3 | — | Corte | T$ 10 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Mangual** | Marcial | Uma Mão | 1d8 | x2 | — | Impacto | T$ 8 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Martelo de Guerra** | Marcial | Uma Mão | 1d8 | x3 | — | Impacto | T$ 12 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Picareta** | Marcial | Uma Mão | 1d6 | x4 | — | Perfuração | T$ 8 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Tridente** | Marcial | Uma Mão | 1d8 | x2 | Curto | Perfuração | T$ 15 | 1 | Livro Básico | Jogo do Ano | p. 144 | N/A |
| **Alabarda** | Marcial | Duas Mãos | 1d10 | x3 | — | Corte/Perfuração | T$ 10 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Alfange** | Marcial | Duas Mãos | 2d4 | 18/x2 | — | Corte | T$ 75 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Gadanho** | Marcial | Duas Mãos | 2d4 | x4 | — | Corte | T$ 18 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Lança Montada** | Marcial | Duas Mãos | 1d8 | x3 | — | Perfuração | T$ 10 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Machado de Guerra** | Marcial | Duas Mãos | 1d12 | x3 | — | Corte | T$ 20 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Marreta** | Marcial | Duas Mãos | 3d4 | x2 | — | Impacto | T$ 20 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Montante** | Marcial | Duas Mãos | 2d6 | 19/x2 | — | Corte | T$ 50 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Arco Longo** | Marcial | Duas Mãos | 1d8 | x3 | Médio | Perfuração | T$ 100 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Besta Pesada** | Marcial | Duas Mãos | 1d12 | 19/x2 | Médio | Perfuração | T$ 50 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Chicote** | Exótica | Uma Mão | 1d3 | x2 | — | Corte | T$ 2 | 1 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Espada Bastarda** | Exótica | Uma Mão | 1d10/1d12 | 19/x2 | — | Corte | T$ 35 | 1 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Katana** | Exótica | Uma Mão | 1d8/1d10 | 19/x2 | — | Corte | T$ 100 | 1 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Machado Anão** | Exótica | Uma Mão | 1d10 | x3 | — | Corte | T$ 30 | 1 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Corrente de Espinhos** | Exótica | Duas Mãos | 2d4/2d4 | 19/x2 | — | Corte | T$ 25 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Machado Táurico** | Exótica | Duas Mãos | 2d8 | x3 | — | Corte | T$ 50 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Rede** | Exótica | Uma Mão | — | — | Curto | — | T$ 20 | 1 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Pistola** | Fogo | Leve | 2d6 | 19/x3 | Curto | Perfuração | T$ 250 | 1 | Livro Básico | Jogo do Ano | p. 145 | N/A |
| **Mosquete** | Fogo | Duas Mãos | 2d8 | 19/x3 | Médio | Perfuração | T$ 500 | 2 | Livro Básico | Jogo do Ano | p. 145 | N/A |

### Munições

| nome | quantidade | preço | espaços | fonte | edição | página |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Virotes** | 20 | T$ 2 | 1 | Livro Básico | Jogo do Ano | p. 144–145 |
| **Pedras** | 20 | T$ 0,5 | 1 | Livro Básico | Jogo do Ano | p. 144 |
| **Flechas** | 20 | T$ 1 | 1 | Livro Básico | Jogo do Ano | p. 144–145 |
| **Balas** | 20 | T$ 20 | 1 | Livro Básico | Jogo do Ano | p. 145 |

### Armaduras e Escudos

| nome | categoria | Defesa | penalidade | pesada | preço | espaços | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Armadura Acolchoada** | Leve | +1 | 0 | Não | T$ 5 | 2 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Armadura de Couro** | Leve | +2 | 0 | Não | T$ 20 | 2 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Couro Batido** | Leve | +3 | -1 | Não | T$ 35 | 2 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Gibão de Peles** | Leve | +4 | -3 | Não | T$ 25 | 2 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Couraça** | Leve | +5 | -4 | Não | T$ 500 | 2 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Brunea** | Pesada | +5 | -2 | Sim | T$ 50 | 5 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Cota de Malha** | Pesada | +6 | -2 | Sim | T$ 150 | 5 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Loriga Segmentada** | Pesada | +7 | -3 | Sim | T$ 250 | 5 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Meia-Armadura** | Pesada | +8 | -4 | Sim | T$ 600 | 5 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Armadura Completa** | Pesada | +10 | -5 | Sim | T$ 3.000 | 5 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Escudo Leve** | Escudo | +1 | -1 | Não | T$ 5 | 1 | Livro Básico | Jogo do Ano | p. 153 | N/A |
| **Escudo Pesado** | Escudo | +2 | -2 | Não | T$ 15 | 2 | Livro Básico | Jogo do Ano | p. 153 | N/A |

---

## 7. Poderes — amostra parcial (`poderes.md`)

| nome | tipo | classe | nível mínimo | requisitos | efeito calculável | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Acuidade com Arma** | Combate | Geral | 1º | Des 1 | Destreza pode substituir Força em ataque e dano com armas corpo a corpo leves ou de arremesso | Livro Básico | Jogo do Ano | p. 124 | N/A |
| **Ataque Poderoso** | Combate | Geral | 1º | For 1 | -2 no ataque corpo a corpo para +5 no dano | Livro Básico | Jogo do Ano | p. 124 | N/A |
| **Esquiva** | Combate | Geral | 1º | Des 1 | +2 na Defesa e +2 em testes de Reflexos | Livro Básico | Jogo do Ano | p. 125 | N/A |
| **Foco em Arma** | Combate | Geral | 1º | Proficiência com a arma escolhida | +2 em ataques com a arma escolhida; repetível para outra arma | Livro Básico | Jogo do Ano | p. 128 | N/A |
| **Saque Rápido** | Combate | Geral | 1º | Iniciativa treinada | +2 Iniciativa; sacar e guardar itens como ação livre; recarga reduzida em uma categoria de ação | Livro Básico | Jogo do Ano | p. 129 | N/A |
| **Vitalidade** | Combate | Geral | 1º | Con 1 | +1 PV por nível e +2 em Fortitude | Livro Básico | Jogo do Ano | p. 129 | N/A |
| **Vontade de Ferro** | Destino | Geral | 1º | Sab 1 | +1 PM a cada dois níveis e +2 em Vontade | Livro Básico | Jogo do Ano | p. 131 | N/A |
| **Sortudo** | Destino | Geral | 1º | Nenhum | Paga 3 PM para repetir um teste recém-realizado, usando o novo resultado | Livro Básico | Jogo do Ano | p. 131 | N/A |
| **Golpe Relâmpago** | Classe | Lutador | 1º | Habilidade Briga | Paga 1 PM para ataque desarmado adicional na rodada | Livro Básico | Jogo do Ano | p. 75 | N/A |
| **Braços Calejados** | Classe | Lutador | 2º | Nível 2 de Lutador | Soma Força na Defesa (sem armadura) limitado pelo nível | Livro Básico | Jogo do Ano | p. 75 | N/A |
| **Sarado** | Classe | Lutador | 2º | For 3, Lutador 2 | Soma Força nos PVs e testes de Fortitude | Livro Básico | Jogo do Ano | p. 75 | N/A |
| **Golpe Imprudente** | Classe | Lutador | 2º | Golpe Relâmpago | Adiciona +1 dado do mesmo tipo ao dano em troca de -5 na Defesa | Livro Básico | Jogo do Ano | p. 75 | N/A |
| **Trocação** | Classe | Lutador | 6º | Lutador 6 | Ao acertar ataque desarmado, paga PM incremental para atacar de novo | Livro Básico | Jogo do Ano | p. 76 | N/A |
| **Trincado** | Classe | Lutador | 10º | Lutador 10 | Soma Constituição nas rolagens de dano desarmado | Livro Básico | Jogo do Ano | p. 76 | N/A |
| **Ataque Especial** | Classe | Guerreiro | 1º | Guerreiro 1 | 1 PM concede +4 no ataque ou dano; a cada quatro níveis, pode gastar +1 PM para +4 adicionais, distribuídos entre ataque e dano | Livro Básico | Jogo do Ano | p. 65 | N/A |
| **Golpe Pessoal** | Classe | Guerreiro | 5º | Guerreiro 5 | Cria golpe customizado com efeitos/dados extras por PM | Livro Básico | Jogo do Ano | p. 65 | N/A |
| **Fúria** | Classe | Bárbaro | 1º | Bárbaro 1 | 2 PM: +2 em ataque e dano corpo a corpo; a cada cinco níveis, opção de +1 PM para +1 no bônus; não concede RD | Livro Básico | Jogo do Ano | p. 41 | N/A |
| **Inspiração** | Classe | Bardo | 1º | Bardo 1 | Paga 2 PM: aliados em alcance curto ganham +1 em testes | Livro Básico | Jogo do Ano | p. 43 | N/A |
| **Engenhosidade** | Classe | Inventor | 1º | Inventor 1 | Paga 2 PM para somar Inteligência em um teste de perícia | Livro Básico | Jogo do Ano | p. 67 | N/A |
| **Ataque Furtivo** | Classe | Ladino | 1º | Ladino 1 | +1d6 de dano contra alvos desprevenidos/flanqueados | Livro Básico | Jogo do Ano | p. 72 | N/A |
| **Orgulho** | Classe | Nobre | 1º | Nobre 1 | Paga PM para somar Carisma em testes de perícia | Livro Básico | Jogo do Ano | p. 78 | N/A |
| **Golpe Divino** | Classe | Paladino | 1º | Paladino 1 | Paga 2 PM para somar Carisma no ataque e +1d8 de dano | Livro Básico | Jogo do Ano | p. 81 | N/A |
| **Aharadak (Afinidade)**| Concedido | Deuses | 1º | Devoto de Aharadak | +10 em resistência à Tormenta; 1º poder da Tormenta grátis | Livro Básico | Jogo do Ano | p. 133 | N/A |
| **Arsenal (Sangue)** | Concedido | Deuses | 1º | Devoto de Arsenal | Paga 2 PM para ganhar +2 em dano corpo a corpo e RD 5 | Livro Básico | Jogo do Ano | p. 133 | N/A |
| **Kallyadranoch (Escamas)**| Concedido| Deuses | 1º | Devoto de Kally | +2 na Defesa | Livro Básico | Jogo do Ano | p. 134 | N/A |
| **Lin-Wu (Kiai Divino)**| Concedido | Deuses | 1º | Devoto de Lin-Wu | Paga 3 PM para causar dano máximo na rolagem de ataque | Livro Básico | Jogo do Ano | p. 135 | N/A |
| **Lin-Wu (Mestre de Si)**| Concedido| Deuses | 1º | Devoto de Lin-Wu | Passa automaticamente no primeiro teste de Vontade da cena | Deuses de Arton | 1ª Edição | p. 55 | N/A |
| **Megalokk (Presas)** | Concedido | Deuses | 1º | Devoto de Megalokk| Paga 1 PM para realizar ataque extra de Mordida (1d6) | Livro Básico | Jogo do Ano | p. 135 | N/A |
| **Thwor (Fúria Divina)**| Concedido | Deuses | 1º | Devoto de Thwor | Paga 2 PM para entrar em Fúria (+2 em ataque e dano) | Livro Básico | Jogo do Ano | p. 136 | N/A |
| **Anatomia Insana** | Tormenta | Geral | 1º | Nível 1 | 25% de chance de anular crítico/furtivo (+25% a cada 2 poderes) | Livro Básico | Jogo do Ano | p. 136 | N/A |
| **Carapaça** | Tormenta | Geral | 1º | Nível 1 | +2 na Defesa (+2 a cada 2 outros poderes da Tormenta) | Livro Básico | Jogo do Ano | p. 136 | N/A |
| **Corpo Aberrante** | Tormenta | Geral | 1º | 1 poder Tormenta | Dano desarmado aumenta em +1 passo (+1 passo a cada 4 poderes) | Livro Básico | Jogo do Ano | p. 137 | N/A |
| **Dentes Afiados** | Tormenta | Geral | 1º | Nível 1 | Ganha ataque natural de Mordida (1d4) como ataque extra por 1 PM | Livro Básico | Jogo do Ano | p. 137 | N/A |
| **Empunhadura Rubra** | Tormenta | Geral | 1º | Nível 1 | Paga 1 PM: +1 em Luta (+1 a cada 2 outros poderes da Tormenta) | Livro Básico | Jogo do Ano | p. 137 | N/A |
| **Membros Extras** | Tormenta | Geral | 1º | 4 poderes Tormenta | Ganha 2 braços extras para empunhar armas ou escudos | Livro Básico | Jogo do Ano | p. 137 | N/A |
| **Pele Corrompida** | Tormenta | Geral | 1º | Nível 1 | RD 2 contra todos os elementos (+2 a cada 2 poderes) | Livro Básico | Jogo do Ano | p. 137 | N/A |
| **Visco Rubro** | Tormenta | Geral | 1º | Nível 1 | Paga 1 PM: +1 em dano corpo a corpo (+1 a cada 2 outros poderes) | Livro Básico | Jogo do Ano | p. 137 | N/A |

---

## 8. Magias — amostra parcial (`magias.md`)

Revisão: as 24 magias abaixo foram conferidas no Livro Básico, com paginação impressa. O campo de aprimoramentos contém **opções selecionadas**, não a lista integral; consulte a página indicada para os demais efeitos, alvos, condições e requisitos. Esta tabela não representa todas as magias do livro.

| nome | círculo | tradição | execução | alcance | duração | resistência | custo base | dano/cura | aprimoramentos | fonte | edição | página | errata aplic |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Adaga Mental** | 1º | Arcana | Padrão | Curto | Instantânea | Vontade parcial | 1 PM | 2d6 psíquico e atordoa por 1 rodada | +2 PM: +1d6 de dano; +1 PM: conjuração discreta | Livro Básico | Jogo do Ano | p. 178 | N/A |
| **Área Escorregadia** | 1º | Arcana | Padrão | Curto | Cena | Reflexos (ver texto) | 1 PM | Área de 3m ou 1 objeto; pode derrubar | +1 PM: +1 quadrado de 1,5m; +2 PM: CD de Acrobacia 15 | Livro Básico | Jogo do Ano | p. 180 | N/A |
| **Armadura Arcana** | 1º | Arcana | Padrão | Pessoal | Cena | Nenhuma | 1 PM | +5 Defesa | +1 PM: reação contra um ataque; +2 PM: +1 Defesa | Livro Básico | Jogo do Ano | p. 181 | N/A |
| **Arma Mágica** | 1º | Universal | Padrão | Toque | Cena | Nenhuma | 1 PM | +1 ataque/dano | +2 PM: +1 no bônus; +2 PM: +1d6 elemental | Livro Básico | Jogo do Ano | p. 181 | N/A |
| **Bênção** | 1º | Divina | Padrão | Curto | Cena | Nenhuma | 1 PM | +1 ataque/dano para aliados | +2 PM: +1 nos bônus | Livro Básico | Jogo do Ano | p. 182 | N/A |
| **Comando** | 1º | Divina | Padrão | Curto | 1 rodada | Vontade anula | 1 PM | Ordem para 1 humanoide | +1 PM: alvo pode ser criatura; +2 PM: +1 alvo | Livro Básico | Jogo do Ano | p. 184 | N/A |
| **Concentração de Combate** | 1º | Arcana | Livre | Pessoal | 1 rodada | Nenhuma | 1 PM | Rola 2 dados no ataque, usa o melhor | +2 PM: ação padrão e duração cena (requer 2º círculo) | Livro Básico | Jogo do Ano | p. 185 | N/A |
| **Curar Ferimentos** | 1º | Divina | Padrão | Toque | Instantânea | Nenhuma | 1 PM | Cura 2d8+2 | +1 PM: +1d8+1 de cura; +2 PM: alcance curto | Livro Básico | Jogo do Ano | p. 189 | N/A |
| **Explosão de Chamas** | 1º | Arcana | Padrão | Pessoal (cone de 6m) | Instantânea | Reflexos reduz à metade | 1 PM | 2d6 fogo | +1 PM: +1d6 de dano; +1 PM: falha na resistência deixa em chamas | Livro Básico | Jogo do Ano | p. 193 | N/A |
| **Seta Infalível de Talude** | 1º | Arcana | Padrão | Médio | Instantânea | Nenhuma | 1 PM | 2 setas de 1d4+1 essência cada | +2 PM: passa a 3 setas; +2 PM: setas viram lanças de 1d8+1 (requer 2º círculo) | Livro Básico | Jogo do Ano | p. 206 | N/A |
| **Toque Chocante** | 1º | Arcana | Padrão | Toque | Instantânea | Fortitude reduz à metade | 1 PM | 2d8+2 eletricidade | +1 PM: +1d8+1 de dano; +2 PM: combina com ataque corpo a corpo | Livro Básico | Jogo do Ano | p. 209 | N/A |
| **Bola de Fogo** | 2º | Arcana | Padrão | Médio | Instantânea | Reflexos reduz à metade | 3 PM | 6d6 fogo em esfera de 6m de raio | +2 PM: +2d6 de dano | Livro Básico | Jogo do Ano | p. 182 | N/A |
| **Campo de Força** | 2º | Arcana | Padrão | Pessoal | Cena | Nenhuma | 3 PM | 30 PV temporários | +1 PM: reação, duração instantânea e RD 30 contra o próximo dano | Livro Básico | Jogo do Ano | p. 183 | N/A |
| **Oração** | 2º | Divina | Padrão | Curto | Sustentada | Nenhuma | 3 PM | +2 testes/dano aliados; -2 testes/dano inimigos | +2 PM: +1 no bônus ou -1 na penalidade; +7 PM: alcance médio (requer 3º círculo) | Livro Básico | Jogo do Ano | p. 200 | N/A |
| **Relâmpago** | 2º | Arcana | Padrão | Pessoal (linha de 30m) | Instantânea | Reflexos reduz à metade | 3 PM | 6d6 eletricidade | +2 PM: +2d6 de dano | Livro Básico | Jogo do Ano | p. 203 | N/A |
| **Velocidade** | 2º | Arcana | Padrão | Curto | Sustentada | Nenhuma | 3 PM | +1 ação padrão ou de movimento por turno | +0 PM: duração cena, mas ação extra apenas de movimento; +7 PM: múltiplos alvos (requer 4º círculo) | Livro Básico | Jogo do Ano | p. 210 | N/A |
| **Coluna de Chamas** | 3º | Divina | Padrão | Longo | Instantânea | Reflexos reduz à metade | 6 PM | 6d6 fogo + 6d6 luz | +1 PM: +1d6 fogo; +1 PM: +1d6 luz | Livro Básico | Jogo do Ano | p. 184 | N/A |
| **Pele de Pedra** | 3º | Universal | Padrão | Pessoal | Cena | Nenhuma | 6 PM | RD 5 | +4 PM: RD 10 (requer 4º círculo); +1 PM: alvo por toque | Livro Básico | Jogo do Ano | p. 201 | N/A |
| **Voo** | 3º | Arcana | Padrão | Pessoal | Cena | Nenhuma | 6 PM | Deslocamento de voo 12m | +1 PM: alvo por toque; +4 PM: duração de 1 dia (requer 4º círculo) | Livro Básico | Jogo do Ano | p. 211 | N/A |
| **Desintegrar** | 4º | Arcana | Padrão | Médio | Instantânea | Fortitude parcial | 10 PM | 10d12 essência; sucesso: 2d12 | +4 PM: +2d12 no dano total e +1d12 no mínimo | Livro Básico | Jogo do Ano | p. 190 | N/A |
| **Raio Polar** | 4º | Arcana | Padrão | Médio | Instantânea | Fortitude parcial | 10 PM | 10d8 frio e paralisa; sucesso: metade e lento | +3 PM: +2d8 de dano | Livro Básico | Jogo do Ano | p. 203 | N/A |
| **Aura Divina** | 5º | Divina | Padrão | Pessoal (esfera de 9m) | Cena | Vontade parcial (inimigos) | 15 PM | +10 Defesa/resistência a devotos; +5 aos demais aliados | +2 PM: +1 nos bônus de Defesa e resistência | Livro Básico | Jogo do Ano | p. 182 | N/A |
| **Chuva de Meteoros** | 5º | Arcana | Completa | Longo | Instantânea | Reflexos parcial | 15 PM | 15d6 impacto + 15d6 fogo; pode agarrar | +2 PM: +2d6 impacto e +2d6 fogo | Livro Básico | Jogo do Ano | p. 183 | N/A |
| **Mata-Dragão** | 5º | Arcana | Duas rodadas | Pessoal (cone de 30m) | Instantânea | Reflexos reduz à metade | 15 PM | 20d12 essência | +1 PM: +1d12 de dano | Livro Básico | Jogo do Ano | p. 198 | N/A |

---

## 9. Regras Auxiliares (`regras_auxiliares.md`)

### 9.1. Limites de Carga por Força
- **Carga Máxima Sem Penalidade**: **10 + 2 por ponto de Força positivo**, ou **10 - 1 por ponto de Força negativo**.
- **Carga Máxima Absoluta**: o dobro da carga máxima sem penalidade.
- **Condição Sobrecarregado**: Se carregar mais do que a Carga Máxima Sem Penalidade (até o limite absoluto), o personagem fica **Sobrecarregado**: sofre **-5 em penalidade de armadura** e tem seu **deslocamento reduzido em -3m**.

### 9.2. Modificadores de Tamanho
| Categoria de Tamanho | Espaço Ocupado | Alcance Natural | Modificador Furtividade | Modificador Manobra |
| :--- | :--- | :--- | :--- | :--- |
| **Minúsculo** | 1,5m x 1,5m | 1,5m | +5 | -5 |
| **Pequeno** | 1,5m x 1,5m | 1,5m | +2 | -2 |
| **Médio** | 1,5m x 1,5m | 1,5m | 0 | 0 |
| **Grande** | 3m x 3m | 3m | -2 | +2 |
| **Enorme** | 4,5m x 4,5m | 4,5m | -5 | +5 |
| **Colossal** | 9m x 9m ou mais | 9m | -10 | +10 |

### 9.3. Custo de PM por Círculo de Magia
- **1º Círculo**: Custo Base 1 PM (o limite de PM usa o nível na classe com a qual a magia é lançada).
- **2º Círculo**: Custo Base 3 PM (Exige nível 5 em classe conjuradora principal).
- **3º Círculo**: Custo Base 6 PM (Exige nível 9 em classe conjuradora principal).
- **4º Círculo**: Custo Base 10 PM (Exige nível 13 em classe conjuradora principal).
- **5º Círculo**: Custo Base 15 PM (Exige nível 17 em classe conjuradora principal).

### 9.4. Classe de Dificuldade (CD) de Habilidades e Magias
- **Fórmula Padrão de CD**:
  $$\text{CD} = 10 + \frac{1}{2} \text{ do Nível do Personagem} + \text{Atributo-Chave} + \text{Bônus Adicionais}$$

### 9.5. Progressão do Bônus de Treinamento e Perícias
- **Bônus de Treinamento por Patamar**:
  - *Níveis 1º ao 6º (Iniciante)*: **+2**
  - *Níveis 7º ao 14º (Veterano)*: **+4**
  - *Níveis 15º ao 20º (Lenda)*: **+6**
- **Fórmula do Valor Final de Perícia**:
  $$\text{Total da Perícia} = \left(\lfloor \frac{\text{Nível}}{2} \rfloor\right) + \text{Modificador de Atributo} + \text{Bônus de Treinamento (se treinado)} + \text{Outros Bônus}$$

### 9.6. Regras de PV e PM Temporários
- **Ordem de Consumo**: Dano sofrido e custos de habilidade consomem primeiro os PV e PM temporários antes de reduzir os pontos atuais do personagem.
- **Duração**: A menos que especificado de forma diferente, PV/PM temporários desaparecem no fim do dia.

### 9.7. Regras de Armaduras
- **Magia Arcana**: Lançar magia arcana usando qualquer armadura exige um teste de Misticismo com $\text{CD} = 20 + \text{Custo em PM da magia}$. O teste sofre a **Penalidade de Armadura**. Em caso de falha, os PM são gastos e a magia não funciona. Magias de raça, poderes ou itens mágicos não sofrem esta limitação.
- **Restrição de Destreza**: Armaduras pesadas **não permitem aplicar o modificador de Destreza na Defesa** (exceto quando feitas de materiais especiais como Mitral, que permitem até +2).
- **Penalidade em Perícias**: Aplica-se diretamente a Acrobacia, Furtividade e Ladinagem, além de testes de Atletismo para natação.

### 9.8. Conversão e Limites de Moedas
- **Tibar de Ouro (TO)**: Moeda de alto valor. $1 \text{ TO} = 10 \text{ T\$}$.
- **Tibar de Prata (T\$)**: Moeda padrão de comércio em Arton.
- **Tibar de Cobre (TC)**: Moeda fracionária. $10 \text{ TC} = 1 \text{ T\$}$.
- **Carga/Peso de Moedas**: Cada grupo de 1.000 moedas (qualquer combinação de tipos) ocupa **1 espaço** de carga no inventário.

---
*Catálogo estrutural parcial para Tormenta20 (Edição Jogo do Ano). Não importar no `core` sem validação por tabela.*
