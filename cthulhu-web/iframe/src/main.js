import {createApp} from 'vue'
import '/src/style.css'

import App from './App.vue'

import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import router from "./router.js";
import store from "./store.js";

import 'virtual:svg-icons-register'


const app = createApp(App)
app.use(ElementPlus)

// app.component("SvgIcon",SvgIcon)
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
    app.component(key, component)
}

function getUrlParams() {
    let params = {}
    if (location.search) {
        location.search.substring(1).split("&").forEach(kv => {
            let [k, v] = kv.split("=");
            if (!k) return;
            if (v) v = decodeURIComponent(v)
            params[k] = v
        })
    }
    return params;
}

self["CTHULHU_SCOPE_ID"] = getUrlParams()["scopeId"]


window.onload = () => {
    app.use(router)
    app.use(store)
    app.mount("#app")
}


