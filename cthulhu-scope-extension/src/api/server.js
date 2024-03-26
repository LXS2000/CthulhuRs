import urlParser from "parse-url";
import {getHostList, getOption,} from "@/api/exten";
import {guid, hashcode, randomInt, randomItem} from "@/utils/util";
import {minimatch} from "minimatch";

import userAgents from "@/jsons/userAgents.json";
import {UAParser} from "ua-parser-js";
import {gpus, languages, regions} from "@/jsons";
import * as common from "@/api/common";


const tabIdMapBrowser = new Map()
const tabIdMapRequestHeaders = new Map()
const tabIdMapHostMapCookie = new Map()
let lastRandomBrowser = null

const mod = location.host.startsWith("localhost") ? 'dev' : ''
const extension = self.chrome || self.browser
let info = (() => {
    let os, device;
    let ua = navigator.userAgent
    let parser = new UAParser(ua)
    os = parser.getOS().name
    if (/ios|andriod/i.test(os)) device = 'mobile'
    else if (/win|window|mac|linux|unix/i.test(os)) device = 'desktop'
    device = device || parser.getDevice().type
    if (mod !== 'dev') {
        extension.system.display.getInfo((arr) => {
            let display = (arr || [])[0]
            info = {...info, height: display.height, width: display.width}
        })
    }
    return {os, device,}
})()

export async function getDevice(tabId) {
    return await common.getLocal('device') || info.device
}

export function getTab(tabId) {
    return new Promise((rs, rj) => {
        extension.tabs.get(tabId, rs)
    })
}

export function getTabRequestHeaders(tabId) {
    return tabIdMapRequestHeaders.get(tabId)
}

export function getTabId(tabId) {
    return tabId
}

export function setTabBrowser(tabId, browser) {
    tabIdMapBrowser.set(tabId, browser)
}

export function getTabBrowser(tabId) {
    return tabIdMapBrowser.get(tabId)
}

export function delTabBrowser(tabId) {
    return tabIdMapBrowser.delete(tabId)
}

export function getCookie(tabId, host) {
    getSuitBrowser(tabId, 'https://' + host).then()
    return ((tabIdMapHostMapCookie.get(tabId) || {})[host]) || ''
}

export async function setCookie(tabId, host, cookie) {
    let hosts = tabIdMapHostMapCookie.get(tabId)
    if (!hosts) {
        hosts = {}
        tabIdMapHostMapCookie.set(tabId, hosts)
    }
    hosts[host] = cookie
    return await getHeaderRules(tabId).then(rules => {
        for (const rule of rules) {
            if (rule.header === 'Cookie') {
                rule.value = cookie
                setHeaderRules(tabId, rules)
                return
            }
        }
    })
}

export async function deleteBrowser(tabId, browserId) {
    if (!browserId) return
    await common.getLocal("browserIds").then(async browserIds => {
        if (!Array.isArray(browserIds)) return
        browserIds = new Set(browserIds)
        browserIds.delete(browserId)
        await common.removeLocal("Browser-" + browserId)
        await common.setLocal('browserIds', [...browserIds])
    })
}

export async function getBrowser(tabId, browserId) {
    if (!browserId) return browserId
    return await common.getLocal("Browser-" + browserId)
}

export async function setBrowser(tabId, browser) {
    if (!browser) return null
    let browserId = browser.browserId
    if (!browserId) {
        //新增
        browser.browserId = guid()
        browserId = browser.browserId
    } else {
        //修改
        if (tabId !== 0 && mod !== 'dev') {
            let browserInTab = (await getTabBrowser(tabId))
            if (browserInTab && browserId === browserInTab.browserId) {
                await setTabBrowser(tabId, browser)
                await modifyHeaderByTabId(tabId)
            }
        }
    }
    let temp = await common.getLocal("browserIds")
    if (!Array.isArray(temp)) temp = []
    let browserIds = new Set(temp)
    browserIds.add(browserId);
    hashBrowser(browser)
    await common.setLocal("browserIds", [...browserIds])
    await common.setLocal("Browser-" + browserId, browser)
    return browserId
}

function hashBrowser(browser) {
    let browserTemp = {...browser}
    delete browserTemp.isTemp
    delete browserTemp.windowURL
    delete browserTemp.workerURL
    delete browserTemp.webrtc
    delete browserTemp.hashcode
    let str = JSON.stringify(browserTemp)
    browser.hashcode = hashcode(str)
}

