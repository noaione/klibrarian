import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import Components from "unplugin-vue-components/vite";
import VueRouter from "unplugin-vue-router/vite";
import { VueUseComponentsResolver, VueUseDirectiveResolver } from "unplugin-vue-components/resolvers";
import Icons from "unplugin-icons/vite";
import IconsResolver from "unplugin-icons/resolver";
import { VueRouterAutoImports } from "unplugin-vue-router";
import { unheadVueComposablesImports } from "@unhead/vue";
import AutoImport from "unplugin-auto-import/vite";
import type { ManualChunkMeta } from "rollup";
import browserslist from "browserslist";
import { browserslistToTargets } from "lightningcss";

// https://vitejs.dev/config/
export default defineConfig({
  appType: "spa",
  plugins: [
    VueRouter({
      routesFolder: "src/pages",
      extensions: [".vue"],
      dts: "./src/types/router.d.ts",
    }),
    vue({
      script: {
        defineModel: true,
        propsDestructure: true,
      },
    }),
    Icons({
      compiler: "vue3",
      defaultClass: "v-icon",
    }),
    Components({
      dts: "./src/types/components.d.ts",
      resolvers: [IconsResolver(), VueUseComponentsResolver(), VueUseDirectiveResolver()],
    }),
    AutoImport({
      imports: ["vue", VueRouterAutoImports, unheadVueComposablesImports],
      dts: "./src/types/imports.d.ts",
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("src", import.meta.url)),
    },
  },
  build: {
    cssMinify: "lightningcss",
    cssCodeSplit: false,
    target: "es2022",
    modulePreload: false,
    sourcemap: true,
  },
  css: {
    lightningcss: {
      nonStandard: {
        deepSelectorCombinator: true,
      },
      targets: browserslistToTargets(browserslist()),
    },
  },
});
