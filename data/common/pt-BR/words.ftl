### What pa3ds says in words of its own, every game alike: the ways and distances on a map, the
### navigation keys' answers, P's guidance, touch mode, and what pa3ds says of itself (ADR-0040).
### A game's own `words.ftl` is read before this one. The game's own words, a place's name, a
### Pokémon's, never come from here: they are read from the player's ROM.
###
### A message's values are inserted as `{ $name }`: a number, or words pa3ds or the game has
### already put together, "Route 101". Each message says what it is said of, and the code says
### where (`own("steps")`).

## Ways and distances

# The ways of the compass, as a leg of a way, a direction key's answer and a bearing say them.
# North is up on the pad (spike 119).
way-north = norte
way-east = leste
way-south = sul
way-west = oeste
way-north-east = nordeste
way-south-east = sudeste
way-south-west = sudoeste
way-north-west = noroeste
# The ways of the player's own frame, soaring and on Lumiose's streets: "9 ahead 8 left".
way-ahead = à frente
way-behind = atrás
way-left = esquerda
way-right = direita
# So many steps one way: a leg of a way, "5 east", and half of a bearing, "3 east 2 north".
steps = { $count } { $way }
# A leg walked with a move, "Surf 3 south": the move's name is the game's.
leg-with = { $how } { $steps } { $way }
# Where something is when it is where the player stands.
here = aqui
# How far a thing stands above or below the player, in the game's units, 18 to a cell, where
# it is more than two cells' height: "ladder, 2 west, 54 above".
above = { $count } acima
below = { $count } abaixo
# A leg on Lumiose's rails, where up walks the way the camera looks and left and right turn the
# player (spike 105): "hold right, 5 forward, 3 back".
rail-forward = { $steps } para a frente
rail-back = { $steps } para trás
rail-hold-right = segure à direita
rail-hold-left = segure à esquerda
# A camera a tap left pointing between two ways, which a hold either way settles.
rail-angled = câmera em ângulo, segure para a esquerda, segure para a direita
# A leg in Glittering Cave, where the camera is behind the player and left, right and
# down turn them with it (spike 239): "turn left, 5 forward".
turn-right = vire à direita
turn-left = vire à esquerda
turn-around = vire-se

## The list of what is on the map, and the navigation keys' answers

# The categories the list is walked by (`things.CATEGORIES`), and soaring's (ORAS's
# `words.ftl`): "exits, 3".
category-all = tudo
category-exits = saídas
category-signs = interagíveis
category-obstacles = obstáculos
category-people = pessoas
category-items = itens
category-berries = bagas
category-ash = cinza
category-bike = bicicleta
category-riding = montaria
category-climbing = escalada
category-currents = correntes
category-pokemon = Pokémon
# A category chosen, with how many things are in it.
key-category = { $category }, { $count }
# Where a thing is in its category, "1 of 3", and in a list of ways a boulder can be pushed.
position = { $number } de { $count }
# Nothing at all on the map's list that way, or nothing of the category chosen: "no exits north".
key-nothing-way = nada { $way }
key-none-way = sem { $category } { $way }
key-nothing-here = nada aqui
key-no-map = sem mapa
key-cant-walk-now = não é possível andar agora
# The ways a key steps through the list: list mode, direction mode and touch mode, which names
# the first thing to touch as it comes on.
key-list-mode = modo de lista
key-direction-mode = modo de direção
key-touch-mode = modo de toque, { $first }
# Ctrl and P, leaving out what Shift and P finds no way to, and putting it back.
key-unreachable-hidden = itens inacessíveis ocultos
key-unreachable-shown = itens inacessíveis mostrados
# Shift and P with no way being followed, and with one, which it drops.
key-path-off = rota desativada
key-no-path = sem caminho
# The battle keys out of a battle.
key-no-battle = sem batalha
# Comma and period on something with only one way to it.
key-one-way = um caminho para isso
# An exit's arrival somewhere else on the map: "teleporter, 3 east, out at 10 north".
out-at = saída em { $bearing }
# What an exit is called, then where it leads, when that is another place, the game's own word
# for it: "edge to Route 101".
leads-to = { $thing } para { $place }
# A thing's coordinates spanning several cells: "x 80 to 119".
span = { $low } até { $high }
# One of two or more: "north, east or west".
either = { $these } ou { $last }

## Big boulders

