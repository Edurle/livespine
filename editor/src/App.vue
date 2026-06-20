<script setup lang="ts">
import { ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Viewport from "./components/Viewport.vue";
import Timeline from "./components/Timeline.vue";
import type { Pose, SkeletonInfo } from "./types";

const pose = ref<Pose | null>(null);
const info = ref<SkeletonInfo | null>(null);
const time = ref(0);
const playing = ref(false);
const currentAnim = ref<string | null>(null);
const statusMsg = ref("未加载");

let rafId = 0;
let lastTs = 0;

// 默认加载 demo_char.lp（P4 简化：不做文件选择器，路径由后端 command 决定）
async function loadDefault() {
  try {
    const i = await invoke<SkeletonInfo>("load_default");
    info.value = i;
    currentAnim.value = i.animation_names[0] ?? null;
    time.value = 0;
    statusMsg.value = `已加载 demo_char：${i.bone_names.length} 骨，${i.animation_names.length} 动画`;
    await refreshPose();
  } catch (e) {
    statusMsg.value = `加载失败：${e}`;
  }
}

async function refreshPose() {
  if (!info.value) return;
  try {
    pose.value = await invoke<Pose>("sample_pose", {
      anim: currentAnim.value,
      time: time.value,
    });
  } catch (e) {
    statusMsg.value = `采样失败：${e}`;
  }
}

function togglePlay() {
  playing.value = !playing.value;
  if (playing.value) {
    lastTs = performance.now();
    loop();
  } else {
    cancelAnimationFrame(rafId);
  }
}

function loop() {
  rafId = requestAnimationFrame(async (ts) => {
    const dt = (ts - lastTs) / 1000;
    lastTs = ts;
    if (info.value && info.value.duration > 0) {
      time.value = (time.value + dt) % info.value.duration;
    }
    await refreshPose();
    if (playing.value) loop();
  });
}

function onSeek(t: number) {
  time.value = t;
  refreshPose();
}

function selectAnim(name: string) {
  currentAnim.value = name || null;
  time.value = 0;
  refreshPose();
}

onUnmounted(() => cancelAnimationFrame(rafId));
loadDefault();
</script>

<template>
  <div class="app">
    <div class="toolbar">
      <span class="status">{{ statusMsg }}</span>
    </div>
    <div class="main">
      <Viewport :pose="pose" />
    </div>
    <Timeline
      v-if="info"
      :duration="info.duration"
      :time="time"
      :playing="playing"
      :animations="info.animation_names"
      :current-anim="currentAnim"
      @toggle-play="togglePlay"
      @seek="onSeek"
      @select-anim="selectAnim"
    />
  </div>
</template>

<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
html, body, #app { height: 100%; }
body {
  font-family: system-ui, -apple-system, sans-serif;
  background: #11111b;
  color: #cdd6f4;
}
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: #181825;
  border-bottom: 1px solid #45475a;
}
.toolbar button {
  background: #cba6f7;
  color: #1e1e2e;
  border: none;
  padding: 6px 14px;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 600;
}
.toolbar button:hover { background: #b4befe; }
.status { color: #a6adc8; font-size: 13px; }
.main {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}
</style>
