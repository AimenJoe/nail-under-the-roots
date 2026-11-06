"""Epic Games Store 상점 글(짧은 설명·헤드라인·About)의 언어별 판, 그리고 Epic 현지화 CSV 템플릿 채우기.

용어는 게임 번역 표(src/i18n/<언어>.rs)의 것을 그대로 쓴다: 반딧불 glowbugs, 씨앗 seeds, 길 지도 칸(굴·둥지·저장고·샘·실타래),
길의 주인 node_boss, 실뜨기 knot_title, 실의 꿈 dream_title, 포포의 응원 set_assist, 큰 나무 tree_title, 발자국 tab_journal,
도감 tab_bestiary, 앨범 title_album, 이웃의 부탁 h_quest, 캐릭터 이름 name_*.
상점 글에는 무료판(웹·APK)을 알리지 않는다(0276). 영어 원문은 docs/epic/store-text.md.

    python3 tools/store/epic_text.py <포털에서 내려받은 localization.csv> <내보낼 csv>

Epic 템플릿 규칙: 기본 언어(en) 칸은 올려도 무시된다 — 영어는 포털에서 고친다.
"""
import csv
import sys

# Epic CSV 열 이름 → 글. es와 es-ES에는 같은 스페인어(스페인)를 넣는다. pt(포르투갈)는 비워 둔다(영어로 보인다 — 게임은 pt-BR).
T = {}

T["ko"] = dict(
    short="아기 두더지 나일이 시들어 가는 큰 나무의 뿌리를 따라 땅속으로 내려가는 귀여운 탑다운 액션 로그라이트. 박자에 맞춰 받아치며, 보라색 실에 묶인 이웃들을 풀어 주세요.",
    headline="큰 나무가 시들어 간다. 아기 두더지가 혼자 길을 나선다.",
    body="""큰 나무가 시들어 간다. 뿌리를 타고 번진 보라색 실이 땅속 이웃들을 하나둘 감아, 지나는 이를 막아서게 만들었다. 아기 두더지 나일이 처음으로 혼자 굴을 나서 뿌리를 따라 내려간다.

두더지답게 싸운다
박자에 맞추면 더 세지는 발톱 콤보, 적을 벽까지 밀어붙이는 강타, 땅속으로 파고들어 공격을 흘리고 솟구쳐 들이받기, 돌진과 던지기를 되받아치는 받아치기. 적은 공격하기 전에 붉게 예고한다 — 띠는 받아치고, 원은 피하자.

런마다 새 길
갈래 지도에서 다음 굴을 고른다. 굴·둥지·저장고·샘·실타래, 매번 새로 파인다. 깊이 내려갈수록 어두워지는 지대 넷, 지대마다 다른 생물과 위험한 바닥, 길 끝마다 기다리는 길의 주인. 모은 반딧불과 씨앗은 쓰러져도 잃지 않는다.

이웃을 깨워 마을로
실에 물든 이웃을 이기면 실이 풀린다. 토끼 모모, 다람쥐 도토, 아르마딜로 콩이…… 깨어난 이웃은 큰 나무 아래 마을에 자리를 잡고, 나무는 다시 꽃을 피운다. 모모와 도토를 깨우면 직접 골라 내려갈 수도 있다 — 뒷발차기와 멀리뛰기, 또는 멀리서 던지는 도토리.

더 파고들 것
• 큰 나무 재주: 세 갈래 × 여덟 깊이, 캐릭터마다 따로
• 실뜨기 1~10 단계, 실의 꿈(보스 다시 싸우기), 이웃의 부탁, 발자국 목표, 도감과 앨범
• 엔딩을 본 뒤 열리는 넷째 길

내 방식대로
키보드·게임패드, 버튼 바꾸기. 포포의 응원(도움 모드), 색 없이도 읽히는 예고, 흔들림·번쩍임·진동 줄이기. 열한 언어.

캐릭터 스프라이트·초상화·일러스트는 Google Gemini 이미지 모델로 그린 원본을 도트로 옮겨 손으로 다듬은 것입니다.""",
)

