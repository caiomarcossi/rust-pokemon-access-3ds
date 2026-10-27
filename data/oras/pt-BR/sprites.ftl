### What to call the people and things on ORAS's maps, when nothing better is known.
###
### A key is an overworld sprite, the one a move model is drawn with (findings, spikes 75 and 89),
### so a name here is everyone drawn as it, everywhere in the game. A name is looked for in order:
### what the player has called them, then what the game itself calls them, out of the player's own
### ROM and in their language (`games.gen6.archives`), and only then this file. So most of what is
### below is heard only when pa3ds cannot read the ROM (`pa3ds.core.names` has the rules).
###
### Each entry says where it came from, because a sprite is only as good as the reading behind it.

## sprites
# People the game has no list of: never a trainer, so the ROM names them nowhere. Read in replays,
# or from the ROM's own id for the sprite and where it stands (`spikes/86_door_kinds.py stands`).
# signs.ctm: "Now take care out there, honey!", in the player's house
sprite-0x118 = Mãe
# opening.ctm: "Halloo! You over there! Please! Help me out!"
sprite-0x13e = Professor Bétula
# HAKASE_RS: Birch on Route 101 in movie3 (spike 52)
sprite-0xb3 = Professor Bétula
# wally.ctm: "Welcome to the Pokémon Center", in two Centers on two saves
sprite-0x12a = Enfermeira
# HAGI_RS: Dewford, Routes 104 and 109, the S.S. Tidal
sprite-0x11b = Sr. Briney
# KUSUNOKI_RS: Slateport and Lilycove
sprite-0x135 = Capitão Stern
# SORAISI_RS: Fallarbor, Meteor Falls, Mossdeep
sprite-0x136 = Professor Cozmo
# MAYUMI_RS: Fallarbor and Route 114
sprite-0x161 = Lanette
# CHAMPFATHER_RS: Rustboro
sprite-0x11a = Sr. Stone
# HANDSOME_RS: the Battle Resort
sprite-0xd7 = Looker
# LEADER7_M_RS: Mossdeep; the ROM names only "Liza & Tate"
sprite-0xc1 = Tate
# LEADER7_F_RS: Mossdeep
sprite-0xc2 = Liza
# CONTEST_RS: beside Ali in every Contest Hall, "That performance by Lisia and Ali just now was amazing!" (story 39:7)
sprite-0xd2 = Lisia
# KIZUNA_RS: Mauville, "I'm Mr. Bonding, the expert on O-Powers!" (story 570:43)
sprite-0x134 = Sr. Bonding
# DIRECTOR_RS: Lilycove, "I'm the Game Director." (story 530:0)
sprite-0x139 = Diretor de jogo
# EV_POKE35: Lilycove's three; its model names Nidoran♂ too, and draws Wailmer's body
sprite-0x1030 = Wailmer

