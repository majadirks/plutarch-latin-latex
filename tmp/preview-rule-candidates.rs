use plutarch_reader::facsimile::{columns::{detect_latin_column,crop_column},lines::line_bands,reflow::{unit_body_strips,compose_pages,TrimGeom,BODY_DPI},units::{load_units,ranges}};
fn candidate(source:usize,page:u32)->bool { matches!((source,page),(0,281|287|296|298|299|304|306|308|470)|(1,359|361|539|625)) }
fn main(){
 let dir=std::path::Path::new("tmp/pdfs/inner_tomi_repairs/candidate_rules"); std::fs::create_dir_all(dir).unwrap();
 for (s,p) in [(0,281),(0,287),(0,296),(0,298),(0,299),(0,304),(0,306),(0,308),(0,470),(1,359),(1,361),(1,539),(1,625)] {
  let raw=image::open(format!("build_facsimile/render_cache/Ploutarchou_Bioi_v{}-p{:04}-300dpi.png",s+1,p)).unwrap().to_luma8();
  let original=detect_latin_column(&raw).unwrap();
  let mut cleaned=raw.clone();let left=if s==0 && p==281 {1900} else {raw.width()-25};
  for y in 0..raw.height(){for x in left..raw.width(){cleaned.put_pixel(x,y,image::Luma([255]));}}
  let col=detect_latin_column(&cleaned).unwrap(); let crop=crop_column(&cleaned,&col);
  println!("s{s}/p{p} dims {:?} candidate rectangle [{left},0,{},{}] normalized left {:.12}; crop {}..{} -> {}..{}; bands {}",raw.dimensions(),raw.width(),raw.height(),left as f64/raw.width() as f64,original.left,original.right,col.left,col.right,line_bands(&crop,3).len());
  crop.save(dir.join(format!("s{s}-p{p}-candidate-column.png"))).unwrap();
  let mut box_image=raw.clone();
  for y in 0..raw.height(){ box_image.put_pixel(left,y,image::Luma([120]));}
  box_image.save(dir.join(format!("s{s}-p{p}-boundary.png"))).unwrap();
 }
 let units=load_units(std::path::Path::new("facsimile_units.json")).unwrap(); let geom=TrimGeom::default();
 for id in ["timoleon","marcius_coriolanus","brutus","pyrrhus","cato_minor","antonius"]{
  let range=ranges(&units,&[638,711]).into_iter().find(|r|r.unit.id==id).unwrap();
  let (pages,metrics)=unit_body_strips(&range,|s,p|{
   let mut img=image::open(format!("build_facsimile/render_cache/Ploutarchou_Bioi_v{}-p{:04}-{BODY_DPI}dpi.png",s+1,p))?.to_luma8();
   if candidate(s,p){for y in 0..img.height(){for x in (if s==0 && p==281 {1900} else {img.width()-25})..img.width(){img.put_pixel(x,y,image::Luma([255]));}}}
   Ok(img)
  }).unwrap();
  for (p,(_,bands)) in (range.unit.start_page..=range.end_page).zip(&pages){if candidate(range.unit.source,p){println!("{id} candidate page {p}: {bands:?}");}}
  let canvases=compose_pages(&pages,1.2023952,&geom,geom.first_body_budget(),geom.body_budget(),&metrics);
  println!("{id} candidate canvas count {}",canvases.len());
  for (i,c) in canvases.iter().enumerate(){c.save(dir.join(format!("{id}-candidate-{:03}.png",i+1))).unwrap();}
 }
}