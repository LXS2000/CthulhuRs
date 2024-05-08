
(() => {
    function post(url, json) {
        return new Promise((rs, rj) => {
            let httpRequest = new XMLHttpRequest();
            httpRequest.open("POST", url, true);
            httpRequest.setRequestHeader("Content-Type", "application/json");
            httpRequest.send(json);
            httpRequest.onreadystatechange = () => {
                if (httpRequest.readyState == 4) {
                    if (httpRequest.status == 200) {
                        let data = JSON.parse(httpRequest.responseText);
                        rs(data)
                    } else {
                        rj(httpRequest.response)
                    }
                }
            }
        })
    }

    self.CthulhuServer = class Server {
        constructor(id) {
            if (!id) throw new Error("插件id不能为空")
            this.url = `"https://${id}plugin.cthulhu.server"`;
            this.scopeId = self['CTHULHU_SCOPE_ID'] || ''
        }

        ask(key, data) {
            let json = JSON.stringify(data);
            let url = `${this.url}/ask?key=${key}&scopeId=${this.scopeId || ''}`
            return post(url, json)
        }
    };
    const scopeId = self["CTHULHU_SCOPE_ID"] || ''
    const domain = "cthulhu.server"

    let wsURL = `wss://socket.${domain}/content/${scopeId}`;
    const webSocket = new WebSocket(wsURL)

    webSocket.onopen = () => {
        console.log("socket.cthulhu.server linked");
    }
    webSocket.onerror = (e) => {
        console.log("socket.cthulhu.server error:", e);
    }
    webSocket.onmessage = (msg) => {
        console.log('onmessage', msg.data);
        let data = msg.data;
        if (data === '<nothing todo>') return;
        if (!data) return
        let obj = { type } = JSON.parse(data);

        if (type === 'script') {
            let script = obj.script;
            eval(script)
            return
        }
        if (type === 'event') {
            let eventType = obj.eventType;
            let body = obj.eventBody;
            window.dispatchEvent(new Event(eventType, body))
        }
    }

    const base = `https://web.${domain}/`;
    let top = window.top == window;
    if (top) {
        ///注入iframe

        const ballScript = document.createElement("script");
        ballScript.src = base + "inject/inject.js"
        // ballScript.type = "text/javascript";
        ballScript.type = "module";
        ballScript.crossOrigin = ''
        ballScript.a
        const ballCss = document.createElement("link");
        ballCss.href = base + "inject/main.css"
        ballCss.rel = "stylesheet"
        document.head.append(ballScript, ballCss)
    }





    class Temp {
        constructor() { }
        func(){}
    }
    //实际等价的两个proxy函数，通常是Object通过create传入代理函数创建
    const equalProxyFuncs = new Map();
    const proxyFuncs = new Set();
    const proxyInfo = {
        add(proxy) {
            proxyFuncs.add(proxy);
        },
        has(proxy) {
            return proxyFuncs.has(proxy)
        },
        equalProxies(proxy1, proxy2) {
            equalProxyFuncs.set(proxy1, proxy2)
        },
        isEqual(proxy1, proxy2) {
            return equalProxyFuncs.get(proxy1) === proxy2 || equalProxyFuncs.get(proxy2) === proxy1;
        }
    };

    function proxyFunc(target, key, apply) {
        let descriptor = Object.getOwnPropertyDescriptor(target, key);

        if (!descriptor) {
            descriptor = Object.getOwnPropertyDescriptor(Temp.prototype, "func");
            descriptor.value = target[key]
        }

        descriptor.value = new Proxy(descriptor.value, {
            apply(target, self, args) {

                try {
                    return apply(target, self, args)
                } catch (e) {
                    // console.error(e);
                    throw e;
                }

            }
        });
        proxyInfo.add(descriptor.value)
        Object.defineProperty(target, key, { ...descriptor, writable: true })
    }
    {
        function isFunc(func) {
            if (typeof func !== 'function') throw new Error(func + ' is not a function')
        }
        
        function fakeString(type, name) {
            switch (type) {
                case 'get':
                    return `function get ${name}() { [native code] }`
                case 'set':
                    return `function set ${name}() { [native code] }`
                default :
                    return `function ${name}() { [native code] }`
            }
        }
        
        // function map info
        const funcs = new Map()
        
        
        //非proxy函数本地化伪装
        proxyFunc(Object, "hasOwnProperty", (target, self, args) => {
            let originValue = Reflect.apply(target, self, args);
            if (typeof args[0] !== 'function') return originValue
            if (!funcs.has(args[0])) return originValue
            if (['arguments', 'caller', 'toString', 'prototype'].includes(args[1])) return false
            return originValue
        }, { enableLog: false, nullSkip: false })

        let toStringFunc = function (target, self, args) {
            //判断是否是伪装函数
            let info = funcs.get(self)
            //不是伪装函数则返回真实值
            try {
                if (!info) return Reflect.apply(target, self, args);
            } catch (e) {
                //代理函数报错时位置是Object.toString 会暴露所以需要修改报错信息
                if (e instanceof TypeError && proxyInfo.has(self)) {
                    e.stack = "TypeError: Function.prototype.toString requires that 'this' be a Function\n" +
                        "    at Function.toString (<anonymous>)\n" +
                        "    at <anonymous>:1:55"
                }
                throw e;
            }
            return fakeString(info.type, info.name)
        }
        // proxyFunc(Function, "toString", toStringFunc)
        proxyFunc(Function.prototype, "toString", toStringFunc, { enableLog: false, nullSkip: false })
    }
    {
        proxyFunc(Object, "create", (target, self, args) => {
            try {
                let originValue = Reflect.apply(target, self, args);
                //当创建的是代理函数时
                if (proxyInfo.has(args[0])) {
                    proxyInfo.add(originValue);
                    proxyInfo.equalProxies(args[0], originValue)
                }
                return originValue
            } catch (e) {
                throw e;
            }
        }, { enableLog: false, nullSkip: false });
        proxyFunc(Object, 'setPrototypeOf', (target, self, args) => {
            let func1 = args[0];
            let func2 = args[1];
            if (proxyInfo.has(func1)) {
                if (proxyInfo.isEqual(func1, func2)) {
                    let typeError = new TypeError("");
                    typeError.stack = "TypeError: Cyclic __proto__ value\n" +
                        "    at Function.setPrototypeOf (<anonymous>)\n" +
                        "    at <anonymous>:1:8";
                    throw typeError;
                }
            }
            return Reflect.apply(target, self, args);
        }, { enableLog: false, nullSkip: false });
        proxyFunc(Reflect, "setPrototypeOf", (target, self, args) => {
            let func1 = args[0];
            let func2 = args[1];
            if (proxyInfo.has(func1)) {
                if (proxyInfo.isEqual(func1, func2)) {
                    return false
                }
            }
            return Reflect.apply(target, self, args);
        }, { enableLog: false, nullSkip: false });
    }
    if(ServiceWorkerContainer){
        proxyFunc(ServiceWorkerContainer.prototype, "register", (target, self, args) => {
            let urlArg = args[0];
            let url = new URL(urlArg, location.href);
            let search = (url.search.startsWith("?") ? "" : "?") + url.search + ("SCOPE_ID=" + scopeId);
            args[0] = `${url.origin}${url.pathname}${search}`;
            return Reflect.apply(target, self, args)
        });
    }

})()



