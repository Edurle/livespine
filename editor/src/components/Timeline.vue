<script setup lang="ts">
const props = defineProps<{
  duration: number;
  time: number;
  playing: boolean;
  animations: string[];
  currentAnim: string | null;
}>();

const emit = defineEmits<{
  (e: "toggle-play"): void;
  (e: "seek", t: number): void;
  (e: "select-anim", name: string): void;
}>();

function onSeek(e: Event) {
  const v = parseFloat((e.target as HTMLInputElement).value);
  emit("seek", v);
}
</script>

<template>
  <div class="timeline">
    <div class="controls">
      <button @click="emit('toggle-play')">
        {{ props.playing ? "⏸ 暂停" : "▶ 播放" }}
      </button>
      <select
        v-if="props.animations.length"
        :value="props.currentAnim ?? ''"
        @change="emit('select-anim', ($event.target as HTMLSelectElement).value)"
      >
        <option value="">（无动画 / setup）</option>
        <option v-for="a in props.animations" :key="a" :value="a">{{ a }}</option>
      </select>
      <span class="time">
        {{ props.time.toFixed(2)}}s / {{ props.duration.toFixed(2) }}s
      </span>
    </div>
    <input
      type="range"
      :min="0"
      :max="props.duration || 1"
      :step="0.01"
      :value="props.time"
      @input="onSeek"
      class="scrubber"
    />
  </div>
</template>

<style scoped>
.timeline {
  padding: 8px 12px;
  background: #181825;
  border-top: 1px solid #45475a;
}
.controls {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 6px;
}
button {
  background: #cba6f7;
  color: #1e1e2e;
  border: none;
  padding: 6px 14px;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 600;
}
button:hover { background: #b4befe; }
select {
  background: #313244;
  color: #cdd6f4;
  border: 1px solid #45475a;
  padding: 4px 8px;
  border-radius: 4px;
}
.time {
  color: #a6adc8;
  font-family: monospace;
  font-size: 13px;
}
.scrubber {
  width: 100%;
}
</style>
