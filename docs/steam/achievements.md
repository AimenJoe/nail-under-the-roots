# Steam 업적 등록표(0276 S2)

`src/achievements.rs`가 만든다 — 손으로 고치지 않는다(`cargo test write_steam_achievement_sheet -- --ignored`).
파트너 사이트 → 앱 관리 → 통계 및 업적 → 업적에 이 차례로 넣는다. API 이름은 한 번 올리면 바꾸지 않는다.

| # | API 이름 | 숨김 | 한국어 | English |
|---|---|---|---|---|
| 1 | `TUTORIAL` |  | **첫 굴** — 포포에게 싸우는 법을 배운다. | **First Burrow** — Learn how to fight from Popo. |
| 2 | `BORI_BACK` |  | **들쥐 대장** — 남쪽 공터의 대장을 이겨 보리를 되찾는다. | **Field Mouse Chief** — Beat the chief of the southern glade and bring Bori back. |
| 3 | `MOMO_FIRST` |  | **그 아이** — 첫 길 끝을 지키는 굴토끼를 이긴다. | **That Child** — Beat the rabbit guarding the end of the first path. |
| 4 | `MOMO_BACK` |  | **꽃이 피다** — 굴토끼를 세 번 이겨 마을로 데려온다. | **A Flower Blooms** — Beat the rabbit three times and bring her to the village. |
| 5 | `DOTO_MEMORY` |  | **뿌리 엉킨 곳** — 둘째 길 끝에서 청설모를 정신 차리게 한다. | **Tangled Roots** — Bring the squirrel at the second path's end back to its senses. |
| 6 | `SUSU_SPROUT` |  | **파는 손** — 첫 쉼터에서 수수에게 새싹 발톱을 받는다. | **Digging Hands** — Get the Sprout Claws from Susu at the first camp. |
| 7 | `KONGI_BACK` |  | **깊은 곳의 문** — 셋째 길 끝에서 아르마딜로를 정신 차리게 한다. | **The Deep Door** — Bring the armadillo at the third path's end back to its senses. |
| 8 | `DEEPEST` |  | **가장 깊은 곳** — 셋째 길 끝, 가장 깊은 곳에 닿는다. | **The Deepest Place** — Reach the deepest place at the end of the third path. |
| 9 | `CLOVER_FAMILY` | 예 | **가족** — 굴토끼 옛 굴의 끝에서 토끼풀과 아기들을 깨운다. | **Family** — Wake Clover and her little ones at the end of the old rabbit warren. |
| 10 | `ALL_BACK` | 예 | **모두 돌아왔다** — 실에 감겼던 이웃 다섯이 모두 마을로 돌아온다. | **Everyone's Home** — Bring all five neighbors who were bound by the threads back to the village. |
| 11 | `TRUE_ENDING` | 예 | **마지막 꽃** — 잊힌 씨앗을 가장 깊은 곳에 심는다. | **The Last Bloom** — Plant the forgotten seed in the deepest place. |
| 12 | `LAST_MEMORY` | 예 | **실의 기억** — 실의 기억 여섯 조각을 모두 본다. | **Threads of Memory** — See all six threads of memory. |
| 13 | `SIDE_PATH` | 예 | **옆 굴** — 가장 깊은 곳 앞에서 다른 길을 골라 물길 굴로 내려간다. | **The Side Burrow** — At the door of the deepest place, take the other way down into the Waterway Burrow. |
| 14 | `MUMU_FREED` | 예 | **물길지기** — 물길 굴 끝에서 무무의 실을 풀어 준다. | **The Waterway Keeper** — Free Mumu from the threads at the end of the Waterway Burrow. |
| 15 | `MOMO_DEEP` | 예 | **꽃잎을 따라** — 모모와 함께 가장 깊은 곳에 닿는다. | **Following the Petal** — Reach the deepest place with Momo. |
| 16 | `DOTO_DEEP` | 예 | **묻어 둔 길** — 도토와 함께 가장 깊은 곳에 닿는다. | **The Buried Way** — Reach the deepest place with Doto. |
| 17 | `KILLS` |  | **땅속 사냥꾼** — 물든 이웃 200마리를 쓰러뜨린다. | **Underground Hunter** — Defeat 200 tainted neighbors. |
| 18 | `AMBUSH` |  | **땅울림** — 솟구치기로 50마리를 쓰러뜨린다. | **Ground Rumble** — Defeat 50 with a burst out of the ground. |
| 19 | `PARRY` |  | **빛나는 발톱** — 패링을 60번 한다. | **Shining Claws** — Parry 60 times. |
| 20 | `HEAVY` |  | **땅을 울리는 발톱** — 강공격을 150번 맞힌다. | **Earth-Shaking Claws** — Land 150 heavy strikes. |
| 21 | `FINISHER` |  | **마지막 타** — 발톱 마지막 타를 100번 맞힌다. | **Final Blow** — Land the last claw hit 100 times. |
| 22 | `BOWLING` |  | **스트라이크!** — 모모로 날려 보낸 적을 다른 적에 20번 부딪힌다. | **Strike!** — As Momo, send a foe flying into another foe 20 times. |
| 23 | `CHESTS` |  | **저장고 털이** — 상자를 25개 연다. | **Pantry Raider** — Open 25 chests. |
| 24 | `BREAKER` |  | **와장창** — 굴에서 부술 수 있는 것을 30개 부순다. | **Crash and Smash** — Break 30 breakable things in the burrows. |
| 25 | `FALLS` |  | **쓰러져도** — 10번 쓰러지고 다시 일어난다. | **Even If I Fall** — Fall 10 times and get back up. |
| 26 | `BESTIARY` |  | **낯익은 얼굴들** — 도감에 이웃 10종을 적는다. | **Familiar Faces** — Note 10 kinds of neighbors in the bestiary. |
| 27 | `TALENTS` |  | **자라는 재주** — 재주 넷을 깨운다. | **Growing Skills** — Awaken four skills. |
| 28 | `DREAM_ALL` | 예 | **끝나지 않는 실** — 넷을 이어서 꾸는 꿈을 끝까지 깬다. | **Endless Thread** — Clear the dream of all four in a row. |
| 29 | `DECOR` |  | **북적이는 마을** — 포포의 꾸미기 6개를 다 산다. | **Bustling Village** — Buy all 6 of Popo's decorations. |
| 30 | `KNOTS` |  | **첫 매듭** — 실뜨기 1단계로 가장 깊은 곳에 닿는다. | **First Knot** — Reach the deepest place on Cat's Cradle level 1. |

