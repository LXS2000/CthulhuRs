import {isTrial} from "@/utils/util";

import {UAParser} from "ua-parser-js";


const mod = location.host.startsWith("localhost") ? 'dev' : ''
const extension = self.chrome || self.browser

let info = (() => {
    let os, device;
    let ua = navigator.userAgent
    let parser = new UAParser(ua)
    os = parser.getOS().name
    if (/ios|andriod/i.test(os)) device = 'mobile'
    else if (/win|window|mac|linux|unix/i.test(os)) device = 'desktop'
    device = parser.getDevice().type || device
    return {os, device}
})()
const storage = mod === 'dev' ? {} : (info.os === 'android' ? extension.storage.local : extension.storage.sync)

export function curTabId() {
    return new Promise((rs, rj) => {
        return extension.tabs.query({}, tabs => {
            tabs = tabs.filter(tab => tab.selected && tab.active)
            rs(((tabs[0] || {}).id) || 0)
        })
    })

}

export function getLocal(key) {
    return new Promise((rs, rj) => {
        if (!key) return rs()
        if (mod === 'dev') {
            let value = localStorage.getItem(key);
            if (value) value = JSON.parse(value)
            return rs(value)
        }
        isTrial()
        storage.get([key], result => {
            rs(result[key])
        })
    })
}

export  function setLocal(key, value) {
    return new Promise((rs, rj) => {
        if (!key || !value) return rs()
        if (mod === 'dev') {
            localStorage.setItem(key, JSON.stringify(value))
            return rs()
        }
        isTrial()
        let a = {}
        a[key] = value
        return storage.set(a, rs)
    })

}

export  function removeLocal(key) {
    return new Promise((rs, rj) => {
        if (!key) return rs()
        if (mod === 'dev') {
            localStorage.removeItem(key)
            return rs()
        }
        isTrial()
        return storage.remove(key, rs)
    })
}
