### What X and Y show without words, in pa3ds's own words, a beat at a time, said under [pa3ds.ui]
### descriptions. When each is said is `../scenes.toml`'s, every language's; a beat is found here
### by its id there (ADR-0040). A beat this language has no words for is said in English's.
###
### A name the game shows is never written here: `{ POKEMON(716) }` is the game's own word out of
### the player's ROM, as are `{ ITEM(4) }` and the game's own name, `{ GAME() }`. Words drawn as a
### picture no line holds are the picture's words in `pictures.ftl`, `{ PICTURE("push_start") }`.
###
### Each scene was described from pictures of a replay; the comments say which, and what the
### game shows around it.

# X's title screen, from boot (x_intro_and_title_screen). The prompt is a picture with no line in
# the game's text, said as `pictures.ftl` calls it; the logo is the game's name, as its ROM calls
# it. The bottom screen plays a reel of its own under the logo, Xerneas and then the journey.
x-title-light = A tela de título. No escuro, uma pequena luz azul brilha e cresce, se transforma em uma cruz brilhante de luz e explode em faíscas.
x-title-emblem = Uma grande letra X fica escura contra um feixe de luz branca e verde, seus braços ramificando-se como chifres e tornando-se azul profundo.
x-title-logo = A vista corre para a luz. O logotipo Pokémon brilha sobre a luz azul-petróleo rodopiante e o X azul aparece ao lado dele.
x-title = O logotipo diz { GAME() }. Na parte inferior, piscando: { PICTURE("push_start") }.
x-title-xerneas = Na tela inferior, { POKEMON(716) } está em uma floresta enevoada sob raios de sol: um cervo alto, azul e preto, com pernas longas e finas, seus grandes chifres brilhando em todas as cores do arco-íris. A visão circula lentamente em torno de sua cabeça, suas pernas e chifres bem abertos.
x-title-riding = Depois cenas de uma viagem. Uma garota com um chapéu vermelho anda de { POKEMON(672) } ao longo de um caminho de terra cercado e por saliências gramadas, e um menino com um boné vermelho anda de { POKEMON(111) } sobre terreno rochoso entre pedras.
x-title-city = Um arco branco com um emblema dourado e uma alta torre branca de treliça de aço que se estende até o céu. Uma garota de boina vermelha caminha por uma ampla avenida arborizada e altos prédios de pedra em direção à torre. Depois, uma longa avenida arborizada leva a um palácio.
x-title-amie = { POKEMON(255) }, { POKEMON(25) } e { POKEMON(700) } sorriem em um céu brilhante. Em um gramado ensolarado, { POKEMON(700) } dança, um { POKEMON(667) } mastiga uma guloseima rosa, uma mão acaricia um { POKEMON(694) } e corações flutuam.
x-title-skates = O menino, de patins, serpenteia entre cones laranja em uma quadra, desliza ao longo de um corrimão por entre as árvores e desliza por um caminho à beira da água.
x-title-hats = A menina gira em um anel de notas musicais douradas e depois mostra chapéus: um branco com óculos escuros na aba, um boné branco com uma flor amarela, um boné rosa e azul.
x-title-flight = O menino está na areia sob raios de luz azul enquanto penas brancas passam. Uma forma alada escura desce e ele sobe para o céu. Um flash de branco.
x-title-credit = { POKEMON(716) } está novamente na floresta enevoada e as palavras desaparecem abaixo dela: { PICTURE("presented_by") } { PICTURE("gf_logo") }.

# Its film, when the title screen is left alone for a minute: quick cuts of battles.
x-movie-ball = Um { ITEM(4) } forma um arco em um céu azul e { POKEMON(650) } irrompe em um campo gramado.
x-movie-starters = { POKEMON(653) }, { POKEMON(650) } e { POKEMON(656) } aparecem lado a lado.
x-movie-fire = { POKEMON(650) } ataca com um turbilhão de folhas verdes em { POKEMON(653) }, que responde com uma explosão de fogo que preenche a tela.
x-movie-water = { POKEMON(656) } lança uma bola de água que cai sobre { POKEMON(653) }.
x-movie-flashes = Flashes rápidos: { POKEMON(697) } rugindo quando uma explosão o atinge, { POKEMON(668) } em uma inundação de luz branca, um { POKEMON(694) } brilhando em um bando de pássaros brancos.
x-movie-charizard = No escuro, um { POKEMON(6) } preto com chamas azuis na boca abre suas asas em um palco rochoso sob um holofote. Um { POKEMON(150) } pálido e de ombros largos reúne uma esfera azul brilhante e { POKEMON(6) } atinge-a através da luz amarela. Tudo fica escuro.