## 모든 언어

### `TUTORIAL`
- korean: **첫 굴** — 포포에게 싸우는 법을 배운다.
- english: **First Burrow** — Learn how to fight from Popo.
- japanese: **はじめての穴** — ポポからたたかい方をおそわる。
- schinese: **第一个洞穴** — 跟波波学习战斗。
- tchinese: **第一個洞穴** — 跟波波學習戰鬥。
- brazilian: **Primeira toca** — Aprenda a lutar com o Popo.
- french: **Premier terrier** — Apprends à te battre avec Popo.
- spanish: **Primera madriguera** — Aprende a pelear con Popo.
- german: **Erster Bau** — Lerne bei Popo das Kämpfen.
- russian: **Первая нора** — Научись драться у Попо.
- polish: **Pierwsza nora** — Naucz się walczyć od Popo.

### `BORI_BACK`
- korean: **들쥐 대장** — 남쪽 공터의 대장을 이겨 보리를 되찾는다.
- english: **Field Mouse Chief** — Beat the chief of the southern glade and bring Bori back.
- japanese: **ノネズミの親分** — 南の空き地の親分をやっつけて、ボリをとりもどす。
- schinese: **田鼠首领** — 打败南边空地的首领，把波利找回来。
- tchinese: **田鼠首領** — 打敗南邊空地的首領，把波利找回來。
- brazilian: **Chefe Rato-do-campo** — Vença o chefe da clareira do sul e traga o Bori de volta.
- french: **Chef des mulots** — Bats le chef de la clairière du sud et ramène Bori à la raison.
- spanish: **Jefe Ratón de Campo** — Vence al jefe del claro del sur y recupera a Bori.
- german: **Mäusehäuptling** — Besiege den Häuptling der Lichtung im Süden und hol Bori zurück.
- russian: **Староста полёвок** — Победи старосту южной поляны и верни Бори.
- polish: **Wódz myszy polnych** — Pokonaj wodza z południowej polany i sprowadź Bori z powrotem.

### `MOMO_FIRST`
- korean: **그 아이** — 첫 길 끝을 지키는 굴토끼를 이긴다.
- english: **That Child** — Beat the rabbit guarding the end of the first path.
- japanese: **あの子** — 最初の道の先を守るアナウサギをやっつける。
- schinese: **那个孩子** — 打败守在第一条路尽头的穴兔。
- tchinese: **那個孩子** — 打敗守在第一條路盡頭的穴兔。
- brazilian: **Aquela menina** — Vença a coelha que guarda o fim da primeira trilha.
- french: **Cette petite** — Bats la lapine qui garde le bout du premier chemin.
- spanish: **Esa niña** — Vence a la coneja que guarda el final del primer camino.
- german: **Die Kleine** — Besiege die Häsin, die das Ende des ersten Pfads bewacht.
- russian: **Та малышка** — Победи крольчиху, стерегущую конец первой тропы.
- polish: **Tamta mała** — Pokonaj królicę strzegącą końca pierwszej ścieżki.

