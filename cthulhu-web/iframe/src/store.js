// index.ts
import {createStore} from 'vuex'
import {nextTick} from "vue";


const store = createStore({
    state: {
        curPlugin: {},
        windowHeight: window.innerHeight,
        windowWidth: window.innerWidth,
    },
    mutations: {
        resize(state) {
            state.windowHeight = window.innerHeight;
            state.windowWidth = window.innerWidth;
        },
        curPlugin(state, curPlugin) {
            state.curPlugin = curPlugin
        },
    },
    // strict:true,//开启严格模式，只有在mutation中才可以更改state
})
window.addEventListener("resize", () => {
    nextTick().then(()=> store.commit("resize"))

})
export default store
