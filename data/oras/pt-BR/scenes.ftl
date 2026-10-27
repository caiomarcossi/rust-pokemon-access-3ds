### What Omega Ruby and Alpha Sapphire show without words, in pa3ds's own words, a beat at a time,
### said under [pa3ds.ui] descriptions. When each is said is `../scenes.toml`'s, every language's;
### a beat is found here by its id there (ADR-0040). A beat this language has no words for is said
### in English's.
###
### A name the game shows is never written here: `{ POKEMON(383) }` is the game's own word out of
### the player's ROM, as are `{ TRAINER(235) }`, `{ ITEM(534) }`, `{ PLACE(198) }`, and a whole line
### of the game's text drawn as a picture, `{ TEXT(123, 91) }`, and the game's own name, `{ GAME() }`.
### Words drawn as a picture no line holds are the picture's words in `pictures.ftl`, by its
### texture, `{ PICTURE("push_start") }`. `{ MEMORY(24) }` is the Pokémon the
### player's own save keeps for the credits' 24th moment, in the form it keeps, and a beat naming one
### the save has not kept is left out. `{ YOURS(24) }` is the same Pokémon as the player's own: the
### nickname they gave it, or `scene-yours` below with its name in. Someone the game never names is
### `{ SPRITE(179) }`, called what a speaker drawn as sprite 179 is: `sprites.ftl`, behind the
### player's own name for them. A scene whose names the ROM does not have is left out, and logged.
###
### Each scene was described from screenshots of a replay; the comments say which, and what the
### game shows around it.

# The player's own Pokémon they gave no nickname, the game's name for it in: "your Mega Latios".
scene-yours = seu { $pokemon }

# Omega Ruby's intro, from boot to the title screen, and the title screen
# (or_intro_and_title_screen, spike 236). The studio's credit and the title screen's prompt are
# pictures with no line in the game's text, said as `pictures.ftl` calls them, `{ PICTURE(...) }`;
# the logo is the game's name, as its ROM calls it, `{ GAME() }`.
#
# The intro's first part, on the field's engine.
or-intro-forest = Um { POKEMON(267) } flutua entre folhas altas e úmidas. Abaixo está um pequeno lago escuro na floresta, dois pequenos Pokémon espreitando em sua borda.
or-intro-drops = A vista afunda na água. Uma gota de orvalho cai da ponta de uma folha e anéis se espalham pelo lago.
or-intro-credit = As palavras desaparecem sobre a água: { PICTURE("presented_by") } { PICTURE("gf_logo") }.
or-intro-volcano = A vista sobe através das plantas. Além da floresta, há duas montanhas sob um céu azul, uma delas é um vulcão fumegante.
or-intro-latios = { POKEMON(381) } desce sobre as árvores e sobe para o céu. Um flash de branco.
or-intro-island = Bem acima do mar, { POKEMON(381) } passa por uma ilha: um vulcão marrom brilhando em vermelho em sua cratera, um campo amarelo de cinzas, uma floresta verde. Ele mergulha em uma faixa de névoa.
or-intro-bike = Em uma estrada de terra, uma bicicleta vermelha passa. Seu cavaleiro é um menino de boné branco, roupa vermelha e preta e mochila verde.
or-intro-loop = Ele olha para cima. { POKEMON(381) } percorre o céu acima dele e foge. Ele fecha os olhos e sorri.
or-intro-face = A visão se aproxima de seu rosto e tudo fica branco.

# Its second part, a movie of quick cuts.
or-intro-movie-1 = Um { ITEM(4) }, desenhado com pinceladas de tinta, corre em sua direção. Em seguida, flashes rápidos: { POKEMON(323) } em um anel de fogo, { POKEMON(319) } saltando de águas escuras, { POKEMON(383) } rugindo sob um sol escaldante.
or-intro-movie-2 = { POKEMON(257) } em um deserto, chamas saindo de seus pulsos. { POKEMON(260) } à beira-mar, punhos levantados. { POKEMON(254) } em uma floresta escura.
or-intro-movie-3 = { TRAINER(235) } está em uma coluna de fogo e levanta os óculos. Então { TRAINER(557) }, em uma caverna de paredes esculpidas.
or-intro-movie-4 = Contra um céu laranja, a forma negra de { POKEMON(383) }, uma luz acesa em seu peito. Um sinal como a letra ômega se forma no fogo, as chamas giram em torno dele e a luz branca o inunda.

# The title screen.
or-title-logo = A tela de título. O logotipo Pokémon brilha na escuridão entre fragmentos de cristal vermelho e depois explode em luz branca.
or-title = O logotipo diz { GAME() }, queimando em dourado e vermelho sobre um raio de chamas. Na parte inferior, piscando: { PICTURE("push_start") }. Na tela inferior, { POKEMON(383) } surge em meio à fumaça e às brasas e depois desaparece.

# Alpha Sapphire's intro, the same as Omega Ruby's but for Latias in Latios's place, and the same
# boy on a girl's save (as_intro_and_title_screen); the rest of its beats are Omega Ruby's words.
as-intro-forest = { or-intro-forest }
as-intro-drops = { or-intro-drops }
as-intro-credit = { or-intro-credit }
as-intro-volcano = { or-intro-volcano }
as-intro-latias = { POKEMON(380) } desce sobre as árvores e sobe para o céu. Um flash de branco.
as-intro-island = Bem acima do mar, { POKEMON(380) } passa por uma ilha: um vulcão marrom brilhando em vermelho em sua cratera, um campo amarelo de cinzas, uma floresta verde. Ele mergulha em uma faixa de névoa.
as-intro-bike = { or-intro-bike }
as-intro-loop = Ele olha para cima. { POKEMON(380) } percorre o céu acima dele e foge. Ele fecha os olhos e sorri.
as-intro-face = { or-intro-face }

# Its movie: Team Aqua's side where Omega Ruby's shows Team Magma's.
as-intro-movie-1 = Um { ITEM(4) }, desenhado com pinceladas de tinta, corre em sua direção. Em seguida, flashes rápidos: { POKEMON(319) } saltando de águas escuras, { POKEMON(323) } em um anel de fogo, { POKEMON(382) } flutuando no mar enquanto nuvens de tempestade se reúnem e chuva açoitando um mar escuro.
as-intro-movie-2 = { or-intro-movie-2 }
as-intro-movie-3 = { TRAINER(178) }, uma forma escura nas profundezas entre bolhas crescentes, depois ri enquanto segura o colarinho. Então { TRAINER(557) }, em uma caverna de paredes esculpidas.
as-intro-movie-4 = Contra o azul profundo, a forma preta de { POKEMON(382) }, uma luz queimando em seu coração. Um sinal como a letra alfa se forma na água, gotas azuis giram em torno dele e uma luz branca inunda.

# Its title screen.
as-title-logo = A tela de título. O logotipo Pokémon brilha na escuridão entre fragmentos de cristal azul e depois explode em luz branca.
as-title = O logotipo diz { GAME() }, brilhando em dourado e azul sobre luz cintilante. Na parte inferior, piscando: { PICTURE("push_start") }. Na tela inferior, { POKEMON(382) } nada através de bolhas nas profundezas, suas marcas brilhando em branco.

# The opening, before the player is anywhere: the professor's lines are the rest of the game's
# text, file 118, and the stretches between them where he says nothing are described
# (OR_character-desc.ctm, 2026-10-15).
#
# "And this...is what we call a POKéMON.", then 12 s without a word.
opening-pokemon-1 = { POKEMON(298) } aparece ao lado do professor: pequeno, redondo e azul, com grandes orelhas redondas e uma bola azul na ponta da cauda.
opening-pokemon-2 = A imagem recua. O professor está na telinha de um portátil amarelo com alça de transporte, deitado na traseira escura de um caminhão em movimento entre beliches, caixas de papelão e um rato de pelúcia.
opening-pokemon-3 = Lá fora, as portas traseiras marrons da caminhonete balançam ao longo de uma estrada de terra através de uma floresta ensolarada.

# "...But that's enough about me!", then 21 s without a word.
opening-forest-1 = { POKEMON(357) } bate suas largas asas de folhas e voa entre as árvores. A vista afunda na grama alta.
opening-forest-2 = Um { POKEMON(278) } surge das copas das árvores com asas brancas longas e finas, e um pequeno pássaro escuro sobe atrás dele em direção ao céu azul.
opening-forest-3 = Eles voam em direção ao sol, e sua luz inunda tudo de branco.
opening-forest-4 = De volta ao caminhão escuro, uma linha brilhante de luz brilha entre as portas fechadas do computador portátil, onde o professor espera na tela.