### `MOMO_BACK`
- korean: **꽃이 피다** — 굴토끼를 세 번 이겨 마을로 데려온다.
- english: **A Flower Blooms** — Beat the rabbit three times and bring her to the village.
- japanese: **花がひらく** — アナウサギを3回やっつけて、村へつれてくる。
- schinese: **花开了** — 打败穴兔三次，把她带回村子。
- tchinese: **花開了** — 打敗穴兔三次，把她帶回村子。
- brazilian: **A flor desabrocha** — Vença a coelha três vezes e traga-a para a vila.
- french: **Une fleur s'épanouit** — Bats la lapine trois fois et ramène-la au village.
- spanish: **La flor se abre** — Vence tres veces a la coneja y tráela a la aldea.
- german: **Eine Blume blüht** — Besiege die Häsin dreimal und bring sie ins Dorf.
- russian: **Цветок распустился** — Победи крольчиху трижды и приведи её в деревню.
- polish: **Kwiat rozkwita** — Pokonaj królicę trzy razy i przyprowadź ją do wioski.

### `DOTO_MEMORY`
- korean: **뿌리 엉킨 곳** — 둘째 길 끝에서 청설모를 정신 차리게 한다.
- english: **Tangled Roots** — Bring the squirrel at the second path's end back to its senses.
- japanese: **根のからまる所** — 2本目の道の先で、キタリスの目をさまさせる。
- schinese: **树根缠结处** — 在第二条路的尽头，让松鼠清醒过来。
- tchinese: **樹根纏結處** — 在第二條路的盡頭，讓松鼠清醒過來。
- brazilian: **Raízes emaranhadas** — Faça o esquilo no fim da segunda trilha voltar a si.
- french: **Racines emmêlées** — Ramène à la raison l'écureuil du bout du deuxième chemin.
- spanish: **Raíces enredadas** — Haz que la ardilla del final del segundo camino vuelva en sí.
- german: **Wurzelgewirr** — Bring das Eichhörnchen am Ende des zweiten Pfads wieder zur Vernunft.
- russian: **Спутанные корни** — Приведи в чувство белку в конце второй тропы.
- polish: **Splątane korzenie** — Przywróć zmysły wiewiórce z końca drugiej ścieżki.

### `SUSU_SPROUT`
- korean: **파는 손** — 첫 쉼터에서 수수에게 새싹 발톱을 받는다.
- english: **Digging Hands** — Get the Sprout Claws from Susu at the first camp.
- japanese: **ほる手** — 最初の休み場で、ススから新芽のツメをもらう。
- schinese: **挖土的手** — 在第一个营地，从苏苏那里得到嫩芽之爪。
- tchinese: **挖土的手** — 在第一個營地，從蘇蘇那裡得到嫩芽之爪。
- brazilian: **Mãos de cavar** — Receba as Garras de broto da Susu no primeiro refúgio.
- french: **Mains qui creusent** — Reçois les Griffes de pousse de Susu au premier refuge.
- spanish: **Manos para cavar** — Recibe las Garras de brote de Susu en el primer refugio.
- german: **Grabende Hände** — Hol dir im ersten Lager bei Susu die Sprosskrallen.
- russian: **Копающие лапки** — Получи когти-ростки у Сусу на первом привале.
- polish: **Łapki do kopania** — Odbierz kiełkowe pazury od Susu w pierwszym obozie.

### `KONGI_BACK`
- korean: **깊은 곳의 문** — 셋째 길 끝에서 아르마딜로를 정신 차리게 한다.
- english: **The Deep Door** — Bring the armadillo at the third path's end back to its senses.
- japanese: **深いところの門** — 3本目の道の先で、アルマジロの目をさまさせる。
- schinese: **深处之门** — 在第三条路的尽头，让犰狳清醒过来。
- tchinese: **深處之門** — 在第三條路的盡頭，讓犰狳清醒過來。
- brazilian: **A porta do fundo** — Faça o tatu no fim da terceira trilha voltar a si.
- french: **La porte du fond** — Ramène à la raison le tatou du bout du troisième chemin.
- spanish: **La puerta honda** — Haz que el armadillo del final del tercer camino vuelva en sí.
- german: **Die tiefe Tür** — Bring das Gürteltier am Ende des dritten Pfads wieder zur Vernunft.
- russian: **Глубинная дверь** — Приведи в чувство броненосца в конце третьей тропы.
- polish: **Głębokie drzwi** — Przywróć zmysły pancernikowi z końca trzeciej ścieżki.