# The way a big boulder is pushed, chosen with comma and period: "push east, 2 of 2".
key-push = empurrar { $way }
boulder-into-hole = { $way } até um buraco
# What a big boulder can do, after where it is (Orion, 2026-10-11): "moveable north or east".
boulder-moveable = pode ser movida para { $ways }
boulder-immoveable = imóvel
# How far a big boulder has moved from where its room starts it, under `track_boulders`.
boulder-moved = moveu-se { $bearing }

## Renaming

rename-title = Renomear
# The box's words, for a thing: "Sign, 1 north. Called: Signpost 1".
rename-label = { $thing }, { $bearing }. { $name }
# And for whoever last spoke.
rename-speaker-label = A última pessoa que falou. { $name }
rename-called = Nome: { $name }
rename-not-named = Ainda sem nome
# The box's checkbox, for somebody or something that can be named alone, apart from everyone
# drawn as it (Orion, 2026-10-11).
rename-only-this-one = Somente este
# Said back once the box closes: what was renamed, as it was called when the box opened.
rename-now = { $thing } agora se chama { $name }
rename-no-name = { $thing } não tem nome
rename-the-last-speaker = a última pessoa que falou
rename-nothing = nada para renomear
rename-cannot = { $thing } não pode ser renomeado
rename-already = renomeação já aberta
rename-nobody-spoke = ninguém falou ainda

## P: the way to the thing chosen, and guiding the player along it

# A leg a move or a bike takes the player along, where the game's own name for it cannot be read:
# where it goes (`guide.OWN_MOVE_WORDS`). "Surf" and the bikes' names are the game's.
how-water = água
how-bush = arbusto
how-boulder = pedra grande
how-rock = rocha
how-waterfall = cachoeira
how-slope = declive
how-bumpy-slope = ladeira irregular
# A step into one of Fortree Gym's rotating doors, which turns it: "push 1 west" (spike 161).
how-push = empurre
# A leg a current carries the player along (spike 208), and a run of them, by where it leaves
# them: "Surf 3 south, ride the current 28 west 4 north" (Orion, 2026-10-13).
how-ride-the-current = siga pela corrente
p-current = siga pela corrente { $bearing }
# The step off a ridden Pokémon, from flat ground onto plain ground, and the rest of its leg: "get
# off 3 north" (spike 238). Getting on is said with the Pokémon's own name: "Rhyhorn 1 east".
how-get-off = desça
# The step into a rock the ridden Pokémon smashes with A: "smash 1 east" (spike 238).
how-smash = quebre
# A leg along a climbing wall's face, and the handholds climbed in place before or after one:
# "climb 3 up, 6 east", "climb 6 west, climb 3 down, 1 west" (spike 245).
how-climb = escale
p-climb-up = suba { $holds } apoios
p-climb-down = desça { $holds } apoios
# A leg down a climbing wall's slide, which carries the player once they step onto its top:
# "slide 5 south" (spike 245).
how-slide = deslizar
# A leg along a grind rail, skated onto from the end cell before it, and a run of them, by
# where it lands the player: "3 east, grind the rail 17 east" (spike 247).
how-grind-the-rail = deslize pelo trilho
p-grind = deslize pelo trilho { $bearing }
# The step that grabs a vine and its ride, as one leg, said with the way faced to grab it and
# what the ride does: "3 north, face north, climb the vine up, 2 east" (spike 250).
how-climb-the-vine-up = suba pela trepadeira
how-climb-the-vine-down = desça pela trepadeira
how-swing-on-the-vine = balance-se na trepadeira
p-vine = vire para { $way }, { $how }
# How many more vines a way rides past its first, said after it in place of the rest: "face
# north, climb the vine up, then 16 more vines" (spike 250).
p-more-vines = { $count ->
    [one] depois, mais 1 trepadeira
   *[other] depois, mais { $count } trepadeiras
}
# A run of steps up or down a ramp, said as one leg by the way it starts, the key held all along
# it: "up the ramp 25 south-east" (spike 248).
how-up-the-ramp = suba a rampa
how-down-the-ramp = desça a rampa
# The way chosen to push a big boulder, before the way to its side: "to push east: 3 west".
p-to-push = para empurrar { $way }:
# A step to line up with the next leg, where a wall's corner would catch the player: "a little
# west, then 5 north" (spike 202), and once they have.
p-nudge = um pouco para { $way }
p-nudge-then = { $nudge }, depois
p-lined-up = alinhado
# A way that goes as near as it can: "5 east, then can't continue, Team Magma Grunt in the way".
p-then-cant-continue = então não é possível continuar
p-cant-continue = não é possível continuar
p-in-the-way = { $thing } no caminho
# Under the sea, where the way goes on from another dive spot (spike 165).
p-surface = suba à superfície e vá ao ponto de mergulho { $bearing }
# A leg that crosses into another place, said with it: "10 east into Route 117" (Orion,
# 2026-10-24).
p-into = { $leg } entrando em { $place }
# A way over a room of thin ice, which cracks every tile on the way (spike 167).
p-and-ice-cleared = e todo o gelo foi quebrado
p-ice-cleared = todo o gelo foi quebrado
p-no-path-to = nenhum caminho para { $thing }
# What P says where it finds no way at all: in a game whose ground is not read yet, and off the
# streets of Lumiose the game has loaded.
p-no-route-in-game = ainda não há rota neste jogo
p-no-route-on-streets = não há rota nestas ruas
# What the way led to has gone: a person walked off, a ball was picked up.
p-gone = { $thing } desapareceu

