<script setup lang="ts">
import { computed } from "vue";
import { Quaternion, Vector3 } from "three";
import type { LaidOutEdge } from "../../lib/layout_trunk";

const props = defineProps<{
  edge: LaidOutEdge;
  active: boolean;
  dimmed: boolean;
}>();

const transform = computed(() => {
  const a = new Vector3(props.edge.from_pos.x, props.edge.from_pos.y, props.edge.from_pos.z);
  const b = new Vector3(props.edge.to_pos.x, props.edge.to_pos.y, props.edge.to_pos.z);
  const dir = new Vector3().subVectors(b, a);
  const len = dir.length() || 0.001;
  const mid = new Vector3().addVectors(a, b).multiplyScalar(0.5);
  const quat = new Quaternion().setFromUnitVectors(
    new Vector3(0, 1, 0),
    dir.clone().normalize(),
  );
  return {
    position: [mid.x, mid.y, mid.z] as [number, number, number],
    quaternion: [quat.x, quat.y, quat.z, quat.w] as [number, number, number, number],
    length: len,
  };
});

const along_x = computed(() => {
  const a = props.edge.from_pos;
  const b = props.edge.to_pos;
  const dx = Math.abs(b.x - a.x);
  const dy = Math.abs(b.y - a.y);
  const dz = Math.abs(b.z - a.z);
  return dx > dy + dz;
});

const color = computed(() => {
  if (props.active) return "#75eec5";
  if (along_x.value || props.edge.relation === "next") return "#5fd4b0";
  if (props.edge.relation === "contrasts") return "#ff1b1b";
  if (props.edge.relation === "derives") return "#8ed9bc";
  return "#6a5f5c";
});

const radius = computed(() => {
  if (props.active) return 0.04;
  if (along_x.value) return 0.05;
  return 0.022;
});
</script>

<template>
  <TresMesh
    :position="transform.position"
    :quaternion="transform.quaternion"
  >
    <TresCylinderGeometry :args="[radius, radius, transform.length, 8]" />
    <TresMeshStandardMaterial
      :color="color"
      :transparent="true"
      :opacity="dimmed ? 0.16 : active ? 0.95 : 0.5"
      :roughness="0.6"
      :metalness="0.05"
    />
  </TresMesh>
</template>