### `DEEPEST`
- korean: **가장 깊은 곳** — 셋째 길 끝, 가장 깊은 곳에 닿는다.
- english: **The Deepest Place** — Reach the deepest place at the end of the third path.
- japanese: **いちばん深いところ** — 3本目の道の先、いちばん深いところに着く。
- schinese: **最深处** — 在第三条路的尽头，到达最深处。
- tchinese: **最深處** — 在第三條路的盡頭，到達最深處。
- brazilian: **O lugar mais fundo** — Chegue ao lugar mais fundo, no fim da terceira trilha.
- french: **Le Tréfonds** — Atteins le Tréfonds, au bout du troisième chemin.
- spanish: **Lo más profundo** — Llega a lo más profundo, al final del tercer camino.
- german: **Der tiefste Grund** — Erreiche den tiefsten Grund am Ende des dritten Pfads.
- russian: **Самая глубь** — Дойди до самой глуби в конце третьей тропы.
- polish: **Najgłębsze miejsce** — Dotrzyj do najgłębszego miejsca na końcu trzeciej ścieżki.

### `CLOVER_FAMILY`
- korean: **가족** — 굴토끼 옛 굴의 끝에서 토끼풀과 아기들을 깨운다.
- english: **Family** — Wake Clover and her little ones at the end of the old rabbit warren.
- japanese: **家族** — アナウサギの古い巣穴の奥で、クローバーと子どもたちを起こす。
- schinese: **家人** — 在穴兔旧洞的尽头唤醒三叶草和孩子们。
- tchinese: **家人** — 在穴兔舊洞的盡頭喚醒三葉草和孩子們。
- brazilian: **Família** — Acorde Trevo e seus filhotes no fim da antiga toca dos coelhos.
- french: **Famille** — Réveille Trèfle et ses petits au bout du vieux terrier des lapins.
- spanish: **Familia** — Despierta a Trébol y a sus crías al final de la vieja madriguera de los conejos.
- german: **Familie** — Wecke Klee und ihre Kleinen am Ende des alten Kaninchenbaus.
- russian: **Семья** — Разбуди Клевер и её малышей в конце старой кроличьей норы.
- polish: **Rodzina** — Obudź Koniczynkę i jej maluchy na końcu starej króliczej nory.

### `ALL_BACK`
- korean: **모두 돌아왔다** — 실에 감겼던 이웃 다섯이 모두 마을로 돌아온다.
- english: **Everyone's Home** — Bring all five neighbors who were bound by the threads back to the village.
- japanese: **みんな帰ってきた** — 糸にからまれていたご近所さん5人が、みんな村に帰ってくる。
- schinese: **大家都回来了** — 让曾被丝线缠住的五位邻居全部回到村子。
- tchinese: **大家都回來了** — 讓曾被絲線纏住的五位鄰居全部回到村子。
- brazilian: **Todos em casa** — Traga de volta à vila os cinco vizinhos que estavam presos pelos fios.
- french: **Tous rentrés** — Ramène au village les cinq voisins qui étaient pris dans les fils.
- spanish: **Todos en casa** — Trae de vuelta a la aldea a los cinco vecinos que estaban atados por los hilos.
- german: **Alle daheim** — Bring alle fünf Nachbarn, die von den Fäden gefangen waren, ins Dorf zurück.
- russian: **Все дома** — Верни в деревню всех пятерых соседей, опутанных нитями.
- polish: **Wszyscy w domu** — Sprowadź do wioski wszystkich pięciu sąsiadów spętanych nićmi.

### `TRUE_ENDING`
- korean: **마지막 꽃** — 잊힌 씨앗을 가장 깊은 곳에 심는다.
- english: **The Last Bloom** — Plant the forgotten seed in the deepest place.
- japanese: **さいごの花** — 忘れられた種を、いちばん深いところに植える。
- schinese: **最后的花** — 把被遗忘的种子种在最深处。
- tchinese: **最後的花** — 把被遺忘的種子種在最深處。
- brazilian: **A última flor** — Plante a semente esquecida no lugar mais fundo.
- french: **Ultime floraison** — Plante la graine oubliée au Tréfonds.
- spanish: **La última flor** — Planta la semilla olvidada en lo más profundo.
- german: **Die letzte Blüte** — Pflanze den vergessenen Samen am tiefsten Grund.
- russian: **Последнее цветение** — Посади забытое семечко в самой глуби.
- polish: **Ostatnie kwitnienie** — Zasadź zapomniane nasionko w najgłębszym miejscu.