# Y's title screen (y_intro_and_title_screen): X's to the frame, with Yveltal for Xerneas and a
# red Y for the blue X; the journey between them is the same.
y-title-light = A tela de título. No escuro, uma pequena luz vermelha brilha e depois explode em um flash de luz rosa e vermelha.
y-title-emblem = Uma grande letra Y fica escura contra um feixe de luz branca, seus braços abertos como asas irregulares e fica vermelho.
y-title-logo = A vista corre para a luz. O logotipo Pokémon brilha sobre as nuvens rodopiantes e o Y vermelho surge ao lado dele.
y-title = { x-title }
y-title-yveltal = Na tela inferior, { POKEMON(717) } desliza entre nuvens em um céu roxo: um enorme pássaro vermelho e preto com asas largas e irregulares e cauda longa. A visão circunda suas asas, suas garras e sua cabeça afiada com um olho azul.
y-title-riding = { x-title-riding }
y-title-city = { x-title-city }
y-title-amie = { x-title-amie }
y-title-skates = { x-title-skates }
y-title-hats = { x-title-hats }
y-title-flight = { x-title-flight }
y-title-credit = { POKEMON(717) } gira entre as nuvens novamente, e as palavras desaparecem abaixo dele: { PICTURE("presented_by") } { PICTURE("gf_logo") }.

# Its film: X's, with Charizard and Mewtwo in other forms.
y-movie-ball = { x-movie-ball }
y-movie-starters = { x-movie-starters }
y-movie-fire = { x-movie-fire }
y-movie-water = { x-movie-water }
y-movie-flashes = { x-movie-flashes }
y-movie-charizard = No escuro, um { POKEMON(6) } laranja com chifres e asas largas está em um palco rochoso sob um holofote. Um { POKEMON(150) } esguio e pálido com uma cauda longa reúne uma esfera azul brilhante e { POKEMON(6) } atinge-a através da luz amarela. Tudo fica escuro.

# Prism Tower's quiz rooms, from the rig's own run through the Gym (spike 252): the picture each
# question shows for a few seconds before it is asked, never named, and the booths at the back.
prism-quiz-1 = Antes da pergunta, uma forma preta em uma moldura branca: uma criatura pequena em duas pernas, com duas orelhas longas e pontudas e uma cauda dobrada em zigue-zague como um raio.
prism-quiz-2 = Antes da pergunta, uma forma preta em uma moldura branca: um pequeno pássaro redondo sobre duas pernas finas, com bico curto e pontudo e uma cauda de penas longas aberta em leque atrás dele.
prism-quiz-3 = Antes da pergunta, uma forma preta em uma moldura branca: um pequeno macaco em duas pernas, com grandes orelhas redondas e uma cauda fina curvada na ponta. Um tufo liso sobe da cabeça e se curva para trás em uma longa onda arredondada.
prism-quiz-4 = Antes da pergunta, três borboletas pretas, uma após a outra. A primeira: corpo roliço, cabeça redonda com duas antenas longas curvadas nas pontas e asas arredondadas. A segunda, vista de frente: asas largas com bordas onduladas e recortadas, com as pontas do par superior afiadas. A terceira: uma mariposa com as asas abertas, largas e planas, entalhadas nas pontas, dois chifres curtos e uma franja sob o corpo.
prism-booths-1 = Ao fundo há três cabines, cada uma com um treinador, e números grandes acima delas: 1 à esquerda, 2 no centro e 3 à direita.
prism-booths-2 = { prism-booths-1 }
prism-booths-3 = { prism-booths-1 }
prism-booths-4 = { prism-booths-1 }
