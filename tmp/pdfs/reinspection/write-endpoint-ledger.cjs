const fs=require('fs'),d='tmp/pdfs/reinspection';
const rows=JSON.parse(fs.readFileSync(`${d}/endpoint-map.json`,'utf8').replace(/^\uFEFF/,''));
const anchors=`appellamus.
homines non ultra visum.
non fuisse diis cordi natales Thesei
Atque hæc de Lycurgo.
siquidem traditur fulmine percussus conflagrasse.
redegit atque composuit.
tum Aristoteles etiam philosophus.
mortuus inde aufertur.
jungeret, et omnem belli apparatum tolleret.
cum degeremus una apud Ammonium philosophum.
dolorem attulerit.
omni remedio superior evaderet.
quæ vitæ responderent, invenit.
periorem habuerunt.
proripientem telis confecisse.
spondent s.
honestatisque negligentissimo homine.
citate exegerunt.
Hi mores et vita Æmilii Pauli fuisse dicuntur.
pusilli tamen animi est.
tiis passus judicari potest.
Cæsar ipse theatrum Marcello inscriptum.
virtus amatur.
merito in admiratione est atque suspicitur.
ætatis homines fuit clarissimus.
satis idoneus affinis futurus.
Atque hæc quidem de Philopœ-
restat ut comparationem ipsam inspiciamus.
probene arbitrium tulerimus.
præbuit.
mortem sibi ipse conscivit.
commemoranda habuimus.
mico maleficiis superatum fuisse.
fortitudinis primam laudem deferamus.
Talis quidem Græcorum erat dux.
que mortis ab obitu ejus non multum distans sortitus est.
fuisse.
quæ eventura sæpe prædixerat.
pendio utens eum suffocavit.
adjecit.
exosus consenuit.
cum mare inspicere posset.
potemque fecit.
Leonidas interfecit.
Pompeii sibi allatas Cornelia in Albano humavit.
defectionem ad hostes eorum fecit.
ptus, sanæ mentis jacturam fecisse.
ictui vehementiam addente mortuus est.
et error fuit et Athenarum detrimentum.
apud Philippos periit.
ceteris credidisset.
verunt.
Vita Scipionis singulatim scriptum.
nostros moderate ferendi facultatem ei non adimit.
haud paullo inferiorem fuisse judicaverim.
accepimus conscriptam.
intulit.
Antipatri quasi subsannans crudelitatem.
tempus est ut Romana in scenam mittatur.
Quintus ab Antonio ratione stirpis fuit.
sese vitæ subduxit.
sumus in Vita Timoleontis prosecuti.
ris et mortis modum significat.
loco relinqui.
Ochus, crudelitate et paricidiis omnibus superior.
propagatum est.
reliquit.
se conjunxerunt.`.split('\n');
if(rows.length!==68||anchors.length!==68)throw Error(`counts ${rows.length}/${anchors.length}`);
const romans=['','I','II','III','IV','V','VI','VII','VIII'];
function folio(s,p){if(s===1)return p-13;if(p<=102)return p+607;if(p===105)return 710;if(p<=118)return p+605;if(p<=122)return p+601;if(p<=212)return p+601;if(p<=214)return p+599;if(p<=296)return p+599;if(p<=298)return p+597;if(p<=338)return p+597;if(p<=348)return p+587;if(p<=360)return p+587;if(p<=362)return p+585;if(p<=368)return p+585;if(p<=370)return p+583;if(p<=376)return p+583;if(p<=378)return p+581;if(p<=464)return p+581;if(p<=466)return p+579;if(p<=522)return p+579;if(p<=524)return p+577;if(p<=672)return p+577;if(p<=674)return p+575;if(p<=700)return p+575;if(p<=702)return p+573;return p+573;}
let table='| Endpoint ID | Unit | Physical inner page | Reader folio | Source physical / printed folio | Short terminal Latin anchor | Assessment |\n| --- | --- | --- | ---: | --- | --- | --- |\n';
rows.forEach((r,i)=>{if(r.id==='comparatio_sertorii_et_eumenis')r.source=105;table+=`| E${String(i+1).padStart(2,'0')} | ${r.id.replaceAll('_',' ')} | ${romans[r.t]}/${r.p} | ${r.folio} | v${r.s}/${r.source}, f${folio(r.s,r.source)} | \`${anchors[i]}\` | Confirmed complete |\n`;r.anchor=anchors[i];r.printed_source_folio=folio(r.s,r.source);});
fs.writeFileSync(`${d}/endpoint-ledger.md`,table);
fs.writeFileSync(`${d}/endpoint-ledger.json`,JSON.stringify(rows,null,2));
fs.appendFileSync('llm_artifacts/2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md',table+'\n');
