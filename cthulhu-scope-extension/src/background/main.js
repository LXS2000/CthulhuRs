//接收消息并处理


(async () => {

    const extension = self.chrome || self.browser
    const {email: EMAIL, id: _userId} = await extension.identity.getProfileUserInfo()
    const HEADER = "Cthulhu-Extra-Scope"
    const INFO = (await extension.storage.local.get(["email", "window", "tab", "frame", "custom"]) || {})

    function getNewHeader(windowId, tabId, frameId) {
        let infos = [];
        if (INFO.email) infos.push(`email=${EMAIL}`)
        if (INFO.window) infos.push(`window=${windowId}`)
        if (INFO.tab) infos.push(`tab=${tabId}`)
        if (INFO.frame) infos.push(`frame=${frameId}`)
        if (INFO.custom) infos.push(`custom=${INFO.custom}`)
        let info = infos.join(";");
        if (!info) return null
        return {
            header: HEADER,
            operation: "set",
            value: info
        }
    }

    extension.storage.onChanged.addListener(async (changes, _namespace) => {
        for (let [key, {_oldValue, newValue}] of Object.entries(changes)) {
            INFO[key] = newValue
        }
        let win = await extension.windows.getCurrent();
        let rules = await extension.declarativeNetRequest.getSessionRules() || [];
        let ids = []
        let newRules = []
        rules.forEach(rule => {
            ids.push(rule.id);
            let newHeader = getNewHeader(win.id,rule.id, 0);
            if (newHeader) {
                rule.action.requestHeaders = [newHeader];
                newRules.push(rule)
            }
        })
        await extension.declarativeNetRequest.updateSessionRules({
            removeRuleIds: ids,
            addRules: newRules,
        }, () => {
            console.log('值监听' + ": 请求头动态修改规则 添加成功", newRules)
        })
    });
    if (process.env.VUE_APP_MANIFEST_VERSION === 'v2') {
        extension.webRequest.onBeforeSendHeaders.addListener(
            async function (details) {
                let win = await extension.windows.getCurrent();
                let header = getNewHeader(win.id,details.tabId, details.frameId)
                if (header != null) {
                    details.requestHeaders.push({name: HEADER, value: header.value})
                }
                return {requestHeaders: details.requestHeaders};
            },
            {urls: ['<all_urls>']},
            ['blocking', 'requestHeaders']
        );
    } else {
        extension.webNavigation.onBeforeNavigate.addListener((async details => {
            if (!details.url.toString().startsWith('http')) return
            let win = await extension.windows.getCurrent();
            let header = getNewHeader(win.id,details.tabId, details.frameId)
            if (header != null) {
                await setHeaderRules(details.tabId, [header])
            }
        }));

    }

    async function setHeaderRules(tabId, headerRules = []) {
        if (!headerRules || headerRules.length === 0) {
            return await extension.declarativeNetRequest.updateSessionRules({
                removeRuleIds: [tabId],
                addRules: [],
            }, () => {
                console.log(tabId + ": 请求头动态修改规则 清空成功")
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
})()