export async function setHeaderRules(tabId, headerRules = []) {
    if (!headerRules || headerRules.length === 0) {
        return await extension.declarativeNetRequest.updateSessionRules({
            removeRuleIds: [tabId],
            addRules: [],
        }, () => {
            console.log(tabId + ": 请求头动态修改规则 清空成功", headerRules)
        })
    }
    return await extension.declarativeNetRequest.updateSessionRules({
        removeRuleIds: [tabId],
        addRules: [{
            id: tabId,
            priority: 1,
            action: {
                type: "modifyHeaders",
                requestHeaders: headerRules,
                // responseHeaders: [{
                //     header: "Content-Security-Policy",
                //     operation: "append",
                //     value: "; script-src 'self'; object-src 'self'; frame-src 'self'; child-src 'self'"
                // }]
            },
            condition: {
                urlFilter: "|http",
                tabIds: [tabId],
                resourceTypes: ['main_frame',
                    'sub_frame',
                    'stylesheet',
                    'script',
                    'image',
                    'font',
                    'object',
                    'xmlhttprequest',
                    'ping',
                    'csp_report',
                    'media',
                    'websocket',
                    'other',]
            }
        },],
    }, () => {
        console.log(tabId + ": 请求头动态修改规则 添加成功", headerRules)
    })
}

export async function getHeaderRules(tabId) {
    return new Promise(async (rs) => {
        let list = (await extension.declarativeNetRequest.getSessionRules())
        list.forEach(item => {
            if (item === tabId) return rs(item.action.requestHeaders)
        })
        return rs([])
    })
}

export async function modifyHeaderByTabId(tabId) {
    if (mod === 'dev') return
    let url = (await getTab(tabId)).url
    if (!url.toString().startsWith('http')) return

    let browser = tabIdMapBrowser.get(tabId)

    if (!browser) {
        if (process.env.VUE_APP_MANIFEST_VERSION !== 'v2') {
            await setHeaderRules(tabId, [])
        }
        tabIdMapRequestHeaders.set(tabId, [])
        return
    }
    let parser = new UAParser(browser.userAgent);

    let userAgent = '', language = '', platform = '', isMobile = false, uaData = ''
    if (browser.userAgent) userAgent = browser.userAgent;
    if (browser.language >= 0) {
        let code = languages[browser.language].code
        let [p, s] = code.split('-')
        language = code + "," + p + ';q=0.9';
    }
    if (parser) {
        platform = parser.getOS().name || ''
        isMobile = parser.getDevice() === 'mobile'
        if (navigator && navigator.userAgentData && navigator.userAgentData.brands) {
            let arr = []
            let brands = navigator.userAgentData.brands
            for (let i = 0; i < brands.length - 1; i++) {
                let brand = brands[i]
                arr.push(`"${brand.brand}";v="${brand.version}"`)
            }
            let b = parser.getBrowser();
            let browserName = b.name || ''
            let brandMapCompany = {
                Chrome: 'Google',
                Edge: 'Microsoft',
                IE: 'Microsoft',
                Firefox: 'Mozilla',
                Safari: 'Apple',
                QQ: 'Tencent',
            }

            for (let k of Object.keys(brandMapCompany)) {
                if (browserName.toUpperCase().includes(k.toUpperCase())) {
                    arr.push(`"${brandMapCompany[k] + " " + browserName}";v="${b.version}"`)
                    break
                }
            }
            uaData = arr.join(', ')
        }
    }

    let requestHeaders = []

    if (userAgent) requestHeaders.push({
        header: "User-Agent",
        operation: "set",
        value: userAgent
    })
    if (language) requestHeaders.push({
        header: "Accept-Language",
        operation: "set",
        value: language
    })
    if (platform) requestHeaders.push({
        header: "Sec-Ch-Ua-Platform",
        operation: "set",
        value: platform
    })
    if (isMobile) requestHeaders.push({
        header: "Sec-Ch-Ua-Mobile",
        operation: "set",
        value: info.device === 'mobile' ? '?0' : '?1'
    })
    if (uaData) requestHeaders.push({
        header: "Sec-Ch-Ua",
        operation: "set",
        value: uaData
    })
    // requestHeaders.push({
    //     header: "Cookie",
    //     operation: "set",
    //     value: cookie
    // })
    tabIdMapRequestHeaders.set(tabId, requestHeaders)

    if (process.env.VUE_APP_MANIFEST_VERSION !== 'v2') {
        if (!requestHeaders.length) {
            await setHeaderRules(tabId, [])
            tabIdMapRequestHeaders.set(tabId, [])
            return
        }
        await setHeaderRules(tabId, requestHeaders)
    }

}

export async function setEnable(tabId, browserId) {
    if (!browserId || !tabId) return
    let browser = (await getTabBrowser(tabId)) || {}
    if (browser.browserId === browserId) return
    browser = await getBrowser(tabId, browserId)
    await setTabBrowser(tabId, browser)
    if (tabId !== 0 && mod !== 'dev') {
        await modifyHeaderByTabId(tabId)
    }
}

export async function getEnable(tabId) {
    let browser = (await getTabBrowser(tabId)) || {}
    return browser.browserId
}