## Touch mode

touch-nothing = nada para tocar
# A tap asked for where pa3ds cannot tap, and one Azahar is not listening for.
touch-off = modo de toque desativado
touch-not-listening = Azahar não está aceitando toques do pa3ds

## What the field shows without words

# The Dowsing Machine's lights, said as they change: how near a hidden item is, never where
# (spike 145).
dowsing = Detector de Itens: { $light }
dowsing-blue = azul
dowsing-green = verde
dowsing-yellow = amarelo
dowsing-red = vermelho
dowsing-nothing = nada
# Whether the decoration held over a Secret Base's cell fits there, before the cell's words
# (spike 154).
decorating-placeable = pode ser colocado
decorating-not-placeable = não pode ser colocado

## The Pokédex's habitat map (spike 152)

# A place drawn as the Pokémon's, then how it is found there, by the icons the top screen draws
# in colour (Orion's words, 2026-10-01). The DexNav's icons say how they are met in the same
# words: "Marill water" (2026-10-20).
habitat-found = Encontrado em { $place }
habitat-grass = grama
habitat-water = água
habitat-fishing = pesca

## Super Training and Pokémon-Amie (spikes 198 and 200)

# A goal of a regimen's, which a tap shoots at: the game's dialogue speaks of "the white goals"
# and "red goals" and names none.
training-goal = alvo
training-red-goal = alvo vermelho
# A ball of the Balloon Bot's in flight, said interrupting: the game says nothing.
training-shot-coming = disparo vindo
# The clock, every half minute and at ten seconds: "1:30 left".
training-left = { $time } restante
# Touch mode's one gesture that is not a tap, a slide back and forth over the Pokémon, the
# manual's word: "stroke Sceptile".
amie-stroke = acaricie { $pokemon }

## What pa3ds says of itself

app-waiting = Aguardando o jogo
app-ready = pa3ds pronto
# Said where "pa3ds ready" would be, by a pa3ds started over one that was running (ADR-0039).
app-restarted = pa3ds reiniciado
# A reader that failed, logged and snapshotted (ADR-0019).
app-reader-error = Erro de leitura
# The rename box's buttons.
dialog-ok = OK
dialog-cancel = Cancelar
# pa3ds's own window, opened when pa3ds is double-clicked: its title, and its menus and their
# items. The letter after & is the one that opens it, underlined, with Alt on a menu.
window-title = Pokémon Access 3DS
window-menu-mod = &Mod
# The Rust port's alone, which updates itself; the Python has no updater.
window-updates = Verificar &atualizações
window-exit = &Sair
window-menu-azahar = &Azahar
window-azahar-portable = Baixar Azahar &portátil
window-azahar-installer = Baixar Azahar pelo &instalador
window-menu-help = &Ajuda
# Each opens a page of the manual in the browser.
window-documentation = &Documentação...
window-keys = &Comandos de teclas...
window-known-issues = &Problemas conhecidos...
window-translating = &Tradução...
window-changelog = &Registro de alterações...
window-settings-guide = &Guia de configurações...
window-issues = &Informar um problema no GitHub