### `LAST_MEMORY`
- korean: **실의 기억** — 실의 기억 여섯 조각을 모두 본다.
- english: **Threads of Memory** — See all six threads of memory.
- japanese: **糸の記憶** — 糸の記憶を6つ、ぜんぶ見る。
- schinese: **丝线的记忆** — 看完全部六段丝线的记忆。
- tchinese: **絲線的記憶** — 看完全部六段絲線的記憶。
- brazilian: **Fios da memória** — Veja todos os seis fios da memória.
- french: **Fils de mémoire** — Voir les six fils de mémoire.
- spanish: **Hilos de memoria** — Ve los seis hilos de memoria.
- german: **Fäden der Erinnerung** — Sieh alle sechs Fäden der Erinnerung.
- russian: **Нити памяти** — Увидеть все шесть нитей памяти.
- polish: **Nici pamięci** — Zobacz wszystkie sześć nici pamięci.

### `SIDE_PATH`
- korean: **옆 굴** — 가장 깊은 곳 앞에서 다른 길을 골라 물길 굴로 내려간다.
- english: **The Side Burrow** — At the door of the deepest place, take the other way down into the Waterway Burrow.
- japanese: **となりの穴** — いちばん深い場所の手前で別の道をえらび、水みちの巣穴へおりる。
- schinese: **旁边的洞** — 在最深处前选另一条路，下到水路洞穴。
- tchinese: **旁邊的洞** — 在最深處前選另一條路，下到水路洞穴。
- brazilian: **A toca ao lado** — Diante do lugar mais fundo, escolha o outro caminho e desça à Toca do córrego.
- french: **Le terrier d'à côté** — Devant le Tréfonds, prends l'autre chemin et descends dans le Terrier des eaux.
- spanish: **La madriguera de al lado** — Ante lo más profundo, toma el otro camino y baja a la Madriguera del arroyo.
- german: **Der Nebenbau** — Nimm vor dem tiefsten Grund den anderen Weg hinab in den Bachbau.
- russian: **Соседняя нора** — Перед самой глубью выбери другой путь и спустись в нору у ручья.
- polish: **Nora obok** — Przed najgłębszym miejscem wybierz drugą drogę i zejdź do nory strumienia.

### `MUMU_FREED`
- korean: **물길지기** — 물길 굴 끝에서 무무의 실을 풀어 준다.
- english: **The Waterway Keeper** — Free Mumu from the threads at the end of the Waterway Burrow.
- japanese: **水みちの番人** — 水みちの巣穴の奥で、ムムの糸をほどく。
- schinese: **水道守护者** — 在水路洞穴的尽头解开沐沐身上的丝。
- tchinese: **水道守護者** — 在水路洞穴的盡頭解開沐沐身上的絲。
- brazilian: **Quem guarda o córrego** — Solte Mumu dos fios no fim da Toca do córrego.
- french: **Les eaux gardées** — Libère Mumu de ses fils au bout du Terrier des eaux.
- spanish: **Quien guarda el arroyo** — Libera a Mumu de los hilos al final de la Madriguera del arroyo.
- german: **Das gehütete Wasser** — Befreie Mumu am Ende des Bachbaus von den Fäden.
- russian: **Страж ручья** — Освободи Муму от нитей в конце норы у ручья.
- polish: **Strzeżony strumień** — Uwolnij Mumu od nici na końcu nory strumienia.

### `MOMO_DEEP`
- korean: **꽃잎을 따라** — 모모와 함께 가장 깊은 곳에 닿는다.
- english: **Following the Petal** — Reach the deepest place with Momo.
- japanese: **花びらをたどって** — モモといっしょに、いちばん深い場所にたどりつく。
- schinese: **追随花瓣** — 和茉茉一起到达最深处。
- tchinese: **追隨花瓣** — 和茉茉一起到達最深處。
- brazilian: **Seguindo a pétala** — Chegue ao lugar mais profundo com a Momo.
- french: **Suivre le pétale** — Atteins le Tréfonds avec Momo.
- spanish: **Siguiendo el pétalo** — Llega a lo más profundo con Momo.
- german: **Dem Blütenblatt nach** — Erreiche mit Momo den tiefsten Grund.
- russian: **По следу лепестка** — Дойди до самой глуби с Момо.
- polish: **Śladem płatka** — Dotrzyj z Momo do najgłębszego miejsca.

