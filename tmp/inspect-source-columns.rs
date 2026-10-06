use plutarch_reader::facsimile::{columns::{detect_latin_column,crop_column},lines::{line_bands,drop_furniture},reflow::{unit_body_strips,BODY_DPI},units::{load_units,ranges}};
fn main() {
 for (source,page) in [(0,296),(0,297),(0,305),(0,306),(1,626)] {
  let image=image::open(format!("build_facsimile/render_cache/Ploutarchou_Bioi_v{}-p{:04}-300dpi.png",source+1,page)).unwrap().to_luma8();
  let col=detect_latin_column(&image).unwrap();
  let crop=crop_column(&image,&col);
  let raw=line_bands(&crop,3);
  println!("source {source} page {page}: dimensions {:?}, crop {}..{}, bands {:?}, retained {:?}",image.dimensions(),col.left,col.right,raw,drop_furniture(&raw,image.height()));
  crop.save(format!("tmp/pdfs/inner_tomi_repairs/source-contact/s{source}-p{page}-detected.png")).unwrap();
 }
 let units=load_units(std::path::Path::new("facsimile_units.json")).unwrap();
 let range=ranges(&units,&[638,711]).into_iter().find(|r|r.unit.id=="timoleon").unwrap();
 let (pages,_)=unit_body_strips(&range,|s,p|Ok(image::open(format!("build_facsimile/render_cache/Ploutarchou_Bioi_v{}-p{:04}-{BODY_DPI}dpi.png",s+1,p))?.to_luma8())).unwrap();
 for (p,(_,bands)) in (range.unit.start_page..=range.end_page).zip(&pages) { println!("Timoleon source {p}: {bands:?}");}
}