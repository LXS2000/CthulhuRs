
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

    const domain = "cthulhu.server"

    let scopeId = self["CTHULHU_SCOPE_ID"] || ''

    let workerName = self.constructor.name;
    let type = "";
    switch (workerName) {
        case "DedicatedWorkerGlobalScope":
            type = "worker"
            break;
        case "ServiceWorkerGlobalScope":
            type = "serviceworker"
            break;
        case "SharedWorkerGlobalScope":
            type = "sharedworker"
            break;
        default:
            break;
    }
    let wsURL = `wss://socket.${domain}/${type}/${scopeId}`;
    const webSocket = new WebSocket(wsURL)

    webSocket.onopen = () => {
        console.log("socket.cthulhu.server linked");
    }

    webSocket.onmessage = (msg) => {
        let data = msg.data;
        if (data === '<nothing todo>') return;
        if (!data) return
        let obj = { type } = JSON.parse(data);
        console.log('onmessage', obj);
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
})();