### `DOTO_DEEP`
- korean: **묻어 둔 길** — 도토와 함께 가장 깊은 곳에 닿는다.
- english: **The Buried Way** — Reach the deepest place with Doto.
- japanese: **うめておいた道** — ドトといっしょに、いちばん深い場所にたどりつく。
- schinese: **埋下的路** — 和多托一起到达最深处。
- tchinese: **埋下的路** — 和多托一起到達最深處。
- brazilian: **O caminho enterrado** — Chegue ao lugar mais profundo com o Doto.
- french: **Le chemin enterré** — Atteins le Tréfonds avec Doto.
- spanish: **El camino enterrado** — Llega a lo más profundo con Doto.
- german: **Der vergrabene Weg** — Erreiche mit Doto den tiefsten Grund.
- russian: **Закопанный путь** — Дойди до самой глуби с Дото.
- polish: **Zakopana droga** — Dotrzyj z Doto do najgłębszego miejsca.

### `KILLS`
- korean: **땅속 사냥꾼** — 물든 이웃 200마리를 쓰러뜨린다.
- english: **Underground Hunter** — Defeat 200 tainted neighbors.
- japanese: **地中のかりうど** — 染まった近所を200ぴきたおす。
- schinese: **地下猎手** — 打倒200只被染上的邻居。
- tchinese: **地下獵手** — 打倒200隻被染上的鄰居。
- brazilian: **Caçador subterrâneo** — Derrote 200 vizinhos contaminados.
- french: **Chasse souterraine** — Bats 200 voisins corrompus.
- spanish: **Caza subterránea** — Derrota a 200 vecinos contaminados.
- german: **Jagd unter Tage** — Besiege 200 verhexte Nachbarn.
- russian: **Подземная охота** — Победи 200 опутанных соседей.
- polish: **Podziemne łowy** — Pokonaj 200 splątanych sąsiadów.

### `AMBUSH`
- korean: **땅울림** — 솟구치기로 50마리를 쓰러뜨린다.
- english: **Ground Rumble** — Defeat 50 with a burst out of the ground.
- japanese: **じなり** — とびだしで50ぴきたおす。
- schinese: **地鸣** — 用冲出打倒50只。
- tchinese: **地鳴** — 用衝出打倒50隻。
- brazilian: **Tremor** — Derrote 50 emergindo da terra.
- french: **Terre qui gronde** — Bats-en 50 en jaillissant du sol.
- spanish: **Temblor de tierra** — Derrota a 50 emergiendo del suelo.
- german: **Grollende Erde** — Besiege 50 mit einem Ausbruch aus der Erde.
- russian: **Гул земли** — Победи 50 врагов, выпрыгнув из-под земли.
- polish: **Dudnienie ziemi** — Pokonaj 50 wrogów, wyskakując spod ziemi.

### `PARRY`
- korean: **빛나는 발톱** — 패링을 60번 한다.
- english: **Shining Claws** — Parry 60 times.
- japanese: **光るつめ** — はじきを60回する。
- schinese: **闪亮的爪子** — 弹开60次。
- tchinese: **閃亮的爪子** — 彈開60次。
- brazilian: **Garras brilhantes** — Apare 60 vezes.
- french: **Griffes brillantes** — Pare 60 fois.
- spanish: **Garras brillantes** — Desvía 60 veces.
- german: **Glänzende Krallen** — Pariere 60-mal.
- russian: **Сияющие когти** — Парируй 60 раз.
- polish: **Lśniące pazury** — Sparuj 60 razy.

### `HEAVY`
- korean: **땅을 울리는 발톱** — 강공격을 150번 맞힌다.
- english: **Earth-Shaking Claws** — Land 150 heavy strikes.
- japanese: **地をゆらすツメ** — 強いこうげきを150回当てる。
- schinese: **重击之爪** — 命中150次重击。
- tchinese: **重擊之爪** — 命中150次重擊。
- brazilian: **Garras que abalam a terra** — Acerte 150 golpes pesados.
- french: **Griffes qui ébranlent la terre** — Réussis 150 coups lourds.
- spanish: **Garras que sacuden la tierra** — Acierta 150 golpes fuertes.
- german: **Erdbeben-Krallen** — Triff mit 150 schweren Hieben.
- russian: **Когти, сотрясающие землю** — Попади 150 тяжёлыми ударами.
- polish: **Pazury wstrząsające ziemią** — Traf 150 ciężkimi ciosami.