# "...You're the one who's moving to Littleroot Town, where I live!", then 8 s without a word. The
# same words are line 56 too, and either matches.
opening-handheld-1 = Na tela do computador de mão, você é mostrado por trás e depois se vira para encarar a frente com um sorriso.
opening-handheld-2 = A tela muda para um mapa da região: ilhas verdes em um mar azul, suas cidades marcadas em vermelho.
opening-handheld-3 = O computador de mão é pousado e o caminhão escuro segue em frente.

# "...Come see me in my Pokémon Lab!", then 11 s without a word until the player stands in the
# truck, zone 0x18f.
opening-truck-1 = As portas traseiras do caminhão se abrem e a luz do dia entra até que tudo fique branco.
opening-truck-2 = A luz desaparece. Você fica na traseira do caminhão estacionado entre caixas de papelão e móveis, com a luz do sol entrando pelas portas abertas.

# May, before she says her name (movie1_opening, Omega Ruby): her mother's "She should be in her
# room upstairs", then a demo of 9 s of her room, and the player arrives in it.
may-room-1 = Um quarto no andar de cima: um mapa na parede, uma cama com colchas vermelhas embaixo da janela, uma escrivaninha encostada na parede. Uma garota está parada na mesa, de costas para você, com as mãos levantadas para o laço vermelho e branco em seu cabelo.
may-room-2 = De perto, ela folheia um caderno sobre a mesa e verifica a bolsa em seu cinto.
may-room-3 = Você fica no topo da escada, em um canto do quarto dela.

# "Pokémon fully restored, items all ready, and..."
may-turns-1 = Ela se vira, fica assustada ao ver você e se aproxima.

# "[name], I'll see you later!"
may-leaves-1 = Ela sorri, passa por você e desce as escadas, deixando você sozinho no quarto dela.

# Brendan, before he says his name (or_opening_as_a_girl, Omega Ruby): his mother's "He should be in
# his room upstairs", then a demo of 9 s of his room, and the player arrives in it.
brendan-room-1 = Um quarto no andar de cima: uma janela com cortinas azuis claras, uma cama com colchas claras embaixo, uma escrivaninha encostada na parede, um mapa na parede. Um menino está parado na mesa, de costas para você, com as mãos levantadas para o chapéu branco na cabeça.
brendan-room-2 = De perto, ele folheia um caderno sobre a mesa e depois segura a alça da bolsa com um sorrisinho.
brendan-room-3 = Você fica no topo da escada, em um canto do quarto dele.

# "Pokémon fully restored... Items all packed, and..."
brendan-turns-1 = Ele se vira e caminha até você.

# "Some other time, OK?"
brendan-leaves-1 = Ele sorri, passa por você e desce as escadas, deixando você sozinha no quarto dele.

# Route 102 (movie4_petalburg): "Look! Up there! Check 'em out, [name]!", 15 s before she names
# what they saw.
may-beautifly-1 = Você e { TRAINER(4) } ficam lado a lado na grama alta, diante de uma fileira de pinheiros. Três Pokémon borboleta saem das árvores batendo suas asas amarelas e pretas brilhantes.
may-beautifly-2 = Eles passam voando pelas árvores em direção aos telhados vermelhos de uma cidade.
may-beautifly-3 = Eles sobem ao céu e flutuam entre as nuvens.

# Its twin for a girl: Brendan's "Whoa! Check 'em out, [name]!", composed from May's, and seen
# (or_beautifly_scene_as_a_girl).
brendan-beautifly-1 = Você e { TRAINER(1) } estão lado a lado na grama alta, de frente para uma fileira de pinheiros. Três borboletas Pokémon saem das árvores com asas amarelas e pretas brilhantes.
brendan-beautifly-2 = Eles passam voando pelas árvores em direção aos telhados vermelhos de uma cidade.
brendan-beautifly-3 = Eles sobem ao céu e flutuam entre as nuvens.

# Granite Cave's last room (scene_granite_cave_mural, in Alpha Sapphire and replayed in Omega
# Ruby): a demo of 22 s as the player walks in toward the steps, before Steven's first line, then
# the player is walked up to him. He is not named until he names himself, as a sighted player
# does not know him yet. Each version paints a wall of its own; the rest is the same.
or-granite-mural-1 = A tela fica escura. Um círculo de luz se move lentamente sobre imagens pintadas na rocha: uma grande fera escura com costas irregulares e escamosas e mandíbulas bem abertas, depois uma fileira de montanhas fumegantes.
or-granite-mural-2 = A visão recua. As imagens cobrem toda uma parede da caverna: a fera erguendo-se entre anéis e plumas onduladas à esquerda, figuras minúsculas sob um raio de luz à direita e as montanhas fumegantes abaixo. Um homem está sozinho no topo de um lance de degraus de pedra abaixo dele, olhando para cima.
or-granite-mural-3 = De perto, o homem tem cabelo azul claro e terno escuro. Ele olha para a parede com a mão no queixo e depois cruza os braços, imerso em pensamentos.
or-granite-mural-4 = Você fica ao pé dos degraus e sobe até o lado dele.

as-granite-mural-1 = A tela fica escura. Um círculo de luz se move lentamente sobre imagens pintadas na rocha: listras irregulares, fileiras de espirais onduladas como ondas e uma grande forma escura marcada com um anel.
as-granite-mural-2 = A visão recua. As imagens cobrem toda uma parede da caverna: pequenas figuras sob raios de luz à esquerda, nuvens rodopiantes e ondas à direita, e a grande forma entre elas. Um homem está sozinho no topo de um lance de degraus de pedra abaixo dele, olhando para cima.
as-granite-mural-3 = De perto, o homem tem cabelo azul claro e terno escuro. Ele olha para a parede com a mão no queixo e depois cruza os braços, imerso em pensamentos.
as-granite-mural-4 = Você fica ao pé dos degraus e sobe até o lado dele.

# Steven's "Good luck to you. I must be going now."
granite-steven-leaves-1 = { TRAINER(557) } desce os degraus e sai da sala, deixando você sozinho diante da parede pintada.

# Seafloor Cavern, zone 0x6f: Archie's "Augh... You...", after Maxie's battle and before Groudon
# wakes, a demo of 65 s.
or-groudon-wakes-1 = { TRAINER(235) } caminha até a borda da lava, onde { POKEMON(383) } está deitado enrolado, cinza como pedra.
or-groudon-wakes-2 = { TRAINER(235) } segura o { ITEM(534) }. Ele brilha com luz vermelha e fendas de luz percorrem a concha de pedra de { POKEMON(383) }.
or-groudon-wakes-3 = { POKEMON(383) } surge da lava, vermelho novamente. Ele abre os olhos e olha para o teto da caverna.
or-groudon-wakes-4 = Ele se eleva sobre a lava como { TRAINER(235) } e você observa da costa.
or-groudon-wakes-5 = { POKEMON(383) } irrompe no céu aberto sobre o mar em um flash de luz.
or-groudon-wakes-6 = O céu fica com um amarelo ardente e { POKEMON(383) } ruge.

# Tabitha's "Child! You're coming, too! Hurry up!": the lava pool is empty, and everyone runs out
# of the cavern onto Route 128, zone 0x39.
or-cavern-empties-1 = A piscina de lava está vazia. { POKEMON(383) } desapareceu.
or-cavern-empties-2 = O céu brilha branco e dourado. Você está em um banco de areia no mar com { TRAINER(178) }, { TRAINER(235) } e { TRAINER(691) }, sob um sol escaldante.

# Route 128: Tabitha's "Let's go. Boss Maxie...".
or-route128-tabitha-1 = { TRAINER(235) } e { TRAINER(691) } vão embora.

# Archie's "So...I'll be waitin' for you in Sootopolis.", before Steven comes.
or-route128-archie-1 = { TRAINER(178) } sai. Você fica sozinho no banco de areia, olhando para a claridade.
or-route128-archie-2 = Alguém voa para fora do brilho e pousa ao seu lado.

# Steven's "Good luck to you."
or-route128-steven-1 = { TRAINER(557) } voa para a luz.

# The Cave of Origin's deepest floor, zone 0x76, each time the player comes down into it: the
# only way on is north, where the floor narrows onto a ledge at the magma, and the scene where the
# Magma Suit goes on starts as the player walks into that narrowing (an invisible trigger seven
# cells wide at (17, 24), which fills it). Nothing stands there; a sighted player sees the floor
# narrow toward the glow.
or-cave-depths-1 = As escadas descem na extremidade leste de uma passagem. Ela segue para oeste e depois para norte através de uma abertura em um amplo corredor, que se estreita em sua extremidade norte em direção a um rio de magma brilhante.

# The Magma Suit goes on as the scene starts, and its line comes after.
or-magma-suit-1 = O traje é vermelho, com botas pesadas e um capacete redondo através do qual você olha.

# "fitted the suit with the Blue Orb!"
or-blue-orb-1 = O { ITEM(535) } brilha e uma bolha de luz azul se fecha ao seu redor.
or-blue-orb-2 = A luz azul passa pelo rio de magma ao seu lado, então tudo pisca em branco.

