//! 临时：扫描 PNG 的非透明像素 bbox（诊断用）。
//! 用法：cargo run -p lp-render --example scan_png -- <path.png>

use image::ImageBuffer;
use std::env;

fn main() {
    let path = env::args().nth(1).expect("需要 PNG 路径");
    let img: ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::open(&path).unwrap().to_rgba8();
    let (w, h) = img.dimensions();
    let (mut minx, mut miny, mut maxx, mut maxy) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let mut cnt = 0u32;
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            if p.0[3] > 10 {
                cnt += 1;
                if x < minx { minx = x; }
                if x > maxx { maxx = x; }
                if y < miny { miny = y; }
                if y > maxy { maxy = y; }
            }
        }
    }
    println!("画布 {w}x{h}, 非透明 {cnt} 像素");
    if cnt > 0 {
        println!("bbox x[{minx}..{maxx}] y[{miny}..{maxy}] 宽{} 高{}", maxx-minx+1, maxy-miny+1);
    } else {
        println!("全透明");
    }
}
