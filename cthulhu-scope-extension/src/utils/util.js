import UAParser from "ua-parser-js";

/**
 * 时间戳
 * @param {*} timestamp  时间戳
 */
const timestampToTime = (timestamp) => {
    let date = new Date(timestamp) //时间戳为10位需*1000，时间戳为13位的话不需乘1000
    let Y = date.getFullYear() + '-'
    let M =
        (date.getMonth() + 1 < 10 ?
            '0' + (date.getMonth() + 1) :
            date.getMonth() + 1) + '-'
    let D =
        (date.getDate() < 10 ? '0' + date.getDate() : date.getDate()) + ' '
    let h =
        (date.getHours() < 10 ? '0' + date.getHours() : date.getHours()) + ':'
    let m =
        (date.getMinutes() < 10 ? '0' + date.getMinutes() : date.getMinutes()) +
        ':'
    let s =
        date.getSeconds() < 10 ? '0' + date.getSeconds() : date.getSeconds()
    return Y + M + D + h + m + s
};

function deepError() {
    let e = new Error('')
    e.stack = ''
    throw e
}

/**
 * 存储localStorage
 */
const setStore = (name, content) => {
    if (!name) return;
    if (typeof content !== 'string') {
        content = JSON.stringify(content);
    }
    window.localStorage.setItem(name, content);
}

/**
 * 获取localStorage
 */
const getStore = name => {
    if (!name) return;
    return window.localStorage.getItem(name);
}

/**
 * 删除localStorage
 */
const removeStore = name => {
    if (!name) return;
    window.localStorage.removeItem(name);
}

/**
 * 设置cookie
 **/
function setCookie(name, value, day) {
    let date = new Date();
    date.setDate(date.getDate() + day);
    document.cookie = name + '=' + value + ';expires=' + date;
};

/**
 * 获取cookie
 **/
function getCookie(name) {
    let reg = RegExp(name + '=([^;]+)');
    let arr = document.cookie.match(reg);
    if (arr) {
        return arr[1];
    } else {
        return '';
    }
};

/**
 * 删除cookie
 **/
function delCookie(name) {
    setCookie(name, null, -1);
}

function getValue(obj, str) {
    let keys = str.split(/[.\[\]]/).filter(key => key !== '');
    let value = obj;
    for (let key of keys) {
        if (value[key] === undefined) {
            return undefined;
        }
        value = value[key];
    }
    return value;
}

function isValidPath(path) {
    const regex = /^[a-z]+(\[\d+\]|\.[a-z]+)*$/i;
    return regex.test(path);
}

function copy(obj) {
    return JSON.parse(JSON.stringify(obj))
}

function isPlugin() {
    let extension = self.chrome || self.browser
    return extension && extension.runtime
}

const randomInt = function (factor, min, max) {
    factor = (factor || 0) + 9
    let arr = [1, 0.6, 5, -65.4, 465, 10, -0.545, 5.61, 0.65, 34, 5]
    for (let i = 0; i < arr.length; i++) {
        factor *= arr[i]
    }
    factor |= 0
    return Math.abs(factor) % (max - min + 1) + min;
}

const randomItem = function (factor, items) {
    return items[randomInt(factor, 0, items.length - 1)]
}

const randomNum = function (factor, max) {
    return randomInt(factor, 0, max);
}

function getLogo(userAgent = '') {
    let parser = new UAParser(userAgent)
    let name = (parser.getBrowser().name);
    let logo = './assets/icon/'
    if (name) {
        name = name.toLowerCase()
        if (name.includes('edg') || name.includes('edge')) logo += 'edg.png'
        else if (name.includes('firefox')) logo += 'firefox.png'
        else if (name.includes('chrome')) logo += 'chrome.png'
        else if (name.includes('safari')) logo += 'safari.png'
        else if (name.includes('360')) logo += '360.png'
        else if (name.includes('baidu')) logo += 'baidu.png'
        else if (name.includes('qq')) logo += 'qq.png'
        else if (name.includes('uc')) logo += 'uc.png'
        else if (name.includes('opera')) logo += 'opera.jpg'
        else if (name.includes('brave')) logo += 'brave.jpg'
        else logo += 'default.png'
        return logo
    }
    return logo += 'default.png'
}

function splitExpression(expression) {
    const match = expression.match(/(.*)(\.[^.[]+|\[[^\]]+\])$/);
    if (match) {
        return [match[1], match[2]];
    } else if (/^([^.[]+|\[[^\]]+\])$/.test(expression)) {
        return ['window', expression];
    } else {
        throw new Error('Invalid expression:' + expression);
    }
}