# Groudon's roar there.
or-groudon-roars-1 = { POKEMON(383) } surge do magma ao seu lado.

# "The sound cut out...", then a demo of 24 s, to zone 0x7a.
or-sound-cut-out-1 = Você pula nas costas de { POKEMON(383) }.
or-sound-cut-out-2 = { POKEMON(383) } penetra no magma e carrega você ao longo de um rio de fogo, cada vez mais fundo.
or-sound-cut-out-3 = Uma luz ofuscante preenche o túnel à frente e { POKEMON(383) } leva você para dentro dele.

# Zone 0x7a, Groudon's "Gurrrooouuuhhh!!!" as the player faces it, then a demo of 56 s to the
# battle.
or-groudon-faced-1 = Grandes cristais brilham sobre o magma. { POKEMON(383) } está diante de você.
or-groudon-faced-2 = A luz rosa se derrama sobre { POKEMON(383) } e o sela dentro de um cristal gigante.
or-groudon-faced-3 = O cristal fica vermelho e quebra, e um símbolo dourado do Ômega brilha.
or-groudon-faced-4 = { POKEMON(383) } irrompe em sua forma primitiva, enorme e resplandecente de lava, e eleva-se sobre você.

# "Gotcha! Groudon was caught!", then a demo of 40 s and the field across Hoenn's towns, to
# Sootopolis.
or-groudon-caught-1 = Os cristais acima brilham suavemente. Você faz o seu caminho de volta pela caverna.
or-groudon-caught-2 = Muito acima, a luz sobe da cratera de { PLACE(198) } para o céu escaldante e o brilho se desfaz.
or-groudon-caught-3 = O céu fica azul profundo e brilhos verdes caem como neve sobre um mar calmo.
or-groudon-caught-4 = As pessoas saem e olham para o céu cintilante.
or-groudon-caught-5 = { TRAINER(518) } olha para o céu.
or-groudon-caught-6 = Uma multidão olha para o céu e, na praia, um grupo de capuzes vermelhos também observa.
or-groudon-caught-7 = Perto de um foguete em sua torre de lançamento, { TRAINER(694) } olha para o céu.
or-groudon-caught-8 = O brilho desapareceu. Seus amigos esperam no cais antes da caverna.

# Alpha Sapphire, from movie1_kyogre_awakens. Seafloor Cavern, zone 0x6e: Maxie's "No... That...",
# after Archie's battle and before Kyogre wakes, a demo of 65 s.
as-kyogre-wakes-1 = { TRAINER(178) } caminha até a beira da água, onde { POKEMON(382) } permanece imóvel, cinza como pedra.
as-kyogre-wakes-2 = { TRAINER(178) } segura o { ITEM(535) }. Ele brilha com luz azul e a luz irrompe sob a concha de pedra de { POKEMON(382) }.
as-kyogre-wakes-3 = { POKEMON(382) } brilha em azul novamente. Ele abre bem a boca e sobe na água.
as-kyogre-wakes-4 = Ele nada ao redor da piscina como { TRAINER(178) } e você observa da costa.
as-kyogre-wakes-5 = { POKEMON(382) } irrompe do mar para o céu aberto em um jato de água.
as-kyogre-wakes-6 = Nuvens de tempestade se fecham no céu, chuva forte açoita o mar e { POKEMON(382) } ruge.

# Shelly's "You, too! Well, hurry up!": the pool is still, and everyone runs out of the cavern onto
# Route 128, zone 0x39.
as-cavern-empties-1 = A piscina está parada. { POKEMON(382) } desapareceu.
as-cavern-empties-2 = Nuvens negras de tempestade enchem o céu e uma forte chuva cai. Você está em um banco de areia no mar com { TRAINER(235) }, { TRAINER(178) } e { TRAINER(688) }.

# Route 128: Shelly's "Let's go, Archie!".
as-route128-shelly-1 = { TRAINER(178) } e { TRAINER(688) } vão embora.

# Maxie's "I will expect your arrival in Sootopolis.", before Steven comes.
as-route128-maxie-1 = { TRAINER(235) } sai. Você fica sozinho no banco de areia sob uma chuva torrencial.
as-route128-maxie-2 = Alguém sai voando da tempestade e pousa ao seu lado.

# Steven's "Good luck to you."
as-route128-steven-1 = { TRAINER(557) } voa para a tempestade.

# Alpha Sapphire, from movie2_cave_of_origin. The Cave of Origin's deepest floor is zone 0x75, Omega Ruby's 0x76 laid out the
# same: its warp and its trigger stand on the same cells, but the hall opens onto deep water.
as-cave-depths-1 = As escadas descem na extremidade leste de uma passagem. Ele segue para oeste e depois para norte através de uma abertura em um amplo salão, que se estreita na extremidade norte em direção a uma piscina de águas azuis profundas.

# The suit goes on as the scene starts, and its line comes after.
as-aqua-suit-1 = O traje é azul com detalhes brancos, com luzes amarelas no peito e um capacete redondo através do qual você olha.

# "fitted the suit with the Red Orb!"
as-red-orb-1 = O { ITEM(534) } brilha e uma bolha de luz vermelha se fecha ao seu redor.
as-red-orb-2 = A luz vermelha passa pelas águas profundas ao seu lado, então tudo pisca em branco.

# Kyogre's roar there.
as-kyogre-roars-1 = { POKEMON(382) } surge da água ao seu lado.

# "The sound cut out...", then a demo of 24 s, to zone 0x79.
as-sound-cut-out-1 = Você monta nas costas de { POKEMON(382) }.
as-sound-cut-out-2 = { POKEMON(382) } mergulha e leva você para baixo em águas azuis profundas, passando por grandes rochas, cada vez mais fundo.
as-sound-cut-out-3 = Uma luz ofuscante enche a água à frente e { POKEMON(382) } leva você para dentro dela.

# Zone 0x79, Kyogre's "Krrraaawwwrrr!!!" as the player faces it, then a demo of 57 s to the
# battle.
as-kyogre-faced-1 = Grandes cristais brilham sobre a água, sob as quedas de água do teto da caverna. { POKEMON(382) } flutua diante de você.
as-kyogre-faced-2 = A luz azul jorra sobre { POKEMON(382) } e o sela dentro de um cristal gigante.
as-kyogre-faced-3 = O cristal escurece e um símbolo Alfa azul brilhante brilha.
as-kyogre-faced-4 = { POKEMON(382) } irrompe em sua forma primal, enorme e azul, com marcas brilhantes, erguendo-se sobre você.

# "Gotcha! Kyogre was caught!", then a demo of 40 s and the field across Hoenn's towns, to
# Sootopolis.
as-kyogre-caught-1 = Os cristais acima brilham suavemente. Você faz o seu caminho de volta pela caverna.
as-kyogre-caught-2 = Muito acima, a luz verde sobe da cratera de { PLACE(198) } para as nuvens de tempestade, e as nuvens se desfazem.
as-kyogre-caught-3 = O céu fica azul profundo e brilhos verdes caem como neve sobre um mar calmo.
as-kyogre-caught-4 = As pessoas saem e olham para o céu cintilante.
as-kyogre-caught-5 = { TRAINER(518) } olha para o céu.
as-kyogre-caught-6 = Uma multidão olha para o céu e na praia um grupo de listras azuis e brancas também observa.
as-kyogre-caught-7 = Perto de um foguete em sua torre de lançamento, { TRAINER(683) } olha para o céu.
as-kyogre-caught-8 = A chuva acabou. Seus amigos esperam no cais antes da caverna.

# Victory Road's way out, zone 0x7f, from movie48_battle_with_wally: Wally's "Ah! [name]!" as the
# player walks onto the trigger across the path at (21, 50).
wally-waits-1 = O caminho segue para o norte através de um prado de flores vermelhas entre duas piscinas, até degraus de madeira. { TRAINER(518) } fica no topo deles e desce para encontrá-lo.

# "Standing here facing you again gives me chills..."
wally-chills-1 = { TRAINER(518) } fecha os olhos e inclina a cabeça, depois olha para você, determinado.

# "Arghhh!!!", after his battle.
wally-loses-1 = { TRAINER(518) } abaixa os braços e grita, com os olhos bem fechados, depois abaixa a cabeça.

# "When I do, promise we'll battle again! That's a promise I intend to live up to!"
wally-promises-1 = { TRAINER(518) } sai do caminho em direção às flores e o caminho para o norte até os degraus está livre.