## The settings: each one's name in the settings dialog, and what it does, said after the
## name there and above the setting in config.toml
setting-pa3ds-keys = Responder às teclas do mod
setting-pa3ds-keys-about = Permitir as teclas do pa3ds e associar os botões do Azahar conforme definido em keys.toml.
setting-pa3ds-poll_rate = Leituras por segundo
setting-pa3ds-poll_rate-about = Quantas vezes o jogo é lido por segundo: 60, a taxa de quadros do próprio jogo; 30 para uma máquina mais lenta.
setting-pa3ds-azahar_folder = Pasta do Azahar portátil
setting-pa3ds-azahar_folder-about = A pasta que contém um Azahar portátil, com azahar.exe e uma pasta user; deixe em branco para um Azahar instalado normalmente, cujas configurações ficam em AppData. O pa3ds preenche este campo quando você escolhe a pasta solicitada.
setting-pa3ds-hide_window = Ocultar a janela do mod enquanto Azahar estiver aberto
setting-pa3ds-hide_window-about = Ocultar a janela aberta pelo pa3ds enquanto o Azahar estiver aberto, evitando que ela apareça no Alt+Tab entre você e o jogo; false mantém a janela visível para que o OBS grave os sons do pa3ds por meio dela.
setting-pa3ds-ui-pauses = Pausas no texto da tela
setting-pa3ds-ui-pauses-about = Coloque pausas entre partes do texto da tela ditas juntas: dois pontos após um rótulo, como em "OT: Raiden", e uma vírgula entre o resto.
setting-pa3ds-ui-interrupt = Menus interrompem a fala
setting-pa3ds-ui-interrupt-about = Interromper a fala quando o cursor de um menu se mover, em vez de colocá-la na fila; as teclas do próprio pa3ds sempre interrompem.
setting-pa3ds-ui-unfocused = Fale enquanto Azahar estiver em segundo plano
setting-pa3ds-ui-unfocused-about = Continuar falando o que o jogo mostra enquanto outra janela estiver em primeiro plano; caso contrário, o pa3ds fica em silêncio, exceto para suas próprias teclas e mensagens.
setting-pa3ds-ui-descriptions = Descrições de cenas
setting-pa3ds-ui-descriptions-about = Descrever as cenas que o jogo mostra sem palavras, como Groudon despertando. Cada descrição é falada depois da linha de texto que antecede a cena. Apenas algumas cenas foram descritas até agora.
setting-pa3ds-ui-language = Idioma
setting-pa3ds-ui-language-about = O idioma usado nas palavras do próprio pa3ds, identificado pelo nome da pasta de dados: en, fr, de, es, it, ja, ko, pt-BR, zh-CN ou zh-TW. Em branco acompanha o idioma do jogo lido no arquivo salvo. Defina pt-BR ao usar uma tradução de fã sobre outro idioma e em Sun e Moon, cujo idioma do arquivo salvo o pa3ds não consegue ler.
setting-pa3ds-audio-tones = Volume relativo para tons
setting-pa3ds-audio-tones-about = Volume do tom curto emitido quando o cursor do mapa ou o modo de toque se move. O tom fica mais agudo quanto mais acima e mais à direita quanto maior a posição horizontal: 1 tem o volume do jogo, 0.5 tem metade, 1.5 é mais alto e 0 silencia; na janela de configurações, 100 por cento corresponde ao volume do jogo.
setting-pa3ds-audio-footsteps = Volume relativo para passos
setting-pa3ds-audio-footsteps-about = Volume dos passos do próprio pa3ds em terrenos onde o jogo não os reproduz, como ruas, pisos e cavernas: 1 tem o volume do jogo, 0.5 tem metade, 1.5 é mais alto e 0 silencia; na janela de configurações, 100 por cento corresponde ao volume do jogo.
setting-pa3ds-navigation-positions = Anunciar posição na lista
setting-pa3ds-navigation-positions-about = Diga onde uma coisa está na lista, como em “porta, 1 de 3”.
setting-pa3ds-navigation-position_first = Listar posição antes das direções
setting-pa3ds-navigation-position_first-about = Diga o local da lista antes do caminho até ele, “porta, 1 de 3, 1 norte”; false diz "porta, 1 norte, 1 de 3".
setting-pa3ds-navigation-turn_by_turn = Direções passo a passo
setting-pa3ds-navigation-turn_by_turn-about = Assim que P tiver encontrado o caminho, diga cada curva à medida que for alcançada; false diz o caminho uma vez, e P diz novamente de onde você está.
setting-pa3ds-navigation-guide_to_hidden = Guia para descobertas de DexNav
setting-pa3ds-navigation-guide_to_hidden-about = Quando uma busca do DexNav encontrar o Pokémon procurado, ou quando o DexNav encontrar um por conta própria enquanto estiver aberto e você estiver no modo de toque, traçar o caminho até ele como P e manter P seguindo esse caminho, independentemente da posição da lista; um Pokémon encontrado com o AreaNav aberto não altera a rota.
setting-pa3ds-navigation-track_boulders = Acompanhar o movimento das pedras grandes
setting-pa3ds-navigation-track_boulders-about = Informar quanto uma pedra grande se moveu desde a posição inicial da sala, como em "pedra grande, 2 leste, moveu-se 1 norte", para que você saiba qual delas empurrou; o jogo mostra a posição atual, mas não a posição inicial de cada pedra.
setting-pa3ds-navigation-hide_door_signs = Ocultar placas de porta
setting-pa3ds-navigation-hide_door_signs-about = Deixe uma placa fora da lista quando tudo o que ela diz é o nome de uma porta ao lado dela, como "Apt. 13 Wattson's Home" de Mauville Hills; uma placa que diz mais, como faz uma academia, fica.
setting-pa3ds-navigation-show_y = Anunciar altura
setting-pa3ds-navigation-show_y-about = Informar também a altura, como y depois de x e z, quando C disser onde você está e Shift+C disser onde está o item selecionado: "x 71, z 35, y 20"; nas unidades do jogo, 18 correspondem à largura de uma célula. A altura é omitida quando o pa3ds não a conhece, como em Sun e Moon.
setting-pa3ds-navigation-sort_by_distance = Classificar por distância
setting-pa3ds-navigation-sort_by_distance-about = No modo de lista, ordenar primeiro o que estiver mais perto, mudando a ordem enquanto você anda; false mantém cada item na mesma posição, seguindo a ordem das tabelas do jogo, para ajudar a reconhecer quais casas já foram visitadas. O modo de direção sempre ordena pelo mais próximo.
setting-pa3ds-extras-conditions = Condições do concurso como números
setting-pa3ds-extras-conditions-about = Informar numericamente as condições de Concurso de um Pokémon no resumo e no Estojo de Pokéblocos, onde o jogo as representa por uma estrela. Os números são mais precisos do que o desenho que o pa3ds não consegue ler.
setting-pa3ds-extras-hidden_items = Itens ocultos
setting-pa3ds-extras-hidden_items-about = Listar como "item oculto" os itens que ainda não foram encontrados. "underwater" lista somente os que estão debaixo d'água, onde o Detector de Itens não pode ser usado; "all" lista todos os itens ocultos; "off" não lista nenhum.
# What each of the hidden items setting's words is called in the settings dialog.
setting-pa3ds-extras-hidden_items-off = Desligado
setting-pa3ds-extras-hidden_items-underwater = Sob o mar
setting-pa3ds-extras-hidden_items-all = Em todos os lugares
# The settings dialog: its title, its pages, and what is not a setting's.
settings-title = Configurações de mod
settings-page-general = Geral
settings-page-speech = Fala
settings-page-audio = Áudio
settings-page-navigation = Navegação
settings-page-extras = Extras
settings-browse = &Navegar...
# The first of the languages pa3ds can speak in: none chosen.
settings-language-game = Idioma do jogo
# Asked once settings are applied.
settings-restart = Reiniciar o pa3ds agora para aplicar estas configurações? Se você escolher Não, elas serão aplicadas na próxima inicialização do pa3ds.
settings-unwritten = pa3ds não conseguiu escrever { $path }, então nada mudou.
dialog-apply = &Aplicar
window-settings = &Configurações...
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = Português do Brasil

