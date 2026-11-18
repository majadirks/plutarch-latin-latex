# Pulp scene drawings: decisions and working plan

## Context

Create separate, moderately detailed line drawings of the constituent scenes of
the ten Plutarch cover montages. The owner will print them on US Letter paper,
trace and draw them by hand, and digitally assemble the finished drawings into
covers. Figures and essential props, and architecture and scenery, must be
supplied as separate assets. This work will span multiple LLM sessions.

**Current authorization: planning and documentation. Do not generate images
until the owner explicitly says to do so.** Move existing cover references into
the corresponding tomus folders as work on each tomus proceeds, not during this
initial documentation step.

## Current state of the world

- `llm_artifacts/2026_10_25_15_25_PULP_COVER_PROMPTS.md` contains the ten original
  prompts, scene citations, and optional alternative scenes.
- `pulp_cover_example/` contains `tomus_01_pulp_cover_example.png` through
  `tomus_10_pulp_cover_example.png` and the corresponding
  `tomus_01_line_drawing.png` through `tomus_10_line_drawing.png` (20 files).
- Those images are visual references. Their poses and compositions should
  generally guide the new drawings, but recomposition and AI variation are allowed.
- The prompt document's tomus membership table is derived from an earlier build.
  Its own instructions require checking actual tomus title pages before committing
  to artwork. Do not assume its membership table describes every later build.
- The owner confirms that the tomi are not machine readable. Determine which
  Lives each contains from build metadata and visual inspection of title/contents
  pages; use online texts of those Lives to research and verify individual scenes.
  Do not require full-text extraction or OCR of the tomi for this art workflow.
- No scene assets have been generated for this plan; no reference files have
  been moved. Prior visual inspection was representative, not a completed
  scene-by-scene audit of all twenty images.
- The repository's Latin transcription cache is source text. This art workflow
  must not change or normalize Latin source text or modify the book generator.

## Desired end state

Each tomus has preserved original cover references, a verified scene inventory,
separate PNG assets for figures/essential props and environment/architecture,
and exact saved generation prompts. Every selected scene closely follows an
episode in a Life actually present in that tomus. The checklist records what
exists, what has passed visual inspection, and what the owner has accepted.
The owner can resume in a later session without reconstructing prior decisions.
The deliverables are drawing bases; producing the owner's finished hand drawings
or final assembled covers is outside this plan.

## Confirmed decisions and source precedence

1. Preserve the general poses, composition, and dramatic pulp character of the
   references where practical; exact copies are unnecessary and recomposition is
   permitted.
2. Use moderate detail: clear contours, readable faces and hands, selected anatomy
   and fabric folds, and essential object details. Avoid dense hatching, heavy
   black shadow masses, and exhaustive fur, foliage, or masonry texture.
3. Supply figures and essential interacting props separately from architecture
   and scenery. Multiple PNGs per scene are expected; two is not a hard limit.
4. Trust the written prompt over accidental content in the generated reference
   image. **The owner's subsequent clarification takes priority over both:
   scenes must closely adhere to actual contents of the relevant tomus.**
5. Verify actual tomus membership and the episode's passage before generating.
   A prompt citation to a different Life is not sufficient. A scene also told in
   an included Life may qualify, but verify that passage rather than rejecting
   or accepting it from its character's name alone.
6. When a prompt conflicts with the actual contents, record the discrepancy in
   that tomus's scene inventory and correct the proposed scene. Do not silently
   carry over a cross-volume episode or merge separate episodes. Ask the owner
   about substantive replacements when there is no clear supported correction.
7. Print for tracing on US Letter (8.5 × 11 inches). PNGs are required. Use
   descriptive filenames and save exact prompts. Move the existing references
   into the tomus folders as work proceeds, preserving their bytes and names.
8. Do not generate until the owner explicitly authorizes generation. Retain this
   plan as a historical record; update its current text and checkboxes in place.

## Working defaults to calibrate with the pilot

These are proposed production defaults, not additional owner decisions.

- Black linework on white for printing; no lettering, frames, cover banners,
  artificial aged-paper effects, grayscale washes, or color.