# The Pokémon League, from movies 49 to 52. Each of the Elite Four's corridors is a bridge north to
# a hall, the same in shape and dressed for the member beyond it, and plays its scene as the
# player walks across the trigger halfway along it (row 34, five cells wide) with no line before
# or after: Sidney's 0x16b, Phoebe's 0x167, Glacia's 0x168 and Drake's 0x169, about 17 s before
# the player walks on into the room. The first is entered after "You heard doors closing!".
corridor-one-1 = Atrás de você, o caminho de volta está fechado. Uma longa ponte de madeira laqueada de vermelho segue para o norte através de um céu escuro de brasas flutuantes, até um grande salão com telhados curvos e janelas brilhantes.

corridor-one-bridge-1 = Nuvens douradas surgem em ambos os lados abaixo da ponte e brasas giram ao seu redor.
corridor-one-bridge-2 = As portas do salão se abrem para a luz branca.

corridor-two-1 = Uma ponte de pedra roxa, coberta de glicínias, atravessa um lago escuro de flores de lótus pálidas e brilhantes até um salão sombrio.

corridor-two-bridge-1 = Uma chama roxa fantasmagórica surge diante das portas do salão e fragmentos dela se afastam.
corridor-two-bridge-2 = Lanternas ao longo do corredor acendem uma por uma.
corridor-two-bridge-3 = As portas do salão se abrem para a escuridão.

corridor-three-1 = A neve flutua através de uma névoa azul-clara ao redor de uma ponte de pedra rosa. À frente está um salão coberto de neve e pendurado com pingentes de gelo.

corridor-three-bridge-1 = Uma nevasca aumenta e esconde o corredor de branco.
corridor-three-bridge-2 = Isso limpa. As portas do salão estão abertas para a escuridão, e o gelo se espalha pela ponte em direção aos seus pés.

corridor-four-1 = Uma ponte vermelha atravessa um mar de lava brilhante sob um céu esfumaçado cheio de faíscas, até um salão com uma grande cabeça de dragão esculpida na porta.

corridor-four-bridge-1 = Faixas de fogo dourado varrem o salão como algo voando.
corridor-four-bridge-2 = Chamas irromperam da cabeça do dragão, seus olhos brilharam e as portas se abriram para uma luz brilhante.

# Sidney's room, zone 0x162: as the player comes in he sits on a black chair, eyes closed, then
# opens them and gets up, before his first line.
sidney-rises-1 = Uma sala quadrada com uma quadra de batalha marcada no chão e uma grande porta de Pokébola na parede oposta. { TRAINER(553) } se levanta de uma cadeira preta diante dele, sorrindo.

# "Go on to the next room and challenge your next battle!"
sidney-beaten-1 = A porta da Pokébola atrás de { TRAINER(553) } se abre.

# Phoebe's room, zone 0x163: her scene plays as the player walks up to her.
phoebe-waits-1 = Uma sala escura de pedra roxa, iluminada por chamas pálidas ao longo das paredes. { TRAINER(554) } ri, com a mão na boca.

phoebe-rises-1 = Ela anda lentamente em círculo ao seu redor enquanto fala.

# "Come on! Just try and see if you can even inflict damage on my Pokémon!"
phoebe-challenges-1 = { TRAINER(554) } salta para trás antes da porta, abre os braços e ri.

# "Go ahead. Move on to the next room."
phoebe-beaten-1 = A porta da Pokébola atrás de { TRAINER(554) } se abre.

# Glacia's room, zone 0x164.
glacia-waits-1 = Uma sala de gelo azul claro, com neve caindo pelas janelas. { TRAINER(555) } está diante de uma porta de Pokébola com um longo vestido lilás.

# "Advance to the next room. There you will learn how fearsome the Pokémon League can truly be!"
glacia-beaten-1 = A porta da Pokébola atrás de { TRAINER(555) } se abre.

# Drake's room, zone 0x165.
drake-waits-1 = Uma sala de âmbar quente, com lâmpadas vermelhas brilhando nas paredes de madeira. A visão sobe das botas de { TRAINER(556) } até seu rosto severo; ele fica com os braços cruzados.

# "Go! Go onward! The Champion is waiting!"
drake-beaten-1 = { TRAINER(556) } dá um passo para o lado e a porta da Pokébola atrás dele se abre.

# The Champion's corridor, zone 0x16a, from movie53_battle_with_steven: no trigger; the player
# walks up it to the light. Then the Champion's room, zone 0x166, whose own script starts as the
# player arrives: it walks them 12 rows up a staircase, whatever they hold, and keeps them there
# while the view goes to Steven, his first line 22.2 s after arriving in both of Orion's movies.
# So the room is described as the player arrives, a beat as each shot comes (Orion, 2026-10-16).
champion-corridor-1 = Um longo e estreito corredor de pedra cinza segue para o norte até uma porta que emite luz branca.
champion-corridor-2 = Você sobe uma ampla escadaria até um vasto salão de pedra clara.
champion-corridor-3 = Na extremidade oposta, { TRAINER(557) } está nos degraus abaixo de um arco escuro, de costas para você.
champion-corridor-4 = Ele vira a cabeça e sorri.

# "You are rightfully the Hoenn region's new...", after his battle; the friend who runs in is the
# rival, named as the speaker after it.
steven-beaten-1 = Alguém corre atrás de você.

# May's "Huh? Wait—what? What? ... It's already over?"; Birch is drawn as HAKASE_RS, sprite 0xb3.
may-runs-in-1 = { SPRITE(179) } entra atrás dela.

# Brendan's "You didn't... No, you couldn't have... Did you already beat the Champion?!", where
# the player is a girl (movie53_battle_with_steven_as_a_girl); each of May's scenes here has a
# twin of his, on his line, for him alone.
brendan-runs-in-1 = { SPRITE(179) } entra atrás dele.

# "No, I should call you the Hoenn region's new Pokémon League Champion... Come with me, Champion."
steven-leads-on-1 = { TRAINER(557) } conduz você pelas escadas até o arco, e os outros ficam abaixo.

# May's "Raiden! Honestly, congratulations!"
may-congratulates-1 = { TRAINER(4) } acena do pé da escada enquanto você segue { TRAINER(557) } pelo arco.

# Brendan's "That's awesome, [name]. Congratulations...really!"
brendan-congratulates-1 = { TRAINER(1) } levanta a mão da base da escada enquanto você segue { TRAINER(557) } pelo arco.

# The Hall of Fame, zone 0x16d: "This room... This is where we maintain the records of Pokémon..."
hall-of-fame-room-1 = Um salão longo e escuro, iluminado por lâmpadas ao longo das paredes. Na sua extremidade está uma máquina redonda com uma parte superior azul brilhante.

# "Come. We will use this machine to record your name...", then the Hall of Fame's demo of 37 s,
# and back in the Champion's room, zone 0x166, where the script holds the player at the top of
# the steps for 21.6 s before the friend's "Welcome back, [name]": the high windows, the player
# stepping out of the archway, a close-up of the friend with eyes shut.
hall-of-fame-1 = Você coloca suas Pokébolas nas cavidades ao redor da máquina e elas brilham.
hall-of-fame-2 = Um por um, seus Pokémon aparecem em uma explosão de luz contra um fundo brilhante, enquanto os confetes caem.
hall-of-fame-3 = Você fica com seu Pokémon ao seu redor. { TEXT(123, 111) }
hall-of-fame-4 = A luz entra pelas janelas altas.
hall-of-fame-5 = Você sai no topo da escada. Abaixo, seu amigo espera sozinho, de olhos fechados.

# May's "Hee hee..."
may-giggles-1 = { TRAINER(4) } sorri, olhos fechados, bochechas rosadas.

# Brendan's "D-don't go trying to read anything into it, though!"
brendan-blushes-1 = { TRAINER(1) } sorri, depois franze as sobrancelhas e desvia o olhar.

