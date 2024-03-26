import * as common from "@/api/common";
import {Message} from "@element-plus/icons-vue";
import * as server from "@/api/server";

const extension = self.chrome || self.browser
const mod = location.host.startsWith("localhost") ? 'dev' : ''

async function send(message = {}) {
    return new Promise((rs, rj) => {
        if (mod === 'dev') {
            let action = message.action
            let handler = server[action]
            delete message['action']
            let args = Object.keys({...message}).map(k => message[k])
            args.unshift(0)
            let result = handler(...args)
            if (!result) return rs()
            if (result.then) result.then(data => {
                console.log('request=====' + action, message, 'data:', data)
            })
            else {
                console.log('request=====' + action, message, 'data:', result)
            }
            return rs(result)
        }
        extension.runtime.sendMessage({...message}, (re) => {
            if (re instanceof Error) {
                Message.error(re.message)
                rj(re)
            }
            return rs(re)
        })

    })

}

export  function getLocal(key) {
    return  common.getLocal(key)
}

export  function setLocal(key, value) {
    return  common.setLocal(key, value)
}

export  function removeLocal(key) {
    return  common.removeLocal(key)
}

export  function setTabBrowser(browser) {
    return  send({action: 'setTabBrowser', browser})
}

export  function getTabBrowser() {
    return  send({action: 'getTabBrowser'})
}

export  function delTabBrowser() {
    return  send({action: 'delTabBrowser'})
}

export  function getCookie(host) {
    return  send({action: 'getCookie', host})
}

export  function setCookie(host, cookie) {
    return  send({action: 'setCookie', host, cookie})
}

export  function getIP() {
    return  send({action: 'getIP'})
}


export async function getTab(tabId) {
    return await extension.tabs.get(tabId)
}

export async function getDevice() {
    return send({action: "getDevice"})
}

export function getTabId() {
    return send({action: "getTabId"})
}

export function setEnable(browserId) {
    return send({action: "setEnable", browserId})
}

export function getEnable() {
    return send({action: "getEnable"})
}

export function delEnable() {
    return send({action: "delEnable"})
}

export function deleteBrowser(browserId) {
    return send({action: "deleteBrowser", browserId})
}

export function getBrowser(browserId) {
    return send({action: "getBrowser", browserId})
}

export function setBrowser(browser) {
    return send({action: "setBrowser", browser})
}

export async function setMatch(regex, browserId) {
    let matches = await common.getLocal("matches").then(s => s) || {}
    if (matches[regex]) {
        throw new Error(regex + " 已定义")
    }
    matches[regex] = browserId
    return common.setLocal("matches", matches)
}

export async function delMatch(regex) {
    let matches = await common.getLocal("matches") || {}
    delete matches[regex]
    return common.setLocal("matches", matches)
}

export async function getMatches(page, size) {
    let matches = await common.getLocal("matches") || {}
    let matchesList = []
    let start = (page - 1) * size
    let keys = Object.keys(matches)
    for (let i = start; i < keys.length; i++) {
        let key = keys[i]
        matchesList.push({regex: key, browserId: matches[key], index: i + 1})
        if (matchesList.length >= size) break
    }
    return {data: matchesList, total: keys.length}
}

export async function setOption(option) {
    await common.removeLocal("enables")
    return common.setLocal("option", option)
}

export async function getOption() {
    return await common.getLocal("option") || '随机生成'
}

export async function setHost(host, isWhite) {
    if (isWhite === undefined || isWhite === null) isWhite = true
    let hostList = await common.getLocal('hostList') || {}
    if (hostList[host]) {
        throw new Error(`域名 ${host} 在${hostList[host] === true ? '白' : '黑'}名单中已存在`)
    }
    hostList[host] = isWhite
    return common.setLocal('hostList', hostList)
}

export async function delHost(host) {
    let hostList = await common.getLocal('hostList') || {}
    delete hostList[host]
    return common.setLocal('hostList', hostList)
}

export async function getHostList() {
    let hostList = await common.getLocal('hostList') || {}
    let white = []
    let black = []
    for (let k in hostList) {
        if (hostList[k] === true) white.push({index: white.length + 1, host: k})
        else black.push({index: black.length + 1, host: k})
    }
    return {white, black}
}


export async function download(url, filename) {
    if (!(extension || {}).downloads) {
        const a = document.createElement('a')
        a.href = url
        a.download = filename // 下载后文件名
        a.style.display = 'none'
        document.body.appendChild(a)
        a.click() // 点击下载
        document.body.removeChild(a) // 下载完成移除元素
        return
    }
    return await extension.downloads.download({url, filename});
}

export function getSuitBrowser() {
    return send({action: 'getSuitBrowser'})
}

export function randomBrowser() {
    return send({action: 'randomBrowser'})
}

export function obtainBrowser(id) {
    return send({action: 'obtainBrowser', id})
}

export function selectBrowsers(keyword, page = 1, size = -1) {
    return send({action: 'selectBrowsers', keyword, page, size})
}