# Shared looks the game never names, called what the ROM's own id for the sprite says they are
# (file 94, `spikes/85_rom_names.py sprite`), since everyone drawn as one hears the same word:
# WOMAN_RS is Mrs. Birch in Littleroot and a woman in 27 other zones. A `_ROOM_RS` sprite is
# the same look indoors. Every one here stands on ORAS's maps; STOPPER3, an invisible blocker
# outside Mauville Gym, is left out.
# BROTHER_RS: Littleroot and Route 101
sprite-0x119 = Jovem
# CHILDBOY_RS
sprite-0x11c = Garotinho
# CHILDBOY_ROOM_RS
sprite-0x150 = Garotinho
# CHILDGIRL_RS
sprite-0x11d = menina
# CHILDGIRL_ROOM_RS
sprite-0x151 = menina
# OLDMAN_RS
sprite-0x11e = Velho
# OLDMAN_ROOM_RS
sprite-0x152 = Velho
# OLDWOMAN_RS
sprite-0x11f = Velha
# OLDWOMAN_ROOM_RS
sprite-0x153 = Velha
# BIGMAN1_RS
sprite-0x120 = Grande homem
# BIGMAN2_RS
sprite-0x121 = Grande homem
# BIGMAN1_ROOM_RS
sprite-0x154 = Grande homem
# BIGMAN2_ROOM_RS
sprite-0x155 = Grande homem
# BIGMAN, X's sprite, once in ORAS
sprite-0x70 = Grande homem
# OYAJI1_RS: Mauville, as are the four after it
sprite-0x122 = Homem de meia idade
# OYAJI2_RS
sprite-0x123 = Homem de meia idade
# OYAJI3_RS
sprite-0x124 = Homem de meia idade
# OYAJI4_RS
sprite-0x125 = Homem de meia idade
# OYAJI5_RS
sprite-0x126 = Homem de meia idade
# WOMAN_RS
sprite-0x128 = Mulher
# WOMAN2_RS
sprite-0x142 = Mulher
# WOMAN_ROOM_RS
sprite-0x156 = Mulher
# MAN_RS
sprite-0x132 = Homem
# MAN_ROOM_RS
sprite-0x157 = Homem
# MAN1, X's sprite, once in ORAS
sprite-0x31 = Homem
# GIRL_RS
sprite-0x131 = Garota
# COUPLE_M_RS: the young couples' man
sprite-0x10c = jovem
# COUPLE_F_RS: and woman
sprite-0x10d = Mulher jovem
# TWIN1_RS
sprite-0x10a = Gêmeo
# TWIN2_RS
sprite-0x10b = Gêmeo
# KARAKURI_RS: Route 110 only, the Trick House
sprite-0x127 = Mestre dos Truques
# SUNGLASS_RS
sprite-0x129 = Homem de óculos escuros
# SHOP_M_RS
sprite-0x12b = Escriturário
# INFORMATION_RS
sprite-0x12c = Recepcionista
# INFORMATIONW, X's sprite
sprite-0x3e = Recepcionista
# DANPEI_RS: in all eight Gym towns, as X's DANPEI is in all eight Gyms
sprite-0x12d = Guia do ginásio
# DELIVER1_RS: in all sixteen Pokémon Centers
sprite-0x12e = Entregador
# DELIVER2_RS: likewise
sprite-0x12f = Entregador
# ASSISTANT_M_RS
sprite-0x133 = Assistente
# ASSISTANTM, X's sprite
sprite-0x5f = Assistente
# KIMONO3_RS
sprite-0x137 = Mulher de quimono
# OFFICEWORKER_RS
sprite-0x140 = Funcionário de escritório
# GAMEBOY_RS
sprite-0x141 = Garoto jogando um jogo
# REPORTER_RS
sprite-0x108 = Repórter
# CAMERAMAN_RS
sprite-0x109 = Cinegrafista
# ELDER_RS: the elder of each pair of Teammates, Anna & Meg and three more (spike 193)
sprite-0x10e = Companheiro de equipe mais velho
# JUNIOR_RS: and the younger
sprite-0x10f = Companheiro de equipe mais jovem

# Trainers' looks the ROM leaves unnamed, one trainer or none of a class standing as each, or a
# pair class beside the class (spike 90); called by the class their id and their trainers share.
# SCHOOL_M_RS
sprite-0xdd = Estudante
# SCHOOL_M_ROOM_RS
sprite-0x145 = Estudante
# SCHOOL_W_ROOM_RS
sprite-0x146 = Estudante
# TANPAN_ROOM_RS
sprite-0x143 = Jovem
# MINI_ROOM_RS
sprite-0x144 = Moça
# MOUNTMAN_ROOM_RS
sprite-0x147 = Caminhante
# AROMA_ROOM_RS
sprite-0x148 = Dama do Aroma
# SWIMMER_M_RS
sprite-0xe5 = Nadador ♂
# UKIWA_W2_RS
sprite-0x15c = Banhista
# BREADER_M_RS
sprite-0xea = Criador Pokémon
# BREADER_W_RS
sprite-0xeb = Criador Pokémon
# FISHING_RS
sprite-0xef = Pescador
# TRIRUN_M_RS
sprite-0x101 = Triatleta
# ELITE_M_ROOM_RS
sprite-0x149 = Treinador Ás
# ELITE_F_ROOM_RS
sprite-0x14a = Treinador Ás
# VETERAN_M_ROOM_RS
sprite-0x14b = Especialista
# VETERAN_F_ROOM_RS
sprite-0x14c = Especialista
# SCARY_M_ROOM_RS
sprite-0x14d = Bandido
# SCARY_W_ROOM_RS
sprite-0x14e = Delinquente