T["ja"] = dict(
    short="かわいい見下ろし型アクションローグライト。子どもモグラのナイルが、枯れかけた大きな木の根をたどって地下へ——拍子に合わせてはじき、紫の糸にしばられたご近所さんを助けよう。",
    headline="大きな木が枯れていく。子どもモグラがひとりで旅に出る。",
    body="""大きな木が枯れていく。根をつたって広がった紫の糸が、地下の住人をひとりまたひとりと絡めとり、通る者の前に立ちはだからせた。子どもモグラのナイルは、はじめてひとりで巣穴を出て、根をたどって下りていく。

モグラらしく戦う
拍子に合わせるほど強くなるツメの連続攻撃、敵を壁までふき飛ばす強こうげき、地中にもぐって攻撃をやりすごし、飛び出して体当たり。突進や投げ物は、はじきで打ち返す。敵は攻撃の前に赤く予告する——帯ははじき、円はよけよう。

毎回ちがう道
分かれ道の地図で次の場所を選ぶ。穴・巣・たくわえ穴・泉・糸だま、どれも毎回新しく掘られる。深く下りるほど暗くなる四つの地帯には、それぞれちがう生き物と危ない地面があり、道の終わりには道のぬしが待つ。集めたホタルとたねは、倒れてもなくならない。

ご近所さんを起こして村へ
糸に染まったご近所さんに勝つと、糸がほどける。ウサギのモモ、リスのドト、アルマジロのコンイ……目を覚ました仲間は大きな木の下の村に住みつき、木はふたたび花を咲かせる。モモとドトを起こせば、自分で選んで潜ることも——後ろ足のキックと大ジャンプ、または遠くから投げるどんぐり。

もっと掘りすすむ
• 大きな木のわざ:三つの枝×八段、キャラクターごとに別の木
• あやとり1〜10段階、糸の夢(ボスともう一度)、ご近所さんのおねがい、あしあと、ずかんとアルバム
• エンディングを見たあとに開く四つめの道

自分のやり方で
キーボード・ゲームパッド、ボタンの割り当て変更。ポポの応援(アシストモード)、色がなくても読める予告、ゆれ・点滅・振動をおさえる設定。11言語。

キャラクターのスプライト・顔グラフィック・イラストは、Google Gemini の画像モデルで作った原画をドット絵に起こし、手で仕上げたものです。""",
)

T["zh-CN"] = dict(
    short="一款可爱的俯视角动作 Roguelite。小鼹鼠拿日沿着日渐枯萎的大树之根深入地下——踩着节拍弹开攻击,解救被紫色丝线缠住的邻居们。",
    headline="大树正在枯萎。小鼹鼠独自踏上旅途。",
    body="""大树正在枯萎。紫色丝线顺着树根蔓延,把地下的居民一个个缠住,让他们挡住每一个路过的人。小鼹鼠拿日第一次独自离开洞穴,沿着树根向下走去。

像鼹鼠一样战斗
踩准节拍就会更强的爪击连段,把敌人撞向墙壁的重击,钻进地下躲过攻击再破土冲撞,还能用弹开把冲锋和投掷物打回去。敌人出手前会用红色预告——条带就弹开,圆圈就闪避。

每局都是新的路
在分岔的路线图上选择下一站:洞穴、巢穴、储藏洞、泉水、线团,每次都重新挖出。越往深处越暗的四个地带,各有不同的生物和危险地面,每条路的尽头都有路之主在等待。收集的萤火虫和种子即使倒下也不会丢失。

唤醒邻居,带他们回家
打败被丝线染上的邻居,丝线就会松开。兔子茉茉、松鼠多托、犰狳孔伊……醒来的邻居会在大树下的村子里安家,大树也会再次开花。唤醒茉茉和多托后,还能亲自选择他们下去冒险——后腿踢击与远跳,或从远处投掷橡果。

继续深挖
• 大树的才能:三条分支 × 八层,每个角色各有一棵树
• 翻花绳 1–10 级、丝之梦(再战首领)、邻居的请求、足迹目标、图鉴与相册
• 看过结局后开启的第四条路

按你的方式玩
键盘或手柄,可自定义按键。波波的加油(辅助模式)、不靠颜色也能看懂的预告,以及减弱震屏、闪光和震动的选项。支持十一种语言。

角色像素图、头像与插画的原画由 Google Gemini 图像模型生成,再转换为像素画并手工修整。""",
)