- Keep complete silhouettes and essential props inside generous margins. Supply
  missing limbs or objects coherently rather than retaining montage occlusions.
- A practical Letter printing area is 7.5 × 10 inches with half-inch margins;
  landscape orientation is allowed. Record intended printed size and print
  scaling for each asset. Aim for sufficient native resolution for clean tracing;
  2250 × 3000 pixels corresponds to that portrait area at 300 pixels per inch,
  but do not promise that exact native output from the built-in tool. Inspect
  actual dimensions and line quality; enlarged pixels do not add drawing detail.
- Put contact-critical objects with the figures: weapons, reins, seats, ladders,
  ropes, or a short support surface as needed. Put the wider setting in the
  environment asset. Document any repeated alignment aid so it is not traced
  twice. Preserve compatible viewpoint, horizon, scale, and contact locations.
- Boats, siege engines, horses, crowds, and complex action may require additional
  component assets rather than a forced two-file split. Do not separate a hand
  from its held object or a rider from the horse merely to meet a category rule.
- White-background PNGs satisfy tracing. Transparent companion versions can be
  added if useful for assembly; they are not required by the owner at this stage.
- Begin with one pilot, provisionally Theseus and the Minotaur, before applying
  a style to a whole tomus. Pilot choice and acceptance remain pending.

## Folder and record contract

Use the existing project convention of descriptive names and dated markdown
artifacts; no Rust changes or new generation framework are needed.

```text
pulp_cover_scenes/
  tomus_01/
    references/
      tomus_01_pulp_cover_example.png
      tomus_01_line_drawing.png
    SCENES.md
    scenes/
      t01_s01_theseus_minotaur/
        figures_v01.png
        environment_v01.png
        figures_v01.prompt.md
        environment_v01.prompt.md
```

Repeat for `tomus_02` through `tomus_10`. Allocate stable scene IDs from the
inventory below. Append letter suffixes for additional scenes without renumbering
existing IDs. Extra components use descriptive names such as
`bystanders_v01.png` and matching `.prompt.md` files. Increment versions for
revisions; never overwrite a selected asset or original reference.

Each `SCENES.md` records actual book/title-page evidence, verified Lives,
scene IDs and citations to included Lives (online edition, passage URL, chapter
and section, and access date), source discrepancies and resolutions,
component boundaries, intended size/orientation, current selected versions,
reference move provenance and SHA-256 hashes, and outstanding decisions.

Each `.prompt.md` records the exact submitted prompt, tool used, reference image
paths and their roles, actual output filename and pixel dimensions, intended
print size, and inspection findings. Save the complete submitted text even if
it repeats shared instructions; centralize the reusable style specification in
`pulp_cover_scenes/STYLE.md` after pilot acceptance to avoid drifting independent
style definitions. This duplication of submitted prompt text is intentional
provenance, not a second authoritative style specification.

## Phase 1 — Verify membership and decompose scenes — Suggested implementer: Opus

### 1.1 Establish the actual contents of each tomus

Locate the intended ten-tomus build and establish its included Lives from relevant
build metadata and visual inspection of title/contents pages. The tomi are not
machine readable: do not depend on PDF text search or extract their entire text.
Record exact local evidence paths and inspected page numbers in each `SCENES.md`. If the
intended build is unavailable or ambiguous, ask which build to use before
generating affected assets. Do not regenerate books or alter their split for art.

**LLM verification:** use `rg --files tmp -g '*tomus*' -g '*body_map*'` to locate
build evidence and inspect the relevant metadata and rendered title/contents pages.
Use existing rendering tools if necessary; this does not require OCR. Compare with the
original prompt document's membership table. Record discrepancies explicitly.

**Human verification:** confirm the inventory describes the books being illustrated.

### 1.2 Verify episodes and specify components for the next tomus

Work one tomus at a time. Read its full original prompt, including motifs, and
the relevant online passages in the Lives established in 1.1, using the sources
below. Preserve passage-level citations. Treat the inventory
below as candidates, not approved historical claims. Exclude the original
document's optional “Other strong scenes” from initial production unless selected
as a supported replacement or later requested. Create a component checklist per
scene in `SCENES.md`; enumerate every prompted figure/motif even if several share
an episode. Explain proposed omissions or corrections.