# Things standing on the map that the game never names.
# omega_ruby_bush.ctm: "You can't pass because of prickly thorns, but a Pokémon may be able to cut the tree."
sprite-0x200b = arbusto
# omega_ruby_smash.ctm: "It's a cracked boulder. A Pokémon may be able to break it."
sprite-0x200c = pedra rachada
# omega_ruby_strength-boulders.ctm: "It's a big boulder, but a Pokémon may be able to push it aside."
sprite-0x2009 = pedra grande

# Things standing on the map, by the model their sprite's record in a/1/3/7 draws (spike 211), and
# called what the game calls that thing among the Secret Base decorations (text 56) or the items
# (text 114).
# KIRAKIRA: the sparkle at each of the 28 places a Mega Stone lies, a script each from 0x41f
# to 0x43a (or_mega_stone: "Raiden found a Sablenite!"); the word is story 330:64's
sprite-0x203d = Megapedra
# DOLL_RURIRI: text 56:111, in both houses in Littleroot
sprite-0x204a = Boneca Azurill
# DOLL_ENEKO: text 56:112
sprite-0x204b = Boneca Skitty
# DOLL_TIRUTTO: text 56:113
sprite-0x204c = Boneca Swablu
# DOLL_RAPURASU: text 56:120
sprite-0x204d = Boneca Lapras
# DOLL_KABIGON: text 56:118
sprite-0x204e = Boneca Snorlax
# AIIRONOTAMA: text 114:535, on Mt. Pyre's summit
sprite-0x204f = Orbe Azul
# BENIIRONOTAMA: text 114:534, beside it
sprite-0x2050 = Orbe Vermelho
# MAGMASUIT: text 114:741, in the Magma Hideout
sprite-0x2056 = Traje Magma
# AQUASUIT: text 114:742, in the Aqua Hideout
sprite-0x2057 = Traje Aqua
# ASASE_SHIO: text 114:70, in Shoal Cave
sprite-0x2058 = Sal Cardume
# ASASE_KAIGARA: text 114:71, in Shoal Cave
sprite-0x2059 = Concha Cardume
# LETTER: in Steven's house; the item, text 114:725, is the one to him
sprite-0x205b = Carta

