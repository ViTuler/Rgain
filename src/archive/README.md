# Archived source (not compiled)

`scene3d/` holds the former Tres/Three trunk view. It is excluded from `tsconfig` / Vite builds while 3D deps are removed.

To restore:
1. Re-add `three`, `@tresjs/core`, `@tresjs/cientos`, `@types/three`
2. Move files back to `src/components/scene/`
3. Wire `LogicScene` into `LogicStage.vue` again