T["zh-Hant"] = dict(
    short="一款可愛的俯視角動作 Roguelite。小鼴鼠拿日沿著日漸枯萎的大樹之根深入地下——踩著節拍彈開攻擊,解救被紫色絲線纏住的鄰居們。",
    headline="大樹正在枯萎。小鼴鼠獨自踏上旅程。",
    body="""大樹正在枯萎。紫色絲線順著樹根蔓延,把地下的居民一個個纏住,讓他們擋住每一個路過的人。小鼴鼠拿日第一次獨自離開洞穴,沿著樹根往下走。

像鼴鼠一樣戰鬥
踩準節拍就會更強的爪擊連段、把敵人撞向牆壁的重擊、鑽進地下躲過攻擊再破土衝撞,還能用彈開把衝鋒和投擲物打回去。敵人出手前會用紅色預告——帶狀就彈開,圓圈就閃避。

每局都是新的路
在分岔的路線圖上選擇下一站:洞穴、巢穴、儲藏洞、泉水、線團,每次都重新挖出。越往深處越暗的四個地帶,各有不同的生物和危險地面,每條路的盡頭都有路之主在等待。收集的螢火蟲和種子即使倒下也不會遺失。

喚醒鄰居,帶他們回家
打敗被絲線染上的鄰居,絲線就會鬆開。兔子茉茉、松鼠多托、犰狳孔伊……醒來的鄰居會在大樹下的村子裡安家,大樹也會再次開花。喚醒茉茉和多托後,還能親自選他們下去冒險——後腿踢擊與遠跳,或從遠處投擲橡實。

繼續深挖
• 大樹的才能:三條分支 × 八層,每個角色各有一棵樹
• 翻花繩 1–10 級、絲之夢(再戰頭目)、鄰居的請求、足跡目標、圖鑑與相簿
• 看過結局後開啟的第四條路

照你的方式玩
鍵盤或手把,可自訂按鍵。波波的加油(輔助模式)、不靠顏色也看得懂的預告,以及減弱畫面震動、閃光與震動回饋的選項。支援十一種語言。

角色像素圖、頭像與插畫的原畫由 Google Gemini 影像模型生成,再轉換為像素畫並手工修整。""",
)

T["pt-BR"] = dict(
    short="Um roguelite de ação fofo com visão de cima. Naïl, uma toupeirinha, segue as raízes de uma grande árvore que está murchando — apare no ritmo e liberte os vizinhos presos por fios roxos.",
    headline="A grande árvore está murchando. Uma toupeirinha parte sozinha.",
    body="""A grande árvore está murchando. Fios roxos se espalharam pelas raízes e enrolaram, um a um, os moradores do subsolo, colocando-os contra quem passa. Naïl, uma toupeirinha, sai da toca sozinha pela primeira vez para seguir as raízes para baixo.

LUTE COMO UMA TOUPEIRA
Combos de garra que batem mais forte no ritmo, golpes pesados que jogam os inimigos contra a parede, cavar para escapar dos ataques e sair do chão numa investida, e aparar para rebater investidas e projéteis. Todo inimigo mostra o ataque em vermelho antes — apare as faixas, desvie dos círculos.

UM CAMINHO NOVO A CADA PARTIDA
Escolha seu rumo num mapa de trilhas que se ramifica: tocas, ninhos, despensas, nascentes e novelos, cavados de novo a cada vez. Quatro zonas ficam mais escuras quanto mais fundo você vai, cada uma com suas criaturas e chão perigoso, e um Dono da trilha espera no fim de cada caminho. Vaga-lumes e sementes coletados nunca se perdem quando você cai.

DESPERTE OS VIZINHOS E LEVE-OS PARA CASA
Vença um vizinho enrolado e os fios se soltam. Momo, a coelha, Doto, o esquilo, Kongi, o tatu, e outros se mudam para a vila sob a grande árvore, e a árvore volta a florescer. Desperte Momo e Doto e jogue com eles também — coices e saltos longos, ou bolotas à distância.

CONTINUE CAVANDO
• Talentos da Grande árvore: três ramos de oito passos, uma árvore para cada herói
• Cama de gato 1–10, Sonho de fio (enfrente os chefes de novo), pedidos dos vizinhos, metas de Pegadas, bestiário e álbum
• Um quarto caminho que se abre depois de ver um final

JOGUE DO SEU JEITO
Teclado ou controle, com botões remapeáveis. Modo de ajuda Torcida do Popo, avisos legíveis sem cor e opções para diminuir tremor, flashes e vibração. Onze idiomas.

Os sprites, retratos e ilustrações dos personagens foram feitos a partir de originais gerados com um modelo de imagem Google Gemini, depois convertidos em pixel art e retocados à mão.""",
)