function splitPath(path = "") {
    let pathNodes = [];
    let start = 0;
    for (let i = 0; i < path.length; i++) {
        let c = path[i]
        if (c === "." || c === '[') {
            let node = path.substring(start, i)
            pathNodes.push(node);
            start = i + 1;
            continue
        }
        if (c === "]") {
            let node = path.substring(start, i)
            pathNodes.push(Number(node));
            start = i + 1;
        }
    }
    if (start < path.length) {
        let node = path.substring(start)
        if (path[start - 1] === '[') pathNodes.push(Number(node));
        else pathNodes.push(node);
    }
    return pathNodes
}

function combinePath(nodes = []) {
    let path = "";
    for (let i = 0; i < nodes.length; i++) {
        let node = nodes[i];
        if (node === '' || node === undefined || node === null) continue

        if (Number.isInteger(node))  path += ("[" + node + "]")
        else  path += (node + ".")
    }
    if(path.endsWith("."))path=path.substring(0,path.length-1)
    return path
}

function sleep(ms = 1) {
    return new Promise((rs) => {
        setTimeout(rs, ms)
    })
}

function getSupportedLocales() {
    let result = new Set();
    let letters = 'abcdefghijklmnopqrstuvwxyz';
    const forLetter = (fn) => {
        for (let i = 0; i < letters.length; i++) {
            fn(letters[i])
        }
    }
    let scode = []
    forLetter(l1 => {
        forLetter(l2 => {
            let code = `${l1 + l2}`
            try {
                let option = new Intl.DateTimeFormat(code).resolvedOptions()
                if (option.locale.toString().toLowerCase() === code) scode.push(option.locale);
            } catch (e) {
            }
        })
    })
    for (const c of scode) {
        result.add(c)
        forLetter(l3 => {
            forLetter(l4 => {
                let code = `${c}-${l3 + l4}`
                try {
                    let option = new Intl.DateTimeFormat(code).resolvedOptions()
                    if (option.locale.toString().toLowerCase() === code) result.add(option.locale);
                } catch (e) {
                }
            })
        })
    }
    return result;
}

function isTrial() {
    let key = 'EXPIRATION_DATE'
    if (key.toLowerCase() === 'expiration_date') return;
    try {
        const now = Date.now()
        if (now > new Date(atob(key)).getTime()) deepError()
    } catch (e) {
        if (e.message.startsWith('EXPIRATION')) return;
        deepError()
    }
}

function guid() {
    function S4() {
        return (((1 + Math.random()) * 0x10000) | 0).toString(16).substring(1);
    }

    return (S4() + S4() + "-" + S4() + "-" + S4() + "-" + S4() + "-" + S4() + S4() + S4());
}

function stringToBytes(str) {
    let bytes = [];
    if (!str) return bytes
    let len, c;
    len = str.length;
    for (let i = 0; i < len; i++) {
        c = str.charCodeAt(i);
        if (c >= 0x010000 && c <= 0x10FFFF) {
            bytes.push(((c >> 18) & 0x07) | 0xF0);
            bytes.push(((c >> 12) & 0x3F) | 0x80);
            bytes.push(((c >> 6) & 0x3F) | 0x80);
            bytes.push((c & 0x3F) | 0x80);
        } else if (c >= 0x000800 && c <= 0x00FFFF) {
            bytes.push(((c >> 12) & 0x0F) | 0xE0);
            bytes.push(((c >> 6) & 0x3F) | 0x80);
            bytes.push((c & 0x3F) | 0x80);
        } else if (c >= 0x000080 && c <= 0x0007FF) {
            bytes.push(((c >> 6) & 0x1F) | 0xC0);
            bytes.push((c & 0x3F) | 0x80);
        } else {
            bytes.push(c & 0xFF);
        }
    }
    return bytes;
}

function hashcode(str = '') {
    let hash = 0, i, chr;
    if (str.length === 0) return hash;
    for (i = 0; i < str.length; i++) {
        chr = str.charCodeAt(i);
        hash = ((hash << 5) - hash) + chr;
        hash |= 0; // Convert to 32bit integer
    }
    return hash;
}

/**
 * 导出
 **/
export {
    getLogo, splitExpression,
    randomInt, randomItem, randomNum,
    isPlugin,
    splitPath,
    combinePath,
    isTrial,
    copy,
    guid,
    hashcode,
    getValue,
    isValidPath,
    timestampToTime,
    setStore,
    getStore,
    removeStore,
    setCookie,
    getCookie,
    delCookie
}
