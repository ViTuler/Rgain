<script setup lang="ts">
defineProps<{
  sidecar_status: string;
}>();
</script>

<template>
  <section class="scene">
    <div class="scene_grid" />
    <div class="scene_copy">
      <p class="brand_mark">Rgain</p>
      <h1>3D 逻辑舞台</h1>
      <p class="desc">
        Phase C 将在此绘制 Trunk Timeline。主干色
        <span class="mint">#75eec5</span>，强调色
        <span class="crimson">#ff1b1b</span>。
      </p>
      <p class="status">Sidecar：{{ sidecar_status }}</p>
    </div>
  </section>
</template>

<style scoped>
.scene {
  position: relative;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  background:
    radial-gradient(ellipse 70% 45% at 50% 0%, color-mix(in srgb, #75eec5 14%, transparent), transparent),
    var(--color_bg_deep);
}

.scene_grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(color-mix(in srgb, #75eec5 8%, transparent) 1px, transparent 1px),
    linear-gradient(90deg, color-mix(in srgb, #75eec5 8%, transparent) 1px, transparent 1px);
  background-size: 48px 48px;
  mask-image: radial-gradient(ellipse at center, #000 35%, transparent 80%);
  opacity: 0.55;
  pointer-events: none;
}

.scene_copy {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--space_lg);
  text-align: center;
}

.brand_mark {
  margin: 0;
  font-size: 0.75rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--color_accent_mint);
}

h1 {
  margin: 0.4rem 0 0.75rem;
  font-size: 1.75rem;
  font-weight: 650;
}

.desc {
  max-width: 28rem;
  margin: 0;
  color: var(--color_text_muted);
}

.mint {
  color: var(--color_accent_mint);
}

.crimson {
  color: var(--color_accent_crimson);
}

.status {
  margin-top: var(--space_md);
  padding: 0.35rem 0.75rem;
  border-radius: 999px;
  border: 1px solid var(--color_border);
  background: color-mix(in srgb, #161c19 80%, transparent);
  font-size: 0.8rem;
  color: var(--color_text_on_deep);
}
</style>
