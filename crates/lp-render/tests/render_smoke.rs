//! P1 渲染冒烟测试 + 像素采样验证（docs 15-P1渲染最小闭环 §模块5）。
//!
//! 验证：渲染一个已知矩形 region，采样其中心像素应 == region 颜色，
//! 背景像素应 == 透明。

use lp_render::{RegionDraw, Renderer};

/// 在 width×height 画布上渲染一个居中的 100×100 矩形，验证像素。
fn render_centered_rect(width: u32, height: u32, color: [f32; 4]) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    // 用临时目录写 PNG 再读回（P1 渲染器只暴露 PNG 输出接口）
    let tmp = std::env::temp_dir().join(format!("lp_render_smoke_{width}x{height}.png"));
    let renderer = Renderer::new(width, height);

    // 居中 100×100 矩形（像素坐标，y 向上）
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let half = 50.0;
    let draws = vec![RegionDraw {
        vertices: [
            [cx - half, cy - half, 0.0, 0.0], // 左下
            [cx + half, cy - half, 1.0, 0.0], // 右下
            [cx + half, cy + half, 1.0, 1.0], // 右上
            [cx - half, cy + half, 0.0, 1.0], // 左上
        ],
        color,
    }];
    renderer.render_to_png(&draws, &tmp);

    let img = image::open(&tmp).unwrap().to_rgba8();
    let _ = std::fs::remove_file(&tmp);
    img
}

#[test]
fn renders_solid_color_at_center() {
    let color = [0.8, 0.2, 0.4, 1.0]; // 粉红
    let expected: [u8; 4] = [
        (color[0] * 255.0_f32).round() as u8,
        (color[1] * 255.0_f32).round() as u8,
        (color[2] * 255.0_f32).round() as u8,
        255,
    ];
    let img = render_centered_rect(128, 128, color);

    // 中心像素应是矩形颜色
    let center = img.get_pixel(64, 64);
    assert_eq!(center.0, expected, "中心像素颜色不对");

    // 矩形内另一像素也应是该颜色
    let inside = img.get_pixel(70, 70);
    assert_eq!(inside.0, expected, "矩形内像素颜色不对");
}

#[test]
fn background_is_transparent() {
    let img = render_centered_rect(128, 128, [1.0, 1.0, 1.0, 1.0]);
    // 角落（远离矩形）应是透明背景
    let corner = img.get_pixel(0, 0);
    assert_eq!(corner.0, [0, 0, 0, 0], "背景角应透明，实际 {:?}", corner.0);
}

#[test]
fn rectangle_bounded_correctly() {
    // 128×128 画布，居中 100×100 矩形 → 边界在 [14, 114]（cx±50=64±50）
    // 画布外（如 x=10）应在矩形外，是透明背景
    let img = render_centered_rect(128, 128, [1.0, 0.0, 0.0, 1.0]);
    // 像素 (5, 64)：左边远端，应在矩形外
    let outside = img.get_pixel(5, 64);
    assert_eq!(outside.0, [0, 0, 0, 0], "矩形外像素应透明");
}
