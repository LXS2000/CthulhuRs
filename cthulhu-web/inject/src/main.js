import {createApp} from 'vue'
import App from './App.vue'


window.onload = () => {
    let cthulhu = document.createElement("div");
    cthulhu.id = "cthulhuInject"
    cthulhu.style.position = 'absolute';
    cthulhu.style.zIndex = '9999999'
    document.body.prepend(cthulhu)
    createApp(App).mount(cthulhu);
}