## As pa3ds starts: its keys, its settings and Azahar's

# Two or more things, as a list is heard: "c, d and the C-stick".
both = { $these } e { $last }
# The emulator's buttons pa3ds's keys sit on, the buttons by Azahar's names for them.
keys-taken = pa3ds usa { $controls }
keys-applied = teclas aplicadas ao Azahar
keys-waiting = feche o Azahar e inicie o pa3ds novamente para aplicar suas teclas
# A key pa3ds gained that the player's file already uses for something else (ADR-0022).
keys-commented-out = nova tecla comentada em { $file }: { $key } para { $action }, porque { $key } já está atribuída a { $taken }
keys-edit-to-choose = edite { $file } para escolher
# A file of the player's that will not parse; the problem is the TOML reader's own words.
file-unreadable = { $file } não pôde ser lido: { $problem }. Corrija-o ou exclua-o para pa3ds escrever um novo
# The Azahar settings pa3ds writes (ADR-0026).
azahar-settings-applied = Configurações do Azahar aplicadas
azahar-settings-waiting = feche o Azahar e inicie o pa3ds novamente para aplicar as configurações
azahar-settings-missing = As configurações do Azahar não foram encontradas: inicie o Azahar uma vez, feche-o e inicie o pa3ds novamente
# The folder picker's title, read out with it, asking for a portable Azahar's folder, and again
# after a folder that is not one.
azahar-folder-ask = As configurações do Azahar não foram encontradas. Escolha a pasta que contém azahar.exe ou cancele se o Azahar estiver instalado
azahar-folder-again = { $problem }, escolha outra pasta ou cancele
azahar-folder-no-exe = { $folder } não contém azahar.exe
azahar-folder-not-portable = { $folder } não contém uma pasta user ao lado de azahar.exe; portanto, este Azahar mantém suas configurações em AppData

