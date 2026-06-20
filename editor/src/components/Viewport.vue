<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import type { Pose } from "../types";

const props = defineProps<{ pose: Pose | null }>();

const canvas = ref<HTMLCanvasElement | null>(null);

function draw() {
  const cv = canvas.value;
  const pose = props.pose;
  if (!cv || !pose) return;
  const ctx = cv.getContext("2d");
  if (!ctx) return;

  const w = cv.width;
  const h = cv.height;
  ctx.clearRect(0, 0, w, h);

  // 背景
  ctx.fillStyle = "#1e1e2e";
  ctx.fillRect(0, 0, w, h);

  // 坐标变换：内核 y 向上、原点左下；Canvas y 向下、原点左上。
  // 简单适配：平移原点到画布中心，y 翻转。不缩放（P4 固定）。
  ctx.save();
  ctx.translate(w / 2, h / 2 + 100); // 略下偏，让角色居中可见
  ctx.scale(1, -1); // y 翻转

  // 先画 region 多边形（纯色填充）—— 骨骼画在其上，避免被不透明矩形遮挡
  for (const r of pose.regions) {
    if (r.vertices.length < 3) continue;
    ctx.fillStyle = `rgba(${Math.round(r.color[0] * 255)},${Math.round(r.color[1] * 255)},${Math.round(r.color[2] * 255)},${r.color[3]})`;
    ctx.beginPath();
    ctx.moveTo(r.vertices[0][0], r.vertices[0][1]);
    for (let i = 1; i < r.vertices.length; i++) {
      ctx.lineTo(r.vertices[i][0], r.vertices[i][1]);
    }
    ctx.closePath();
    ctx.fill();
  }

  // 后画骨骼（线段 + 关节圆点）—— 画在最上层
  ctx.strokeStyle = "#89b4fa";
  ctx.lineWidth = 4;
  ctx.fillStyle = "#f9e2af";
  for (const b of pose.bones) {
    ctx.beginPath();
    ctx.moveTo(b.x1, b.y1);
    ctx.lineTo(b.x2, b.y2);
    ctx.stroke();
    // 关节圆点（根）
    ctx.beginPath();
    ctx.arc(b.x1, b.y1, 5, 0, Math.PI * 2);
    ctx.fill();
  }

  ctx.restore();
}

onMounted(() => {
  const cv = canvas.value;
  if (cv) {
    cv.width = 800;
    cv.height = 600;
  }
  draw();
});

watch(() => props.pose, draw, { deep: true });
</script>

<template>
  <canvas ref="canvas" class="viewport"></canvas>
</template>

<style scoped>
.viewport {
  background: #1e1e2e;
  border: 1px solid #45475a;
  display: block;
}
</style>