# The game's own words, copied from the ROM's English text by `spikes/90_sprite_trainers.py`: the
# story's people by the trainer each is, and every other sprite that two trainers or more of one
# class stand as, by that class (findings, spike 90). With the ROM, these come from it instead.
# RIVAL_RS
sprite-0xb4 = Wally
# RIVAL_EX_RS
sprite-0xb5 = Wally
# CHAMPION_RS
sprite-0xb6 = Steven
# BIGFOUR1_RS
sprite-0xb7 = Sidney
# BIGFOUR2_RS
sprite-0xb8 = Febe
# BIGFOUR3_RS
sprite-0xb9 = Glácia
# BIGFOUR4_RS
sprite-0xba = Drake
# LEADER1_RS
sprite-0xbb = Roxane
# LEADER2_RS
sprite-0xbc = Brawly
# LEADER3_RS
sprite-0xbd = Wattson
# LEADER4_RS
sprite-0xbe = Flannery
# LEADER5_RS
sprite-0xbf = Norman
# LEADER6_RS
sprite-0xc0 = Winona
# LEADER8_RS
sprite-0xc3 = Wallace
# MAGMABOSS_RS
sprite-0xc4 = Maxie
# MAGMAKANBU_M_RS
sprite-0xc5 = Tabitha
# MAGMAKANBU_F_RS
sprite-0xc6 = Courtney
# AQUABOSS_RS
sprite-0xc9 = Archie
# AQUAKANBU_M_RS
sprite-0xca = Matt
# AQUAKANBU_F_RS
sprite-0xcb = Shelly
# HIMITU_RS
sprite-0xd3 = Aarune
# ZAZAKO_RS: Lorekeeper Zinnia, outside Sky Pillar (spike 193)
sprite-0xd4 = Zínia
# SUPPORT_F_RS, drawn into work object 0x7a (spike 89)
sprite-0x13a = May
# SUPPORT_M_RS, likewise
sprite-0x13b = Brendan
# SUPPORT_F_BIKE_RS, by the ROM's own id; no replay yet
sprite-0x13c = May
# SUPPORT_M_BIKE_RS, likewise
sprite-0x13d = Brendan
# LEADER5A_RS: wally.ctm, "thanked as Mr. Norman"
sprite-0x13f = Norman
# LEADER2A_RS
sprite-0x15d = Brawly
# BIGFOUR2A_RS
sprite-0x15e = Febe
# LEADER6A_RS
sprite-0x15f = Winona
# BIGFOUR1A_RS
sprite-0x160 = Sidney
# MAGMA_M_RS
sprite-0xc7 = Recruta da Equipe Magma
# MAGMA_F_RS
sprite-0xc8 = Recruta da Equipe Magma
# AQUA_M_RS
sprite-0xcc = Recruta da Equipe Aqua
# AQUA_F_RS
sprite-0xcd = Recruta da Equipe Aqua
# TANPAN_RS
sprite-0xda = Jovem
# MINI_RS
sprite-0xdb = Moça
# MUSITORI_RS
sprite-0xdc = Caçador de Insetos
# SCHOOL_W_RS
sprite-0xde = Estudante
# CAMPBOY_RS
sprite-0xdf = Campista
# PICNICGIRL_RS
sprite-0xe0 = Garota do Piquenique
# RICHBOY_RS
sprite-0xe1 = Garoto rico
# RICHGIRL_RS
sprite-0xe2 = Senhora
# UKIWA_M_RS
sprite-0xe3 = Banhista
# UKIWA_W_RS
sprite-0xe4 = Banhista
# SWIMMER_W_RS
sprite-0xe6 = Nadador ♀
# GENTLEMAN_RS
sprite-0xe7 = Cavalheiro
# MARINER_RS
sprite-0xe8 = Marinheiro
# MOUNTMAN_RS
sprite-0xe9 = Caminhante
# FIGHTER_W_RS
sprite-0xed = Lutadora
# COLLECTOR_RS
sprite-0xee = Colecionador
# MONSTERMANIA_RS
sprite-0xf0 = Poké Maníaco
# BUGMANIA_RS
sprite-0xf1 = Maníaco por Insetos
# RUINMANIA_RS
sprite-0xf2 = Maníaco das Ruínas
# OCCULT_RS
sprite-0xf3 = Maníaca Hex
# ESP_M_RS
sprite-0xf4 = Psíquico
# NINJA_RS
sprite-0xf5 = Menino Ninja
# CAMPFIRE_RS
sprite-0xf6 = Fogueiro
# BIRDTAMER_RS
sprite-0xf7 = Criador de Pássaros
# DRAGONTAMER_RS
sprite-0xf8 = Domador de Dragões
# GUITAR_RS
sprite-0xf9 = Guitarrista
# RANGER_M_RS
sprite-0xfa = Ranger Pokémon
# RANGER_W_RS
sprite-0xfb = Ranger Pokémon
# ADULT_RS
sprite-0xfc = Beldade
# PARASOL_RS
sprite-0xfd = Dama da Sombrinha
# AROMA_RS
sprite-0xfe = Dama do Aroma
# DAISUKI_M_RS
sprite-0xff = Fã de Pokémon
# DAISUKI_W_RS
sprite-0x100 = Fã de Pokémon
# TRIBIKE_M_RS
sprite-0x102 = Triatleta
# TRISWIM_M_RS
sprite-0x103 = Triatleta
# ELITE_M_RS
sprite-0x104 = Treinador Ás
# SWIMMER_W2_RS
sprite-0x159 = Nadador ♀
# FIGHTER_M_RS
sprite-0xec = Faixa Preta
# ELITE_F_RS
sprite-0x105 = Treinador Ás
# VETERAN_M_RS
sprite-0x106 = Especialista
# FAIRYTALE_RS, beside two pairs of Mysterious Sisters (spike 193)
sprite-0x110 = Garota de Contos de Fadas
# VETERAN_F_RS
sprite-0x107 = Especialista
# NAMINORI_M_RS
sprite-0x111 = Treinador Ás
# NAMINORI_F_RS
sprite-0x112 = Treinador Ás
# BACKPACKER_M_RS
sprite-0x113 = Mochileiro
# SCARY_M_RS
sprite-0x114 = Bandido
# SCARY_W_RS
sprite-0x115 = Delinquente
# DIVER_M_RS
sprite-0x116 = Mergulhador
# DIVER_F_RS
sprite-0x117 = Mergulhador Livre
# FISHING2_RS
sprite-0x158 = Pescador
# SWIMMER_M2_RS
sprite-0x15a = Nadador ♂
# UKIWA_M2_RS
sprite-0x15b = Banhista