export async function delEnable(tabId) {
    if (!tabId) return
    delTabBrowser(tabId)
    if (tabId !== 0 && mod !== 'dev') {
        await setHeaderRules(tabId, [])
    }
}


const getCurrentBrowserId = async (tabId) => {
    let url = (await getTab(tabId)).url
    let localhost = urlParser(url).resource

    let enableList = await common.getLocal("enableList") || ''
    if (enableList === 'white' || enableList === 'black') {
        let hostList = await getHostList()
        let hosts = {}
        hostList.black.forEach(item => hosts[item.host] = false)
        hostList.white.forEach(item => hosts[item.host] = true)
        if (enableList === 'white') if (!hosts[localhost]) return console.log(localhost + ' 不在白名单', tabId), undefined
        if (enableList === 'black') if (hosts[localhost] === false) return console.log(localhost + ' 在黑名单', tabId), undefined
    }
    let browserId = await getEnable(tabId)
    if (!!browserId) return browserId;
    let option = await getOption(tabId)
    if (option === '无') return undefined
    let browserIds = await common.getLocal('browserIds') || []
    let randomBrowserId = randomItem(Math.random(), browserIds)
    if (option === '随机选取') return randomBrowserId
    if (option === '随机生成') return 'randomGenerate'
    if (option === '随机一次') return 'randomGenerateOnce'
    if (option === '静态') return await common.getLocal("fixedBrowserId")
    if (option === '匹配') {
        let matches = await common.getLocal('matches') || {}
        for (let regex of Object.keys(matches)) {
            try {
                let b = minimatch(url, regex)
                if (b) {
                    return matches[regex]
                }
            } catch (e) {

            }
        }
    }
    console.log(localhost + 'not match', tabId)
    return undefined
}
let id = 0;
const generateBrowser = (device, cur_os) => {
    let infos = []

    for (let ua of userAgents) {
        let {device: {type}, os: {name: osName}, engine: {name: engineName, version: engineVersion}} = ua
        if (device && device !== type) continue
        if (cur_os && cur_os !== osName) continue
        let version = Number(/\d+/g.exec(engineVersion)[0])
        if (/blink.*/.test(engineName) && version < 76) continue
        if (/gecko.*/.test(engineName) && version < 71) continue
        infos.push(ua)
    }
    let userAgent = randomItem(Math.random(), infos)

    let baseWidth, baseHeight, maxTouchPoints = 0;

    if (device === 'desktop') {
        baseWidth = info.width || 1536
        baseHeight = info.height || 864
    } else if (device === 'mobile') {
        baseWidth = info.width || 540
        baseHeight = info.height || 960
        maxTouchPoints = randomInt(Math.random(), 1, 10)
    } else if (device === 'tablet') {
        baseWidth = info.width || 1024
        baseHeight = info.height || 768
        maxTouchPoints = randomInt(Math.random(), 1, 10)
    }
    let depth = randomItem(Math.random(), [4, 6, 8, 10, 16, 24])
    let vendor = randomItem(Math.random(), Object.keys(gpus))
    let renderer = randomItem(Math.random(), gpus[vendor])
    let noise = randomInt(Math.random(), -15, 15)
    //HTMLElement.prototype.ontouchstart
    let browser = {
        browserId: 'randomGenerate',
        name: '随机生成配置:' + (id++),
        isTemp: true,
        userAgent: userAgent.ua,
        customProtos: [
            {
                name: 'Screen',
                properties: [
                    {key: 'colorDepth', value: depth, type: 'number'},
                    {key: 'pixelDepth', value: depth, type: 'number'},]
            },
            {
                name: 'Navigator',
                properties: [
                    {key: 'maxTouchPoints', value: maxTouchPoints, type: 'number'},]
            },],
        customVars: [],
        webrtc: 'dynamic',
        factors: {
            audio: Math.random() * 10,
            canvas: Math.random() * 10,
            fonts: Math.random() * 10,
            plugins: Math.random() * 10,
            webgl: Math.random() * 10,
        },
        gpu: {vendor, renderer},
        screen: {
            width: baseWidth + noise,
            height: baseHeight + noise,
        },
        timeZone: randomInt(Math.random(), 0, regions.length - 1),
        language: randomInt(Math.random(), 0, languages.length - 1),
        memoryCapacity: randomItem(Math.random(), [0.25, 0.5, 1, 2, 4, 8]),
        processors: randomItem(Math.random(), [1, 2, 4, 8, 16]),
    };
    if (device === 'mobile' || device === 'tablet') {
        let touchEvents = ['ontouchstart', 'ontouchend', 'ontouchmove', 'ontouchcancel']
        touchEvents.forEach(e => {
            browser.customVars.push({path: 'HTMLElement.prototype.' + e, value: null})
            browser.customVars.push({path: 'window.' + e, value: null})
        })
    }
    return browser
}
export const obtainBrowser = async (id) => {
    if (!id) return null
    if (id === 'randomGenerate') {
        let device = await getDevice()
        let os = info.os
        if (device !== info.device) os = ''
        return generateBrowser(device, os)
    }
    if (id === 'randomGenerateOnce') {
        if (lastRandomBrowser) return lastRandomBrowser
        let device = await getDevice()
        let os = info.os
        if (device !== info.device) os = ''

        lastRandomBrowser = generateBrowser(device, os)
        lastRandomBrowser.browserId = 'randomGenerateOnce'
        lastRandomBrowser.name = '随机生成固定配置'
        return lastRandomBrowser
    }
    return common.getLocal("Browser-" + id)
}

