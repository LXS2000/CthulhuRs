function getFmt(local, key) {
    let fmts = local.fmts[key] || (Local.globalFmts || {})[key];
    if (!fmts) return "<unkonw>";
    let fmt = fmts[local.curLan];
    if (!fmt) return "<unkonw language>";
    return fmt
}


class Local {
    constructor(fmts = [], firstUpper = false) {
        this.curLan = Local.curLan || navigator.language.split("-")[0];
        // this.curLan = "en"
        this.fmts = fmts;
        this.firstUpper = firstUpper;
    }

    /**
     * 翻译并填充模版中的值
     * @param formatK
     * @param values
     * @returns {*}
     */
    tlf(formatK, ...values) {
        let fmt = getFmt(this, formatK);
        let text = this.tlp(fmt, ...values).toLowerCase()
        if (this.firstUpper) return firstUpper(text)
        return text
    }

    /**
     * 直接翻译，仅填充模版本身的占位
     * @param formatK
     */
    tl(formatK,) {
        let fmt = getFmt(this, formatK);
        let text = this.tlp(fmt,)
        if (this.firstUpper) return firstUpper(text)
        return text
    }

    /**
     * 翻译 模板填充 值填充
     * user:{zh:"用户",en:"User"}
     * tlp('{user} {}',111) => 'User 111'
     * @param pat 模板
     * @param values
     */
    tlp(pat, ...values) {
        const regex = /\{([^}]+)?}/g;
        // 使用正则表达式匹配 {} 占位符
        let text = pat.replace(regex, (match, placeholder) => {
            // 如果括号中有内容，则使用表中对应的值
            if (placeholder) {
                let fmt = getFmt(this, placeholder)
                let text = this.tlp(fmt,).toLowerCase();
                if (["en"].includes(this.curLan)) return text + " "
                else return text
            }
            // 否则按顺序使用 values 中的值
            if (["en"].includes(this.curLan)) return values.shift() + " "
            else return values.shift()
        }).trim();
        if (this.firstUpper) return firstUpper(text)
        return text
    }

    setLan(lan) {
        this.curLan = lan
    }
}

Local.globalFmts = {
    k: {
        zh: "键",
        en: "key",
    },
    v: {
        zh: "值",
        en: "value",
    },
    user: {
        zh: "用户",
        en: "user",
    },
    login: {
        zh: "登录",
        en: "login",
    },
    register: {
        zh: "注册",
        en: "register",
    },
    reset: {
        zh: "重置",
        en: "reset",
    },
    send: {
        zh: "发送",
        en: "send",
    },
    del: {
        zh: "删除",
        en: "delete",
    },
    email: {
        zh: "邮箱",
        en: "email",
    },
    pwd: {
        zh: "密码",
        en: "password",
    },
    vcode: {
        zh: "验证码",
        en: "verification code",
    },
    plugin: {
        zh: "插件",
        en: "plugin",
    },
    logs: {
        zh: "日志",
        en: "logs",
    },
    stores: {
        zh: "存储",
        en: "stores",
    },
    func: {
        zh: "功能",
        en: "function",
    },
    warning: {
        zh: "警告",
        en: "warning",
    },
    sure: {
        zh: "确定",
        en: "sure",
    }, cancel: {
        zh: "取消",
        en: "cancel",
    },
    plsInput: {
        zh: "请输入",
        en: "please enter your",
    },
    notEmpty: {
        zh: "不能为空",
        en: "cannot be empty",
    },
    fmtErr: {
        zh: "格式错误",
        en: "format error",
    },
    reload: {
        zh: "重新加载",
        en: "reload"
    }, back: {
        zh: "返回",
        en: "back"
    },
}
Local.curLan = navigator.language.split("-")[0];
const global = new Local([], true)
export const tl = global.tl.bind(global)
export const tlp = global.tlp.bind(global)
export const tlf = global.tlf.bind(global)
export const setLan = global.setLan.bind(global)

function format(fmt, ...values) {
    // 使用正则表达式匹配 {} 占位符
    return fmt.replace(/\{}/g, () => {
        // 从 values 中取出下一个值
        const nextValue = values.shift();
        // 将值转换为字符串并返回
        return String(nextValue);
    });
}


function firstUpper(text) {
    if (text) {
        let c = text.at(0).toUpperCase();
        return c + text.substring(1)
    }
    return text
}

export default Local