### `FINISHER`
- korean: **마지막 타** — 발톱 마지막 타를 100번 맞힌다.
- english: **Final Blow** — Land the last claw hit 100 times.
- japanese: **さいごの一打** — つめのさいごの打を100回当てる。
- schinese: **最后一下** — 爪子的最后一下命中100次。
- tchinese: **最後一下** — 爪子的最後一下命中100次。
- brazilian: **Golpe final** — Acerte o último golpe da garra 100 vezes.
- french: **Coup de grâce** — Place le dernier coup de griffe 100 fois.
- spanish: **Golpe final** — Acierta el último golpe de garra 100 veces.
- german: **Letzter Hieb** — Lande den letzten Krallenhieb 100-mal.
- russian: **Последний удар** — Нанеси добивающий удар когтями 100 раз.
- polish: **Ostatni cios** — Zadaj ostatni cios pazurami 100 razy.

### `BOWLING`
- korean: **스트라이크!** — 모모로 날려 보낸 적을 다른 적에 20번 부딪힌다.
- english: **Strike!** — As Momo, send a foe flying into another foe 20 times.
- japanese: **ストライク！** — モモでふっとばした敵を、ほかの敵に20回ぶつける。
- schinese: **全中！** — 用茉茉把击飞的敌人撞向其他敌人20次。
- tchinese: **全中！** — 用茉茉把擊飛的敵人撞向其他敵人20次。
- brazilian: **Strike!** — Com a Momo, lance um inimigo contra outro 20 vezes.
- french: **Strike !** — Avec Momo, projette un ennemi sur un autre 20 fois.
- spanish: **¡Pleno!** — Con Momo, lanza a un enemigo contra otro 20 veces.
- german: **Alle neune!** — Schleudere als Momo 20-mal einen Gegner in einen anderen.
- russian: **Страйк!** — За Момо 20 раз запусти врага в другого врага.
- polish: **Wszystkie kręgle!** — Jako Momo wybij wroga w innego wroga 20 razy.

### `CHESTS`
- korean: **저장고 털이** — 상자를 25개 연다.
- english: **Pantry Raider** — Open 25 chests.
- japanese: **くらあらし** — 箱を25こあける。
- schinese: **清空储藏室** — 打开25个箱子。
- tchinese: **清空儲藏室** — 打開25個箱子。
- brazilian: **Saque na despensa** — Abra 25 baús.
- french: **Pillage de réserve** — Ouvre 25 coffres.
- spanish: **Asalto a la despensa** — Abre 25 cofres.
- german: **Vorräte plündern** — Öffne 25 Truhen.
- russian: **Набег на кладовые** — Открой 25 сундуков.
- polish: **Najazd na spiżarnie** — Otwórz 25 skrzyń.

### `BREAKER`
- korean: **와장창** — 굴에서 부술 수 있는 것을 30개 부순다.
- english: **Crash and Smash** — Break 30 breakable things in the burrows.
- japanese: **ガッシャーン** — 巣穴でこわせるものを30こ こわす。
- schinese: **打碎它们** — 在洞穴里打碎30个可以打碎的东西。
- tchinese: **打碎它們** — 在洞穴裡打碎30個可以打碎的東西。
- brazilian: **Quebra-quebra** — Quebre 30 coisas nas tocas.
- french: **Casse-tout** — Casse 30 choses dans les terriers.
- spanish: **Rompelotodo** — Rompe 30 cosas en las madrigueras.
- german: **Scherbenhaufen** — Zerbrich 30 Dinge in den Bauen.
- russian: **Вдребезги** — Разбей 30 вещей в норах.
- polish: **Trzask-prask** — Rozbij 30 rzeczy w norach.

### `FALLS`
- korean: **쓰러져도** — 10번 쓰러지고 다시 일어난다.
- english: **Even If I Fall** — Fall 10 times and get back up.
- japanese: **倒れても** — 10回倒れて、また立ちあがる。
- schinese: **就算倒下** — 倒下10次，又站起来。
- tchinese: **就算倒下** — 倒下10次，又站起來。
- brazilian: **Mesmo caindo** — Caia 10 vezes e levante de novo.
- french: **Même si je tombe** — Tombe 10 fois et relève-toi.
- spanish: **Aunque caiga** — Cae 10 veces y vuelve a levantarte.
- german: **Auch wenn ich falle** — Fall 10-mal hin und steh wieder auf.
- russian: **Даже если упаду** — Упади и поднимись 10 раз.
- polish: **Nawet jeśli upadnę** — Upadnij i wstań 10 razy.

