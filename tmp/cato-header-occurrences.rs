use plutarch_reader::facsimile::lines::line_bands;
fn main(){for p in [337,347]{
let im=image::open(format!("build_facsimile/render_cache/Ploutarchou_Bioi_v2-p{p:04}-300dpi.png")).unwrap().to_luma8();
println!("source1/{p} dims {:?}",im.dimensions());
for (left,right) in [(100,im.width()-80),(1000,im.width()-80)]{
let c=image::imageops::crop_imm(&im,left,0,right-left,400).to_image();
println!("x{left}..{right} upper bands {:?}",line_bands(&c,3));}
image::imageops::crop_imm(&im,0,0,im.width(),400).to_image().save(format!("tmp/pdfs/inner_tomi_repairs/rebuilt/cato-source-{p}-top.png")).unwrap();
}}