export async function getSuitBrowser(tabId, url) {
    let browser = await getTabBrowser(tabId)
    const handleBrowser = async (browserTemp) => {
        let browser = {...browserTemp}
        if (browser.webrtc === 'dynamic') browser.webrtc = await getIP()
        await setEnable(tabId, browserId)
        try {
            if (!process.env.VUE_APP_FIXED_LANGUAGE) {
                if (languages[browser.language]) browser.language = languages[browser.language].code
                else browser.language = ''
            } else browser.language = process.env.VUE_APP_FIXED_LANGUAGE
            if (!process.env.VUE_APP_FIXED_TIMEZONE) {
                if (regions[browser.timeZone]) browser.timeZone = regions[browser.timeZone].timezone
                else browser.timeZone = ''
            } else browser.timeZone = process.env.VUE_APP_FIXED_TIMEZONE
            return browser
        } catch (e) {
            console.log(browser, e)
        }
    }
    let browserId = await getCurrentBrowserId(tabId, url)
    if (!browserId) {
        delTabBrowser(tabId)
        return {tabId}
    }
    if (!browser || browserId !== browser.browserId) {
        browser = await obtainBrowser(browserId)
        if (!browser) return {tabId}
        await setTabBrowser(tabId, browser)
    }
    hashBrowser(browser)
    browser = await handleBrowser(browser)
    return {browser, tabId}
}

export async function randomBrowser(tabId) {
    const getRandomBrowserId = async () => {
        let option = await getOption(tabId)
        let browserIds = await common.getLocal('browserIds') || []
        let randomBrowserId = randomItem(Math.random(), browserIds)
        if (option === '随机选取') return randomBrowserId
        if (option === '随机生成') return 'randomGenerate'
        if (option === '随机一次') return 'randomGenerateOnce'
        return undefined
    }
    let browserId = await getRandomBrowserId()
    if (!browserId) return false
    let browser = await obtainBrowser(browserId)
    if (!browser) return false
    await setTabBrowser(tabId, browser)
    return true
}

export async function selectBrowsers(tabId, keyword, page = 1, size = -1) {
    page -= 1
    console.log('selectBrowsers')
    let temp = await common.getLocal("browserIds")
    if (!Array.isArray(temp)) temp = []
    let browserIds = new Set(temp)
    let list = []
    let suitBrowser = await getTabBrowser(tabId)
    for (const browserId of browserIds) {
        let browser = await common.getLocal("Browser-" + browserId)
        if (!browser) {
            browserIds.delete(browserId)
            continue;
        }
        if (suitBrowser && suitBrowser.browserId === browserId) continue;
        if (keyword) {
            if (((browser.name || '') + "").includes(keyword)) {
                list.push(browser)
            }
            continue;
        }
        list.push(browser)
    }

    if (browserIds.length) common.setLocal("browserIds", [...browserIds]).then()
    let browser = (await getTabBrowser(tabId))
    if (browser) list.unshift(browser)
    if (size < 0) return {data: list, total: list.length}
    let sub = list.slice(page * size, page * size + size)
    return {data: sub, total: list.length}
}

let dynamicIP = undefined

export async function getIP() {
    if (dynamicIP) return dynamicIP
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort(), 5000);
    let ip = await fetch("https://api.ipify.org/?format=text", {
        method: 'get',
        headers: {
            'Host': 'api.ipify.org',
            'Cache-Control': 'max-age=0',
            'Connection': 'keep-alive',
            'Referer': 'https://api.ipify.org',
        },
        signal: controller.signal
    }).then(async res => {
        let ip = await res.text();
        console.log('获取ip地址成功:' + ip)
        return ip
    }).catch(e => new Error("获取IP地址超时,请检查网络设置"))
    clearTimeout(id);

    dynamicIP = ip + ""
    return ip
}

async function waitAndRepeat(func, delay) {
    while (true) {
        await func();
        await new Promise(resolve => setTimeout(resolve, delay));
    }
}

waitAndRepeat(async () => {
    return await getIP()
}, 10000)