# May's "So...come on! Let's go home! Back to Littleroot Town!", then the credits: they start
# three times, before May's battle, after it and after the game saves, and their pictures run at
# their own pace, so each beat is timed from the start of its part. The Pokémon in each of the
# journey's pictures are the player's own, `{yours:N}` by their nicknames, but for the rival's
# starter, `{memory:2}`; the rest are the story's, Wally's and
# Steven's in their Mega forms, named from the game's list of forms (text 5). Seen in Omega
# Ruby with May as the rival, and with a line of hers, so with her alone; Brendan's twin follows.
or-credits-may-1 = Você desce as etapas para { TRAINER(4) }. A luz entra pelas janelas altas até que tudo fique branco.
or-credits-may-2 = Luz solar através de árvores altas. Você anda de bicicleta por um caminho na floresta.
or-credits-may-3 = { TRAINER(4) } vem ao seu lado em sua própria bicicleta.
or-credits-may-4 = Agora, fotos da sua viagem vão e vêm sobre um mapa antigo de Hoenn. Uma sacola caiu no caminho com Pokébolas caindo; um { POKEMON(261) }; e você, com { YOURS(0) }.
or-credits-may-5 = { TRAINER(4) } com ela { MEMORY(2) }, e você com { YOURS(1) }, perto da foto de um pequeno lago.
or-credits-may-6 = { TRAINER(518) } se lança para lançar uma Pokébola em { POKEMON(280) }, com você ao lado dele.
or-credits-may-7 = Retrato de { TRAINER(561) } e seu { POKEMON(299) } e { YOURS(5) }.
or-credits-may-8 = Retrato de { TRAINER(563) } e seu { POKEMON(296) } e { YOURS(6) }.
or-credits-may-9 = Você e { YOURS(7) } com { TRAINER(557) }.
or-credits-may-10 = { TRAINER(235) }, ao lado de uma imagem de um salão de exposições.
or-credits-may-11 = Retrato de { TRAINER(567) } e seu { POKEMON(82) } e { YOURS(8) }.
or-credits-may-12 = Você e { TRAINER(4) } olham para a foto de uma montanha, com { YOURS(9) }.
or-credits-may-13 = { TRAINER(178) } levanta o punho.
or-credits-may-14 = Você envia { YOURS(10) } contra { TRAINER(235) } e seu { POKEMON(323) }.
or-credits-may-15 = Retrato de { TRAINER(569) } e seu { POKEMON(324) } e { YOURS(12) }.
or-credits-may-16 = Retrato de { TRAINER(570) } e seu { POKEMON(289) } e { YOURS(13) }.
or-credits-may-17 = { POKEMON(381) } e { POKEMON(380) } voam lado a lado.
or-credits-may-18 = Retrato de { TRAINER(571) } e seu { POKEMON(334) } e { YOURS(16) }.
or-credits-may-19 = Você e { YOURS(17) }, perto da foto de um longo corredor.
or-credits-may-20 = Retrato de { TRAINER(552) }, com { YOURS(18) } e { YOURS(19) }.
or-credits-may-21 = { POKEMON(383) }, pela imagem de uma caverna.
or-credits-may-22 = Você e { YOURS(21) } antes de { POKEMON(383) } em sua forma original.
or-credits-may-23 = Retrato de { TRAINER(572) } e seu { POKEMON(350) } e { YOURS(23) }.
or-credits-may-24 = { TRAINER(518) } e seu { TEXT(5, 816) }, e você com { YOURS(24) }.
or-credits-may-25 = Você e { YOURS(25) } enfrentam { TRAINER(557) } e seu { TEXT(5, 796) }.
or-credits-may-26 = Noite. Você passa de bicicleta por uma cidade adormecida.
or-credits-may-27 = As estrelas enchem o céu noturno.
or-credits-may-28 = Noite em { PLACE(208) }. Você está com { TRAINER(4) } entre as árvores sob as estrelas.
or-credits-may-29 = Um pequeno lago entre as folhas.
or-credits-may-30 = { TRAINER(4) } se vira para você e sorri.
or-credits-may-31 = Ela levanta os punhos, pronta para a batalha.
or-credits-may-32 = { TRAINER(4) } joga os braços para o alto e se vira para você.
or-credits-may-33 = Visto de cima, vocês dois caminham lado a lado por um caminho de terra durante a noite.
or-credits-may-34 = { TRAINER(4) } aponta para o céu.
or-credits-may-35 = Estrelas cadentes cruzam o céu noturno, uma após a outra.
or-credits-may-36 = { TEXT(123, 91) }
or-credits-may-37 = A luz da noite incide sobre sua mesa em casa: um caderno aberto, duas Pokébolas e um mapa de Hoenn.
or-credits-may-38 = A luz desaparece e a sala fica escura.
or-credits-may-39 = Um emblema escuro desaparece sobre a mesa e brilha com uma luz verde.

# The same credits in Alpha Sapphire (movie4_hall_of_fame), picture for picture and second for
# second, but five: Archie by the hall of exhibits and Maxie where Archie stood, Archie's Sharpedo
# where Maxie's Camerupt was, and Kyogre where Groudon was.
as-credits-may-1 = Você desce as etapas para { TRAINER(4) }. A luz entra pelas janelas altas até que tudo fique branco.
as-credits-may-2 = Luz solar através de árvores altas. Você anda de bicicleta por um caminho na floresta.
as-credits-may-3 = { TRAINER(4) } vem ao seu lado em sua própria bicicleta.
as-credits-may-4 = Agora, fotos da sua viagem vão e vêm sobre um mapa antigo de Hoenn. Uma sacola caiu no caminho com Pokébolas caindo; um { POKEMON(261) }; e você, com { YOURS(0) }.
as-credits-may-5 = { TRAINER(4) } com ela { MEMORY(2) }, e você com { YOURS(1) }, perto da foto de um pequeno lago.
as-credits-may-6 = { TRAINER(518) } se lança para lançar uma Pokébola em { POKEMON(280) }, com você ao lado dele.
as-credits-may-7 = Retrato de { TRAINER(561) } e seu { POKEMON(299) } e { YOURS(5) }.
as-credits-may-8 = Retrato de { TRAINER(563) } e seu { POKEMON(296) } e { YOURS(6) }.
as-credits-may-9 = Você e { YOURS(7) } com { TRAINER(557) }.
as-credits-may-10 = { TRAINER(178) }, pela foto de um salão de exposições.
as-credits-may-11 = Retrato de { TRAINER(567) } e seu { POKEMON(82) } e { YOURS(8) }.
as-credits-may-12 = Você e { TRAINER(4) } olham para a foto de uma montanha, com { YOURS(9) }.
as-credits-may-13 = { TRAINER(235) } fica de pé, com as mãos ao lado do corpo.
as-credits-may-14 = Você envia { YOURS(10) } contra { TRAINER(178) } e seu { POKEMON(319) }.
as-credits-may-15 = Retrato de { TRAINER(569) } e seu { POKEMON(324) } e { YOURS(12) }.
as-credits-may-16 = Retrato de { TRAINER(570) } e seu { POKEMON(289) } e { YOURS(13) }.
as-credits-may-17 = { POKEMON(381) } e { POKEMON(380) } voam lado a lado.
as-credits-may-18 = Retrato de { TRAINER(571) } e seu { POKEMON(334) } e { YOURS(16) }.
as-credits-may-19 = Você e { YOURS(17) }, perto da foto de um longo corredor.
as-credits-may-20 = Retrato de { TRAINER(552) }, com { YOURS(18) } e { YOURS(19) }.
as-credits-may-21 = { POKEMON(382) }, pela imagem de uma caverna.
as-credits-may-22 = Você e { YOURS(21) } antes de { POKEMON(382) } em sua forma original.
as-credits-may-23 = Retrato de { TRAINER(572) } e seu { POKEMON(350) } e { YOURS(23) }.
as-credits-may-24 = { TRAINER(518) } e seu { TEXT(5, 816) }, e você com { YOURS(24) }.
as-credits-may-25 = Você e { YOURS(25) } enfrentam { TRAINER(557) } e seu { TEXT(5, 796) }.
as-credits-may-26 = Noite. Você passa de bicicleta por uma cidade adormecida.
as-credits-may-27 = As estrelas enchem o céu noturno.
as-credits-may-28 = Noite em { PLACE(208) }. Você está com { TRAINER(4) } entre as árvores sob as estrelas.
as-credits-may-29 = Um pequeno lago entre as folhas.
as-credits-may-30 = { TRAINER(4) } se vira para você e sorri.
as-credits-may-31 = Ela levanta os punhos, pronta para a batalha.
as-credits-may-32 = { TRAINER(4) } joga os braços para o alto e se vira para você.
as-credits-may-33 = Visto de cima, vocês dois caminham lado a lado por um caminho de terra durante a noite.
as-credits-may-34 = { TRAINER(4) } aponta para o céu.
as-credits-may-35 = Estrelas cadentes cruzam o céu noturno, uma após a outra.
as-credits-may-36 = { TEXT(123, 91) }
as-credits-may-37 = A luz da noite incide sobre sua mesa em casa: um caderno aberto, duas Pokébolas e um mapa de Hoenn.
as-credits-may-38 = A luz desaparece e a sala fica escura.
as-credits-may-39 = Um emblema escuro desaparece sobre a mesa e brilha com uma luz verde.

