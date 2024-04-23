
#Please move to vite-plugin-svgs-icons
[Please move to vite-plugin-svgs-icons](https://github.com/335296558/vite-plugin-svgs-icons)

# vite-plugin-vue-svg-icons
[简体中文文档](README.md)
v3.0.0 English

[Nuxt3 nuxt-svg-icon](https://github.com/335296558/nuxt-svg-icon)

#### introduce
    An SVG icon plug-in without each SVG a HTTP request component can change the color, size only supports single color


<img src="../demo/src/assets/demo.gif" width="400px"></img> 
#### install
    yarn add vite-plugin-vue-svg-icons -D

    npm i vite-plugin-vue-svg-icons -D

    pnpm add vite-plugin-vue-svg-icons -D

#### vite.config.js Directions for use
```js
    import vitePluginVueSvgIcons from 'vite-plugin-vue-svg-icons'
    export default defineConfig({
        plugins: [
            vitePluginVueSvgIcons(Object)
        ],
    })
```
#### vitePluginVueSvgIcons Methods the reference，Object， If you don't need to SVG icon is not recommended in the directory

| param name | type | description | default |
| -------- | -------- | -------- | -------- |
|dir|String|SVG icon directory|path.resolve(__dirname, 'src', 'assets', 'svg')|
|moduleId|String|define import name|svg-icon|
```js
    // Directory, pay attention to
    // Multicolor SVG please put multicolor this directory, multicolor can't change color
    // If polychromatic SVG is not on the multicolor this directory, will be modified to monochrome
    // No multicolor directory you from already built
    |assets
        |svg
            |multicolor
                xxx.svg
                xxx.svg
                xxx.svg

            xxx.svg
            xxx.svg
            xxx.svg
```
#### The instructions on the SVG components
```js
    // app.vue Local registration using
    <script setup>
        import svgIcon from 'svg-icon'
    </script>
    // name Param is the SVG file name，such as：svg/logo.svg
    // So what do you refer to the SVG only need the name logo
    <template>
        <svgIcon name="logo" color="#f00" size="80" />
    </template>
```

```js
    // main.js All registered
    import svgIcon from 'svg-icon'
    VueApp.component('svg-icon', svgIcon);
```

#### Component param that
| param | type | default |
| -------- | -------- | -------- |
|name|String|Necessary to set the name, the same as the file name, or does not display well.The name parameter is a SVG file name, such as: name="logo"|
|color|String| inherit，Please set your ideal color, Support only monochrome SVG|
|size|Number|default 20, The default value is set to false, SVG size will not be set|
|class|String| - |

<!-- [示列图像]() -->


#### 版本描述：
    v3.1.2 > Optimized build configuration, optimized import method!
    v3.1.0 > Upgrade the new version, replace the vite build tool
    v3.0.21 < Abandoned and deleted versions smaller than this version! upgrade new version
    v3.0.21 normal version
    v3.0.22 optimize internally defined id: add prefix to id