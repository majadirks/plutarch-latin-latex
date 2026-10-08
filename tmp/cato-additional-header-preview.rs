fn main(){
 let raw=image::open("build_facsimile/render_cache/Ploutarchou_Bioi_v2-p0347-300dpi.png").unwrap().to_luma8();
 let mut masked=raw.clone();
 for y in 145..205 {for x in 140..1720 {masked.put_pixel(x,y,image::Luma([255]));}}
 for y in 0..raw.height(){for x in 0..raw.width(){if !(145..205).contains(&y)||!(140..1720).contains(&x){assert_eq!(raw.get_pixel(x,y),masked.get_pixel(x,y));}}}
 image::imageops::crop_imm(&masked,0,0,masked.width(),400).to_image().save("tmp/pdfs/inner_tomi_repairs/rebuilt/cato-source-347-header-candidate.png").unwrap();
 let mut boxed=raw;
 for x in 140..1720{for y in [145,204]{boxed.put_pixel(x,y,image::Luma([120]));}}
 for y in 145..205{for x in [140,1719]{boxed.put_pixel(x,y,image::Luma([120]));}}
 image::imageops::crop_imm(&boxed,0,0,boxed.width(),400).to_image().save("tmp/pdfs/inner_tomi_repairs/rebuilt/cato-source-347-header-box.png").unwrap();
 println!("s1/347 proposed header [140,145,1720,205); all outside pixels unchanged; first source body ink starts row219");
}