## What a thing on the map is, where neither the game nor a name says

# Each kind is said as the thing's name where it has none: "door, 3 north". The game's own word
# for a kind comes first where it has one ("PC"), and so do the use tiles and furniture of each
# game's `sprites.ftl`.
kind-door = porta
kind-way-out = saída
# The way off Coumarine Gym's top level, walked onto east, which drops the player to the
# ground (spike 250).
kind-way-down = descida
kind-warp = teletransporte
kind-path = caminho
kind-edge = borda
kind-stairs = escadas
kind-stairs-up = escadas para cima
kind-stairs-down = escadas para baixo
# A warp walked through on a level with where it leads (spike 159).
kind-passage = passagem
kind-entrance = entrada
kind-teleporter = teletransportador
kind-elevator = elevador
kind-sign = placa
kind-flag = bandeira
kind-object = objeto
kind-person = pessoa
kind-item-ball = Pokébola de item
# What the Dowsing Machine finds, listed under the hidden items setting.
kind-hidden-item = item oculto
kind-barrier = barreira
kind-hole = buraco
kind-dive-spot = ponto de mergulho
kind-current = corrente
kind-flat-ground = terreno plano
# Where one of Lumiose's streets ends on ground walked on the grid, as its plazas are (spike 251).
kind-open-ground = terreno aberto
# The small square in each of Prism Tower's quiz rooms that asks Bonnie's question, and asks it again (spike 252).
kind-quiz-panel = painel de perguntas
# A run of the blue spots a climbing wall is climbed from, and one on a ledge at its side, where
# the climber steps off it: the Gym's guide says "those blue spots" (spike 245).
kind-blue-spot = ponto azul
kind-wall-ledge = beirada da parede
# An end of one of Coumarine Gym's vines, by what its ride does from there: up a level, down
# one, or across a gap on one (spike 250).
kind-vine-up = trepadeira para subir
kind-vine-down = trepadeira para descer
kind-swinging-vine = trepadeira para se balançar
# Where Shalour Gym's leader comes out to meet the player, at the centre's edges (spike 247).
kind-gym-leader-spot = local da líder do ginásio
kind-slope = declive
kind-bumpy-slope = ladeira irregular
kind-rail = trilho
# Sootopolis Gym's rooms of thin ice, by where the way over each comes out (spike 167).
kind-ice = gelo
# What the Secret Base's decorating cursor is on, where no decoration is (spike 154).
kind-blank = vazio
kind-wall = parede
kind-crumbled-cliff = penhasco desmoronado

## Battle

# Each side's Pokémon, its name and gender sign, its level, and its HP: yours as the game shows it,
# "20/20", the enemy's as a share of its bar, since a bar is all the game shows (ADR-0015).
battle-battler = { $name } L{ $level } { $hp }
battle-share = { $percent }%

## Scenes

# Who a scene's description is said as, named as a speaker is when it takes over from whoever
# spoke last: "Narration: Groudon rises." (Orion, 2026-10-12).
narration = Narração