T["fr"] = dict(
    short="Un roguelite d'action mignon en vue de dessus. Naïl, un bébé taupe, suit les racines d'un grand arbre qui se fane — parez en rythme et libérez les voisins prisonniers des fils violets.",
    headline="Le grand arbre se fane. Un bébé taupe part seul.",
    body="""Le grand arbre se fane. Des fils violets ont gagné ses racines et enroulé un à un les habitants du sous-sol, qui barrent désormais la route à quiconque passe. Naïl, un bébé taupe, quitte le terrier seul pour la première fois et suit les racines vers les profondeurs.

SE BATTRE COMME UNE TAUPE
Des combos de griffes plus forts en rythme, des coups lourds qui projettent les ennemis contre les murs, s'enfouir pour laisser passer les attaques puis jaillir pour charger, et des parades qui renvoient charges et projectiles. Chaque ennemi annonce son attaque en rouge — parez les bandes, esquivez les cercles.

UN NOUVEAU CHEMIN À CHAQUE PARTIE
Choisissez votre route sur une carte qui se ramifie : terriers, nids, réserves, sources et pelotes, creusés à neuf à chaque fois. Quatre zones de plus en plus sombres à mesure que l'on descend, chacune avec ses créatures et son sol dangereux, et un Gardien au bout de chaque chemin. Les lucioles et les graines récoltées ne sont jamais perdues quand vous tombez.

RÉVEILLER LES VOISINS, LES RAMENER CHEZ EUX
Battez un voisin emmêlé et les fils se dénouent. Momo la lapine, Doto l'écureuil, Kongi le tatou et d'autres s'installent au village sous le grand arbre, qui fleurit de nouveau. Réveillez Momo et Doto pour jouer avec eux aussi — coups de pattes arrière et longs sauts, ou glands lancés à distance.

CREUSER ENCORE
• Talents du Grand Arbre : trois branches de huit étapes, un arbre par héros
• Jeu de ficelle 1–10, le Rêve de fil (affrontez à nouveau les boss), les demandes des voisins, les objectifs des Traces, le bestiaire et l'album
• Un quatrième chemin qui s'ouvre après avoir vu une fin

JOUER À SA FAÇON
Clavier ou manette, touches reconfigurables. Mode d'aide Soutien de Popo, avertissements lisibles sans les couleurs, et options pour réduire tremblements, flashs et vibrations. Onze langues.

Les sprites, portraits et illustrations des personnages proviennent d'originaux générés avec un modèle d'image Google Gemini, puis convertis en pixel art et retouchés à la main.""",
)

