# Pulp-cover prompts for the ten tomi

One image-generation prompt per Tomus of the facsimile reader. Each cover is a
1930s–50s pulp-magazine montage of scenes taken from the Lives bound in that
Tomus. Every scene is cited to Plutarch in Bernadotte Perrin's English
translation (Loeb Classical Library, 1914–1926), so an English-only illustrator
can read the passage the picture is drawn from.

## How to use this file

- **The prompt** for each Tomus is the fenced block under "Prompt". Paste it
  whole. It names its own style references.
- **The scene sources** table under each prompt is for the illustrator. Each row
  gives the Life, Plutarch's chapter and section (e.g. *Theseus* 19.1), a short
  quotation from Perrin's translation, and a link to that passage. The
  chapter.section numbers are the same in the printed Loeb, on Bill Thayer's
  LacusCurtius site (<https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Plutarch/Lives/home.html>),
  and on Perseus/Scaife, so any copy of Perrin will do.
- **Links.** Each citation links to the passage in the Scaife viewer by its
  Perseus CTS identifier. Every link was checked to open the cited passage and
  every quotation was checked against it. Scaife shows Perrin's footnotes inline
  in a few passages, so a quotation may be interrupted there by a note. The same
  text is in Perseus's TEI file for each Life on GitHub (listed in
  [Sources](#sources)).
- **Reference images.** Image generators generally cannot open web pages. Open
  each linked Syracuse item page, download the cover image, and attach it to the
  generation request as a style reference. Give the illustrator the same pages.
- **Lettering.** Generators garble text. Each prompt asks for a plain colour band
  across the top for the masthead, to be lettered in layout. An optional
  lettering line is included for generators that can render text.
- **Shape.** The book trims to 10.5 × 16 cm (about 2:3 portrait), and the sample
  dust-cover front panel in `tools/README-dust-cover.md` is 5.5 × 9.5 in. Generate
  at 2:3 portrait and keep faces and key action out of the outer 5% so the image
  can be cropped to either.

## Which Lives are in each Tomus

The repository does not record the split; it is computed at build time by
`split_into_volumes` (`src/volumes.rs`) with a 320-page target and a 384-page
cap. The table below comes from running that same rule over the per-unit page
counts of the most recent inner-tomi build on disk
(`tmp/pdfs/inner_review/body_map.csv`). The same counts with the old 400/500
limits reproduce the old eight-tomus split exactly, and the ten-tomus result
does not change when every unit is moved by ±1 page, so it should match a
fresh build. Check the title page of each `tomus_N_inner.pdf` before
committing to art.

| Tomus | Lives (English name, with the reader's running title in brackets where it differs) |
|---|---|
| I | Theseus, Romulus, Lycurgus, Numa, Solon, Publicola [POPLICOLA] |
| II | Themistocles, Camillus, Pericles, Fabius Maximus, Alcibiades, Coriolanus [MARCIUS CORIOLANUS] |
| III | Timoleon, Aemilius Paulus, Pelopidas, Marcellus, Aristides, Cato the Elder [CATO MAJOR] |
| IV | Philopoemen, Flamininus [T. QUINCTIUS FLAMININUS], Pyrrhus, Marius [C. MARIUS], Lysander, Sulla [SYLLA] |
| V | Cimon, Lucullus |
| VI | Nicias, Crassus, Sertorius, Eumenes, Agesilaus, Pompey [POMPEIUS] |
| VII | Alexander, Julius Caesar [C. JULIUS CAESAR], Phocion, Cato the Younger [CATO MINOR] |
| VIII | Agis, Cleomenes, Tiberius Gracchus [TIB. GRACCHUS], Caius Gracchus [C. GRACCHUS], Demosthenes, Cicero |
| IX | Demetrius, Antony [ANTONIUS], Dion [DIO], Brutus |
| X | Artaxerxes [ARTOXERXES], Aratus, Galba, Otho |

## Style references from the Syracuse Pulp Covers Collection

The [Pulp Covers Collection](https://digitalcollections.syr.edu/pulp-covers-collection)
at Syracuse University Libraries holds cover scans of dime novels, pulp
magazines and juvenile series, mostly Street & Smith publications. The
collection's site could not be reached from the environment this file was
written in, so these are item pages found through web search. **They have not
been viewed:** the per-Tomus picks below go by title only. Look at the
thumbnails and swap in better matches; the collection's own search will turn up
pulp magazines (war, adventure, sea and science-fiction titles) that would be
closer still.

| Key | Syracuse item page | Date (from catalogue record) |
|---|---|---|
| A | [Jack Harkaway among the Pirates (Round the World Library no. 5)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-among-the-pirates-round-the-world-library-no.-5/19086) | — |
| B | [Jack Harkaway in Australia (Round the World Library no. 16)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-in-australia-round-the-world-library-no.-16/19110) | — |
| C | [The Boys of St Aldates (Round the World Library no. 34)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-boys-of-st-aldates-round-the-world-library-no.-34/19146) | 1926? |
| D | [Phil's Rivals (Round the World Library no. 45)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-phils-rivals-round-the-world-library-no.-45/19170) | c. 1903 |
| E | [Jack Harkaway after Schooldays (Round the World Library no. 120)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-after-schooldays-round-the-world-library-no.-120/19304) | — |
| F | [Phil's Rivals (Round the World Library no. 162)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-phils-rivals-round-the-world-library-no.-162/19376) | — |
| G | [George Arnold's Pluck (Round the World Library no. 173)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-george-arnolds-pluck-round-the-world-library-no.-173/19396) | 1931 |
| H | [The Tour of the Zero Club, or, For Fame and Fortune (Round the World Library no. 180)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-tour-of-the-zero-club-or-for-fame-and-fortune-round-the-world-library-no.-180/19410) | 1931 |
| I | [The Young Railroader's Flyer, or, Astride the Iron Horse (Round the World Library no. 181)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-young-railroaders-flyer-or-astride-the-iron-horse-round-the-world-library-no.-181/19412) | — |
| J | [Frank Merriwell's Trust, or, Never Say Die (Merriwell Series no. 69)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwells-trust-or-never-say-die-merriwell-series-no.-69/17020) | c. 1901 |
| K | [Dick Merriwell's Western Mission, or, Out for a Record (Merriwell Series no. 97)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-dick-merriwells-western-mission-or-out-for-a-record-merriwell-series-no.-97/17022) | 1903 |
| L | [Frank Merriwell, Jr's Rivals, or, Team-work (Merriwell Series no. 215)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwell-jrs-rivals-or-team-work-merriwell-series-no.-215/17048) | 1929 |
| M | [Frank Merriwell's Races, or, Won by Pluck (Burt L. Standish Library no. 11)](https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwells-races-or-won-by-pluck-burt-l.-standish-library-no.-11/16480) | 1930 |

## The shared style paragraph

Each prompt ends with this paragraph. It is repeated in every prompt so each
one can be pasted on its own:

> Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
> gouache on board, reproduced in four-colour process with slightly coarse
> halftone and faint registration drift. Saturated primaries against deep
> shadow: hot reds, cadmium yellow, cobalt and ultramarine, with black-violet
> darks. Hard theatrical lighting from one side (firelight, lightning, lamplight
> or a low sun) and strong rim light on the figures. Muscular, dramatic,
> close-cropped figures caught at the peak of the action, with wide eyes,
> clenched jaws and flying cloaks. A diagonal composition that pushes the main
> action toward the viewer; secondary scenes recede into smoke, cloud or flame
> behind it in the montage manner of the period's covers. Visible brushwork,
> no photographic rendering, no modern digital gloss. Ancient Greek and Roman
> dress, arms and architecture drawn with period-pulp flair rather than museum
> exactness: crested bronze helmets, red cloaks, round hoplite shields, Roman
> scuta and pila, triremes with bronze rams. Keep the top 18% of the picture a
> flat, slightly mottled colour band for the masthead.

---

## Tomus I — Theseus · Romulus · Lycurgus · Numa · Solon · Publicola

**Reference covers:** B, D, J.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for the first volume of
Plutarch's Lives: heroes and founders of the mythic age.

DOMINANT SCENE (lower two-thirds, foreground): Inside the torch-lit stone
Labyrinth of Crete, a young bare-chested Greek hero, Theseus, braces against the
charge of the Minotaur, a towering bull-headed giant with a man's body, horns
lowered, nostrils smoking. Theseus grips a short bronze sword; a single bright
red thread runs from his wrist back through the dark corridor behind him. Torch
glare picks out bones on the flagstones and the frightened faces of Athenian
youths and maidens huddled in a side passage.

UPPER-LEFT VIGNETTE (fading into storm cloud): A Roman king in a purple cloak,
Romulus, raised on a rock above a terrified crowd, swallowed by a sudden
blackness: the sun gone out, rain driving sideways, lightning splitting the sky
as he vanishes into heaven.

UPPER-RIGHT VIGNETTE: On a cliff above the sea, an old king in a white robe,
Aegeus, leaps to his death, arms flung wide, as a galley under a black sail
rounds the headland below.

BACKGROUND MOTIFS worked into the smoke and stone: a she-wolf suckling twin
babies in a wooden trough beside a river, a woodpecker bringing food; a Sabine
maiden crushed beneath a heap of gold armlets and round shields (Tarpeia); a
Roman youth holding his right hand steadily in a brazier of glowing coals before
an astonished Etruscan king (Mucius Scaevola).

Colour key: torchlight orange and blood red against labyrinth blue-black; a band
of storm-green sky at the top.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-in-australia-round-the-world-library-no.-16/19110
https://digitalcollections.syr.edu/Documents/Detail/cover-of-phils-rivals-round-the-world-library-no.-45/19170
https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwells-trust-or-never-say-die-merriwell-series-no.-69/17020

Optional lettering (only if the generator renders text well): masthead
"PLUTARCHI VITÆ"; a round yellow corner badge reading "TOMUS I".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek and Roman dress, arms and architecture drawn
with period-pulp flair rather than museum exactness. Keep the top 18% of the
picture a flat, slightly mottled colour band for the masthead.
```

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Theseus, the thread and the Minotaur | [*Theseus* 19.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg001.perseus-eng3:19.1/) | “he got from Ariadne, who had fallen in love with him, the famous thread … he slew the Minotaur” |
| The black sail; Aegeus leaps | [*Theseus* 17.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg001.perseus-eng3:17.4/), [*Theseus* 22.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg001.perseus-eng3:22.1/) | “they sent the ship with a black sail”; “Aegeus, who therefore, in despair, threw himself down from the rock and was dashed in pieces” |
| Romulus vanishes in the storm | [*Romulus* 27.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg002.perseus-eng2:27.6/) | “the light of the sun failed, and night came down upon them … with awful peals of thunder and furious blasts driving rain from every quarter” |
| She-wolf and woodpecker | [*Romulus* 7.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg002.perseus-eng2:7.6/) | “by the dugs of a she-wolf and the morsels of a woodpecker, as we lay in a little trough by the side of the great river” |
| Tarpeia buried under the shields | [*Romulus* 17.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg002.perseus-eng2:17.4/) | “the girl was smitten by the gold and buried under the shields, and died from the number and weight of them” |
| Mucius Scaevola's hand in the coals | [*Publicola* 17.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg008.perseus-eng2:17.3/) | “Mucius held his right hand over the flames and, while the flesh was burning, stood looking at Porsena with a bold and steadfast countenance” |

Other strong scenes in this Tomus, if the illustrator wants substitutes:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Theseus hurls the robber Sciron off the cliffs | [*Theseus* 10.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg001.perseus-eng3:10.1/) | “He also slew Sciron on the borders of Megara, by hurling him down the cliffs” |
| The Sabine women run between the armies | [*Romulus* 19.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg002.perseus-eng2:19.1/) | “some carrying young children in their arms, some veiled in their dishevelled hair” |
| Remus leaps the trench and is struck down | [*Romulus* 10.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg002.perseus-eng2:10.1/) | “when he leaped across it, he was smitten … and fell dead there” |
| The Spartan boy and the stolen fox | [*Lycurgus* 18.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg004.perseus-eng2:18.1/) | “suffered the animal to tear out his bowels with its teeth and claws, and died rather than have his theft detected” |
| Lycurgus loses an eye to Alcander's staff | [*Lycurgus* 11.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg004.perseus-eng2:11.1/) | “smote him with his staff and put out one of his eyes” |
| A bronze shield falls from heaven to Numa | [*Numa* 13.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg005.perseus-eng2:13.1/) | “a bronze buckler fell from heaven, which came into the hands of Numa” |
| An unchaste Vestal buried alive | [*Numa* 10.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg005.perseus-eng2:10.4/) | “she that has broken her vow of chastity is buried alive near the Colline gate” |
| Solon, feigning madness, recites from the herald's stone | [*Solon* 8.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg007.perseus-eng2:8.2/) | “he got upon the herald’s stone and recited the poem” |
| Croesus on the pyre cries out to Solon | [*Solon* 28.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg007.perseus-eng2:28.2/) | “as he lay bound upon the pyre in the sight of all the Persians and of Cyrus himself … he called out thrice … O Solon!” |
| Brutus watches his sons beheaded | [*Publicola* 6.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg008.perseus-eng2:6.3/) | “watched the dreadful punishment of his sons until the lictors threw them on the ground and cut off their heads with the axe” |
| Horatius Cocles holds the bridge | [*Publicola* 16.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg008.perseus-eng2:16.6/) | “standing at the head of the bridge, kept the enemy back until his companions had cut the bridge in two behind him” |
| Cloelia crosses the Tiber on horseback | [*Publicola* 19.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg008.perseus-eng2:19.2/) | “Cloelia, crossed the stream on horseback, exhorting and encouraging the rest as they swam” |

---

## Tomus II — Themistocles · Camillus · Pericles · Fabius Maximus · Alcibiades · Coriolanus

**Reference covers:** A, K, G.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for a volume of Plutarch's
Lives: Athens and Rome against invaders and against themselves.

DOMINANT SCENE (foreground, diagonal from lower left): The sea-fight at Salamis.
Two Greek triremes have rammed a towering Persian flagship bow-on, bronze beaks
locked together, oars splintering. On the high Persian deck the admiral
Ariamenes, in scale armour and a tall Persian cap, looses arrows down "as though
from a city wall" while Greek marines in crested bronze helmets swarm up to
board him. An owl wheels through the rigging of the nearest Greek ship. Spray,
wreckage, a rising swell.

UPPER-RIGHT VIGNETTE (on the headland, small and gleaming): Xerxes on a gilded
throne, scribes kneeling around him with tablets, watching his fleet break up.

UPPER-LEFT VIGNETTE (over the burning roofs of Rome, at night): Sacred white
geese flapping and screaming on the Capitol as shaggy Gauls with long swords
scale the cliff toward sleeping Roman sentries.

MIDDLE-GROUND MOTIF (between the vignettes, in firelight): The Gallic chieftain
Brennus, laughing, throwing his sword and belt onto the scales piled with Roman
gold.

BACKGROUND MOTIFS dissolving into smoke: a herd of oxen running through a
mountain pass at night with blazing torches tied to their horns (Hannibal's
ruse); a handsome Athenian, Alcibiades, bursting out of a burning house with his
cloak wrapped around one arm and a sword in the other as arrows and javelins fly
at him.

Colour key: sea ultramarine and foam white, sunset gold on the bronze, a band of
red night-glow over Rome.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-among-the-pirates-round-the-world-library-no.-5/19086
https://digitalcollections.syr.edu/Documents/Detail/cover-of-dick-merriwells-western-mission-or-out-for-a-record-merriwell-series-no.-97/17022
https://digitalcollections.syr.edu/Documents/Detail/cover-of-george-arnolds-pluck-round-the-world-library-no.-173/19396

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS II".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek and Roman dress, arms and architecture drawn
with period-pulp flair rather than museum exactness. Keep the top 18% of the
picture a flat, slightly mottled colour band for the masthead.
```

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Ariamenes' flagship rammed at Salamis | [*Themistocles* 14.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg010.perseus-eng2:14.3/) | “kept shooting arrows and javelins as though from a city wall … the two ships struck each other bow on, crashed together, and hung fast by their bronze beaks” |
| The owl in the rigging | [*Themistocles* 12.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg010.perseus-eng2:12.1/) | “an owl was seen to fly through the fleet from the right and alight in his rigging” |
| Xerxes on his gilded throne | [*Themistocles* 13.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg010.perseus-eng2:13.1/) | “a gilded throne had been set for him at his command, and many secretaries stationed near at hand” |
| The geese of Juno wake the Capitol | [*Camillus* 27.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg011.perseus-eng2:27.3/) | “perceived the approach of the Gauls, dashed at them with loud cries, and so waked all the garrison” |
| Brennus throws his sword on the scales | [*Camillus* 28.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg011.perseus-eng2:28.5/) | “Brennus, with a mocking laugh, stripped off his sword, and added it, belt and all, to the weights” |
| Hannibal's oxen with burning horns | [*Fabius Maximus* 6.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg013.perseus-eng2:6.4/), [*Fabius Maximus* 6.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg013.perseus-eng2:6.5/) | “fasten to each of their horns a torch”; “amazed at the flames gleaming on the tips of their horns” |
| The death of Alcibiades | [*Alcibiades* 39.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg015.perseus-eng2:39.3/) | “wrapping his cloak about his left arm, and drawing his sword with his right, he dashed out, unscathed by the fire” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Three captive Persian princes brought to the altar | [*Themistocles* 13.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg010.perseus-eng2:13.2/) | “three prisoners of war were brought to him, of visage most beautiful to behold, conspicuously adorned with raiment and with gold” |
| Themistocles smuggled to Persia in a curtained women's wagon | [*Themistocles* 26.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg010.perseus-eng2:26.4/) | “carried in tents closely hung round about with curtains and set upon four-wheeled wagons” |
| Themistocles drinks bull's blood | [*Themistocles* 31.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg010.perseus-eng2:31.5/) | “drank bull’s blood, or as some say, took a quick poison, and so died in Magnesia” |
| The traitor schoolmaster whipped home by his pupils | [*Camillus* 10.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg011.perseus-eng2:10.4/) | “put rods and scourges in the hands of the boys, that they might chastise the traitor and drive him back into the city” |
| A Gaul strokes a senator's beard | [*Camillus* 22.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg011.perseus-eng2:22.5/) | “gently grasped his chin and stroked his long beard, whereupon Papirius, with his staff, smote him a crushing blow on the head” |
| Pericles covers the frightened steersman's eyes during an eclipse | [*Pericles* 35.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg012.perseus-eng2:35.2/) | “Pericles held up his cloak before the man’s eyes” |
| The one-horned ram | [*Pericles* 6.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg012.perseus-eng2:6.2/) | “the head of a one-horned ram was brought to Pericles from his country-place” |
| Cannae: the hot wind and dust | [*Fabius Maximus* 16.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg013.perseus-eng2:16.1/) | “This wind came down like a fiery hurricane, and raised a huge cloud of dust” |
| Alcibiades sails home under a purple sail | [*Alcibiades* 32.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg015.perseus-eng2:32.2/) | “the commander’s ship put into harbors with a sail of purple hue” |
| The Hermae mutilated in a single night | [*Alcibiades* 18.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg015.perseus-eng2:18.3/) | “the mutilation of the Hermae, most of which, in a single night, had their faces and forms disfigured” |
| Coriolanus bursts through the gate of Corioli | [*Coriolanus* 8.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg016.perseus-eng2:8.5/) | “he pushed his way through the enemy, leaped against the gate, and burst in along with them” |
| Coriolanus, muffled, at his enemy's hearth | [*Coriolanus* 23.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg016.perseus-eng2:23.1/) | “took his seat at the hearth in silence, covered his head, and remained there motionless” |
| “What hast thou done to me, my mother!” | [*Coriolanus* 34.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg016.perseus-eng2:34.1/), [*Coriolanus* 36.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg016.perseus-eng2:36.4/) | “went with the other women to the camp of the Volscians”; “What hast thou done to me, my mother!” |

---

## Tomus III — Timoleon · Aemilius Paulus · Pelopidas · Marcellus · Aristides · Cato the Elder

**Reference covers:** A, H, L.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for a volume of Plutarch's
Lives: war machines, assassins and thunderbolts.

DOMINANT SCENE (foreground, right two-thirds): The siege of Syracuse. A Roman
war-galley is being hauled straight up out of the sea by a gigantic iron claw
swung from a beam over the city wall, its crew tumbling from the deck into the
foam; another ship below is being crushed by a falling weight. On the battlement
an old Greek with a white beard, Archimedes, works the lever, lit from below by
the spray and a low sun.

LEFT-SIDE VIGNETTE (in a dark banquet hall, warm lamplight): Theban conspirators
disguised as women, in long gowns over their breastplates and with thick wreaths
of pine and fir hiding their faces, drawing swords as they burst among the
drunken tyrants reclining at table.

UPPER VIGNETTE (in a black thunder-sky): A Carthaginian army of
heavily armoured spearmen and four-horse war chariots blinded by a storm of
hail, rain and lightning driving into their faces, while Greek hoplites charge
with the storm at their backs (the battle at the river Crimisus).

BACKGROUND MOTIFS: a Macedonian phalanx with a wall of very long levelled spears
and interlocked shields; a giant Gallic king in gold-and-silver armour that
"gleamed like lightning", unhorsed by a Roman general's spear (Marcellus); a
full moon going dark and turning strange colours over a sleeping camp.

Colour key: sea-green and storm-violet; lightning white; gold on the Gaul;
lamplight amber in the banquet.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-among-the-pirates-round-the-world-library-no.-5/19086
https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-tour-of-the-zero-club-or-for-fame-and-fortune-round-the-world-library-no.-180/19410
https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwell-jrs-rivals-or-team-work-merriwell-series-no.-215/17048

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS III".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek and Roman dress, arms and architecture drawn
with period-pulp flair rather than museum exactness. Keep the top 18% of the
picture a flat, slightly mottled colour band for the masthead.
```

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Archimedes' claws lift Roman ships | [*Marcellus* 15.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg022.perseus-eng2:15.2/), [*Marcellus* 15.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg022.perseus-eng2:15.3/) | “seized at the prow by iron claws, or beaks like the beaks of cranes, drawn straight up into the air”; “a ship would be lifted out of the water into mid-air, whirled hither and thither as it hung there, a dreadful spectacle” |
| Conspirators dressed as women at the tyrants' banquet | [*Pelopidas* 11.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg021.perseus-eng2:11.1/), [*Pelopidas* 11.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg021.perseus-eng2:11.2/) | “had put on women’s apparel over their breastplates, and wore thick garlands of pine and fir which shaded their faces”; “the visitors drew their swords, and rushing through the midst of the tables” |
| Storm at the Crimisus | [*Timoleon* 28.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg018.perseus-eng2:28.2/), [*Timoleon* 28.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg018.perseus-eng2:28.3/) | “fearful peals of thunder crashed down, and vivid flashes of lightning darted forth”; “it smote the Barbarians in the face and dazzled their eyes” |
| Carthaginian chariots and armour | [*Timoleon* 25.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg018.perseus-eng2:25.1/), [*Timoleon* 28.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg018.perseus-eng2:28.1/) | “four-horse chariots”; “the iron breastplates and bronze helmets with which their persons were protected, and the great shields” |
| The Macedonian phalanx at Pydna | [*Aemilius Paulus* 19.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg019.perseus-eng2:19.2/) | “with long spears sat at one level were withstanding his shield-bearing troops, and saw too the strength of their interlocked shields” |
| Marcellus kills the Gallic king | [*Marcellus* 7.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg022.perseus-eng2:7.1/), [*Marcellus* 7.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg022.perseus-eng2:7.2/) | “conspicuous for a suit of armour which was set off with gold and silver and bright colours … it gleamed like lightning”; “by a thrust of his spear which pierced his adversary’s breastplate” |
| The moon goes dark before Pydna | [*Aemilius Paulus* 17.7](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg019.perseus-eng2:17.7/) | “the moon, which was full and high in the heavens, grew dark, lost its light, took on all sorts of colours in succession, and finally disappeared” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Timoleon weeps, veiled, as his tyrant brother is killed | [*Timoleon* 4.8](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg018.perseus-eng2:4.8/) | “Timoleon withdrew a little space from him and stood weeping with muffled head, while the other two, drawing their swords speedily despatched him” |
| The mules laden with parsley | [*Timoleon* 26.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg018.perseus-eng2:26.1/) | “there met them by chance some mules laden with parsley” |
| King Perseus walks in the Roman triumph | [*Aemilius Paulus* 34.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg019.perseus-eng2:34.1/) | “clad in a dark robe and wearing the high boots of his country” |
| The fiery-maned filly | [*Pelopidas* 22.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg021.perseus-eng2:22.1/) | “a filly broke away from the herd of horses and sped through the camp … her glossy mane, which was fiery red” |
| Pelopidas charges the tyrant and falls | [*Pelopidas* 32.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg021.perseus-eng2:32.6/), [*Pelopidas* 32.7](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg021.perseus-eng2:32.7/) | “rushed with challenging cries upon the tyrant”; “thrusting their spears through his armour and covering him with wounds” |
| Archimedes killed over his diagram | [*Marcellus* 19.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg022.perseus-eng2:19.4/) | “working out some problem with the aid of a diagram … Suddenly a soldier came upon him” |
| An illiterate voter asks Aristides to write “Aristides” | [*Aristides* 7.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg024.perseus-eng2:7.6/) | “I am tired of hearing him everywhere called The Just … wrote his name on the ostrakon and handed it back” |
| Spartans sit under the arrows at Plataea | [*Aristides* 17.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg024.perseus-eng2:17.6/) | “to sit quiet with their shields planted in front of them … presently their missiles actually reached them” |
| Cato drops a Libyan fig in the Senate | [*Cato the Elder* 27.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg025.perseus-eng2:27.1/) | “contrived to drop a Libyan fig in the Senate, as he shook out the folds of his toga” |
| Antiochus' teeth knocked out at Thermopylae | [*Cato the Elder* 14.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg025.perseus-eng2:14.1/) | “Antiochus, being hit in the mouth with a stone which knocked his teeth out, wheeled his horse about for very anguish” |

---

## Tomus IV — Philopoemen · Flamininus · Pyrrhus · Marius · Lysander · Sulla

**Reference covers:** B, K, M.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for a volume of Plutarch's
Lives: elephants, barbarians and warlords.

DOMINANT SCENE (foreground): Street fighting inside the gate of Argos at dusk.
A war elephant lies fallen across the gateway, trumpeting and roaring, while a
second elephant thrashes through the crowd searching for its fallen rider. In
the centre a king on horseback, Pyrrhus, in a crested helmet, turns to strike
at a young Argive soldier; above him, on a flat rooftop, an old woman in a dark
shawl lifts a heavy roof tile in both hands to hurl down on his head.

UPPER-LEFT VIGNETTE (in a dim room, a single shaft of light): An old Roman
general, Marius, half in shadow, his eyes blazing like flame, roaring at a
barbarian executioner who drops his sword and flees.

UPPER-RIGHT VIGNETTE (sun-scorched plain): The wagon-laager of the defeated
Cimbri: tall barbarian women in black garments standing on the wagons against a
red sky, cattle stampeding below.

BACKGROUND MOTIFS: a Greek stadium where ravens fall stunned out of the sky as a
huge crowd roars (Flamininus proclaiming Greek freedom); an Achaean cavalry
general with a javelin through both thighs wrenching the shaft in two
(Philopoemen); a little gold figure of Apollo kissed by a red-faced Roman general
on the battlefield (Sulla).

Colour key: elephant grey and dust ochre, dusk magenta sky, firelight on the
gate.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-in-australia-round-the-world-library-no.-16/19110
https://digitalcollections.syr.edu/Documents/Detail/cover-of-dick-merriwells-western-mission-or-out-for-a-record-merriwell-series-no.-97/17022
https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwells-races-or-won-by-pluck-burt-l.-standish-library-no.-11/16480

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS IV".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek and Roman dress, arms and architecture drawn
with period-pulp flair rather than museum exactness. Keep the top 18% of the
picture a flat, slightly mottled colour band for the masthead.
```

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Elephants jam the gate of Argos | [*Pyrrhus* 33.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg030.perseus-eng2:33.4/) | “the largest of the elephants had fallen athwart the gateway and lay there roaring … another elephant … seeking to recover his rider” |
| The old woman's roof tile kills Pyrrhus | [*Pyrrhus* 34.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg030.perseus-eng2:34.2/) | “watching the battle from the house-top … lifting up a tile with both her hands threw it at Pyrrhus” |
| “Man, dost thou dare to slay Caius Marius?” | [*Marius* 39.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg031.perseus-eng2:39.2/) | “the eyes of Marius seemed to shoot out a strong flame, and that a loud voice issued from the shadows” |
| The Cimbrian women at the wagons | [*Marius* 27.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg031.perseus-eng2:27.2/) | “The women, in black garments, stood at the waggons and slew the fugitives” |
| Ravens fall at the shout of freedom | [*Flamininus* 10.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg028.perseus-eng2:10.6/) | “ravens which chanced to be flying overhead fell down into the stadium” |
| Philopoemen breaks the javelin through his thighs | [*Philopoemen* 6.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg027.perseus-eng2:6.4/), [*Philopoemen* 6.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg027.perseus-eng2:6.5/) | “he had both his thighs pierced through by a thonged javelin”; “by moving his legs backward and forward he broke the shaft of the weapon in two in the middle” |
| Sulla kisses his golden Apollo | [*Sulla* 29.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg033.perseus-eng2:29.6/) | “a little golden image of Apollo from Delphi which he always carried in his bosom … he took it out and kissed it” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Philopoemen kills the tyrant Machanidas at the ditch | [*Philopoemen* 10.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg027.perseus-eng2:10.6/) | “a powerful hunter attacking a wild beast that has been forced to turn at bay” |
| Philopoemen given poison in the dungeon | [*Philopoemen* 20.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg027.perseus-eng2:20.2/) | “lying down wrapped in his soldier’s cloak … he saw a light and a man standing by him holding the cup of poison” |
| Cynoscephalae fought in fog | [*Flamininus* 8.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg028.perseus-eng2:8.1/) | “the clouds turned to mist, the whole plain was filled with profound darkness” |
| An elephant revealed behind a curtain to frighten Fabricius | [*Pyrrhus* 20.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg030.perseus-eng2:20.2/) | “ordered the largest of these animals to be stationed behind a hanging” |
| Elephants rout the Roman horse at Heraclea | [*Pyrrhus* 17.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg030.perseus-eng2:17.3/) | “their horses, before they got near the animals, were terrified and ran away with their riders” |
| Marius hidden in the marsh at Minturnae | [*Marius* 37.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg031.perseus-eng2:37.6/) | “bade him crouch down in a hollow place by the side of the river, and threw over him a mass of reeds” |
| Marius among the ruins of Carthage | [*Marius* 40.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg031.perseus-eng2:40.4/) | “thou hast seen Caius Marius a fugitive, seated amid the ruins of Carthage” |
| A bronze shield hoisted as the signal at Aegospotami | [*Lysander* 11.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg032.perseus-eng2:11.1/) | “to hoist a brazen shield at the prow, as a signal for the onset” |
| A blazing meteor before the stone fell | [*Lysander* 12.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg032.perseus-eng2:12.4/) | “a fiery body of vast size, as if it had been a flaming cloud” |
| Athens' walls torn down to flute music | [*Lysander* 15.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg032.perseus-eng2:15.4/) | “tore down the walls, and burned up the triremes, to the sound of the flute” |
| Sulla's face | [*Sulla* 2.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg033.perseus-eng2:2.1/) | “Sulla is a mulberry sprinkled o’er with meal” |
| Sulla storms Athens at midnight | [*Sulla* 14.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg033.perseus-eng2:14.3/) | “blasts of many trumpets and bugles … rushing through the narrow streets with drawn swords” |
| A satyr brought before Sulla | [*Sulla* 27.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg033.perseus-eng2:27.2/) | “a hoarse cry that was something between the neighing of a horse and the bleating of a goat” |

---

## Tomus V — Cimon · Lucullus

Only two Lives, so the cover can give each a full half.

**Reference covers:** A, E, H.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for a volume of Plutarch's
Lives: an Athenian admiral and a Roman conqueror of the East. Split the picture
on a steep diagonal, lower left to upper right.

LOWER-RIGHT HALF (the East, gold and crimson): A doomed Pontic queen, Monime, in
a palace chamber hung with silk, has torn the jewelled royal diadem from her
hair and holds it up, snapped, in disgust, her bare throat offered to a grim
eunuch officer with a drawn blade; behind her another princess drinks from a
poison cup. Through an arch beyond, ranks of Armenian mail-clad horsemen, men and
horses sealed head to hoof in iron, break and stampede down a hillside as Roman
legionaries strike at their legs.

UPPER-LEFT HALF (the sea, ultramarine and storm): A Greek fleet off the mouth of a
river smashes into Persian ships, and armoured Athenian hoplites leap from the
bows into the surf to charge a huge Persian army on the beach; the Persian camp
behind is heaped with treasure.

INSET along the dividing diagonal, framed by steam: a dark marble bath-house at
night, a sealed doorway, a pale ghostly young man in the vapour, faint groans
(the haunted bath at Chaeroneia).

SMALL MOTIF in a corner: a mule loaded with spilling royal gold coins blocking a
narrow road, Roman soldiers fighting each other over the gold while a king
gallops away.

Colour key: imperial crimson, gold leaf, ice-blue mail; Aegean blue for the sea
half; ghost-green steam.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-among-the-pirates-round-the-world-library-no.-5/19086
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-after-schooldays-round-the-world-library-no.-120/19304
https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-tour-of-the-zero-club-or-for-fame-and-fortune-round-the-world-library-no.-180/19410

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS V".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek, Roman and Persian dress, arms and
architecture drawn with period-pulp flair rather than museum exactness. Keep the
top 18% of the picture a flat, slightly mottled colour band for the masthead.
```

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Monime and the diadem | [*Lucullus* 18.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:18.4/) | “Monimé snatched the diadem from her head, fastened it round her neck, and hanged herself. But her halter quickly broke in two. O cursed bauble, she cried” |
| Berenice drinks poison | [*Lucullus* 18.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:18.5/) | “Berenicé, taking a cup of poison, shared it with her mother” |
| The mail-clad horsemen at Tigranocerta | [*Lucullus* 28.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:28.3/), [*Lucullus* 28.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:28.4/) | “owing to the weight and rigidity of their armour; in this they are, as it were, immured”; “to smite the enemy’s legs and thighs” |
| The double victory at the Eurymedon | [*Cimon* 13.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg035.perseus-eng2:13.1/), [*Cimon* 13.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg035.perseus-eng2:13.2/) | “he landed his hoplites still hot with the struggle of the sea-fight”; “their camp, which was full of all sorts of treasure” |
| The haunted bath at Chaeroneia | [*Cimon* 1.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg035.perseus-eng2:1.6/) | “certain phantoms appeared in the place, and groans were heard there … the door of the vapor-bath was walled up” |
| The mule of royal gold | [*Lucullus* 17.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:17.6/) | “one of the mules which carried the royal gold came between him and them … The soldiers fell to plundering and collecting the gold” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Damon's band, faces smeared with soot, strikes at dawn | [*Cimon* 1.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg035.perseus-eng2:1.3/) | “smeared their faces with soot … heated themselves with wine, and at daybreak fell upon the Roman” |
| Cimon brings home the bones of Theseus | [*Cimon* 8.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg035.perseus-eng2:8.6/) | “discovered at last the hallowed spot, had the bones bestowed in his own trireme” |
| The great earthquake at Sparta | [*Cimon* 16.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg035.perseus-eng2:16.4/), [*Cimon* 16.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg035.perseus-eng2:16.5/) | “shook Taygetus so that sundry peaks were torn away”; “a hare made its appearance … the gymnasium fell” |
| A storm wrecks Mithridates' siege engines at Cyzicus | [*Lucullus* 10.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:10.2/) | “the sea began to toss under a boisterous wind, and the siege-engines of the king along the walls” |
| Tigranes mocks the small Roman army | [*Lucullus* 27.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:27.4/) | “If they are come as ambassadors, they are too many; if as soldiers, too few” |
| Lucullus' sea-villas | [*Lucullus* 39.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:39.3/) | “suspended hills over vast tunnels, girdled his residences with zones of sea … Xerxes in a toga” |
| “Lucullus dines with Lucullus” | [*Lucullus* 41.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg036.perseus-eng2:41.2/) | “dost thou not know that to-day Lucullus dines with Lucullus?” |

---

## Tomus VI — Nicias · Crassus · Sertorius · Eumenes · Agesilaus · Pompey

**Reference covers:** K, B, F.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for a volume of Plutarch's
Lives: desert catastrophes, rebel gladiators and doomed generals.

DOMINANT SCENE (foreground): The desert of Carrhae under a white-hot sky.
Parthian horse-archers and armoured lancers sweep in from the right, having
just flung off their cloaks to reveal blazing steel helmets and breastplates,
their horses covered in plates of bronze and steel. Huge drums hung with
bronze bells boom in the dust behind them. A square of Roman legionaries, red
shields locked, cowers under the arrow-storm; at its centre a grey-haired
Roman general in a crimson cloak, Crassus, stares in horror.

UPPER-LEFT VIGNETTE (night, volcanic slope): Escaped gladiators climbing down a
sheer cliff on ladders woven from wild vines, moonlight on their short swords;
a sleeping gladiator with a snake coiled around his face (Spartacus).

UPPER-RIGHT VIGNETTE (sunset beach): A small boat drawn up on the sand, Roman
ships fleeing out to sea; on the shore, a lone freedman builds a funeral pyre
from the broken timbers of an old fishing boat beside a body wrapped in a
tunic (Pompey).

BACKGROUND MOTIFS: a full moon in eclipse over a Greek fleet in a harbour;
Athenian soldiers drinking from a river running red as enemies cut them down;
a pure white fawn standing beside a Roman general in Spanish hill country
(Sertorius); two cavalry commanders who have dropped their reins and are
grappling hand to hand on rearing horses (Eumenes).

Colour key: bleached sand and white sky against Roman red and Parthian steel;
night blue for the gladiators; sunset orange for the beach.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-dick-merriwells-western-mission-or-out-for-a-record-merriwell-series-no.-97/17022
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-in-australia-round-the-world-library-no.-16/19110
https://digitalcollections.syr.edu/Documents/Detail/cover-of-phils-rivals-round-the-world-library-no.-162/19376

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS VI".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek, Roman and Parthian dress, arms and
architecture drawn with period-pulp flair rather than museum exactness. Keep the
top 18% of the picture a flat, slightly mottled colour band for the masthead.
```

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| The Parthians unmask their armour | [*Crassus* 24.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg039.perseus-eng2:24.1/) | “suddenly their enemies dropped the coverings of their armour, and were seen to be themselves blazing in helmets and breastplates … their horses clad in plates of bronze and steel” |
| The Parthian drums | [*Crassus* 23.7](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg039.perseus-eng2:23.7/) | “hollow drums of distended hide, covered with bronze bells … a blend of wild beast’s roar and harsh thunder peal” |
| Spartacus' men climb down vine ladders | [*Crassus* 9.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg039.perseus-eng2:9.2/) | “wove these into strong ladders of such strength and length that when they were fastened at the top they reached along the face of the cliff” |
| The serpent on Spartacus' face | [*Crassus* 8.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg039.perseus-eng2:8.3/) | “a serpent was seen coiled about his face as he slept” |
| Pompey's pyre on the beach | [*Pompey* 80.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg045.perseus-eng2:80.2/) | “found the remnants of a small fishing-boat, old stuff, indeed, but sufficient to furnish a funeral pyre” |
| The ships flee as Pompey is killed | [*Pompey* 80.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg045.perseus-eng2:80.1/) | “they uttered a wailing cry that could be heard as far as the shore, and weighing anchor quickly, took to flight” |
| The eclipse at Syracuse | [*Nicias* 23.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg038.perseus-eng2:23.1/) | “there came an eclipse of the moon by night. This was a great terror to Nicias” |
| Slaughter at the river Assinarus | [*Nicias* 27.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg038.perseus-eng2:27.4/) | “an exceeding great and savage carnage raged in the river itself, men being butchered as they drank” |
| Sertorius' white fawn | [*Sertorius* 11.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg042.perseus-eng2:11.2/) | “the fawn—and he was struck with its unusual colour, for it was entirely white” |
| Eumenes and Neoptolemus grapple on horseback | [*Eumenes* 7.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg041.perseus-eng2:7.5/) | “Their horses dashed together with the violence of colliding triremes, and dropping the reins they clutched one another with their hands” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Athenian prisoners spared for reciting Euripides | [*Nicias* 29.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg038.perseus-eng2:29.2/) | “Some also were saved for the sake of Euripides” |
| Spartacus' last stand | [*Crassus* 11.7](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg039.perseus-eng2:11.7/) | “he stood alone, surrounded by a multitude of foes, and was still defending himself when he was cut down” |
| Crassus' head thrown into a performance of the *Bacchae* | [*Crassus* 33.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg039.perseus-eng2:33.2/) | “cast the head of Crassus into the centre” |
| Sertorius swims the Rhone in full armour | [*Sertorius* 3.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg042.perseus-eng2:3.1/) | “he made his way across the Rhone, swimming, shield and breastplate and all, against a strongly adverse current” |
| Sertorius murdered at a banquet | [*Sertorius* 26.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg042.perseus-eng2:26.5/), [*Sertorius* 26.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg042.perseus-eng2:26.6/) | “Perpenna, after taking a cup of wine in his hands, dropped it as he was drinking”; “died from the blows of many” |
| Eumenes' army: gold armour and towered elephants | [*Eumenes* 14.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg041.perseus-eng2:14.4/) | “The gleams of their golden armour in the sun … on the backs of the elephants the towers and purple trappings were seen” |
| Eumenes, betrayed, raises his bound hands | [*Eumenes* 17.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg041.perseus-eng2:17.3/) | “he stretched forth his hands, bound as they were” |
| The naked Spartan youth Isadas fights the invaders | [*Agesilaus* 34.7](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg044.perseus-eng2:34.7/) | “Naked as he was … he took a spear in one hand, and a sword in the other, leaped forth from his house” |
| Pirates with gilded sails and silvered oars | [*Pompey* 24.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg045.perseus-eng2:24.3/) | “their gilded sails, and purple awnings, and silvered oars” |
| Pirates make a Roman captive walk the ladder | [*Pompey* 24.8](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg045.perseus-eng2:24.8/) | “let down a ladder in mid ocean and bid him disembark and go on his way rejoicing” |
| Pompey covers his face as he is struck down | [*Pompey* 79.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg045.perseus-eng2:79.4/) | “drawing his toga down over his face with both hands … submitted to their blows” |

---

## Tomus VII — Alexander · Julius Caesar · Phocion · Cato the Younger

**Reference covers:** I, M, J.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for a volume of Plutarch's
Lives: Alexander the Great and Julius Caesar.

DOMINANT SCENE (foreground, left): The palace of Persepolis set ablaze by night.
A young golden-haired king, Alexander, garlanded with flowers, leaps forward with
a flaming torch held high; beside him an Athenian courtesan, Thaïs, in a
sheer saffron robe, flings her own torch at the carved cedar columns. Drunken
Macedonian companions with torches surge behind them; flames pour up the
staircases and lion-reliefs of the palace.

UPPER-RIGHT VIGNETTE (marble hall, cold light): The assassination of Caesar: a
ring of senators in white togas stabbing with daggers, Caesar in a purple-bordered
toga collapsing against the pedestal of a great marble statue of Pompey, the
pedestal splashed with blood.

MIDDLE-RIGHT VIGNETTE (river at dawn): A Roman general on horseback urging his
cavalry into a small river, his cloak whipping behind him: the crossing of the
Rubicon.

BACKGROUND MOTIFS: a boy taming a huge black stallion by turning its head toward
the sun (Bucephalus); a dim tent at night, a guttering lamp, and a giant shadowy
phantom standing silently over a seated general (the apparition seen by Brutus);
a rolled-up, carpet-like bedding sack tied with cord, carried on a servant's
shoulder into a lamplit palace, a woman's dark eyes showing at the opening
(Cleopatra smuggled to Caesar).

Colour key: inferno orange and gold over midnight blue; marble white and blood
red for the Senate; dawn rose for the Rubicon.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-young-railroaders-flyer-or-astride-the-iron-horse-round-the-world-library-no.-181/19412
https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwells-races-or-won-by-pluck-burt-l.-standish-library-no.-11/16480
https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwells-trust-or-never-say-die-merriwell-series-no.-69/17020

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS VII".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek, Persian and Roman dress, arms and
architecture drawn with period-pulp flair rather than museum exactness. Keep the
top 18% of the picture a flat, slightly mottled colour band for the masthead.
```

The bedding-sack vignette: Plutarch has Cleopatra stretched out inside a
bed-sack, tied with cord and carried in by Apollodorus. The prompt's "rolled-up
carpet" is the popular image; ask for a cloth sack if fidelity matters more.

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Thaïs and Alexander fire the palace | [*Alexander* 38.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg047.perseus-eng2:38.3/), [*Alexander* 38.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg047.perseus-eng2:38.4/) | “leaping to his feet, with a garland on his head and a torch in his hand, led them the way”; “surrounded the palace … ran thither with torches” |
| Caesar dies at the foot of Pompey's statue | [*Caesar* 66.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg048.perseus-eng2:66.6/), [*Caesar* 66.7](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg048.perseus-eng2:66.7/) | “driven hither and thither like a wild beast”; “against the pedestal on which the statue of Pompey stood. And the pedestal was drenched with his blood” |
| The Rubicon | [*Caesar* 32.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg048.perseus-eng2:32.6/) | “Let the die be cast, he hastened to cross the river” |
| Bucephalus turned toward the sun | [*Alexander* 6.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg047.perseus-eng2:6.3/) | “took hold of his bridle-rein, and turned him towards the sun” |
| The phantom by the lamp | [*Caesar* 69.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg048.perseus-eng2:69.6/) | “looking towards the light of the lamp, which was slowly going out, he saw a fearful vision of a man of unnatural size and harsh aspect” |
| Cleopatra carried in a bed-sack | [*Caesar* 49.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg048.perseus-eng2:49.1/) | “she stretched herself at full length inside a bed-sack, while Apollodorus tied the bedsack up with a cord and carried it indoors to Caesar” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| “Macedonia has not room for thee” | [*Alexander* 6.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg047.perseus-eng2:6.5/) | “My son, seek thee out a kingdom equal to thyself; Macedonia has not room for thee” |
| Alexander's white plumes at the Granicus | [*Alexander* 16.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg047.perseus-eng2:16.4/) | “conspicuous by his buckler and by his helmet’s crest, on either side of which was fixed a plume of wonderful size and whiteness” |
| Alexander spears Cleitus at the curtain | [*Alexander* 51.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg047.perseus-eng2:51.5/), [*Alexander* 51.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg047.perseus-eng2:51.6/) | “met Cleitus as he was drawing aside the curtain before the door, and ran him through”; “No sooner had Cleitus fallen with a roar and a groan” |
| The Gordian wagon | [*Alexander* 18.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg047.perseus-eng2:18.1/) | “the much-talked-of waggon bound fast to its yoke with bark of the cornel-tree” |
| Caesar crucifies the pirates who held him | [*Caesar* 2.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg048.perseus-eng2:2.4/) | “took the robbers out of prison, and crucified them all, just as he had often warned them” |
| Caesar swims for it at Pharos, holding his papers | [*Caesar* 49.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg048.perseus-eng2:49.4/) | “he threw himself into the sea … holding many papers in his hand and would not let them go, though missiles were flying at him” |
| A sea-beast seizes a worshipper in the harbour | [*Phocion* 28.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg049.perseus-eng2:28.3/) | “as a mystic initiate was washing a pig in the harbour of Cantharus, a great fish seized the man and devoured” |
| Phocion's hemlock | [*Phocion* 36.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg049.perseus-eng2:36.2/) | “seeing the executioner bruising the hemlock” |
| Phocion's wife carries his bones home by night | [*Phocion* 37.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg049.perseus-eng2:37.3/) | “putting the bones in her bosom and carrying them by night to her dwelling, she buried them by the hearth” |
| The boy Cato held out of a window | [*Cato the Younger* 2.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg050.perseus-eng2:2.3/) | “lifted him up through a window, as if he would cast him out … frequently shaking the boy as he held his body out at the window” |
| Cato's desert march with snake-charmers | [*Cato the Younger* 56.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg050.perseus-eng2:56.3/) | “the people called Psylli. These cure the bites of serpents by sucking out the venom” |
| Cato reads Plato, then finds his sword gone | [*Cato the Younger* 68.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg050.perseus-eng2:68.2/) | “he took up Plato’s dialogue On the Soul … not seeing his sword hanging there” |
| Cato's death at Utica | [*Cato the Younger* 70.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg050.perseus-eng2:70.5/) | “drew his sword from its sheath and stabbed himself below the breast … overturning a geometrical abacus” |

---

## Tomus VIII — Agis · Cleomenes · Tiberius Gracchus · Caius Gracchus · Demosthenes · Cicero

**Reference covers:** C, G, L.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for a volume of Plutarch's
Lives: reformers and orators hunted to the death.

DOMINANT SCENE (foreground): A dark Italian woodland road at dawn. A litter has
been set down; from between its curtains an old Roman orator with a haggard,
unshaven face, Cicero, stretches out his neck toward the sword of a soldier
standing over him, one hand cupping his chin as he stares steadily at his
killers. Other soldiers turn their faces away. A black crow flaps on the litter
pole.

UPPER-LEFT VIGNETTE (Rome, the Capitol steps): A riot of senators and their
servants clubbing a young tribune in a torn tunic with the legs of broken
benches; his toga lies trampled behind him (Tiberius Gracchus).

UPPER-RIGHT VIGNETTE (wooden bridge over the Tiber): Two loyal friends with
swords hold the head of a narrow wooden bridge against a mob, while beyond them
a fleeing man in a toga runs with a single servant toward a dark sacred grove
(Caius Gracchus).

MIDDLE MOTIF (a temple doorway on an island): a Greek orator, Demosthenes,
seated in the shadow of a temple of Poseidon, biting the end of a pen as if to
write, while spearmen laugh at the door; the pen holds poison.

BACKGROUND MOTIFS: the Roman Forum rostra, with a severed head and two hands
nailed above the bronze ships' beaks; in Alexandria, a huge serpent coiling
around the head of a crucified Spartan king to hide his face from the
birds (Cleomenes).

Colour key: grey-green dawn woods and cold steel; ochre and blood red on the
Capitol; a sickly violet sky over Alexandria.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-boys-of-st-aldates-round-the-world-library-no.-34/19146
https://digitalcollections.syr.edu/Documents/Detail/cover-of-george-arnolds-pluck-round-the-world-library-no.-173/19396
https://digitalcollections.syr.edu/Documents/Detail/cover-of-frank-merriwell-jrs-rivals-or-team-work-merriwell-series-no.-215/17048

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS VIII".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek and Roman dress, arms and architecture drawn
with period-pulp flair rather than museum exactness. Keep the top 18% of the
picture a flat, slightly mottled colour band for the masthead.
```

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Cicero stretches his neck from the litter | [*Cicero* 48.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg055.perseus-eng2:48.3/), [*Cicero* 48.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg055.perseus-eng2:48.4/) | “clasping his chin with his left hand, as was his wont, looked steadfastly at his slayers, his head all squalid and unkempt”; “he stretched his neck forth from the litter and was slain” |
| The crows | [*Cicero* 47.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg055.perseus-eng2:47.5/), [*Cicero* 47.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg055.perseus-eng2:47.6/) | “a flock of crows flew with loud clamour towards the vessel of Cicero”; “with its beak, little by little, tried to remove the garment from his face” |
| Head and hands on the rostra | [*Cicero* 49.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg055.perseus-eng2:49.1/) | “he ordered the head and hands to be placed over the ships’ beaks on the rostra” |
| Tiberius Gracchus clubbed with a bench leg | [*Tiberius Gracchus* 19.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg052.perseus-eng1:Tiberius.19.6/) | “he let his toga go and fled in his tunic … smote him on the head with the leg of a bench” |
| Caius Gracchus' friends hold the bridge | [*Caius Gracchus* 17.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg052.perseus-eng1:Caius.17.1/), [*Caius Gracchus* 17.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg052.perseus-eng1:Caius.17.2/) | “fighting there at the head of the bridge, would suffer no man to pass”; “escaping into a sacred grove of the Furies” |
| Demosthenes bites the poisoned pen | [*Demosthenes* 29.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg054.perseus-eng2:29.3/), [*Demosthenes* 29.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg054.perseus-eng2:29.4/) | “he put his pen to his mouth and bit it”; “The spear-men, then, who stood at the door, laughed at him” |
| The serpent on the crucified Cleomenes | [*Cleomenes* 39.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg051.perseus-eng1:Cleomenes.39.1/) | “a serpent of great size coiling itself about the head and hiding away the face so that no ravening bird of prey could light upon it” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Agis' mother finds her son and mother dead in the prison | [*Agis* 20.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg051.perseus-eng1:Agis.20.4/) | “she saw her son lying dead upon the ground, and her mother’s dead body still hanging in the noose” |
| Cleomenes' last charge through Alexandria | [*Cleomenes* 37.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg051.perseus-eng1:Cleomenes.37.3/) | “they seized the animal … and then rushed at full speed through the narrow streets” |
| Serpents hatch in Tiberius' helmet | [*Tiberius Gracchus* 17.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg052.perseus-eng1:Tiberius.17.2/) | “into this serpents crawled unnoticed, laid eggs there and hatched them out” |
| Licinia bars the door with their little son | [*Caius Gracchus* 15.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg052.perseus-eng1:Caius.15.2/) | “with one arm round her husband and the other round their little son” |
| Caius' head weighed in gold, filled with lead | [*Caius Gracchus* 17.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg052.perseus-eng1:Caius.17.4/) | “Septimuleius stuck the head of Cams on a spear … poured melted lead in its place” |
| Demosthenes practises with pebbles in his mouth | [*Demosthenes* 11.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg054.perseus-eng2:11.1/) | “taking pebbles in his mouth and then reciting speeches” |
| Demosthenes' underground study | [*Demosthenes* 7.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg054.perseus-eng2:7.3/) | “he built a subterranean study” |

*Caius Gracchus* 17.4 reads "Cams" in the Perseus file, a transcription slip for
"Caius"; the quotation keeps the file's reading.

---

## Tomus IX — Demetrius · Antony · Dion · Brutus

**Reference covers:** A, H, E.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for a volume of Plutarch's
Lives: siege-masters, lovers and conspirators.

DOMINANT SCENE (foreground, lower half): Cleopatra's barge coming up the river
Cydnus at golden evening: a gilded stern, sails of deep purple, silver oars
dipping to the sound of flutes and lutes. The queen reclines beneath a canopy
spangled with gold, dressed as Venus, fanned by boys dressed as Cupids; her
serving-women, dressed as sea-nymphs, work the rudders and the ropes. Clouds of
incense drift over crowds on both banks.

UPPER-LEFT VIGNETTE (dark sky over a walled harbour city): A colossal wooden
siege tower many storeys high, tapering to its top, rolling up to the walls of
Rhodes, missiles pouring from a window on every storey.

UPPER-RIGHT VIGNETTE (night tent, lamplight): A Roman general, Brutus, alone at a
camp table looks up from his book at a monstrous, silent phantom standing
beside him in the shadows of the tent.

MIDDLE MOTIF (the face of a tall stone tomb): Cleopatra and two women at a high
window hauling up a dying, blood-smeared Antony on ropes as he stretches his
hands toward her.

BACKGROUND MOTIFS: a basket of figs with a small asp lifting its head from the
leaves; a tall woman like a tragic Fury sweeping the colonnade of a Greek house
with a broom at dusk (the omen before Dion's death); two eagles riding on the
foremost legionary standards.

Colour key: Egyptian gold and royal purple on Nile-green water; smoke-black and
ember red over Rhodes; a cold lamp-yellow in the tent.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-among-the-pirates-round-the-world-library-no.-5/19086
https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-tour-of-the-zero-club-or-for-fame-and-fortune-round-the-world-library-no.-180/19410
https://digitalcollections.syr.edu/Documents/Detail/cover-of-jack-harkaway-after-schooldays-round-the-world-library-no.-120/19304

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS IX".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek, Roman and Egyptian dress, arms and
architecture drawn with period-pulp flair rather than museum exactness. Keep the
top 18% of the picture a flat, slightly mottled colour band for the masthead.
```

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Cleopatra's barge on the Cydnus | [*Antony* 26.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg058.perseus-eng2:26.1/), [*Antony* 26.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg058.perseus-eng2:26.2/) | “a barge with gilded poop, its sails spread purple, its rowers urging it on with silver oars to the sound of the flute blended with pipes and lutes”; “reclined beneath a canopy spangled with gold, adorned like Venus in a painting” |
| The city-taker at Rhodes | [*Demetrius* 21.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg057.perseus-eng2:21.1/), [*Demetrius* 21.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg057.perseus-eng2:21.2/) | “It rose to a height of sixty-six cubits, and tapered from base to summit”; “windows opening out of every storey, and out through these issued missiles of every sort” |
| “I am thy evil genius, Brutus” | [*Brutus* 36.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg061.perseus-eng2:36.6/), [*Brutus* 36.7](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg061.perseus-eng2:36.7/) | “a monstrous and fearful shape standing silently by his side”; “I am thy evil genius, Brutus, and thou shalt see me at Philippi” |
| Antony hauled up into the tomb | [*Antony* 77.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg058.perseus-eng2:77.1/), [*Antony* 77.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg058.perseus-eng2:77.2/) | “showed herself at a window, from which she let down ropes and cords”; “Smeared with blood and struggling with death he was drawn up, stretching out his hands to her” |
| The asp in the figs | [*Antony* 86.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg058.perseus-eng2:86.1/) | “the asp was brought with those figs and leaves and lay hidden beneath them” |
| The Fury with the broom | [*Dion* 55.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg060.perseus-eng2:55.2/) | “a woman of lofty stature, in garb and countenance exactly like a tragic Fury, sweeping the house with a sort of broom” |
| Eagles on the standards | [*Brutus* 37.7](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg061.perseus-eng2:37.7/) | “two eagles perched upon the foremost standards and were borne along with them” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Demetrius' giant galleys | [*Demetrius* 20.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg057.perseus-eng2:20.4/) | “his galleys of fifteen or sixteen banks of oars” |
| Demetrius' ashes come home in state | [*Demetrius* 53.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg057.perseus-eng2:53.2/) | “the cinerary vase was conspicuous on the vessel’s poop, adorned with royal purple and a king’s diadem” |
| Cleopatra's salted fish on Antony's hook | [*Antony* 29.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg058.perseus-eng2:29.4/) | “fastening on it a salted Pontic herring” |
| Cleopatra's squadron flees at Actium | [*Antony* 66.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg058.perseus-eng2:66.3/) | “the sixty ships of Cleopatra were seen hoisting their sails for flight” |
| Dion marches through burning Syracuse | [*Dion* 44.8](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg060.perseus-eng2:44.8/), [*Dion* 46.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg060.perseus-eng2:46.2/) | “scattering fiery arrows from their bows”; “through blood and fire and the mass of dead bodies lying in the streets” |
| Bees swarm on Dion's ships | [*Dion* 24.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg060.perseus-eng2:24.4/) | “the bees, which were seen settling in swarms upon the sterns of Dion’s transports” |
| Dion's murder | [*Dion* 57.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg060.perseus-eng2:57.4/) | “Lycon the Syracusan handed through the window to one of the Zacynthians a shortsword” |
| Porcia tests herself with a knife | [*Brutus* 13.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg061.perseus-eng2:13.5/) | “made a deep gash in her thigh” |
| Brutus falls on his sword | [*Brutus* 52.8](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg061.perseus-eng2:52.8/) | “Strato … with averted eyes, held the sword in front of him” |
| Porcia swallows live coals | [*Brutus* 53.5](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg061.perseus-eng2:53.5/) | “she snatched up live coals from the fire, swallowed them” |

---

## Tomus X — Artaxerxes · Aratus · Galba · Otho

**Reference covers:** I, F, D.

### Prompt

```text
A vintage 1930s pulp-magazine cover, portrait 2:3, for the last volume of
Plutarch's Lives: Persian palace poison, a night raid, and emperors murdered in
Rome.

DOMINANT SCENE (foreground): A night commando raid under a full summer moon.
A band of Greek soldiers in dark cloaks hurries scaling ladders up against the
wall of a hilltop citadel (Acrocorinth), their bronze helmets glinting in the
moonlight, while the leader, Aratus, waves them on. In a tower above, a huge
hunting dog bays at them, and sentries peer out with torches. Missiles fly
down from the citadel; the shouts echo off the rocks.

UPPER-LEFT VIGNETTE (Persian palace, perfumed lamplight): An imperious old
Persian queen-mother, Parysatis, in jewelled robes, slicing a small roasted bird
in two with a little knife; one blade-side glistens with poison as she offers
the clean half to herself and the poisoned half to a younger queen.

UPPER-RIGHT VIGNETTE (the Roman Forum, by day): An emperor's litter overturned
on the paving, an old bald emperor in a corselet sprawled beside it offering his
neck to the swords of soldiers; behind him a soldier whirls the severed head of
an old man on a spear "like a bacchanal". A toppled statue lies in the street.

BACKGROUND MOTIFS: a Persian prince in a purple tunic over his breastplate
reeling from his horse in the thick of battle (Cyrus at Cunaxa); a funeral pyre
at dawn where weeping legionaries light the wood with torches and then turn
their swords on themselves (the soldiers of Otho).

Colour key: moonlight blue and silver for the raid; Persian lamp-amber and
emerald; harsh noon white and red for the Forum.

Style references (attach the cover images from these pages):
https://digitalcollections.syr.edu/Documents/Detail/cover-of-the-young-railroaders-flyer-or-astride-the-iron-horse-round-the-world-library-no.-181/19412
https://digitalcollections.syr.edu/Documents/Detail/cover-of-phils-rivals-round-the-world-library-no.-162/19376
https://digitalcollections.syr.edu/Documents/Detail/cover-of-phils-rivals-round-the-world-library-no.-45/19170

Optional lettering: masthead "PLUTARCHI VITÆ"; corner badge "TOMUS X".

Style: an American pulp-magazine cover painting of the 1930s–1950s, oil or
gouache on board, reproduced in four-colour process with slightly coarse
halftone and faint registration drift. Saturated primaries against deep shadow:
hot reds, cadmium yellow, cobalt and ultramarine, with black-violet darks. Hard
theatrical lighting from one side (firelight, lightning, lamplight or a low sun)
and strong rim light on the figures. Muscular, dramatic, close-cropped figures
caught at the peak of the action, with wide eyes, clenched jaws and flying
cloaks. A diagonal composition that pushes the main action toward the viewer;
secondary scenes recede into smoke, cloud or flame behind it in the montage
manner of the period's covers. Visible brushwork, no photographic rendering, no
modern digital gloss. Ancient Greek, Persian and Roman dress, arms and
architecture drawn with period-pulp flair rather than museum exactness. Keep the
top 18% of the picture a flat, slightly mottled colour band for the masthead.
```

The guard dog in the prompt belongs to Aratus' earlier night raid on Sicyon,
which also used scaling ladders; the moonlit setting is from the Acrocorinth
raid. Merging them makes one stronger picture. Keep them apart if the
illustrator prefers to follow a single episode.

### Scene sources

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Acrocorinth by full moon | [*Aratus* 21.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg063.perseus-eng2:21.1/), [*Aratus* 21.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg063.perseus-eng2:21.3/) | “It was midsummer, the moon was at its full, and the night was cloudless and clear, so that they feared lest the gleam of their arms in the moonlight should disclose them”; “the ladders were clapped to the wall” |
| Missiles and echoing shouts | [*Aratus* 22.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg063.perseus-eng2:22.3/) | “assailed with missiles from the citadel … cries echoed round about which the reverberations from the hills rendered confused” |
| The dog in the tower at Sicyon | [*Aratus* 7.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg063.perseus-eng2:7.4/), [*Aratus* 8.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg063.perseus-eng2:8.1/) | “planted their ladders against it”; “the tower, in which a huge dog was on the watch, a hunter” |
| Parysatis' one-sided poisoned knife | [*Artaxerxes* 19.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg064.perseus-eng2:19.3/) | “Parysatis cut in two with a little knife smeared with poison on one side” |
| Galba murdered at the Lacus Curtius | [*Galba* 26.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg065.perseus-eng2:26.4/), [*Galba* 27.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg065.perseus-eng2:27.1/) | “the overthrow of a statue of Galba … the soldiers hurled javelins at the litter”; “The litter was upset at the place called Lacus Curtius, and there Galba tumbled out and lay in his corselet” |
| Galba's head on a spear | [*Galba* 27.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg065.perseus-eng2:27.3/) | “ran with it, like a bacchanal, whirling about often, and brandishing the spear” |
| Cyrus falls at Cunaxa | [*Artaxerxes* 11.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg064.perseus-eng2:11.6/) | “the tunics over their breastplates were of a purple colour, whereas all the king’s people wore white ones” |
| Otho's soldiers at his pyre | [*Otho* 17.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg066.perseus-eng2:17.4/) | “There were some, too, who first put their torches to the pyre and then slew themselves” |

Other strong scenes in this Tomus:

| Scene | Citation (Perrin) | Plutarch's words |
|---|---|---|
| Cyrus' fierce horse and the Cadusian commander | [*Artaxerxes* 9.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg064.perseus-eng2:9.1/) | “riding a horse that was high-bred, but fierce and hard to guide (his name was Pasacas …)” |
| Parysatis wins a eunuch at dice | [*Artaxerxes* 17.4](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg064.perseus-eng2:17.4/) | “since the dice also fell in her favour, she won the game, and selected Masabates” |
| Aratus' night march by the moon | [*Aratus* 7.2](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg063.perseus-eng2:7.2/) | “quickening or retarding his progress according to the revolution of the moon” |
| Headless senators in the Forum | [*Galba* 28.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg065.perseus-eng2:28.1/) | “the dead bodies, all headless in their consular robes, were still strewn over the forum” |
| Soldiers storm Otho's dinner party | [*Otho* 3.6](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg066.perseus-eng2:3.6/) | “the soldiers, forcing their way through the guards into the great hall, asked what was become of the enemies of Caesar” |
| Otho chooses between two swords | [*Otho* 17.1](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg066.perseus-eng2:17.1/) | “He had two swords, and after examining the blade of each for a long time, he laid one of them aside” |
| Otho's death | [*Otho* 17.3](https://scaife.perseus.org/reader/urn:cts:greekLit:tlg0007.tlg066.perseus-eng2:17.3/) | “with both hands he held his sword upright beneath him, and fell upon it” |

The "torture of the boats" in *Artaxerxes* 16.1–16.2 is the period's most
notorious atrocity but too grisly to show; it is left out.

---

## Sources

- **Plutarch, *Lives*, trans. Bernadotte Perrin**, Loeb Classical Library, 11
  vols. (London: Heinemann; Cambridge, Mass.: Harvard University Press,
  1914–1926). Public domain in the United States. Quotations were taken from,
  and checked against, Perseus Digital Library's TEI edition of this
  translation in [PerseusDL/canonical-greekLit](https://github.com/PerseusDL/canonical-greekLit/tree/master/data/tlg0007)
  on GitHub. The source file for each Life:
  - Theseus: [tlg0007.tlg001.perseus-eng3.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg001/tlg0007.tlg001.perseus-eng3.xml)
  - Romulus: [tlg0007.tlg002.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg002/tlg0007.tlg002.perseus-eng2.xml)
  - Lycurgus: [tlg0007.tlg004.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg004/tlg0007.tlg004.perseus-eng2.xml)
  - Numa: [tlg0007.tlg005.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg005/tlg0007.tlg005.perseus-eng2.xml)
  - Solon: [tlg0007.tlg007.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg007/tlg0007.tlg007.perseus-eng2.xml)
  - Publicola: [tlg0007.tlg008.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg008/tlg0007.tlg008.perseus-eng2.xml)
  - Themistocles: [tlg0007.tlg010.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg010/tlg0007.tlg010.perseus-eng2.xml)
  - Camillus: [tlg0007.tlg011.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg011/tlg0007.tlg011.perseus-eng2.xml)
  - Pericles: [tlg0007.tlg012.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg012/tlg0007.tlg012.perseus-eng2.xml)
  - Fabius Maximus: [tlg0007.tlg013.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg013/tlg0007.tlg013.perseus-eng2.xml)
  - Alcibiades: [tlg0007.tlg015.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg015/tlg0007.tlg015.perseus-eng2.xml)
  - Coriolanus: [tlg0007.tlg016.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg016/tlg0007.tlg016.perseus-eng2.xml)
  - Timoleon: [tlg0007.tlg018.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg018/tlg0007.tlg018.perseus-eng2.xml)
  - Aemilius Paulus: [tlg0007.tlg019.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg019/tlg0007.tlg019.perseus-eng2.xml)
  - Pelopidas: [tlg0007.tlg021.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg021/tlg0007.tlg021.perseus-eng2.xml)
  - Marcellus: [tlg0007.tlg022.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg022/tlg0007.tlg022.perseus-eng2.xml)
  - Aristides: [tlg0007.tlg024.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg024/tlg0007.tlg024.perseus-eng2.xml)
  - Cato the Elder: [tlg0007.tlg025.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg025/tlg0007.tlg025.perseus-eng2.xml)
  - Philopoemen: [tlg0007.tlg027.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg027/tlg0007.tlg027.perseus-eng2.xml)
  - Flamininus: [tlg0007.tlg028.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg028/tlg0007.tlg028.perseus-eng2.xml)
  - Pyrrhus: [tlg0007.tlg030.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg030/tlg0007.tlg030.perseus-eng2.xml)
  - Marius: [tlg0007.tlg031.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg031/tlg0007.tlg031.perseus-eng2.xml)
  - Lysander: [tlg0007.tlg032.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg032/tlg0007.tlg032.perseus-eng2.xml)
  - Sulla: [tlg0007.tlg033.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg033/tlg0007.tlg033.perseus-eng2.xml)
  - Cimon: [tlg0007.tlg035.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg035/tlg0007.tlg035.perseus-eng2.xml)
  - Lucullus: [tlg0007.tlg036.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg036/tlg0007.tlg036.perseus-eng2.xml)
  - Nicias: [tlg0007.tlg038.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg038/tlg0007.tlg038.perseus-eng2.xml)
  - Crassus: [tlg0007.tlg039.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg039/tlg0007.tlg039.perseus-eng2.xml)
  - Eumenes: [tlg0007.tlg041.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg041/tlg0007.tlg041.perseus-eng2.xml)
  - Sertorius: [tlg0007.tlg042.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg042/tlg0007.tlg042.perseus-eng2.xml)
  - Agesilaus: [tlg0007.tlg044.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg044/tlg0007.tlg044.perseus-eng2.xml)
  - Pompey: [tlg0007.tlg045.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg045/tlg0007.tlg045.perseus-eng2.xml)
  - Alexander: [tlg0007.tlg047.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg047/tlg0007.tlg047.perseus-eng2.xml)
  - Caesar: [tlg0007.tlg048.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg048/tlg0007.tlg048.perseus-eng2.xml)
  - Phocion: [tlg0007.tlg049.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg049/tlg0007.tlg049.perseus-eng2.xml)
  - Cato the Younger: [tlg0007.tlg050.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg050/tlg0007.tlg050.perseus-eng2.xml)
  - Agis and Cleomenes: [tlg0007.tlg051.perseus-eng1.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg051/tlg0007.tlg051.perseus-eng1.xml)
  - Tiberius Gracchus and Caius Gracchus: [tlg0007.tlg052.perseus-eng1.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg052/tlg0007.tlg052.perseus-eng1.xml)
  - Demosthenes: [tlg0007.tlg054.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg054/tlg0007.tlg054.perseus-eng2.xml)
  - Cicero: [tlg0007.tlg055.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg055/tlg0007.tlg055.perseus-eng2.xml)
  - Demetrius: [tlg0007.tlg057.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg057/tlg0007.tlg057.perseus-eng2.xml)
  - Antony: [tlg0007.tlg058.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg058/tlg0007.tlg058.perseus-eng2.xml)
  - Dion: [tlg0007.tlg060.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg060/tlg0007.tlg060.perseus-eng2.xml)
  - Brutus: [tlg0007.tlg061.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg061/tlg0007.tlg061.perseus-eng2.xml)
  - Aratus: [tlg0007.tlg063.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg063/tlg0007.tlg063.perseus-eng2.xml)
  - Artaxerxes: [tlg0007.tlg064.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg064/tlg0007.tlg064.perseus-eng2.xml)
  - Galba: [tlg0007.tlg065.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg065/tlg0007.tlg065.perseus-eng2.xml)
  - Otho: [tlg0007.tlg066.perseus-eng2.xml](https://github.com/PerseusDL/canonical-greekLit/blob/master/data/tlg0007/tlg066/tlg0007.tlg066.perseus-eng2.xml)
- **Free online reading copies** of the same translation, with the same
  chapter.section numbers: the Scaife viewer links above, and Bill Thayer's
  LacusCurtius edition at
  <https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Plutarch/Lives/home.html>.
- **Cover references:** the [Syracuse University Libraries Pulp Covers
  Collection](https://digitalcollections.syr.edu/pulp-covers-collection); item
  pages as listed above.