# Brendan's "Now, come on! We're going home! Back to Littleroot Town!", then the credits where the
# player is a girl (movie53_battle_with_steven_as_a_girl): May's credits, picture for picture and
# second for second, with Brendan where she was.
or-credits-brendan-1 = Você desce as etapas para { TRAINER(1) }. A luz entra pelas janelas altas até que tudo fique branco.
or-credits-brendan-2 = Luz solar através de árvores altas. Você anda de bicicleta por um caminho na floresta.
or-credits-brendan-3 = { TRAINER(1) } chega ao seu lado em sua própria bicicleta.
or-credits-brendan-4 = Agora, fotos da sua viagem vão e vêm sobre um mapa antigo de Hoenn. Uma sacola caiu no caminho com Pokébolas caindo; um { POKEMON(261) }; e você, com { YOURS(0) }.
or-credits-brendan-5 = { TRAINER(1) } com seu { MEMORY(2) }, e você com { YOURS(1) }, perto da foto de um pequeno lago.
or-credits-brendan-6 = { TRAINER(518) } se lança para lançar uma Pokébola em { POKEMON(280) }, com você ao lado dele.
or-credits-brendan-7 = Retrato de { TRAINER(561) } e seu { POKEMON(299) } e { YOURS(5) }.
or-credits-brendan-8 = Retrato de { TRAINER(563) } e seu { POKEMON(296) } e { YOURS(6) }.
or-credits-brendan-9 = Você e { YOURS(7) } com { TRAINER(557) }.
or-credits-brendan-10 = { TRAINER(235) }, ao lado de uma imagem de um salão de exposições.
or-credits-brendan-11 = Retrato de { TRAINER(567) } e seu { POKEMON(82) } e { YOURS(8) }.
or-credits-brendan-12 = Você e { TRAINER(1) } olham para a foto de uma montanha, com { YOURS(9) }.
or-credits-brendan-13 = { TRAINER(178) } levanta o punho.
or-credits-brendan-14 = Você envia { YOURS(10) } contra { TRAINER(235) } e seu { POKEMON(323) }.
or-credits-brendan-15 = Retrato de { TRAINER(569) } e seu { POKEMON(324) } e { YOURS(12) }.
or-credits-brendan-16 = Retrato de { TRAINER(570) } e seu { POKEMON(289) } e { YOURS(13) }.
or-credits-brendan-17 = { POKEMON(381) } e { POKEMON(380) } voam lado a lado.
or-credits-brendan-18 = Retrato de { TRAINER(571) } e seu { POKEMON(334) } e { YOURS(16) }.
or-credits-brendan-19 = Você e { YOURS(17) }, perto da foto de um longo corredor.
or-credits-brendan-20 = Retrato de { TRAINER(552) }, com { YOURS(18) } e { YOURS(19) }.
or-credits-brendan-21 = { POKEMON(383) }, pela imagem de uma caverna.
or-credits-brendan-22 = Você e { YOURS(21) } antes de { POKEMON(383) } em sua forma original.
or-credits-brendan-23 = Retrato de { TRAINER(572) } e seu { POKEMON(350) } e { YOURS(23) }.
or-credits-brendan-24 = { TRAINER(518) } e seu { TEXT(5, 816) }, e você com { YOURS(24) }.
or-credits-brendan-25 = Você e { YOURS(25) } enfrentam { TRAINER(557) } e seu { TEXT(5, 796) }.
or-credits-brendan-26 = Noite. Você passa de bicicleta por uma cidade adormecida.
or-credits-brendan-27 = As estrelas enchem o céu noturno.
or-credits-brendan-28 = Noite em { PLACE(208) }. Você está com { TRAINER(1) } entre as árvores sob as estrelas.
or-credits-brendan-29 = Um pequeno lago entre as folhas.
or-credits-brendan-30 = { TRAINER(1) } abre os olhos e sorri para você.
or-credits-brendan-31 = Ele levanta os punhos, pronto para a batalha.
or-credits-brendan-32 = { TRAINER(1) } joga os braços para o alto, coloca as mãos nos quadris e se vira para você.
or-credits-brendan-33 = Visto de cima, vocês dois caminham lado a lado por um caminho de terra durante a noite.
or-credits-brendan-34 = { TRAINER(1) } aponta para o céu.
or-credits-brendan-35 = Estrelas cadentes cruzam o céu noturno, uma após a outra.
or-credits-brendan-36 = { TEXT(123, 91) }
or-credits-brendan-37 = A luz da noite incide sobre sua mesa em casa: um caderno aberto, duas Pokébolas e um mapa de Hoenn.
or-credits-brendan-38 = A luz desaparece e a sala fica escura.
or-credits-brendan-39 = Um emblema escuro desaparece sobre a mesa e brilha com uma luz verde.

# Brendan's credits in Alpha Sapphire: his Omega Ruby credits above with Alpha Sapphire's five
# pictures, composed before any replay showed them (ADR-0038, Orion, 2026-10-16), then seen so in
# movie4_hall_of_fame_as_a_girl.
as-credits-brendan-1 = Você desce as etapas para { TRAINER(1) }. A luz entra pelas janelas altas até que tudo fique branco.
as-credits-brendan-2 = Luz solar através de árvores altas. Você anda de bicicleta por um caminho na floresta.
as-credits-brendan-3 = { TRAINER(1) } chega ao seu lado em sua própria bicicleta.
as-credits-brendan-4 = Agora, fotos da sua viagem vão e vêm sobre um mapa antigo de Hoenn. Uma sacola caiu no caminho com Pokébolas caindo; um { POKEMON(261) }; e você, com { YOURS(0) }.
as-credits-brendan-5 = { TRAINER(1) } com seu { MEMORY(2) }, e você com { YOURS(1) }, perto da foto de um pequeno lago.
as-credits-brendan-6 = { TRAINER(518) } se lança para lançar uma Pokébola em { POKEMON(280) }, com você ao lado dele.
as-credits-brendan-7 = Retrato de { TRAINER(561) } e seu { POKEMON(299) } e { YOURS(5) }.
as-credits-brendan-8 = Retrato de { TRAINER(563) } e seu { POKEMON(296) } e { YOURS(6) }.
as-credits-brendan-9 = Você e { YOURS(7) } com { TRAINER(557) }.
as-credits-brendan-10 = { TRAINER(178) }, pela foto de um salão de exposições.
as-credits-brendan-11 = Retrato de { TRAINER(567) } e seu { POKEMON(82) } e { YOURS(8) }.
as-credits-brendan-12 = Você e { TRAINER(1) } olham para a foto de uma montanha, com { YOURS(9) }.
as-credits-brendan-13 = { TRAINER(235) } fica de pé, com as mãos ao lado do corpo.
as-credits-brendan-14 = Você envia { YOURS(10) } contra { TRAINER(178) } e seu { POKEMON(319) }.
as-credits-brendan-15 = Retrato de { TRAINER(569) } e seu { POKEMON(324) } e { YOURS(12) }.
as-credits-brendan-16 = Retrato de { TRAINER(570) } e seu { POKEMON(289) } e { YOURS(13) }.
as-credits-brendan-17 = { POKEMON(381) } e { POKEMON(380) } voam lado a lado.
as-credits-brendan-18 = Retrato de { TRAINER(571) } e seu { POKEMON(334) } e { YOURS(16) }.
as-credits-brendan-19 = Você e { YOURS(17) }, perto da foto de um longo corredor.
as-credits-brendan-20 = Retrato de { TRAINER(552) }, com { YOURS(18) } e { YOURS(19) }.
as-credits-brendan-21 = { POKEMON(382) }, pela imagem de uma caverna.
as-credits-brendan-22 = Você e { YOURS(21) } antes de { POKEMON(382) } em sua forma original.
as-credits-brendan-23 = Retrato de { TRAINER(572) } e seu { POKEMON(350) } e { YOURS(23) }.
as-credits-brendan-24 = { TRAINER(518) } e seu { TEXT(5, 816) }, e você com { YOURS(24) }.
as-credits-brendan-25 = Você e { YOURS(25) } enfrentam { TRAINER(557) } e seu { TEXT(5, 796) }.
as-credits-brendan-26 = Noite. Você passa de bicicleta por uma cidade adormecida.
as-credits-brendan-27 = As estrelas enchem o céu noturno.
as-credits-brendan-28 = Noite em { PLACE(208) }. Você está com { TRAINER(1) } entre as árvores sob as estrelas.
as-credits-brendan-29 = Um pequeno lago entre as folhas.
as-credits-brendan-30 = { TRAINER(1) } abre os olhos e sorri para você.
as-credits-brendan-31 = Ele levanta os punhos, pronto para a batalha.
as-credits-brendan-32 = { TRAINER(1) } joga os braços para o alto, coloca as mãos nos quadris e se vira para você.
as-credits-brendan-33 = Visto de cima, vocês dois caminham lado a lado por um caminho de terra durante a noite.
as-credits-brendan-34 = { TRAINER(1) } aponta para o céu.
as-credits-brendan-35 = Estrelas cadentes cruzam o céu noturno, uma após a outra.
as-credits-brendan-36 = { TEXT(123, 91) }
as-credits-brendan-37 = A luz da noite incide sobre sua mesa em casa: um caderno aberto, duas Pokébolas e um mapa de Hoenn.
as-credits-brendan-38 = A luz desaparece e a sala fica escura.
as-credits-brendan-39 = Um emblema escuro desaparece sobre a mesa e brilha com uma luz verde.