T["es-ES"] = dict(
    short="Un roguelite de acción adorable con vista cenital. Naïl, un topo bebé, sigue las raíces de un gran árbol que se marchita — desvía al ritmo y libera a los vecinos atrapados por hilos morados.",
    headline="El gran árbol se marchita. Un topo bebé parte solo.",
    body="""El gran árbol se marchita. Unos hilos morados han trepado por sus raíces y han ido envolviendo uno a uno a los habitantes del subsuelo, que ahora cierran el paso a quien se acerque. Naïl, un topo bebé, sale solo de la madriguera por primera vez para seguir las raíces hacia abajo.

LUCHA COMO UN TOPO
Combos de garras que golpean más fuerte al ritmo, golpes fuertes que estampan a los enemigos contra la pared, excavar para esquivar ataques y salir del suelo embistiendo, y desvíos que devuelven cargas y proyectiles. Cada enemigo avisa de su ataque en rojo — desvía las franjas, esquiva los círculos.

UN CAMINO NUEVO EN CADA PARTIDA
Elige tu ruta en un mapa de caminos que se ramifica: madrigueras, nidos, despensas, manantiales y ovillos, excavados de nuevo cada vez. Cuatro zonas cada vez más oscuras cuanto más bajas, cada una con sus criaturas y su suelo peligroso, y un Guardián al final de cada camino. Las luciérnagas y semillas que reúnas nunca se pierden al caer.

DESPIERTA A LOS VECINOS Y LLÉVALOS A CASA
Vence a un vecino enredado y los hilos se sueltan. Momo la coneja, Doto la ardilla, Kongi el armadillo y más se instalan en la aldea bajo el gran árbol, y el árbol vuelve a florecer. Despierta a Momo y a Doto y juega también con ellos — patadas traseras y saltos largos, o bellotas a distancia.

SIGUE CAVANDO
• Talentos del Gran árbol: tres ramas de ocho pasos, un árbol para cada héroe
• Cuna de gato 1–10, el Sueño de hilo (vuelve a enfrentarte a los jefes), pedidos de los vecinos, objetivos de Huellas, bestiario y álbum
• Un cuarto camino que se abre tras ver un final

JUEGA A TU MANERA
Teclado o mando, con controles reasignables. Modo de ayuda Ánimos de Popo, avisos legibles sin color y opciones para reducir temblores, destellos y vibración. Once idiomas.

Los sprites, retratos e ilustraciones de los personajes se hicieron a partir de originales generados con un modelo de imagen de Google Gemini, convertidos después a pixel art y retocados a mano.""",
)
T["es"] = T["es-ES"]

T["de"] = dict(
    short="Ein niedliches Top-down-Action-Roguelite. Naïl, ein Maulwurfbaby, folgt den Wurzeln eines welkenden großen Baums in die Tiefe — pariere im Takt und befreie die Nachbarn aus den lila Fäden.",
    headline="Der große Baum welkt. Ein Maulwurfbaby macht sich allein auf den Weg.",
    body="""Der große Baum welkt. Lila Fäden sind an seinen Wurzeln entlanggekrochen und haben die Bewohner unter der Erde einen nach dem anderen umwickelt – nun stellen sie sich jedem in den Weg. Naïl, ein Maulwurfbaby, verlässt zum ersten Mal allein den Bau und folgt den Wurzeln nach unten.

KÄMPFEN WIE EIN MAULWURF
Krallen-Kombos, die im Takt härter treffen, schwere Hiebe, die Gegner gegen die Wand schleudern, Eingraben, um Angriffen auszuweichen, und Hervorbrechen zum Rammstoß – dazu Parieren, das Ansturm und Wurfgeschosse zurückschlägt. Jeder Gegner kündigt seinen Angriff rot an: Streifen parieren, Kreisen ausweichen.

JEDER DURCHLAUF EIN NEUER WEG
Wähle deinen Weg auf einer verzweigten Pfadkarte: Baue, Nester, Vorräte, Quellen und Knäuel, jedes Mal frisch gegraben. Vier Zonen, die tiefer unten immer dunkler werden, jede mit eigenen Kreaturen und gefährlichem Boden – und am Ende jedes Pfads wartet ein Pfadhüter. Gesammelte Glühwürmchen und Samen gehen nie verloren, wenn du fällst.

NACHBARN WECKEN, NACH HAUSE BRINGEN
Besiege einen umgarnten Nachbarn, und die Fäden lösen sich. Momo das Kaninchen, Doto das Eichhörnchen, Kongi das Gürteltier und weitere ziehen ins Dorf unter dem großen Baum, und der Baum blüht wieder auf. Wecke Momo und Doto, um auch mit ihnen zu spielen – Hinterbeintritte und weite Sprünge oder Eicheln aus der Ferne.

WEITERGRABEN
• Talente des Großen Baums: drei Äste mit je acht Stufen, ein eigener Baum für jede Figur
• Fadenspiel 1–10, der Fadentraum (Bosse erneut bekämpfen), Bitten der Nachbarn, Spuren-Ziele, Tierbuch und Album
• Ein vierter Pfad, der sich öffnet, nachdem du ein Ende gesehen hast

SPIEL AUF DEINE ART
Tastatur oder Gamepad, frei belegbare Tasten. Hilfsmodus Popos Ansporn, Warnungen, die auch ohne Farben lesbar sind, und Optionen, um Wackeln, Blitze und Vibration zu verringern. Elf Sprachen.

Charakter-Sprites, Porträts und Illustrationen entstanden aus Vorlagen, die mit einem Bildmodell von Google Gemini erzeugt, dann in Pixel-Art umgesetzt und von Hand nachbearbeitet wurden.""",
)

