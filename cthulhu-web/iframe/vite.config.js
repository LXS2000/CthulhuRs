import {defineConfig} from 'vite'
import vue from '@vitejs/plugin-vue'

import alias from "@rollup/plugin-alias";
import path from 'path'

import {createSvgIconsPlugin} from 'vite-plugin-svg-icons'

export default defineConfig({
    plugins: [alias(), vue(),
        createSvgIconsPlugin({
            // 指定需要缓存的图标文件夹
            iconDirs: [path.resolve(process.cwd(), 'src/assets/icons')],//可能会出现‘process is not defined’报错，但并不影响
            // 指定symbolId格式
            symbolId: 'icon-[name]'
        }),
    ],
    base: 'https://web.cthulhu.server/iframe/',
    envPrefix:"APP_",
    resolve: {
        alias: {
            "/@": path.resolve(__dirname, "./src"),
        },
    },

    build: {
        outDir: 'dist/',
        assetsDir: 'assets',
        manifest: false,
        rollupOptions: {
            output: {
                entryFileNames: `assets/[name].js`,
                chunkFileNames: `chunks/[name].js`,
                assetFileNames: `assets/[name].[ext]`,
            }
        }
    },


})
