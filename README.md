
# Optimalna politika zamene opreme
### SV40/2022

Ovo je generalna verzija projekta. Moguće je da ću tokom implementacije promeniti neke sitne stvari ukoliko bude neophodno.


Optimalna politika zamene opreme predstavlja jedan od problema iz domena dinamičkog programiranja. U nastavku je ukratko opisano:

* šta predstavlja dinamičko programiranje
* opis problema
* opis rešenja
* način implementacije

U ovom konkretnom slučaju, oprema će biti mašine nekog proizvodnog pogona.

## 1. Dinamičko programiranje

Dinamičko programiranje omogućava optimalno planiranje višeetapnih procesa upravljanja. Mnogi zadaci upravljanja procesima u tehnici, ekonomiji, vojsci, fizici, biologiji itd. mogu se predstaviti kao višeetapni procesi, na koje se primenjuje metod dinamičkog programiranja sa ciljem dobijanja optimalnog plana upravljanja.

Za praktičnu primenu dinamičkog programiranja potrebno je da svaki razmatrani proces ima svoj jasno definisani matematički model sa precizno definisanim ciljem (funkcija cilja), koji treba maksimizovati ili minimizovati, kao i ograničenjima koja se moraju uzeti u obzir. U većini slučajeva rešenje se nalazi numeričkim procesima.

## 2. Opis problema

Zamislimo proizvodni pogon koji ima **M mašina** koje rade nezavisno i donose profit, ali koje je potrebno servisirati i koje stare tokom vremena. S tim, raste mogućnost otkaza mašine. U nekim slučajevima mašinu je neophodno prodati i kupiti novu.

Cilj je da se odredi **optimalni trenutak zamene mašine**, a da se maksimizuje ukupna očekivana dobit tokom vremenskog perioda od **N godina**, uzimajući u obzir:

* prihode od rada mašine
* troškove održavanja
* troškove kvarova
* cenu nabavke nove opreme
* preostalu vrednost stare opreme

Svaka mašina *i* poseduje sledeće karakteristike:

* t – starost mašine (godine)
* D(t) – dobit ostvarena korišćenjem mašine stare t godina
* C(t) – godišnji troškovi održavanja mašine stare t godina
* V(t) – preostala vrednost mašine stare t godina
* C – nabavna cena nove mašine
* a(0) – početna starost mašine

Stohastički karakter kvarova:

* p(t) – verovatnoća kvara mašine u zavisnosti od starosti
* C(kvar) – trošak nastao zbog kvara ili prinudne zamene mašine

U svakom vremenskom koraku donosi se odluka o **daljem korišćenju ili zameni mašine**.

Složenost problema serijskim pristupom je **O(S*M*N)**, gde je:

* S – broj Monte Carlo simulacija
* M – broj mašina
* N – broj godina

## 3. Opis rešenja

Cilj je pronaći politiku odlučivanja **u(t)** za svaku mašinu *i* i svaki vremenski korak, tako da se maksimizuje očekivana ukupna dobit sistema.

Pristup koristi **Monte Carlo simulaciju** za stohastičke kvarove i **dinamičko programiranje / pretragu politika zamene** za određivanje optimalnog vremena zamene.

Rezultati uključuju:

* očekivani ukupni profit sistema po godinama
* optimalnu godinu zamene svake mašine
* očekivani godišnji profit po mašini

Opis algoritma:
##### generisanje politike zamene za svaku masinu
Za svaku mašinu se nezavisno procenjuje kada je optimalno zameniti je, poređenjem očekivanog profita pri zadržavanju i zameni.
##### izvršavanje Monte Carlo simulacija
Za unapred definisan broj scenarija simulira se rad mašina tokom N godina pri čemu se u svakom koraku se uzima u obzir stohastički kvar i odluka iz politike zamene. Rezultat je očekivani profit po mašinama i godinama.
##### određivanje optimalne godine zamene po mašinama
Računa se kumulativni profit po mašinama kroz godine. Računa se kumulativni profit po mašinama kroz godine.
##### čuvanje rezultata u CSV formatu
Rezultat se zapisuje u CSV datoteku `machine_optimal_profit.csv` sa kolonama:

  * `machine_id` – identifikator mašine
  * `optimal_replace_year` – optimalna godina zamene
  * `year_1_profit, ..., year_T_profit` – očekivani profit po godinama
## 4. Način implementacije

Rešenje je implementirano u **Python-u i Rust-u** sa sekvencijalnim i paralelnim verzijama. Paralelizacija se oslanja na **nezavisnost Monte Carlo simulacija**.

### Python

#### Sekvencijalna verzija

* Koristi standardni Python sa NumPy za numeričku obradu.
* Iterira kroz unapred definisan broj Monte Carlo simulacija.
* Za svaki scenario simulira rad svih mašina i računa očekivani profit.

#### Paralelna verzija

* Koristi biblioteku `multiprocessing` za izvršavanje simulacija na više jezgara.
* Svaki proces obrađuje deo simulacija, a rezultati se agregiraju za izračunavanje očekivanog profita.

#### Praćenje optimalne politike zamene po mašini

* Za svaku mašinu računa se **optimalna godina zamene** koja maksimizuje kumulativni profit.
* Čuva se **očekivani godišnji profit po godinama po mašini**.

Ovi podaci omogućavaju vizualizaciju i analizu po mašini.

### Rust

#### Sekvencijalna verzija

* Fokus na efikasno upravljanje memorijom i minimalan runtime trošak.
* Striktnom tipizacijom i vektorima postižu se visoke performanse.
* Dinamičko programiranje se implementira kroz funkcije i strukture, a rezultati se čuvaju u CSV.

#### Paralelna verzija

* Koristi Rayon za paralelizaciju simulacija po scenarijima.
* Svaka nit izvršava deo simulacija nezavisno, rezultati se agregiraju.

### Eksperimenti jakog i slabog skaliranja

#### Jako skaliranje (Amdal)

* Fiksna veličina problema, povećava se broj procesora.
* Analizira se smanjenje vremena izvršavanja.

#### Slabo skaliranje (Gustafson)

* Veličina problema proporcionalno raste sa brojem procesora.
* Analizira se očuvanje performansi po procesoru.

### Vizualizacija rešenja

Na osnovu CSV datoteka moguće je kreirati grafike:

1. **Profit po godinama** – linijski graf očekivanog ukupnog profita sistema.
2. **Kumulativni profit** – linijski graf ukupnog profita tokom godina.
3. **Profit po mašini po godinama** – linijski graf za svaku mašinu sa markiranom optimalnom godinom zamene.
4. **Broj zamena po godinama** – bar chart ili stacked bar chart.
5. **Optimalna godina zamene po mašini** – bar chart ili heatmap.

Ove vizualizacije omogućavaju jasan pregled:

* kada je optimalno izvršiti zamenu mašina
* kako pojedinačne mašine doprinose ukupnom profitu
* efekte stohastičkih kvarova i održavanja