T["ru"] = dict(
    short="Милый экшн-рогалайт с видом сверху. Кротёнок Naïl следует за корнями увядающего Великого Древа под землю — парируй в ритм и освободи соседей, опутанных фиолетовыми нитями.",
    headline="Великое Древо увядает. Кротёнок отправляется в путь один.",
    body="""Великое Древо увядает. Фиолетовые нити поползли по его корням и одного за другим опутали подземных жителей, обратив их против всех, кто идёт мимо. Кротёнок Naïl впервые один покидает нору и спускается вслед за корнями.

СРАЖАЙСЯ КАК КРОТ
Серии ударов когтями, которые бьют сильнее в ритм, тяжёлые удары, впечатывающие врагов в стены, уход под землю от атак и таран из-под земли, а парирование отбивает рывки и брошенные снаряды. Каждый враг заранее показывает удар красным: полосы — парируй, круги — уклоняйся.

НОВЫЙ ПУТЬ В КАЖДОМ ЗАБЕГЕ
Выбирай дорогу на ветвящейся карте троп: норы, гнёзда, кладовые, родники и мотки — каждый раз выкопанные заново. Четыре зоны, тем темнее, чем глубже, у каждой свои существа и опасная земля, а в конце каждой тропы ждёт Страж тропы. Собранные светлячки и семена не пропадают, даже если ты падёшь.

РАЗБУДИ СОСЕДЕЙ И ВЕРНИ ИХ ДОМОЙ
Победи опутанного соседа — и нити ослабнут. Крольчиха Момо, белка Дото, броненосец Конги и другие поселятся в деревне под Великим Древом, и Древо снова зацветёт. Разбуди Момо и Дото, чтобы играть и за них — удары задними лапами и дальние прыжки или жёлуди издалека.

КОПАЙ ДАЛЬШЕ
• Таланты Великого Древа: три ветви по восемь ступеней, у каждого героя своё дерево
• Верёвочки 1–10, Сон нитей (повторные бои с боссами), просьбы соседей, цели «Следов», бестиарий и альбом
• Четвёртая тропа, которая открывается после того, как увидишь концовку

ИГРАЙ ПО-СВОЕМУ
Клавиатура или геймпад, переназначаемые кнопки. Режим помощи «Поддержка Попо», предупреждения, понятные и без цвета, и настройки для ослабления тряски, вспышек и вибрации. Одиннадцать языков.

Спрайты персонажей, портреты и иллюстрации сделаны на основе оригиналов, созданных моделью изображений Google Gemini, затем переведены в пиксель-арт и доработаны вручную.""",
)