### `BESTIARY`
- korean: **낯익은 얼굴들** — 도감에 이웃 10종을 적는다.
- english: **Familiar Faces** — Note 10 kinds of neighbors in the bestiary.
- japanese: **見なれた顔** — ずかんに近所を10種書く。
- schinese: **认识的邻居** — 在图鉴里记下10种邻居。
- tchinese: **認識的鄰居** — 在圖鑑裡記下10種鄰居。
- brazilian: **Rostos conhecidos** — Anote 10 tipos de vizinhos no bestiário.
- french: **Visages familiers** — Note 10 sortes de voisins dans le bestiaire.
- spanish: **Caras conocidas** — Anota 10 tipos de vecinos en el bestiario.
- german: **Bekannte Gesichter** — Trag 10 Arten von Nachbarn ins Tierbuch ein.
- russian: **Знакомые лица** — Запиши в бестиарий 10 видов соседей.
- polish: **Znajome twarze** — Zapisz w bestiariuszu 10 rodzajów sąsiadów.

### `TALENTS`
- korean: **자라는 재주** — 재주 넷을 깨운다.
- english: **Growing Skills** — Awaken four skills.
- japanese: **育つわざ** — わざを4つ起こす。
- schinese: **成长的本领** — 唤醒四项本领。
- tchinese: **成長的本領** — 喚醒四項本領。
- brazilian: **Habilidades crescendo** — Desperte quatro habilidades.
- french: **Talents en herbe** — Éveille quatre talents.
- spanish: **Habilidades que crecen** — Despierta cuatro habilidades.
- german: **Wachsende Talente** — Erwecke vier Talente.
- russian: **Умения растут** — Пробуди четыре умения.
- polish: **Zdolności rosną** — Obudź cztery zdolności.

### `DREAM_ALL`
- korean: **끝나지 않는 실** — 넷을 이어서 꾸는 꿈을 끝까지 깬다.
- english: **Endless Thread** — Clear the dream of all four in a row.
- japanese: **終わらない糸** — 四つをつづけて見る夢を最後までやりとげる。
- schinese: **不断的丝** — 把连续四场的梦做到最后。
- tchinese: **不斷的絲** — 把連續四場的夢做到最後。
- brazilian: **Fio sem fim** — Complete o sonho dos quatro seguidos.
- french: **Fil sans fin** — Termine le rêve des quatre d'affilée.
- spanish: **Hilo sin fin** — Supera el sueño de los cuatro seguidos.
- german: **Endloser Faden** — Schaffe den Traum aller vier am Stück.
- russian: **Бесконечная нить** — Пройди сон всех четырёх подряд.
- polish: **Nić bez końca** — Przejdź sen całej czwórki pod rząd.

### `DECOR`
- korean: **북적이는 마을** — 포포의 꾸미기 6개를 다 산다.
- english: **Bustling Village** — Buy all 6 of Popo's decorations.
- japanese: **にぎやかな村** — ポポのかざり6つをぜんぶ買う。
- schinese: **欢乐的村子** — 买下波波的全部6件装饰。
- tchinese: **歡樂的村子** — 買下波波的全部6件裝飾。
- brazilian: **Vila animada** — Compre os 6 enfeites do Popo.
- french: **Village animé** — Achète les 6 décorations de Popo.
- spanish: **Aldea animada** — Compra los 6 adornos de Popo.
- german: **Lebhaftes Dorf** — Kauf alle 6 Dekos von Popo.
- russian: **Живая деревня** — Купи у Попо все 6 украшений.
- polish: **Gwarna wioska** — Kup wszystkie 6 ozdób Popo.

### `KNOTS`
- korean: **첫 매듭** — 실뜨기 1단계로 가장 깊은 곳에 닿는다.
- english: **First Knot** — Reach the deepest place on Cat's Cradle level 1.
- japanese: **はじめての結び目** — あやとり1段で、いちばん深いところに着く。
- schinese: **第一个结** — 在翻花绳第1级到达最深处。
- tchinese: **第一個結** — 在翻花繩第1級到達最深處。
- brazilian: **Primeiro nó** — Chegue ao lugar mais fundo na Cama de gato nível 1.
- french: **Premier nœud** — Atteins le Tréfonds au niveau 1 du jeu de ficelle.
- spanish: **Primer nudo** — Llega a lo más profundo en el nivel 1 de Cuna de gato.
- german: **Erster Knoten** — Erreiche den tiefsten Grund auf Stufe 1 des Fadenspiels.
- russian: **Первый узелок** — Дойди до самой глуби на 1-м уровне верёвочек.
- polish: **Pierwszy supełek** — Dotrzyj do najgłębszego miejsca na 1. poziomie kociej kołyski.