**LLM verification:** compare each scene against the full prompt and a passage in
an included Life. Confirm every prompted component is accounted for and every
scene has a figures/props and environment disposition (including “not needed”
with a reason where appropriate).

**Human verification:** review substantive scene corrections and component splits.

### Online texts for scene research

Use [LacusCurtius: Plutarch's Lives](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Plutarch/Lives/home.html)
as the first reading source. Verified accessible on 2026-11-17, its index links
all the Lives needed by the original ten-tomus membership table, including
Artaxerxes, Aratus, Galba, and Otho. It presents Bernadotte Perrin's English
translation in readable HTML with chapter and section anchors for passage links.
This is the same translation cited by the original cover-prompt document.

The [Perseus Digital Library text repository](https://github.com/PerseusDL/canonical-greekLit/tree/master/data/tlg0007)
provides a structured-text fallback. The original prompt document's Sources
section already identifies the particular Perrin English TEI XML file for each
Life; follow those mappings rather than guessing filenames or selecting a Greek
text by accident. Its existing Scaife passage links provide another reading route.
Prefer LacusCurtius for routine reading; use the mapped Perseus text when access,
searchability, or passage checking requires it.

Keep two separate evidence chains: local build/title-page evidence establishes
**which Lives belong to a tomus**; the online text establishes **what happens in
an episode of an included Life**. An online edition's volume divisions do not
establish this project's tomus boundaries. Read the surrounding passage to check
participants, actions, props, and setting, distinguishing Plutarch's narrative
from translator/editor notes. Record meaningful uncertainties or edition
differences instead of inventing precision. English translations guide artwork;
they do not authorize changes to the cached Latin or establish verbatim identity
with this project's printed Latin edition.

## Candidate scene inventory and progress

Every unchecked row awaits passage verification and production. A checked row
means all its required components exist, prompts are saved, visual QA is complete,
and the owner has accepted the scene. Record intermediate progress in `SCENES.md`.
Do not equate generation with acceptance.

| Done | ID | Prompt-derived candidate scene; verify against actual tomus |
|---|---|---|
| [ ] | t01_s01 | Theseus and Minotaur; thread, labyrinth, bones, Athenian captives |
| [ ] | t01_s02 | Romulus disappearing in a storm before a crowd |
| [ ] | t01_s03 | Aegeus leaping from the cliff; black-sailed galley |
| [ ] | t01_s04 | She-wolf, twins, trough, river, and woodpecker |
| [ ] | t01_s05 | Tarpeia beneath armlets and shields |
| [ ] | t01_s06 | Mucius Scaevola at the brazier before the Etruscan king |
| [ ] | t02_s01 | Salamis: ramming triremes, Ariamenes, boarding marines, owl |
| [ ] | t02_s02 | Xerxes on his throne with scribes overlooking the fleet |
| [ ] | t02_s03 | Capitol geese, climbing Gauls, and Roman sentries |
| [ ] | t02_s04 | Brennus adding sword and belt to the gold scales |
| [ ] | t02_s05 | Hannibal's oxen with torches on their horns in a mountain pass |
| [ ] | t02_s06 | Alcibiades escaping a burning house under attack |
| [ ] | t03_s01 | Syracuse: Archimedes, claw, suspended galley, falling weight and second ship |
| [ ] | t03_s02 | Disguised Theban conspirators attacking at a banquet |
| [ ] | t03_s03 | Crimisus: Carthaginian spearmen/chariots and Greek charge in a storm |
| [ ] | t03_s04 | Macedonian phalanx with levelled spears and interlocked shields |
| [ ] | t03_s05 | Marcellus unhorsing the armoured Gallic king |
| [ ] | t03_s06 | Lunar eclipse above a sleeping camp |
| [ ] | t04_s01 | Pyrrhus at Argos: mounted combat, rooftop woman and tile, two elephants |
| [ ] | t04_s02 | Marius confronting the fleeing executioner |
| [ ] | t04_s03 | Cimbrian women on wagons and stampeding cattle |
| [ ] | t04_s04 | Flamininus' proclamation, stadium crowd and falling ravens |
| [ ] | t04_s05 | Philopoemen breaking the javelin through his thighs |
| [ ] | t04_s06 | Sulla kissing the small figure of Apollo |
| [ ] | t05_s01 | Monime, snapped diadem, armed eunuch, princess with poison cup |
| [ ] | t05_s02 | Armenian armoured cavalry routing before Roman infantry |
| [ ] | t05_s03 | Eurymedon fleet action and hoplites landing against Persian troops |
| [ ] | t05_s04 | Haunted bath at Chaeroneia: apparition, doorway, vapour |
| [ ] | t05_s05 | Gold-laden mule, soldiers fighting over treasure, escaping king |
| [ ] | t06_s01 | Carrhae: Parthian cavalry, drums, Roman square and Crassus |
| [ ] | t06_s02 | Spartacus' gladiators descending the cliff on vine ladders |
| [ ] | t06_s03 | Sleeping Spartacus with a snake around his face |
| [ ] | t06_s04 | Pompey's body, freedman, broken fishing boat and funeral pyre |
| [ ] | t06_s05 | Lunar eclipse above the Greek fleet in harbour |
| [ ] | t06_s06 | Athenians drinking in the river while attacked |
| [ ] | t06_s07 | Sertorius and the white fawn in Spanish hill country |
| [ ] | t06_s08 | Eumenes and opposing commander grappling on rearing horses |
| [ ] | t07_s01 | Alexander, Thaïs and companions burning Persepolis |
| [ ] | t07_s02 | Caesar's assassination beside Pompey's statue |
| [ ] | t07_s03 | Mounted Caesar and cavalry crossing the Rubicon |
| [ ] | t07_s04 | Young Alexander turning Bucephalus toward the sun |
| [ ] | t07_s05 | Brutus' apparition: verify whether Caesar's Life supports this scene here |
| [ ] | t07_s06 | Cleopatra carried to Caesar in a tied bedding sack |
| [ ] | t08_s01 | Cicero's death at the litter, soldiers and crow |
| [ ] | t08_s02 | Tiberius Gracchus attacked with broken bench legs |
| [ ] | t08_s03 | Bridge defenders and Caius Gracchus fleeing with a servant |
| [ ] | t08_s04 | Demosthenes with poisoned pen at Poseidon's temple |
| [ ] | t08_s05 | Cicero's head and hands displayed on the rostra |
| [ ] | t08_s06 | Serpent shielding the crucified Cleomenes' head |
| [ ] | t09_s01 | Cleopatra's Cydnus barge, attendants, musicians and riverbank crowds |
| [ ] | t09_s02 | Siege tower attacking Rhodes |
| [ ] | t09_s03 | Brutus reading in his tent confronted by the phantom |
| [ ] | t09_s04 | Cleopatra and attendants hauling Antony to the tomb window |
| [ ] | t09_s05 | Asp rising from the basket of figs |
| [ ] | t09_s06 | Fury-like woman sweeping the colonnade before Dion's death |
| [ ] | t09_s07 | Two eagles on legionary standards |
| [ ] | t10_s01 | Aratus' raid: choose a coherent episode; do not merge Acrocorinth and Sicyon |
| [ ] | t10_s02 | Parysatis, poisoned knife, roasted bird and younger queen |
| [ ] | t10_s03 | Galba's overturned litter, soldiers, head on spear and toppled statue |
| [ ] | t10_s04 | Cyrus falling at Cunaxa |
| [ ] | t10_s05 | Otho's funeral pyre and grieving soldiers' deaths |

The original Tomus X note explicitly combines the Sicyon guard dog with the
Acrocorinth moonlit raid. Select one episode and its supported details, or propose
a separately identified scene; do not reproduce that composite as one event.
Repeated subjects across tomi need independent membership/passage checks. Share
an asset only when the owner wants it reused and its episode, pose and framing fit.

## Phase 2 — Organize references and calibrate the pilot — Suggested implementer: Opus

### 2.1 Move references for the tomus being worked on

Create that tomus's directory and `references/`. Resolve source and destination
absolute paths and verify they stay under this workspace before moving files.
Use PowerShell `Get-FileHash`, then `Move-Item -LiteralPath` with explicit paths;
do not overwrite an existing destination. Verify hashes afterward and record old
and new paths in `SCENES.md`. Check `rg -n 'pulp_cover_example|tomus_01_line_drawing'`
(substitute the current tomus) for references that need attention. Preserve the
historical prompt document; record any stale historical paths in the manifest.

**LLM verification:** compare SHA-256 hashes and confirm exactly two original
reference files at the destination. Inspect `git diff --stat` and `git status
--short` for unintended changes; do not alter unrelated user work.

**Human verification:** confirm both original references remain accessible.

### 2.2 Produce and evaluate one pilot after generation is authorized

Read the imagegen skill, inspect the chosen references, and use the built-in
image tool. Generate the pilot figures/props and environment separately with
compatible geometry. Save PNGs and exact prompts according to the folder contract.
Copy project deliverables from the tool's default output directory into the
workspace. Inspect outputs and make focused revisions as needed. Do not use a
CLI/API fallback unless the owner explicitly chooses it.

**LLM verification:** verify image files open, actual dimensions are recorded,
prompts match their outputs, all essential content is present, and there are no
obvious anatomy, held-object, perspective, clipping, or accidental text defects.
Compare the two components together for compatible placement and contact points.

**Human verification:** print on Letter at the recorded size; assess tracing
clarity, density, hand/face readability, and ease of combining the components.
Record owner feedback and acceptance. Save accepted style in `STYLE.md`.

## Phase 3 — Complete the tomi incrementally — Suggested implementer: Opus

### 3.1 Repeat the verified asset workflow for each remaining scene

Proceed in tomus order unless directed otherwise. Complete 1.1/1.2 and 2.1 for
each new tomus, then apply the accepted pilot style. Generate one asset per tool
call, save the exact submitted prompt, inspect it, and record its status before
starting another. Complete required environment assets as well as figures.
Keep pending revision and pending owner acceptance distinct from completion.

**LLM verification:** enumerate files with `rg --files pulp_cover_scenes`; compare
against component lists in `SCENES.md`. Confirm each selected PNG has a matching
prompt and each episode has supporting evidence from an included Life.

**Human verification:** review each scene and representative Letter prints;
request revisions before accepting a tomus's set.

### 3.2 Close each session with a durable handoff

Update the existing manifests and checklist in place: completed assets, selected
versions, defects, owner feedback, blocked source questions, and exact next
scene/component. Preserve stable scene IDs and all original references. Report
saved paths to the owner. Do not claim unseen images or unprinted sheets passed
human verification.

**LLM verification:** re-read the handoff and confirm another session can locate
the source evidence, references, current assets, prompts and next task from disk.

**Human verification:** confirm the recorded next action and open questions match
the owner's intent.

## Session checklist and resumption point

- [x] Record confirmed decisions and the subsequent tomus-content requirement.
- [x] Create prompt-derived candidate inventory with stable IDs.
- [ ] Identify and verify the intended ten-tomus build.
- [ ] Verify Tomus I episodes and component splits.
- [ ] Receive explicit authorization to generate images.
- [ ] Move Tomus I reference files with hash verification.
- [ ] Generate and save pilot components and exact prompts.
- [ ] Obtain Letter-print feedback and pilot acceptance.
- [ ] Save accepted shared style specification.
- [ ] Complete and accept Tomus I.
- [ ] Complete and accept Tomus II.
- [ ] Complete and accept Tomus III.
- [ ] Complete and accept Tomus IV.
- [ ] Complete and accept Tomus V.
- [ ] Complete and accept Tomus VI.
- [ ] Complete and accept Tomus VII.
- [ ] Complete and accept Tomus VIII.
- [ ] Complete and accept Tomus IX.
- [ ] Complete and accept Tomus X.

**Next action:** offer plan review; await owner direction. No generation is
authorized. Membership and episode verification can proceed without generation.
No scene folders exist yet under this plan.

**Validation scope:** this is an artwork/documentation workflow and changes no
Rust or generator behavior, so Rust tests are not needed for this documentation
change. If implementation later expands into Rust changes, add explicit behavior
tests and run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test` before declaring
that code work done. Do not expand into code changes merely to generate art.
