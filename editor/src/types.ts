// 与 Rust 后端 lib.rs 的结构对齐。

export interface BoneDraw {
  name: string;
  x1: number; y1: number; // 根部
  x2: number; y2: number; // 末端
}

export interface RegionDraw {
  name: string;
  vertices: [number, number][]; // 世界坐标，y 向上
  color: [number, number, number, number];
}

export interface Pose {
  bones: BoneDraw[];
  regions: RegionDraw[];
}

export interface SkeletonInfo {
  bone_names: string[];
  animation_names: string[];
  duration: number;
}