# After the game restarts, Mossdeep City, zone 0x13: "XX:XX, the XXth day of the XXth month".
mossdeep-date-1 = Um céu noturno estrelado.
mossdeep-date-2 = Um edifício alto com janelas redondas iluminadas fica em um penhasco acima do mar escuro.

# "Mossdeep Space Center Celestial Observatory", then inside, zone 0x152; Cozmo is drawn as
# SORAISI_RS, sprite 0x136.
space-center-1 = Uma sala de controle: fileiras de mesas com monitores brilhantes sob uma grande tela. Os cientistas sentam-se diante deles, e { SPRITE(310) }, um velho de jaleco branco, está entre eles.

# "...Hmmn. So it's come to this.", then the Sky Pillar's top, zone 0xb5: at night in Omega Ruby,
# the disguise Magma's; by day in Alpha Sapphire (movie5_delta_episode_intro), the disguise Aqua's.
or-sky-pillar-1 = Noite, no topo de uma torre de pedra em ruínas. Alguém com uniforme com capuz vermelho está diante de degraus quebrados, com um { POKEMON(293) } a seus pés.

as-sky-pillar-1 = Dia, no topo de uma torre de pedra em ruínas. Alguém com uma bandana preta, um top listrado de azul e branco e meias listradas está diante de degraus quebrados, um { POKEMON(293) } a seus pés.

# "...Looks like I won't be needing THIS... Hiyyup!"
sky-pillar-hiyyup-1 = O uniforme sai voando. Abaixo dele está uma jovem com cabelo preto curto, uma capa longa e esfarrapada e botas verdes.

# "I swear I'll protect you. ...Aster."
or-aster-1 = A vista eleva-se muito acima da torre, cujo topo é um pequeno triângulo de pedra num mar de nuvens roxas.

as-aster-1 = A vista eleva-se muito acima da torre, cujo topo é um pequeno triângulo de pedra num mar de nuvens brancas.

# Zinnia's tale, a chapter a floor of the Sky Pillar (or_sky_pillar_1 to 5, spike 237): the painted
# wall each chapter shows, Omega Ruby's, and her leaving at each chapter's end.
or-tale-1 = A vista se volta para a parede, pintada de marrom escuro sobre pedra clara: uma fileira de pessoas minúsculas com os braços levantados no topo, uma estrela cadente entre duas nuvens, e grandes redemoinhos escuros e formas irregulares abaixo, pequenas figuras entre elas.
or-tale-2 = A tela escurece e a parede pintada volta: a fileira de pessoas minúsculas, agora de mãos dadas, e a estrela cadente abaixo delas.
or-tale-2b = A vista sobe pela parede. Acima das pessoas, uma grande faixa escura forma um arco sobre duas torres altas como escadas e uma marca semelhante a um olho em um anel.
or-tale-3 = A parede pintada novamente: as pessoas de mãos dadas sob a marca em forma de olho e suas duas torres.
or-tale-3b = Uma forma longa e escura, com espinhos nas costas, desce pela parede em direção a eles, e acima dela uma grande forma alada se espalha.
or-tale-4 = A parede pintada: um rosto grande com chifres enrolados e olhos grandes e redondos.
or-tale-4b = A vista se move para uma alta torre pintada com telhado pontiagudo.
or-tale-5 = A parede pintada: uma torre alta com telhado pontiagudo, pessoas minúsculas em pé no topo com os braços levantados, uma nuvem de um lado e um sol anelado do outro.
or-tale-5b = A vista flutua sobre fileiras de triângulos e círculos pintados, bem próximos.
tale-zinnia-gone-1 = { TRAINER(713) } desapareceu. Você está sozinho.
tale-zinnia-gone-2 = { TRAINER(713) } desapareceu. Você está sozinho.
tale-zinnia-gone-3 = { TRAINER(713) } desapareceu. Você está sozinho.
tale-zinnia-gone-4 = { TRAINER(713) } desapareceu. Você está sozinho.
tale-zinnia-gone-5 = { TRAINER(713) } desapareceu. Você está sozinho.

# Rayquaza summoned at the Sky Pillar's top (or_sky_pillar_6, spike 237). Zinnia is the game's own
# word for her, the trainer she is, `{ TRAINER(713) }`.
#
# "...The show is about to start.": what the climb up showed, before her first line.
summit-stars-1 = Você chega ao topo da torre, um triângulo de pedra entre as nuvens, sob um céu noturno estrelado. Longe, { TRAINER(713) } está de costas para você, seu { POKEMON(293) } ao lado dela, ambos olhando para as estrelas.
# "Aren't we happy now?"
summit-walk-up-1 = Ela se vira para sorrir para seu { POKEMON(293) }, e você caminha atrás deles.
# "That is my... It is our duty."
summit-turns-away-1 = Ela vira as costas para você, em direção ao altar, e a vista se eleva até as estrelas.
# "Just a little longer..."
summit-night-passes-1 = A vista sobe para o céu estrelado.
summit-night-passes-2 = Tudo fica preto.
# "Mur, mur!"
summit-wakes-1 = A luz volta. { TRAINER(713) } sorri para ela { POKEMON(293) }.
# "Please... Take this little one..."
summit-steps-forward-1 = Ela se afasta de você até o meio do altar, deixando seu { POKEMON(293) } ao seu lado.
# "The Key Stones that Zinnia holds are giving off a blinding light!"
summit-light-1 = Vista do alto, uma luz no meio do topo da torre aumenta até tudo ficar branco.
# "...Ugh...ngh...gh..."
summit-pain-1 = Ela se curva, com as mãos cruzadas no peito, fazendo uma careta.
# "Rayquazaaa!!!"
summit-rayquaza-comes-1 = A vista eleva-se acima da torre. Um flash de branco.
summit-rayquaza-comes-2 = A luz verde ondula no céu noturno, e uma longa serpente verde de Pokémon desce em direção à torre: { POKEMON(384) }.
# "Kiryararararahhh!!!"
summit-rayquaza-lands-1 = { POKEMON(384) } pousa no topo da torre antes de { TRAINER(713) } e você, suas grandes bobinas empilhadas ao redor dela.
# "This...can't...be..."
summit-kneels-1 = { TRAINER(713) } cai de joelhos na sua frente, com a cabeça baixa.
# "The glow of [name]'s Meteorite grew even stronger!"
summit-meteorite-1 = A luz branca preenche tudo.
# "Kiryarararaaahhh!!!"
summit-swallows-1 = { POKEMON(384) } ataca você em uma explosão de luz verde, com a mandíbula aberta.
# "Light is spilling forth from within Rayquaza's body!"
summit-glows-1 = { POKEMON(384) } fica reto e a luz aumenta até que tudo fique branco.
summit-glows-2 = A luz desaparece. { POKEMON(384) } está enrolado no topo do altar, olhando para você e { TRAINER(713) }.
# "It's you! You have to do this!"
summit-rayquaza-waits-1 = Vistos de trás de você, degraus levam ao altar, onde { POKEMON(384) } espera, enrolado.

# After Zinnia's battle (or_sky_pillar_7): "You and Rayquaza must protect nature...Pokémon...the
# people...", then the demo of the flight to space, Omega Ruby's.
or-to-space-1 = O dia chegou no topo da torre. Visto além de { TRAINER(713) }, você está ao lado de { POKEMON(384) } em um terno vermelho e um capacete redondo escuro.
or-to-space-2 = { TRAINER(713) } sorri, uma mão no peito, seu { POKEMON(293) } ao seu lado.
or-to-space-3 = Você sobe nas costas de { POKEMON(384) }.
or-to-space-4 = A torre fica sozinha contra o céu azul. Um flash verde e um feixe de luz verde disparam de seu topo.
or-to-space-5 = Muito abaixo, a torre é um pontinho no mar, iluminada em verde e dourado. No topo, { TRAINER(713) } e seu { POKEMON(293) } olham para cima.
or-to-space-6 = { POKEMON(384) } dispara para cima em um clarão de luz verde, anéis dourados em seu corpo agora, você de costas. Um raio de luz sobe pelo céu.
or-to-space-7 = Espaço: estrelas e nuvens roxas de luz flutuantes, a curva da Terra abaixo.
or-to-space-8 = { POKEMON(384) } corre pelo espaço, brilhando, com você de costas.
or-to-space-9 = À frente está pendurada uma enorme pedra vermelha escura.
or-to-space-10 = { POKEMON(384) } perfura-o em uma faixa verde. A rocha explode em grandes pedaços acima da terra.
or-to-space-11 = Entre as peças à deriva, um pequeno triângulo escuro flutua mais perto.
or-to-space-12 = É uma pirâmide vermelha. Fitas pretas se desenrolam dele.
or-to-space-13 = As fitas ficam laranja e azul, e a pirâmide se abre em um Pokémon: { POKEMON(386) }, laranja e azul, uma joia roxa em seu peito.
or-to-space-14 = Brilhos verdes percorrem o céu azul acima das nuvens.
or-to-space-15 = A vista desce até o topo da torre, isolada acima das nuvens.
or-to-space-16 = Você fica no topo da torre novamente, de frente para o altar. { TRAINER(713) } de { POKEMON(293) } caminha atrás de você.

