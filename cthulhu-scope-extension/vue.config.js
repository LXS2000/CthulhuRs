const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require("path");
const ExtensionReloader = require('webpack-extension-reloader');
const version = process.env.VUE_APP_MANIFEST_VERSION || 'v3'
const packageName = "dist/" + version // 定义项目打包后文件名
// 复制文件夹到指定目录
const copyFiles = [
    {
        from: path.resolve(version === "v2" ? "src/manifest-v2.json" : "src/manifest.json"),
        to: `${path.resolve(packageName)}/manifest.json`
    },
    {
        from: path.resolve("src/assets"),
        to: path.resolve(packageName + "/assets")
    },
    {
        from: path.resolve("src/_locales"),
        to: path.resolve(packageName + "/_locales")
    },
];
// 复制插件
const plugins = [
    new CopyWebpackPlugin({
        patterns: copyFiles
    }),
    new ExtensionReloader({
        reloadPage: false,
    }),
];

module.exports = {
    publicPath: './',
    lintOnSave: false,
    transpileDependencies: version === 'v2' ? ['*'] : undefined,
    pages: {
        index: {
            entry: version === 'v2' ? ["node_modules/@babel/polyfill/dist/polyfill.js", "src/popup/main.js"] : "src/popup/main.js",
            template: 'src/popup/index.html',
            filename: 'index.html'
        },
    },
    productionSourceMap: false,
    // 根目录  如果不写 默认是dist
    outputDir: __dirname + '/' + packageName,
    pluginOptions: {
        browserExtension: {
            componentOptions: {
                background: {
                    entry: "src/background/main.js",
                },
            }
        }
    },
    configureWebpack: {
        output: {
            globalObject: 'self',
        },
        watch: true, // 自动打包
        plugins,
        optimization: {
            splitChunks: false,
        },
        // 打包文件大小配置
        performance: {
            maxEntrypointSize: 10000000,
            maxAssetSize: 30000000
        }
    },
    css: {
        extract: {
            filename: "css/[name].css"
        }
    },
    chainWebpack: config => {
        if (process.argv.includes('development')) {
            config.optimization.minimize(false);
            config.optimization.splitChunks(false);
        }
        config.output.filename('js/[name].js').end()
        config.output.chunkFilename('js/[name].js').end()
    }
}