# The Pokémon on the map, by the Pokémon each one's model draws (spike 193); with the ROM, its
# own species names.
# RIDE_POKE01
sprite-0x1006 = Rhyhorn
# EV_POKE07
sprite-0x1009 = Lucario
# RIDE_POKE05
sprite-0x100f = Rhyhorn
# EV_POKE15
sprite-0x1013 = Marill
# RIDE_POKE01_EX
sprite-0x1014 = Rhyhorn
# EV_POKE20
sprite-0x101b = Groudon
# EV_POKE21
sprite-0x101d = Kyogre
# EV_POKE22
sprite-0x101f = Rayquaza
# EV_POKE23
sprite-0x1021 = Latias
# EV_POKE24
sprite-0x1022 = Latios
# RIDE_POKE13
sprite-0x1023 = Latias
# RIDE_POKE14
sprite-0x1024 = Latios
# EV_POKE25
sprite-0x1025 = Machoke
# EV_POKE26
sprite-0x1026 = Wingull
# EV_POKE28
sprite-0x1029 = Kecleon
# EV_POKE29
sprite-0x102a = Regirock
# EV_POKE30
sprite-0x102b = Regice
# EV_POKE31
sprite-0x102c = Registeel
# EV_POKE32
sprite-0x102d = Poochyena
# EV_POKE33
sprite-0x102e = Skitty
# EV_POKE34
sprite-0x102f = Zigzagoon
# EV_POKE36
sprite-0x1031 = Plusle
# EV_POKE37
sprite-0x1032 = Minun
# EV_POKE38
sprite-0x1033 = Meditite
# EV_POKE39
sprite-0x1034 = Shroomish
# EV_POKE40
sprite-0x1035 = Slakoth
# EV_POKE41
sprite-0x1036 = Roselia
# EV_POKE42
sprite-0x1037 = Taillow
# EV_POKE44
sprite-0x1039 = Whismur
# EV_POKE45
sprite-0x103a = Electrike
# EV_POKE46
sprite-0x103b = Altaria
# EV_POKE47
sprite-0x103c = Poochyena
# EV_POKE48
sprite-0x103d = Groudon
# EV_POKE49
sprite-0x103e = Kyogre
# EV_POKE50
sprite-0x103f = Machoke

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 155, which says what is there; the word here is taken from
# that line (spike 210). The PC's and the TV's are the game's own names for them. The PC's word is
# read from the ROM ahead of this one.
# the PC's own lines (archives.PC_LINES)
tile-0xd5 = PC
# text 56:138, the TV among the Secret Base decorations
tile-0xd7 = TV
# text 155:0
tile-0xd8 = bugigangas Pokémon
# text 155:9
tile-0xd9 = revistas Pokémon
# text 155:2
tile-0xda = armário
# text 155:3
tile-0xdb = itens úteis
# text 155:4
tile-0xdc = produtos Pokémon
# text 155:6
tile-0xdd = lata de lixo
# text 155:7
tile-0xde = livros ilustrados de Pokémon
# text 155:10
tile-0xdf = vaso

