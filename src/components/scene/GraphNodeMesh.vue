<script setup lang="ts">
import { computed } from "vue";
import type { LaidOutNode } from "../../lib/layout_trunk";

const props = defineProps<{
  node: LaidOutNode;
  selected: boolean;
  hovered: boolean;
  dimmed: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  hover: [id: string | null];
}>();

const is_trunk = computed(() => props.node.kind === "trunk");
const radius = computed(() => (is_trunk.value ? 0.42 : 0.28));
const trunk_len = computed(() => Math.max(1.1, props.node.trunk_half_len * 2));

const emissive = computed(() => {
  if (props.selected) return props.node.color;
  if (props.hovered) return "#ffffff";
  return "#000000";
});
const emissive_intensity = computed(() => {
  if (props.selected) return 0.45;
  if (props.hovered) return 0.18;
  return 0;
});
const opacity = computed(() => (props.dimmed ? 0.35 : 1));
</script>

<template>
  <TresGroup
    :position="[node.position.x, node.position.y, node.position.z]"
    @click.stop="emit('select', node.id)"
    @pointerenter="emit('hover', node.id)"
    @pointerleave="emit('hover', null)"
  >
    <!-- Trunk: cylinder along X (default cylinder is Y-up → rotate Z 90°) -->
    <TresMesh
      v-if="is_trunk"
      :rotation="[0, 0, Math.PI / 2]"
      :cast-shadow="false"
      :receive-shadow="false"
    >
      <TresCylinderGeometry :args="[radius, radius, trunk_len, 28]" />
      <TresMeshStandardMaterial
        :color="node.color"
        :emissive="emissive"
        :emissive-intensity="emissive_intensity"
        :transparent="dimmed"
        :opacity="opacity"
        :roughness="0.4"
        :metalness="0.16"
      />
    </TresMesh>
    <TresMesh
      v-if="is_trunk && (selected || hovered)"
      :rotation="[0, 0, Math.PI / 2]"
      :scale="1.12"
    >
      <TresCylinderGeometry :args="[radius, radius, trunk_len, 16]" />
      <TresMeshBasicMaterial
        color="#75eec5"
        :wireframe="true"
        :transparent="true"
        :opacity="selected ? 0.9 : 0.4"
      />
    </TresMesh>

    <!-- Branch: sphere in YZ -->
    <TresMesh v-if="!is_trunk" :cast-shadow="false" :receive-shadow="false">
      <TresSphereGeometry :args="[radius, 24, 24]" />
      <TresMeshStandardMaterial
        :color="node.color"
        :emissive="emissive"
        :emissive-intensity="emissive_intensity"
        :transparent="dimmed"
        :opacity="opacity"
        :roughness="0.45"
        :metalness="0.12"
      />
    </TresMesh>
    <TresMesh v-if="!is_trunk && selected" :scale="1.18">
      <TresSphereGeometry :args="[radius, 16, 16]" />
      <TresMeshBasicMaterial
        color="#ff1b1b"
        :wireframe="true"
        :transparent="true"
        :opacity="0.85"
      />
    </TresMesh>
  </TresGroup>
</template>
