import {createWebHistory, createRouter} from 'vue-router'
import {CONFIG} from "/@/config.js";


const history = createWebHistory("/iframe/")
const subPage =
    {
        path: 'sub',
        name: 'Sub',
        component: () => import('./subPages/SubPage.vue'),
        children: [
            {
                path: 'store',
                name: 'Store',
                meta: {
                    title: "数据存储"
                },
                component: () => import('./subPages/Store.vue')
            },
            {
                path: 'func',
                name: 'Func',
                meta: {
                    title: "功能主页"
                },
                component: () => import('./subPages/Func.vue')
            },
            {
                path: 'log',
                name: 'Log',
                meta: {
                    title: "运行日志"
                },
                component: () => import('./subPages/Log.vue')
            },
        ]
    };

const router = createRouter({
    history, // 路由模式

    routes: [
        {
            path: '/index',
            name: 'Index',
            component: () => import('./pages/Index.vue'),
            children: [
                {
                    path: 'list',
                    name: 'List',
                    component: () => import('./pages/home/List.vue')
                },
                {
                    path: 'home',
                    name: 'Home',
                    component: () => import('./pages/home/Home.vue'),
                },
                {
                    path: 'intro',
                    name: 'Intro',
                    component: () => import('./pages/home/Intro.vue')
                },
                {
                    path: 'setting',
                    name: 'Setting',
                    component: () => import('./pages/home/Setting.vue')
                },

            ]
        },
    ]
})
let isSp = 0

async function handle() {
    const ratio = window.innerWidth / window.innerHeight
    if ((ratio >= CONFIG.ratio && (isSp === 1 || isSp === 0))) {
        // console.log("横屏")
        if (router.hasRoute("Sub")) {
            router.removeRoute('Sub');
        }
        subPage.path = "sub"
        await router.addRoute('Index', subPage)
        handlePath(2)
    } else if ((ratio < CONFIG.ratio && (isSp === 2 || isSp === 0))) {
        // console.log("竖屏")
        if (router.hasRoute("Sub")) {
            router.removeRoute('Sub');
        }
        subPage.path = "/sub"
        await router.addRoute('', subPage)
        handlePath(1)
    }
    await router.isReady()
    isSp = ratio < CONFIG.ratio ? 1 : 2;
}

handle()
window.addEventListener("resize", handle)

function handlePath(sp) {
    //当 当前路由时特殊路由时， 如果屏幕比例发生变化，就跳转到对应的地址
    let path = router.currentRoute.value.fullPath;
    let routers = ["func", "log", "store", "edit"]
    for (let r of routers) {
        if (path.endsWith("/" + r)) {
            if (sp === 1) {
                router.push("/sub/" + r);
                return
            }
            if (sp === 2) {
                router.push("/index/sub/" + r);
                return
            }
            break
        }
    }
}

router.beforeEach((to, from, next) => {
    if (to.path === '/' || to.path === '/index/' || !router.hasRoute(to.name)) {
        next({
            path: '/index/home',
        })
        return
    }
    next() // 确保一定要调用 next()
})

export default router