## furniture
# What a piece of furniture is, by the game's shared script it runs: a record's own number from
# 0x7d0 up, or the one its zone's script runs for it (spike 210). Every vending machine runs one of
# 0x28f0 to 0x28f4, and is called what the game calls the one among the Secret Base decorations
# (text 56:144); the wardrobes say so (story 460:35).
# text 56:144
furniture-0x28f0 = máquina de venda automática
# text 56:144
furniture-0x28f1 = máquina de venda automática
# text 56:144
furniture-0x28f2 = máquina de venda automática
# text 56:144
furniture-0x28f3 = máquina de venda automática
# text 56:144
furniture-0x28f4 = máquina de venda automática
# story 460:35
furniture-0x284e = guarda-roupa
# The elevators' word is the Mauville Hills computer's, story 264:15; the bedrooms', the lines the
# player's room shows.
# Mauville's and the Lilycove Department Store's doors
furniture-0x27da = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27db = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27dc = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27dd = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27de = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27df = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e0 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e1 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e2 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e3 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e4 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e5 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e6 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e7 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e8 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e9 = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27ea = elevador
# Mauville's and the Lilycove Department Store's doors
furniture-0x27eb = elevador
# story 253:36, the player's room and the neighbour's
furniture-0x2956 = relógio
# story 253:43, both rooms
furniture-0x2961 = Wii U
# text 114:442, the item; both rooms
furniture-0x2962 = Mapa da cidade
# story 253:41, both rooms
furniture-0x2963 = caderno
# Mom's, in both houses
furniture-0x2965 = caderno

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spike 211). A slot is in a
# group where its message opens as the group's does in any one language; Omega Ruby and Alpha
# Sapphire have the same.
# story 341:8, "There's a bed.", in the Magma Hideout
place-0xaa-furniture-0 = cama
place-0xaa-furniture-1 = cama
place-0xaa-furniture-2 = cama
place-0xaa-furniture-3 = cama
place-0xaa-furniture-4 = cama
place-0xaa-furniture-5 = cama
place-0xaa-furniture-6 = cama
place-0xaa-furniture-7 = cama
# story 420:16, in the Aqua Hideout
place-0x5d-furniture-0 = cama
place-0x5d-furniture-1 = cama
place-0x5d-furniture-2 = cama
place-0x5d-furniture-3 = cama
place-0x5d-furniture-4 = cama
place-0x5d-furniture-5 = cama
place-0x5d-furniture-6 = cama
place-0x5d-furniture-7 = cama
# story 169:14, at the Weather Institute
place-0x188-furniture-0 = cama
place-0x188-furniture-1 = cama
# story 70:0, "It's a bed.", on the S.S. Tidal
place-0xda-furniture-0 = cama
# story 58:5, "Aha! Pokémon Journal", in the Pokémon Centers
place-0xe4-furniture-0 = Diário Pokémon
place-0xe8-furniture-0 = Diário Pokémon
place-0xec-furniture-0 = Diário Pokémon
place-0xf0-furniture-0 = Diário Pokémon
place-0xf6-furniture-0 = Diário Pokémon
place-0xfd-furniture-0 = Diário Pokémon
place-0x103-furniture-0 = Diário Pokémon
place-0x109-furniture-0 = Diário Pokémon
place-0x116-furniture-0 = Diário Pokémon
place-0x11c-furniture-0 = Diário Pokémon
place-0x12d-furniture-0 = Diário Pokémon
place-0x135-furniture-0 = Diário Pokémon
place-0x14a-furniture-0 = Diário Pokémon
place-0x155-furniture-0 = Diário Pokémon
place-0x161-furniture-0 = Diário Pokémon
place-0x206-furniture-0 = Diário Pokémon
# story 587:13, "Hoenn's Local Heroes / The Hoenn Rangers Coexistence Force!": posters, figures, a face board
place-0x115-furniture-5 = Rangers de Hoenn
place-0x115-furniture-6 = Rangers de Hoenn
place-0x143-furniture-4 = Rangers de Hoenn
place-0x143-furniture-5 = Rangers de Hoenn
place-0x148-furniture-0 = Rangers de Hoenn
place-0x151-furniture-1 = Rangers de Hoenn
place-0x151-furniture-2 = Rangers de Hoenn
place-0x188-furniture-4 = Rangers de Hoenn
place-0x188-furniture-5 = Rangers de Hoenn
place-0x1c2-furniture-10 = Rangers de Hoenn
place-0x1c2-furniture-11 = Rangers de Hoenn
place-0x20d-furniture-0 = Rangers de Hoenn
# story 206:0, the Trick House's scroll doors
place-0x175-furniture-0 = porta trancada
place-0x176-furniture-0 = porta trancada
place-0x177-furniture-0 = porta trancada
place-0x178-furniture-0 = porta trancada
place-0x179-furniture-0 = porta trancada
place-0x17a-furniture-0 = porta trancada
# story 341:0, "PROJECT AZOTH / File No. 1", in both hideouts
place-0xaa-furniture-8 = Projeto AZOTH
place-0xaa-furniture-9 = Projeto AZOTH
place-0xaa-furniture-10 = Projeto AZOTH
place-0xaa-furniture-11 = Projeto AZOTH
place-0xab-furniture-1 = Projeto AZOTH
place-0x5d-furniture-8 = Projeto AZOTH
place-0x5d-furniture-9 = Projeto AZOTH
place-0x5d-furniture-10 = Projeto AZOTH
place-0x5d-furniture-11 = Projeto AZOTH
place-0x5e-furniture-1 = Projeto AZOTH
# story 338:0, "It's the Team Magma flag!"
place-0xad-furniture-0 = Bandeira da Equipe Magma
place-0xad-furniture-1 = Bandeira da Equipe Magma
place-0xae-furniture-2 = Bandeira da Equipe Magma
# story 417:4, "It's the Team Aqua flag!"
place-0x60-furniture-4 = Bandeira da Equipe Aqua
place-0x60-furniture-5 = Bandeira da Equipe Aqua
place-0x61-furniture-2 = Bandeira da Equipe Aqua
# story 65:25, "It's an Azurill Doll!", where the doll stands
place-0xdf-furniture-0 = Boneca Azurill
place-0xe1-furniture-0 = Boneca Azurill
# in the moving truck
place-0x18f-furniture-0 = Boneca Azurill
# The rocks that evolve Eevee, which no text names: each shows only a message of its own, the
# same records in every language (spike 211's `same`). Story 378:0, "It's a rock that is covered
# in ice.", in Shoal Cave's ice room (or_ice_rock)
place-0x86-furniture-0 = rocha de gelo
# story 432:45, "The surface is covered with moss.", in Petalburg Woods (or_mossy_rock)
place-0x52-furniture-2 = rocha musgosa