T["pl"] = dict(
    short="Urocza akcja roguelite z widokiem z góry. Naïl, mały kret, podąża za korzeniami usychającego Wielkiego Drzewa w głąb ziemi — paruj w rytm i uwolnij sąsiadów spętanych fioletowymi nićmi.",
    headline="Wielkie Drzewo usycha. Mały kret wyrusza sam.",
    body="""Wielkie Drzewo usycha. Fioletowe nici wpełzły na jego korzenie i jednego po drugim oplotły podziemnych mieszkańców, zwracając ich przeciw każdemu, kto przechodzi. Naïl, mały kret, po raz pierwszy sam opuszcza norę i schodzi w dół śladem korzeni.

WALCZ JAK KRET
Kombinacje pazurów, które w rytm uderzają mocniej, ciężkie ciosy rzucające wrogów na ściany, zakopywanie się przed atakami i wyskakiwanie z ziemi z taranem, a do tego parowanie, które odbija szarże i pociski. Każdy wróg najpierw zapowiada atak na czerwono — pasy paruj, koła omijaj.

NOWA ŚCIEŻKA W KAŻDYM PODEJŚCIU
Wybieraj drogę na rozgałęzionej mapie ścieżek: nory, gniazda, spiżarnie, źródła i kłębki, za każdym razem wykopane od nowa. Cztery strefy, tym ciemniejsze, im głębiej schodzisz, każda z własnymi stworzeniami i niebezpiecznym podłożem, a na końcu każdej ścieżki czeka Strażnik ścieżki. Zebrane świetliki i nasiona nigdy nie przepadają, gdy padniesz.

OBUDŹ SĄSIADÓW I SPROWADŹ ICH DO DOMU
Pokonaj spętanego sąsiada, a nici się rozluźnią. Królica Momo, wiewiórka Doto, pancernik Kongi i inni zamieszkają w wiosce pod Wielkim Drzewem, a drzewo znów zakwitnie. Obudź Momo i Doto, by grać także nimi — kopniaki tylnymi łapami i dalekie skoki albo żołędzie z dystansu.

KOP DALEJ
• Talenty Wielkiego Drzewa: trzy gałęzie po osiem stopni, osobne drzewo dla każdego bohatera
• Kocia kołyska 1–10, Sen nici (ponowne walki z bossami), prośby sąsiadów, cele Śladów, bestiariusz i album
• Czwarta ścieżka, która otwiera się po zobaczeniu zakończenia

GRAJ PO SWOJEMU
Klawiatura lub pad, przypisywanie przycisków. Tryb pomocy Doping Popo, ostrzeżenia czytelne bez kolorów oraz opcje osłabienia wstrząsów, błysków i wibracji. Jedenaście języków.

Sprite'y postaci, portrety i ilustracje powstały z oryginałów wygenerowanych modelem graficznym Google Gemini, a następnie przerobionych na pixel art i poprawionych ręcznie.""",
)

# (엔티티 종류, 필드) → 글 종류
FIELDS = {
    ("Offer", "description"): "short",
    ("Offer", "aboutItem.0.textBlock.headline"): "headline",
    ("Offer", "aboutItem.0.textBlock.body"): "body",
    ("Product Home", "aboutItem.0.textBlock.headline"): "headline",
    ("Product Home", "aboutItem.0.textBlock.body"): "body",
    ("Store Configs", "shortDescription"): "short",
}


def fill(src: str, dst: str) -> int:
    with open(src, encoding="utf-8-sig", newline="") as f:
        rows = list(csv.reader(f))
    head = rows[0]
    col = {name: i for i, name in enumerate(head)}
    missing = [lang for lang in T if lang not in col]
    assert not missing, f"템플릿에 없는 언어 열: {missing}"
    n = 0
    for r in rows[1:]:
        if len(r) < len(head):
            continue
        kind = FIELDS.get((r[0], r[4]))
        if not kind:
            continue
        for lang, txt in T.items():
            r[col[lang]] = txt[kind]
            n += 1
    with open(dst, "w", encoding="utf-8-sig", newline="") as f:
        w = csv.writer(f, quoting=csv.QUOTE_ALL, lineterminator="\n")
        w.writerows(rows)
    return n


if __name__ == "__main__":
    print(fill(sys.argv[1], sys.argv[2]), "칸 채움")