# Alpha Sapphire's, the suit blue (as_sky_pillar_7, 2026-10-23).
as-to-space-1 = O dia chegou no topo da torre. Visto além de { TRAINER(713) }, você está ao lado de { POKEMON(384) } em um terno azul e um capacete redondo azul.
as-to-space-2 = { TRAINER(713) } sorri, uma mão no peito, seu { POKEMON(293) } ao seu lado.
as-to-space-3 = Você sobe nas costas de { POKEMON(384) }.
as-to-space-4 = A torre fica sozinha contra o céu azul. Um flash verde e um feixe de luz verde disparam de seu topo.
as-to-space-5 = Muito abaixo, a torre é um pontinho no mar, iluminada em verde e dourado. No topo, { TRAINER(713) } e seu { POKEMON(293) } olham para cima.
as-to-space-6 = { POKEMON(384) } dispara para cima em um clarão de luz verde, anéis dourados em seu corpo agora, você de costas. Um raio de luz sobe pelo céu.
as-to-space-7 = Espaço: estrelas e nuvens roxas de luz flutuantes, a curva da Terra abaixo.
as-to-space-8 = { POKEMON(384) } corre pelo espaço, brilhando, com você de costas.
as-to-space-9 = À frente está pendurada uma enorme pedra vermelha escura.
as-to-space-10 = { POKEMON(384) } perfura-o em uma faixa verde. A rocha explode em grandes pedaços acima da terra.
as-to-space-11 = Entre as peças à deriva, um pequeno triângulo escuro flutua mais perto.
as-to-space-12 = É uma pirâmide vermelha. Fitas pretas se desenrolam dele.
as-to-space-13 = As fitas ficam laranja e azul, e a pirâmide se abre em um Pokémon: { POKEMON(386) }, laranja e azul, uma joia roxa em seu peito.
as-to-space-14 = Brilhos verdes percorrem o céu azul acima das nuvens.
as-to-space-15 = A vista desce até o topo da torre, isolada acima das nuvens.
as-to-space-16 = Você fica no topo da torre novamente, de frente para o altar. { TRAINER(713) } de { POKEMON(293) } caminha atrás de você.

# The last visits round Hoenn (or_ and as_sky_pillar_7 from their states after Zinnia's battle,
# 2026-10-23), the Space Center's control room in the words of the Delta Episode's start, and May
# for a boy, Brendan for a girl. Professor Birch is { SPRITE(179) }, and his wife, whom the game
# calls only "Woman", the professor's wife.
# "Mur murrr!"
letter-1 = Ele foge e você fica sozinho.
letter-2 = Visto do alto, o topo da torre fica entre as nuvens, você pequeno sobre ele.
# "Until we meet again, Zinnia"
letter-3 = A vista sobe para o céu azul.
epilogue-space-center-1 = Uma sala de controle: fileiras de mesas com monitores brilhantes sob uma grande tela. Os cientistas sentam-se diante deles, e { SPRITE(310) }, um velho de jaleco branco, está entre eles.
# "Thank you, [name]."
epilogue-wally-1 = Uma sala de estar junto a uma cozinha. { TRAINER(518) } caminha em direção à porta, e sua mãe e seu pai estão atrás dele.
# "I can't wait to go and check it out!"
epilogue-wally-2 = { TRAINER(518) } sai pela porta.
# "...that I can't help feeling happy, too."
epilogue-magma-1 = Uma sala escura enfeitada em vermelho. { TRAINER(235) } está sentado na cabeceira de uma longa mesa sob uma bandeira vermelha, com a cabeça baixa, e { TRAINER(691) } está ao lado de um modelo de veleiro. { TRAINER(694) } está no final da mesa, de frente para { TRAINER(235) }.
# "And that settles that!"
epilogue-league-1 = Um salão alto de pedra, com luz entrando através de altas janelas em arco. { TRAINER(557) } fica ao pé dos degraus de uma porta escura e { TRAINER(572) } caminha até ele.
# Alpha Sapphire's, after "...that I can't help feeling happy, too."
epilogue-aqua-1 = Uma sala escura decorada em azul. { TRAINER(178) } está sentado na cabeceira de uma longa mesa sob uma bandeira azul, com a cabeça baixa, e { TRAINER(688) } está ao lado de um modelo de veleiro. { TRAINER(683) } está no final da mesa, de frente para { TRAINER(178) }.
# "Well... I guess it could be worse."
epilogue-aqua-league-1 = Um salão alto de pedra, com luz entrando através de altas janelas em arco. { TRAINER(557) } fica ao pé dos degraus de uma porta escura e { TRAINER(572) } caminha até ele.
# "Thank you, my friend..."
epilogue-league-2 = { TRAINER(572) } vai embora e { TRAINER(557) } se vira para vê-lo partir.
epilogue-falls-1 = Uma grande cachoeira desce até uma caverna. { TRAINER(713) } está na beira de uma saliência arenosa, de costas para você, olhando para ela, seu { POKEMON(293) } ao lado dela.
# "It's all over..."
epilogue-falls-2 = Uma velha com uma bandana branca se aproxima e { TRAINER(713) } se vira para ela.
# "... Yes."
epilogue-falls-3 = { TRAINER(713) } desce as escadas, deixando-a { POKEMON(293) } ao lado da velha.
epilogue-parents-1 = Uma sala ao lado da cozinha, luz noturna na janela. { SPRITE(179) } fica ao lado da TV e { TRAINER(570) } perto da cozinha, enquanto sua mãe e a esposa do professor sentam-se em uma mesa rosa.
# "How about our little boy's favorite?"
epilogue-parents-2 = { TRAINER(570) } e sua mãe vão para a cozinha cozinhar, e { SPRITE(179) } e sua esposa saem.
epilogue-litleonids-1 = Um salão com um modelo de foguete em um anel isolado no meio. Você está diante disso.
epilogue-litleonids-2 = { TRAINER(4) } chega atrás de você e vocês dois olham ao redor do corredor.
epilogue-litleonids-3 = No balcão de uma loja, { TRAINER(4) } observa as prateleiras de lembranças.
epilogue-litleonids-4 = { TRAINER(4) } coloca seu rosto em um recorte de um astronauta segurando uma bandeira laranja, a terra atrás, e sorri para você.
epilogue-litleonids-5 = Vocês dois sentam-se lado a lado em um banco diante de uma grande janela, com vista para o mar e a cidade.
# A girl's: "How about our little girl's favorite?", and Brendan.
brendan-parents-2 = { TRAINER(570) } e sua mãe vão para a cozinha cozinhar, e { SPRITE(179) } e sua esposa saem.
brendan-litleonids-1 = Um salão com um modelo de foguete em um anel isolado no meio. Você está diante disso.
brendan-litleonids-2 = { TRAINER(1) } chega atrás de você e vocês dois olham ao redor do corredor.
brendan-litleonids-3 = No balcão de uma loja, { TRAINER(1) } examina as prateleiras de souvenirs.
brendan-litleonids-4 = { TRAINER(1) } coloca o rosto em um painel recortado de um astronauta segurando uma bandeira laranja, com a Terra ao fundo, e sorri.
brendan-litleonids-5 = Vocês dois sentam-se lado a lado em um banco diante de uma grande janela, com vista para o mar e a cidade.
# "Time really flies when you're having fun..."
epilogue-home-1 = Noite. Luzes brilham nas janelas das casas dos dois lados do caminho, e você e { TRAINER(4) } caminham juntos por ele.
brendan-home-1 = Noite. Luzes brilham nas janelas das casas dos dois lados do caminho, e você e { TRAINER(1) } caminham juntos por ele.
# "See you tomorrow!"
epilogue-home-2 = Vocês dois se separam e você caminha até sua própria porta.
# "You're back at last! Welcome home, [name]!"
epilogue-home-3 = Você entra.
epilogue-home-4 = Você fica ao lado da cama no seu quarto.
# Arriving in Shoal Cave's entrance room at high tide.
shoal-cave-high-tide = A maré está alta. A areia corre para norte, desde a entrada da gruta, entre paredes de rocha castanha, até à água do mar que fluiu para dentro da gruta.

# Arriving in Shoal Cave's entrance room at low tide.
shoal-cave-low-tide = A maré está baixa. A areia molhada corre para norte a partir da entrada da caverna, contornando uma poça rasa que o mar deixou, até saliências baixas de rocha castanha com degraus de madeira e uma passagem escura mais além.
